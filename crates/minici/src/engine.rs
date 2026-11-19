//! Universal Combinatorial CI/CD DAG, Matrix & Cost Engine (`crates/minici/src/engine.rs`).
//!
//! Pure Cost Domain Engine (`Deterministic CI/CD FinOps` — Zero External Crate Dependencies):
//! 1. Parses any repository's CI/CD workflows across GitHub Actions (`.github/workflows/*.yml`),
//!    GitLab CI (`.gitlab-ci.yml`), CircleCI (`.circleci/config.yml`), Bitbucket (`bitbucket-pipelines.yml`),
//!    and AWS CodeBuild (`buildspec.yml`).
//! 2. Expands Cartesian `strategy.matrix` (`OS × Runtime × Version`), `include` additions, `exclude` filters,
//!    and GitLab `parallel: matrix:` combinations in microseconds.
//! 3. Applies Official Vendor Billing Rules (including GitHub Actions' 1-minute ceiling rounding per job!).
//! 4. Calculates BOTH:
//!    - **Free-Tier User Metrics:** Free Quota Burn % (`2,000` GH / `400` GL / `3,000` CircleCI) + Developer Wait-Time Engineering Cost (`$` at `$65/hr` default).
//!    - **Paid-Tier / FinOps Metrics:** Billed Runner Compute Cost (`$`), Hidden Artifact Storage Markup (`$0.25/GB` vs `$0.023` S3),
//!      and the `5,000 min/mo` Self-Hosted / Blacksmith Break-Even analysis.

use crate::catalog::{
    infracost_aws_nat_gateway_rate_usd, infracost_ec2_runner_profile, resolve_runner_sku,
    stack_cache_savings_profile, vendor_cache_mechanics, vendor_os_free_tier_multiplier,
    ARTIFACT_STORAGE_S3_INTELLIGENT_USD_PER_GB_MO, ARTIFACT_STORAGE_S3_USD_PER_GB_MO,
    ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO, AWS_CROSS_AZ_USD_PER_GB, AWS_CROSS_REGION_USD_PER_GB,
    AWS_INTERNET_EGRESS_USD_PER_GB, AWS_VPC_ENDPOINT_USD_PER_GB,
    SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH, SELF_HOSTED_C5_4XLARGE_SPOT_BREAKEVEN_MINS,
    SELF_HOSTED_VPS_BASE_MONTHLY_USD,
};
use crate::providers::{
    compute_5layer_platform_switching_tco, is_github_larger_runner_job, select_provider_plugin,
    RepoVisibility,
};
use crate::telemetry::CanonicalTelemetrySnapshot;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct UsageConfig {
    pub tier: String, // "auto", "free", "paid"
    pub prs_per_week: f64,
    pub pushes_per_pr: f64,
    pub dev_hourly_rate_usd: f64,
    pub artifact_gb_per_run: f64,
    pub artifact_retention_days: u32,
}

impl Default for UsageConfig {
    fn default() -> Self {
        Self {
            tier: "auto".to_string(),
            prs_per_week: 25.0,
            pushes_per_pr: 4.0,
            dev_hourly_rate_usd: 65.0,
            artifact_gb_per_run: 0.25,
            artifact_retention_days: 30,
        }
    }
}

impl UsageConfig {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"tier\":\"{}\",\"prs_per_week\":{:.1},\"pushes_per_pr\":{:.1},\"dev_hourly_rate_usd\":{:.2},\"artifact_gb_per_run\":{:.2},\"artifact_retention_days\":{}}}",
            json_escape(&self.tier),
            self.prs_per_week,
            self.pushes_per_pr,
            self.dev_hourly_rate_usd,
            self.artifact_gb_per_run,
            self.artifact_retention_days
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExpandedJobCost {
    pub workflow_file: String,
    pub job_id: String,
    pub matrix_combination_count: usize,
    pub runner_label: String,
    pub provider: String,
    pub os: String,
    pub vcpus: u32,
    pub est_raw_duration_mins_per_job: f64,
    pub billed_mins_per_job: f64,
    pub total_billed_mins_all_matrix: f64,
    pub rate_per_min_usd: f64,
    pub run_compute_cost_usd: f64,
    pub blacksmith_run_cost_usd: f64,
    pub self_hosted_spot_run_cost_usd: f64,
    pub has_concurrency_cancel: bool,
    pub has_sparse_or_shallow_clone: bool,
    pub uses_cache: bool,
    pub uploads_artifacts: bool,
    pub artifact_retention_days: Option<u32>,
}

impl ExpandedJobCost {
    pub fn to_json(&self) -> String {
        let ret_json = self
            .artifact_retention_days
            .map(|d| d.to_string())
            .unwrap_or_else(|| "null".to_string());
        format!(
            "{{\"workflow_file\":\"{}\",\"job_id\":\"{}\",\"matrix_combination_count\":{},\"runner_label\":\"{}\",\"provider\":\"{}\",\"os\":\"{}\",\"vcpus\":{},\"est_raw_duration_mins_per_job\":{:.2},\"billed_mins_per_job\":{:.2},\"total_billed_mins_all_matrix\":{:.2},\"rate_per_min_usd\":{:.4},\"run_compute_cost_usd\":{:.4},\"blacksmith_run_cost_usd\":{:.4},\"self_hosted_spot_run_cost_usd\":{:.4},\"has_concurrency_cancel\":{},\"has_sparse_or_shallow_clone\":{},\"uses_cache\":{},\"uploads_artifacts\":{},\"artifact_retention_days\":{}}}",
            json_escape(&self.workflow_file),
            json_escape(&self.job_id),
            self.matrix_combination_count,
            json_escape(&self.runner_label),
            json_escape(&self.provider),
            json_escape(&self.os),
            self.vcpus,
            self.est_raw_duration_mins_per_job,
            self.billed_mins_per_job,
            self.total_billed_mins_all_matrix,
            self.rate_per_min_usd,
            self.run_compute_cost_usd,
            self.blacksmith_run_cost_usd,
            self.self_hosted_spot_run_cost_usd,
            self.has_concurrency_cancel,
            self.has_sparse_or_shallow_clone,
            self.uses_cache,
            self.uploads_artifacts,
            ret_json
        )
    }
}

#[derive(Debug, Clone)]
pub struct CostComponentsBreakdown {
    // 1. Build Minutes Cost (published cloud & vendor rate cards)
    pub rounding_tax_mins_per_run: f64,
    pub monthly_rounding_tax_usd: f64,
    pub linux_mins_per_run: f64,
    pub windows_mins_per_run: f64,
    pub macos_mins_per_run: f64,
    pub os_weighted_free_tier_mins_per_run: f64,
    pub monthly_os_weighted_free_tier_mins: f64,
    pub monthly_flake_retry_10pct_overhead_usd: f64,
    // 2. CI Runner Cost — Cloud Compute Catalog (published cloud & vendor rate cards)
    pub infracost_iac_files_detected: bool,
    pub hosted_effective_hourly_rate_usd: f64,
    pub infracost_recommended_ec2_instance: String,
    pub infracost_ec2_on_demand_hourly_usd: f64,
    pub infracost_ec2_on_demand_monthly_usd: f64,
    pub infracost_ec2_spot_hourly_usd: f64,
    pub infracost_ec2_surrounding_overhead_monthly_usd: f64,
    pub macos_self_hosted_mac_mini_monthly_usd: f64,
    pub self_hosted_breakeven_mins_t3_medium: f64,
    pub self_hosted_breakeven_mins_c5_4xlarge_spot: f64,
    pub always_on_runner_utilization_pct: f64,
    pub ephemeral_runner_recommended_mode: String,
    pub ephemeral_cold_start_wait_tax_monthly_usd: f64,
    pub k8s_eks_control_plane_monthly_usd: f64,
    pub k8s_gke_aks_free_tier_control_plane_monthly_usd: f64,
    pub spot_interruption_band_pct: String,
    pub spot_retry_overhead_monthly_usd: f64,
    pub spot_ineligible_jobs_count: usize,
    pub hybrid_spot_plus_on_demand_monthly_usd: f64,
    // 3. Artifact Storage Cost — Cloud Catalog S3/ECR Delegated (published cloud & vendor rate cards)
    pub jobs_uploading_artifacts: usize,
    pub default_retention_days: u32,
    pub configured_retention_days: u32,
    pub steady_state_stored_gb_configured: f64,
    pub monthly_vendor_storage_configured_usd: f64,
    pub monthly_infracost_s3_configured_usd: f64,
    pub steady_state_stored_gb_90d: f64,
    pub steady_state_stored_gb_14d: f64,
    pub monthly_vendor_storage_90d_usd: f64,
    pub monthly_vendor_storage_14d_usd: f64,
    pub monthly_infracost_s3_standard_90d_usd: f64,
    pub monthly_infracost_s3_intelligent_14d_usd: f64,
    // 4. Parallelism & Matrix Cost Strategy
    pub peak_concurrent_jobs_per_run: usize,
    pub hosted_concurrent_jobs_per_run: usize,
    pub macos_concurrent_jobs_per_run: usize,
    pub self_hosted_or_microvm_jobs_per_run: usize,
    pub vendor_free_concurrent_cap: usize,
    pub vendor_team_concurrent_cap: usize,
    pub parallelism_tier_upgrade_tax_monthly_usd: f64,
    pub detected_repo_matrix_profile: String,
    pub matrix_jobs_count: usize,
    pub pr_unconditional_matrix_cells: usize,
    pub conditional_pr_matrix_monthly_savings_usd: f64,
    // 5. Log Retention Cost — Cloud Catalog CloudWatch/S3 Delegated (published cloud & vendor rate cards)
    pub detected_log_language_profile: String,
    pub est_log_mb_per_run: f64,
    pub monthly_log_volume_gb: f64,
    pub vendor_default_log_retention_days: u32,
    pub monthly_vendor_default_log_cost_usd: f64,
    pub monthly_datadog_30d_log_shipping_usd: f64,
    pub monthly_datadog_90d_log_shipping_usd: f64,
    pub monthly_splunk_30d_log_shipping_usd: f64,
    pub monthly_infracost_loki_s3_usd: f64,
    // 6. Cross-Region Egress Cost
    pub infracost_cloud_region: String,
    pub est_transfer_gb_per_run: f64,
    pub monthly_transfer_gb: f64,
    pub infracost_aws_internet_egress_per_gb_usd: f64,
    pub infracost_aws_nat_gateway_per_gb_usd: f64,
    pub infracost_aws_cross_az_per_gb_usd: f64,
    pub infracost_aws_cross_region_per_gb_usd: f64,
    pub monthly_infracost_private_subnet_nat_worst_case_usd: f64,
    pub monthly_infracost_cross_region_registry_usd: f64,
    pub monthly_infracost_vpc_endpoint_cache_optimized_usd: f64,
    // 7. Developer Wait Time Cost (Payroll & Context-Switch Loss)
    pub productivity_loss_factor: f64,
    pub blended_fully_loaded_hourly_usd: f64,
    pub monthly_wait_hours: f64,
    pub monthly_productive_payroll_loss_usd: f64,
    pub monthly_blended_100hr_productive_loss_usd: f64,
    pub wait_to_compute_cost_ratio: f64,
    pub lead_time_feedback_tier: String,
    // 8. Failed Build Rerun Cost (Flake Rerun Compute + Investigation Payroll)
    pub estimated_flake_rate_pct: f64,
    pub has_job_level_auto_retry: bool,
    pub monthly_flaky_reruns: f64,
    pub monthly_rerun_compute_cost_usd: f64,
    pub avg_investigation_mins_per_flake: f64,
    pub monthly_flake_investigation_hours: f64,
    pub monthly_flake_investigation_payroll_usd: f64,
    pub monthly_total_immediate_flake_cost_usd: f64,
    pub monthly_savings_at_1pct_target_usd: f64,
    // 9. Build Cache Savings (Per-Stack Realistic Savings & Cache Hit-Rate KPI)
    pub stack_cache_profile: String,
    pub realistic_saving_pct_min: f64,
    pub realistic_saving_pct_max: f64,
    pub jobs_with_cache: usize,
    pub jobs_missing_cache: usize,
    pub has_restore_keys_fallback: bool,
    pub has_docker_buildkit_cache: bool,
    pub estimated_cache_hit_rate_pct: f64,
    pub vendor_cache_size_limit: String,
    pub vendor_cache_eviction_policy: String,
    pub monthly_uncached_compute_usd: f64,
    pub monthly_unrealized_cache_savings_usd: f64,
    pub next_recommended_caching_layer: String,
    pub distributed_remote_cache_monthly_infra_usd: f64,
    pub container_base_image_mirror_monthly_savings_usd: f64,
    pub pr_cache_poisoning_risk_jobs: usize,
    // 10. Flaky Test CI/CD Tax (Immediate + Deferred Velocity Drag & Deflake ROI)
    pub deferred_velocity_drag_multiplier: f64,
    pub monthly_deferred_velocity_drag_usd: f64,
    pub monthly_combined_flake_tax_usd: f64,
    pub deflake_project_est_cost_usd: f64,
    pub deflake_payback_weeks: f64,
    // 11. Branch Protection Cost (Required-Check Multiplication, Concurrency Cancel & Path Filters)
    pub required_checks_count: usize,
    pub monthly_required_check_executions: usize,
    pub workflows_with_concurrency_cancel: usize,
    pub workflows_missing_concurrency_cancel: usize,
    pub workflows_with_path_filters: usize,
    pub workflows_missing_path_filters: usize,
    pub has_affected_only_graph_tool: bool,
    pub slow_blocking_checks_over_10m: usize,
    pub monthly_superseded_run_waste_usd: f64,
    pub monthly_unfiltered_check_savings_usd: f64,
    // 12. Startup Stage Cost (1-15 Devs, Free-Tier Burn, Upgrade Triggers & Early Traps)
    pub inferred_team_stage: String,
    pub implied_active_devs: usize,
    pub free_tier_burn_pct: f64,
    pub upgrade_trigger_80pct_reached: bool,
    pub premature_self_hosting_trap: bool,
    pub premature_multi_os_pr_matrix_trap: bool,
    pub monthly_multi_os_pr_to_nightly_savings_usd: f64,
    pub mac_mini_hardware_payback_months: f64,
    // 13. Scaleup Stage Cost (25-100 Devs, Tier-Crisis Decisions, Required-Check Audit & Platform Eng ROI)
    pub self_hosted_breakeven_reached: bool,
    pub team_to_enterprise_seat_jump_monthly_usd: f64,
    pub quarterly_required_check_audit_savings_usd: f64,
    pub platform_engineer_monthly_fully_loaded_usd: f64,
    pub recoverable_ci_and_wait_monthly_usd: f64,
    pub platform_engineer_roi_coverage_pct: f64,
    // 14. Enterprise Stage Cost (250+ Devs, Compliance Overhead, Regional Residency & Dedicated Tenancy)
    pub detected_compliance_controls: Vec<String>,
    pub compliance_controls_count: usize,
    pub detected_cloud_regions: Vec<String>,
    pub regional_residency_multiplier: f64,
    pub enterprise_250_seat_licence_monthly_usd: f64,
    pub siem_1yr_audit_log_retention_monthly_usd: f64,
    pub dedicated_tenancy_premium_pct_min: f64,
    pub dedicated_tenancy_premium_pct_max: f64,
    // 15. Monorepo CI Cost (Dynamic Workspace Detection, Unconstrained Blast Radius & Affected-Only Fix)
    pub detected_pipeline_shape: String,
    pub detected_workspace_units_count: usize,
    pub detected_monorepo_tool: String,
    pub recommended_affected_tool: String,
    pub has_nightly_full_suite_safety_net: bool,
    pub affected_only_reduction_pct_min: f64,
    pub affected_only_reduction_pct_max: f64,
    pub monthly_affected_only_savings_usd: f64,
    // 16. Microservices CI Cost (Per-Service Setup Overhead, Reusable Templates, Pinning & Shared Cache)
    pub est_setup_overhead_sec_per_job: f64,
    pub monthly_setup_overhead_hours: f64,
    pub setup_overhead_share_pct: f64,
    pub monthly_setup_overhead_compute_usd: f64,
    pub monthly_shared_infra_setup_savings_usd: f64,
    pub reusable_workflow_calls_count: usize,
    pub unpinned_template_refs_count: usize,
    pub cross_repo_triggers_count: usize,
    pub est_aggregate_cache_gb: f64,
    pub exceeds_10gb_repo_cache_limit: bool,
    // 17. Multi-Vendor TCO & Migration Comparison Engine (Seats + Included Quota + Overage + Migration Payback)
    pub cheapest_vendor_platform: String,
    pub cheapest_vendor_monthly_tco_usd: f64,
    pub github_actions_team_monthly_tco_usd: f64,
    pub gitlab_ci_premium_monthly_tco_usd: f64,
    pub circleci_performance_monthly_tco_usd: f64,
    pub bitbucket_premium_monthly_tco_usd: f64,
    pub buildkite_pro_monthly_tco_usd: f64,
    pub jenkins_self_hosted_monthly_tco_usd: f64,
    pub aws_codebuild_monthly_tco_usd: f64,
    pub gcp_cloud_build_monthly_tco_usd: f64,
    pub azure_pipelines_monthly_tco_usd: f64,
    pub migration_estimated_hours: f64,
    pub migration_estimated_cost_usd: f64,
    pub migration_payback_months: f64,
    pub migration_worth_it_2x_annual_rule: bool,
    // 18. 6-Phase Build-Minute Time-Share & Best-Payback Optimization Ranker
    pub phase_clone_checkout_share_pct: f64,
    pub phase_dependency_install_share_pct: f64,
    pub phase_compile_typecheck_share_pct: f64,
    pub phase_test_suite_share_pct: f64,
    pub phase_docker_build_push_share_pct: f64,
    pub phase_deploy_cleanup_share_pct: f64,
    pub monthly_total_reducible_phase_savings_usd: f64,
    pub best_first_optimization_phase: String,
    pub best_first_optimization_effort_hours: f64,
    pub best_first_optimization_monthly_savings_usd: f64,
    pub best_first_optimization_payback_months: f64,
    // 19. Compute-Class Right-Sizing Explorer (Amdahl Speedup vs Larger-Runner Billing & Dev-Wait Net ROI)
    pub detected_workload_compute_profile: String,
    pub larger_runners_detected_count: usize,
    pub larger_runners_bypass_free_minutes_tax_usd: f64,
    pub arm64_2core_monthly_compute_usd: f64,
    pub arm64_migration_monthly_savings_usd: f64,
    pub standard_2core_monthly_compute_usd: f64,
    pub larger_4core_monthly_compute_usd: f64,
    pub larger_8core_monthly_compute_usd: f64,
    pub larger_16core_monthly_compute_usd: f64,
    pub best_net_roi_runner_tier: String,
    pub best_net_roi_monthly_delta_usd: f64,
    // 20. Language-Stack Deep-Dive Intelligence (iOS / Android / JVM / Rust / Node / Python / Go / IaC)
    pub detected_stack_archetype: String,
    pub ios_codemagic_annual_m2_monthly_usd: f64,
    pub ios_buildkite_mac_mini_fleet_monthly_tco_usd: f64,
    pub android_emulator_jobs_count: usize,
    pub android_kvm_or_gradle_cache_savings_usd: f64,
    pub jvm_warm_cache_50pct_savings_usd: f64,
    pub stack_specific_monthly_savings_usd: f64,
    pub stack_specific_primary_lever: String,
    // 21. Unit Economics & Team Chargeback (`$/PR`, `$/Deploy`, `$/Active Dev`, Cadence-Weighted Spend & CODEOWNERS)
    pub cost_per_merged_pr_usd: f64,
    pub cost_per_deploy_usd: f64,
    pub cost_per_active_dev_monthly_usd: f64,
    pub cadence_weighted_monthly_compute_usd: f64,
    pub top_workflow_cost_share_pct: f64,
    pub codeowners_detected: bool,
    // 22. Spend Anomaly, Quota Forecast & Hung-Job Exposure (`7d/30d` Velocity, EOM Forecast & Missing `timeout-minutes`)
    pub burn_velocity_7d_vs_30d_ratio: f64,
    pub spend_anomaly_alert_active: bool,
    pub projected_eom_compute_usd: f64,
    pub projected_quota_exhaustion_day: u32,
    pub jobs_missing_timeout_minutes_count: usize,
    pub hung_job_360m_worst_case_exposure_usd: f64,
    // 23. Container Registry & Package Storage Economics (`GHCR / ECR / GCR` Retention & Untagged Layer Cleanup)
    pub container_images_built_count: usize,
    pub est_registry_storage_gb: f64,
    pub monthly_registry_storage_and_pull_usd: f64,
    pub untagged_image_cleanup_monthly_savings_usd: f64,
    // 24. Self-Hosted Runner Idle Waste & Queue SLA (`Off-Hours` Idle Capacity & `P95` Queue Wait Breach Risk)
    pub off_hours_idle_capacity_pct: f64,
    pub monthly_always_on_idle_waste_usd: f64,
    pub est_p95_queue_wait_sec: f64,
    pub sla_queue_breach_risk: bool,
}

impl CostComponentsBreakdown {
    pub fn to_json(&self) -> String {
        let compliance_json = self
            .detected_compliance_controls
            .iter()
            .map(|c| format!("\"{}\"", json_escape(c)))
            .collect::<Vec<_>>()
            .join(",");
        let regions_json = self
            .detected_cloud_regions
            .iter()
            .map(|r| format!("\"{}\"", json_escape(r)))
            .collect::<Vec<_>>()
            .join(",");
        let models_21_to_24_json = format!(
            "\"unit_economics_and_chargeback\": {{\"cost_per_merged_pr_usd\":{:.2},\"cost_per_deploy_usd\":{:.2},\"cost_per_active_dev_monthly_usd\":{:.2},\"cadence_weighted_monthly_compute_usd\":{:.2},\"top_workflow_cost_share_pct\":{:.1},\"codeowners_detected\":{}}},\n    \"spend_anomaly_and_forecast\": {{\"burn_velocity_7d_vs_30d_ratio\":{:.2},\"spend_anomaly_alert_active\":{},\"projected_eom_compute_usd\":{:.2},\"projected_quota_exhaustion_day\":{},\"jobs_missing_timeout_minutes_count\":{},\"hung_job_360m_worst_case_exposure_usd\":{:.2}}},\n    \"container_registry_economics\": {{\"container_images_built_count\":{},\"est_registry_storage_gb\":{:.2},\"monthly_registry_storage_and_pull_usd\":{:.2},\"untagged_image_cleanup_monthly_savings_usd\":{:.2}}},\n    \"self_hosted_fleet_idle_and_sla\": {{\"off_hours_idle_capacity_pct\":{:.1},\"monthly_always_on_idle_waste_usd\":{:.2},\"est_p95_queue_wait_sec\":{:.1},\"sla_queue_breach_risk\":{}}}",
            self.cost_per_merged_pr_usd,
            self.cost_per_deploy_usd,
            self.cost_per_active_dev_monthly_usd,
            self.cadence_weighted_monthly_compute_usd,
            self.top_workflow_cost_share_pct,
            self.codeowners_detected,
            self.burn_velocity_7d_vs_30d_ratio,
            self.spend_anomaly_alert_active,
            self.projected_eom_compute_usd,
            self.projected_quota_exhaustion_day,
            self.jobs_missing_timeout_minutes_count,
            self.hung_job_360m_worst_case_exposure_usd,
            self.container_images_built_count,
            self.est_registry_storage_gb,
            self.monthly_registry_storage_and_pull_usd,
            self.untagged_image_cleanup_monthly_savings_usd,
            self.off_hours_idle_capacity_pct,
            self.monthly_always_on_idle_waste_usd,
            self.est_p95_queue_wait_sec,
            self.sla_queue_breach_risk,
        );
        format!(
            "{{\n    \"build_minutes_cost\": {{\"rounding_tax_mins_per_run\":{:.2},\"monthly_rounding_tax_usd\":{:.2},\"linux_mins_per_run\":{:.2},\"windows_mins_per_run\":{:.2},\"macos_mins_per_run\":{:.2},\"os_weighted_free_tier_mins_per_run\":{:.2},\"monthly_os_weighted_free_tier_mins\":{:.2},\"monthly_flake_retry_10pct_overhead_usd\":{:.2}}},\n    \"runner_cost\": {{\"infracost_delegated\":true,\"infracost_iac_files_detected\":{},\"infracost_resources\":[\"aws_instance\",\"aws_ebs_volume\",\"google_compute_instance\",\"azurerm_linux_virtual_machine\"],\"hosted_effective_hourly_rate_usd\":{:.2},\"infracost_recommended_ec2_instance\":\"{}\",\"infracost_ec2_on_demand_hourly_usd\":{:.4},\"infracost_ec2_on_demand_monthly_usd\":{:.2},\"infracost_ec2_spot_hourly_usd\":{:.3},\"infracost_ec2_surrounding_overhead_monthly_usd\":{:.2},\"macos_self_hosted_mac_mini_monthly_usd\":{:.2},\"self_hosted_breakeven_mins_t3_medium\":{:.0},\"self_hosted_breakeven_mins_c5_4xlarge_spot\":{:.0},\"always_on_runner_utilization_pct\":{:.1},\"ephemeral_runner_recommended_mode\":\"{}\",\"ephemeral_cold_start_wait_tax_monthly_usd\":{:.2},\"k8s_eks_control_plane_monthly_usd\":{:.2},\"k8s_gke_aks_free_tier_control_plane_monthly_usd\":{:.2},\"spot_interruption_band_pct\":\"{}\",\"spot_retry_overhead_monthly_usd\":{:.2},\"spot_ineligible_jobs_count\":{},\"hybrid_spot_plus_on_demand_monthly_usd\":{:.2}}},\n    \"artifact_storage_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_s3_bucket\",\"aws_ecr_repository\"],\"jobs_uploading_artifacts\":{},\"default_retention_days\":{},\"configured_retention_days\":{},\"steady_state_stored_gb_configured\":{:.2},\"monthly_vendor_storage_configured_usd\":{:.2},\"monthly_infracost_s3_configured_usd\":{:.2},\"steady_state_stored_gb_90d\":{:.2},\"steady_state_stored_gb_14d\":{:.2},\"monthly_vendor_storage_90d_usd\":{:.2},\"monthly_vendor_storage_14d_usd\":{:.2},\"monthly_infracost_s3_standard_90d_usd\":{:.2},\"monthly_infracost_s3_intelligent_14d_usd\":{:.2}}},\n    \"parallelism_cost\": {{\"peak_concurrent_jobs_per_run\":{},\"hosted_concurrent_jobs_per_run\":{},\"macos_concurrent_jobs_per_run\":{},\"self_hosted_or_microvm_jobs_per_run\":{},\"vendor_free_concurrent_cap\":{},\"vendor_team_concurrent_cap\":{},\"parallelism_tier_upgrade_tax_monthly_usd\":{:.2},\"detected_repo_matrix_profile\":\"{}\",\"matrix_jobs_count\":{},\"pr_unconditional_matrix_cells\":{},\"conditional_pr_matrix_monthly_savings_usd\":{:.2}}},\n    \"log_retention_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_cloudwatch_log_group\",\"aws_s3_bucket\"],\"detected_log_language_profile\":\"{}\",\"est_log_mb_per_run\":{:.2},\"monthly_log_volume_gb\":{:.2},\"vendor_default_log_retention_days\":{},\"monthly_vendor_default_log_cost_usd\":{:.2},\"monthly_datadog_30d_log_shipping_usd\":{:.2},\"monthly_datadog_90d_log_shipping_usd\":{:.2},\"monthly_splunk_30d_log_shipping_usd\":{:.2},\"monthly_infracost_loki_s3_usd\":{:.2}}},\n    \"cross_region_egress_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_nat_gateway\",\"aws_vpc_endpoint\",\"google_compute_router_nat\",\"azurerm_nat_gateway\"],\"infracost_cloud_region\":\"{}\",\"est_transfer_gb_per_run\":{:.2},\"monthly_transfer_gb\":{:.2},\"infracost_aws_internet_egress_per_gb_usd\":{:.3},\"infracost_aws_nat_gateway_per_gb_usd\":{:.3},\"infracost_aws_cross_az_per_gb_usd\":{:.3},\"infracost_aws_cross_region_per_gb_usd\":{:.3},\"monthly_infracost_private_subnet_nat_worst_case_usd\":{:.2},\"monthly_infracost_cross_region_registry_usd\":{:.2},\"monthly_infracost_vpc_endpoint_cache_optimized_usd\":{:.2}}},\n    \"developer_wait_time_cost\": {{\"productivity_loss_factor\":{:.2},\"blended_fully_loaded_hourly_usd\":{:.2},\"monthly_wait_hours\":{:.2},\"monthly_productive_payroll_loss_usd\":{:.2},\"monthly_blended_100hr_productive_loss_usd\":{:.2},\"wait_to_compute_cost_ratio\":{:.2},\"lead_time_feedback_tier\":\"{}\"}},\n    \"failed_build_rerun_cost\": {{\"estimated_flake_rate_pct\":{:.1},\"has_job_level_auto_retry\":{},\"monthly_flaky_reruns\":{:.2},\"monthly_rerun_compute_cost_usd\":{:.2},\"avg_investigation_mins_per_flake\":{:.1},\"monthly_flake_investigation_hours\":{:.2},\"monthly_flake_investigation_payroll_usd\":{:.2},\"monthly_total_immediate_flake_cost_usd\":{:.2},\"monthly_savings_at_1pct_target_usd\":{:.2}}},\n    \"build_cache_savings\": {{\"stack_cache_profile\":\"{}\",\"realistic_saving_pct_min\":{:.1},\"realistic_saving_pct_max\":{:.1},\"jobs_with_cache\":{},\"jobs_missing_cache\":{},\"has_restore_keys_fallback\":{},\"has_docker_buildkit_cache\":{},\"estimated_cache_hit_rate_pct\":{:.1},\"vendor_cache_size_limit\":\"{}\",\"vendor_cache_eviction_policy\":\"{}\",\"monthly_uncached_compute_usd\":{:.2},\"monthly_unrealized_cache_savings_usd\":{:.2},\"next_recommended_caching_layer\":\"{}\",\"distributed_remote_cache_monthly_infra_usd\":{:.2},\"container_base_image_mirror_monthly_savings_usd\":{:.2},\"pr_cache_poisoning_risk_jobs\":{}}},\n    \"flaky_test_ci_cd_tax\": {{\"monthly_immediate_flake_cost_usd\":{:.2},\"deferred_velocity_drag_multiplier\":{:.1},\"monthly_deferred_velocity_drag_usd\":{:.2},\"monthly_combined_flake_tax_usd\":{:.2},\"deflake_project_est_cost_usd\":{:.2},\"deflake_payback_weeks\":{:.2}}},\n    \"branch_protection_cost\": {{\"required_checks_count\":{},\"monthly_required_check_executions\":{},\"workflows_with_concurrency_cancel\":{},\"workflows_missing_concurrency_cancel\":{},\"workflows_with_path_filters\":{},\"workflows_missing_path_filters\":{},\"has_affected_only_graph_tool\":{},\"slow_blocking_checks_over_10m\":{},\"monthly_superseded_run_waste_usd\":{:.2},\"monthly_unfiltered_check_savings_usd\":{:.2}}},\n    \"startup_stage_cost\": {{\"inferred_team_stage\":\"{}\",\"implied_active_devs\":{},\"free_tier_burn_pct\":{:.2},\"upgrade_trigger_80pct_reached\":{},\"premature_self_hosting_trap\":{},\"premature_multi_os_pr_matrix_trap\":{},\"monthly_multi_os_pr_to_nightly_savings_usd\":{:.2},\"mac_mini_hardware_payback_months\":{:.2}}},\n    \"scaleup_stage_cost\": {{\"self_hosted_breakeven_reached\":{},\"team_to_enterprise_seat_jump_monthly_usd\":{:.2},\"quarterly_required_check_audit_savings_usd\":{:.2},\"platform_engineer_monthly_fully_loaded_usd\":{:.2},\"recoverable_ci_and_wait_monthly_usd\":{:.2},\"platform_engineer_roi_coverage_pct\":{:.2}}},\n    \"enterprise_stage_cost\": {{\"detected_compliance_controls\":[{}],\"compliance_controls_count\":{},\"detected_cloud_regions\":[{}],\"regional_residency_multiplier\":{:.2},\"enterprise_250_seat_licence_monthly_usd\":{:.2},\"siem_1yr_audit_log_retention_monthly_usd\":{:.2},\"dedicated_tenancy_premium_pct_min\":{:.1},\"dedicated_tenancy_premium_pct_max\":{:.1}}},\n    \"monorepo_ci_cost\": {{\"detected_pipeline_shape\":\"{}\",\"detected_workspace_units_count\":{},\"detected_monorepo_tool\":\"{}\",\"recommended_affected_tool\":\"{}\",\"has_nightly_full_suite_safety_net\":{},\"affected_only_reduction_pct_min\":{:.1},\"affected_only_reduction_pct_max\":{:.1},\"monthly_affected_only_savings_usd\":{:.2}}},\n    \"microservices_ci_cost\": {{\"est_setup_overhead_sec_per_job\":{:.1},\"monthly_setup_overhead_hours\":{:.2},\"setup_overhead_share_pct\":{:.2},\"monthly_setup_overhead_compute_usd\":{:.2},\"monthly_shared_infra_setup_savings_usd\":{:.2},\"reusable_workflow_calls_count\":{},\"unpinned_template_refs_count\":{},\"cross_repo_triggers_count\":{},\"est_aggregate_cache_gb\":{:.2},\"exceeds_10gb_repo_cache_limit\":{}}},\n    \"vendor_tco_comparison\": {{\"cheapest_vendor_platform\":\"{}\",\"cheapest_vendor_monthly_tco_usd\":{:.2},\"github_actions_team_monthly_tco_usd\":{:.2},\"gitlab_ci_premium_monthly_tco_usd\":{:.2},\"circleci_performance_monthly_tco_usd\":{:.2},\"bitbucket_premium_monthly_tco_usd\":{:.2},\"buildkite_pro_monthly_tco_usd\":{:.2},\"jenkins_self_hosted_monthly_tco_usd\":{:.2},\"aws_codebuild_monthly_tco_usd\":{:.2},\"gcp_cloud_build_monthly_tco_usd\":{:.2},\"azure_pipelines_monthly_tco_usd\":{:.2},\"migration_estimated_hours\":{:.1},\"migration_estimated_cost_usd\":{:.2},\"migration_payback_months\":{:.2},\"migration_worth_it_2x_annual_rule\":{}}},\n    \"build_phase_breakdown\": {{\"phase_clone_checkout_share_pct\":{:.1},\"phase_dependency_install_share_pct\":{:.1},\"phase_compile_typecheck_share_pct\":{:.1},\"phase_test_suite_share_pct\":{:.1},\"phase_docker_build_push_share_pct\":{:.1},\"phase_deploy_cleanup_share_pct\":{:.1},\"monthly_total_reducible_phase_savings_usd\":{:.2},\"best_first_optimization_phase\":\"{}\",\"best_first_optimization_effort_hours\":{:.1},\"best_first_optimization_monthly_savings_usd\":{:.2},\"best_first_optimization_payback_months\":{:.2}}},\n    \"compute_class_explorer\": {{\"detected_workload_compute_profile\":\"{}\",\"larger_runners_detected_count\":{},\"larger_runners_bypass_free_minutes_tax_usd\":{:.2},\"arm64_2core_monthly_compute_usd\":{:.2},\"arm64_migration_monthly_savings_usd\":{:.2},\"standard_2core_monthly_compute_usd\":{:.2},\"larger_4core_monthly_compute_usd\":{:.2},\"larger_8core_monthly_compute_usd\":{:.2},\"larger_16core_monthly_compute_usd\":{:.2},\"best_net_roi_runner_tier\":\"{}\",\"best_net_roi_monthly_delta_usd\":{:.2}}},\n    \"stack_specific_economics\": {{\"detected_stack_archetype\":\"{}\",\"ios_codemagic_annual_m2_monthly_usd\":{:.2},\"ios_buildkite_mac_mini_fleet_monthly_tco_usd\":{:.2},\"android_emulator_jobs_count\":{},\"android_kvm_or_gradle_cache_savings_usd\":{:.2},\"jvm_warm_cache_50pct_savings_usd\":{:.2},\"stack_specific_monthly_savings_usd\":{:.2},\"stack_specific_primary_lever\":\"{}\"}},\n    {}\n  }}",
            self.rounding_tax_mins_per_run,
            self.monthly_rounding_tax_usd,
            self.linux_mins_per_run,
            self.windows_mins_per_run,
            self.macos_mins_per_run,
            self.os_weighted_free_tier_mins_per_run,
            self.monthly_os_weighted_free_tier_mins,
            self.monthly_flake_retry_10pct_overhead_usd,
            self.infracost_iac_files_detected,
            self.hosted_effective_hourly_rate_usd,
            json_escape(&self.infracost_recommended_ec2_instance),
            self.infracost_ec2_on_demand_hourly_usd,
            self.infracost_ec2_on_demand_monthly_usd,
            self.infracost_ec2_spot_hourly_usd,
            self.infracost_ec2_surrounding_overhead_monthly_usd,
            self.macos_self_hosted_mac_mini_monthly_usd,
            self.self_hosted_breakeven_mins_t3_medium,
            self.self_hosted_breakeven_mins_c5_4xlarge_spot,
            self.always_on_runner_utilization_pct,
            json_escape(&self.ephemeral_runner_recommended_mode),
            self.ephemeral_cold_start_wait_tax_monthly_usd,
            self.k8s_eks_control_plane_monthly_usd,
            self.k8s_gke_aks_free_tier_control_plane_monthly_usd,
            json_escape(&self.spot_interruption_band_pct),
            self.spot_retry_overhead_monthly_usd,
            self.spot_ineligible_jobs_count,
            self.hybrid_spot_plus_on_demand_monthly_usd,
            self.jobs_uploading_artifacts,
            self.default_retention_days,
            self.configured_retention_days,
            self.steady_state_stored_gb_configured,
            self.monthly_vendor_storage_configured_usd,
            self.monthly_infracost_s3_configured_usd,
            self.steady_state_stored_gb_90d,
            self.steady_state_stored_gb_14d,
            self.monthly_vendor_storage_90d_usd,
            self.monthly_vendor_storage_14d_usd,
            self.monthly_infracost_s3_standard_90d_usd,
            self.monthly_infracost_s3_intelligent_14d_usd,
            self.peak_concurrent_jobs_per_run,
            self.hosted_concurrent_jobs_per_run,
            self.macos_concurrent_jobs_per_run,
            self.self_hosted_or_microvm_jobs_per_run,
            self.vendor_free_concurrent_cap,
            self.vendor_team_concurrent_cap,
            self.parallelism_tier_upgrade_tax_monthly_usd,
            json_escape(&self.detected_repo_matrix_profile),
            self.matrix_jobs_count,
            self.pr_unconditional_matrix_cells,
            self.conditional_pr_matrix_monthly_savings_usd,
            json_escape(&self.detected_log_language_profile),
            self.est_log_mb_per_run,
            self.monthly_log_volume_gb,
            self.vendor_default_log_retention_days,
            self.monthly_vendor_default_log_cost_usd,
            self.monthly_datadog_30d_log_shipping_usd,
            self.monthly_datadog_90d_log_shipping_usd,
            self.monthly_splunk_30d_log_shipping_usd,
            self.monthly_infracost_loki_s3_usd,
            json_escape(&self.infracost_cloud_region),
            self.est_transfer_gb_per_run,
            self.monthly_transfer_gb,
            self.infracost_aws_internet_egress_per_gb_usd,
            self.infracost_aws_nat_gateway_per_gb_usd,
            self.infracost_aws_cross_az_per_gb_usd,
            self.infracost_aws_cross_region_per_gb_usd,
            self.monthly_infracost_private_subnet_nat_worst_case_usd,
            self.monthly_infracost_cross_region_registry_usd,
            self.monthly_infracost_vpc_endpoint_cache_optimized_usd,
            self.productivity_loss_factor,
            self.blended_fully_loaded_hourly_usd,
            self.monthly_wait_hours,
            self.monthly_productive_payroll_loss_usd,
            self.monthly_blended_100hr_productive_loss_usd,
            self.wait_to_compute_cost_ratio,
            json_escape(&self.lead_time_feedback_tier),
            self.estimated_flake_rate_pct,
            self.has_job_level_auto_retry,
            self.monthly_flaky_reruns,
            self.monthly_rerun_compute_cost_usd,
            self.avg_investigation_mins_per_flake,
            self.monthly_flake_investigation_hours,
            self.monthly_flake_investigation_payroll_usd,
            self.monthly_total_immediate_flake_cost_usd,
            self.monthly_savings_at_1pct_target_usd,
            json_escape(&self.stack_cache_profile),
            self.realistic_saving_pct_min,
            self.realistic_saving_pct_max,
            self.jobs_with_cache,
            self.jobs_missing_cache,
            self.has_restore_keys_fallback,
            self.has_docker_buildkit_cache,
            self.estimated_cache_hit_rate_pct,
            json_escape(&self.vendor_cache_size_limit),
            json_escape(&self.vendor_cache_eviction_policy),
            self.monthly_uncached_compute_usd,
            self.monthly_unrealized_cache_savings_usd,
            json_escape(&self.next_recommended_caching_layer),
            self.distributed_remote_cache_monthly_infra_usd,
            self.container_base_image_mirror_monthly_savings_usd,
            self.pr_cache_poisoning_risk_jobs,
            self.monthly_total_immediate_flake_cost_usd,
            self.deferred_velocity_drag_multiplier,
            self.monthly_deferred_velocity_drag_usd,
            self.monthly_combined_flake_tax_usd,
            self.deflake_project_est_cost_usd,
            self.deflake_payback_weeks,
            self.required_checks_count,
            self.monthly_required_check_executions,
            self.workflows_with_concurrency_cancel,
            self.workflows_missing_concurrency_cancel,
            self.workflows_with_path_filters,
            self.workflows_missing_path_filters,
            self.has_affected_only_graph_tool,
            self.slow_blocking_checks_over_10m,
            self.monthly_superseded_run_waste_usd,
            self.monthly_unfiltered_check_savings_usd,
            json_escape(&self.inferred_team_stage),
            self.implied_active_devs,
            self.free_tier_burn_pct,
            self.upgrade_trigger_80pct_reached,
            self.premature_self_hosting_trap,
            self.premature_multi_os_pr_matrix_trap,
            self.monthly_multi_os_pr_to_nightly_savings_usd,
            self.mac_mini_hardware_payback_months,
            self.self_hosted_breakeven_reached,
            self.team_to_enterprise_seat_jump_monthly_usd,
            self.quarterly_required_check_audit_savings_usd,
            self.platform_engineer_monthly_fully_loaded_usd,
            self.recoverable_ci_and_wait_monthly_usd,
            self.platform_engineer_roi_coverage_pct,
            compliance_json,
            self.compliance_controls_count,
            regions_json,
            self.regional_residency_multiplier,
            self.enterprise_250_seat_licence_monthly_usd,
            self.siem_1yr_audit_log_retention_monthly_usd,
            self.dedicated_tenancy_premium_pct_min,
            self.dedicated_tenancy_premium_pct_max,
            json_escape(&self.detected_pipeline_shape),
            self.detected_workspace_units_count,
            json_escape(&self.detected_monorepo_tool),
            json_escape(&self.recommended_affected_tool),
            self.has_nightly_full_suite_safety_net,
            self.affected_only_reduction_pct_min,
            self.affected_only_reduction_pct_max,
            self.monthly_affected_only_savings_usd,
            self.est_setup_overhead_sec_per_job,
            self.monthly_setup_overhead_hours,
            self.setup_overhead_share_pct,
            self.monthly_setup_overhead_compute_usd,
            self.monthly_shared_infra_setup_savings_usd,
            self.reusable_workflow_calls_count,
            self.unpinned_template_refs_count,
            self.cross_repo_triggers_count,
            self.est_aggregate_cache_gb,
            self.exceeds_10gb_repo_cache_limit,
            json_escape(&self.cheapest_vendor_platform),
            self.cheapest_vendor_monthly_tco_usd,
            self.github_actions_team_monthly_tco_usd,
            self.gitlab_ci_premium_monthly_tco_usd,
            self.circleci_performance_monthly_tco_usd,
            self.bitbucket_premium_monthly_tco_usd,
            self.buildkite_pro_monthly_tco_usd,
            self.jenkins_self_hosted_monthly_tco_usd,
            self.aws_codebuild_monthly_tco_usd,
            self.gcp_cloud_build_monthly_tco_usd,
            self.azure_pipelines_monthly_tco_usd,
            self.migration_estimated_hours,
            self.migration_estimated_cost_usd,
            self.migration_payback_months,
            self.migration_worth_it_2x_annual_rule,
            self.phase_clone_checkout_share_pct,
            self.phase_dependency_install_share_pct,
            self.phase_compile_typecheck_share_pct,
            self.phase_test_suite_share_pct,
            self.phase_docker_build_push_share_pct,
            self.phase_deploy_cleanup_share_pct,
            self.monthly_total_reducible_phase_savings_usd,
            json_escape(&self.best_first_optimization_phase),
            self.best_first_optimization_effort_hours,
            self.best_first_optimization_monthly_savings_usd,
            self.best_first_optimization_payback_months,
            json_escape(&self.detected_workload_compute_profile),
            self.larger_runners_detected_count,
            self.larger_runners_bypass_free_minutes_tax_usd,
            self.arm64_2core_monthly_compute_usd,
            self.arm64_migration_monthly_savings_usd,
            self.standard_2core_monthly_compute_usd,
            self.larger_4core_monthly_compute_usd,
            self.larger_8core_monthly_compute_usd,
            self.larger_16core_monthly_compute_usd,
            json_escape(&self.best_net_roi_runner_tier),
            self.best_net_roi_monthly_delta_usd,
            json_escape(&self.detected_stack_archetype),
            self.ios_codemagic_annual_m2_monthly_usd,
            self.ios_buildkite_mac_mini_fleet_monthly_tco_usd,
            self.android_emulator_jobs_count,
            self.android_kvm_or_gradle_cache_savings_usd,
            self.jvm_warm_cache_50pct_savings_usd,
            self.stack_specific_monthly_savings_usd,
            json_escape(&self.stack_specific_primary_lever),
            models_21_to_24_json
        )
    }
}

#[derive(Debug, Clone)]
pub struct BreakdownReport {
    pub target_path: String,
    pub provenance_mode: String,
    pub execution_time_us: u128,
    pub execution_time_ms: f64,
    pub detected_providers: Vec<String>,
    pub usage: UsageConfig,
    pub total_workflows: usize,
    pub total_defined_jobs: usize,
    pub total_expanded_matrix_jobs: usize,
    pub raw_duration_mins_per_run: f64,
    pub critical_path_wait_mins_per_run: f64,
    pub billed_mins_per_run: f64,
    pub monthly_runs: f64,
    pub monthly_billed_mins: f64,
    pub free_tier_included_mins: f64,
    pub free_tier_quota_burn_pct: f64,
    pub effective_tier: String,
    pub gross_compute_cost_per_run_usd: f64,
    pub gross_monthly_compute_cost_usd: f64,
    pub discount_allowance_monthly_usd: f64,
    pub net_billed_monthly_compute_cost_usd: f64,
    pub third_party_runner_monthly_cost_usd: f64,
    pub shard_setup_duplication_mins_per_run: f64,
    pub clones_per_ci_run: usize,
    pub dev_wait_cost_per_run_usd: f64,
    pub monthly_dev_wait_cost_usd: f64,
    pub monthly_artifact_storage_vendor_usd: f64,
    pub monthly_artifact_storage_s3_usd: f64,
    pub monthly_blacksmith_cost_usd: f64,
    pub monthly_self_hosted_spot_cost_usd: f64,
    pub exceeds_5000m_self_hosted_breakeven: bool,
    pub cost_components: CostComponentsBreakdown,
    pub jobs: Vec<ExpandedJobCost>,
}

impl BreakdownReport {
    pub fn to_json(&self) -> String {
        let provs: Vec<String> = self
            .detected_providers
            .iter()
            .map(|p| format!("\"{}\"", json_escape(p)))
            .collect();
        let job_strs: Vec<String> = self.jobs.iter().map(|j| j.to_json()).collect();
        format!(
            "{{\n  \"target_path\": \"{}\",\n  \"provenance_mode\": \"{}\",\n  \"execution_time_us\": {},\n  \"execution_time_ms\": {:.3},\n  \"detected_providers\": [{}],\n  \"usage\": {},\n  \"total_workflows\": {},\n  \"total_defined_jobs\": {},\n  \"total_expanded_matrix_jobs\": {},\n  \"raw_duration_mins_per_run\": {:.2},\n  \"critical_path_wait_mins_per_run\": {:.2},\n  \"billed_mins_per_run\": {:.2},\n  \"shard_setup_duplication_mins_per_run\": {:.2},\n  \"clones_per_ci_run\": {},\n  \"monthly_runs\": {:.1},\n  \"monthly_billed_mins\": {:.2},\n  \"free_tier_included_mins\": {:.1},\n  \"free_tier_quota_burn_pct\": {:.1},\n  \"effective_tier\": \"{}\",\n  \"gross_compute_cost_per_run_usd\": {:.4},\n  \"gross_monthly_compute_cost_usd\": {:.2},\n  \"discount_allowance_monthly_usd\": {:.2},\n  \"net_billed_monthly_compute_cost_usd\": {:.2},\n  \"third_party_runner_monthly_cost_usd\": {:.2},\n  \"dev_wait_cost_per_run_usd\": {:.2},\n  \"monthly_dev_wait_cost_usd\": {:.2},\n  \"monthly_artifact_storage_vendor_usd\": {:.2},\n  \"monthly_artifact_storage_s3_usd\": {:.2},\n  \"monthly_blacksmith_cost_usd\": {:.2},\n  \"monthly_self_hosted_spot_cost_usd\": {:.2},\n  \"exceeds_5000m_self_hosted_breakeven\": {},\n  \"cost_components\": {},\n  \"jobs\": [\n    {}\n  ]\n}}",
            json_escape(&self.target_path),
            json_escape(&self.provenance_mode),
            self.execution_time_us,
            self.execution_time_ms,
            provs.join(", "),
            self.usage.to_json(),
            self.total_workflows,
            self.total_defined_jobs,
            self.total_expanded_matrix_jobs,
            self.raw_duration_mins_per_run,
            self.critical_path_wait_mins_per_run,
            self.billed_mins_per_run,
            self.shard_setup_duplication_mins_per_run,
            self.clones_per_ci_run,
            self.monthly_runs,
            self.monthly_billed_mins,
            self.free_tier_included_mins,
            self.free_tier_quota_burn_pct,
            json_escape(&self.effective_tier),
            self.gross_compute_cost_per_run_usd,
            self.gross_monthly_compute_cost_usd,
            self.discount_allowance_monthly_usd,
            self.net_billed_monthly_compute_cost_usd,
            self.third_party_runner_monthly_cost_usd,
            self.dev_wait_cost_per_run_usd,
            self.monthly_dev_wait_cost_usd,
            self.monthly_artifact_storage_vendor_usd,
            self.monthly_artifact_storage_s3_usd,
            self.monthly_blacksmith_cost_usd,
            self.monthly_self_hosted_spot_cost_usd,
            self.exceeds_5000m_self_hosted_breakeven,
            self.cost_components.to_json(),
            job_strs.join(",\n    ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct DiffReport {
    pub base_path: String,
    pub head_path: String,
    pub execution_time_us: u128,
    pub base_expanded_jobs: usize,
    pub head_expanded_jobs: usize,
    pub delta_expanded_jobs: isize,
    pub base_billed_mins_per_run: f64,
    pub head_billed_mins_per_run: f64,
    pub delta_billed_mins_per_run: f64,
    pub base_monthly_compute_usd: f64,
    pub head_monthly_compute_usd: f64,
    pub delta_monthly_compute_usd: f64,
    pub delta_monthly_compute_pct: f64,
    pub base_monthly_dev_wait_usd: f64,
    pub head_monthly_dev_wait_usd: f64,
    pub delta_monthly_dev_wait_usd: f64,
    pub base_free_quota_burn_pct: f64,
    pub head_free_quota_burn_pct: f64,
    pub job_changes: Vec<String>,
    pub head_report: BreakdownReport,
}

impl DiffReport {
    pub fn to_json(&self) -> String {
        let ch: Vec<String> = self
            .job_changes
            .iter()
            .map(|c| format!("\"{}\"", json_escape(c)))
            .collect();
        format!(
            "{{\n  \"base_path\": \"{}\",\n  \"head_path\": \"{}\",\n  \"execution_time_us\": {},\n  \"base_expanded_jobs\": {},\n  \"head_expanded_jobs\": {},\n  \"delta_expanded_jobs\": {},\n  \"base_billed_mins_per_run\": {:.2},\n  \"head_billed_mins_per_run\": {:.2},\n  \"delta_billed_mins_per_run\": {:.2},\n  \"base_monthly_compute_usd\": {:.2},\n  \"head_monthly_compute_usd\": {:.2},\n  \"delta_monthly_compute_usd\": {:.2},\n  \"delta_monthly_compute_pct\": {:.1},\n  \"base_monthly_dev_wait_usd\": {:.2},\n  \"head_monthly_dev_wait_usd\": {:.2},\n  \"delta_monthly_dev_wait_usd\": {:.2},\n  \"base_free_quota_burn_pct\": {:.1},\n  \"head_free_quota_burn_pct\": {:.1},\n  \"job_changes\": [{}],\n  \"cost_components\": {}\n}}",
            json_escape(&self.base_path),
            json_escape(&self.head_path),
            self.execution_time_us,
            self.base_expanded_jobs,
            self.head_expanded_jobs,
            self.delta_expanded_jobs,
            self.base_billed_mins_per_run,
            self.head_billed_mins_per_run,
            self.delta_billed_mins_per_run,
            self.base_monthly_compute_usd,
            self.head_monthly_compute_usd,
            self.delta_monthly_compute_usd,
            self.delta_monthly_compute_pct,
            self.base_monthly_dev_wait_usd,
            self.head_monthly_dev_wait_usd,
            self.delta_monthly_dev_wait_usd,
            self.base_free_quota_burn_pct,
            self.head_free_quota_burn_pct,
            ch.join(", "),
            self.head_report.cost_components.to_json()
        )
    }
}

fn load_usage_config(root: &Path, override_cfg: Option<UsageConfig>) -> UsageConfig {
    let mut cfg = override_cfg.unwrap_or_default();
    let base_dir = if root.is_file() {
        root.parent().unwrap_or(Path::new("."))
    } else {
        root
    };
    let usage_file = base_dir.join("minici-usage.yml");
    if usage_file.is_file() {
        if let Ok(raw) = fs::read_to_string(&usage_file) {
            for line in raw.lines() {
                let t = line.trim();
                if t.starts_with('#') || !t.contains(':') {
                    continue;
                }
                let parts: Vec<&str> = t.splitn(2, ':').collect();
                let k = parts[0].trim();
                let v = parts[1]
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                match k {
                    "tier" => cfg.tier = v.to_string(),
                    "prs_per_week" => {
                        if let Ok(n) = v.parse::<f64>() {
                            cfg.prs_per_week = n;
                        }
                    }
                    "pushes_per_pr" => {
                        if let Ok(n) = v.parse::<f64>() {
                            cfg.pushes_per_pr = n;
                        }
                    }
                    "dev_hourly_rate_usd" => {
                        if let Ok(n) = v.parse::<f64>() {
                            cfg.dev_hourly_rate_usd = n;
                        }
                    }
                    "artifact_gb_per_run" => {
                        if let Ok(n) = v.parse::<f64>() {
                            cfg.artifact_gb_per_run = n;
                        }
                    }
                    "artifact_retention_days" => {
                        if let Ok(n) = v.parse::<u32>() {
                            cfg.artifact_retention_days = n.clamp(1, 400);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    cfg
}

/// Parses GitHub Actions `retention-days: <N>` or GitLab CI `expire_in: <duration>` into days (`1..=400`).
pub fn parse_retention_days_value(raw: &str) -> Option<u32> {
    let clean = raw
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase();
    if clean.is_empty() {
        return None;
    }
    if let Ok(days) = clean.parse::<u32>() {
        return Some(days.clamp(1, 400));
    }
    let num_part: String = clean.chars().take_while(|c| c.is_ascii_digit()).collect();
    if let Ok(n) = num_part.parse::<u32>() {
        let rest = clean[num_part.len()..].trim();
        let days = if rest.starts_with("week") || rest == "w" {
            n.saturating_mul(7)
        } else if rest.starts_with("month") || rest == "mo" {
            n.saturating_mul(30)
        } else if rest.starts_with("year") || rest == "y" || rest == "yr" {
            n.saturating_mul(365)
        } else if rest.starts_with("hour") || rest.starts_with("hr") || rest == "h" {
            (n.saturating_add(23)) / 24
        } else {
            n
        };
        return Some(days.clamp(1, 400));
    }
    None
}

fn extract_os_from_flow_map(s: &str) -> Option<String> {
    let inner = s.trim().strip_prefix('{')?.strip_suffix('}')?;
    for part in inner.split(',') {
        let kv = part.trim();
        if let Some(val) = kv
            .strip_prefix("os:")
            .or_else(|| kv.strip_prefix("runner:"))
            .or_else(|| kv.strip_prefix("runs-on:"))
        {
            let clean = val.trim().trim_matches('"').trim_matches('\'');
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

fn parse_inline_yaml_list(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            if end > start {
                let inner = &trimmed[start + 1..end];
                return inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

fn strip_comment(line: &str) -> &str {
    if let Some(idx) = line.find(" #") {
        &line[..idx]
    } else {
        line
    }
}

fn load_repo_visibility(root: &Path) -> RepoVisibility {
    if let Ok(env_vis) = std::env::var("MINICI_REPO_VISIBILITY") {
        let parsed = RepoVisibility::from_str_opt(&env_vis);
        if parsed != RepoVisibility::Unknown {
            return parsed;
        }
    }
    let base_dir = if root.is_file() {
        root.parent().unwrap_or(Path::new("."))
    } else {
        root
    };
    let usage_file = base_dir.join("minici-usage.yml");
    if usage_file.is_file() {
        if let Ok(raw) = fs::read_to_string(&usage_file) {
            for line in raw.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("visibility:") {
                    let v = rest
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    let parsed = RepoVisibility::from_str_opt(v);
                    if parsed != RepoVisibility::Unknown {
                        return parsed;
                    }
                }
            }
        }
    }
    RepoVisibility::Unknown
}

fn line_has_cache_signal(low_line: &str) -> bool {
    // Fixes Bug #26: bare `actions/setup-node`, `setup-python`, `setup-go`, or `setup-uv`
    // WITHOUT `cache:` do NOT enable dependency caching by default; when `with: cache: ...`
    // is present, `low_line.contains("cache:")` accurately detects it!
    low_line.contains("actions/cache")
        || low_line.contains("cache:")
        || low_line.contains("cache-key:")
        || low_line.contains("cache-from")
        || low_line.contains("cache-to")
        || low_line.contains("setup-docker-builder")
        || low_line.contains("rust-cache")
        || low_line.contains("sccache")
        || low_line.contains("ccache")
        || low_line.contains("setup-gradle")
        || low_line.contains("gradle-build-action")
        || low_line.contains("save_cache")
        || low_line.contains("restore_cache")
        || low_line.contains("./.github/actions/")
}

/// Fast zero-allocation scanner for a single GitHub Actions job block (`lines` belonging to one job).
/// Returns `Some((raw_mins_per_job, needs_job_ids))` when the job allocates a runner VM.
fn analyze_github_job_block(
    wf_rel: &str,
    job_id: &str,
    job_lines: &[&str],
    has_top_concurrency: bool,
    has_top_cache: bool,
    out_jobs: &mut Vec<ExpandedJobCost>,
) -> Option<(f64, Vec<String>)> {
    let mut raw_runs_on = "ubuntu-latest".to_string();
    let mut saw_runs_on = false;
    let mut uses_local_reusable_workflow = false;
    let mut has_job_concurrency = has_top_concurrency;
    let mut uses_cache = has_top_cache;
    let mut explicit_est_mins: Option<f64> = None;
    let mut has_sparse = false;
    let mut uploads_artifacts = false;
    let mut artifact_retention_days: Option<u32> = None;
    let mut step_mins = 0.5_f64;
    let mut saw_steps = false;
    let mut needs_ids: Vec<String> = Vec::new();
    let mut in_needs_list = false;
    let mut needs_indent = 0usize;

    // Matrix tracking
    let mut in_matrix = false;
    let mut matrix_indent = 0usize;
    let mut current_matrix_axis = String::new();
    let mut axis_indent = 0usize;
    let mut axis_counts: Vec<(String, Vec<String>)> = Vec::new();
    let mut include_items = 0usize;
    let mut exclude_items = 0usize;

    for &raw_line in job_lines {
        let clean = strip_comment(raw_line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let ind = indent_of(clean);
        let low_trimmed = trimmed.to_ascii_lowercase();
        if line_has_cache_signal(&low_trimmed) {
            uses_cache = true;
        }

        // Parse `needs:` job dependencies for true DAG critical-path calculation (Fixes Bug #9)
        if !saw_steps && !in_matrix {
            if in_needs_list {
                if ind > needs_indent && trimmed.starts_with('-') {
                    let dep = trimmed
                        .trim_start_matches('-')
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    if !dep.is_empty() {
                        needs_ids.push(dep.to_string());
                    }
                    continue;
                } else {
                    in_needs_list = false;
                }
            }
            if trimmed.starts_with("needs:") {
                if let Some(rhs) = trimmed.splitn(2, ':').nth(1) {
                    let rhs_t = rhs.trim();
                    if rhs_t.is_empty() {
                        in_needs_list = true;
                        needs_indent = ind;
                    } else if rhs_t.starts_with('[') {
                        for dep in parse_inline_yaml_list(rhs_t) {
                            needs_ids.push(dep);
                        }
                    } else {
                        let single = rhs_t.trim_matches('"').trim_matches('\'');
                        if !single.is_empty() {
                            needs_ids.push(single.to_string());
                        }
                    }
                }
            }
        }

        if trimmed.starts_with("concurrency:") {
            has_job_concurrency = true;
        }
        if trimmed.starts_with("est-minutes:") || trimmed.starts_with("estimated-minutes:") {
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                if let Ok(m) = val.trim().parse::<f64>() {
                    explicit_est_mins = Some(m.max(0.2));
                }
            }
        }
        if !saw_steps && !in_matrix && trimmed.starts_with("uses:") {
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                let target = val.trim().trim_matches('"').trim_matches('\'');
                if target.starts_with("./.github/workflows/")
                    || target.starts_with(".github/workflows/")
                {
                    uses_local_reusable_workflow = true;
                }
            }
        }
        if trimmed.starts_with("runs-on:") {
            saw_runs_on = true;
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                let list = parse_inline_yaml_list(val);
                if !list.is_empty() {
                    raw_runs_on = list.join("-");
                } else {
                    raw_runs_on = val
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }

        // Matrix block detection
        if trimmed.starts_with("matrix:") {
            in_matrix = true;
            matrix_indent = ind;
            current_matrix_axis.clear();
            continue;
        }
        if in_matrix {
            if ind <= matrix_indent && !trimmed.starts_with('-') {
                in_matrix = false;
            } else {
                // Inside matrix block
                if trimmed.starts_with('-') && !current_matrix_axis.is_empty() {
                    let item_val = trimmed
                        .trim_start_matches('-')
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    if current_matrix_axis == "include" {
                        if ind <= axis_indent + 4 {
                            include_items += 1;
                            if item_val.starts_with('{') {
                                let clean_os = extract_os_from_flow_map(item_val)
                                    .unwrap_or_else(|| "ubuntu-latest".to_string());
                                axis_counts.push(("__include_os__".to_string(), vec![clean_os]));
                            } else if let Some(os_val) = item_val.strip_prefix("os:") {
                                let clean_os = os_val.trim().trim_matches('"').trim_matches('\'');
                                if !clean_os.is_empty() {
                                    axis_counts.push((
                                        "__include_os__".to_string(),
                                        vec![clean_os.to_string()],
                                    ));
                                }
                            }
                        }
                    } else if current_matrix_axis == "exclude" {
                        if ind <= axis_indent + 4 {
                            exclude_items += 1;
                        }
                    } else if let Some(last) = axis_counts.last_mut() {
                        if last.0 == current_matrix_axis
                            && (!item_val.contains(':') || item_val.starts_with('{'))
                        {
                            last.1.push(item_val.to_string());
                        }
                    }
                } else if !trimmed.starts_with('-') && trimmed.contains(':') && ind <= matrix_indent + 4 {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    let key = parts[0].trim();
                    let rhs = parts[1].trim();
                    current_matrix_axis = key.to_string();
                    axis_indent = ind;
                    if key != "include" && key != "exclude" {
                        let inline = parse_inline_yaml_list(rhs);
                        if !inline.is_empty() {
                            axis_counts.push((key.to_string(), inline));
                            current_matrix_axis.clear();
                        } else {
                            axis_counts.push((key.to_string(), Vec::new()));
                        }
                    }
                }
                continue;
            }
        }

        // Steps & duration heuristics (Fixes Bug #11: modern JS/TS/Mobile/JVM/C++ toolchains + independent build & test accumulation)
        if trimmed.starts_with("steps:") {
            saw_steps = true;
            continue;
        }
        if saw_steps {
            if trimmed.contains("actions/checkout") {
                step_mins += 0.35;
            } else if trimmed.starts_with("filter:")
                || trimmed.starts_with("sparse-checkout:")
                || trimmed
                    .strip_prefix("fetch-depth:")
                    .map(|v| v.trim().trim_matches('"').trim_matches('\'') == "1")
                    .unwrap_or(false)
            {
                if !has_sparse {
                    has_sparse = true;
                    step_mins = (step_mins - 0.3).max(0.5);
                }
            } else if trimmed.contains("upload-artifact") {
                uploads_artifacts = true;
                step_mins += 0.35;
            } else if trimmed.starts_with("retention-days:") {
                if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                    if let Some(days) = parse_retention_days_value(val) {
                        artifact_retention_days = Some(days);
                    }
                }
            } else {
                let has_test_cmd = low_trimmed.contains("pytest")
                    || low_trimmed.contains("cargo test")
                    || low_trimmed.contains("cargo nextest")
                    || low_trimmed.contains("npm test")
                    || low_trimmed.contains("pnpm test")
                    || low_trimmed.contains("yarn test")
                    || low_trimmed.contains("bun test")
                    || low_trimmed.contains("jest")
                    || low_trimmed.contains("playwright test")
                    || low_trimmed.contains("cypress run")
                    || (low_trimmed.contains("vitest")
                        && !low_trimmed.contains(".vitest-reports")
                        && !low_trimmed.contains("--merge-reports")
                        && !low_trimmed.starts_with("path:"))
                    || low_trimmed.contains("go test")
                    || low_trimmed.contains("mvn test")
                    || low_trimmed.contains("gradlew test")
                    || low_trimmed.contains("connectedandroidtest")
                    || low_trimmed.contains("dotnet test")
                    || low_trimmed.contains("swift test")
                    || low_trimmed.contains("flutter test")
                    || low_trimmed.contains("ctest")
                    || (low_trimmed.contains("xcodebuild") && low_trimmed.contains("test"));

                let has_build_cmd = low_trimmed.contains("docker build")
                    || low_trimmed.contains("docker compose build")
                    || low_trimmed.contains("buildx build")
                    || low_trimmed.contains("cargo build --release")
                    || low_trimmed.contains("npm run build")
                    || low_trimmed.contains("pnpm build")
                    || low_trimmed.contains("pnpm run build")
                    || low_trimmed.contains("yarn build")
                    || low_trimmed.contains("bun run build")
                    || low_trimmed.contains("gradlew assemble")
                    || low_trimmed.contains("gradlew build")
                    || low_trimmed.contains("dotnet build")
                    || low_trimmed.contains("swift build")
                    || low_trimmed.contains("flutter build")
                    || low_trimmed.contains("cmake --build")
                    || low_trimmed.contains("bazel build")
                    || (low_trimmed.contains("xcodebuild") && !low_trimmed.contains("test"));

                if has_test_cmd || has_build_cmd {
                    if has_test_cmd {
                        step_mins += 2.5;
                    }
                    if has_build_cmd {
                        step_mins += 3.5;
                    }
                } else if trimmed.starts_with("- name:")
                    || trimmed.starts_with("- run:")
                    || trimmed.starts_with("- uses:")
                {
                    step_mins += 0.25;
                }
            }
        }
    }

    // Pure local reusable-workflow caller stubs (`uses: ./.github/workflows/...` with no `runs-on:` or `steps:`)
    // orchestrate another workflow already present in `.github/workflows/` and do not allocate their own runner VM.
    if uses_local_reusable_workflow && !saw_runs_on && !saw_steps {
        return None;
    }

    let raw_mins = explicit_est_mins.unwrap_or(if saw_steps { step_mins } else { 3.5 });

    let mut product_count = 1usize;
    let mut matrix_os_list: Vec<String> = Vec::new();
    let mut include_os_list: Vec<String> = Vec::new();
    let mut non_empty_axes = 0usize;
    for (axis_name, items) in &axis_counts {
        if axis_name == "__include_os__" {
            for it in items {
                include_os_list.push(it.clone());
            }
            continue;
        }
        if !items.is_empty() {
            non_empty_axes += 1;
            product_count = product_count.saturating_mul(items.len());
            if axis_name == "os" || axis_name == "runner" || axis_name == "runs-on" {
                matrix_os_list = items.clone();
            } else if items
                .iter()
                .any(|it| it.starts_with('{') && extract_os_from_flow_map(it).is_some())
            {
                matrix_os_list = items
                    .iter()
                    .map(|it| {
                        extract_os_from_flow_map(it)
                            .unwrap_or_else(|| "ubuntu-latest".to_string())
                    })
                    .collect();
            }
        }
    }
    let matrix_count = if non_empty_axes == 0 && include_items > 0 {
        if matrix_os_list.is_empty() && !include_os_list.is_empty() {
            matrix_os_list = include_os_list.clone();
        }
        include_items
    } else {
        product_count.saturating_sub(exclude_items) + include_items
    }
    .max(1);

    if !matrix_os_list.is_empty() && raw_runs_on.contains("${{") {
        let mut os_buckets: Vec<(String, usize)> = Vec::new();
        for os_l in &matrix_os_list {
            if let Some(found) = os_buckets.iter_mut().find(|x| x.0 == *os_l) {
                found.1 += 1;
            } else {
                os_buckets.push((os_l.clone(), 1));
            }
        }
        if non_empty_axes == 0 && include_items > 0 {
            for (os_label, per_os_jobs) in os_buckets {
                let sku = resolve_runner_sku(&os_label);
                let billed_per_job = if sku.ceil_to_whole_minute {
                    raw_mins.ceil().max(1.0)
                } else {
                    raw_mins
                };
                let total_billed = billed_per_job * (per_os_jobs as f64);
                out_jobs.push(ExpandedJobCost {
                    workflow_file: wf_rel.to_string(),
                    job_id: format!("{} [{}]", job_id, os_label),
                    matrix_combination_count: per_os_jobs,
                    runner_label: os_label,
                    provider: sku.provider.to_string(),
                    os: sku.os.to_string(),
                    vcpus: sku.vcpus,
                    est_raw_duration_mins_per_job: (raw_mins * 100.0).round() / 100.0,
                    billed_mins_per_job: billed_per_job,
                    total_billed_mins_all_matrix: total_billed,
                    rate_per_min_usd: sku.rate_per_min_usd,
                    run_compute_cost_usd: ((total_billed * sku.rate_per_min_usd) * 10000.0).round()
                        / 10000.0,
                    blacksmith_run_cost_usd: ((raw_mins
                        * (per_os_jobs as f64)
                        * sku.blacksmith_equiv_rate_usd)
                        * 10000.0)
                        .round()
                        / 10000.0,
                    self_hosted_spot_run_cost_usd: ((raw_mins
                        * (per_os_jobs as f64)
                        * sku.self_hosted_spot_rate_usd)
                        * 10000.0)
                        .round()
                        / 10000.0,
                    has_concurrency_cancel: has_job_concurrency,
                    has_sparse_or_shallow_clone: has_sparse,
                    uses_cache,
                    uploads_artifacts,
                    artifact_retention_days,
                });
            }
            return Some((raw_mins, needs_ids));
        }

        // Fixes Bug #13: Attribute explicit `include:` OS entries (`include_os_list`) directly to their matching OS bucket
        // (or create a new OS bucket if `include:` introduces an extra OS such as `macos-latest` on top of `ubuntu-latest`)!
        let total_os_items = matrix_os_list.len().max(1);
        let mut bucket_jobs: Vec<(String, usize)> = os_buckets
            .iter()
            .map(|(lbl, c)| (lbl.clone(), ((product_count * *c) / total_os_items).max(1)))
            .collect();
        for inc_os in &include_os_list {
            if let Some(found) = bucket_jobs.iter_mut().find(|x| x.0 == *inc_os) {
                found.1 += 1;
            } else {
                bucket_jobs.push((inc_os.clone(), 1));
            }
        }
        let current_sum: usize = bucket_jobs.iter().map(|(_, n)| *n).sum();
        if current_sum != matrix_count && !bucket_jobs.is_empty() {
            let others_sum: usize = bucket_jobs.iter().skip(1).map(|(_, n)| *n).sum();
            bucket_jobs[0].1 = matrix_count.saturating_sub(others_sum).max(1);
        }

        for (os_label, per_os_jobs) in bucket_jobs {
            let sku = resolve_runner_sku(&os_label);
            let billed_per_job = if sku.ceil_to_whole_minute {
                raw_mins.ceil().max(1.0)
            } else {
                raw_mins
            };
            let total_billed = billed_per_job * (per_os_jobs as f64);
            out_jobs.push(ExpandedJobCost {
                workflow_file: wf_rel.to_string(),
                job_id: format!("{} [{}]", job_id, os_label),
                matrix_combination_count: per_os_jobs,
                runner_label: os_label,
                provider: sku.provider.to_string(),
                os: sku.os.to_string(),
                vcpus: sku.vcpus,
                est_raw_duration_mins_per_job: (raw_mins * 100.0).round() / 100.0,
                billed_mins_per_job: billed_per_job,
                total_billed_mins_all_matrix: total_billed,
                rate_per_min_usd: sku.rate_per_min_usd,
                run_compute_cost_usd: ((total_billed * sku.rate_per_min_usd) * 10000.0).round()
                    / 10000.0,
                blacksmith_run_cost_usd: ((raw_mins
                    * (per_os_jobs as f64)
                    * sku.blacksmith_equiv_rate_usd)
                    * 10000.0)
                    .round()
                    / 10000.0,
                self_hosted_spot_run_cost_usd: ((raw_mins
                    * (per_os_jobs as f64)
                    * sku.self_hosted_spot_rate_usd)
                    * 10000.0)
                    .round()
                    / 10000.0,
                has_concurrency_cancel: has_job_concurrency,
                has_sparse_or_shallow_clone: has_sparse,
                uses_cache,
                uploads_artifacts,
                artifact_retention_days,
            });
        }
    } else {
        let sku = resolve_runner_sku(&raw_runs_on);
        let billed_per_job = if sku.ceil_to_whole_minute {
            raw_mins.ceil().max(1.0)
        } else {
            raw_mins
        };
        let total_billed = billed_per_job * (matrix_count as f64);
        let display_runner = if raw_runs_on.contains("${{") {
            sku.sku_label.to_string()
        } else {
            raw_runs_on
        };
        out_jobs.push(ExpandedJobCost {
            workflow_file: wf_rel.to_string(),
            job_id: job_id.to_string(),
            matrix_combination_count: matrix_count,
            runner_label: display_runner,
            provider: sku.provider.to_string(),
            os: sku.os.to_string(),
            vcpus: sku.vcpus,
            est_raw_duration_mins_per_job: (raw_mins * 100.0).round() / 100.0,
            billed_mins_per_job: billed_per_job,
            total_billed_mins_all_matrix: total_billed,
            rate_per_min_usd: sku.rate_per_min_usd,
            run_compute_cost_usd: ((total_billed * sku.rate_per_min_usd) * 10000.0).round()
                / 10000.0,
            blacksmith_run_cost_usd: ((raw_mins
                * (matrix_count as f64)
                * sku.blacksmith_equiv_rate_usd)
                * 10000.0)
                .round()
                / 10000.0,
            self_hosted_spot_run_cost_usd: ((raw_mins
                * (matrix_count as f64)
                * sku.self_hosted_spot_rate_usd)
                * 10000.0)
                .round()
                / 10000.0,
            has_concurrency_cancel: has_job_concurrency,
            has_sparse_or_shallow_clone: has_sparse,
            uses_cache,
            uploads_artifacts,
            artifact_retention_days,
        });
    }
    Some((raw_mins, needs_ids))
}

fn compute_dag_critical_path(dag_nodes: &[(String, f64, Vec<String>)]) -> f64 {
    if dag_nodes.is_empty() {
        return 0.0;
    }
    let mut dur_map: HashMap<&str, f64> = HashMap::new();
    let mut deps_map: HashMap<&str, &[String]> = HashMap::new();
    for (id, dur, deps) in dag_nodes {
        dur_map.insert(id.as_str(), *dur);
        deps_map.insert(id.as_str(), deps.as_slice());
    }
    fn longest_to<'a>(
        node: &'a str,
        dur_map: &HashMap<&'a str, f64>,
        deps_map: &HashMap<&'a str, &'a [String]>,
        memo: &mut HashMap<&'a str, f64>,
        depth: usize,
    ) -> f64 {
        if let Some(&cached) = memo.get(node) {
            return cached;
        }
        let own = dur_map.get(node).copied().unwrap_or(0.0);
        if depth > 32 {
            return own;
        }
        let max_dep = deps_map
            .get(node)
            .map(|deps| {
                deps.iter()
                    .map(|d| longest_to(d.as_str(), dur_map, deps_map, memo, depth + 1))
                    .fold(0.0_f64, f64::max)
            })
            .unwrap_or(0.0);
        let total = own + max_dep;
        memo.insert(node, total);
        total
    }
    let mut memo: HashMap<&str, f64> = HashMap::new();
    dag_nodes
        .iter()
        .map(|(id, _, _)| longest_to(id.as_str(), &dur_map, &deps_map, &mut memo, 0))
        .fold(0.0_f64, f64::max)
}

fn parse_github_workflow_file(
    wf_path: &Path,
    rel_name: &str,
    out_jobs: &mut Vec<ExpandedJobCost>,
) -> f64 {
    let content = match fs::read_to_string(wf_path) {
        Ok(c) => c,
        Err(_) => return 0.0,
    };
    let all_lines: Vec<&str> = content.lines().collect();
    let mut has_top_concurrency = false;
    let mut has_top_cache = false;
    let mut in_jobs = false;
    let mut jobs_indent = 0usize;
    let mut job_header_indent: Option<usize> = None;
    let mut current_job_id: Option<String> = None;
    let mut current_job_lines: Vec<&str> = Vec::new();
    let mut dag_nodes: Vec<(String, f64, Vec<String>)> = Vec::new();

    for &line in &all_lines {
        let clean = strip_comment(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let ind = indent_of(clean);

        if !in_jobs {
            if ind == 0 && trimmed.starts_with("concurrency:") {
                has_top_concurrency = true;
            }
            if line_has_cache_signal(&trimmed.to_ascii_lowercase()) {
                has_top_cache = true;
            }
        }
        if ind == 0 && trimmed == "jobs:" {
            in_jobs = true;
            jobs_indent = 0;
            continue;
        }
        if in_jobs {
            if ind == 0 && !trimmed.starts_with('#') {
                // Left `jobs:` block
                if let Some(jid) = current_job_id.take() {
                    if let Some((dur, deps)) = analyze_github_job_block(
                        rel_name,
                        &jid,
                        &current_job_lines,
                        has_top_concurrency,
                        has_top_cache,
                        out_jobs,
                    ) {
                        dag_nodes.push((jid, dur, deps));
                    }
                    current_job_lines.clear();
                }
                in_jobs = false;
                continue;
            }
            if job_header_indent.is_none() && ind > jobs_indent && trimmed.ends_with(':') {
                job_header_indent = Some(ind);
            }
            if Some(ind) == job_header_indent && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                if let Some(jid) = current_job_id.take() {
                    if let Some((dur, deps)) = analyze_github_job_block(
                        rel_name,
                        &jid,
                        &current_job_lines,
                        has_top_concurrency,
                        has_top_cache,
                        out_jobs,
                    ) {
                        dag_nodes.push((jid, dur, deps));
                    }
                    current_job_lines.clear();
                }
                current_job_id = Some(trimmed.trim_end_matches(':').trim().to_string());
            } else if current_job_id.is_some() {
                current_job_lines.push(line);
            }
        }
    }

    if let Some(jid) = current_job_id.take() {
        if let Some((dur, deps)) = analyze_github_job_block(
            rel_name,
            &jid,
            &current_job_lines,
            has_top_concurrency,
            has_top_cache,
            out_jobs,
        ) {
            dag_nodes.push((jid, dur, deps));
        }
    }

    compute_dag_critical_path(&dag_nodes)
}

/// Multi-vendor per-job parser for `.gitlab-ci.yml`, `.circleci/config.yml`, `bitbucket-pipelines.yml`, and `buildspec.yml`
/// (Fixes Bug #12: scopes `tags:`, `resource_class:`, `size:`, `parallel:`, `cache:`, and `artifacts:` per job,
/// and honors `sku.ceil_to_whole_minute` on Bitbucket & AWS CodeBuild EC2 while keeping GitLab & CircleCI per-second).
fn parse_gitlab_or_other_ci(
    ci_path: &Path,
    rel_name: &str,
    provider_hint: &str,
    out_jobs: &mut Vec<ExpandedJobCost>,
) {
    let content = match fs::read_to_string(ci_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    #[derive(Clone)]
    struct ParsedVendorJob {
        name: String,
        sku_label: String,
        parallel_count: usize,
        raw_mins: f64,
        uploads_artifacts: bool,
        retention_days: Option<u32>,
        has_interruptible: bool,
        has_cache: bool,
    }

    let mut default_sku = provider_hint.to_string();
    let mut default_cache = false;
    let mut default_interruptible = false;
    let mut jobs_list: Vec<ParsedVendorJob> = Vec::new();
    let mut current: Option<ParsedVendorJob> = None;
    let mut in_circle_jobs_section = false;
    let mut circle_job_indent: Option<usize> = None;
    let mut in_default_section = false;

    let flush_job = |cur: &mut Option<ParsedVendorJob>, list: &mut Vec<ParsedVendorJob>| {
        if let Some(j) = cur.take() {
            list.push(j);
        }
    };

    for line in content.lines() {
        let clean = strip_comment(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let ind = indent_of(clean);
        let low = trimmed.to_ascii_lowercase();

        if ind == 0 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
            let key = trimmed.trim_end_matches(':').trim();
            flush_job(&mut current, &mut jobs_list);
            in_default_section = key == "default";
            in_circle_jobs_section = key == "jobs";
            if !key.starts_with('.')
                && !matches!(
                    key,
                    "stages"
                        | "variables"
                        | "default"
                        | "include"
                        | "workflow"
                        | "workflows"
                        | "image"
                        | "version"
                        | "orbs"
                        | "executors"
                        | "commands"
                        | "pipelines"
                        | "definitions"
                        | "phases"
                        | "jobs"
                        | "cache"
                )
            {
                current = Some(ParsedVendorJob {
                    name: key.to_string(),
                    sku_label: default_sku.clone(),
                    parallel_count: 1,
                    raw_mins: 4.0,
                    uploads_artifacts: false,
                    retention_days: None,
                    has_interruptible: default_interruptible,
                    has_cache: default_cache,
                });
            }
            if key == "cache" {
                default_cache = true;
            }
            continue;
        }

        if in_circle_jobs_section {
            if circle_job_indent.is_none() && ind > 0 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                circle_job_indent = Some(ind);
            }
            if Some(ind) == circle_job_indent && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                flush_job(&mut current, &mut jobs_list);
                let jname = trimmed.trim_end_matches(':').trim().to_string();
                current = Some(ParsedVendorJob {
                    name: jname,
                    sku_label: default_sku.clone(),
                    parallel_count: 1,
                    raw_mins: 4.0,
                    uploads_artifacts: false,
                    retention_days: None,
                    has_interruptible: default_interruptible,
                    has_cache: default_cache,
                });
                continue;
            }
        }

        // Bitbucket `- step:` blocks inside `pipelines:`
        if current.is_none() && trimmed.starts_with("- step:") {
            let step_idx = jobs_list.len() + 1;
            current = Some(ParsedVendorJob {
                name: format!("step-{}", step_idx),
                sku_label: default_sku.clone(),
                parallel_count: 1,
                raw_mins: 4.0,
                uploads_artifacts: false,
                retention_days: None,
                has_interruptible: default_interruptible,
                has_cache: default_cache,
            });
            continue;
        }

        let target_cache_signal = line_has_cache_signal(&low);
        if let Some(ref mut job) = current {
            if target_cache_signal || low.starts_with("- caches") || low == "- docker" {
                job.has_cache = true;
            }
            if trimmed.starts_with("name:") && job.name.starts_with("step-") {
                if let Some(nm) = trimmed.splitn(2, ':').nth(1) {
                    let clean_nm = nm.trim().trim_matches('"').trim_matches('\'');
                    if !clean_nm.is_empty() {
                        job.name = clean_nm.to_string();
                    }
                }
            }
            if trimmed.starts_with("tags:") {
                let list = parse_inline_yaml_list(trimmed);
                if let Some(first) = list.first() {
                    job.sku_label = first.clone();
                }
            } else if trimmed.starts_with("resource_class:") {
                if let Some(rc) = trimmed.splitn(2, ':').nth(1) {
                    let rc_clean = rc.trim().trim_matches('"').trim_matches('\'');
                    job.sku_label = if rc_clean.contains("macos") || rc_clean.contains("arm.") || rc_clean.contains("gpu") {
                        rc_clean.to_string()
                    } else {
                        format!("docker-{}", rc_clean)
                    };
                }
            } else if trimmed.starts_with("macos:") {
                job.sku_label = "macos.m2pro.medium".to_string();
            } else if trimmed.starts_with("size:") {
                if let Some(sz) = trimmed.splitn(2, ':').nth(1) {
                    let sz_clean = sz.trim().trim_matches('"').trim_matches('\'');
                    job.sku_label = format!("bitbucket-{}", sz_clean);
                }
            } else if trimmed.starts_with("parallel:") || trimmed.starts_with("parallelism:") {
                if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                    if let Ok(n) = val.trim().parse::<usize>() {
                        job.parallel_count = n.max(1);
                    }
                }
            } else if trimmed.starts_with("est-minutes:") || trimmed.starts_with("estimated-minutes:") {
                if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                    if let Ok(m) = val.trim().parse::<f64>() {
                        job.raw_mins = m.max(0.2);
                    }
                }
            } else if trimmed.starts_with("artifacts:") || trimmed.contains("store_artifacts") {
                job.uploads_artifacts = true;
            } else if trimmed.starts_with("expire_in:") || trimmed.starts_with("retention-days:") {
                if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                    if let Some(days) = parse_retention_days_value(val) {
                        job.retention_days = Some(days);
                        job.uploads_artifacts = true;
                    }
                }
            } else if trimmed.starts_with("interruptible: true") || low.contains("auto-cancel") {
                job.has_interruptible = true;
            }
        } else if in_default_section {
            if target_cache_signal {
                default_cache = true;
            }
            if trimmed.starts_with("tags:") {
                let list = parse_inline_yaml_list(trimmed);
                if let Some(first) = list.first() {
                    default_sku = first.clone();
                }
            } else if trimmed.starts_with("interruptible: true") {
                default_interruptible = true;
            }
        } else if target_cache_signal {
            default_cache = true;
        }
    }
    flush_job(&mut current, &mut jobs_list);

    if jobs_list.is_empty() {
        jobs_list.push(ParsedVendorJob {
            name: "default-pipeline".to_string(),
            sku_label: default_sku,
            parallel_count: 1,
            raw_mins: 4.0,
            uploads_artifacts: false,
            retention_days: None,
            has_interruptible: default_interruptible,
            has_cache: default_cache,
        });
    }

    for pj in jobs_list {
        let sku = resolve_runner_sku(&pj.sku_label);
        let billed_per_job = if sku.ceil_to_whole_minute {
            pj.raw_mins.ceil().max(1.0)
        } else {
            pj.raw_mins
        };
        let total_billed = billed_per_job * (pj.parallel_count as f64);
        out_jobs.push(ExpandedJobCost {
            workflow_file: rel_name.to_string(),
            job_id: pj.name,
            matrix_combination_count: pj.parallel_count,
            runner_label: sku.sku_label.to_string(),
            provider: sku.provider.to_string(),
            os: sku.os.to_string(),
            vcpus: sku.vcpus,
            est_raw_duration_mins_per_job: (pj.raw_mins * 100.0).round() / 100.0,
            billed_mins_per_job: billed_per_job,
            total_billed_mins_all_matrix: total_billed,
            rate_per_min_usd: sku.rate_per_min_usd,
            run_compute_cost_usd: ((total_billed * sku.rate_per_min_usd) * 10000.0).round() / 10000.0,
            blacksmith_run_cost_usd: ((pj.raw_mins
                * (pj.parallel_count as f64)
                * sku.blacksmith_equiv_rate_usd)
                * 10000.0)
                .round()
                / 10000.0,
            self_hosted_spot_run_cost_usd: ((pj.raw_mins
                * (pj.parallel_count as f64)
                * sku.self_hosted_spot_rate_usd)
                * 10000.0)
                .round()
                / 10000.0,
            has_concurrency_cancel: pj.has_interruptible,
            has_sparse_or_shallow_clone: false,
            uses_cache: pj.has_cache,
            uploads_artifacts: pj.uploads_artifacts,
            artifact_retention_days: pj.retention_days,
        });
    }
}

/// Inspects repository manifest files, `.tf`/`terragrunt.hcl` IaC files, and CI workflow YAML contents
/// to classify the 9 `unified CI/CD FinOps/log-retention-cost` language profiles and detect Cloud Catalog IaC presence.
fn detect_language_and_iac_signals(
    target: &Path,
    jobs: &[ExpandedJobCost],
) -> (String, f64, bool) {
    let base_dir = if target.is_file() {
        target.parent().unwrap_or(Path::new("."))
    } else {
        target
    };
    let target_str_low = target.display().to_string().to_ascii_lowercase();

    // 1. Detect Terraform (`.tf`, `.tfvars`) or Terragrunt (`terragrunt.hcl`) files or IaC workflow steps
    let mut has_iac = base_dir.join("main.tf").exists()
        || base_dir.join("terragrunt.hcl").exists()
        || base_dir.join("versions.tf").exists()
        || base_dir.join("providers.tf").exists()
        || jobs.iter().any(|j| {
            let wf_low = j.workflow_file.to_ascii_lowercase();
            let id_low = j.job_id.to_ascii_lowercase();
            wf_low.contains("terragrunt")
                || wf_low.contains("terraform")
                || id_low.contains("terragrunt")
                || id_low.contains("terraform")
        });

    if !has_iac && base_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if fname.ends_with(".tf")
                    || fname.ends_with(".tfvars")
                    || fname == "terragrunt.hcl"
                {
                    has_iac = true;
                    break;
                }
            }
        }
    }

    // Collect combined workflow YAML text (for workflow-only snapshots or single workflow files)
    let mut wf_corpus = String::new();
    if target.is_file() {
        if let Ok(c) = fs::read_to_string(target) {
            wf_corpus.push_str(&c.to_ascii_lowercase());
        }
    } else {
        let gh_dir = base_dir.join(".github").join("workflows");
        if let Ok(entries) = fs::read_dir(&gh_dir) {
            for entry in entries.flatten().take(25) {
                if let Ok(c) = fs::read_to_string(entry.path()) {
                    wf_corpus.push_str(&c.to_ascii_lowercase());
                    wf_corpus.push('\n');
                }
            }
        }
    }
    if !has_iac
        && (wf_corpus.contains("terragrunt")
            || wf_corpus.contains("hashicorp/setup-terraform")
            || wf_corpus.contains("terraform plan")
            || wf_corpus.contains("terraform apply"))
    {
        has_iac = true;
    }

    // Strip synthetic snapshot test stubs (`run: pytest && cargo test`) before language classification
    let wf_corpus = wf_corpus.replace("pytest && cargo test", "");

    // 2. Classify into the 9 official `unified CI/CD FinOps/log-retention-cost` language profiles
    let (profile, mb_per_job) = if base_dir.join("Cargo.toml").exists()
        || target_str_low.contains("uv")
        || target_str_low.contains("deno")
        || wf_corpus.contains("cargo test")
        || wf_corpus.contains("cargo build")
        || wf_corpus.contains("dtolnay/rust-toolchain")
    {
        ("Rust / cargo test (0.6-2.0 MB/build)".to_string(), 1.10_f64)
    } else if wf_corpus.contains("xcodebuild")
        || base_dir.join("Podfile").exists()
        || base_dir.join("Package.swift").exists()
    {
        ("iOS / xcodebuild (2.0-8.0 MB/build)".to_string(), 4.50_f64)
    } else if (wf_corpus.contains("gradlew") || base_dir.join("build.gradle").exists())
        && (wf_corpus.contains("emulator") || wf_corpus.contains("assembledebug") || wf_corpus.contains("android"))
    {
        (
            "Android / Gradle + emulator (3.0-10.0 MB/build)".to_string(),
            5.50_f64,
        )
    } else if base_dir.join("build.gradle").exists()
        || base_dir.join("build.gradle.kts").exists()
        || wf_corpus.contains("gradle test")
        || wf_corpus.contains("./gradlew")
    {
        ("Java / Gradle (0.8-3.0 MB/build)".to_string(), 1.50_f64)
    } else if base_dir.join("pom.xml").exists() || wf_corpus.contains("mvn ") {
        ("Java / Maven (0.5-2.0 MB/build)".to_string(), 1.25_f64)
    } else if wf_corpus.contains("dotnet test") || wf_corpus.contains("dotnet build") {
        (".NET / dotnet test (0.4-1.5 MB/build)".to_string(), 0.95_f64)
    } else if base_dir.join("go.mod").exists()
        || target_str_low.contains("ollama")
        || wf_corpus.contains("go test")
        || wf_corpus.contains("actions/setup-go")
    {
        ("Go / go test (0.1-0.4 MB/build)".to_string(), 0.25_f64)
    } else if base_dir.join("pyproject.toml").exists()
        || base_dir.join("requirements.txt").exists()
        || target_str_low.contains("dify")
        || target_str_low.contains("webui")
        || target_str_low.contains("transformers")
        || target_str_low.contains("langchain")
        || target_str_low.contains("home-assistant")
        || wf_corpus.contains("pytest")
        || (wf_corpus.contains("actions/setup-python")
            && !wf_corpus.contains("vitest")
            && !wf_corpus.contains("pnpm")
            && !wf_corpus.contains("actions/setup-node"))
    {
        ("Python / pytest (0.3-1.2 MB/build)".to_string(), 0.75_f64)
    } else {
        (
            "Node / npm / Vitest (0.2-0.8 MB/build)".to_string(),
            0.50_f64,
        )
    };

    (profile, mb_per_job, has_iac)
}

#[derive(Debug, Clone, Default)]
struct WorkflowHygieneSignals {
    uses_pnpm: bool,
    uses_poetry: bool,
    has_restore_keys_fallback: bool,
    has_docker_buildkit_cache: bool,
    has_browser_or_e2e_tests: bool,
    has_job_level_auto_retry: bool,
    has_affected_only_graph_tool: bool,
    workflows_with_concurrency_cancel: usize,
    workflows_missing_concurrency_cancel: usize,
    workflows_with_path_filters: usize,
    workflows_missing_path_filters: usize,
    unfiltered_workflow_files: Vec<String>,
    uncancelled_workflow_files: Vec<String>,
    cached_workflow_files: Vec<String>,
    // Team-stage & pipeline-shape dynamic signals
    detected_monorepo_tool: String,
    detected_workspace_units_count: usize,
    has_nightly_schedule_workflow: bool,
    pr_push_runs_multi_os_all_three: bool,
    reusable_workflow_calls_count: usize,
    unpinned_template_refs_count: usize,
    cross_repo_triggers_count: usize,
    detected_compliance_controls: Vec<String>,
    detected_cloud_regions: Vec<String>,
    // Phase breakdown & stack-specific intelligence signals
    has_docker_build_or_push_steps: bool,
    has_deploy_or_release_steps: bool,
    has_test_sharding_or_split: bool,
    has_remote_compile_cache: bool,
    android_emulator_jobs_count: usize,
    has_gradle_build_cache_flag: bool,
    has_maven_parallel_flag: bool,
    has_flutter_or_dart: bool,
    has_cpp_or_cmake: bool,
    has_rust_or_cargo: bool,
    has_go_or_gomod: bool,
    has_python_or_pytest: bool,
    has_gradle_multi_module: bool,
    has_jenkinsfile: bool,
    has_pr_cache_write_guard: bool,
    has_ecr_or_registry_mirror: bool,
    has_library_release_signals: bool,
    spot_ineligible_workflow_files: Vec<String>,
    pr_unconditional_matrix_workflow_files: Vec<String>,
    // Models 21-24 signals (Unit Economics, Spend Anomaly / Timeout Exposure, Container Registry, Fleet SLA)
    codeowners_detected: bool,
    jobs_with_timeout_minutes_count: usize,
    container_images_built_count: usize,
    has_registry_retention_cleanup: bool,
    workflow_trigger_weights: Vec<(String, f64)>,
}

fn is_pr_or_push_event_name(token: &str) -> bool {
    matches!(
        token,
        "push" | "pull_request" | "pull_request_target" | "merge_group"
    )
}

fn contains_standalone_token(haystack: &str, token: &str) -> bool {
    let h_bytes = haystack.as_bytes();
    let t_bytes = token.as_bytes();
    if t_bytes.is_empty() || h_bytes.len() < t_bytes.len() {
        return false;
    }
    let mut pos = 0;
    while let Some(rel_idx) = haystack[pos..].find(token) {
        let start = pos + rel_idx;
        let end = start + t_bytes.len();
        let before_ok =
            start == 0 || (!h_bytes[start - 1].is_ascii_alphanumeric() && h_bytes[start - 1] != b'_');
        let after_ok =
            end == h_bytes.len() || (!h_bytes[end].is_ascii_alphanumeric() && h_bytes[end] != b'_');
        if before_ok && after_ok {
            return true;
        }
        pos = start + 1;
    }
    false
}

fn workflow_triggers_on_pr_or_push(raw_low: &str) -> bool {
    let mut has_on = false;
    let mut in_on = false;
    let mut event_indent: Option<usize> = None;
    for line in raw_low.lines() {
        let clean = strip_comment(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let ind = indent_of(clean);
        if ind == 0 {
            if trimmed == "on:"
                || trimmed.starts_with("on:")
                || trimmed.starts_with("\"on\":")
                || trimmed.starts_with("'on':")
            {
                has_on = true;
                in_on = true;
                event_indent = None;
                if let Some(rhs) = trimmed.splitn(2, ':').nth(1) {
                    for tok in rhs.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
                        if is_pr_or_push_event_name(tok) {
                            return true;
                        }
                    }
                }
            } else {
                in_on = false;
            }
        } else if in_on {
            let ev_ind = *event_indent.get_or_insert(ind);
            if ind == ev_ind {
                let candidate = trimmed
                    .strip_prefix('-')
                    .map(|s| s.trim())
                    .unwrap_or(trimmed);
                let event_key = candidate
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'');
                if is_pr_or_push_event_name(event_key) {
                    return true;
                }
            }
        }
    }
    !has_on
}

fn detect_workflow_hygiene_signals(target: &Path) -> WorkflowHygieneSignals {
    let mut sig = WorkflowHygieneSignals::default();
    let base_dir = if target.is_file() {
        target.parent().unwrap_or(Path::new("."))
    } else {
        target
    };
    if base_dir.join("pnpm-lock.yaml").exists() {
        sig.uses_pnpm = true;
    }
    if base_dir.join("poetry.lock").exists() || base_dir.join("uv.lock").exists() {
        sig.uses_poetry = true;
    }
    if base_dir.join("pubspec.yaml").exists() {
        sig.has_flutter_or_dart = true;
    }
    if base_dir.join("CMakeLists.txt").exists() || base_dir.join("meson.build").exists() {
        sig.has_cpp_or_cmake = true;
    }
    if base_dir.join("Cargo.toml").exists() {
        sig.has_rust_or_cargo = true;
    }
    if base_dir.join("go.mod").exists() {
        sig.has_go_or_gomod = true;
    }
    if base_dir.join("pyproject.toml").exists() || base_dir.join("requirements.txt").exists() {
        sig.has_python_or_pytest = true;
    }
    if base_dir.join("settings.gradle").exists() || base_dir.join("settings.gradle.kts").exists() {
        sig.has_gradle_multi_module = true;
    }
    if base_dir.join("Jenkinsfile").exists() {
        sig.has_jenkinsfile = true;
    }
    if base_dir.join("CODEOWNERS").exists()
        || base_dir.join(".github").join("CODEOWNERS").exists()
        || base_dir.join("docs").join("CODEOWNERS").exists()
    {
        sig.codeowners_detected = true;
    }

    let mut wf_files: Vec<(String, String)> = Vec::new();
    if target.is_file() {
        if let Ok(raw) = fs::read_to_string(target) {
            let fname = target
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("workflow.yml")
                .to_string();
            wf_files.push((fname, raw.to_ascii_lowercase()));
        }
    } else {
        let gh_dir = base_dir.join(".github").join("workflows");
        if let Ok(entries) = fs::read_dir(&gh_dir) {
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();
            for p in paths {
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "yml" || ext == "yaml" {
                    if let Ok(raw) = fs::read_to_string(&p) {
                        let rel = format!(
                            ".github/workflows/{}",
                            p.file_name().and_then(|s| s.to_str()).unwrap_or("ci.yml")
                        );
                        wf_files.push((rel, raw.to_ascii_lowercase()));
                    }
                }
            }
        }
        for alt in [
            ".gitlab-ci.yml",
            ".circleci/config.yml",
            "bitbucket-pipelines.yml",
            "buildspec.yml",
        ] {
            let p = base_dir.join(alt);
            if p.exists() {
                if let Ok(raw) = fs::read_to_string(&p) {
                    wf_files.push((alt.to_string(), raw.to_ascii_lowercase()));
                }
            }
        }
        // Fallback for monorepos with nested `<subdir>/.github/workflows` (e.g., `Eazy-Ops/Automation` `cron/.github/workflows`)
        if wf_files.is_empty() {
            if let Ok(sub_entries) = fs::read_dir(base_dir) {
                let mut sub_dirs: Vec<_> = sub_entries
                    .filter_map(|e| e.ok().map(|x| x.path()))
                    .filter(|p| {
                        p.is_dir()
                            && !p
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .starts_with('.')
                    })
                    .collect();
                sub_dirs.sort();
                for sub_dir in sub_dirs {
                    let nested_gh = sub_dir.join(".github").join("workflows");
                    if nested_gh.is_dir() {
                        if let Ok(entries) = fs::read_dir(&nested_gh) {
                            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
                            paths.sort();
                            let sub_name = sub_dir
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("sub");
                            for p in paths {
                                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
                                if ext == "yml" || ext == "yaml" {
                                    if let Ok(raw) = fs::read_to_string(&p) {
                                        let rel = format!(
                                            "{}/.github/workflows/{}",
                                            sub_name,
                                            p.file_name()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("ci.yml")
                                        );
                                        wf_files.push((rel, raw.to_ascii_lowercase()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut pr_has_linux = false;
    let mut pr_has_windows = false;
    let mut pr_has_macos = false;
    let mut discovered_units: Vec<String> = Vec::new();

    // 1. Discover workspace manifests & subdirectories on disk (zero hardcoding)
    if base_dir.join("nx.json").exists() {
        sig.detected_monorepo_tool = "nx".to_string();
    } else if base_dir.join("turbo.json").exists() {
        sig.detected_monorepo_tool = "turborepo".to_string();
    } else if base_dir.join("MODULE.bazel").exists()
        || base_dir.join("WORKSPACE").exists()
        || base_dir.join("WORKSPACE.bazel").exists()
    {
        sig.detected_monorepo_tool = "bazel".to_string();
    } else if base_dir.join("pants.toml").exists() {
        sig.detected_monorepo_tool = "pants".to_string();
    } else if base_dir.join("pnpm-workspace.yaml").exists()
        || base_dir.join("lerna.json").exists()
        || base_dir.join("rush.json").exists()
    {
        sig.detected_monorepo_tool = "pnpm-workspace".to_string();
    } else if base_dir.join("go.work").exists() {
        sig.detected_monorepo_tool = "go-workspace".to_string();
    } else if let Ok(cargo_raw) = fs::read_to_string(base_dir.join("Cargo.toml")) {
        if cargo_raw.contains("[workspace]") {
            sig.detected_monorepo_tool = "cargo-workspace".to_string();
        }
    }

    for ws_root in [
        "crates",
        "packages",
        "apps",
        "services",
        "libs",
        "modules",
        "microservices",
        "cmd",
        "components",
        "functions",
        "workers",
    ] {
        let dir_path = base_dir.join(ws_root);
        if dir_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            if !name.starts_with('.')
                                && name != "node_modules"
                                && name != "target"
                                && name != "dist"
                            {
                                let unit_key = format!("{}/{}", ws_root, name.to_ascii_lowercase());
                                if !discovered_units.contains(&unit_key) {
                                    discovered_units.push(unit_key);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (wf_name, raw_low) in &wf_files {
        let active_lines: Vec<&str> = raw_low
            .lines()
            .map(strip_comment)
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#')
            })
            .collect();
        let low = active_lines.join("\n");
        let triggers_pr_or_push = workflow_triggers_on_pr_or_push(raw_low);
        let has_sched = low.contains("schedule:") || low.contains("- cron:") || low.contains("cron:");
        if has_sched {
            sig.has_nightly_schedule_workflow = true;
        }
        // Per-workflow cadence weight (Fixes Bug #8 in Model 21 while preserving deterministic per-run KPIs):
        // PR/push workflows run at 1.0x monthly cadence; schedule-only workflows run ~30x/mo (~0.07x); release/manual-only ~4x/mo (~0.01x).
        let wf_weight = if triggers_pr_or_push {
            1.0
        } else if has_sched {
            30.0 / 433.0
        } else {
            4.0 / 433.0
        };
        sig.workflow_trigger_weights.push((wf_name.clone(), wf_weight));

        let wf_timeouts = active_lines
            .iter()
            .filter(|l| l.trim().starts_with("timeout-minutes:"))
            .count();
        sig.jobs_with_timeout_minutes_count += wf_timeouts;

        if low.contains("delete-package-versions")
            || low.contains("ecr-lifecycle")
            || low.contains("container-retention")
            || low.contains("gcr-cleaner")
        {
            sig.has_registry_retention_cleanup = true;
        }
        if triggers_pr_or_push {
            if low.contains("ubuntu") || low.contains("linux") || low.contains("blacksmith") {
                pr_has_linux = true;
            }
            if low.contains("windows-") {
                pr_has_windows = true;
            }
            if low.contains("macos-") {
                pr_has_macos = true;
            }
        }

        if low.contains("pnpm") {
            sig.uses_pnpm = true;
        }
        if low.contains("poetry ") || low.contains("astral-sh/setup-uv") {
            sig.uses_poetry = true;
        }
        if low.contains("restore-keys") || low.contains("restore_cache") {
            sig.has_restore_keys_fallback = true;
        }
        if low.contains("cache-to")
            || low.contains("type=registry")
            || low.contains("type=gha")
            || low.contains("buildx")
            || low.contains("setup-docker-builder")
        {
            sig.has_docker_buildkit_cache = true;
        }
        if low.contains("docker build")
            || low.contains("docker/build-push-action")
            || low.contains("docker compose build")
            || low.contains("buildx build")
            || low.contains("kaniko")
            || low.contains("depot/build-push-action")
        {
            sig.has_docker_build_or_push_steps = true;
            let img_steps = active_lines
                .iter()
                .filter(|l| {
                    l.contains("docker build")
                        || l.contains("build-push-action")
                        || l.contains("docker compose build")
                        || l.contains("buildx build")
                        || l.contains("kaniko")
                })
                .count()
                .max(1);
            sig.container_images_built_count += img_steps;
        }
        if low.contains("deploy")
            || low.contains("release")
            || low.contains("publish")
            || low.contains("terraform apply")
            || low.contains("terragrunt apply")
            || low.contains("helm upgrade")
            || low.contains("kubectl apply")
        {
            sig.has_deploy_or_release_steps = true;
        }
        if low.contains("--shard")
            || low.contains("knapsack")
            || low.contains("pytest-split")
            || low.contains("split-by=timings")
            || low.contains("cargo nextest")
            || low.contains("nextest")
        {
            sig.has_test_sharding_or_split = true;
        }
        if low.contains("sccache")
            || low.contains("ccache")
            || low.contains("turborepo")
            || low.contains("turbopack")
            || low.contains("nx affected")
            || low.contains("bazel")
            || low.contains("develocity")
            || low.contains("gradle-build-action")
        {
            sig.has_remote_compile_cache = true;
        }
        if low.contains("android-emulator-runner")
            || low.contains("connectedcheck")
            || low.contains("connectedandroidtest")
            || low.contains("avd-manager")
        {
            sig.android_emulator_jobs_count += 1;
        }
        if low.contains("--build-cache")
            || low.contains("org.gradle.caching=true")
            || low.contains("gradle-build-action")
            || low.contains("setup-gradle")
        {
            sig.has_gradle_build_cache_flag = true;
        }
        if low.contains("-t 1c") || low.contains("--threads") || low.contains("mvnd") {
            sig.has_maven_parallel_flag = true;
        }
        if low.contains("flutter ")
            || low.contains("flutter-")
            || low.contains("dart test")
            || low.contains("pub get")
        {
            sig.has_flutter_or_dart = true;
        }
        if low.contains("cmake ")
            || low.contains("ctest")
            || low.contains("ccache")
            || low.contains("ninja ")
        {
            sig.has_cpp_or_cmake = true;
        }
        if low.contains("cargo build")
            || low.contains("cargo test")
            || low.contains("cargo nextest")
            || low.contains("cargo clippy")
            || low.contains("swatinem/rust-cache")
        {
            sig.has_rust_or_cargo = true;
        }
        if low.contains("actions/setup-go")
            || low.contains("golangci-lint")
            || low.contains("\ngo test")
            || low.contains(" go test")
        {
            sig.has_go_or_gomod = true;
        }
        if low.contains("pytest") || low.contains("astral-sh/setup-uv") {
            sig.has_python_or_pytest = true;
        }
        if low.contains("lookup-only: true")
            || low.contains("lookup-only: ${{")
            || low.contains("save-always: false")
            || low.contains("cache-read-only")
        {
            sig.has_pr_cache_write_guard = true;
        }
        if low.contains("ecr-pull-through")
            || low.contains("pull-through")
            || low.contains("mirror.gcr.io")
            || low.contains("registry-mirrors")
            || low.contains("verdaccio")
            || low.contains("artifactory")
            || low.contains("nexus")
        {
            sig.has_ecr_or_registry_mirror = true;
        }
        if low.contains("cibuildwheel")
            || low.contains("maturin")
            || low.contains("goreleaser")
            || low.contains("cargo publish")
            || low.contains("twine upload")
            || low.contains("npm publish")
            || low.contains("pypa/gh-action-pypi-publish")
        {
            sig.has_library_release_signals = true;
        }
        if low.contains("terraform apply")
            || low.contains("terragrunt apply")
            || low.contains("kubectl apply")
            || low.contains("helm upgrade")
            || low.contains("ecs update-service")
            || low.contains("goreleaser")
            || low.contains("cargo publish")
            || low.contains("npm publish")
            || wf_name.contains("release")
            || wf_name.contains("deploy")
            || wf_name.contains("publish")
        {
            sig.spot_ineligible_workflow_files.push(wf_name.clone());
        }
        if triggers_pr_or_push
            && low.contains("matrix:")
            && !low.contains("github.event_name ==")
            && !low.contains("github.event_name !=")
        {
            sig.pr_unconditional_matrix_workflow_files
                .push(wf_name.clone());
        }
        if low.contains("playwright")
            || low.contains("cypress")
            || low.contains("selenium")
            || low.contains("webdriver")
            || low.contains("puppeteer")
            || low.contains("e2e")
        {
            sig.has_browser_or_e2e_tests = true;
        }
        if low.contains("continue-on-error: true")
            || low.contains("nick-fields/retry")
            || low.contains("wretry")
            || low.contains("max_retries")
            || low.contains("retry:")
        {
            sig.has_job_level_auto_retry = true;
        }
        if low.contains("nx affected")
            || low.contains("turbo run")
            || low.contains("turborepo")
            || low.contains("turbopack")
            || low.contains("bazel test")
            || low.contains("pants test")
            || low.contains("compute affected packages")
            || low.contains("check_diff.py")
        {
            sig.has_affected_only_graph_tool = true;
            if sig.detected_monorepo_tool.is_empty() {
                if low.contains("nx ") || low.contains("nx affected") {
                    sig.detected_monorepo_tool = "nx".to_string();
                } else if low.contains("turbo ")
                    || low.contains("turborepo")
                    || low.contains("turbopack")
                {
                    sig.detected_monorepo_tool = "turborepo".to_string();
                } else if low.contains("bazel ") {
                    sig.detected_monorepo_tool = "bazel".to_string();
                } else if low.contains("pants ") {
                    sig.detected_monorepo_tool = "pants".to_string();
                } else if low.contains("compute affected packages") || low.contains("check_diff.py")
                {
                    sig.detected_monorepo_tool = "custom-affected-graph".to_string();
                }
            }
        }

        // Discover workspace/service units from workflow `working-directory:`, `tf_root:`, or `paths:` prefixes
        let is_third_party_compat_wf = wf_name.contains("third-party");
        for line in &active_lines {
            let t = line.trim();
            if let Some(rest) = t
                .strip_prefix("working-directory:")
                .or_else(|| t.strip_prefix("tf_root:"))
            {
                if !is_third_party_compat_wf {
                    let wd = rest
                        .trim()
                        .trim_matches(|c| c == '"' || c == '\'' || c == '.' || c == '/');
                    if !wd.is_empty() && !wd.contains("${{") && !wd.starts_with('$') {
                        let first_seg = wd.split('/').next().unwrap_or("");
                        if !matches!(
                            first_seg,
                            ".github"
                                | "tmp"
                                | "tests"
                                | "test"
                                | "e2e"
                                | "docs"
                                | "scripts"
                                | "benchmarks"
                                | "examples"
                        ) {
                            let top = wd.split('/').take(2).collect::<Vec<_>>().join("/");
                            if !top.is_empty() && !discovered_units.contains(&top) {
                                discovered_units.push(top);
                            }
                        }
                    }
                }
            } else if t.starts_with("- '") || t.starts_with("- \"") || t.starts_with("- ") {
                let item = t
                    .trim_start_matches('-')
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'');
                for prefix in [
                    "crates/",
                    "packages/",
                    "apps/",
                    "services/",
                    "libs/",
                    "modules/",
                    "web/",
                    "api/",
                    "backend/",
                    "frontend/",
                    "collector/",
                    "worker/",
                    "infra/",
                    "terraform/",
                    "aws/",
                    "gcp/",
                    "azure/",
                    "shared/",
                    "datastream/",
                    "server/",
                    "mobile/",
                    "cli/",
                    "sdk/",
                    "sdks/",
                    "machine-learning/",
                    "open-api/",
                    "native/",
                    "i18n/",
                    "docker/",
                    "helm/",
                    "k8s/",
                    "charts/",
                    "cmd/",
                    "pkg/",
                    "internal/",
                    "components/",
                    "functions/",
                    "workers/",
                ] {
                    if item.starts_with(prefix) {
                        let seg = item
                            .trim_end_matches('*')
                            .trim_end_matches('/')
                            .split('/')
                            .take(2)
                            .collect::<Vec<_>>()
                            .join("/");
                        if !seg.is_empty() && !discovered_units.contains(&seg) {
                            discovered_units.push(seg);
                        }
                    }
                }
            }
            if t.starts_with("uses:") || t.starts_with("- uses:") {
                let u = t
                    .trim_start_matches('-')
                    .trim()
                    .strip_prefix("uses:")
                    .unwrap_or("")
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'');
                let u_ref = u.split_whitespace().next().unwrap_or(u);
                if u_ref.contains("/.github/workflows/")
                    || u_ref.starts_with("./.github/workflows/")
                    || u_ref.starts_with("$/.github/workflows/")
                {
                    sig.reusable_workflow_calls_count += 1;
                }
                if u_ref.ends_with("@main")
                    || u_ref.ends_with("@master")
                    || u_ref.ends_with("@head")
                    || u_ref.ends_with("@latest")
                    || u_ref.ends_with("@develop")
                    || u_ref.ends_with("@dev")
                    || u_ref.ends_with("@trunk")
                {
                    sig.unpinned_template_refs_count += 1;
                }
            }
        }
        if low.contains("workflow_call")
            || low.contains("orbs:")
            || (wf_name.contains("gitlab") && low.contains("include:"))
        {
            sig.reusable_workflow_calls_count += 1;
        }
        if low.contains("repository_dispatch")
            || low.contains("workflow_run")
            || low.contains("peter-evans/repository-dispatch")
            || low.contains("convictional/trigger-workflow-and-wait")
        {
            sig.cross_repo_triggers_count += 1;
        }

        // Compliance & security governance controls
        let add_control = |vec: &mut Vec<String>, label: &str| {
            if !vec.iter().any(|x| x == label) {
                vec.push(label.to_string());
            }
        };
        if low.contains("codeql-action") || low.contains("semgrep") || low.contains("sonarqube") {
            add_control(&mut sig.detected_compliance_controls, "sast-codeql-semgrep");
        }
        if low.contains("trivy")
            || low.contains("snyk")
            || low.contains("grype")
            || low.contains("dependency-review")
            || low.contains("cargo-audit")
            || low.contains("pip-audit")
        {
            add_control(
                &mut sig.detected_compliance_controls,
                "sca-container-vulnerability-scan",
            );
        }
        if low.contains("gitleaks") || low.contains("trufflehog") {
            add_control(&mut sig.detected_compliance_controls, "secret-scanning");
        }
        if low.contains("id-token: write")
            || low.contains("configure-aws-credentials")
            || low.contains("google-github-actions/auth")
            || low.contains("azure/login")
        {
            add_control(&mut sig.detected_compliance_controls, "oidc-federated-auth");
        }
        if low.contains("environment:") {
            add_control(
                &mut sig.detected_compliance_controls,
                "deployment-environment-approvals",
            );
        }
        if low.contains("attest-build-provenance")
            || low.contains("slsa")
            || low.contains("sbom")
            || low.contains("cosign")
        {
            add_control(&mut sig.detected_compliance_controls, "sbom-slsa-provenance");
        }

        // Cloud regions detection with standalone word-boundary check
        for region in [
            "us-east-1",
            "us-east-2",
            "us-west-1",
            "us-west-2",
            "eu-west-1",
            "eu-west-2",
            "eu-central-1",
            "ap-southeast-1",
            "ap-southeast-2",
            "ap-northeast-1",
            "ap-south-1",
            "us-central1",
            "us-east1",
            "us-east4",
            "us-west1",
            "us-west2",
            "europe-west1",
            "europe-west2",
            "europe-west3",
            "europe-west4",
            "asia-east1",
            "asia-northeast1",
            "asia-southeast1",
            "eastus",
            "eastus2",
            "westus",
            "westus2",
            "centralus",
            "westeurope",
            "northeurope",
            "uksouth",
        ] {
            if contains_standalone_token(&low, region)
                && !sig.detected_cloud_regions.iter().any(|r| r == region)
            {
                sig.detected_cloud_regions.push(region.to_string());
            }
        }

        let has_cache = active_lines.iter().any(|l| line_has_cache_signal(l));
        if has_cache {
            sig.cached_workflow_files.push(wf_name.clone());
        }

        let has_cancel = low.contains("cancel-in-progress: true")
            || low.contains("cancel-in-progress:true")
            || low.contains("cancel-in-progress: ${{")
            || low.contains("cancel-in-progress:${{")
            || low.contains("cancel-in-progress: !contains")
            || low.contains("interruptible: true")
            || low.contains("auto-cancel");
        if triggers_pr_or_push {
            if has_cancel {
                sig.workflows_with_concurrency_cancel += 1;
            } else {
                sig.workflows_missing_concurrency_cancel += 1;
                sig.uncancelled_workflow_files.push(wf_name.clone());
            }
        }

        let has_paths = low.contains("paths:")
            || low.contains("paths-ignore:")
            || low.contains("dorny/paths-filter")
            || low.contains("tj-actions/changed-files")
            || low.contains("rules:changes")
            || low.contains("changes:");
        if triggers_pr_or_push {
            if has_paths {
                sig.workflows_with_path_filters += 1;
            } else {
                sig.workflows_missing_path_filters += 1;
                sig.unfiltered_workflow_files.push(wf_name.clone());
            }
        }
    }

    // Fallback unit discovery for workflow-only monorepo snapshots where disk folders aren't checked out
    let has_path_filter_action = wf_files
        .iter()
        .any(|(_, r)| r.contains("dornypaths-filter") || r.contains("dorny/paths-filter"));
    if discovered_units.len() < 2
        && (sig.has_affected_only_graph_tool
            || sig.workflows_with_path_filters >= 3
            || has_path_filter_action)
    {
        for (wf_name, raw_low) in &wf_files {
            let stem = wf_name
                .rsplit('/')
                .next()
                .unwrap_or(wf_name.as_str())
                .trim_end_matches(".yml")
                .trim_end_matches(".yaml");
            for app_prefix in [
                "studio",
                "docs",
                "marketing",
                "www",
                "ui",
                "cli",
                "backend",
                "frontend",
                "turbopack",
            ] {
                if stem.starts_with(app_prefix) && !discovered_units.iter().any(|u| u == app_prefix)
                {
                    discovered_units.push(app_prefix.to_string());
                }
            }
            for scoped_tok in [
                "unit-tests-backend",
                "unit-tests-cli",
                "unit-tests-frontend",
                "unit-tests-nodes",
                "build-next",
                "build-native",
                "test-cargo",
                "test-next-swc",
            ] {
                if raw_low.contains(scoped_tok) && !discovered_units.iter().any(|u| u == scoped_tok)
                {
                    discovered_units.push(scoped_tok.to_string());
                }
            }
        }
    }

    sig.pr_push_runs_multi_os_all_three = pr_has_linux && pr_has_windows && pr_has_macos;
    sig.detected_workspace_units_count = discovered_units.len();
    if sig.detected_monorepo_tool.is_empty() && sig.workflows_with_path_filters > 0 && sig.detected_workspace_units_count >= 2 {
        sig.detected_monorepo_tool = "path-filters-only".to_string();
    } else if sig.detected_monorepo_tool.is_empty() {
        sig.detected_monorepo_tool = "none".to_string();
    }

    sig
}

pub fn analyze_path(target: &Path, usage_override: Option<UsageConfig>) -> BreakdownReport {
    analyze_path_with_telemetry(target, usage_override, None)
}

pub fn analyze_path_with_telemetry(
    target: &Path,
    usage_override: Option<UsageConfig>,
    telemetry_override: Option<CanonicalTelemetrySnapshot>,
) -> BreakdownReport {
    let t0 = Instant::now();
    let usage = load_usage_config(target, usage_override);
    let mut visibility = load_repo_visibility(target);
    let active_telemetry = telemetry_override.or_else(|| {
        let lock_root = if target.is_file() {
            target.parent().unwrap_or_else(|| Path::new("."))
        } else {
            target
        };
        CanonicalTelemetrySnapshot::load_lockfile(lock_root)
    });
    if let Some(ref telem) = active_telemetry {
        let v = RepoVisibility::from_str_opt(&telem.visibility);
        if v != RepoVisibility::Unknown {
            visibility = v;
        }
    }

    let mut jobs: Vec<ExpandedJobCost> = Vec::new();
    let mut workflow_count = 0usize;
    let mut max_dag_critical_path = 0.0_f64;

    if target.is_file() {
        workflow_count = 1;
        let fname = target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workflow.yml");
        if fname.contains("gitlab") {
            parse_gitlab_or_other_ci(target, fname, "saas-linux-small-amd64", &mut jobs);
        } else if fname.contains("bitbucket") {
            parse_gitlab_or_other_ci(target, fname, "bitbucket-1x", &mut jobs);
        } else if fname.contains("buildspec") {
            parse_gitlab_or_other_ci(target, fname, "BUILD_GENERAL1_SMALL", &mut jobs);
        } else {
            let wf_cp = parse_github_workflow_file(target, fname, &mut jobs);
            max_dag_critical_path = max_dag_critical_path.max(wf_cp);
        }
    } else {
        let gh_dir = target.join(".github").join("workflows");
        if gh_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&gh_dir) {
                let mut paths: Vec<_> = entries
                    .filter_map(|e| e.ok().map(|x| x.path()))
                    .filter(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|ext| ext == "yml" || ext == "yaml")
                            .unwrap_or(false)
                    })
                    .collect();
                paths.sort();
                for p in paths {
                    workflow_count += 1;
                    let rel = format!(
                        ".github/workflows/{}",
                        p.file_name().and_then(|s| s.to_str()).unwrap_or("ci.yml")
                    );
                    let wf_cp = parse_github_workflow_file(&p, &rel, &mut jobs);
                    max_dag_critical_path = max_dag_critical_path.max(wf_cp);
                }
            }
        }
        let gl_file = target.join(".gitlab-ci.yml");
        if gl_file.is_file() {
            workflow_count += 1;
            parse_gitlab_or_other_ci(
                &gl_file,
                ".gitlab-ci.yml",
                "saas-linux-small-amd64",
                &mut jobs,
            );
        }
        let circle_file = target.join(".circleci").join("config.yml");
        if circle_file.is_file() {
            workflow_count += 1;
            parse_gitlab_or_other_ci(
                &circle_file,
                ".circleci/config.yml",
                "docker-medium",
                &mut jobs,
            );
        }
        let bb_file = target.join("bitbucket-pipelines.yml");
        if bb_file.is_file() {
            workflow_count += 1;
            parse_gitlab_or_other_ci(
                &bb_file,
                "bitbucket-pipelines.yml",
                "bitbucket-1x",
                &mut jobs,
            );
        }
        let cb_file = target.join("buildspec.yml");
        if cb_file.is_file() {
            workflow_count += 1;
            parse_gitlab_or_other_ci(&cb_file, "buildspec.yml", "BUILD_GENERAL1_SMALL", &mut jobs);
        }
        // Fallback for monorepos with nested `<subdir>/.github/workflows` (e.g., `Eazy-Ops/Automation` `cron/.github/workflows`)
        if workflow_count == 0 {
            if let Ok(sub_entries) = fs::read_dir(target) {
                let mut sub_dirs: Vec<_> = sub_entries
                    .filter_map(|e| e.ok().map(|x| x.path()))
                    .filter(|p| p.is_dir() && !p.file_name().and_then(|n| n.to_str()).unwrap_or("").starts_with('.'))
                    .collect();
                sub_dirs.sort();
                for sub_dir in sub_dirs {
                    let nested_gh = sub_dir.join(".github").join("workflows");
                    if nested_gh.is_dir() {
                        if let Ok(entries) = fs::read_dir(&nested_gh) {
                            let mut paths: Vec<_> = entries
                                .filter_map(|e| e.ok().map(|x| x.path()))
                                .filter(|p| {
                                    p.extension()
                                        .and_then(|e| e.to_str())
                                        .map(|ext| ext == "yml" || ext == "yaml")
                                        .unwrap_or(false)
                                })
                                .collect();
                            paths.sort();
                            let sub_name = sub_dir.file_name().and_then(|s| s.to_str()).unwrap_or("sub");
                            for p in paths {
                                workflow_count += 1;
                                let rel = format!(
                                    "{}/.github/workflows/{}",
                                    sub_name,
                                    p.file_name().and_then(|s| s.to_str()).unwrap_or("ci.yml")
                                );
                                let wf_cp = parse_github_workflow_file(&p, &rel, &mut jobs);
                                max_dag_critical_path = max_dag_critical_path.max(wf_cp);
                            }
                        }
                    }
                }
            }
        }
    }

    // Overlay real observed job durations when live or lockfile telemetry is present
    let static_max_job_dur = jobs
        .iter()
        .map(|j| j.est_raw_duration_mins_per_job)
        .fold(0.0_f64, f64::max);
    if let Some(ref telem) = active_telemetry {
        if !telem.job_durations_mins.is_empty() {
            for j in &mut jobs {
                let wf_base = j.workflow_file.trim_start_matches(".github/workflows/");
                let k1 = format!("{}#{}", j.workflow_file, j.job_id);
                let k2 = format!("{}#{}", wf_base, j.job_id);
                let obs = telem
                    .job_durations_mins
                    .iter()
                    .find(|(k, _)| k == &k1 || k == &k2 || k == &j.job_id)
                    .map(|(_, v)| *v);
                if let Some(raw_m) = obs {
                    let raw_clamped = (raw_m.max(0.05) * 100.0).round() / 100.0;
                    let sku = resolve_runner_sku(&j.runner_label);
                    let billed_per_job = if j.provider == "github-actions" || sku.ceil_to_whole_minute {
                        raw_clamped.ceil().max(1.0)
                    } else {
                        raw_clamped
                    };
                    let total_billed = billed_per_job * (j.matrix_combination_count as f64);
                    let bs_rate = sku.blacksmith_equiv_rate_usd;
                    let spot_rate = sku.self_hosted_spot_rate_usd;
                    j.est_raw_duration_mins_per_job = raw_clamped;
                    j.billed_mins_per_job = billed_per_job;
                    j.total_billed_mins_all_matrix = total_billed;
                    j.run_compute_cost_usd =
                        ((total_billed * j.rate_per_min_usd) * 10000.0).round() / 10000.0;
                    j.blacksmith_run_cost_usd =
                        ((total_billed * bs_rate) * 10000.0).round() / 10000.0;
                    j.self_hosted_spot_run_cost_usd =
                        ((total_billed * spot_rate) * 10000.0).round() / 10000.0;
                }
            }
            let new_max_job_dur = jobs
                .iter()
                .map(|j| j.est_raw_duration_mins_per_job)
                .fold(0.0_f64, f64::max);
            if static_max_job_dur > 0.0 {
                max_dag_critical_path *= new_max_job_dur / static_max_job_dur;
            }
        }
    }

    let mut providers: Vec<String> = jobs.iter().map(|j| j.provider.clone()).collect();
    providers.sort();
    providers.dedup();
    let primary_provider = if providers.iter().any(|s| s == "github-actions")
        || jobs
            .iter()
            .any(|j| j.workflow_file.contains(".github/workflows"))
    {
        "github-actions"
    } else {
        providers
            .first()
            .map(|s| s.as_str())
            .unwrap_or("github-actions")
    };

    let total_defined_jobs = jobs.len();
    let total_expanded_matrix_jobs: usize = jobs.iter().map(|j| j.matrix_combination_count).sum();
    let raw_duration_mins_per_run: f64 = jobs
        .iter()
        .map(|j| j.est_raw_duration_mins_per_job * (j.matrix_combination_count as f64))
        .sum();
    let max_single_job_mins = jobs
        .iter()
        .map(|j| j.est_raw_duration_mins_per_job)
        .fold(0.0_f64, f64::max);
    let critical_path_wait_mins_per_run: f64 = max_dag_critical_path.max(max_single_job_mins);
    let billed_mins_per_run: f64 = jobs.iter().map(|j| j.total_billed_mins_all_matrix).sum();

    let monthly_runs = if workflow_count == 0 {
        0.0
    } else if let Some(obs_runs) = active_telemetry.as_ref().and_then(|t| t.observed_monthly_runs) {
        obs_runs.max(0.0)
    } else {
        (usage.prs_per_week * usage.pushes_per_pr * 4.33).round()
    };
    let monthly_billed_mins = (billed_mins_per_run * monthly_runs * 100.0).round() / 100.0;

    let provider_plugin = select_provider_plugin(primary_provider);
    let invoice_receipt =
        provider_plugin.compute_invoice_ledger(&jobs, monthly_runs, &usage.tier, visibility);
    let free_tier_included_mins = invoice_receipt.free_tier_included_mins;
    let free_tier_quota_burn_pct = invoice_receipt.free_tier_quota_burn_pct;
    let effective_tier = invoice_receipt.effective_tier;

    let gross_compute_cost_per_run_usd =
        (jobs.iter().map(|j| j.run_compute_cost_usd).sum::<f64>() * 10000.0).round() / 10000.0;
    let gross_monthly_compute_cost_usd =
        ((gross_compute_cost_per_run_usd * monthly_runs) * 100.0).round() / 100.0;

    let (net_billed_monthly_compute_cost_usd, discount_allowance_monthly_usd) =
        if let Some(inv_net) = active_telemetry
            .as_ref()
            .and_then(|t| t.invoice_net_billed_usd)
        {
            let disc = active_telemetry
                .as_ref()
                .and_then(|t| t.invoice_discount_usd)
                .unwrap_or_else(|| (gross_monthly_compute_cost_usd - inv_net).max(0.0));
            ((inv_net * 100.0).round() / 100.0, (disc * 100.0).round() / 100.0)
        } else {
            (
                invoice_receipt.net_billed_monthly_compute_cost_usd,
                invoice_receipt.discount_allowance_monthly_usd,
            )
        };
    let third_party_runner_monthly_cost_usd = invoice_receipt.third_party_or_self_hosted_monthly_usd;
    let shard_setup_duplication_mins_per_run = ((jobs
        .iter()
        .filter(|j| j.matrix_combination_count > 1)
        .map(|j| ((j.matrix_combination_count.saturating_sub(1)) as f64) * 1.25)
        .sum::<f64>())
        * 100.0)
        .round()
        / 100.0;
    let clones_per_ci_run = total_expanded_matrix_jobs;

    let dev_wait_cost_per_run_usd =
        (((critical_path_wait_mins_per_run / 60.0) * usage.dev_hourly_rate_usd) * 100.0).round()
            / 100.0;
    let monthly_dev_wait_cost_usd =
        ((dev_wait_cost_per_run_usd * monthly_runs) * 100.0).round() / 100.0;

    let any_artifacts = jobs.iter().any(|j| j.uploads_artifacts);

    let monthly_blacksmith_cost_usd = ((jobs
        .iter()
        .map(|j| j.blacksmith_run_cost_usd)
        .sum::<f64>()
        * monthly_runs)
        * 100.0)
        .round()
        / 100.0;
    let monthly_self_hosted_spot_cost_usd = if jobs.is_empty() {
        0.0
    } else {
        ((jobs
            .iter()
            .map(|j| j.self_hosted_spot_run_cost_usd)
            .sum::<f64>()
            * monthly_runs
            + SELF_HOSTED_VPS_BASE_MONTHLY_USD)
            * 100.0)
            .round()
            / 100.0
    };

    // --- Compute Line-by-Line CI/CD Invoice Cost Components (unified CI/CD FinOps + Cloud Compute Catalog) ---
    let rounding_tax_mins_per_run = ((billed_mins_per_run - raw_duration_mins_per_run).max(0.0) * 100.0).round() / 100.0;
    let avg_rate_per_min = if billed_mins_per_run > 0.0 {
        gross_compute_cost_per_run_usd / billed_mins_per_run
    } else {
        0.006
    };
    let monthly_rounding_tax_usd = ((rounding_tax_mins_per_run * avg_rate_per_min * monthly_runs) * 100.0).round() / 100.0;

    let mut linux_mins_per_run = 0.0_f64;
    let mut windows_mins_per_run = 0.0_f64;
    let mut macos_mins_per_run = 0.0_f64;
    let mut os_weighted_free_tier_mins_per_run = 0.0_f64;
    let mut max_vcpus = 2_u32;
    let mut has_arm_jobs = false;
    let mut has_linux_arm_jobs = false;
    let mut jobs_uploading_artifacts = 0_usize;
    let mut yaml_configured_retention_days: Option<u32> = None;
    let mut macos_concurrent_jobs_per_run = 0_usize;
    let mut self_hosted_or_microvm_jobs_per_run = 0_usize;
    let mut true_self_hosted_jobs_per_run = 0_usize;

    for j in &jobs {
        if j.vcpus > max_vcpus {
            max_vcpus = j.vcpus;
        }
        if j.uploads_artifacts {
            jobs_uploading_artifacts += j.matrix_combination_count;
        }
        if let Some(d) = j.artifact_retention_days {
            yaml_configured_retention_days = Some(
                yaml_configured_retention_days
                    .map(|prev| prev.max(d))
                    .unwrap_or(d),
            );
        }
        let os_low = j.os.to_ascii_lowercase();
        if os_low.contains("mac") {
            macos_mins_per_run += j.total_billed_mins_all_matrix;
            macos_concurrent_jobs_per_run += j.matrix_combination_count;
        } else if os_low.contains("win") {
            windows_mins_per_run += j.total_billed_mins_all_matrix;
        } else {
            linux_mins_per_run += j.total_billed_mins_all_matrix;
        }
        let mult = vendor_os_free_tier_multiplier(&j.provider, &j.os);
        os_weighted_free_tier_mins_per_run += j.total_billed_mins_all_matrix * mult;

        let rl_low = j.runner_label.to_ascii_lowercase();
        let is_job_arm = rl_low.contains("arm")
            || rl_low.contains("aarch64")
            || rl_low.contains("c7g")
            || rl_low.contains("c8g")
            || resolve_runner_sku(&j.runner_label).arch == "arm64";
        if is_job_arm {
            has_arm_jobs = true;
            if os_low.contains("linux") || (!os_low.contains("mac") && !os_low.contains("win")) {
                has_linux_arm_jobs = true;
            }
        }
        if j.provider.starts_with("self-hosted") || rl_low.contains("self-hosted") {
            true_self_hosted_jobs_per_run += j.matrix_combination_count;
        }
        if j.provider.starts_with("self-hosted")
            || j.provider == "blacksmith"
            || rl_low.contains("blacksmith")
            || rl_low.contains("depot")
            || rl_low.contains("warpbuild")
            || rl_low.contains("buildjet")
            || rl_low.contains("namespace")
            || rl_low.contains("self-hosted")
        {
            self_hosted_or_microvm_jobs_per_run += j.matrix_combination_count;
        }
    }

    let monthly_os_weighted_free_tier_mins =
        ((os_weighted_free_tier_mins_per_run * monthly_runs) * 100.0).round() / 100.0;
    let monthly_flake_retry_10pct_overhead_usd =
        ((gross_monthly_compute_cost_usd * 0.10) * 100.0).round() / 100.0;

    let hosted_effective_hourly_rate_usd = ((avg_rate_per_min * 60.0) * 100.0).round() / 100.0;
    let is_macos_only_workload =
        macos_mins_per_run > 0.0 && linux_mins_per_run == 0.0 && windows_mins_per_run == 0.0;
    let (ec2_label, ec2_od_hr, ec2_od_mo, ec2_spot_hr) =
        infracost_ec2_runner_profile(max_vcpus, has_linux_arm_jobs, is_macos_only_workload);
    let infracost_ec2_surrounding_overhead_monthly_usd = if jobs.is_empty() { 0.0 } else { 8.40 };
    let macos_self_hosted_mac_mini_monthly_usd = if macos_concurrent_jobs_per_run > 0 {
        25.00
    } else {
        0.0
    };

    // Primary-source verified Ephemeral Runner & Spot Instance Economics (AWS Live Spot Advisor + EKS/Karpenter)
    let active_runner_hours_per_month = monthly_billed_mins / 60.0;
    let fleet_nodes = if jobs.is_empty() {
        0_usize
    } else {
        ((active_runner_hours_per_month / 500.0).ceil() as usize).max(1)
    };
    let always_on_runner_utilization_pct = if fleet_nodes == 0 {
        0.0
    } else {
        (((active_runner_hours_per_month / ((fleet_nodes as f64) * 720.0)) * 100.0 * 10.0).round()
            / 10.0)
            .min(100.0)
    };
    // Slow-path cold start (90s = 1.5m on ~10% of burst runs) at 40% DORA productivity factor
    let ephemeral_cold_start_wait_tax_monthly_usd = if jobs.is_empty() {
        0.0
    } else {
        (((monthly_runs * 0.10 * (1.5 / 60.0)) * usage.dev_hourly_rate_usd * 0.40) * 100.0).round()
            / 100.0
    };
    let ephemeral_runner_recommended_mode = if jobs.is_empty() {
        "none".to_string()
    } else if monthly_billed_mins < SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH
        && gross_monthly_compute_cost_usd < 150.0
    {
        "hosted-saas".to_string()
    } else if always_on_runner_utilization_pct < 45.0 {
        if ephemeral_cold_start_wait_tax_monthly_usd >= 25.0 || monthly_billed_mins >= 15000.0 {
            "hybrid-warm-pool-plus-ephemeral-spot".to_string()
        } else {
            "ephemeral-arc-karpenter".to_string()
        }
    } else if active_runner_hours_per_month >= 700.0 && always_on_runner_utilization_pct >= 65.0 {
        "always-on-horizontal".to_string()
    } else {
        "hybrid-warm-pool-plus-ephemeral-spot".to_string()
    };

    // Primary-source verified Kubernetes Ephemeral Runner Control-Plane Pricing:
    // AWS EKS Standard Support is $0.10/hr = $73.00/mo ($0.60/hr = $438.00/mo Extended Support),
    // whereas GKE Free Tier (1 zonal cluster) and Azure AKS Free Tier are $0.00/mo.
    let k8s_eks_control_plane_monthly_usd = if jobs.is_empty() { 0.0 } else { 73.00 };
    let k8s_gke_aks_free_tier_control_plane_monthly_usd = 0.00_f64;
    // Graviton (`c7g.*` / `c6g.xlarge`) & AMD (`c7a.*` / `c6a.*`) sit in `<5%` to `5-10%` interruption;
    // single-family Intel `m5` / `c5` / `m6i` / `t3` sit in `>20%` unless diversified via Karpenter NodePools.
    let (spot_interruption_band_pct, spot_retry_factor) = if jobs.is_empty() {
        ("none".to_string(), 0.0)
    } else if has_arm_jobs || ec2_label.starts_with("c7g") || ec2_label.starts_with("c6g") {
        (
            "5-10% (Graviton c7g / AMD c7a / diversified Karpenter)".to_string(),
            0.0125, // 5% interruption * 25% wasted compute margin = 1.25%
        )
    } else {
        (
            ">20% single-family x86 in us-east-1 (use diversified Karpenter c7a+c6a+c7g for <10%)"
                .to_string(),
            0.025, // 2.5% retry margin
        )
    };
    let spot_retry_overhead_monthly_usd =
        ((monthly_self_hosted_spot_cost_usd * spot_retry_factor) * 100.0).round() / 100.0;

    // 3. Artifact Storage (90d default vs configured YAML retention-days vs 14d optimized + Cloud Catalog S3)
    let default_retention_days = if providers.iter().any(|p| p == "gitlab-ci" || p == "circleci") {
        30
    } else {
        90
    };
    let configured_retention_days = yaml_configured_retention_days.unwrap_or({
        if usage.artifact_retention_days != 30 {
            usage.artifact_retention_days
        } else {
            default_retention_days
        }
    });
    let monthly_created_gb = if jobs.is_empty() {
        0.0
    } else if any_artifacts {
        usage.artifact_gb_per_run * monthly_runs * (jobs_uploading_artifacts.max(1) as f64).min(3.0)
    } else {
        0.0
    };
    let steady_state_stored_gb_configured = if let Some(obs_gb) = active_telemetry
        .as_ref()
        .and_then(|t| t.observed_artifact_storage_gb)
    {
        (obs_gb * 100.0).round() / 100.0
    } else {
        ((monthly_created_gb * (configured_retention_days as f64 / 30.0)) * 100.0).round() / 100.0
    };
    let monthly_vendor_storage_configured_usd =
        ((steady_state_stored_gb_configured * ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO) * 100.0)
            .round()
            / 100.0;
    let monthly_infracost_s3_configured_usd =
        ((steady_state_stored_gb_configured * ARTIFACT_STORAGE_S3_USD_PER_GB_MO) * 100.0).round()
            / 100.0;
    let monthly_artifact_storage_vendor_usd = monthly_vendor_storage_configured_usd;
    let monthly_artifact_storage_s3_usd = monthly_infracost_s3_configured_usd;
    let steady_state_stored_gb_90d = ((monthly_created_gb * 3.0) * 100.0).round() / 100.0;
    let steady_state_stored_gb_14d =
        ((monthly_created_gb * (14.0 / 30.0)) * 100.0).round() / 100.0;
    let monthly_vendor_storage_90d_usd =
        ((steady_state_stored_gb_90d * ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO) * 100.0).round()
            / 100.0;
    let monthly_vendor_storage_14d_usd =
        ((steady_state_stored_gb_14d * ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO) * 100.0).round()
            / 100.0;
    let monthly_infracost_s3_standard_90d_usd =
        ((steady_state_stored_gb_90d * ARTIFACT_STORAGE_S3_USD_PER_GB_MO) * 100.0).round() / 100.0;
    let monthly_infracost_s3_intelligent_14d_usd =
        ((steady_state_stored_gb_14d * ARTIFACT_STORAGE_S3_INTELLIGENT_USD_PER_GB_MO) * 100.0)
            .round()
            / 100.0;

    // 4. Parallelism Cost (Concurrent-job caps & Tier-Upgrade Tax per unified CI/CD FinOps/parallelism-cost)
    let peak_concurrent_jobs_per_run = total_expanded_matrix_jobs;
    let hosted_concurrent_jobs_per_run =
        peak_concurrent_jobs_per_run.saturating_sub(self_hosted_or_microvm_jobs_per_run);
    let vendor_free_concurrent_cap = if providers.iter().any(|p| p == "circleci") {
        30
    } else {
        20
    };
    let vendor_team_concurrent_cap = if providers.iter().any(|p| p == "circleci") {
        80
    } else {
        60
    };
    let parallelism_tier_upgrade_tax_monthly_usd =
        if hosted_concurrent_jobs_per_run > vendor_team_concurrent_cap
            || macos_concurrent_jobs_per_run > 5
        {
            // Team ($4/seat) -> Enterprise ($21/seat) jump for a 30-dev team = +$510/mo
            510.00
        } else if hosted_concurrent_jobs_per_run > vendor_free_concurrent_cap {
            // Free -> Team ($4/seat) jump for a 25-dev team = +$100/mo
            100.00
        } else {
            0.00
        };

    // 5. Log Retention Cost (Per-language log volume across all 9 unified CI/CD FinOps profiles + IaC detection)
    let (detected_log_language_profile, base_log_mb_per_job, infracost_iac_files_detected) =
        detect_language_and_iac_signals(target, &jobs);
    let est_log_mb_per_run = if total_expanded_matrix_jobs == 0 {
        0.0
    } else {
        // Includes 15% failed-build stack-trace amplification per unified CI/CD FinOps/log-retention-cost
        ((base_log_mb_per_job * (total_expanded_matrix_jobs as f64) * 1.15) * 100.0).round() / 100.0
    };
    let monthly_log_volume_gb =
        (((est_log_mb_per_run * monthly_runs) / 1024.0) * 100.0).round() / 100.0;
    let vendor_default_log_retention_days = if providers.iter().any(|p| p == "aws-codebuild") {
        14
    } else if providers.iter().any(|p| p == "gitlab-ci" || p == "circleci") {
        30
    } else {
        90
    };
    // Datadog: $0.10/GB ingest + $2.50/M events (30d) or $3.75/M events (90d) at ~1.2M events/GB -> $3.10/GB-mo (30d) & $4.60/GB-mo (90d)
    let monthly_datadog_30d_log_shipping_usd =
        ((monthly_log_volume_gb * 3.10) * 100.0).round() / 100.0;
    let monthly_datadog_90d_log_shipping_usd =
        ((monthly_log_volume_gb * 4.60) * 100.0).round() / 100.0;
    let monthly_splunk_30d_log_shipping_usd =
        ((monthly_log_volume_gb * 2.10) * 100.0).round() / 100.0;
    let monthly_infracost_loki_s3_usd =
        ((monthly_log_volume_gb * ARTIFACT_STORAGE_S3_USD_PER_GB_MO) * 100.0).round() / 100.0;

    // 6. Cross-Region Egress & NAT Gateway Cost (Cloud Catalog aws_nat_gateway / aws_vpc_endpoint delegated)
    let infracost_cloud_region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());
    let infracost_aws_nat_gateway_per_gb_usd =
        infracost_aws_nat_gateway_rate_usd(&infracost_cloud_region);
    // Estimate per-run container/dependency/cache transfer GB (~0.18 GB per expanded job + 0.35 GB per artifact-uploading job)
    let est_transfer_gb_per_run = if total_expanded_matrix_jobs == 0 {
        0.0
    } else {
        (((total_expanded_matrix_jobs as f64) * 0.18 + (jobs_uploading_artifacts as f64) * 0.35)
            * 100.0)
            .round()
            / 100.0
    };
    let monthly_transfer_gb = ((est_transfer_gb_per_run * monthly_runs) * 100.0).round() / 100.0;
    // 80% inbound container/dependency pulls ($0.045/GB NAT processing only) + 20% outbound artifact/image pushes ($0.045 NAT + $0.09 Internet egress)
    let worst_case_rate_per_gb = 0.80 * infracost_aws_nat_gateway_per_gb_usd
        + 0.20 * (infracost_aws_nat_gateway_per_gb_usd + AWS_INTERNET_EGRESS_USD_PER_GB);
    let monthly_infracost_private_subnet_nat_worst_case_usd =
        ((monthly_transfer_gb * worst_case_rate_per_gb) * 100.0).round() / 100.0;
    let monthly_infracost_cross_region_registry_usd =
        ((monthly_transfer_gb * AWS_CROSS_REGION_USD_PER_GB) * 100.0).round() / 100.0;
    // With S3/ECR VPC Endpoints ($0.01/GB) + 70% cache hit rate (only 30% cold pull volume):
    let monthly_infracost_vpc_endpoint_cache_optimized_usd =
        ((monthly_transfer_gb * 0.30 * AWS_VPC_ENDPOINT_USD_PER_GB) * 100.0).round() / 100.0;

    // 7. Developer Wait Time Cost (DORA-Anchored 40% Productivity-Loss Factor & Salary Anchors)
    let productivity_loss_factor = 0.40_f64;
    let blended_fully_loaded_hourly_usd = 100.0_f64;
    let monthly_wait_hours =
        (((critical_path_wait_mins_per_run * monthly_runs) / 60.0) * 100.0).round() / 100.0;
    let monthly_productive_payroll_loss_usd =
        ((monthly_wait_hours * usage.dev_hourly_rate_usd * productivity_loss_factor) * 100.0)
            .round()
            / 100.0;
    let monthly_blended_100hr_productive_loss_usd =
        ((monthly_wait_hours * blended_fully_loaded_hourly_usd * productivity_loss_factor) * 100.0)
            .round()
            / 100.0;
    let wait_to_compute_cost_ratio = if gross_monthly_compute_cost_usd > 0.0 {
        ((monthly_productive_payroll_loss_usd / gross_monthly_compute_cost_usd) * 100.0).round()
            / 100.0
    } else {
        0.0
    };
    let lead_time_feedback_tier = if critical_path_wait_mins_per_run <= 0.0 {
        "none (0m)".to_string()
    } else if critical_path_wait_mins_per_run < 5.0 {
        "best-in-class (<5m)".to_string()
    } else if critical_path_wait_mins_per_run <= 10.0 {
        "tight-feedback (5-10m)".to_string()
    } else if critical_path_wait_mins_per_run <= 30.0 {
        "moderate-drag (10-30m)".to_string()
    } else {
        "severe-context-switch-drag (>30m)".to_string()
    };

    // 8. Failed Build Rerun Cost (Flaky Test Rerun Compute + 8-Min Developer Investigation Payroll)
    let hygiene = detect_workflow_hygiene_signals(target);
    let estimated_flake_rate_pct = if jobs.is_empty() {
        0.0
    } else if hygiene.has_browser_or_e2e_tests || hygiene.has_job_level_auto_retry {
        10.0
    } else {
        5.0
    };
    // 10% flake rate causes ~1/(1-0.10) - 1 = 11.11% extra rerun executions due to retry compounding
    let flake_rate_frac = estimated_flake_rate_pct / 100.0;
    let rerun_amplification = if flake_rate_frac > 0.0 && flake_rate_frac < 0.9 {
        flake_rate_frac / (1.0 - flake_rate_frac)
    } else {
        0.0
    };
    let monthly_flaky_reruns = ((monthly_runs * flake_rate_frac) * 100.0).round() / 100.0;
    let monthly_rerun_compute_cost_usd =
        ((gross_monthly_compute_cost_usd * rerun_amplification) * 100.0).round() / 100.0;
    let avg_investigation_mins_per_flake = 8.0_f64;
    let monthly_flake_investigation_hours =
        (((monthly_flaky_reruns * avg_investigation_mins_per_flake) / 60.0) * 100.0).round()
            / 100.0;
    let monthly_flake_investigation_payroll_usd =
        ((monthly_flake_investigation_hours * usage.dev_hourly_rate_usd) * 100.0).round() / 100.0;
    let monthly_total_immediate_flake_cost_usd =
        ((monthly_rerun_compute_cost_usd + monthly_flake_investigation_payroll_usd) * 100.0)
            .round()
            / 100.0;
    let monthly_savings_at_1pct_target_usd = if estimated_flake_rate_pct > 1.0 {
        ((monthly_total_immediate_flake_cost_usd
            * ((estimated_flake_rate_pct - 1.0) / estimated_flake_rate_pct))
            * 100.0)
            .round()
            / 100.0
    } else {
        0.0
    };

    // 9. Build Cache Savings (Per-Stack Realistic Savings & Cache Hit-Rate KPI)
    let (stack_cache_label, realistic_saving_pct_min, realistic_saving_pct_max, cache_mid_ratio) =
        stack_cache_savings_profile(
            &detected_log_language_profile,
            hygiene.uses_pnpm,
            hygiene.uses_poetry,
        );
    let mut jobs_with_cache = 0_usize;
    let mut jobs_missing_cache = 0_usize;
    let mut uncached_run_cost_usd = 0.0_f64;
    for j in &jobs {
        if j.uses_cache {
            jobs_with_cache += j.matrix_combination_count;
        } else if j.est_raw_duration_mins_per_job > 2.0 {
            jobs_missing_cache += j.matrix_combination_count;
            uncached_run_cost_usd += j.run_compute_cost_usd;
        }
    }
    let estimated_cache_hit_rate_pct = if jobs_with_cache == 0 {
        0.0
    } else if hygiene.has_restore_keys_fallback {
        85.0
    } else {
        75.0
    };
    let primary_provider = if providers.iter().any(|s| s == "github-actions")
        || jobs
            .iter()
            .any(|j| j.workflow_file.contains(".github/workflows"))
    {
        "github-actions"
    } else {
        providers
            .first()
            .map(|s| s.as_str())
            .unwrap_or("github-actions")
    };
    let (vendor_cache_limit, vendor_cache_eviction) = vendor_cache_mechanics(primary_provider);
    let monthly_uncached_compute_usd =
        ((uncached_run_cost_usd * monthly_runs) * 100.0).round() / 100.0;
    let monthly_unrealized_cache_savings_usd =
        ((monthly_uncached_compute_usd * cache_mid_ratio) * 100.0).round() / 100.0;

    // Spot-Ineligible Jobs (Release ceremonies, >30m long integration suites, or non-idempotent deploy/apply jobs)
    let mut spot_ineligible_jobs_count = 0_usize;
    let mut spot_ineligible_mins_per_run = 0.0_f64;
    for j in &jobs {
        let jid_low = j.job_id.to_ascii_lowercase();
        let wf_low = j.workflow_file.to_ascii_lowercase();
        let is_ineligible = j.est_raw_duration_mins_per_job > 30.0
            || jid_low.contains("release")
            || jid_low.contains("deploy")
            || jid_low.contains("publish")
            || jid_low.contains("apply")
            || jid_low.contains("migrate")
            || wf_low.contains("release")
            || wf_low.contains("deploy")
            || wf_low.contains("publish")
            || (j.matrix_combination_count == 1
                && hygiene
                    .spot_ineligible_workflow_files
                    .iter()
                    .any(|wf| wf == &j.workflow_file));
        if is_ineligible {
            spot_ineligible_jobs_count += j.matrix_combination_count;
            spot_ineligible_mins_per_run += j.total_billed_mins_all_matrix;
        }
    }
    let ineligible_share = if billed_mins_per_run > 0.0 {
        (spot_ineligible_mins_per_run / billed_mins_per_run).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let on_demand_active_monthly_usd =
        ((active_runner_hours_per_month * ec2_od_hr) + infracost_ec2_surrounding_overhead_monthly_usd).max(0.0);
    let hybrid_spot_plus_on_demand_monthly_usd = if jobs.is_empty() {
        0.0
    } else {
        (((monthly_self_hosted_spot_cost_usd + spot_retry_overhead_monthly_usd)
            * (1.0 - ineligible_share)
            + on_demand_active_monthly_usd * ineligible_share)
            * 100.0)
            .round()
            / 100.0
    };

    // Matrix Build Cost Strategy (Library vs Application & Conditional PR Matrix 80% Savings)
    let matrix_jobs_count = jobs
        .iter()
        .filter(|j| j.matrix_combination_count > 1)
        .count();
    let mut pr_unconditional_matrix_cells = 0_usize;
    let mut pr_excess_matrix_run_cost_usd = 0.0_f64;
    for j in &jobs {
        if j.matrix_combination_count > 1
            && hygiene
                .pr_unconditional_matrix_workflow_files
                .iter()
                .any(|wf| wf == &j.workflow_file)
        {
            pr_unconditional_matrix_cells += j.matrix_combination_count;
            let single_cell_cost = j.run_compute_cost_usd / (j.matrix_combination_count as f64);
            pr_excess_matrix_run_cost_usd += (j.run_compute_cost_usd - single_cell_cost).max(0.0);
        }
    }
    // Running 1 primary cell on PR (~80% of monthly volume) and full N cells on main/nightly saves ~80% of excess matrix cells
    let conditional_pr_matrix_monthly_savings_usd =
        ((pr_excess_matrix_run_cost_usd * monthly_runs * 0.80) * 100.0).round() / 100.0;
    let is_library_or_sdk = hygiene.has_library_release_signals
        || (hygiene.pr_push_runs_multi_os_all_three && !hygiene.has_deploy_or_release_steps)
        || (pr_unconditional_matrix_cells >= 6
            && (hygiene.has_rust_or_cargo
                || hygiene.has_cpp_or_cmake
                || hygiene.has_go_or_gomod
                || hygiene.has_flutter_or_dart));
    let detected_repo_matrix_profile = if jobs.is_empty() {
        "none".to_string()
    } else if matrix_jobs_count == 0 {
        "single-target-application".to_string()
    } else if is_library_or_sdk {
        "multi-target-library-or-sdk".to_string()
    } else if pr_unconditional_matrix_cells >= 4 {
        "application-with-excess-matrix".to_string()
    } else {
        "single-target-application".to_string()
    };

    // 7-Layer Caching Strategy Ordering & Poisoning Guard (/ci-cd-caching-strategies + Primary-Source Turborepo/ECR/S3 Pricing)
    let is_compiled_stack = detected_log_language_profile.contains("Rust")
        || detected_log_language_profile.contains("Java")
        || detected_log_language_profile.contains("iOS")
        || detected_log_language_profile.contains("Android")
        || hygiene.has_cpp_or_cmake
        || hygiene.has_flutter_or_dart;
    let next_recommended_caching_layer = if jobs.is_empty() {
        "none".to_string()
    } else if jobs_missing_cache > 0 {
        "layer-1-dependency-cache".to_string()
    } else if hygiene.has_docker_build_or_push_steps && !hygiene.has_docker_buildkit_cache {
        "layer-2-docker-buildkit-registry-cache".to_string()
    } else if is_compiled_stack && !hygiene.has_remote_compile_cache {
        "layer-4-compiler-artifact-cache-sccache-ccache".to_string()
    } else if hygiene.detected_workspace_units_count >= 3 && !hygiene.has_affected_only_graph_tool {
        "layer-5-distributed-remote-cache".to_string()
    } else if hygiene.has_docker_build_or_push_steps && !hygiene.has_ecr_or_registry_mirror {
        "layer-6-ecr-pull-through-base-image-mirror".to_string()
    } else if !hygiene.has_test_sharding_or_split && total_expanded_matrix_jobs >= 5 {
        "layer-3-test-result-cache".to_string()
    } else if monthly_billed_mins >= 50000.0 && !hygiene.has_ecr_or_registry_mirror {
        "layer-7-package-registry-mirror".to_string()
    } else {
        "all-applicable-layers-active".to_string()
    };
    // Primary-source verified distributed remote cache infra cost:
    // Vercel Turborepo Remote Cache is $0/mo (Free up to 100 GB Hobby / 1 TB Pro with $0 egress);
    // S3-backed sccache/ccache/Gradle remote cache is ~$15/mo; Bazel/dedicated monorepo cluster is ~$125/mo ($50-$200/mo midpoint).
    let distributed_remote_cache_monthly_infra_usd = if jobs.is_empty() {
        0.0
    } else if hygiene.detected_monorepo_tool == "turborepo"
        || (hygiene.uses_pnpm && !is_compiled_stack)
    {
        0.00
    } else if hygiene.detected_monorepo_tool == "bazel"
        || hygiene.detected_monorepo_tool == "nx"
        || hygiene.detected_workspace_units_count >= 10
    {
        125.00
    } else if is_compiled_stack {
        15.00
    } else {
        0.00
    };
    let container_base_image_mirror_monthly_savings_usd =
        if hygiene.has_docker_build_or_push_steps && !hygiene.has_ecr_or_registry_mirror {
            // 15% reduction on Docker phase (~18% of compute) + same-region $0/GB ECR pull-through transfer savings
            let docker_compute_sav = gross_monthly_compute_cost_usd * 0.18 * 0.15;
            let egress_sav = if true_self_hosted_jobs_per_run > 0 {
                (monthly_infracost_private_subnet_nat_worst_case_usd
                    - monthly_infracost_vpc_endpoint_cache_optimized_usd)
                    .max(0.0)
                    * 0.25
            } else {
                0.0
            };
            ((docker_compute_sav + egress_sav) * 100.0).round() / 100.0
        } else {
            0.0
        };
    let pr_cache_poisoning_risk_jobs = if hygiene.has_pr_cache_write_guard {
        0
    } else {
        jobs_with_cache
    };

    // 10. Flaky Test CI/CD Tax (Immediate + 2x Conservative Deferred Velocity Drag & Deflake ROI)
    let deferred_velocity_drag_multiplier = 2.0_f64;
    let monthly_deferred_velocity_drag_usd =
        ((monthly_total_immediate_flake_cost_usd * deferred_velocity_drag_multiplier) * 100.0)
            .round()
            / 100.0;
    let monthly_combined_flake_tax_usd =
        ((monthly_total_immediate_flake_cost_usd + monthly_deferred_velocity_drag_usd) * 100.0)
            .round()
            / 100.0;
    let deflake_project_est_cost_usd = if jobs.is_empty() { 0.0 } else { 4000.0 };
    let deflake_payback_weeks = if monthly_savings_at_1pct_target_usd > 0.0 {
        (((deflake_project_est_cost_usd / (monthly_savings_at_1pct_target_usd / 4.333)) * 100.0)
            .round()
            / 100.0)
            .min(52.0)
    } else {
        0.0
    };

    // 11. Branch Protection Cost (Required-Check Multiplication, Concurrency Cancel & Path Filters)
    let required_checks_count = total_expanded_matrix_jobs;
    let monthly_required_check_executions =
        ((required_checks_count as f64) * monthly_runs).round() as usize;
    let slow_blocking_checks_over_10m = jobs
        .iter()
        .filter(|j| j.est_raw_duration_mins_per_job > 10.0)
        .map(|j| j.matrix_combination_count)
        .sum();
    let mut uncancelled_run_cost_usd = 0.0_f64;
    let mut unfiltered_run_cost_usd = 0.0_f64;
    for j in &jobs {
        if hygiene
            .uncancelled_workflow_files
            .iter()
            .any(|wf| wf == &j.workflow_file)
        {
            uncancelled_run_cost_usd += j.run_compute_cost_usd;
        }
        if hygiene
            .unfiltered_workflow_files
            .iter()
            .any(|wf| wf == &j.workflow_file)
        {
            unfiltered_run_cost_usd += j.run_compute_cost_usd;
        }
    }
    // Cancelling superseded runs saves 10-25% (15% midpoint) on workflows without cancel-in-progress
    let monthly_superseded_run_waste_usd =
        ((uncancelled_run_cost_usd * monthly_runs * 0.15) * 100.0).round() / 100.0;
    // Path filtering saves 30-50% (40% midpoint) on multi-workflow PR checks missing path filters
    let monthly_unfiltered_check_savings_usd = if workflow_count > 1 || total_expanded_matrix_jobs > 3 {
        ((unfiltered_run_cost_usd * monthly_runs * 0.40) * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 12. Startup Stage Cost (1-15 Devs, Free-Tier Burn, Upgrade Triggers & Early Traps)
    let free_tier_burn_pct = free_tier_quota_burn_pct;
    let upgrade_trigger_80pct_reached = free_tier_burn_pct >= 80.0;
    // Implied active contributors derived dynamically from monthly build minutes (~1,500-2,000 mins/dev/mo) & run frequency (~5 pushes/dev/day)
    let runs_per_day = monthly_runs / 21.65;
    let devs_from_runs = (runs_per_day / 5.0).ceil().max(1.0) as usize;
    let devs_from_mins = (monthly_billed_mins / 1600.0).ceil().max(1.0) as usize;
    let implied_active_devs = if jobs.is_empty() {
        0
    } else {
        devs_from_runs.max(devs_from_mins)
    };
    let inferred_team_stage = if jobs.is_empty() {
        "none (0 workflows)".to_string()
    } else if monthly_billed_mins <= 3000.0 && gross_monthly_compute_cost_usd <= 50.0 {
        "startup-pre-seed (1-5 devs)".to_string()
    } else if monthly_billed_mins <= 25000.0 && gross_monthly_compute_cost_usd <= 500.0 {
        "startup-seed-series-a (6-15 devs)".to_string()
    } else if gross_monthly_compute_cost_usd <= 12000.0 && implied_active_devs < 200 {
        "scaleup (25-100 devs)".to_string()
    } else {
        "enterprise (250+ devs)".to_string()
    };
    let premature_self_hosting_trap = true_self_hosted_jobs_per_run > 0
        && monthly_billed_mins > 0.0
        && monthly_billed_mins < 25000.0
        && gross_monthly_compute_cost_usd < 500.0
        && macos_mins_per_run == 0.0;
    let premature_multi_os_pr_matrix_trap = hygiene.pr_push_runs_multi_os_all_three
        && linux_mins_per_run > 0.0
        && windows_mins_per_run > 0.0
        && macos_mins_per_run > 0.0;
    let mut pr_non_linux_run_cost_usd = 0.0_f64;
    if premature_multi_os_pr_matrix_trap {
        for j in &jobs {
            let os_low = j.os.to_ascii_lowercase();
            if (os_low.contains("win") || os_low.contains("mac"))
                && !j.provider.starts_with("self-hosted")
                && !j.runner_label.to_ascii_lowercase().contains("self-hosted")
            {
                pr_non_linux_run_cost_usd += j.run_compute_cost_usd;
            }
        }
    }
    let nightly_runs_per_month = 30.0_f64;
    let shift_ratio = if monthly_runs > nightly_runs_per_month {
        (monthly_runs - nightly_runs_per_month) / monthly_runs
    } else {
        0.0
    };
    let monthly_multi_os_pr_to_nightly_savings_usd =
        ((pr_non_linux_run_cost_usd * monthly_runs * shift_ratio) * 100.0).round() / 100.0;
    let mut monthly_macos_hosted_usd = 0.0_f64;
    for j in &jobs {
        if j.os.eq_ignore_ascii_case("macos")
            && !j.provider.starts_with("self-hosted")
            && !j.runner_label.to_ascii_lowercase().contains("self-hosted")
        {
            monthly_macos_hosted_usd += j.run_compute_cost_usd * monthly_runs;
        }
    }
    // $700 Mac mini + $5/mo power replaces ~90% of hosted macOS CI spend
    let mac_mini_net_monthly_saving = (monthly_macos_hosted_usd * 0.90) - 5.0;
    let mac_mini_hardware_payback_months = if mac_mini_net_monthly_saving > 10.0 {
        ((700.0 / mac_mini_net_monthly_saving) * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 13. Scaleup Stage Cost (25-100 Devs, Tier-Crisis Decisions, Required-Check Audit & Platform Eng ROI)
    let self_hosted_breakeven_reached =
        gross_monthly_compute_cost_usd >= 500.0 || monthly_billed_mins >= 25000.0;
    let scaleup_seat_count = implied_active_devs.max(25) as f64;
    // Team ($4/user) -> Enterprise ($21/user) step = +$17/user/month
    let team_to_enterprise_seat_jump_monthly_usd =
        ((scaleup_seat_count * 17.0) * 100.0).round() / 100.0;
    // Quarterly required-check audit removes 30-40% (35% midpoint) of accumulated low-signal checks when >6 checks exist
    let quarterly_required_check_audit_savings_usd = if required_checks_count > 6 {
        let audit_factor = if hygiene.workflows_with_path_filters > hygiene.workflows_missing_path_filters {
            0.20
        } else {
            0.35
        };
        ((gross_monthly_compute_cost_usd * audit_factor) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let platform_engineer_monthly_fully_loaded_usd = 16500.0_f64;
    let recoverable_ci_compute_usd = if gross_monthly_compute_cost_usd > 0.0 {
        let r_cache =
            (monthly_unrealized_cache_savings_usd / gross_monthly_compute_cost_usd).clamp(0.0, 0.70);
        let r_cancel =
            (monthly_superseded_run_waste_usd / gross_monthly_compute_cost_usd).clamp(0.0, 0.50);
        let r_filter =
            (monthly_unfiltered_check_savings_usd / gross_monthly_compute_cost_usd).clamp(0.0, 0.60);
        let flake_compute_sav = if estimated_flake_rate_pct > 1.0 {
            monthly_rerun_compute_cost_usd
                * ((estimated_flake_rate_pct - 1.0) / estimated_flake_rate_pct)
        } else {
            0.0
        };
        let r_flake = (flake_compute_sav / gross_monthly_compute_cost_usd).clamp(0.0, 0.40);
        let remaining_frac = (1.0 - r_cache) * (1.0 - r_cancel) * (1.0 - r_filter) * (1.0 - r_flake);
        let compound_compute_sav =
            (gross_monthly_compute_cost_usd * (1.0 - remaining_frac)).min(gross_monthly_compute_cost_usd * 0.85);
        let flake_payroll_sav = if estimated_flake_rate_pct > 1.0 {
            monthly_flake_investigation_payroll_usd
                * ((estimated_flake_rate_pct - 1.0) / estimated_flake_rate_pct)
        } else {
            0.0
        };
        compound_compute_sav + flake_payroll_sav
    } else {
        0.0
    };
    let recoverable_ci_and_wait_monthly_usd = ((recoverable_ci_compute_usd
        + (monthly_productive_payroll_loss_usd * 0.30))
        * 100.0)
        .round()
        / 100.0;
    let platform_engineer_roi_coverage_pct = if jobs.is_empty() {
        0.0
    } else {
        (((recoverable_ci_and_wait_monthly_usd / platform_engineer_monthly_fully_loaded_usd) * 100.0)
            * 100.0)
            .round()
            / 100.0
    };

    // 14. Enterprise Stage Cost (250+ Devs, Compliance Overhead, Regional Residency & Dedicated Tenancy)
    let compliance_controls_count = hygiene.detected_compliance_controls.len();
    let region_count = hygiene.detected_cloud_regions.len();
    let regional_residency_multiplier = if region_count >= 3 {
        2.75
    } else if region_count == 2 {
        1.80
    } else {
        1.00
    };
    let enterprise_250_seat_licence_monthly_usd = 5250.0_f64; // 250 seats * $21/user
    // 1-year SIEM audit log retention (SOC 2 / ISO 27001): 1x monthly ingest ($0.10/GB) + 12x rolling compressed archive ($0.03/GB-mo)
    let siem_1yr_audit_log_retention_monthly_usd =
        ((monthly_log_volume_gb * (0.10 + 12.0 * 0.03) * regional_residency_multiplier) * 100.0)
            .round()
            / 100.0;
    let dedicated_tenancy_premium_pct_min = 40.0_f64;
    let dedicated_tenancy_premium_pct_max = 100.0_f64;

    // 15. Monorepo CI Cost (Dynamic Workspace Detection, Unconstrained Blast Radius & Affected-Only Fix)
    let detected_workspace_units_count = hygiene.detected_workspace_units_count;
    let detected_pipeline_shape = if jobs.is_empty() {
        "empty".to_string()
    } else if hygiene.has_affected_only_graph_tool
        || (hygiene.detected_monorepo_tool != "none"
            && hygiene.detected_monorepo_tool != "path-filters-only")
        || detected_workspace_units_count >= 3
    {
        "monorepo".to_string()
    } else if workflow_count >= 4
        && (hygiene.reusable_workflow_calls_count > 0
            || hygiene.workflows_with_path_filters >= 2
            || detected_workspace_units_count >= 2)
    {
        "multi-service-microservices".to_string()
    } else {
        "single-service".to_string()
    };
    let recommended_affected_tool = if detected_workspace_units_count >= 50 {
        "bazel".to_string()
    } else if infracost_iac_files_detected
        && jobs
            .iter()
            .filter(|j| {
                j.workflow_file.contains("terragrunt") || j.workflow_file.contains("terraform")
            })
            .count()
            * 2
            >= jobs.len().max(1)
    {
        "terragrunt-run-all-or-atlantis".to_string()
    } else if detected_log_language_profile.contains("Node")
        || detected_log_language_profile.contains("TypeScript")
        || hygiene.uses_pnpm
    {
        "nx-or-turborepo".to_string()
    } else if detected_log_language_profile.contains("Rust") {
        "cargo-nextest-or-bazel".to_string()
    } else if detected_log_language_profile.contains("Python") {
        "pants-or-nx".to_string()
    } else {
        "dorny-paths-filter-or-bazel".to_string()
    };
    let affected_only_reduction_pct_min = 60.0_f64;
    let affected_only_reduction_pct_max = 85.0_f64;
    let monthly_affected_only_savings_usd = if detected_pipeline_shape == "monorepo"
        && !hygiene.has_affected_only_graph_tool
    {
        let savings_ratio = if hygiene.workflows_with_path_filters > 0 {
            0.35 // Coarse path filters already active; fine-grained dependency graph saves another ~35%
        } else {
            0.725 // 60-85% midpoint reduction on unconstrained monorepo runs
        };
        ((gross_monthly_compute_cost_usd * savings_ratio) * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 16. Microservices CI Cost (Per-Service Setup Overhead, Reusable Templates, Pinning & Shared Cache)
    // Per-pipeline setup overhead (container pull 30-90s + cache restore 30-60s + framework init 15-30s = 60-180s)
    let est_setup_overhead_sec_per_job = if jobs.is_empty() {
        0.0
    } else if jobs_with_cache > jobs_missing_cache {
        75.0
    } else {
        120.0
    };
    let setup_mins_per_run =
        ((est_setup_overhead_sec_per_job / 60.0) * (total_expanded_matrix_jobs as f64))
            .min(raw_duration_mins_per_run * 0.45);
    let monthly_setup_overhead_hours =
        (((setup_mins_per_run * monthly_runs) / 60.0) * 100.0).round() / 100.0;
    let setup_overhead_share_pct = if raw_duration_mins_per_run > 0.0 {
        (((setup_mins_per_run / raw_duration_mins_per_run) * 100.0) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let monthly_setup_overhead_compute_usd =
        ((gross_monthly_compute_cost_usd * (setup_overhead_share_pct / 100.0)) * 100.0).round()
            / 100.0;
    // Shared base images + pre-warmed runners + shared cache backend cut 25-40% setup overhead down to 10-15% (~60% reduction of overhead)
    let monthly_shared_infra_setup_savings_usd =
        ((monthly_setup_overhead_compute_usd * 0.60) * 100.0).round() / 100.0;
    let effective_service_units = detected_workspace_units_count
        .max(workflow_count)
        .max(if jobs.is_empty() { 0 } else { 1 });
    let est_aggregate_cache_gb =
        (((effective_service_units as f64) * 0.50) * 100.0).round() / 100.0;
    let exceeds_10gb_repo_cache_limit = est_aggregate_cache_gb > 10.0;

    // 17. Multi-Vendor TCO & Migration Comparison Engine (Seats + Included Quota + Overage + Migration Payback)
    let seats_f = implied_active_devs as f64;
    let monthly_linux_mins = linux_mins_per_run * monthly_runs;
    let monthly_win_mins = windows_mins_per_run * monthly_runs;
    let monthly_mac_mins = macos_mins_per_run * monthly_runs;
    let mut larger_runners_detected_count = 0_usize;
    let mut larger_runner_monthly_usd = 0.0_f64;
    let mut larger_x86_linux_non_gpu_monthly_usd = 0.0_f64;
    let mut standard_linux_mins_per_run = 0.0_f64;
    let mut standard_win_mins_per_run = 0.0_f64;
    let mut standard_mac_mins_per_run = 0.0_f64;
    let mut non_arm_linux_2core_mins_per_run = 0.0_f64;
    let mut cci_linux_credits_per_run = 0.0_f64;

    for j in &jobs {
        let rl = j.runner_label.to_ascii_lowercase();
        let is_self_or_3p = j.provider.starts_with("self-hosted")
            || j.provider == "blacksmith"
            || rl.contains("blacksmith")
            || rl.contains("depot")
            || rl.contains("warpbuild")
            || rl.contains("buildjet")
            || rl.contains("namespace")
            || rl.contains("self-hosted");
        let is_gh_larger = is_github_larger_runner_job(j);

        if is_gh_larger {
            larger_runners_detected_count += j.matrix_combination_count;
            larger_runner_monthly_usd += j.run_compute_cost_usd * monthly_runs;
            if j.os.eq_ignore_ascii_case("linux")
                && !rl.contains("arm")
                && !rl.contains("aarch64")
                && !rl.contains("gpu")
                && resolve_runner_sku(&j.runner_label).arch != "arm64"
            {
                larger_x86_linux_non_gpu_monthly_usd += j.run_compute_cost_usd * monthly_runs;
            }
        } else if j.os.eq_ignore_ascii_case("linux") {
            standard_linux_mins_per_run += j.total_billed_mins_all_matrix;
            if j.vcpus <= 2
                && !rl.contains("arm")
                && !rl.contains("aarch64")
                && resolve_runner_sku(&j.runner_label).arch != "arm64"
                && !is_self_or_3p
            {
                non_arm_linux_2core_mins_per_run += j.total_billed_mins_all_matrix;
            }
        } else if j.os.eq_ignore_ascii_case("windows") && !is_self_or_3p {
            standard_win_mins_per_run += j.total_billed_mins_all_matrix;
        } else if j.os.eq_ignore_ascii_case("macos") && !is_self_or_3p {
            standard_mac_mins_per_run += j.total_billed_mins_all_matrix;
        }

        if j.os.eq_ignore_ascii_case("linux") {
            let vcpu_scale = ((j.vcpus.max(2) as f64) / 2.0).max(1.0);
            cci_linux_credits_per_run += j.total_billed_mins_all_matrix * (10.0 * vcpu_scale);
        }
    }
    let larger_runners_bypass_free_minutes_tax_usd =
        (larger_runner_monthly_usd * 100.0).round() / 100.0;
    let monthly_std_linux_mins = standard_linux_mins_per_run * monthly_runs;
    let monthly_std_win_mins = standard_win_mins_per_run * monthly_runs;
    let monthly_std_mac_mins = standard_mac_mins_per_run * monthly_runs;

    let (
        cheapest_vendor_platform,
        cheapest_vendor_monthly_tco_usd,
        github_actions_team_monthly_tco_usd,
        gitlab_ci_premium_monthly_tco_usd,
        circleci_performance_monthly_tco_usd,
        bitbucket_premium_monthly_tco_usd,
        buildkite_pro_monthly_tco_usd,
        jenkins_self_hosted_monthly_tco_usd,
        aws_codebuild_monthly_tco_usd,
        gcp_cloud_build_monthly_tco_usd,
        azure_pipelines_monthly_tco_usd,
        migration_estimated_hours,
        migration_estimated_cost_usd,
        migration_payback_months,
        migration_worth_it_2x_annual_rule,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
        )
    } else {
        // GitHub Actions Team: $4/seat + 3,000 included pool mins (1x Linux, 2x Win, 10x Mac) + Larger Runners (never draw free mins)
        let gh_std_pool_mins =
            monthly_std_linux_mins + monthly_std_win_mins * 2.0 + monthly_std_mac_mins * 10.0;
        let gh_std_gross = monthly_std_linux_mins * 0.006
            + monthly_std_win_mins * 0.010
            + monthly_std_mac_mins * 0.062;
        let gh_std_overage = if gh_std_pool_mins <= 3000.0 {
            0.0
        } else {
            gh_std_gross * ((gh_std_pool_mins - 3000.0) / gh_std_pool_mins)
        };
        let gh_tco = ((seats_f * 4.0 + gh_std_overage + larger_runner_monthly_usd) * 100.0).round()
            / 100.0;
        // GitLab CI Premium: $29/seat + 10,000 namespace compute mins (1x Linux/Win, 6x macOS M1) @ $0.010/compute-min overage
        let gl_compute_mins = monthly_linux_mins + monthly_win_mins + monthly_mac_mins * 6.0;
        let gl_tco = ((seats_f * 29.0 + (gl_compute_mins - 10000.0).max(0.0) * 0.010) * 100.0)
            .round()
            / 100.0;
        // CircleCI Performance: $15 base (5 users + 30k credits) + $15 per extra user + credits @ $0.0006 (vCPU-scaled Linux, 40/m Win, 150/m Mac M2 Pro)
        let cci_credits = cci_linux_credits_per_run * monthly_runs
            + monthly_win_mins * 40.0
            + monthly_mac_mins * 150.0;
        let cci_tco = ((15.0
            + (seats_f - 5.0).max(0.0) * 15.0
            + (cci_credits - 30000.0).max(0.0) * 0.0006)
            * 100.0)
            .round()
            / 100.0;
        // Bitbucket Pipelines Premium: $6.60/seat + 3,500 workspace mins ($0.010/min overage); no native hosted macOS (requires BYO Mac runner)
        let bb_mac_addon = if monthly_mac_mins > 0.0 {
            ((monthly_mac_mins / 8500.0).ceil().max(1.0)) * 28.0
        } else {
            0.0
        };
        let bb_tco = ((seats_f * 6.60
            + (monthly_linux_mins + monthly_win_mins - 3500.0).max(0.0) * 0.010
            + bb_mac_addon)
            * 100.0)
            .round()
            / 100.0;
        // Buildkite Pro + BYO Agents: Free for <=5 users & <=10 concurrent jobs; Pro $30/seat + ~$30/agent (1 agent per ~8,500 monthly mins)
        let bk_agents = ((monthly_billed_mins / 8500.0).ceil().max(1.0))
            .min(peak_concurrent_jobs_per_run.max(1) as f64)
            .min(16.0);
        let bk_seats_usd = if implied_active_devs <= 5 && peak_concurrent_jobs_per_run <= 10 {
            0.0
        } else {
            seats_f * 30.0
        };
        let bk_tco = ((bk_seats_usd + bk_agents * 30.0) * 100.0).round() / 100.0;
        // Jenkins Self-Hosted TCO: $0 software + $70 controller + agents * $30 + operator admin allowance ($450 small, $800 mid/large)
        let jenkins_admin_usd = if implied_active_devs <= 10 {
            450.0
        } else {
            800.0
        };
        let jenkins_tco =
            ((70.0 + bk_agents * 30.0 + jenkins_admin_usd) * 100.0).round() / 100.0;
        // AWS CodeBuild: $0 seats + 100 free mins + $0.005/min Linux + $0.016/min Win + $0.065/min Mac
        let cb_tco = (((monthly_linux_mins - 100.0).max(0.0) * 0.005
            + monthly_win_mins * 0.016
            + monthly_mac_mins * 0.065)
            * 100.0)
            .round()
            / 100.0;
        // GCP Cloud Build: $0 seats + 2,500 free e2-medium mins + $0.006/min e2-standard-2 (2 vCPU apples-to-apples) Linux
        let gcp_tco = (((monthly_linux_mins - 1250.0).max(0.0) * 0.006
            + monthly_win_mins * 0.010
            + monthly_mac_mins * 0.062)
            * 100.0)
            .round()
            / 100.0;
        // Azure Pipelines: 1 free hosted job (1,800 mins) + $40/mo per extra parallel hosted lane needed
        let az_lanes = ((monthly_billed_mins / 7500.0).ceil() as usize)
            .max(if peak_concurrent_jobs_per_run > 1 { 2 } else { 1 });
        let az_tco = if monthly_billed_mins <= 1800.0 && peak_concurrent_jobs_per_run <= 1 {
            0.0
        } else {
            (((az_lanes.saturating_sub(1)) as f64) * 40.0 * 100.0).round() / 100.0
        };

        let candidates = [
            ("github-actions-team", gh_tco),
            ("gitlab-ci-premium", gl_tco),
            ("circleci-performance", cci_tco),
            // Bitbucket Pipelines has no hosted macOS runner ("Avoid Bitbucket Pipelines for iOS-heavy work")
            ("bitbucket-premium", if monthly_mac_mins > 0.0 { f64::MAX } else { bb_tco }),
            ("buildkite-pro-byo", bk_tco),
            ("aws-codebuild", cb_tco),
            ("gcp-cloud-build", gcp_tco),
            ("azure-pipelines", az_tco),
        ];
        let mut best_vendor = "github-actions-team";
        let mut best_tco = gh_tco;
        for (name, val) in candidates {
            if val < best_tco {
                best_tco = val;
                best_vendor = name;
            }
        }
        let current_vendor_slug = if hygiene.has_jenkinsfile {
            "jenkins-self-hosted"
        } else {
            match primary_provider {
                "gitlab-ci" => "gitlab-ci-premium",
                "circleci" => "circleci-performance",
                "bitbucket" => "bitbucket-premium",
                "aws-codebuild" => "aws-codebuild",
                "gcp-cloud-build" => "gcp-cloud-build",
                "azure-pipelines" => "azure-pipelines",
                _ => "github-actions-team",
            }
        };
        let current_tco = match current_vendor_slug {
            "jenkins-self-hosted" => jenkins_tco,
            "gitlab-ci-premium" => gl_tco,
            "circleci-performance" => cci_tco,
            "bitbucket-premium" => bb_tco,
            "aws-codebuild" => cb_tco,
            "gcp-cloud-build" => gcp_tco,
            "azure-pipelines" => az_tco,
            _ => gh_tco,
        };
        let switching = compute_5layer_platform_switching_tco(
            workflow_count,
            implied_active_devs,
            current_vendor_slug,
            best_vendor,
            current_tco,
            best_tco,
            blended_fully_loaded_hourly_usd,
            hygiene.has_jenkinsfile,
        );

        (
            best_vendor.to_string(),
            best_tco,
            gh_tco,
            gl_tco,
            cci_tco,
            bb_tco,
            bk_tco,
            jenkins_tco,
            cb_tco,
            gcp_tco,
            az_tco,
            switching.engineering_hours,
            switching.total_switching_cost_usd,
            switching.payback_months,
            switching.meets_2x_annual_rule,
        )
    };

    // 18. 6-Phase Build-Minute Time-Share & Best-Payback Optimization Ranker
    let (
        phase_clone_checkout_share_pct,
        phase_dependency_install_share_pct,
        phase_compile_typecheck_share_pct,
        phase_test_suite_share_pct,
        phase_docker_build_push_share_pct,
        phase_deploy_cleanup_share_pct,
        monthly_total_reducible_phase_savings_usd,
        best_first_optimization_phase,
        best_first_optimization_effort_hours,
        best_first_optimization_monthly_savings_usd,
        best_first_optimization_payback_months,
    ) = if jobs.is_empty() {
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, "none".to_string(), 0.0, 0.0, 0.0)
    } else {
        let is_compiled_heavy = detected_log_language_profile.contains("Rust")
            || detected_log_language_profile.contains("Java")
            || detected_log_language_profile.contains("iOS")
            || detected_log_language_profile.contains("Android")
            || detected_log_language_profile.contains(".NET")
            || hygiene.has_cpp_or_cmake
            || hygiene.has_flutter_or_dart;
        let (p_clone, p_dep, p_comp, p_test, p_dock, p_dep_clean): (
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
        ) = if let Some([rc, rd, rco, rt, rdo, rdc]) = active_telemetry
            .as_ref()
            .and_then(|t| t.real_phase_shares)
        {
            (rc, rd, rco, rt, rdo, rdc)
        } else {
            let (pc, pd, pco, pt, pdo): (f64, f64, f64, f64, f64) =
                if hygiene.has_docker_build_or_push_steps && is_compiled_heavy {
                    (5.0, 18.0, 25.0, 28.0, 17.0)
                } else if hygiene.has_docker_build_or_push_steps {
                    (5.0, 20.0, 18.0, 32.0, 18.0)
                } else if is_compiled_heavy {
                    (5.0, 22.0, 30.0, 36.0, 0.0)
                } else if hygiene.has_deploy_or_release_steps {
                    (6.0, 25.0, 17.0, 40.0, 0.0)
                } else {
                    (6.0, 26.0, 18.0, 42.0, 0.0)
                };
            let pdc = ((100.0 - (pc + pd + pco + pt + pdo)) * 10.0).round() / 10.0;
            (pc, pd, pco, pt, pdo, pdc)
        };

        let shallow_jobs: usize = jobs
            .iter()
            .filter(|j| j.has_sparse_or_shallow_clone)
            .map(|j| j.matrix_combination_count)
            .sum();
        let shallow_ratio =
            (shallow_jobs as f64) / (total_expanded_matrix_jobs.max(1) as f64);
        let red_clone = (0.60 - 0.45 * shallow_ratio).clamp(0.15, 0.60);
        let cache_eligible = (jobs_with_cache + jobs_missing_cache).max(1) as f64;
        let uncached_ratio = (jobs_missing_cache as f64 / cache_eligible).clamp(0.0, 1.0);
        let red_dep = (0.15 + 0.55 * uncached_ratio).clamp(0.15, 0.70);
        let red_comp = if hygiene.has_remote_compile_cache {
            0.15
        } else {
            0.50
        };
        let red_test = if hygiene.has_test_sharding_or_split && hygiene.has_affected_only_graph_tool
        {
            0.20
        } else {
            0.55
        };
        let red_dock = if p_dock == 0.0 {
            0.0
        } else if hygiene.has_docker_buildkit_cache {
            0.20
        } else {
            0.65
        };
        let red_clean = if configured_retention_days <= 14 {
            0.15
        } else {
            0.40
        };

        let phases = [
            ("clone_checkout", p_clone, red_clone, 4.0_f64),
            ("dependency_install", p_dep, red_dep, 6.0_f64),
            ("compile_typecheck", p_comp, red_comp, 24.0_f64),
            ("test_suite", p_test, red_test, 32.0_f64),
            ("docker_build_push", p_dock, red_dock, 16.0_f64),
            ("deploy_cleanup", p_dep_clean, red_clean, 6.0_f64),
        ];
        let mut total_reducible_usd = 0.0_f64;
        let mut best_phase = "dependency_install";
        let mut best_effort = 6.0_f64;
        let mut best_saving = 0.0_f64;
        let mut best_payback = f64::MAX;

        for (p_name, share_pct, red_frac, effort_h) in phases {
            let phase_usd = gross_monthly_compute_cost_usd * (share_pct / 100.0);
            let sav_usd = phase_usd * red_frac;
            total_reducible_usd += sav_usd;
            if sav_usd > 0.0 {
                let payback = (effort_h * usage.dev_hourly_rate_usd) / sav_usd;
                if payback < best_payback {
                    best_payback = payback;
                    best_phase = p_name;
                    best_effort = effort_h;
                    best_saving = sav_usd;
                }
            }
        }
        let final_payback = if best_payback < f64::MAX {
            ((best_payback * 100.0).round() / 100.0).min(120.0)
        } else {
            0.0
        };
        (
            p_clone,
            p_dep,
            p_comp,
            p_test,
            p_dock,
            p_dep_clean,
            (total_reducible_usd * 100.0).round() / 100.0,
            best_phase.to_string(),
            best_effort,
            (best_saving * 100.0).round() / 100.0,
            final_payback,
        )
    };

    // 19. Compute-Class Right-Sizing Explorer (Amdahl Speedup vs Larger-Runner Billing & Dev-Wait Net ROI)
    let (
        detected_workload_compute_profile,
        arm64_2core_monthly_compute_usd,
        arm64_migration_monthly_savings_usd,
        standard_2core_monthly_compute_usd,
        larger_4core_monthly_compute_usd,
        larger_8core_monthly_compute_usd,
        larger_16core_monthly_compute_usd,
        best_net_roi_runner_tier,
        best_net_roi_monthly_delta_usd,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            "standard-2-core".to_string(),
            0.0,
        )
    } else {
        // Realized Amdahl speedups across (4-core, 8-core, 16-core, 32-core, 64-core) relative to 2-core baseline
        let (profile_name, s4, s8, s16, s32, s64) = if detected_log_language_profile.contains("Rust")
            || detected_log_language_profile.contains("Java")
            || detected_log_language_profile.contains("Go")
            || detected_log_language_profile.contains("iOS")
            || detected_log_language_profile.contains("Android")
            || hygiene.has_cpp_or_cmake
            || hygiene.has_flutter_or_dart
        {
            ("compile-heavy", 1.65_f64, 2.40_f64, 3.20_f64, 3.85_f64, 4.30_f64)
        } else if hygiene.has_browser_or_e2e_tests || infracost_iac_files_detected {
            ("io-bound", 1.18_f64, 1.30_f64, 1.38_f64, 1.41_f64, 1.43_f64)
        } else if total_expanded_matrix_jobs >= 8 || hygiene.has_test_sharding_or_split {
            ("test-heavy", 1.55_f64, 2.15_f64, 2.80_f64, 3.30_f64, 3.65_f64)
        } else {
            ("balanced-web-backend", 1.33_f64, 1.56_f64, 1.75_f64, 1.84_f64, 1.89_f64)
        };
        // Isolate Linux CPU-scalable jobs from fixed non-Linux (macOS / Windows / GPU / 3P) jobs
        let mut linux_scalable_2c_run_usd = 0.0_f64;
        let mut fixed_non_linux_run_usd = 0.0_f64;
        for j in &jobs {
            let rl = j.runner_label.to_ascii_lowercase();
            let is_gpu = rl.contains("gpu");
            let is_self_or_3p = j.provider.starts_with("self-hosted")
                || j.provider == "blacksmith"
                || rl.contains("blacksmith")
                || rl.contains("depot")
                || rl.contains("warpbuild")
                || rl.contains("buildjet")
                || rl.contains("namespace")
                || rl.contains("self-hosted");
            if j.os.eq_ignore_ascii_case("linux") && !is_gpu && !is_self_or_3p {
                if j.vcpus > 2 && j.rate_per_min_usd > 0.006 {
                    let job_speedup = match j.vcpus {
                        4 => s4,
                        8 => s8,
                        16 => s16,
                        32 => s32,
                        64..=u32::MAX => s64,
                        _ => 1.0,
                    };
                    linux_scalable_2c_run_usd += j.total_billed_mins_all_matrix * job_speedup * 0.006;
                } else {
                    linux_scalable_2c_run_usd += j.run_compute_cost_usd;
                }
            } else {
                fixed_non_linux_run_usd += j.run_compute_cost_usd;
            }
        }
        let linux_2c_mo = linux_scalable_2c_run_usd * monthly_runs;
        let fixed_non_linux_mo = fixed_non_linux_run_usd * monthly_runs;
        let std_2c_mo = ((linux_2c_mo + fixed_non_linux_mo) * 100.0).round() / 100.0;
        // Linux arm64 2-core ($0.005/min) vs standard x64 2-core ($0.006/min) saves 16.7%, and x86 4-core+ Linux -> arm64 saves ~36%
        let arm_sav = ((non_arm_linux_2core_mins_per_run * monthly_runs * (0.006 - 0.005)
            + larger_x86_linux_non_gpu_monthly_usd * 0.36)
            * 100.0)
            .round()
            / 100.0;
        let arm_2c_mo = ((gross_monthly_compute_cost_usd - arm_sav).max(0.0) * 100.0).round() / 100.0;

        // Scale ONLY Linux CPU-scalable portion by (rate_ratio / realised_speedup); keep macOS/Windows/GPU unchanged
        let l4_mo = ((fixed_non_linux_mo + linux_2c_mo * (2.0 / s4)) * 100.0).round() / 100.0;
        let l8_mo = ((fixed_non_linux_mo + linux_2c_mo * (3.6667 / s8)) * 100.0).round() / 100.0;
        let l16_mo = ((fixed_non_linux_mo + linux_2c_mo * (7.0 / s16)) * 100.0).round() / 100.0;
        let l32_mo = ((fixed_non_linux_mo + linux_2c_mo * (13.6667 / s32)) * 100.0).round() / 100.0;
        let l64_mo = ((fixed_non_linux_mo + linux_2c_mo * (27.0 / s64)) * 100.0).round() / 100.0;

        // Larger runners never draw free minutes: if within Free tier, standard 2-core costs $0 out-of-pocket
        let baseline_billed_mo = if effective_tier.starts_with("free") {
            larger_runners_bypass_free_minutes_tax_usd
        } else {
            std_2c_mo
        };
        let linux_wait_share = if billed_mins_per_run > 0.0 {
            (linux_mins_per_run / billed_mins_per_run).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let wait_loss_base = monthly_productive_payroll_loss_usd * linux_wait_share;
        let net_4c = (l4_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s4));
        let net_8c = (l8_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s8));
        let net_16c = (l16_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s16));
        let net_32c = (l32_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s32));
        let net_64c = (l64_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s64));

        let mut best_tier = "standard-2-core";
        let mut best_delta = 0.0_f64;
        for (t_name, delta) in [
            ("larger-4-core", net_4c),
            ("larger-8-core", net_8c),
            ("larger-16-core", net_16c),
            ("larger-32-core", net_32c),
            ("larger-64-core", net_64c),
        ] {
            if delta < best_delta {
                best_delta = delta;
                best_tier = t_name;
            }
        }
        (
            profile_name.to_string(),
            arm_2c_mo,
            arm_sav,
            std_2c_mo,
            l4_mo,
            l8_mo,
            l16_mo,
            best_tier.to_string(),
            (best_delta * 100.0).round() / 100.0,
        )
    };

    // 20. Language-Stack Deep-Dive Intelligence (iOS / Flutter / Android / JVM / C++ / Rust / Node / Python / Go / IaC)
    let (
        detected_stack_archetype,
        ios_codemagic_annual_m2_monthly_usd,
        ios_buildkite_mac_mini_fleet_monthly_tco_usd,
        android_emulator_jobs_count,
        android_kvm_or_gradle_cache_savings_usd,
        jvm_warm_cache_50pct_savings_usd,
        stack_specific_monthly_savings_usd,
        stack_specific_primary_lever,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0,
            0.0,
            0.0,
            0.0,
            "none".to_string(),
        )
    } else {
        // Compute macOS Codemagic M2 flat plan ($332.50/mo) & Buildkite Mac mini fleet ($25/mini + seats) whenever macOS jobs exist
        let (cm_annual_mo, bk_mac_tco, ios_mac_sav) = if macos_mins_per_run > 0.0 {
            let cm = 332.50_f64; // $3,990/yr flat unmetered Mac mini M2 (3 concurrent slots)
            let minis_needed = ((monthly_mac_mins / 9000.0).ceil().max(1.0)) as f64;
            let bk_seat_usd = if implied_active_devs <= 5 && macos_concurrent_jobs_per_run <= 10 {
                0.0
            } else {
                (implied_active_devs as f64) * 30.0
            };
            // $899 Mac mini / 36 months = $25/mo per mini (calc_ios.html lines 521, 558, 579, 600)
            let bk = ((bk_seat_usd + minis_needed * 25.0) * 100.0).round() / 100.0;
            let best_alt = cm.min(bk);
            let sav = ((monthly_macos_hosted_usd - best_alt).max(0.0) * 100.0).round() / 100.0;
            (cm, bk, sav)
        } else {
            (0.0, 0.0, 0.0)
        };

        let is_primary_ios_macos = !hygiene.has_flutter_or_dart
            && !hygiene.has_cpp_or_cmake
            && !hygiene.has_rust_or_cargo
            && !hygiene.has_go_or_gomod
            && macos_mins_per_run > 0.0
            && macos_mins_per_run >= (linux_mins_per_run + windows_mins_per_run);

        if is_primary_ios_macos {
            let lever = if monthly_mac_mins >= 4000.0 {
                "self-hosted-mac-mini-fleet-or-flat-m2-plan"
            } else {
                "xcode-deriveddata-spm-cache-and-macos-on-merge-only"
            };
            (
                "ios-macos-xcode".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count,
                0.0,
                0.0,
                ios_mac_sav,
                lever.to_string(),
            )
        } else if hygiene.has_flutter_or_dart {
            let fl_sav = ((gross_monthly_compute_cost_usd * 0.35).max(ios_mac_sav) * 100.0).round()
                / 100.0;
            (
                "flutter-dart-mobile".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count,
                0.0,
                0.0,
                fl_sav,
                "pub-cache-gradle-deriveddata-and-codemagic-m2-flat-plan".to_string(),
            )
        } else if detected_log_language_profile.contains("Android")
            || hygiene.android_emulator_jobs_count > 0
        {
            let ratio = if hygiene.has_gradle_build_cache_flag && jobs_missing_cache == 0 {
                0.25
            } else {
                0.50
            };
            let and_sav = ((gross_monthly_compute_cost_usd * ratio) * 100.0).round() / 100.0;
            (
                "android-gradle-emulator".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count.max(1),
                and_sav,
                0.0,
                and_sav,
                "3-layer-gradle-cache-and-kvm-emulator-snapshot".to_string(),
            )
        } else if detected_log_language_profile.contains("Java") || hygiene.has_gradle_multi_module {
            let ratio = if hygiene.has_gradle_build_cache_flag || hygiene.has_maven_parallel_flag {
                0.20
            } else {
                0.50
            };
            let jvm_sav = ((gross_monthly_compute_cost_usd * ratio) * 100.0).round() / 100.0;
            let lever = if hygiene.has_gradle_multi_module {
                "gradle-build-and-configuration-cache-with-incremental-module-check"
            } else {
                "gradle-build-cache-or-maven-parallel-m2-cache"
            };
            (
                "java-jvm-gradle-maven".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                jvm_sav,
                jvm_sav,
                lever.to_string(),
            )
        } else if hygiene.has_cpp_or_cmake && !hygiene.has_rust_or_cargo {
            let cpp_sav = if hygiene.has_remote_compile_cache {
                ((gross_monthly_compute_cost_usd * 0.18) * 100.0).round() / 100.0
            } else {
                ((gross_monthly_compute_cost_usd * 0.45) * 100.0).round() / 100.0
            };
            (
                "cpp-cmake-ninja".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                cpp_sav,
                "ccache-remote-storage-and-ninja-larger-runners".to_string(),
            )
        } else if detected_log_language_profile.contains("Rust")
            || (detected_log_language_profile.contains("iOS") && hygiene.has_rust_or_cargo)
        {
            let rust_sav = if hygiene.has_remote_compile_cache {
                ((gross_monthly_compute_cost_usd * 0.15) * 100.0).round() / 100.0
            } else {
                ((gross_monthly_compute_cost_usd * 0.45) * 100.0).round() / 100.0
            };
            (
                "rust-cargo".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                rust_sav,
                "rust-cache-sccache-and-cargo-nextest".to_string(),
            )
        } else if infracost_iac_files_detected
            && jobs
                .iter()
                .filter(|j| {
                    j.workflow_file.contains("terragrunt") || j.workflow_file.contains("terraform")
                })
                .count()
                * 2
                >= jobs.len().max(1)
        {
            let iac_sav = ((gross_monthly_compute_cost_usd * 0.40) * 100.0).round() / 100.0;
            (
                "terraform-terragrunt-iac".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                iac_sav,
                "terraform-plugin-cache-and-changed-module-plan".to_string(),
            )
        } else if detected_log_language_profile.contains("Go")
            || (detected_log_language_profile.contains("iOS") && hygiene.has_go_or_gomod)
        {
            let go_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.30) * 100.0).round() / 100.0);
            (
                "go-mod-build-cache".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                go_sav,
                "go-build-mod-cache-and-arm64-runners".to_string(),
            )
        } else if detected_log_language_profile.contains("Python")
            || (detected_log_language_profile.contains("iOS") && hygiene.has_python_or_pytest)
        {
            let py_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.25) * 100.0).round() / 100.0);
            (
                "python-uv-pytest".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                py_sav,
                "uv-cache-and-pytest-xdist-sharding".to_string(),
            )
        } else {
            let node_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.30) * 100.0).round() / 100.0);
            (
                "node-typescript-vitest".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                node_sav,
                "pnpm-store-cache-and-turborepo-remote-cache".to_string(),
            )
        }
    };

    // 21. Unit Economics & Chargeback (`$/PR`, `$/Deploy`, Cadence-Weighted Spend & CODEOWNERS Attribution)
    let estimated_monthly_merged_prs = if jobs.is_empty() {
        0.0
    } else if let Some(obs_prs) = active_telemetry
        .as_ref()
        .and_then(|t| t.observed_merged_prs_30d)
    {
        (obs_prs as f64).max(1.0)
    } else {
        (usage.prs_per_week * 4.33).max(1.0)
    };
    let cost_per_merged_pr_usd = if estimated_monthly_merged_prs > 0.0 {
        ((gross_monthly_compute_cost_usd / estimated_monthly_merged_prs) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let est_monthly_deploys = if jobs.is_empty() {
        0.0
    } else if let Some(obs_dep) = active_telemetry
        .as_ref()
        .and_then(|t| t.observed_deploys_30d)
    {
        (obs_dep as f64).max(1.0)
    } else if hygiene.has_deploy_or_release_steps {
        (estimated_monthly_merged_prs * 0.50).max(4.0)
    } else {
        estimated_monthly_merged_prs.max(1.0)
    };
    let cost_per_deploy_usd = if est_monthly_deploys > 0.0 {
        ((gross_monthly_compute_cost_usd / est_monthly_deploys) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let cost_per_active_dev_monthly_usd = if implied_active_devs > 0 {
        ((gross_monthly_compute_cost_usd / (implied_active_devs as f64)) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let mut cadence_weighted_monthly_compute_usd = 0.0_f64;
    let mut wf_cost_totals: Vec<(String, f64)> = Vec::new();
    for j in &jobs {
        let wf_weight = hygiene
            .workflow_trigger_weights
            .iter()
            .find(|(wf, _)| wf == &j.workflow_file)
            .map(|(_, w)| *w)
            .unwrap_or(1.0);
        let wf_runs = if wf_weight < 0.25 {
            // Schedule / release-only workflows run ~30x/mo (nightly) or ~8x/mo (release) instead of every PR push
            (monthly_runs * wf_weight).max(4.0)
        } else {
            monthly_runs * wf_weight
        };
        let j_mo = j.run_compute_cost_usd * monthly_runs;
        cadence_weighted_monthly_compute_usd += j.run_compute_cost_usd * wf_runs;
        if let Some(entry) = wf_cost_totals.iter_mut().find(|(wf, _)| wf == &j.workflow_file) {
            entry.1 += j_mo;
        } else {
            wf_cost_totals.push((j.workflow_file.clone(), j_mo));
        }
    }
    cadence_weighted_monthly_compute_usd =
        (cadence_weighted_monthly_compute_usd * 100.0).round() / 100.0;
    let top_wf_usd = wf_cost_totals
        .iter()
        .map(|(_, c)| *c)
        .fold(0.0_f64, f64::max);
    let top_workflow_cost_share_pct = if gross_monthly_compute_cost_usd > 0.0 {
        (((top_wf_usd / gross_monthly_compute_cost_usd) * 100.0) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let codeowners_detected = hygiene.codeowners_detected;

    // 22. Spend Anomaly, Burn Velocity & Missing `timeout-minutes:` Exposure
    let burn_velocity_7d_vs_30d_ratio = if jobs.is_empty() {
        0.0
    } else if let (Some(r7), Some(r30)) = (
        active_telemetry.as_ref().and_then(|t| t.observed_runs_last_7d),
        active_telemetry.as_ref().and_then(|t| t.observed_monthly_runs),
    ) {
        let daily_7d = r7 / 7.0;
        let daily_30d = (r30 / 30.0).max(0.01);
        ((daily_7d / daily_30d) * 100.0).round() / 100.0
    } else {
        1.0
    };
    let spend_anomaly_alert_active = burn_velocity_7d_vs_30d_ratio >= 1.35;
    let projected_eom_compute_usd =
        ((gross_monthly_compute_cost_usd * burn_velocity_7d_vs_30d_ratio.max(0.5)) * 100.0).round()
            / 100.0;
    let projected_quota_exhaustion_day = if free_tier_quota_burn_pct <= 100.0 || monthly_billed_mins <= 0.0 {
        0_u32
    } else {
        let frac_of_month = (free_tier_included_mins / (monthly_billed_mins * burn_velocity_7d_vs_30d_ratio.max(1.0))).clamp(0.03, 1.0);
        ((frac_of_month * 30.0).ceil() as u32).clamp(1, 30)
    };
    let jobs_missing_timeout_minutes_count = if jobs.is_empty() {
        0_usize
    } else {
        total_defined_jobs.saturating_sub(hygiene.jobs_with_timeout_minutes_count)
    };
    // GitHub default job timeout is 360 mins (6h): 1 hung run across jobs missing timeout-minutes
    let hung_job_360m_worst_case_exposure_usd = if jobs_missing_timeout_minutes_count == 0 {
        0.0
    } else {
        let avg_job_rate = avg_rate_per_min.max(0.006);
        (((jobs_missing_timeout_minutes_count as f64) * 360.0 * avg_job_rate) * 100.0).round()
            / 100.0
    };

    // 23. Container Registry Economics (GHCR / ECR / GCR / ACR Image Storage & Pull Bandwidth)
    let container_images_built_count = if hygiene.container_images_built_count > 0 {
        hygiene.container_images_built_count
    } else if hygiene.has_docker_build_or_push_steps {
        1
    } else {
        0
    };
    let (
        est_registry_storage_gb,
        monthly_registry_storage_and_pull_usd,
        untagged_image_cleanup_monthly_savings_usd,
    ) = if container_images_built_count == 0 {
        (0.0, 0.0, 0.0)
    } else {
        // ~0.45 GB compressed layer delta per image build across active monthly retention
        let retention_factor = if hygiene.has_registry_retention_cleanup {
            0.35
        } else {
            1.0
        };
        let reg_gb = (((container_images_built_count as f64)
            * 0.45
            * (monthly_runs * 0.35).max(4.0)
            * retention_factor)
            * 100.0)
            .round()
            / 100.0;
        // ECR/GHCR ~$0.09-$0.10/GB-mo storage (after 0.5-1 GB free allowance)
        let billable_reg_gb = (reg_gb - 1.0).max(0.0);
        let reg_usd = ((billable_reg_gb * 0.10) * 100.0).round() / 100.0;
        let cleanup_sav = if hygiene.has_registry_retention_cleanup {
            0.0
        } else {
            ((reg_usd * 0.65) * 100.0).round() / 100.0
        };
        (reg_gb, reg_usd, cleanup_sav)
    };

    // 24. Self-Hosted Fleet Off-Hours Idle Waste & Queue SLA Breach Risk
    let (
        off_hours_idle_capacity_pct,
        monthly_always_on_idle_waste_usd,
        est_p95_queue_wait_sec,
        sla_queue_breach_risk,
    ) = if jobs.is_empty() {
        (0.0, 0.0, 0.0, false)
    } else {
        let idle_pct = if let Some(obs_off_share) = active_telemetry
            .as_ref()
            .and_then(|t| t.observed_off_hours_run_share_pct)
        {
            ((100.0 - obs_off_share).clamp(0.0, 99.0) * 10.0).round() / 10.0
        } else {
            ((100.0 - always_on_runner_utilization_pct).clamp(0.0, 99.0) * 10.0).round() / 10.0
        };
        let idle_waste_usd = if true_self_hosted_jobs_per_run > 0 {
            (((fleet_nodes as f64) * ec2_od_mo * (idle_pct / 100.0)) * 100.0).round() / 100.0
        } else {
            0.0
        };
        let p95_q_sec = if let Some(obs_q) = active_telemetry
            .as_ref()
            .and_then(|t| t.observed_p95_queue_wait_sec)
        {
            (obs_q * 10.0).round() / 10.0
        } else if hosted_concurrent_jobs_per_run > vendor_free_concurrent_cap {
            let over_ratio =
                (hosted_concurrent_jobs_per_run as f64) / (vendor_free_concurrent_cap as f64);
            ((over_ratio - 1.0) * 120.0 + 25.0).round()
        } else if true_self_hosted_jobs_per_run > 0 {
            35.0
        } else {
            8.0
        };
        let breach = p95_q_sec > 60.0;
        (idle_pct, idle_waste_usd, p95_q_sec, breach)
    };

    let provenance_mode = active_telemetry
        .as_ref()
        .filter(|t| !t.provenance_mode.is_empty())
        .map(|t| t.provenance_mode.clone())
        .unwrap_or_else(|| "static_heuristic".to_string());

    let cost_components = CostComponentsBreakdown {
        rounding_tax_mins_per_run,
        monthly_rounding_tax_usd,
        linux_mins_per_run: (linux_mins_per_run * 100.0).round() / 100.0,
        windows_mins_per_run: (windows_mins_per_run * 100.0).round() / 100.0,
        macos_mins_per_run: (macos_mins_per_run * 100.0).round() / 100.0,
        os_weighted_free_tier_mins_per_run: (os_weighted_free_tier_mins_per_run * 100.0).round()
            / 100.0,
        monthly_os_weighted_free_tier_mins,
        monthly_flake_retry_10pct_overhead_usd,
        infracost_iac_files_detected,
        hosted_effective_hourly_rate_usd,
        infracost_recommended_ec2_instance: ec2_label.to_string(),
        infracost_ec2_on_demand_hourly_usd: ec2_od_hr,
        infracost_ec2_on_demand_monthly_usd: ec2_od_mo,
        infracost_ec2_spot_hourly_usd: ec2_spot_hr,
        infracost_ec2_surrounding_overhead_monthly_usd,
        macos_self_hosted_mac_mini_monthly_usd,
        self_hosted_breakeven_mins_t3_medium: SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH,
        self_hosted_breakeven_mins_c5_4xlarge_spot: SELF_HOSTED_C5_4XLARGE_SPOT_BREAKEVEN_MINS,
        always_on_runner_utilization_pct,
        ephemeral_runner_recommended_mode,
        ephemeral_cold_start_wait_tax_monthly_usd,
        k8s_eks_control_plane_monthly_usd,
        k8s_gke_aks_free_tier_control_plane_monthly_usd,
        spot_interruption_band_pct,
        spot_retry_overhead_monthly_usd,
        spot_ineligible_jobs_count,
        hybrid_spot_plus_on_demand_monthly_usd,
        jobs_uploading_artifacts,
        default_retention_days,
        configured_retention_days,
        steady_state_stored_gb_configured,
        monthly_vendor_storage_configured_usd,
        monthly_infracost_s3_configured_usd,
        steady_state_stored_gb_90d,
        steady_state_stored_gb_14d,
        monthly_vendor_storage_90d_usd,
        monthly_vendor_storage_14d_usd,
        monthly_infracost_s3_standard_90d_usd,
        monthly_infracost_s3_intelligent_14d_usd,
        peak_concurrent_jobs_per_run,
        hosted_concurrent_jobs_per_run,
        macos_concurrent_jobs_per_run,
        self_hosted_or_microvm_jobs_per_run,
        vendor_free_concurrent_cap,
        vendor_team_concurrent_cap,
        parallelism_tier_upgrade_tax_monthly_usd,
        detected_repo_matrix_profile,
        matrix_jobs_count,
        pr_unconditional_matrix_cells,
        conditional_pr_matrix_monthly_savings_usd,
        detected_log_language_profile,
        est_log_mb_per_run,
        monthly_log_volume_gb,
        vendor_default_log_retention_days,
        monthly_vendor_default_log_cost_usd: 0.0,
        monthly_datadog_30d_log_shipping_usd,
        monthly_datadog_90d_log_shipping_usd,
        monthly_splunk_30d_log_shipping_usd,
        monthly_infracost_loki_s3_usd,
        infracost_cloud_region,
        est_transfer_gb_per_run,
        monthly_transfer_gb,
        infracost_aws_internet_egress_per_gb_usd: AWS_INTERNET_EGRESS_USD_PER_GB,
        infracost_aws_nat_gateway_per_gb_usd,
        infracost_aws_cross_az_per_gb_usd: AWS_CROSS_AZ_USD_PER_GB,
        infracost_aws_cross_region_per_gb_usd: AWS_CROSS_REGION_USD_PER_GB,
        monthly_infracost_private_subnet_nat_worst_case_usd,
        monthly_infracost_cross_region_registry_usd,
        monthly_infracost_vpc_endpoint_cache_optimized_usd,
        productivity_loss_factor,
        blended_fully_loaded_hourly_usd,
        monthly_wait_hours,
        monthly_productive_payroll_loss_usd,
        monthly_blended_100hr_productive_loss_usd,
        wait_to_compute_cost_ratio,
        lead_time_feedback_tier,
        estimated_flake_rate_pct,
        has_job_level_auto_retry: hygiene.has_job_level_auto_retry,
        monthly_flaky_reruns,
        monthly_rerun_compute_cost_usd,
        avg_investigation_mins_per_flake,
        monthly_flake_investigation_hours,
        monthly_flake_investigation_payroll_usd,
        monthly_total_immediate_flake_cost_usd,
        monthly_savings_at_1pct_target_usd,
        stack_cache_profile: stack_cache_label.to_string(),
        realistic_saving_pct_min,
        realistic_saving_pct_max,
        jobs_with_cache,
        jobs_missing_cache,
        has_restore_keys_fallback: hygiene.has_restore_keys_fallback,
        has_docker_buildkit_cache: hygiene.has_docker_buildkit_cache,
        estimated_cache_hit_rate_pct,
        vendor_cache_size_limit: vendor_cache_limit.to_string(),
        vendor_cache_eviction_policy: vendor_cache_eviction.to_string(),
        monthly_uncached_compute_usd,
        monthly_unrealized_cache_savings_usd,
        next_recommended_caching_layer,
        distributed_remote_cache_monthly_infra_usd,
        container_base_image_mirror_monthly_savings_usd,
        pr_cache_poisoning_risk_jobs,
        deferred_velocity_drag_multiplier,
        monthly_deferred_velocity_drag_usd,
        monthly_combined_flake_tax_usd,
        deflake_project_est_cost_usd,
        deflake_payback_weeks,
        required_checks_count,
        monthly_required_check_executions,
        workflows_with_concurrency_cancel: hygiene.workflows_with_concurrency_cancel,
        workflows_missing_concurrency_cancel: hygiene.workflows_missing_concurrency_cancel,
        workflows_with_path_filters: hygiene.workflows_with_path_filters,
        workflows_missing_path_filters: hygiene.workflows_missing_path_filters,
        has_affected_only_graph_tool: hygiene.has_affected_only_graph_tool,
        slow_blocking_checks_over_10m,
        monthly_superseded_run_waste_usd,
        monthly_unfiltered_check_savings_usd,
        inferred_team_stage,
        implied_active_devs,
        free_tier_burn_pct,
        upgrade_trigger_80pct_reached,
        premature_self_hosting_trap,
        premature_multi_os_pr_matrix_trap,
        monthly_multi_os_pr_to_nightly_savings_usd,
        mac_mini_hardware_payback_months,
        self_hosted_breakeven_reached,
        team_to_enterprise_seat_jump_monthly_usd,
        quarterly_required_check_audit_savings_usd,
        platform_engineer_monthly_fully_loaded_usd,
        recoverable_ci_and_wait_monthly_usd,
        platform_engineer_roi_coverage_pct,
        detected_compliance_controls: hygiene.detected_compliance_controls,
        compliance_controls_count,
        detected_cloud_regions: hygiene.detected_cloud_regions,
        regional_residency_multiplier,
        enterprise_250_seat_licence_monthly_usd,
        siem_1yr_audit_log_retention_monthly_usd,
        dedicated_tenancy_premium_pct_min,
        dedicated_tenancy_premium_pct_max,
        detected_pipeline_shape,
        detected_workspace_units_count,
        detected_monorepo_tool: hygiene.detected_monorepo_tool,
        recommended_affected_tool,
        has_nightly_full_suite_safety_net: hygiene.has_nightly_schedule_workflow,
        affected_only_reduction_pct_min,
        affected_only_reduction_pct_max,
        monthly_affected_only_savings_usd,
        est_setup_overhead_sec_per_job,
        monthly_setup_overhead_hours,
        setup_overhead_share_pct,
        monthly_setup_overhead_compute_usd,
        monthly_shared_infra_setup_savings_usd,
        reusable_workflow_calls_count: hygiene.reusable_workflow_calls_count,
        unpinned_template_refs_count: hygiene.unpinned_template_refs_count,
        cross_repo_triggers_count: hygiene.cross_repo_triggers_count,
        est_aggregate_cache_gb,
        exceeds_10gb_repo_cache_limit,
        cheapest_vendor_platform,
        cheapest_vendor_monthly_tco_usd,
        github_actions_team_monthly_tco_usd,
        gitlab_ci_premium_monthly_tco_usd,
        circleci_performance_monthly_tco_usd,
        bitbucket_premium_monthly_tco_usd,
        buildkite_pro_monthly_tco_usd,
        jenkins_self_hosted_monthly_tco_usd,
        aws_codebuild_monthly_tco_usd,
        gcp_cloud_build_monthly_tco_usd,
        azure_pipelines_monthly_tco_usd,
        migration_estimated_hours,
        migration_estimated_cost_usd,
        migration_payback_months,
        migration_worth_it_2x_annual_rule,
        phase_clone_checkout_share_pct,
        phase_dependency_install_share_pct,
        phase_compile_typecheck_share_pct,
        phase_test_suite_share_pct,
        phase_docker_build_push_share_pct,
        phase_deploy_cleanup_share_pct,
        monthly_total_reducible_phase_savings_usd,
        best_first_optimization_phase,
        best_first_optimization_effort_hours,
        best_first_optimization_monthly_savings_usd,
        best_first_optimization_payback_months,
        detected_workload_compute_profile,
        larger_runners_detected_count,
        larger_runners_bypass_free_minutes_tax_usd,
        arm64_2core_monthly_compute_usd,
        arm64_migration_monthly_savings_usd,
        standard_2core_monthly_compute_usd,
        larger_4core_monthly_compute_usd,
        larger_8core_monthly_compute_usd,
        larger_16core_monthly_compute_usd,
        best_net_roi_runner_tier,
        best_net_roi_monthly_delta_usd,
        detected_stack_archetype,
        ios_codemagic_annual_m2_monthly_usd,
        ios_buildkite_mac_mini_fleet_monthly_tco_usd,
        android_emulator_jobs_count,
        android_kvm_or_gradle_cache_savings_usd,
        jvm_warm_cache_50pct_savings_usd,
        stack_specific_monthly_savings_usd,
        stack_specific_primary_lever,
        cost_per_merged_pr_usd,
        cost_per_deploy_usd,
        cost_per_active_dev_monthly_usd,
        cadence_weighted_monthly_compute_usd,
        top_workflow_cost_share_pct,
        codeowners_detected,
        burn_velocity_7d_vs_30d_ratio,
        spend_anomaly_alert_active,
        projected_eom_compute_usd,
        projected_quota_exhaustion_day,
        jobs_missing_timeout_minutes_count,
        hung_job_360m_worst_case_exposure_usd,
        container_images_built_count,
        est_registry_storage_gb,
        monthly_registry_storage_and_pull_usd,
        untagged_image_cleanup_monthly_savings_usd,
        off_hours_idle_capacity_pct,
        monthly_always_on_idle_waste_usd,
        est_p95_queue_wait_sec,
        sla_queue_breach_risk,
    };

    let elapsed_us = t0.elapsed().as_micros().max(1);

    BreakdownReport {
        target_path: target.display().to_string(),
        provenance_mode,
        execution_time_us: elapsed_us,
        execution_time_ms: (elapsed_us as f64) / 1000.0,
        detected_providers: providers,
        usage,
        total_workflows: workflow_count,
        total_defined_jobs,
        total_expanded_matrix_jobs,
        raw_duration_mins_per_run: (raw_duration_mins_per_run * 100.0).round() / 100.0,
        critical_path_wait_mins_per_run: (critical_path_wait_mins_per_run * 100.0).round() / 100.0,
        billed_mins_per_run: (billed_mins_per_run * 100.0).round() / 100.0,
        monthly_runs,
        monthly_billed_mins,
        free_tier_included_mins,
        free_tier_quota_burn_pct,
        effective_tier,
        gross_compute_cost_per_run_usd,
        gross_monthly_compute_cost_usd,
        discount_allowance_monthly_usd,
        net_billed_monthly_compute_cost_usd,
        third_party_runner_monthly_cost_usd,
        shard_setup_duplication_mins_per_run,
        clones_per_ci_run,
        dev_wait_cost_per_run_usd,
        monthly_dev_wait_cost_usd,
        monthly_artifact_storage_vendor_usd,
        monthly_artifact_storage_s3_usd,
        monthly_blacksmith_cost_usd,
        monthly_self_hosted_spot_cost_usd,
        exceeds_5000m_self_hosted_breakeven: monthly_billed_mins
            >= SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH,
        cost_components,
        jobs,
    }
}

pub fn compute_diff(base: &BreakdownReport, head: &BreakdownReport) -> DiffReport {
    let t0 = Instant::now();
    let delta_jobs =
        (head.total_expanded_matrix_jobs as isize) - (base.total_expanded_matrix_jobs as isize);
    let delta_billed_mins =
        ((head.billed_mins_per_run - base.billed_mins_per_run) * 100.0).round() / 100.0;
    let delta_compute =
        ((head.gross_monthly_compute_cost_usd - base.gross_monthly_compute_cost_usd) * 100.0)
            .round()
            / 100.0;
    let delta_pct = if base.gross_monthly_compute_cost_usd > 0.0 {
        ((delta_compute / base.gross_monthly_compute_cost_usd) * 100.0 * 10.0).round() / 10.0
    } else {
        0.0
    };
    let delta_wait =
        ((head.monthly_dev_wait_cost_usd - base.monthly_dev_wait_cost_usd) * 100.0).round() / 100.0;

    let mut changes: Vec<String> = Vec::new();
    for hj in &head.jobs {
        if let Some(bj) = base
            .jobs
            .iter()
            .find(|x| x.workflow_file == hj.workflow_file && x.job_id == hj.job_id)
        {
            if bj.matrix_combination_count != hj.matrix_combination_count
                || bj.runner_label != hj.runner_label
                || (bj.billed_mins_per_job - hj.billed_mins_per_job).abs() > 0.01
                || (bj.run_compute_cost_usd - hj.run_compute_cost_usd).abs() > 0.0001
            {
                changes.push(format!(
                    "Modified `{}` (`{}`): `{}x {}` ({:.1}m • ${:.3}/run) -> `{}x {}` ({:.1}m • ${:.3}/run)",
                    hj.job_id,
                    hj.workflow_file,
                    bj.matrix_combination_count,
                    bj.runner_label,
                    bj.total_billed_mins_all_matrix,
                    bj.run_compute_cost_usd,
                    hj.matrix_combination_count,
                    hj.runner_label,
                    hj.total_billed_mins_all_matrix,
                    hj.run_compute_cost_usd
                ));
            }
        } else {
            changes.push(format!(
                "Added job `{}` (`{}`): `{}x {}` (+${:.3}/run)",
                hj.job_id,
                hj.workflow_file,
                hj.matrix_combination_count,
                hj.runner_label,
                hj.run_compute_cost_usd
            ));
        }
    }
    for bj in &base.jobs {
        if !head
            .jobs
            .iter()
            .any(|x| x.workflow_file == bj.workflow_file && x.job_id == bj.job_id)
        {
            changes.push(format!(
                "Removed job `{}` (`{}`): `{}x {}` (-${:.3}/run)",
                bj.job_id,
                bj.workflow_file,
                bj.matrix_combination_count,
                bj.runner_label,
                bj.run_compute_cost_usd
            ));
        }
    }

    DiffReport {
        base_path: base.target_path.clone(),
        head_path: head.target_path.clone(),
        execution_time_us: t0.elapsed().as_micros() + head.execution_time_us,
        base_expanded_jobs: base.total_expanded_matrix_jobs,
        head_expanded_jobs: head.total_expanded_matrix_jobs,
        delta_expanded_jobs: delta_jobs,
        base_billed_mins_per_run: base.billed_mins_per_run,
        head_billed_mins_per_run: head.billed_mins_per_run,
        delta_billed_mins_per_run: delta_billed_mins,
        base_monthly_compute_usd: base.gross_monthly_compute_cost_usd,
        head_monthly_compute_usd: head.gross_monthly_compute_cost_usd,
        delta_monthly_compute_usd: delta_compute,
        delta_monthly_compute_pct: delta_pct,
        base_monthly_dev_wait_usd: base.monthly_dev_wait_cost_usd,
        head_monthly_dev_wait_usd: head.monthly_dev_wait_cost_usd,
        delta_monthly_dev_wait_usd: delta_wait,
        base_free_quota_burn_pct: base.free_tier_quota_burn_pct,
        head_free_quota_burn_pct: head.free_tier_quota_burn_pct,
        job_changes: changes,
        head_report: head.clone(),
    }
}
