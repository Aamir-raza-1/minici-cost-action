//! Infracost-Style Markdown, Table, Slack Block Kit & PostToolUse 6-Channel Launch Pack Renderer
//! (`crates/minici/src/renderer.rs` — Zero External Dependencies).

use crate::engine::{json_escape, BreakdownReport, DiffReport};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn detect_github_account_and_git_user() -> String {
    let mut gh_users = Vec::new();
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let hosts_path = Path::new(&home).join(".config").join("gh").join("hosts.yml");
        if let Ok(raw) = fs::read_to_string(hosts_path) {
            for line in raw.lines() {
                let t = line.trim();
                if let Some(u) = t.strip_prefix("user:") {
                    let u = u.trim().trim_matches('"').trim_matches('\'');
                    if !u.is_empty() && !gh_users.contains(&u.to_string()) {
                        gh_users.push(u.to_string());
                    }
                }
            }
        }
    }
    let git_name = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let git_email = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let gh_part = if !gh_users.is_empty() {
        format!("github.com/{}", gh_users.join(", github.com/"))
    } else {
        "github.com (local git)".to_string()
    };

    if !git_name.is_empty() || !git_email.is_empty() {
        format!("{} ({} <{}>)", gh_part, git_name, git_email)
    } else {
        gh_part
    }
}

fn detect_repo_identity(target_path: &str) -> String {
    let p = Path::new(target_path);
    let check_dir = if p.exists() && p.is_dir() {
        p
    } else {
        Path::new(".")
    };
    let remote = Command::new("git")
        .arg("-C")
        .arg(check_dir)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| {
            s.trim()
                .trim_start_matches("https://github.com/")
                .trim_start_matches("git@github.com:")
                .trim_end_matches(".git")
                .to_string()
        })
        .unwrap_or_default();

    let branch = Command::new("git")
        .arg("-C")
        .arg(check_dir)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if target_path.contains("[PUBLIC & ACCESSIBLE") || target_path.starts_with("github.com/") {
        target_path.to_string()
    } else if !remote.is_empty() && (target_path == "." || target_path.contains("ci-casestudies")) {
        if !branch.is_empty() {
            format!("github.com/{} (branch: {} • path: {})", remote, branch, target_path)
        } else {
            format!("github.com/{} (path: {})", remote, target_path)
        }
    } else if !remote.is_empty() && p.join(".git").exists() {
        format!("github.com/{} (branch: {} • path: {})", remote, branch, target_path)
    } else {
        target_path.to_string()
    }
}

fn detect_binary_size_kb() -> u64 {
    env::current_exe()
        .ok()
        .and_then(|p| fs::metadata(p).ok())
        .map(|m| (m.len() + 512) / 1024)
        .unwrap_or(492)
}

pub fn render_cli_compact_summary(r: &BreakdownReport) -> String {
    if r.total_workflows == 0 {
        return "✓ minici: Skipped (0 CI workflow files found)".to_string();
    }

    let repo_info = detect_repo_identity(&r.target_path);
    let mode_env = std::env::var("MINICI_CUSTOMER_MODE").unwrap_or_default();
    let is_owner_mode = mode_env == "0" || mode_env.eq_ignore_ascii_case("false");
    let is_explicit_customer_mode = mode_env == "1" || mode_env.eq_ignore_ascii_case("true");
    let customer_mode = !is_owner_mode;

    let view_suffix = if is_owner_mode {
        " [OWNER / INTERNAL DEBUG VIEW]"
    } else if is_explicit_customer_mode {
        " [CUSTOMER LAUNCH VIEW]"
    } else {
        ""
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Project: {} ({} workflow(s), {} expanded jobs • {:.2} ms){}\n\n",
        repo_info,
        r.total_workflows,
        r.total_expanded_matrix_jobs,
        r.execution_time_ms,
        view_suffix
    ));

    out.push_str(&format!(
        " {:<42}  {:<24}  {:>11}  {:>10}  {:>10}  {:>13}\n",
        "WORKFLOW / JOB", "MATRIX × RUNNER", "BILLED MINS", "RATE/MIN", "COST/RUN", "MONTHLY COST"
    ));
    out.push_str(&format!(
        " {:<42}  {:<24}  {:>11}  {:>10}  {:>10}  {:>13}\n",
        "──────────────────────────────────────────",
        "────────────────────────",
        "───────────",
        "──────────",
        "──────────",
        "─────────────"
    ));

    for j in &r.jobs {
        let wf_short = j
            .workflow_file
            .trim_start_matches(".github/workflows/");
        let wf_job_raw = format!("{} • {}", wf_short, j.job_id);
        let wf_job = if wf_job_raw.chars().count() > 42 {
            let truncated: String = wf_job_raw.chars().take(39).collect();
            format!("{}...", truncated)
        } else {
            wf_job_raw
        };

        let mat_runner_raw = format!("{}x {}", j.matrix_combination_count, j.runner_label);
        let mat_runner = if mat_runner_raw.chars().count() > 24 {
            let truncated: String = mat_runner_raw.chars().take(21).collect();
            format!("{}...", truncated)
        } else {
            mat_runner_raw
        };

        let monthly_job_usd = j.run_compute_cost_usd * r.monthly_runs;
        let mins_col = format!("{:.1}m", j.total_billed_mins_all_matrix);
        let rate_col = format!("${:.4}/m", j.rate_per_min_usd);
        let run_col = format!("${:.3}", j.run_compute_cost_usd);
        let mo_col = format!("${:.2}/mo", monthly_job_usd);
        out.push_str(&format!(
            " {:<42}  {:<24}  {:>11}  {:>10}  {:>10}  {:>13}\n",
            wf_job,
            mat_runner,
            mins_col,
            rate_col,
            run_col,
            mo_col
        ));
    }

    out.push_str(&format!(
        " {:<42}  {:<24}  {:>11}  {:>10}  {:>10}  {:>13}\n",
        "──────────────────────────────────────────",
        "────────────────────────",
        "───────────",
        "──────────",
        "──────────",
        "─────────────"
    ));
    let tier_short = if r.effective_tier.starts_with("free") {
        "free tier"
    } else {
        "paid tier"
    };
    let total_label = format!("OVERALL TOTAL ({:.0} runs/mo • {})", r.monthly_runs, tier_short);
    let total_label_trunc = if total_label.chars().count() > 42 {
        let t: String = total_label.chars().take(39).collect();
        format!("{}...", t)
    } else {
        total_label
    };
    let jobs_summary = format!("{} expanded job(s)", r.total_expanded_matrix_jobs);
    let tot_mins_col = format!("{:.1}m", r.billed_mins_per_run);
    let tot_run_col = format!("${:.3}", r.gross_compute_cost_per_run_usd);
    let tot_mo_col = format!("${:.2}/mo", r.gross_monthly_compute_cost_usd);
    out.push_str(&format!(
        " {:<42}  {:<24}  {:>11}  {:>10}  {:>10}  {:>13}\n\n",
        total_label_trunc,
        jobs_summary,
        tot_mins_col,
        "—",
        tot_run_col,
        tot_mo_col
    ));

    let cc = &r.cost_components;
    let rounding_suffix = if cc.rounding_tax_mins_per_run > 0.0 {
        format!(
            " • Rounding Overhead: {:.1}m/run (${:.2}/mo)",
            cc.rounding_tax_mins_per_run, cc.monthly_rounding_tax_usd
        )
    } else {
        String::new()
    };
    out.push_str(&format!(
        " • Free-Tier Quota  : {:.0} / {:.0} mins ({:.1}% of monthly quota) • Net Billed: ${:.2}/mo{}\n",
        r.monthly_billed_mins,
        r.free_tier_included_mins,
        r.free_tier_quota_burn_pct,
        r.net_billed_monthly_compute_cost_usd,
        rounding_suffix
    ));

    let concurrency_suffix = if cc.parallelism_tier_upgrade_tax_monthly_usd > 0.0 {
        format!(
            " • Concurrency: {} peak jobs ({} hosted > {} cap, +${:.2}/mo tier upgrade)",
            cc.peak_concurrent_jobs_per_run,
            cc.hosted_concurrent_jobs_per_run,
            cc.vendor_free_concurrent_cap,
            cc.parallelism_tier_upgrade_tax_monthly_usd
        )
    } else if cc.self_hosted_or_microvm_jobs_per_run > 0 {
        format!(
            " • Concurrency: {} peak jobs ({} hosted / {} custom)",
            cc.peak_concurrent_jobs_per_run,
            cc.hosted_concurrent_jobs_per_run,
            cc.self_hosted_or_microvm_jobs_per_run
        )
    } else {
        String::new()
    };
    out.push_str(&format!(
        " • Dev Wait-Time    : {:.1}m critical path (${:.2}/run • ${:.2}/mo gross | ${:.2}/mo at 40% context-switch loss){}\n",
        r.critical_path_wait_mins_per_run,
        r.dev_wait_cost_per_run_usd,
        r.monthly_dev_wait_cost_usd,
        cc.monthly_productive_payroll_loss_usd,
        concurrency_suffix
    ));

    if cc.jobs_uploading_artifacts > 0 {
        let retention_hint = if cc.configured_retention_days > 14 {
            format!(
                " (14d retention reduces storage to ${:.2}/mo)",
                cc.monthly_vendor_storage_14d_usd
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            " • Artifact Storage : {} upload job(s) • {}d retention (${:.2}/mo, {:.2} GB steady-state){}\n",
            cc.jobs_uploading_artifacts,
            cc.configured_retention_days,
            cc.monthly_vendor_storage_configured_usd,
            cc.steady_state_stored_gb_configured,
            retention_hint
        ));
    }

    let mut hygiene_parts = Vec::new();
    if cc.jobs_missing_cache > 0 && cc.monthly_unrealized_cache_savings_usd >= 1.0 {
        hygiene_parts.push(format!(
            "Cache {}/{} jobs (caching {} uncached saves ~${:.2}/mo)",
            cc.jobs_with_cache,
            cc.peak_concurrent_jobs_per_run,
            cc.jobs_missing_cache,
            cc.monthly_unrealized_cache_savings_usd
        ));
    } else if cc.jobs_with_cache > 0 {
        hygiene_parts.push(format!(
            "Cache {}/{} jobs ({:.0}% est hit rate)",
            cc.jobs_with_cache, cc.peak_concurrent_jobs_per_run, cc.estimated_cache_hit_rate_pct
        ));
    }
    if cc.workflows_missing_concurrency_cancel > 0 && cc.monthly_superseded_run_waste_usd >= 1.0 {
        hygiene_parts.push(format!(
            "Cancel-in-progress missing on {} wf (~${:.2}/mo superseded waste)",
            cc.workflows_missing_concurrency_cancel, cc.monthly_superseded_run_waste_usd
        ));
    }
    if cc.workflows_missing_path_filters > 0 && cc.monthly_unfiltered_check_savings_usd >= 1.0 {
        hygiene_parts.push(format!(
            "Path filters missing on {} wf (~${:.2}/mo selective savings)",
            cc.workflows_missing_path_filters, cc.monthly_unfiltered_check_savings_usd
        ));
    }
    if !hygiene_parts.is_empty() {
        out.push_str(&format!(
            " • Pipeline Hygiene : {}\n",
            hygiene_parts.join(" • ")
        ));
    }

    let uses_self_hosted_cloud_vpc = r.jobs.iter().any(|j| {
        j.provider.starts_with("self-hosted")
            || j.provider == "aws-codebuild"
            || j.provider == "gcp-cloud-build"
    });
    if uses_self_hosted_cloud_vpc && cc.monthly_transfer_gb > 0.0 {
        out.push_str(&format!(
            " • Cloud Network    : {:.2} GB/mo ({}) • Private Subnet + NAT: ${:.2}/mo (VPC Endpoint + Cache: ${:.2}/mo)\n",
            cc.monthly_transfer_gb,
            cc.infracost_cloud_region,
            cc.monthly_infracost_private_subnet_nat_worst_case_usd,
            cc.monthly_infracost_vpc_endpoint_cache_optimized_usd
        ));
    }

    out.push_str(&format!(
        " • Recommendation   : {}",
        build_contextual_recommendation(r)
    ));

    if !customer_mode {
        let catalog_state = crate::catalog::pricing_catalog_status();
        let account_info = detect_github_account_and_git_user();
        let bin_kb = detect_binary_size_kb();
        out.push_str(&format!(
            "\n\n [OWNER / INTERNAL DEBUG VIEW • {} • {} KB Rust Binary]\n\
               [ON] feat.matrix_ast_expansion  : {} defined jobs -> {} expanded matrix combinations\n\
               [ON] feat.hot_reload_catalog    : {}\n\
               [MATH TRACE] Runs/Mo = {:.1} PRs/wk * 4.333 wks * {:.1} pushes/PR = {:.1} runs/mo\n\
               [MATH TRACE] Gross = ${:.3}/run * {:.1} runs = ${:.2}/mo | Net = ${:.2}/mo",
            account_info,
            bin_kb,
            r.total_defined_jobs,
            r.total_expanded_matrix_jobs,
            catalog_state,
            r.usage.prs_per_week,
            r.usage.pushes_per_pr,
            r.monthly_runs,
            r.gross_compute_cost_per_run_usd,
            r.monthly_runs,
            r.gross_monthly_compute_cost_usd,
            r.net_billed_monthly_compute_cost_usd
        ));
    }

    out
}

pub fn render_breakdown_csv(r: &BreakdownReport) -> String {
    let mut out = String::from("workflow_file,job_id,matrix_count,runner_label,provider,os,vcpus,raw_mins,billed_mins_total,rate_per_min_usd,cost_per_run_usd\n");
    for j in &r.jobs {
        out.push_str(&format!(
            "\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\",{},{:.2},{:.2},{:.4},{:.4}\n",
            j.workflow_file,
            j.job_id,
            j.matrix_combination_count,
            j.runner_label,
            j.provider,
            j.os,
            j.vcpus,
            j.est_raw_duration_mins_per_job,
            j.total_billed_mins_all_matrix,
            j.rate_per_min_usd,
            j.run_compute_cost_usd
        ));
    }
    out
}

fn build_contextual_recommendation(r: &BreakdownReport) -> String {
    let cc = &r.cost_components;
    if cc.self_hosted_or_microvm_jobs_per_run > 0
        && cc.self_hosted_or_microvm_jobs_per_run >= cc.hosted_concurrent_jobs_per_run
    {
        if cc.jobs_uploading_artifacts > 0 && cc.configured_retention_days > 14 {
            return format!(
                "Pipeline compute already optimized ({}/{} jobs on custom runners); set artifact `retention-days: 14` to cut storage from ${:.2}/mo to ${:.2}/mo",
                cc.self_hosted_or_microvm_jobs_per_run,
                cc.peak_concurrent_jobs_per_run,
                cc.monthly_vendor_storage_configured_usd,
                cc.monthly_vendor_storage_14d_usd
            );
        }
        return format!(
            "Pipeline already optimized ({}/{} jobs on custom runners; remaining {} lightweight hosted job(s) are optimal as-is)",
            cc.self_hosted_or_microvm_jobs_per_run,
            cc.peak_concurrent_jobs_per_run,
            cc.hosted_concurrent_jobs_per_run
        );
    }

    if r.exceeds_5000m_self_hosted_breakeven {
        let blacksmith_savings_usd =
            (r.gross_monthly_compute_cost_usd - r.monthly_blacksmith_cost_usd).max(0.0);
        let spot_savings_usd =
            (r.gross_monthly_compute_cost_usd - r.monthly_self_hosted_spot_cost_usd).max(0.0);
        if blacksmith_savings_usd >= 20.0 {
            format!(
                "Right-size heavy jobs to ARM64 / High-Speed Runners (${:.2}/mo, saves ${:.2}/mo) or Cloud Spot (${:.2}/mo, saves ${:.2}/mo)",
                r.monthly_blacksmith_cost_usd,
                blacksmith_savings_usd,
                r.monthly_self_hosted_spot_cost_usd,
                spot_savings_usd
            )
        } else {
            format!(
                "Move sustained compute ({:.0} mins/mo) to Cloud Spot (${:.2}/mo, saves ${:.2}/mo)",
                r.monthly_billed_mins,
                r.monthly_self_hosted_spot_cost_usd,
                spot_savings_usd
            )
        }
    } else if cc.jobs_uploading_artifacts > 0 && cc.configured_retention_days > 14 {
        format!(
            "Stay on hosted runners ({:.0} / 5,000 mins/mo break-even); set artifact `retention-days: 14` to reduce storage to ${:.2}/mo",
            r.monthly_billed_mins,
            cc.monthly_vendor_storage_14d_usd
        )
    } else {
        format!(
            "Stay on hosted runners ({:.0} / 5,000 mins/mo self-hosted break-even threshold)",
            r.monthly_billed_mins
        )
    }
}

pub fn render_breakdown_html(r: &BreakdownReport) -> String {
    let mut rows = String::new();
    for j in &r.jobs {
        rows.push_str(&format!(
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}x (<code>{}</code>)</td><td>{:.1}m ({:.1}m billed)</td><td>${:.4}/m</td><td><strong>${:.3}/run</strong></td></tr>\n",
            j.workflow_file,
            j.job_id,
            j.matrix_combination_count,
            j.runner_label,
            j.est_raw_duration_mins_per_job,
            j.total_billed_mins_all_matrix,
            j.rate_per_min_usd,
            j.run_compute_cost_usd
        ));
    }
    let rec = build_contextual_recommendation(r);
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><title>MiniCI FinOps Report — {}</title>\
         <style>body{{background:#080b10;color:#f1f5f9;font-family:-apple-system,BlinkMacSystemFont,sans-serif;padding:32px;max-width:1100px;margin:0 auto}}\
         .card{{background:#0f151f;border:1px solid #1e293b;border-radius:10px;padding:20px;margin-bottom:20px}}\
         table{{width:100%;border-collapse:collapse;font-size:14px}}th,td{{padding:10px;border-bottom:1px solid #1e293b;text-align:left}}\
         th{{color:#94a3b8}}code{{color:#38bdf8;font-family:monospace}}.kpi{{display:grid;grid-template-columns:repeat(4,1fr);gap:14px;margin-bottom:20px}}\
         .kpi div{{background:#151e2c;padding:14px;border-radius:8px;border:1px solid #1e293b}}.kpi h3{{margin:0 0 6px;font-size:12px;color:#94a3b8}}\
         .kpi p{{margin:0;font-size:20px;font-weight:700;color:#10b981}}</style></head><body>\
         <h1>MiniCI — CI/CD Cost &amp; Matrix FinOps Report</h1>\
         <p>Target: <code>{}</code> • Scanned in <code>{:.2} ms</code> • {} Workflows ({} Expanded Matrix Jobs)</p>\
         <div class=\"kpi\">\
           <div><h3>Monthly CI Bill</h3><p>${:.2}/mo (${:.3}/run)</p></div>\
           <div><h3>Free Quota Burn</h3><p>{:.0} / {:.0}m ({:.1}%)</p></div>\
           <div><h3>Dev Wait (Assumed ${:.0}/hr)</h3><p>${:.2}/mo ({:.1}m wait)</p></div>\
           <div><h3>Peak Concurrency</h3><p>{} jobs ({} hosted)</p></div>\
         </div>\
         <div class=\"card\"><p style=\"margin:0\"><strong>Recommendation:</strong> {}</p></div>\
         <div class=\"card\"><h2>Per-Job Matrix Breakdown</h2>\
         <table><thead><tr><th>Workflow</th><th>Job ID</th><th>Matrix × SKU</th><th>Duration (Billed)</th><th>Official Rate</th><th>Cost / Run</th></tr></thead>\
         <tbody>{}</tbody></table></div></body></html>\n",
        r.target_path,
        r.target_path,
        r.execution_time_ms,
        r.total_workflows,
        r.total_expanded_matrix_jobs,
        r.gross_monthly_compute_cost_usd,
        r.gross_compute_cost_per_run_usd,
        r.monthly_billed_mins,
        r.free_tier_included_mins,
        r.free_tier_quota_burn_pct,
        r.usage.dev_hourly_rate_usd,
        r.monthly_dev_wait_cost_usd,
        r.critical_path_wait_mins_per_run,
        r.cost_components.peak_concurrent_jobs_per_run,
        r.cost_components.hosted_concurrent_jobs_per_run,
        rec,
        rows
    )
}

pub fn render_cli_compact_diff(d: &DiffReport) -> String {
    let sign = if d.delta_monthly_compute_usd >= 0.0 { "+" } else { "-" };
    let pct_sign = if d.delta_monthly_compute_pct >= 0.0 { "+" } else { "" };
    let wait_sign = if d.delta_monthly_dev_wait_usd >= 0.0 { "+" } else { "-" };
    let top_change = d
        .job_changes
        .first()
        .cloned()
        .unwrap_or_else(|| "No workflow or matrix SKU changes between base and head.".to_string());

    format!(
        "+====================================================================================+\n\
         | MINICI — PULL REQUEST COST DIFF ({} µs • Exit Code 0 Informational)                |\n\
         +====================================================================================+\n\
           Comparing            : `{}` (Base) -> `{}` (PR Head)\n\
           1. Matrix Jobs       : {} jobs -> {} jobs ({:+} jobs)\n\
           2. Billed Mins / Run : {:.1}m -> {:.1}m ({:+.1}m / run)\n\
           3. Monthly CI Cost   : ${:.2}/mo -> ${:.2}/mo ({}${:.2}/mo • {}{:.1}%)\n\
           4. Dev Wait Cost ($) : ${:.2}/mo -> ${:.2}/mo ({}${:.2}/mo) [Assumed Rate Configurable]\n\
           5. Key Change        : {} (total changes: {})\n\
         +====================================================================================+\n",
        d.execution_time_us,
        d.base_path,
        d.head_path,
        d.base_expanded_jobs,
        d.head_expanded_jobs,
        d.delta_expanded_jobs,
        d.base_billed_mins_per_run,
        d.head_billed_mins_per_run,
        d.delta_billed_mins_per_run,
        d.base_monthly_compute_usd,
        d.head_monthly_compute_usd,
        sign,
        d.delta_monthly_compute_usd.abs(),
        pct_sign,
        d.delta_monthly_compute_pct,
        d.base_monthly_dev_wait_usd,
        d.head_monthly_dev_wait_usd,
        wait_sign,
        d.delta_monthly_dev_wait_usd.abs(),
        top_change,
        d.job_changes.len()
    )
}

fn render_applicable_summary_notes_markdown(r: &BreakdownReport) -> String {
    let cc = &r.cost_components;
    let mut notes = Vec::new();
    if cc.rounding_tax_mins_per_run > 0.0 {
        notes.push(format!(
            "- **1-Minute Ceiling Rounding:** `{:.1}m/run` (`${:.2}/mo`)",
            cc.rounding_tax_mins_per_run, cc.monthly_rounding_tax_usd
        ));
    }
    if cc.jobs_uploading_artifacts > 0 {
        notes.push(format!(
            "- **Artifact Storage:** `{}` upload job(s) at `{}d` retention (`${:.2}/mo`, `{:.2} GB` steady-state)",
            cc.jobs_uploading_artifacts,
            cc.configured_retention_days,
            cc.monthly_vendor_storage_configured_usd,
            cc.steady_state_stored_gb_configured
        ));
    }
    if cc.parallelism_tier_upgrade_tax_monthly_usd > 0.0 {
        notes.push(format!(
            "- **Concurrency Cap:** `{}` hosted concurrent jobs exceed `{}` tier cap (`+${:.2}/mo` tier upgrade)",
            cc.hosted_concurrent_jobs_per_run,
            cc.vendor_free_concurrent_cap,
            cc.parallelism_tier_upgrade_tax_monthly_usd
        ));
    }
    if cc.jobs_missing_cache > 0 && cc.monthly_unrealized_cache_savings_usd >= 1.0 {
        notes.push(format!(
            "- **Build Cache Opportunity:** `{}/{}` jobs cached; caching remaining `{}` job(s) saves **`~${:.2}/mo`** (`{:.0}-{:.0}%` typical for `{}`)",
            cc.jobs_with_cache,
            cc.peak_concurrent_jobs_per_run,
            cc.jobs_missing_cache,
            cc.monthly_unrealized_cache_savings_usd,
            cc.realistic_saving_pct_min,
            cc.realistic_saving_pct_max,
            cc.stack_cache_profile
        ));
    }
    if cc.workflows_missing_concurrency_cancel > 0 && cc.monthly_superseded_run_waste_usd >= 1.0 {
        notes.push(format!(
            "- **Superseded Push Runs:** `{}` workflow(s) lack `cancel-in-progress: true` (`~${:.2}/mo` wasted on obsolete pushes)",
            cc.workflows_missing_concurrency_cancel,
            cc.monthly_superseded_run_waste_usd
        ));
    }
    if cc.workflows_missing_path_filters > 0 && cc.monthly_unfiltered_check_savings_usd >= 1.0 {
        notes.push(format!(
            "- **Selective PR Checks:** `{}` workflow(s) run on all pushes without `paths:` filters (`~${:.2}/mo` savings with path filtering)",
            cc.workflows_missing_path_filters,
            cc.monthly_unfiltered_check_savings_usd
        ));
    }
    notes.push(format!("- **Recommendation:** {}", build_contextual_recommendation(r)));
    format!("\n{}\n", notes.join("\n"))
}

pub fn render_breakdown_markdown(r: &BreakdownReport) -> String {
    if r.total_workflows == 0 {
        return "<!-- minici-cost-comment -->\n✓ **MiniCI:** Skipped (0 CI workflow files found)\n".to_string();
    }

    let mut rows = String::new();
    for j in &r.jobs {
        let monthly_job_usd = j.run_compute_cost_usd * r.monthly_runs;
        rows.push_str(&format!(
            "| `{}` | `{}` | `{}x {}` | `{:.1}m` | `${:.4}/m` | `${:.3}` | **`${:.2}/mo`** |\n",
            j.workflow_file.trim_start_matches(".github/workflows/"),
            j.job_id,
            j.matrix_combination_count,
            j.runner_label,
            j.total_billed_mins_all_matrix,
            j.rate_per_min_usd,
            j.run_compute_cost_usd,
            monthly_job_usd
        ));
    }

    let notes_md = render_applicable_summary_notes_markdown(r);

    format!(
        "<!-- minici-cost-comment -->\n\
### 💰 MiniCI CI/CD Cost & Quota Breakdown (`{:.2} ms`)\n\n\
**Summary:** `{}` workflow(s) • `{}` expanded matrix job(s) • **`${:.2}/mo`** gross compute (**`${:.2}/mo`** net billed, `{:.1}%` of `{:.0}m` free quota) • **Dev Wait:** `${:.2}/mo` (`{:.1}m` critical path)\n\n\
| Workflow | Job ID | Matrix × Runner | Billed Mins | Rate/Min | Cost/Run | Monthly (`{:.0}` runs) |\n\
| :--- | :--- | :--- | ---: | ---: | ---: | ---: |\n\
{rows}| **OVERALL TOTAL** | **`{} job(s)`** | **`{}`** | **`{:.1}m`** | — | **`${:.3}`** | **`${:.2}/mo`** |\n\
{notes_md}",
        r.execution_time_ms,
        r.total_workflows,
        r.total_expanded_matrix_jobs,
        r.gross_monthly_compute_cost_usd,
        r.net_billed_monthly_compute_cost_usd,
        r.free_tier_quota_burn_pct,
        r.free_tier_included_mins,
        r.monthly_dev_wait_cost_usd,
        r.critical_path_wait_mins_per_run,
        r.monthly_runs,
        r.total_expanded_matrix_jobs,
        r.effective_tier,
        r.billed_mins_per_run,
        r.gross_compute_cost_per_run_usd,
        r.gross_monthly_compute_cost_usd
    )
}

pub fn render_diff_markdown(d: &DiffReport) -> String {
    let sign = if d.delta_monthly_compute_usd >= 0.0 { "+" } else { "" };
    let wait_sign = if d.delta_monthly_dev_wait_usd >= 0.0 { "+" } else { "" };
    let change_lines = if d.job_changes.is_empty() {
        "- *No workflow or matrix SKU changes detected between base and head.*".to_string()
    } else {
        d.job_changes
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let six_comp_md = render_applicable_summary_notes_markdown(&d.head_report);

    format!(
        "<!-- minici-cost-comment -->\n\
### 💰 MiniCI Pull Request CI/CD Cost Diff (`{} µs` Rust Engine)\n\n\
| Metric | Base Branch | PR Branch (Head) | Delta (`Before vs. After`) |\n\
| :--- | :--- | :--- | :--- |\n\
| **Expanded Matrix Jobs** | `{} jobs` | **`{} jobs`** | **`{:+} jobs`** |\n\
| **Billed Minutes / Run** | `{:.1} mins` | **`{:.1} mins`** | **`{:+.1} mins / run`** |\n\
| **Free-Tier Quota Burn %** | `{:.1}%` | **`{:.1}%`** | **`{:+.1}%` monthly quota** |\n\
| **Monthly CI Compute Cost** | `${:.2} / mo` | **`${:.2} / mo`** | **`{}{:.2} / mo` (`{}{:.1}%`)** |\n\
| **Monthly Developer Wait-Time (`$`)** | `${:.2} / mo` | **`${:.2} / mo`** | **`{}{:.2} / mo`** |\n\n\
#### 🔍 Workflow & Matrix Cost Drivers Changed in This PR\n\
{}\n\
{}",
        d.execution_time_us,
        d.base_expanded_jobs,
        d.head_expanded_jobs,
        d.delta_expanded_jobs,
        d.base_billed_mins_per_run,
        d.head_billed_mins_per_run,
        d.delta_billed_mins_per_run,
        d.base_free_quota_burn_pct,
        d.head_free_quota_burn_pct,
        d.head_free_quota_burn_pct - d.base_free_quota_burn_pct,
        d.base_monthly_compute_usd,
        d.head_monthly_compute_usd,
        sign,
        d.delta_monthly_compute_usd,
        sign,
        d.delta_monthly_compute_pct,
        d.base_monthly_dev_wait_usd,
        d.head_monthly_dev_wait_usd,
        wait_sign,
        d.delta_monthly_dev_wait_usd,
        change_lines,
        six_comp_md
    )
}

pub fn render_slack_payload(d: &DiffReport) -> String {
    let sign = if d.delta_monthly_compute_usd >= 0.0 { "+" } else { "" };
    let text = format!(
        "MiniCI PR Cost Diff: {}${:.2}/mo ({}{:.1}%) | Jobs: {} -> {} | Dev Wait: ${:.2}/mo",
        sign,
        d.delta_monthly_compute_usd,
        sign,
        d.delta_monthly_compute_pct,
        d.base_expanded_jobs,
        d.head_expanded_jobs,
        d.head_monthly_dev_wait_usd
    );
    format!(
        "{{\n  \"text\": \"{}\",\n  \"blocks\": [\n    {{\n      \"type\": \"header\",\n      \"text\": {{ \"type\": \"plain_text\", \"text\": \"MiniCI (Skipwise) Pull Request CI/CD Cost Diff\" }}\n    }},\n    {{\n      \"type\": \"section\",\n      \"fields\": [\n        {{ \"type\": \"mrkdwn\", \"text\": \"*Monthly Compute Delta:*\\n`{}${:.2}/mo` (`{}{:.1}%`)\" }},\n        {{ \"type\": \"mrkdwn\", \"text\": \"*Expanded Matrix Jobs:*\\n`{} -> {} jobs` (`{:+} jobs`)\" }},\n        {{ \"type\": \"mrkdwn\", \"text\": \"*Free-Tier Quota Burn:*\\n`{:.1}% -> {:.1}%`\" }},\n        {{ \"type\": \"mrkdwn\", \"text\": \"*Developer Wait Cost:*\\n`${:.2}/mo -> ${:.2}/mo`\" }}\n      ]\n    }}\n  ]\n}}",
        json_escape(&text),
        sign,
        d.delta_monthly_compute_usd,
        sign,
        d.delta_monthly_compute_pct,
        d.base_expanded_jobs,
        d.head_expanded_jobs,
        d.delta_expanded_jobs,
        d.base_free_quota_burn_pct,
        d.head_free_quota_burn_pct,
        d.base_monthly_dev_wait_usd,
        d.head_monthly_dev_wait_usd
    )
}

pub fn generate_post_tool_use_launch_pack(feature_name: &str, sample_latency_us: u128) -> String {
    format!(
        "# 🚀 AUTOMATIC `PostToolUse` 6-CHANNEL NATURAL-LANGUAGE LAUNCH & PARTNER PROFILE PACK (`{feature_name}`)\n\n\
> **Founder & Publisher Contact:** Aamir Raza (`raza.aamir01@gmail.com`)\n\
> **Category Positioning:** **CI/CD with Taste** (`taste-ci-1`) — *\"If we were building CI/CD for software built with AI, MiniCI is what it would look like.\"*\n\
> **Pricing Model (`CommandCode.ai` Style):** **Go (`$1/mo` — 9K PR/hook audits)** • **GOAT (`$10/mo` — 75K audits, Most Popular)** • **Pro (`$20/mo` — 100K audits)** • **Provider (`$15/mo` — Zero-markup API/MCP)** • **Team Pro (`$40/mo` — Pooled crew billing)** • **Max 10×/20× (`$100–$200/mo`)** • **Enterprise (`$5,000+/mo`)** *(Unused credits roll over and never expire)*\n\
> **Empirically Measured Portable Rust Benchmark:** `{sample_latency_us} µs` (`{:.3} ms`) on 1–2 CPU threads • Zero External Dependencies\n\n\
---\n\n\
## 1. Y Combinator (YC) Application — Product & Technical Moat Profile\n\
- **Founder:** Aamir Raza (`raza.aamir01@gmail.com`)\n\
- **Company / Project:** MiniCI (`Skipwise`) — **CI/CD with Taste**\n\
- **50-Character Description:** CI/CD with Taste: sub-ms Rust PR cost & quota gate.\n\
- **What we built (`{feature_name}`):** If you were building CI/CD for software built with AI, **MiniCI (`minici`)** is what it would look like. AI coding agents push 4x more commits per day, silently expanding CI matrices and burning runner budgets. We built `minici` 100% in **Rust** (`{sample_latency_us} µs` execution, zero runtime dependencies) with our **`taste-ci-1` FinOps memory** and a frictionless **CommandCode.ai-inspired pricing model** starting at **`$1/mo` (`Go`)** and **`$10/mo` (`GOAT` — 75K PR/hook audits)** up to **`$40/mo` (`Team Pro`)** and **`$5,000+/mo` (`Enterprise`)**. Every `git push` or PR gets an instant `< 1 ms` receipt showing **Free-Tier Quota Burn %**, **Developer Wait-Time Dollars (`$65/hr` default)**, **Billed Compute Cost (with GitHub's 1-minute per-job ceiling rounding and macOS 10x multiplier)**, **Hidden Artifact Storage Markup (`$0.25/GB-mo` vs `$0.023` S3)**, and **Self-Hosted/Blacksmith Break-Even (`5,000 mins/mo`)**.\n\n\
## 2. Product Hunt — Maker Launch Listing\n\
- **Name:** MiniCI (`Skipwise`) — CI/CD with Taste\n\
- **Maker Contact:** Aamir Raza (`raza.aamir01@gmail.com`)\n\
- **Tagline:** If we were building CI/CD for software built with AI — `< 1ms` Rust PR cost & quota receipts starting at `$1/mo`.\n\
- **Maker Comment (Natural Voice):** Hey Product Hunt! AI coding agents are shipping code faster than ever, but CI/CD hasn't caught up—one tiny `strategy.matrix` edit in a PR can silently multiply your GitHub Actions or GitLab bill by 10x. I built `minici` in pure Rust (`{sample_latency_us} µs` execution, zero dependencies): it brings **CI/CD with Taste** to every repo and local `pre-push` hook, showing your exact Free Quota Burn %, Developer Wait-Time cost, and Per-PR Compute Invoice before merge. Following the developer-first pricing playbook of CommandCode.ai, plans start at **`$1/mo` (`Go`)** and **`$10/mo` (`GOAT`)** with rollover credits that never expire.\n\n\
## 3. Reddit (`r/devops`, `r/rust`, `r/sre`) Technical Post (Natural Engineer Voice)\n\
- **Title:** `I built a zero-dependency Rust CLI (\"CI/CD with Taste\") that expands GitHub/GitLab/CircleCI workflow matrices and calculates PR cost diffs in {sample_latency_us} µs`\n\
- **Body:** GitHub Actions and GitLab CI only show you a lump-sum bill at the end of the month—they never tell you which PR or `strategy.matrix` change spiked your runner bill or burned your 2,000 free minutes. I wrote `minici` in pure Rust with an embedded zero-copy SKU catalog (supporting 1-minute per-job ceiling rounding, macOS 10x multipliers, `$0.25/GB` artifact storage tax, developer wait-time cost, and the 5,000 min/mo self-hosted break-even threshold). Always exits `0` so it never blocks a build. Feedback welcome: `raza.aamir01@gmail.com`.\n\n\
## 4. LinkedIn — Founder Build-in-Public Post (Natural Staff SRE Voice)\n\
- **Post:** If we were building CI/CD for software built with AI, what would it look like? It wouldn't wait 30 days to send you a surprise GitHub Actions invoice after an AI coding agent expanded a workflow matrix to `macos-latest`. So I built **MiniCI (`minici`) — CI/CD with Taste** in pure Rust. In **{sample_latency_us} microseconds** (`< 1 ms`), it expands your CI matrices and posts the exact Per-PR & Monthly Cost Diff, Free-Tier Quota Burn %, and Developer Wait-Time cost right on the PR or local `pre-push` hook. Inspired by Ahmad Awais's CommandCode.ai model, we made pricing dead simple: open-source CLI locally, `$1/mo` Go tier, `$10/mo` GOAT tier, and `$40/mo` Team Pro. Reach out anytime at `raza.aamir01@gmail.com`.\n\n\
## 5. AI Coding Agent & IDE (`PreToolUse` / `PostToolUse` / MCP) Profile\n\
- **Integration Name:** `minici` — Sub-Millisecond CI/CD with Taste Guard for AI Coding Agents\n\
- **Maintainer:** Aamir Raza (`raza.aamir01@gmail.com`)\n\
- **Hook Specification:** Plugs directly into IDE & Git hooks (`PreToolUse` / `PostToolUse` on `git push` or `.github/workflows/*.yml` edits) in `{sample_latency_us} µs` with `exit 0` informational receipts so coding agents respect your repo's CI budget (`taste-ci-1`) before pushing.\n\n\
## 6. Mozilla Builders / Open-Source AI & DevTools Grant Profile\n\
- **Applicant:** Aamir Raza (`raza.aamir01@gmail.com`)\n\
- **Project Summary:** Open-source, zero-telemetry-egress Rust binary (`Apache-2.0`) that brings `CI/CD with Taste` locally to any developer machine or cheap 1–2 vCPU container without sending proprietary source code to third-party servers.\n",
        (sample_latency_us as f64) / 1000.0
    )
}
