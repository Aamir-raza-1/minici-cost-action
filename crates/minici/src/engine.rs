//! Universal Combinatorial CI/CD DAG, Matrix & Cost Engine (`crates/minici/src/engine.rs`).
//!
//! Pure Cost Domain Engine (`Infracost for CI/CD` — Zero External Crate Dependencies):
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
    // 1. Build Minutes Cost (https://cicdcost.com/build-minutes-cost)
    pub rounding_tax_mins_per_run: f64,
    pub monthly_rounding_tax_usd: f64,
    pub linux_mins_per_run: f64,
    pub windows_mins_per_run: f64,
    pub macos_mins_per_run: f64,
    pub os_weighted_free_tier_mins_per_run: f64,
    pub monthly_os_weighted_free_tier_mins: f64,
    pub monthly_flake_retry_10pct_overhead_usd: f64,
    // 2. CI Runner Cost — Infracost Cloud Delegated (https://cicdcost.com/runner-cost)
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
    // 3. Artifact Storage Cost — Infracost S3/ECR Delegated (https://cicdcost.com/artifact-storage-cost)
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
    // 4. Parallelism Cost (https://cicdcost.com/parallelism-cost)
    pub peak_concurrent_jobs_per_run: usize,
    pub hosted_concurrent_jobs_per_run: usize,
    pub macos_concurrent_jobs_per_run: usize,
    pub self_hosted_or_microvm_jobs_per_run: usize,
    pub vendor_free_concurrent_cap: usize,
    pub vendor_team_concurrent_cap: usize,
    pub parallelism_tier_upgrade_tax_monthly_usd: f64,
    // 5. Log Retention Cost — Infracost CloudWatch/S3 Delegated (https://cicdcost.com/log-retention-cost)
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
}

impl CostComponentsBreakdown {
    pub fn to_json(&self) -> String {
        format!(
            "{{\n    \"build_minutes_cost\": {{\"rounding_tax_mins_per_run\":{:.2},\"monthly_rounding_tax_usd\":{:.2},\"linux_mins_per_run\":{:.2},\"windows_mins_per_run\":{:.2},\"macos_mins_per_run\":{:.2},\"os_weighted_free_tier_mins_per_run\":{:.2},\"monthly_os_weighted_free_tier_mins\":{:.2},\"monthly_flake_retry_10pct_overhead_usd\":{:.2}}},\n    \"runner_cost\": {{\"infracost_delegated\":true,\"infracost_iac_files_detected\":{},\"infracost_resources\":[\"aws_instance\",\"aws_ebs_volume\",\"google_compute_instance\",\"azurerm_linux_virtual_machine\"],\"hosted_effective_hourly_rate_usd\":{:.2},\"infracost_recommended_ec2_instance\":\"{}\",\"infracost_ec2_on_demand_hourly_usd\":{:.4},\"infracost_ec2_on_demand_monthly_usd\":{:.2},\"infracost_ec2_spot_hourly_usd\":{:.3},\"infracost_ec2_surrounding_overhead_monthly_usd\":{:.2},\"macos_self_hosted_mac_mini_monthly_usd\":{:.2},\"self_hosted_breakeven_mins_t3_medium\":{:.0},\"self_hosted_breakeven_mins_c5_4xlarge_spot\":{:.0}}},\n    \"artifact_storage_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_s3_bucket\",\"aws_ecr_repository\"],\"jobs_uploading_artifacts\":{},\"default_retention_days\":{},\"configured_retention_days\":{},\"steady_state_stored_gb_configured\":{:.2},\"monthly_vendor_storage_configured_usd\":{:.2},\"monthly_infracost_s3_configured_usd\":{:.2},\"steady_state_stored_gb_90d\":{:.2},\"steady_state_stored_gb_14d\":{:.2},\"monthly_vendor_storage_90d_usd\":{:.2},\"monthly_vendor_storage_14d_usd\":{:.2},\"monthly_infracost_s3_standard_90d_usd\":{:.2},\"monthly_infracost_s3_intelligent_14d_usd\":{:.2}}},\n    \"parallelism_cost\": {{\"peak_concurrent_jobs_per_run\":{},\"hosted_concurrent_jobs_per_run\":{},\"macos_concurrent_jobs_per_run\":{},\"self_hosted_or_microvm_jobs_per_run\":{},\"vendor_free_concurrent_cap\":{},\"vendor_team_concurrent_cap\":{},\"parallelism_tier_upgrade_tax_monthly_usd\":{:.2}}},\n    \"log_retention_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_cloudwatch_log_group\",\"aws_s3_bucket\"],\"detected_log_language_profile\":\"{}\",\"est_log_mb_per_run\":{:.2},\"monthly_log_volume_gb\":{:.2},\"vendor_default_log_retention_days\":{},\"monthly_vendor_default_log_cost_usd\":{:.2},\"monthly_datadog_30d_log_shipping_usd\":{:.2},\"monthly_datadog_90d_log_shipping_usd\":{:.2},\"monthly_splunk_30d_log_shipping_usd\":{:.2},\"monthly_infracost_loki_s3_usd\":{:.2}}},\n    \"cross_region_egress_cost\": {{\"infracost_delegated\":true,\"infracost_resources\":[\"aws_nat_gateway\",\"aws_vpc_endpoint\",\"google_compute_router_nat\",\"azurerm_nat_gateway\"],\"infracost_cloud_region\":\"{}\",\"est_transfer_gb_per_run\":{:.2},\"monthly_transfer_gb\":{:.2},\"infracost_aws_internet_egress_per_gb_usd\":{:.3},\"infracost_aws_nat_gateway_per_gb_usd\":{:.3},\"infracost_aws_cross_az_per_gb_usd\":{:.3},\"infracost_aws_cross_region_per_gb_usd\":{:.3},\"monthly_infracost_private_subnet_nat_worst_case_usd\":{:.2},\"monthly_infracost_cross_region_registry_usd\":{:.2},\"monthly_infracost_vpc_endpoint_cache_optimized_usd\":{:.2}}},\n    \"developer_wait_time_cost\": {{\"productivity_loss_factor\":{:.2},\"blended_fully_loaded_hourly_usd\":{:.2},\"monthly_wait_hours\":{:.2},\"monthly_productive_payroll_loss_usd\":{:.2},\"monthly_blended_100hr_productive_loss_usd\":{:.2},\"wait_to_compute_cost_ratio\":{:.2},\"lead_time_feedback_tier\":\"{}\"}},\n    \"failed_build_rerun_cost\": {{\"estimated_flake_rate_pct\":{:.1},\"has_job_level_auto_retry\":{},\"monthly_flaky_reruns\":{:.2},\"monthly_rerun_compute_cost_usd\":{:.2},\"avg_investigation_mins_per_flake\":{:.1},\"monthly_flake_investigation_hours\":{:.2},\"monthly_flake_investigation_payroll_usd\":{:.2},\"monthly_total_immediate_flake_cost_usd\":{:.2},\"monthly_savings_at_1pct_target_usd\":{:.2}}},\n    \"build_cache_savings\": {{\"stack_cache_profile\":\"{}\",\"realistic_saving_pct_min\":{:.1},\"realistic_saving_pct_max\":{:.1},\"jobs_with_cache\":{},\"jobs_missing_cache\":{},\"has_restore_keys_fallback\":{},\"has_docker_buildkit_cache\":{},\"estimated_cache_hit_rate_pct\":{:.1},\"vendor_cache_size_limit\":\"{}\",\"vendor_cache_eviction_policy\":\"{}\",\"monthly_uncached_compute_usd\":{:.2},\"monthly_unrealized_cache_savings_usd\":{:.2}}},\n    \"flaky_test_ci_cd_tax\": {{\"monthly_immediate_flake_cost_usd\":{:.2},\"deferred_velocity_drag_multiplier\":{:.1},\"monthly_deferred_velocity_drag_usd\":{:.2},\"monthly_combined_flake_tax_usd\":{:.2},\"deflake_project_est_cost_usd\":{:.2},\"deflake_payback_weeks\":{:.2}}},\n    \"branch_protection_cost\": {{\"required_checks_count\":{},\"monthly_required_check_executions\":{},\"workflows_with_concurrency_cancel\":{},\"workflows_missing_concurrency_cancel\":{},\"workflows_with_path_filters\":{},\"workflows_missing_path_filters\":{},\"has_affected_only_graph_tool\":{},\"slow_blocking_checks_over_10m\":{},\"monthly_superseded_run_waste_usd\":{:.2},\"monthly_unfiltered_check_savings_usd\":{:.2}}}\n  }}",
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
            self.monthly_unfiltered_check_savings_usd
        )
    }
}

#[derive(Debug, Clone)]
pub struct BreakdownReport {
    pub target_path: String,
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
            "{{\n  \"target_path\": \"{}\",\n  \"execution_time_us\": {},\n  \"execution_time_ms\": {:.3},\n  \"detected_providers\": [{}],\n  \"usage\": {},\n  \"total_workflows\": {},\n  \"total_defined_jobs\": {},\n  \"total_expanded_matrix_jobs\": {},\n  \"raw_duration_mins_per_run\": {:.2},\n  \"critical_path_wait_mins_per_run\": {:.2},\n  \"billed_mins_per_run\": {:.2},\n  \"shard_setup_duplication_mins_per_run\": {:.2},\n  \"clones_per_ci_run\": {},\n  \"monthly_runs\": {:.1},\n  \"monthly_billed_mins\": {:.2},\n  \"free_tier_included_mins\": {:.1},\n  \"free_tier_quota_burn_pct\": {:.1},\n  \"effective_tier\": \"{}\",\n  \"gross_compute_cost_per_run_usd\": {:.4},\n  \"gross_monthly_compute_cost_usd\": {:.2},\n  \"discount_allowance_monthly_usd\": {:.2},\n  \"net_billed_monthly_compute_cost_usd\": {:.2},\n  \"third_party_runner_monthly_cost_usd\": {:.2},\n  \"dev_wait_cost_per_run_usd\": {:.2},\n  \"monthly_dev_wait_cost_usd\": {:.2},\n  \"monthly_artifact_storage_vendor_usd\": {:.2},\n  \"monthly_artifact_storage_s3_usd\": {:.2},\n  \"monthly_blacksmith_cost_usd\": {:.2},\n  \"monthly_self_hosted_spot_cost_usd\": {:.2},\n  \"exceeds_5000m_self_hosted_breakeven\": {},\n  \"cost_components\": {},\n  \"jobs\": [\n    {}\n  ]\n}}",
            json_escape(&self.target_path),
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

fn line_has_cache_signal(low_line: &str) -> bool {
    low_line.contains("actions/cache")
        || low_line.contains("cache:")
        || low_line.contains("cache-key:")
        || low_line.contains("cache-from")
        || low_line.contains("cache-to")
        || low_line.contains("setup-docker-builder")
        || low_line.contains("rust-cache")
        || low_line.contains("sccache")
        || low_line.contains("setup-gradle")
        || low_line.contains("save_cache")
        || low_line.contains("restore_cache")
        || low_line.contains("./.github/actions/")
        || low_line.contains("setup-node")
        || low_line.contains("setup-python")
        || low_line.contains("setup-go")
        || low_line.contains("setup-uv")
}

/// Fast zero-allocation scanner for a single GitHub Actions job block (`lines` belonging to one job).
fn analyze_github_job_block(
    wf_rel: &str,
    job_id: &str,
    job_lines: &[&str],
    has_top_concurrency: bool,
    has_top_cache: bool,
    out_jobs: &mut Vec<ExpandedJobCost>,
) {
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

        // Steps & duration heuristics
        if trimmed.starts_with("steps:") {
            saw_steps = true;
            continue;
        }
        if saw_steps {
            if trimmed.contains("actions/checkout") {
                step_mins += 0.35;
            } else if trimmed.starts_with("filter:")
                || trimmed.starts_with("sparse-checkout:")
                || (trimmed.starts_with("fetch-depth:") && trimmed.ends_with('1'))
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
            } else if trimmed.contains("pytest")
                || trimmed.contains("cargo test")
                || trimmed.contains("npm test")
                || (trimmed.contains("vitest")
                    && !trimmed.contains(".vitest-reports")
                    && !trimmed.contains("--merge-reports")
                    && !trimmed.starts_with("path:"))
                || trimmed.contains("go test")
                || trimmed.contains("mvn test")
            {
                step_mins += 2.5;
            } else if trimmed.contains("docker build")
                || trimmed.contains("cargo build --release")
            {
                step_mins += 3.5;
            } else if trimmed.starts_with("- name:") || trimmed.starts_with("- run:") || trimmed.starts_with("- uses:") {
                step_mins += 0.25;
            }
        }
    }

    // Pure local reusable-workflow caller stubs (`uses: ./.github/workflows/...` with no `runs-on:` or `steps:`)
    // orchestrate another workflow already present in `.github/workflows/` and do not allocate their own runner VM.
    if uses_local_reusable_workflow && !saw_runs_on && !saw_steps {
        return;
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
            matrix_os_list = include_os_list;
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
            return;
        }
        let total_os_items = matrix_os_list.len().max(1);
        let raw_bucket_jobs: Vec<usize> = os_buckets
            .iter()
            .map(|(_, c)| ((product_count * *c) / total_os_items).max(1))
            .collect();
        let others_sum: usize = raw_bucket_jobs.iter().skip(1).sum();
        for (idx, (os_label, _)) in os_buckets.iter().enumerate() {
            // Distribute any net include/exclude delta cleanly onto the first OS bucket so total jobs == matrix_count
            let per_os_jobs = if idx == 0 {
                matrix_count.saturating_sub(others_sum).max(1)
            } else {
                raw_bucket_jobs[idx]
            };
            let sku = resolve_runner_sku(os_label);
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
                runner_label: os_label.clone(),
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
}

fn parse_github_workflow_file(wf_path: &Path, rel_name: &str, out_jobs: &mut Vec<ExpandedJobCost>) {
    let content = match fs::read_to_string(wf_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let all_lines: Vec<&str> = content.lines().collect();
    let mut has_top_concurrency = false;
    let mut has_top_cache = false;
    let mut in_jobs = false;
    let mut jobs_indent = 0usize;
    let mut job_header_indent: Option<usize> = None;
    let mut current_job_id: Option<String> = None;
    let mut current_job_lines: Vec<&str> = Vec::new();

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
                    analyze_github_job_block(
                        rel_name,
                        &jid,
                        &current_job_lines,
                        has_top_concurrency,
                        has_top_cache,
                        out_jobs,
                    );
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
                    analyze_github_job_block(
                        rel_name,
                        &jid,
                        &current_job_lines,
                        has_top_concurrency,
                        has_top_cache,
                        out_jobs,
                    );
                    current_job_lines.clear();
                }
                current_job_id = Some(trimmed.trim_end_matches(':').trim().to_string());
            } else if current_job_id.is_some() {
                current_job_lines.push(line);
            }
        }
    }

    if let Some(jid) = current_job_id.take() {
        analyze_github_job_block(
            rel_name,
            &jid,
            &current_job_lines,
            has_top_concurrency,
            has_top_cache,
            out_jobs,
        );
    }
}

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
    let mut job_names: Vec<String> = Vec::new();
    let mut sku_label = provider_hint.to_string();
    let mut uploads_artifacts = false;
    let mut artifact_retention_days: Option<u32> = None;
    let mut has_interruptible = false;
    let mut has_cache = false;

    for line in content.lines() {
        let clean = strip_comment(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line_has_cache_signal(&trimmed.to_ascii_lowercase()) {
            has_cache = true;
        }
        let ind = indent_of(clean);
        if ind == 0 && trimmed.ends_with(':') && !trimmed.starts_with('.') {
            let key = trimmed.trim_end_matches(':').trim();
            if !matches!(
                key,
                "stages"
                    | "variables"
                    | "default"
                    | "include"
                    | "workflow"
                    | "image"
                    | "version"
                    | "orbs"
                    | "pipelines"
                    | "phases"
                    | "jobs"
            ) {
                job_names.push(key.to_string());
            }
        }
        if trimmed.starts_with("tags:") {
            let list = parse_inline_yaml_list(trimmed);
            if let Some(first) = list.first() {
                sku_label = first.clone();
            }
        }
        if trimmed.starts_with("resource_class:") {
            if let Some(rc) = trimmed.splitn(2, ':').nth(1) {
                sku_label = format!("docker-{}", rc.trim().trim_matches('"'));
            }
        }
        if trimmed.starts_with("size:") {
            if let Some(sz) = trimmed.splitn(2, ':').nth(1) {
                sku_label = format!("bitbucket-{}", sz.trim().trim_matches('"'));
            }
        }
        if trimmed.starts_with("artifacts:") || trimmed.starts_with("store_artifacts:") {
            uploads_artifacts = true;
        }
        if trimmed.starts_with("expire_in:") || trimmed.starts_with("retention-days:") {
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                if let Some(days) = parse_retention_days_value(val) {
                    artifact_retention_days = Some(days);
                    uploads_artifacts = true;
                }
            }
        }
        if trimmed.starts_with("interruptible: true") {
            has_interruptible = true;
        }
    }

    if job_names.is_empty() {
        job_names.push("default-pipeline".to_string());
    }

    let sku = resolve_runner_sku(&sku_label);
    let raw_mins = 4.0_f64;
    for jname in job_names {
        out_jobs.push(ExpandedJobCost {
            workflow_file: rel_name.to_string(),
            job_id: jname,
            matrix_combination_count: 1,
            runner_label: sku.sku_label.to_string(),
            provider: sku.provider.to_string(),
            os: sku.os.to_string(),
            vcpus: sku.vcpus,
            est_raw_duration_mins_per_job: raw_mins,
            billed_mins_per_job: raw_mins,
            total_billed_mins_all_matrix: raw_mins,
            rate_per_min_usd: sku.rate_per_min_usd,
            run_compute_cost_usd: ((raw_mins * sku.rate_per_min_usd) * 10000.0).round() / 10000.0,
            blacksmith_run_cost_usd: ((raw_mins * sku.blacksmith_equiv_rate_usd) * 10000.0).round()
                / 10000.0,
            self_hosted_spot_run_cost_usd: ((raw_mins * sku.self_hosted_spot_rate_usd) * 10000.0)
                .round()
                / 10000.0,
            has_concurrency_cancel: has_interruptible,
            has_sparse_or_shallow_clone: false,
            uses_cache: has_cache,
            uploads_artifacts,
            artifact_retention_days,
        });
    }
}

/// Inspects repository manifest files, `.tf`/`terragrunt.hcl` IaC files, and CI workflow YAML contents
/// to classify the 9 `cicdcost.com/log-retention-cost` language profiles and detect Infracost IaC presence.
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

    // 2. Classify into the 9 official `cicdcost.com/log-retention-cost` language profiles
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
}

fn workflow_triggers_on_pr_or_push(raw_low: &str) -> bool {
    let mut has_on = false;
    let mut in_on = false;
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
                if let Some(rhs) = trimmed.splitn(2, ':').nth(1) {
                    let r = rhs.trim();
                    if r.contains("push") || r.contains("pull_request") {
                        return true;
                    }
                }
            } else {
                in_on = false;
            }
        } else if in_on
            && (trimmed.starts_with("push")
                || trimmed.starts_with("pull_request")
                || trimmed.starts_with("- push")
                || trimmed.starts_with("- pull_request"))
        {
            return true;
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
            || low.contains("bazel test")
            || low.contains("pants test")
        {
            sig.has_affected_only_graph_tool = true;
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
        if has_cancel {
            sig.workflows_with_concurrency_cancel += 1;
        } else if triggers_pr_or_push {
            sig.workflows_missing_concurrency_cancel += 1;
            sig.uncancelled_workflow_files.push(wf_name.clone());
        }

        let has_paths = low.contains("paths:")
            || low.contains("paths-ignore:")
            || low.contains("dorny/paths-filter")
            || low.contains("tj-actions/changed-files")
            || low.contains("rules:changes")
            || low.contains("changes:");
        if has_paths {
            sig.workflows_with_path_filters += 1;
        } else if triggers_pr_or_push {
            sig.workflows_missing_path_filters += 1;
            sig.unfiltered_workflow_files.push(wf_name.clone());
        }
    }

    sig
}

pub fn analyze_path(target: &Path, usage_override: Option<UsageConfig>) -> BreakdownReport {
    let t0 = Instant::now();
    let usage = load_usage_config(target, usage_override);
    let mut jobs: Vec<ExpandedJobCost> = Vec::new();
    let mut workflow_count = 0usize;

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
            parse_github_workflow_file(target, fname, &mut jobs);
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
                    parse_github_workflow_file(&p, &rel, &mut jobs);
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
                                parse_github_workflow_file(&p, &rel, &mut jobs);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut providers: Vec<String> = jobs.iter().map(|j| j.provider.clone()).collect();
    providers.sort();
    providers.dedup();

    let total_defined_jobs = jobs.len();
    let total_expanded_matrix_jobs: usize = jobs.iter().map(|j| j.matrix_combination_count).sum();
    let raw_duration_mins_per_run: f64 = jobs
        .iter()
        .map(|j| j.est_raw_duration_mins_per_job * (j.matrix_combination_count as f64))
        .sum();
    let critical_path_wait_mins_per_run: f64 = jobs
        .iter()
        .map(|j| j.est_raw_duration_mins_per_job)
        .fold(0.0_f64, f64::max);
    let billed_mins_per_run: f64 = jobs.iter().map(|j| j.total_billed_mins_all_matrix).sum();

    let monthly_runs = if workflow_count == 0 {
        0.0
    } else {
        (usage.prs_per_week * usage.pushes_per_pr * 4.33).round()
    };
    let monthly_billed_mins = (billed_mins_per_run * monthly_runs * 100.0).round() / 100.0;

    let primary_sku = jobs
        .first()
        .map(|j| resolve_runner_sku(&j.runner_label))
        .unwrap_or_else(|| resolve_runner_sku("ubuntu-latest"));
    let free_tier_included_mins = if primary_sku.free_tier_mins_per_month > 0.0 {
        primary_sku.free_tier_mins_per_month
    } else {
        2000.0
    };
    let free_tier_quota_burn_pct =
        ((monthly_billed_mins / free_tier_included_mins) * 100.0 * 10.0).round() / 10.0;

    let effective_tier = match usage.tier.as_str() {
        "free" => "free".to_string(),
        "paid" => "paid".to_string(),
        _ => {
            if monthly_billed_mins <= free_tier_included_mins {
                "free (within included quota)".to_string()
            } else {
                "paid (exceeds free quota)".to_string()
            }
        }
    };

    let gross_compute_cost_per_run_usd =
        (jobs.iter().map(|j| j.run_compute_cost_usd).sum::<f64>() * 10000.0).round() / 10000.0;
    let gross_monthly_compute_cost_usd =
        ((gross_compute_cost_per_run_usd * monthly_runs) * 100.0).round() / 100.0;

    let net_billed_monthly_compute_cost_usd = if effective_tier.starts_with("free") {
        0.0
    } else {
        gross_monthly_compute_cost_usd
    };
    let discount_allowance_monthly_usd =
        ((gross_monthly_compute_cost_usd - net_billed_monthly_compute_cost_usd).max(0.0) * 100.0)
            .round()
            / 100.0;
    let third_party_runner_monthly_cost_usd = ((jobs
        .iter()
        .filter(|j| {
            let l = j.runner_label.to_lowercase();
            l.contains("blacksmith")
                || l.contains("warpbuild")
                || l.contains("buildjet")
                || l.contains("namespace")
        })
        .map(|j| j.run_compute_cost_usd)
        .sum::<f64>()
        * monthly_runs)
        * 100.0)
        .round()
        / 100.0;
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
    let active_gb = if jobs.is_empty() {
        0.0
    } else if any_artifacts {
        usage.artifact_gb_per_run * (monthly_runs * (usage.artifact_retention_days as f64 / 30.0))
    } else {
        usage.artifact_gb_per_run * 10.0
    };
    let monthly_artifact_storage_vendor_usd =
        ((active_gb * ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO) * 100.0).round() / 100.0;
    let monthly_artifact_storage_s3_usd =
        ((active_gb * ARTIFACT_STORAGE_S3_USD_PER_GB_MO) * 100.0).round() / 100.0;

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

    // --- Compute 6 Line-by-Line CI/CD Invoice Cost Components (cicdcost.com + Infracost Cloud Delegated) ---
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
    let mut jobs_uploading_artifacts = 0_usize;
    let mut yaml_configured_retention_days: Option<u32> = None;
    let mut macos_concurrent_jobs_per_run = 0_usize;
    let mut self_hosted_or_microvm_jobs_per_run = 0_usize;

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
        if rl_low.contains("arm")
            || rl_low.contains("aarch64")
            || rl_low.contains("c7g")
            || rl_low.contains("c8g")
            || resolve_runner_sku(&j.runner_label).arch == "arm64"
        {
            has_arm_jobs = true;
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
    let (ec2_label, ec2_od_hr, ec2_od_mo, ec2_spot_hr) =
        infracost_ec2_runner_profile(max_vcpus, has_arm_jobs);
    let infracost_ec2_surrounding_overhead_monthly_usd = if jobs.is_empty() { 0.0 } else { 8.40 };
    let macos_self_hosted_mac_mini_monthly_usd = if macos_concurrent_jobs_per_run > 0 {
        25.00
    } else {
        0.0
    };

    // 3. Artifact Storage (90d default vs configured YAML retention-days vs 14d optimized + Infracost S3)
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
        usage.artifact_gb_per_run * (monthly_runs * 0.1)
    };
    let steady_state_stored_gb_configured =
        ((monthly_created_gb * (configured_retention_days as f64 / 30.0)) * 100.0).round() / 100.0;
    let monthly_vendor_storage_configured_usd =
        ((steady_state_stored_gb_configured * ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO) * 100.0)
            .round()
            / 100.0;
    let monthly_infracost_s3_configured_usd =
        ((steady_state_stored_gb_configured * ARTIFACT_STORAGE_S3_USD_PER_GB_MO) * 100.0).round()
            / 100.0;
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

    // 4. Parallelism Cost (Concurrent-job caps & Tier-Upgrade Tax per cicdcost.com/parallelism-cost)
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

    // 5. Log Retention Cost (Per-language log volume across all 9 cicdcost.com profiles + IaC detection)
    let (detected_log_language_profile, base_log_mb_per_job, infracost_iac_files_detected) =
        detect_language_and_iac_signals(target, &jobs);
    let est_log_mb_per_run = if total_expanded_matrix_jobs == 0 {
        0.0
    } else {
        // Includes 15% failed-build stack-trace amplification per cicdcost.com/log-retention-cost
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

    // 6. Cross-Region Egress & NAT Gateway Cost (Infracost aws_nat_gateway / aws_vpc_endpoint delegated)
    let infracost_cloud_region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());
    let infracost_aws_nat_gateway_per_gb_usd =
        infracost_aws_nat_gateway_rate_usd(&infracost_cloud_region);
    // Estimate per-run container/dependency/cache transfer GB (~0.18 GB per expanded job + 0.45 GB per artifact-uploading job)
    let est_transfer_gb_per_run = if total_expanded_matrix_jobs == 0 {
        0.0
    } else {
        (((total_expanded_matrix_jobs as f64) * 0.18 + (jobs_uploading_artifacts as f64) * 0.35)
            * 100.0)
            .round()
            / 100.0
    };
    let monthly_transfer_gb = ((est_transfer_gb_per_run * monthly_runs) * 100.0).round() / 100.0;
    let worst_case_rate_per_gb =
        AWS_INTERNET_EGRESS_USD_PER_GB + infracost_aws_nat_gateway_per_gb_usd;
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
    };

    let elapsed_us = t0.elapsed().as_micros().max(1);

    BreakdownReport {
        target_path: target.display().to_string(),
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
            {
                changes.push(format!(
                    "Modified `{}` (`{}`): `{}x {}` (${:.3}/run) -> `{}x {}` (${:.3}/run)",
                    hj.job_id,
                    hj.workflow_file,
                    bj.matrix_combination_count,
                    bj.runner_label,
                    bj.run_compute_cost_usd,
                    hj.matrix_combination_count,
                    hj.runner_label,
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
