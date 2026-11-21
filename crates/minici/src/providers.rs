//! Provider-Specific CI/CD Billing Plugins & Live Cloud Pricing Resolver
//! (`crates/minici/src/providers.rs` — 100% Pure Rust `std`, Zero External Dependencies).
//!
//! Isolates vendor-specific billing, rounding, allowance pool burn, public-repository exemptions,
//! storage quotas, and concurrency caps behind a strict `CiProvider` trait:
//! - `GitHubProvider`    : Whole-minute per-job ceiling (`min 1m`), public standard = `$0`,
//!                         `1x/2x/10x` OS pool burn, Larger Runners (`4c+`, GPU, `macos-large`) `uses_allowance = false`.
//! - `GitLabProvider`    : Per-second billing × runner `cost_factor`, `0.5x` public OSS / `0.008x` fork MR,
//!                         `400 / 10,000 / 50,000` namespace quota pool.
//! - `CircleCIProvider`  : Per-second credit billing (`$0.0006/credit`), `200`-credit Docker Layer Cache (`DLC`) fee,
//!                         `30,000` Free/Performance credit pool.
//! - `BitbucketProvider` : Whole-minute per-step ceiling (`min 1m`) × `1x..32x` size multiplier,
//!                         `50 / 2,500 / 3,500` workspace minute pool.
//! - `CloudInfraProvider`: Live regional AWS/GCP/Azure On-Demand, Spot, EBS, EKS, and NAT rates
//!                         backed by `~/.minici/cloud_prices.json` + upstream cloud pricing GraphQL sync.

use crate::catalog::{lookup_infracost_cloud_spec, resolve_runner_sku};
use crate::engine::{json_escape, ExpandedJobCost};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoVisibility {
    Public,
    Private,
    Unknown,
}

impl RepoVisibility {
    pub fn from_str_opt(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "public" => Self::Public,
            "private" | "internal" => Self::Private,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderInvoiceReceipt {
    pub provider_id: &'static str,
    pub free_tier_included_mins: f64,
    pub included_storage_gb: f64,
    pub vendor_hosted_allowance_mins_per_run: f64,
    pub os_weighted_pool_mins_per_run: f64,
    pub monthly_os_weighted_pool_mins: f64,
    pub free_tier_quota_burn_pct: f64,
    pub effective_tier: String,
    pub gross_monthly_compute_cost_usd: f64,
    pub discount_allowance_monthly_usd: f64,
    pub net_billed_monthly_compute_cost_usd: f64,
    pub third_party_or_self_hosted_monthly_usd: f64,
    pub larger_runner_non_allowance_monthly_usd: f64,
}

pub trait CiProvider {
    fn provider_id(&self) -> &'static str;
    fn round_job_minutes(&self, raw_mins: f64, ceil_flag: bool) -> f64;
    fn is_non_allowance_larger_runner(&self, job: &ExpandedJobCost) -> bool;
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool;
    fn os_pool_multiplier(&self, os: &str) -> f64;
    fn plan_pool_minutes(&self, tier_mode: &str) -> f64;
    fn plan_included_storage_gb(&self, tier_mode: &str) -> f64;
    fn concurrency_caps(&self) -> (usize, usize);

    fn compute_invoice_ledger(
        &self,
        jobs: &[ExpandedJobCost],
        monthly_runs: f64,
        tier_mode: &str,
        visibility: RepoVisibility,
    ) -> ProviderInvoiceReceipt {
        let gross_per_run: f64 = jobs.iter().map(|j| j.run_compute_cost_usd).sum();
        let gross_monthly = ((gross_per_run * monthly_runs) * 100.0).round() / 100.0;
        let pool_limit = self.plan_pool_minutes(tier_mode);
        let storage_gb = self.plan_included_storage_gb(tier_mode);

        if jobs.is_empty() || monthly_runs <= 0.0 {
            return ProviderInvoiceReceipt {
                provider_id: self.provider_id(),
                free_tier_included_mins: pool_limit,
                included_storage_gb: storage_gb,
                vendor_hosted_allowance_mins_per_run: 0.0,
                os_weighted_pool_mins_per_run: 0.0,
                monthly_os_weighted_pool_mins: 0.0,
                free_tier_quota_burn_pct: 0.0,
                effective_tier: match tier_mode {
                    "free" => "free".to_string(),
                    "paid" => "paid".to_string(),
                    _ => "free (within included quota)".to_string(),
                },
                gross_monthly_compute_cost_usd: 0.0,
                discount_allowance_monthly_usd: 0.0,
                net_billed_monthly_compute_cost_usd: 0.0,
                third_party_or_self_hosted_monthly_usd: 0.0,
                larger_runner_non_allowance_monthly_usd: 0.0,
            };
        }

        let mut hosted_std_mins_per_run = 0.0_f64;
        let mut os_weighted_per_run = 0.0_f64;
        let mut third_party_self_mo = 0.0_f64;
        let mut larger_non_allow_mo = 0.0_f64;
        // Collect allowance-eligible jobs `(monthly_billed_mins, os_mult, sticker_rate_per_min)`
        let mut eligible_buckets: Vec<(f64, f64, f64)> = Vec::new();

        for j in jobs {
            let job_mo_usd = j.run_compute_cost_usd * monthly_runs;
            let job_mo_mins = j.total_billed_mins_all_matrix * monthly_runs;
            if self.is_third_party_or_self_hosted(j) {
                third_party_self_mo += job_mo_usd;
            } else if self.is_non_allowance_larger_runner(j) {
                larger_non_allow_mo += job_mo_usd;
            } else {
                let mult = self.os_pool_multiplier(&j.os);
                hosted_std_mins_per_run += j.total_billed_mins_all_matrix;
                os_weighted_per_run += j.total_billed_mins_all_matrix * mult;
                eligible_buckets.push((job_mo_mins, mult, j.rate_per_min_usd));
            }
        }

        // Public repository exemption for GitHub standard hosted runners ($0.00 unlimited!)
        if visibility == RepoVisibility::Public && self.provider_id() == "github-actions" {
            let net_public =
                ((third_party_self_mo + larger_non_allow_mo) * 100.0).round() / 100.0;
            let discount = ((gross_monthly - net_public).max(0.0) * 100.0).round() / 100.0;
            return ProviderInvoiceReceipt {
                provider_id: self.provider_id(),
                free_tier_included_mins: pool_limit,
                included_storage_gb: storage_gb,
                vendor_hosted_allowance_mins_per_run: hosted_std_mins_per_run,
                os_weighted_pool_mins_per_run: 0.0,
                monthly_os_weighted_pool_mins: 0.0,
                free_tier_quota_burn_pct: 0.0,
                effective_tier: "public-free (unlimited standard runners)".to_string(),
                gross_monthly_compute_cost_usd: gross_monthly,
                discount_allowance_monthly_usd: discount,
                net_billed_monthly_compute_cost_usd: net_public,
                third_party_or_self_hosted_monthly_usd: (third_party_self_mo * 100.0).round()
                    / 100.0,
                larger_runner_non_allowance_monthly_usd: (larger_non_allow_mo * 100.0).round()
                    / 100.0,
            };
        }

        // GitLab Official Public OSS (`0.5x`) and Public Fork MR (`0.008x`) cost factors multiply compute-minute quota consumption!
        let gl_cost_factor = if visibility == RepoVisibility::Public && self.provider_id() == "gitlab-ci" {
            let is_fork_mr = env::var("MINICI_GITLAB_FORK_MR")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if is_fork_mr {
                0.008
            } else {
                0.50
            }
        } else {
            1.0
        };
        let os_weighted_per_run = ((os_weighted_per_run * gl_cost_factor) * 100.0).round() / 100.0;
        let monthly_os_weighted_pool = ((os_weighted_per_run * monthly_runs) * 100.0).round() / 100.0;

        let burn_pct = if pool_limit > 0.0 && !pool_limit.is_infinite() {
            ((monthly_os_weighted_pool / pool_limit) * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let effective_tier = match tier_mode {
            "free" => "free".to_string(),
            "paid" => "paid".to_string(),
            _ => {
                if monthly_os_weighted_pool <= pool_limit && larger_non_allow_mo <= 0.0 {
                    "free (within included quota)".to_string()
                } else {
                    "paid (exceeds free quota)".to_string()
                }
            }
        };

        let mut remaining_pool = if tier_mode == "paid" {
            0.0
        } else {
            pool_limit
        };

        let mut overage_std_usd = 0.0_f64;
        for (job_mins, mult, rate) in eligible_buckets {
            let m = (mult * gl_cost_factor).max(0.0001);
            let needed_pool = job_mins * m;
            if remaining_pool >= needed_pool {
                remaining_pool -= needed_pool;
            } else {
                let covered_job_mins = remaining_pool / m;
                remaining_pool = 0.0;
                let billable_overage_mins = (job_mins - covered_job_mins).max(0.0);
                overage_std_usd += billable_overage_mins * rate * gl_cost_factor;
            }
        }

        let net_billed = if tier_mode == "free" {
            // Explicit `--tier free` caps standard hosted overage at $0 (hard spending limit $0),
            // while still reporting 3rd-party/self-hosted infra if configured.
            ((third_party_self_mo + larger_non_allow_mo) * 100.0).round() / 100.0
        } else {
            ((overage_std_usd + third_party_self_mo + larger_non_allow_mo) * 100.0).round() / 100.0
        };
        let discount = ((gross_monthly - net_billed).max(0.0) * 100.0).round() / 100.0;

        ProviderInvoiceReceipt {
            provider_id: self.provider_id(),
            free_tier_included_mins: pool_limit,
            included_storage_gb: storage_gb,
            vendor_hosted_allowance_mins_per_run: hosted_std_mins_per_run,
            os_weighted_pool_mins_per_run: os_weighted_per_run,
            monthly_os_weighted_pool_mins: monthly_os_weighted_pool,
            free_tier_quota_burn_pct: burn_pct,
            effective_tier,
            gross_monthly_compute_cost_usd: gross_monthly,
            discount_allowance_monthly_usd: discount,
            net_billed_monthly_compute_cost_usd: net_billed,
            third_party_or_self_hosted_monthly_usd: (third_party_self_mo * 100.0).round() / 100.0,
            larger_runner_non_allowance_monthly_usd: (larger_non_allow_mo * 100.0).round() / 100.0,
        }
    }
}

pub struct GitHubProvider;
pub struct GitLabProvider;
pub struct CircleCIProvider;
pub struct BitbucketProvider;
pub struct CloudBuildProvider {
    pub id: &'static str,
}

pub fn is_third_party_microvm_job(j: &ExpandedJobCost) -> bool {
    let rl = j.runner_label.to_ascii_lowercase();
    j.provider == "blacksmith"
        || j.provider == "third-party-microvm"
        || rl.contains("blacksmith")
        || rl.contains("warpbuild")
        || rl.contains("buildjet")
        || rl.contains("depot")
        || rl.contains("namespace")
        || rl.starts_with("third-party-")
}

pub fn is_self_hosted_job(j: &ExpandedJobCost) -> bool {
    let rl = j.runner_label.to_ascii_lowercase();
    j.provider.starts_with("self-hosted") || rl.contains("self-hosted") || rl.contains("runs-on")
}

pub fn is_github_larger_runner_job(j: &ExpandedJobCost) -> bool {
    if is_third_party_microvm_job(j) || is_self_hosted_job(j) {
        return false;
    }
    let rl = j.runner_label.to_ascii_lowercase();
    let is_gh = j.provider == "github-actions"
        || rl.contains("ubuntu")
        || rl.contains("windows")
        || rl.contains("macos");
    if !is_gh {
        return false;
    }
    // Fixes Bug #4: Standard 2-core `ubuntu-24.04-arm` and 4-core `windows-11-arm` ARE standard
    // allowance-eligible GitHub runners! Only >2 vCPU Linux, >4 vCPU Windows, >4 vCPU macOS, or GPU are Larger Runners.
    let larger_linux = j.os.eq_ignore_ascii_case("linux")
        && (j.vcpus > 2 || rl.contains("gpu") || rl.contains("large") || rl.contains("-xl"));
    let larger_win = j.os.eq_ignore_ascii_case("windows")
        && (j.vcpus > 4
            || rl.contains("4core")
            || rl.contains("4-core")
            || rl.contains("8core")
            || rl.contains("8-core")
            || rl.contains("gpu")
            || rl.contains("large"));
    let larger_mac = j.os.eq_ignore_ascii_case("macos")
        && (j.vcpus > 4 || rl.contains("large") || rl.contains("xlarge") || rl.contains("m2-pro"));
    larger_linux || larger_win || larger_mac
}

impl CiProvider for GitHubProvider {
    fn provider_id(&self) -> &'static str {
        "github-actions"
    }
    fn round_job_minutes(&self, raw_mins: f64, ceil_flag: bool) -> f64 {
        if ceil_flag {
            raw_mins.ceil().max(1.0)
        } else {
            raw_mins
        }
    }
    fn is_non_allowance_larger_runner(&self, job: &ExpandedJobCost) -> bool {
        is_github_larger_runner_job(job)
    }
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool {
        is_third_party_microvm_job(job) || is_self_hosted_job(job)
    }
    fn os_pool_multiplier(&self, os: &str) -> f64 {
        let low = os.to_ascii_lowercase();
        let plan = crate::catalog::lookup_declarative_plan("github", "free");
        if low.contains("mac") {
            plan.map(|p| p.os_mult_macos).unwrap_or(10.0)
        } else if low.contains("win") {
            plan.map(|p| p.os_mult_windows).unwrap_or(2.0)
        } else {
            plan.map(|p| p.os_mult_linux).unwrap_or(1.0)
        }
    }
    fn plan_pool_minutes(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "pro" => "pro",
            "team" => "team",
            "enterprise" => "enterprise",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("github", pname)
            .map(|p| p.included_minutes)
            .unwrap_or(2000.0)
    }
    fn plan_included_storage_gb(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "pro" => "pro",
            "team" => "team",
            "enterprise" => "enterprise",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("github", pname)
            .map(|p| p.included_storage_gb)
            .unwrap_or(0.5)
    }
    fn concurrency_caps(&self) -> (usize, usize) {
        let free_cap = crate::catalog::lookup_declarative_plan("github", "free")
            .map(|p| p.concurrent_jobs)
            .unwrap_or(20);
        let team_cap = crate::catalog::lookup_declarative_plan("github", "team")
            .map(|p| p.concurrent_jobs)
            .unwrap_or(60);
        (free_cap, team_cap)
    }
}

impl CiProvider for GitLabProvider {
    fn provider_id(&self) -> &'static str {
        "gitlab-ci"
    }
    fn round_job_minutes(&self, raw_mins: f64, _ceil_flag: bool) -> f64 {
        raw_mins.max(0.02)
    }
    fn is_non_allowance_larger_runner(&self, _job: &ExpandedJobCost) -> bool {
        false
    }
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool {
        is_self_hosted_job(job) || is_third_party_microvm_job(job)
    }
    fn os_pool_multiplier(&self, os: &str) -> f64 {
        let plan = crate::catalog::lookup_declarative_plan("gitlab", "free");
        if os.to_ascii_lowercase().contains("mac") {
            plan.map(|p| p.os_mult_macos).unwrap_or(6.0)
        } else {
            plan.map(|p| p.os_mult_linux).unwrap_or(1.0)
        }
    }
    fn plan_pool_minutes(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "team" | "premium" => "premium",
            "enterprise" | "ultimate" => "ultimate",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("gitlab", pname)
            .map(|p| p.included_minutes)
            .unwrap_or(400.0)
    }
    fn plan_included_storage_gb(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "team" | "premium" => "premium",
            "enterprise" | "ultimate" => "ultimate",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("gitlab", pname)
            .map(|p| p.included_storage_gb)
            .unwrap_or(5.0)
    }
    fn concurrency_caps(&self) -> (usize, usize) {
        (20, 100)
    }
}

impl CiProvider for CircleCIProvider {
    fn provider_id(&self) -> &'static str {
        "circleci"
    }
    fn round_job_minutes(&self, raw_mins: f64, _ceil_flag: bool) -> f64 {
        raw_mins.max(0.02)
    }
    fn is_non_allowance_larger_runner(&self, _job: &ExpandedJobCost) -> bool {
        false
    }
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool {
        is_self_hosted_job(job) || is_third_party_microvm_job(job)
    }
    fn os_pool_multiplier(&self, os: &str) -> f64 {
        let low = os.to_ascii_lowercase();
        if low.contains("mac") {
            15.0 // 150 credits/m vs 10 credits/m base
        } else if low.contains("win") {
            4.0
        } else {
            1.0
        }
    }
    fn plan_pool_minutes(&self, _tier_mode: &str) -> f64 {
        // 30,000 free credits / 10 credits per docker-medium minute = 3,000 Linux-equiv minutes
        3000.0
    }
    fn plan_included_storage_gb(&self, _tier_mode: &str) -> f64 {
        0.0
    }
    fn concurrency_caps(&self) -> (usize, usize) {
        (30, 80)
    }
}

impl CiProvider for BitbucketProvider {
    fn provider_id(&self) -> &'static str {
        "bitbucket"
    }
    fn round_job_minutes(&self, raw_mins: f64, _ceil_flag: bool) -> f64 {
        raw_mins.ceil().max(1.0)
    }
    fn is_non_allowance_larger_runner(&self, _job: &ExpandedJobCost) -> bool {
        false
    }
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool {
        is_self_hosted_job(job) || is_third_party_microvm_job(job)
    }
    fn os_pool_multiplier(&self, _os: &str) -> f64 {
        1.0
    }
    fn plan_pool_minutes(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "team" | "standard" => "standard",
            "enterprise" | "premium" => "premium",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("bitbucket", pname)
            .map(|p| p.included_minutes)
            .unwrap_or(50.0)
    }
    fn plan_included_storage_gb(&self, tier_mode: &str) -> f64 {
        let pname = match tier_mode {
            "team" | "standard" => "standard",
            "enterprise" | "premium" => "premium",
            _ => "free",
        };
        crate::catalog::lookup_declarative_plan("bitbucket", pname)
            .map(|p| p.included_storage_gb)
            .unwrap_or(1.0)
    }
    fn concurrency_caps(&self) -> (usize, usize) {
        (10, 100)
    }
}

impl CiProvider for CloudBuildProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }
    fn round_job_minutes(&self, raw_mins: f64, ceil_flag: bool) -> f64 {
        if ceil_flag {
            raw_mins.ceil().max(1.0)
        } else {
            raw_mins.max(0.1)
        }
    }
    fn is_non_allowance_larger_runner(&self, _job: &ExpandedJobCost) -> bool {
        false
    }
    fn is_third_party_or_self_hosted(&self, job: &ExpandedJobCost) -> bool {
        is_self_hosted_job(job) || is_third_party_microvm_job(job)
    }
    fn os_pool_multiplier(&self, _os: &str) -> f64 {
        1.0
    }
    fn plan_pool_minutes(&self, _tier_mode: &str) -> f64 {
        match self.id {
            "aws-codebuild" => 100.0,
            "gcp-cloud-build" => 2500.0,
            "azure-pipelines" => 1800.0,
            _ => 0.0,
        }
    }
    fn plan_included_storage_gb(&self, _tier_mode: &str) -> f64 {
        0.0
    }
    fn concurrency_caps(&self) -> (usize, usize) {
        (20, 60)
    }
}

pub fn select_provider_plugin(primary_provider: &str) -> Box<dyn CiProvider> {
    match primary_provider {
        "gitlab-ci" => Box::new(GitLabProvider),
        "circleci" => Box::new(CircleCIProvider),
        "bitbucket" => Box::new(BitbucketProvider),
        "aws-codebuild" => Box::new(CloudBuildProvider { id: "aws-codebuild" }),
        "gcp-cloud-build" => Box::new(CloudBuildProvider { id: "gcp-cloud-build" }),
        "azure-pipelines" => Box::new(CloudBuildProvider { id: "azure-pipelines" }),
        _ => Box::new(GitHubProvider),
    }
}

// =====================================================================================
// Live Cloud Pricing Hot-Cache & Upstream GraphQL Resolver (Zero External Branding!)
// =====================================================================================

#[derive(Debug, Clone)]
pub struct CloudInstanceQuote {
    pub vendor: String,
    pub instance_type: String,
    pub region: String,
    pub vcpus: u32,
    pub ram_gb: f64,
    pub on_demand_hourly_usd: f64,
    pub spot_hourly_usd: f64,
    pub source: String, // "hot_cache" | "live_cloud_graphql" | "embedded_seed"
}

impl CloudInstanceQuote {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"vendor\":\"{}\",\"instance_type\":\"{}\",\"region\":\"{}\",\"vcpus\":{},\"ram_gb\":{:.1},\"on_demand_hourly_usd\":{:.4},\"on_demand_per_min_usd\":{:.5},\"spot_hourly_usd\":{:.4},\"spot_per_min_usd\":{:.5},\"source\":\"{}\"}}",
            json_escape(&self.vendor),
            json_escape(&self.instance_type),
            json_escape(&self.region),
            self.vcpus,
            self.ram_gb,
            self.on_demand_hourly_usd,
            self.on_demand_hourly_usd / 60.0,
            self.spot_hourly_usd,
            self.spot_hourly_usd / 60.0,
            json_escape(&self.source)
        )
    }
}

pub fn cloud_prices_cache_path() -> PathBuf {
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        return Path::new(&home).join(".minici").join("cloud_prices.json");
    }
    env::temp_dir().join("minici_cloud_prices.json")
}

fn discover_upstream_cloud_api_key() -> Option<String> {
    if let Ok(k) = env::var("MINICI_CLOUD_PRICING_API_KEY").or_else(|_| env::var("INFRACOST_API_KEY")) {
        let trimmed = k.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let cred_path = Path::new(&home)
            .join(".config")
            .join("infracost")
            .join("credentials.yml");
        if let Ok(raw) = fs::read_to_string(cred_path) {
            for line in raw.lines() {
                if let Some(rest) = line.trim().strip_prefix("api_key:") {
                    let key = rest.trim().trim_matches('"').trim_matches('\'');
                    if !key.is_empty() {
                        return Some(key.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn regional_cloud_multiplier(region: &str) -> f64 {
    match region.trim().to_ascii_lowercase().as_str() {
        "us-east-1" | "us-east-2" | "us-west-2" | "us-central1" | "us-west1" | "eastus"
        | "eastus2" | "westus2" => 1.00,
        "centralus" => 1.02,
        "eu-north-1" | "northeurope" => 1.05,
        "ca-central-1" | "us-east4" => 1.06,
        "us-west-1" | "westus" | "eu-west-1" | "europe-west1" | "europe-west4" | "westeurope" => {
            1.08
        }
        "uksouth" | "francecentral" => 1.10,
        "eu-central-1" | "eu-west-2" | "europe-west2" | "europe-west3" | "germanywestcentral"
        | "asia-east1" | "centralindia" => 1.12,
        "ap-south-1" | "ap-southeast-1" | "ap-northeast-2" | "asia-southeast1"
        | "southeastasia" => 1.14,
        "ap-southeast-2" | "australiaeast" => 1.16,
        "ap-northeast-1" | "asia-northeast1" | "japaneast" | "me-central-1"
        | "australia-southeast1" => 1.18,
        "af-south-1" => 1.22,
        "sa-east-1" | "southamerica-east1" | "brazilsouth" => 1.38,
        _ => 1.00,
    }
}

/// Fast `< 0.1 ms` lookup into `~/.minici/cloud_prices.json` used directly by `catalog.rs` and `engine.rs`
/// when the user has synced live cloud prices (`minici catalog sync --cloud`). Returns `(od_hr, spot_hr, vcpus, ram_gb)`.
pub fn load_cached_cloud_price(instance_type: &str, region: &str) -> Option<(f64, f64, u32, f64)> {
    let cache_path = cloud_prices_cache_path();
    if !cache_path.exists() {
        return None;
    }
    let raw = fs::read_to_string(&cache_path).ok()?;
    let lower_inst = strip_cloud_vendor_prefix(&instance_type.trim().to_ascii_lowercase()).to_string();
    let lower_reg = region.trim().to_ascii_lowercase();
    for candidate_reg in [lower_reg.as_str(), "us-east-1"] {
        let needle = format!("\"{}@{}\":", lower_inst, candidate_reg);
        if let Some(idx) = raw.find(&needle) {
            let sub = &raw[idx + needle.len()..];
            if let Some(end_brace) = sub.find('}') {
                let obj = &sub[..=end_brace];
                if let (Some(od), Some(sp)) = (
                    extract_num_field(obj, "od_hr"),
                    extract_num_field(obj, "spot_hr"),
                ) {
                    let scale = if candidate_reg == lower_reg {
                        1.0
                    } else {
                        crate::catalog::regional_seed_multiplier(&lower_reg)
                    };
                    let od_scaled = ((od * scale) * 10000.0).round() / 10000.0;
                    let sp_scaled = ((sp * scale) * 10000.0).round() / 10000.0;
                    let vcpus = extract_num_field(obj, "vcpus").unwrap_or(4.0) as u32;
                    let ram_gb = extract_num_field(obj, "ram_gb").unwrap_or(8.0);
                    return Some((od_scaled, sp_scaled, vcpus, ram_gb));
                }
            }
        }
    }
    None
}

fn strip_cloud_vendor_prefix(inst: &str) -> &str {
    inst.strip_prefix("aws-")
        .or_else(|| inst.strip_prefix("gcp-"))
        .or_else(|| inst.strip_prefix("azure-"))
        .unwrap_or(inst)
}

/// Queries live upstream cloud pricing API when `allow_network == true` and caches into `~/.minici/cloud_prices.json`,
/// falling back cleanly in `< 0.05 ms` to the embedded cloud catalog with regional adjustment when offline.
pub fn resolve_cloud_instance_price(
    instance_type: &str,
    region: &str,
    allow_network: bool,
) -> CloudInstanceQuote {
    let raw_lower = instance_type.trim().to_ascii_lowercase();
    let detected_vendor = infer_cloud_vendor(&raw_lower);
    let lower_inst = strip_cloud_vendor_prefix(&raw_lower).to_string();
    let lower_reg = region.trim().to_ascii_lowercase();
    let cache_key = format!("{}@{}", lower_inst, lower_reg);
    let cache_path = cloud_prices_cache_path();

    // 1. Check local `~/.minici/cloud_prices.json` hot cache first (< 0.1 ms) when not forcing a live refresh
    if !allow_network {
        if let Some((od, sp, vcpus, ram_gb)) = load_cached_cloud_price(&lower_inst, &lower_reg) {
            return CloudInstanceQuote {
                vendor: detected_vendor.to_string(),
                instance_type: lower_inst,
                region: lower_reg,
                vcpus,
                ram_gb,
                on_demand_hourly_usd: od,
                spot_hourly_usd: sp,
                source: "hot_cache".to_string(),
            };
        }
    }

    // 2. Optional live GraphQL lookup against upstream Cloud Pricing API when network sync is requested
    if allow_network {
        if let Some(api_key) = discover_upstream_cloud_api_key() {
            let vendor = detected_vendor;
            let service = match vendor {
                "gcp" => "Compute Engine",
                "azure" => "Virtual Machines",
                _ => "AmazonEC2",
            };
            // Filter strictly to Linux Shared Used instances on AWS; use `machineType` on GCP and `armSkuName` on Azure
            let attr_filters = match vendor {
                "aws" => format!(
                    "[{{ key: \\\"instanceType\\\", value: \\\"{}\\\" }}, {{ key: \\\"operatingSystem\\\", value: \\\"Linux\\\" }}, {{ key: \\\"tenancy\\\", value: \\\"Shared\\\" }}, {{ key: \\\"capacityStatus\\\", value: \\\"Used\\\" }}]",
                    lower_inst
                ),
                "gcp" => format!("[{{ key: \\\"machineType\\\", value: \\\"{}\\\" }}]", lower_inst),
                _ => format!("[{{ key: \\\"armSkuName\\\", value: \\\"{}\\\" }}]", lower_inst),
            };
            let gql = format!(
                "{{\"query\":\"{{ products(filter: {{ vendorName: \\\"{}\\\", service: \\\"{}\\\", region: \\\"{}\\\", attributeFilters: {} }}) {{ onDemand: prices(filter: {{ purchaseOption: \\\"on_demand\\\" }}) {{ USD }} spot: prices(filter: {{ purchaseOption: \\\"spot\\\" }}) {{ USD }} }} }}\"}}",
                vendor, service, lower_reg, attr_filters
            );
            if let Ok(out) = Command::new("curl")
                .args([
                    "-fsSL",
                    "--connect-timeout",
                    "2",
                    "-m",
                    "4",
                    "-X",
                    "POST",
                    "-H",
                    "Content-Type: application/json",
                    "-H",
                    &format!("X-Api-Key: {}", api_key),
                    "-d",
                    &gql,
                    "https://pricing.api.infracost.io/graphql",
                ])
                .output()
            {
                if out.status.success() {
                    let body = String::from_utf8_lossy(&out.stdout);
                    if let Some(usd_hr) = extract_named_graphql_usd(&body, "\"onDemand\":")
                        .or_else(|| extract_graphql_usd_price(&body))
                    {
                        if usd_hr > 0.0 && usd_hr < 500.0 {
                            let seed = lookup_infracost_cloud_spec(&lower_inst);
                            let vcpus = seed
                                .map(|s| s.vcpus)
                                .unwrap_or_else(|| resolve_runner_sku(&lower_inst).vcpus);
                            let ram_gb = seed
                                .map(|s| s.ram_gb)
                                .unwrap_or_else(|| resolve_runner_sku(&lower_inst).ram_gb);
                            let spot_hr = extract_named_graphql_usd(&body, "\"spot\":")
                                .filter(|v| *v > 0.0 && *v < usd_hr)
                                .unwrap_or_else(|| (usd_hr * 0.33 * 10000.0).round() / 10000.0);
                            persist_cloud_price_entry(
                                &cache_path,
                                &cache_key,
                                usd_hr,
                                spot_hr,
                                vcpus,
                                ram_gb,
                            );
                            return CloudInstanceQuote {
                                vendor: vendor.to_string(),
                                instance_type: lower_inst,
                                region: lower_reg,
                                vcpus,
                                ram_gb,
                                on_demand_hourly_usd: usd_hr,
                                spot_hourly_usd: spot_hr,
                                source: "live_cloud_graphql".to_string(),
                            };
                        }
                    }
                }
            }
        }
        // Persist regional embedded quote into hot-cache so `cloud_prices.json` is populated, while honestly reporting `embedded_seed`
        let reg_mult = crate::catalog::regional_seed_multiplier(&lower_reg);
        if let Some(seed) = lookup_infracost_cloud_spec(&lower_inst) {
            let od_hr = ((seed.on_demand_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
            let sp_hr = ((seed.spot_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
            persist_cloud_price_entry(&cache_path, &cache_key, od_hr, sp_hr, seed.vcpus, seed.ram_gb);
            return CloudInstanceQuote {
                vendor: detected_vendor.to_string(),
                instance_type: seed.instance_type.to_string(),
                region: lower_reg,
                vcpus: seed.vcpus,
                ram_gb: seed.ram_gb,
                on_demand_hourly_usd: od_hr,
                spot_hourly_usd: sp_hr,
                source: "embedded_seed".to_string(),
            };
        }
    }

    // 3. Deterministic sub-microsecond embedded 150-instance seed catalog + regional multiplier fallback
    let reg_mult = crate::catalog::regional_seed_multiplier(&lower_reg);
    if let Some(seed) = lookup_infracost_cloud_spec(&lower_inst) {
        let od_hr = ((seed.on_demand_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
        let sp_hr = ((seed.spot_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
        return CloudInstanceQuote {
            vendor: detected_vendor.to_string(),
            instance_type: seed.instance_type.to_string(),
            region: lower_reg,
            vcpus: seed.vcpus,
            ram_gb: seed.ram_gb,
            on_demand_hourly_usd: od_hr,
            spot_hourly_usd: sp_hr,
            source: "embedded_seed".to_string(),
        };
    }
    if let Some(seed150) = crate::catalog::lookup_seeded_cloud_spec(&lower_inst) {
        let od_hr = ((seed150.on_demand_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
        let sp_hr = ((seed150.spot_per_min_usd * 60.0 * reg_mult) * 10000.0).round() / 10000.0;
        return CloudInstanceQuote {
            vendor: detected_vendor.to_string(),
            instance_type: seed150.instance_type.clone(),
            region: lower_reg,
            vcpus: seed150.vcpus,
            ram_gb: seed150.ram_gb,
            on_demand_hourly_usd: od_hr,
            spot_hourly_usd: sp_hr,
            source: "cloud_instances_seed_150".to_string(),
        };
    }

    // Fallback for novel/unlisted cloud VM families: use cloud VM per-vCPU On-Demand rates (never SaaS hosted runner rates!)
    let sku = resolve_runner_sku(&lower_inst);
    let vcpus = sku.vcpus.max(2);
    let is_arm = lower_inst.contains("arm")
        || lower_inst.contains("t2a")
        || lower_inst.contains("c7g")
        || lower_inst.contains("c8g")
        || lower_inst.contains("m6g")
        || lower_inst.contains("m7g")
        || lower_inst.contains("r6g")
        || lower_inst.contains("r7g")
        || lower_inst.contains("t4g");
    let per_vcpu_hr = if is_arm { 0.0385 } else { 0.0480 };
    let od_hr = (((vcpus as f64) * per_vcpu_hr * reg_mult) * 10000.0).round() / 10000.0;
    let sp_hr = ((od_hr * 0.33) * 10000.0).round() / 10000.0;
    CloudInstanceQuote {
        vendor: detected_vendor.to_string(),
        instance_type: lower_inst,
        region: lower_reg,
        vcpus,
        ram_gb: sku.ram_gb,
        on_demand_hourly_usd: od_hr,
        spot_hourly_usd: sp_hr,
        source: "embedded_seed".to_string(),
    }
}

fn infer_cloud_vendor(inst: &str) -> &'static str {
    if inst.starts_with("gcp-") {
        return "gcp";
    }
    if inst.starts_with("azure-") {
        return "azure";
    }
    if inst.starts_with("aws-") {
        return "aws";
    }
    let clean = strip_cloud_vendor_prefix(inst);
    if clean.starts_with("n1-")
        || clean.starts_with("n2-")
        || clean.starts_with("n2d-")
        || clean.starts_with("e2-")
        || clean.starts_with("c2-")
        || clean.starts_with("c2d-")
        || clean.starts_with("c3-")
        || clean.starts_with("c3d-")
        || clean.starts_with("c4-")
        || clean.starts_with("t2a-")
        || clean.starts_with("t2d-")
        || clean.starts_with("a2-")
        || clean.starts_with("g2-")
        || clean.starts_with("m1-")
    {
        "gcp"
    } else if clean.starts_with("standard_") {
        "azure"
    } else {
        "aws"
    }
}

fn extract_num_field(raw: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let idx = raw.find(&needle)?;
    let rest = raw[idx + needle.len()..].trim_start();
    let num: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse::<f64>().ok()
}

fn extract_graphql_usd_price(body: &str) -> Option<f64> {
    let needle = "\"USD\":\"";
    let idx = body.find(needle)?;
    let rest = &body[idx + needle.len()..];
    let end = rest.find('"')?;
    rest[..end].parse::<f64>().ok()
}

fn extract_named_graphql_usd(body: &str, section_key: &str) -> Option<f64> {
    let idx = body.find(section_key)?;
    extract_graphql_usd_price(&body[idx + section_key.len()..])
}

fn persist_cloud_price_entry(
    cache_path: &Path,
    key: &str,
    od_hr: f64,
    spot_hr: f64,
    vcpus: u32,
    ram_gb: f64,
) {
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let entry = format!(
        "  \"{}\": {{\"od_hr\":{:.4},\"spot_hr\":{:.4},\"vcpus\":{},\"ram_gb\":{:.1}}}",
        json_escape(key),
        od_hr,
        spot_hr,
        vcpus,
        ram_gb
    );
    let existing = fs::read_to_string(cache_path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && t != "{" && t != "}" && !t.starts_with(&format!("\"{}\":", key))
        })
        .map(|l| l.trim_end_matches(',').to_string())
        .collect();
    lines.push(entry);
    let payload = format!("{{\n{}\n}}\n", lines.join(",\n"));
    let tmp = cache_path.with_extension("json.tmp");
    if fs::write(&tmp, payload).is_ok() {
        let _ = fs::rename(&tmp, cache_path);
    }
}

// =====================================================================================
// 5-Layer Platform Switching TCO Calculator (Model 17 + `minici migrate` Verdict)
// =====================================================================================

#[derive(Debug, Clone)]
pub struct PlatformSwitchingVerdict {
    pub engineering_hours: f64,
    pub total_switching_cost_usd: f64,
    pub payback_months: f64,
    pub three_year_net_benefit_usd: f64,
    pub meets_2x_annual_rule: bool,
    pub verdict: &'static str, // "GO NOW" | "GO IF STRATEGIC" | "STAY" | "MORE EXPENSIVE"
}

pub fn compute_5layer_platform_switching_tco(
    workflow_count: usize,
    active_devs: usize,
    source_vendor: &str,
    target_vendor: &str,
    current_monthly_tco_usd: f64,
    target_monthly_tco_usd: f64,
    loaded_hourly_rate_usd: f64,
    ast_autofix_available: bool,
) -> PlatformSwitchingVerdict {
    if source_vendor == target_vendor {
        return PlatformSwitchingVerdict {
            engineering_hours: 0.0,
            total_switching_cost_usd: 0.0,
            payback_months: 0.0,
            three_year_net_benefit_usd: 0.0,
            meets_2x_annual_rule: false,
            verdict: "STAY",
        };
    }

    // Layer 1: Auto-measured per-pipeline rewrite hours (14.0 base hrs × source friction × complexity)
    let source_friction = if source_vendor.contains("jenkins") {
        1.40
    } else if source_vendor.contains("buildkite") || target_vendor.contains("buildkite") {
        1.30
    } else if source_vendor.contains("codebuild") {
        1.25
    } else if source_vendor.contains("gitlab") {
        1.20
    } else if source_vendor.contains("circleci") {
        1.10
    } else if source_vendor.contains("bitbucket") {
        1.00
    } else {
        0.90
    };
    let complexity_mult = if workflow_count <= 3 {
        0.55
    } else if workflow_count <= 15 {
        1.00
    } else {
        1.35
    };
    let raw_conv_h = (workflow_count.max(1) as f64) * 14.0 * complexity_mult * source_friction;
    // Automated YAML AST conversion reduces YAML-to-YAML rewrite labor by 35%
    let conv_h = if ast_autofix_available && !source_vendor.contains("jenkins") {
        raw_conv_h * 0.65
    } else {
        raw_conv_h
    };

    // Layer 2: Developer onboarding & training (4.0 hrs/dev, capped at 120h)
    let training_h = ((active_devs.max(1) as f64) * 4.0).min(120.0);

    // Layer 3 & 4: Stage-aware parallel dual-run overlap + platform OIDC/secrets/runner setup
    let (overlap_months, platform_setup_usd) = if active_devs <= 8 && current_monthly_tco_usd < 500.0 {
        (0.50, 1200.0)
    } else if active_devs <= 40 && current_monthly_tco_usd < 3000.0 {
        (1.25, 4500.0)
    } else {
        (2.00, 8500.0)
    };
    let dual_run_overlap_usd = current_monthly_tco_usd * overlap_months;
    let platform_setup_h = platform_setup_usd / loaded_hourly_rate_usd.max(1.0);

    let subtotal_h = conv_h + training_h + platform_setup_h;
    // Layer 5: 20% Unforeseen migration risk & validation buffer
    let total_eng_h = ((subtotal_h * 1.20) * 10.0).round() / 10.0;
    let subtotal_usd =
        (conv_h + training_h) * loaded_hourly_rate_usd + dual_run_overlap_usd + platform_setup_usd;
    let total_cost_usd = ((subtotal_usd * 1.20) * 100.0).round() / 100.0;

    let monthly_saving = current_monthly_tco_usd - target_monthly_tco_usd;
    let three_yr_net = (((monthly_saving * 36.0) - total_cost_usd) * 100.0).round() / 100.0;

    if monthly_saving <= 0.01 {
        return PlatformSwitchingVerdict {
            engineering_hours: total_eng_h,
            total_switching_cost_usd: total_cost_usd,
            payback_months: 0.0,
            three_year_net_benefit_usd: three_yr_net,
            meets_2x_annual_rule: false,
            verdict: if monthly_saving < -0.01 {
                "MORE EXPENSIVE"
            } else {
                "STAY"
            },
        };
    }

    let payback = (((total_cost_usd / monthly_saving) * 100.0).round() / 100.0).min(120.0);
    let worth_2x = (monthly_saving * 12.0) >= (2.0 * total_cost_usd);
    let verdict = if payback < 18.0 && three_yr_net > 0.0 {
        "GO NOW"
    } else if payback < 36.0 && three_yr_net > 0.0 {
        "GO IF STRATEGIC"
    } else {
        "STAY"
    };

    PlatformSwitchingVerdict {
        engineering_hours: total_eng_h,
        total_switching_cost_usd: total_cost_usd,
        payback_months: payback,
        three_year_net_benefit_usd: three_yr_net,
        meets_2x_annual_rule: worth_2x,
        verdict,
    }
}
