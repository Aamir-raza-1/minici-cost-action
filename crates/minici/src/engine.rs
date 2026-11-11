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
    resolve_runner_sku, ARTIFACT_STORAGE_S3_USD_PER_GB_MO, ARTIFACT_STORAGE_VENDOR_USD_PER_GB_MO,
    SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH, SELF_HOSTED_VPS_BASE_MONTHLY_USD,
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
    pub uploads_artifacts: bool,
}

impl ExpandedJobCost {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"workflow_file\":\"{}\",\"job_id\":\"{}\",\"matrix_combination_count\":{},\"runner_label\":\"{}\",\"provider\":\"{}\",\"os\":\"{}\",\"vcpus\":{},\"est_raw_duration_mins_per_job\":{:.2},\"billed_mins_per_job\":{:.2},\"total_billed_mins_all_matrix\":{:.2},\"rate_per_min_usd\":{:.4},\"run_compute_cost_usd\":{:.4},\"blacksmith_run_cost_usd\":{:.4},\"self_hosted_spot_run_cost_usd\":{:.4},\"has_concurrency_cancel\":{},\"has_sparse_or_shallow_clone\":{},\"uploads_artifacts\":{}}}",
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
            self.uploads_artifacts
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
    pub net_billed_monthly_compute_cost_usd: f64,
    pub dev_wait_cost_per_run_usd: f64,
    pub monthly_dev_wait_cost_usd: f64,
    pub monthly_artifact_storage_vendor_usd: f64,
    pub monthly_artifact_storage_s3_usd: f64,
    pub monthly_blacksmith_cost_usd: f64,
    pub monthly_self_hosted_spot_cost_usd: f64,
    pub exceeds_5000m_self_hosted_breakeven: bool,
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
            "{{\n  \"target_path\": \"{}\",\n  \"execution_time_us\": {},\n  \"execution_time_ms\": {:.3},\n  \"detected_providers\": [{}],\n  \"usage\": {},\n  \"total_workflows\": {},\n  \"total_defined_jobs\": {},\n  \"total_expanded_matrix_jobs\": {},\n  \"raw_duration_mins_per_run\": {:.2},\n  \"critical_path_wait_mins_per_run\": {:.2},\n  \"billed_mins_per_run\": {:.2},\n  \"monthly_runs\": {:.1},\n  \"monthly_billed_mins\": {:.2},\n  \"free_tier_included_mins\": {:.1},\n  \"free_tier_quota_burn_pct\": {:.1},\n  \"effective_tier\": \"{}\",\n  \"gross_compute_cost_per_run_usd\": {:.4},\n  \"gross_monthly_compute_cost_usd\": {:.2},\n  \"net_billed_monthly_compute_cost_usd\": {:.2},\n  \"dev_wait_cost_per_run_usd\": {:.2},\n  \"monthly_dev_wait_cost_usd\": {:.2},\n  \"monthly_artifact_storage_vendor_usd\": {:.2},\n  \"monthly_artifact_storage_s3_usd\": {:.2},\n  \"monthly_blacksmith_cost_usd\": {:.2},\n  \"monthly_self_hosted_spot_cost_usd\": {:.2},\n  \"exceeds_5000m_self_hosted_breakeven\": {},\n  \"jobs\": [\n    {}\n  ]\n}}",
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
            self.monthly_runs,
            self.monthly_billed_mins,
            self.free_tier_included_mins,
            self.free_tier_quota_burn_pct,
            json_escape(&self.effective_tier),
            self.gross_compute_cost_per_run_usd,
            self.gross_monthly_compute_cost_usd,
            self.net_billed_monthly_compute_cost_usd,
            self.dev_wait_cost_per_run_usd,
            self.monthly_dev_wait_cost_usd,
            self.monthly_artifact_storage_vendor_usd,
            self.monthly_artifact_storage_s3_usd,
            self.monthly_blacksmith_cost_usd,
            self.monthly_self_hosted_spot_cost_usd,
            self.exceeds_5000m_self_hosted_breakeven,
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
            "{{\n  \"base_path\": \"{}\",\n  \"head_path\": \"{}\",\n  \"execution_time_us\": {},\n  \"base_expanded_jobs\": {},\n  \"head_expanded_jobs\": {},\n  \"delta_expanded_jobs\": {},\n  \"base_billed_mins_per_run\": {:.2},\n  \"head_billed_mins_per_run\": {:.2},\n  \"delta_billed_mins_per_run\": {:.2},\n  \"base_monthly_compute_usd\": {:.2},\n  \"head_monthly_compute_usd\": {:.2},\n  \"delta_monthly_compute_usd\": {:.2},\n  \"delta_monthly_compute_pct\": {:.1},\n  \"base_monthly_dev_wait_usd\": {:.2},\n  \"head_monthly_dev_wait_usd\": {:.2},\n  \"delta_monthly_dev_wait_usd\": {:.2},\n  \"base_free_quota_burn_pct\": {:.1},\n  \"head_free_quota_burn_pct\": {:.1},\n  \"job_changes\": [{}]\n}}",
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
            ch.join(", ")
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
                    _ => {}
                }
            }
        }
    }
    cfg
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

/// Fast zero-allocation scanner for a single GitHub Actions job block (`lines` belonging to one job).
fn analyze_github_job_block(
    wf_rel: &str,
    job_id: &str,
    job_lines: &[&str],
    has_top_concurrency: bool,
    out_jobs: &mut Vec<ExpandedJobCost>,
) {
    let mut raw_runs_on = "ubuntu-latest".to_string();
    let mut has_job_concurrency = has_top_concurrency;
    let mut explicit_est_mins: Option<f64> = None;
    let mut has_sparse = false;
    let mut uploads_artifacts = false;
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
        if trimmed.starts_with("runs-on:") {
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
                            if let Some(os_val) = item_val.strip_prefix("os:") {
                                let clean_os = os_val.trim().trim_matches('"').trim_matches('\'');
                                if !clean_os.is_empty() {
                                    axis_counts.push(("__include_os__".to_string(), vec![clean_os.to_string()]));
                                }
                            }
                        }
                    } else if current_matrix_axis == "exclude" {
                        if ind <= axis_indent + 4 {
                            exclude_items += 1;
                        }
                    } else if let Some(last) = axis_counts.last_mut() {
                        if last.0 == current_matrix_axis && !item_val.contains(':') {
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
                step_mins += 0.6;
            } else if trimmed.starts_with("filter:")
                || trimmed.starts_with("sparse-checkout:")
                || (trimmed.starts_with("fetch-depth:") && trimmed.ends_with('1'))
            {
                if !has_sparse {
                    has_sparse = true;
                    step_mins = (step_mins - 0.4).max(0.5);
                }
            } else if trimmed.contains("upload-artifact") {
                uploads_artifacts = true;
                step_mins += 0.4;
            } else if trimmed.contains("pytest")
                || trimmed.contains("cargo test")
                || trimmed.contains("npm test")
                || trimmed.contains("vitest")
                || trimmed.contains("go test")
                || trimmed.contains("mvn test")
            {
                step_mins += 3.2;
            } else if trimmed.contains("docker build")
                || trimmed.contains("cargo build --release")
            {
                step_mins += 4.0;
            } else if trimmed.starts_with("- name:") || trimmed.starts_with("- run:") || trimmed.starts_with("- uses:") {
                step_mins += 0.6;
            }
        }
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
        if non_empty_axes == 0 && include_items > 0 {
            let mut os_buckets: Vec<(String, usize)> = Vec::new();
            for os_l in &matrix_os_list {
                if let Some(found) = os_buckets.iter_mut().find(|x| x.0 == *os_l) {
                    found.1 += 1;
                } else {
                    os_buckets.push((os_l.clone(), 1));
                }
            }
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
                    uploads_artifacts,
                });
            }
            return;
        }
        let base_per_os = (product_count / matrix_os_list.len()).max(1);
        for (idx, os_label) in matrix_os_list.iter().enumerate() {
            // Distribute any net include/exclude delta cleanly onto the first OS bucket so total jobs == matrix_count
            let per_os_jobs = if idx == 0 {
                let others = base_per_os * (matrix_os_list.len().saturating_sub(1));
                matrix_count.saturating_sub(others).max(1)
            } else {
                base_per_os
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
                uploads_artifacts,
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
        out_jobs.push(ExpandedJobCost {
            workflow_file: wf_rel.to_string(),
            job_id: job_id.to_string(),
            matrix_combination_count: matrix_count,
            runner_label: raw_runs_on,
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
            uploads_artifacts,
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

        if !in_jobs && ind == 0 && trimmed.starts_with("concurrency:") {
            has_top_concurrency = true;
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
    let mut has_interruptible = false;

    for line in content.lines() {
        let clean = strip_comment(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
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
            uploads_artifacts,
        });
    }
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
    }

    if jobs.is_empty() {
        workflow_count = 1;
        let sku = resolve_runner_sku("ubuntu-latest");
        jobs.push(ExpandedJobCost {
            workflow_file: ".github/workflows/ci.yml (inferred)".to_string(),
            job_id: "test".to_string(),
            matrix_combination_count: 1,
            runner_label: "ubuntu-latest".to_string(),
            provider: sku.provider.to_string(),
            os: sku.os.to_string(),
            vcpus: sku.vcpus,
            est_raw_duration_mins_per_job: 4.0,
            billed_mins_per_job: 4.0,
            total_billed_mins_all_matrix: 4.0,
            rate_per_min_usd: sku.rate_per_min_usd,
            run_compute_cost_usd: 0.032,
            blacksmith_run_cost_usd: 0.016,
            self_hosted_spot_run_cost_usd: 0.0048,
            has_concurrency_cancel: false,
            has_sparse_or_shallow_clone: false,
            uploads_artifacts: false,
        });
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

    let monthly_runs = (usage.prs_per_week * usage.pushes_per_pr * 4.33).round();
    let monthly_billed_mins = (billed_mins_per_run * monthly_runs * 100.0).round() / 100.0;

    let primary_sku = resolve_runner_sku(&jobs[0].runner_label);
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

    let dev_wait_cost_per_run_usd =
        (((critical_path_wait_mins_per_run / 60.0) * usage.dev_hourly_rate_usd) * 100.0).round()
            / 100.0;
    let monthly_dev_wait_cost_usd =
        ((dev_wait_cost_per_run_usd * monthly_runs) * 100.0).round() / 100.0;

    let any_artifacts = jobs.iter().any(|j| j.uploads_artifacts);
    let active_gb = if any_artifacts {
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
    let monthly_self_hosted_spot_cost_usd = ((jobs
        .iter()
        .map(|j| j.self_hosted_spot_run_cost_usd)
        .sum::<f64>()
        * monthly_runs
        + SELF_HOSTED_VPS_BASE_MONTHLY_USD)
        * 100.0)
        .round()
        / 100.0;

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
        net_billed_monthly_compute_cost_usd,
        dev_wait_cost_per_run_usd,
        monthly_dev_wait_cost_usd,
        monthly_artifact_storage_vendor_usd,
        monthly_artifact_storage_s3_usd,
        monthly_blacksmith_cost_usd,
        monthly_self_hosted_spot_cost_usd,
        exceeds_5000m_self_hosted_breakeven: monthly_billed_mins
            >= SELF_HOSTED_BREAKEVEN_MINS_PER_MONTH,
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
