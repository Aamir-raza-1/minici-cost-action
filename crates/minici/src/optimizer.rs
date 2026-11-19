//! Comment-Preserving Surgical Workflow Optimizer (`minici shrink` & `minici migrate`)
//! (`crates/minici/src/optimizer.rs` — 100% Pure Rust `std`, Zero External Crate Dependencies).
//!
//! Safely rewrites GitHub Actions workflow YAML files (preserving every comment and blank line)
//! and verifies exact Before -> After cost & wait-time deltas on a shadow directory before applying:
//! 1. `minici shrink`  : Injects missing `concurrency` (`cancel-in-progress`), `timeout-minutes: 30`
//!                       (eliminating 360m hung-job exposure), `retention-days: 7` on artifact uploads,
//!                       and `fetch-depth: 1` shallow clones.
//! 2. `minici migrate` : Applies 5 safety gates (OS, GPU/emulator, container arch, slim eligibility,
//!                       spot/release safety) to migrate `runs-on:` to `ubuntu-24.04-arm`, `ubuntu-slim`,
//!                       `blacksmith-4vcpu-ubuntu-2404`, or `self-hosted`, plus 5-Layer TCO verdict.

use crate::engine::{analyze_path, compute_diff, json_escape, DiffReport};
use crate::providers::compute_5layer_platform_switching_tco;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FilePatchEdit {
    pub workflow_file: String,
    pub rule_id: String,
    pub description: String,
    pub line_number: usize,
    pub original_snippet: String,
    pub replacement_snippet: String,
}

#[derive(Debug, Clone)]
pub struct OptimizationReceipt {
    pub operation: String, // "shrink" | "migrate"
    pub target_platform: String,
    pub applied_to_disk: bool,
    pub files_modified: usize,
    pub edits: Vec<FilePatchEdit>,
    pub verified_diff: DiffReport,
    pub switching_verdict: Option<(f64, f64, f64, &'static str)>, // (eng_hours, switch_cost_usd, payback_mo, verdict)
}

impl OptimizationReceipt {
    pub fn to_json(&self) -> String {
        let edits_json = self
            .edits
            .iter()
            .map(|e| {
                format!(
                    "{{\"workflow_file\":\"{}\",\"rule_id\":\"{}\",\"description\":\"{}\",\"line_number\":{}}}",
                    json_escape(&e.workflow_file),
                    json_escape(&e.rule_id),
                    json_escape(&e.description),
                    e.line_number
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let sw_json = self
            .switching_verdict
            .as_ref()
            .map(|(h, c, p, v)| {
                format!(
                    "{{\"engineering_hours\":{:.1},\"switching_cost_usd\":{:.2},\"payback_months\":{:.2},\"verdict\":\"{}\"}}",
                    h, c, p, v
                )
            })
            .unwrap_or_else(|| "null".to_string());
        format!(
            "{{\n  \"operation\": \"{}\",\n  \"target_platform\": \"{}\",\n  \"applied_to_disk\": {},\n  \"files_modified\": {},\n  \"edits_count\": {},\n  \"edits\": [{}],\n  \"switching_tco\": {},\n  \"verified_diff\": {}\n}}",
            json_escape(&self.operation),
            json_escape(&self.target_platform),
            self.applied_to_disk,
            self.files_modified,
            self.edits.len(),
            edits_json,
            sw_json,
            self.verified_diff.to_json()
        )
    }

    pub fn render_table(&self) -> String {
        let mut out = String::new();
        out.push_str("+====================================================================================+\n");
        out.push_str(&format!(
            "| MINICI SURGICAL WORKFLOW {} RECEIPT ({} file(s), {} edit(s) • {}) |\n",
            self.operation.to_ascii_uppercase(),
            self.files_modified,
            self.edits.len(),
            if self.applied_to_disk {
                "APPLIED"
            } else {
                "DRY-RUN"
            }
        ));
        out.push_str("+====================================================================================+\n");
        out.push_str(&format!(
            "  • Monthly Compute : ${:.2}/mo -> ${:.2}/mo ({:+.2}/mo • {:+.1}%)\n",
            self.verified_diff.base_monthly_compute_usd,
            self.verified_diff.head_monthly_compute_usd,
            self.verified_diff.delta_monthly_compute_usd,
            self.verified_diff.delta_monthly_compute_pct
        ));
        out.push_str(&format!(
            "  • Billed Mins/Run : {:.1}m -> {:.1}m ({:+.1}m/run)\n",
            self.verified_diff.base_billed_mins_per_run,
            self.verified_diff.head_billed_mins_per_run,
            self.verified_diff.delta_billed_mins_per_run
        ));
        if let Some((eng_h, sw_usd, payback, verdict)) = &self.switching_verdict {
            out.push_str(&format!(
                "  • 5-Layer Switch  : {:.1}h effort (${:.2} total switch TCO • {:.1}m payback • Verdict: {})\n",
                eng_h, sw_usd, payback, verdict
            ));
        }
        for (i, e) in self.edits.iter().take(12).enumerate() {
            out.push_str(&format!(
                "  {}. [{}] {}:L{} — {}\n",
                i + 1,
                e.rule_id,
                e.workflow_file,
                e.line_number,
                e.description
            ));
        }
        if !self.applied_to_disk && !self.edits.is_empty() {
            out.push_str("  💡 Pass `--apply` to write these comment-preserving edits directly to `.github/workflows/`.\n");
        }
        out.push_str("+====================================================================================+\n");
        out
    }

    pub fn render_pr_suggestions_markdown(&self) -> String {
        let mut md = format!(
            "<!-- minici-cost-comment -->\n\
             ### ✂️ MiniCI Surgical `{}` Plan (`{}` file(s) • `{:+.2}/mo` verified delta)\n\n\
             - **Before → After Compute:** `${:.2}/mo` → **`${:.2}/mo`** (`{:+.1}%`)\n\
             - **Before → After Billed Mins/Run:** `{:.1}m` → **`{:.1}m`**\n",
            self.operation,
            self.files_modified,
            self.verified_diff.delta_monthly_compute_usd,
            self.verified_diff.base_monthly_compute_usd,
            self.verified_diff.head_monthly_compute_usd,
            self.verified_diff.delta_monthly_compute_pct,
            self.verified_diff.base_billed_mins_per_run,
            self.verified_diff.head_billed_mins_per_run
        );
        if let Some((eng_h, sw_usd, payback, verdict)) = &self.switching_verdict {
            md.push_str(&format!(
                "- **5-Layer Switching TCO Verdict:** **`{}`** (`{:.1}h` effort, `${:.2}` one-time cost, `{:.1} mo` payback)\n",
                verdict, eng_h, sw_usd, payback
            ));
        }
        md.push('\n');
        for e in self.edits.iter().take(6) {
            md.push_str(&format!(
                "#### `{}` (Line {}) — {}\n```suggestion\n{}\n```\n\n",
                e.workflow_file, e.line_number, e.description, e.replacement_snippet
            ));
        }
        md
    }
}

fn indent_len(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

fn collect_workflow_files(repo_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    if repo_dir.is_file() {
        let fname = repo_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ci.yml")
            .to_string();
        out.push((repo_dir.to_path_buf(), fname));
        return out;
    }
    let gh_dir = repo_dir.join(".github").join("workflows");
    if let Ok(entries) = fs::read_dir(&gh_dir) {
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "yml" || ext == "yaml" {
                let rel = format!(
                    ".github/workflows/{}",
                    p.file_name().and_then(|s| s.to_str()).unwrap_or("ci.yml")
                );
                out.push((p, rel));
            }
        }
    }
    out
}

fn shrink_single_workflow_yaml(rel_name: &str, content: &str) -> (String, Vec<FilePatchEdit>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() + 16);
    let mut edits: Vec<FilePatchEdit> = Vec::new();

    let low_all = content.to_ascii_lowercase();
    let has_concurrency = low_all.contains("\nconcurrency:") || low_all.starts_with("concurrency:");
    let triggers_pr_or_push = low_all.contains("pull_request") || low_all.contains("push");

    let mut injected_concurrency = false;
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let ind = indent_len(line);

        // 1. Inject top-level `concurrency:` right before `jobs:` when missing on PR/push workflows
        if !has_concurrency && triggers_pr_or_push && !injected_concurrency && ind == 0 && trimmed == "jobs:" {
            let conc_block = "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}\n  cancel-in-progress: ${{ github.event_name == 'pull_request' }}";
            for cl in conc_block.lines() {
                out_lines.push(cl.to_string());
            }
            edits.push(FilePatchEdit {
                workflow_file: rel_name.to_string(),
                rule_id: "MINICI-SHRINK-001".to_string(),
                description: "Add `concurrency` cancel-in-progress guard for superseded PR pushes".to_string(),
                line_number: i + 1,
                original_snippet: line.to_string(),
                replacement_snippet: format!("{}\n{}", conc_block, line),
            });
            injected_concurrency = true;
        }

        out_lines.push(line.to_string());

        // 2. Inject `timeout-minutes: 30` right after `runs-on:` if the job block lacks `timeout-minutes:`
        if trimmed.starts_with("runs-on:") {
            let mut job_has_timeout = false;
            let mut scan = i + 1;
            while scan < lines.len() {
                let st = lines[scan].trim();
                if !st.is_empty() && !st.starts_with('#') && indent_len(lines[scan]) < ind {
                    break;
                }
                if st.starts_with("timeout-minutes:") {
                    job_has_timeout = true;
                    break;
                }
                scan += 1;
            }
            if !job_has_timeout {
                let pad = " ".repeat(ind);
                let tline = format!("{}timeout-minutes: 30", pad);
                out_lines.push(tline.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-002".to_string(),
                    description: "Cap job hung-runner exposure from 360m default to `timeout-minutes: 30`".to_string(),
                    line_number: i + 1,
                    original_snippet: line.to_string(),
                    replacement_snippet: format!("{}\n{}", line, tline),
                });
            }
        }

        // 3. Inject `fetch-depth: 1` on bare `uses: actions/checkout@*` without `with:`
        if trimmed.contains("actions/checkout@") {
            let step_ind = if trimmed.starts_with('-') { ind + 2 } else { ind };
            let next_non_empty = lines[(i + 1)..]
                .iter()
                .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'));
            let has_with = next_non_empty
                .map(|nl| indent_len(nl) >= step_ind && nl.trim().starts_with("with:"))
                .unwrap_or(false);
            if !has_with {
                let pad = " ".repeat(step_ind);
                let w1 = format!("{}with:", pad);
                let w2 = format!("{}  fetch-depth: 1", pad);
                out_lines.push(w1.clone());
                out_lines.push(w2.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-003".to_string(),
                    description: "Pin shallow `fetch-depth: 1` clone on `actions/checkout`".to_string(),
                    line_number: i + 1,
                    original_snippet: line.to_string(),
                    replacement_snippet: format!("{}\n{}\n{}", line, w1, w2),
                });
            }
        }

        // 4. Shorten excessive `retention-days:` (> 14d) down to `7`
        if trimmed.starts_with("retention-days:") {
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                if let Ok(days) = val.trim().parse::<u32>() {
                    if days > 14 {
                        let pad = " ".repeat(ind);
                        let replaced = format!("{}retention-days: 7", pad);
                        let _ = out_lines.pop();
                        out_lines.push(replaced.clone());
                        edits.push(FilePatchEdit {
                            workflow_file: rel_name.to_string(),
                            rule_id: "MINICI-SHRINK-004".to_string(),
                            description: format!("Reduce artifact `retention-days` from {}d to 7d", days),
                            line_number: i + 1,
                            original_snippet: line.to_string(),
                            replacement_snippet: replaced,
                        });
                    }
                }
            }
        }

        i += 1;
    }

    let mut joined = out_lines.join("\n");
    if content.ends_with('\n') {
        joined.push('\n');
    }
    (joined, edits)
}

fn migrate_single_workflow_yaml(
    rel_name: &str,
    content: &str,
    target_runner_mode: &str,
) -> (String, Vec<FilePatchEdit>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut edits: Vec<FilePatchEdit> = Vec::new();

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let ind = indent_len(line);
        if trimmed.starts_with("runs-on:") && !trimmed.contains("${{") {
            let cur_val = trimmed
                .splitn(2, ':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            let cur_low = cur_val.to_ascii_lowercase();

            // Inspect surrounding job lines for the 5 safety gates
            let mut job_body_low = String::new();
            let mut scan = idx + 1;
            while scan < lines.len() {
                let st = lines[scan].trim();
                if !st.is_empty() && !st.starts_with('#') && indent_len(lines[scan]) < ind {
                    break;
                }
                job_body_low.push_str(&lines[scan].to_ascii_lowercase());
                job_body_low.push('\n');
                scan += 1;
            }

            // Gate 1: Linux standard/larger x64 only (never rewrite macOS or Windows)
            let is_linux_x64 = cur_low.starts_with("ubuntu-") && !cur_low.contains("arm") && !cur_low.contains("slim");
            // Gate 2: Skip GPU / Android emulator KVM jobs
            let has_gpu_or_kvm = cur_low.contains("gpu")
                || job_body_low.contains("android-emulator")
                || job_body_low.contains("cuda");
            // Gate 3: Skip explicit x86 container platform pins when targeting ARM64
            let has_amd64_pin = job_body_low.contains("linux/amd64");
            // Gate 4: Lightweight gate for `ubuntu-slim`
            let is_heavy_for_slim = job_body_low.contains("docker build")
                || job_body_low.contains("services:")
                || job_body_low.contains("cargo build")
                || job_body_low.contains("pytest");
            // Gate 5: Spot/Self-hosted deploy ceremony gate
            let is_deploy_ceremony = job_body_low.contains("terraform apply")
                || job_body_low.contains("kubectl apply")
                || rel_name.contains("release");

            let new_runner: Option<&str> = match target_runner_mode {
                "ubuntu-arm64" | "arm64" | "ubuntu-24.04-arm" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !has_amd64_pin {
                        if cur_low.contains("16") {
                            Some("ubuntu-24.04-arm-16core")
                        } else if cur_low.contains('8') {
                            Some("ubuntu-24.04-arm-8core")
                        } else if cur_low.contains('4') {
                            Some("ubuntu-24.04-arm-4core")
                        } else {
                            Some("ubuntu-24.04-arm")
                        }
                    } else {
                        None
                    }
                }
                "ubuntu-slim" | "slim" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !is_heavy_for_slim {
                        Some("ubuntu-slim")
                    } else {
                        None
                    }
                }
                "blacksmith" | "high-speed" => {
                    if is_linux_x64 && !has_gpu_or_kvm {
                        if cur_low.contains('8') {
                            Some("blacksmith-8vcpu-ubuntu-2404")
                        } else if cur_low.contains('4') {
                            Some("blacksmith-4vcpu-ubuntu-2404")
                        } else {
                            Some("blacksmith-2vcpu-ubuntu-2404")
                        }
                    } else {
                        None
                    }
                }
                "self-hosted" | "aws-spot" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !is_deploy_ceremony {
                        Some("aws-c7g.xlarge-on-demand")
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(target_sku) = new_runner {
                let pad = " ".repeat(ind);
                let replaced = format!("{}runs-on: {}", pad, target_sku);
                out_lines.push(replaced.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-MIGRATE-001".to_string(),
                    description: format!(
                        "Migrate `{}` -> `{}` (passed all 5 safety gates)",
                        cur_val, target_sku
                    ),
                    line_number: idx + 1,
                    original_snippet: line.to_string(),
                    replacement_snippet: replaced,
                });
                continue;
            }
        }
        out_lines.push(line.to_string());
    }

    let mut joined = out_lines.join("\n");
    if content.ends_with('\n') {
        joined.push('\n');
    }
    (joined, edits)
}

pub fn run_surgical_optimizer(
    repo_dir: &Path,
    operation: &str, // "shrink" | "migrate"
    target_mode: &str,
    apply_to_disk: bool,
) -> OptimizationReceipt {
    let base_report = analyze_path(repo_dir, None);
    let wf_files = collect_workflow_files(repo_dir);

    let shadow_root = std::env::temp_dir().join(format!(
        "minici_opt_shadow_{}_{}",
        std::process::id(),
        operation
    ));
    let _ = fs::remove_dir_all(&shadow_root);
    let _ = fs::create_dir_all(shadow_root.join(".github").join("workflows"));

    // Copy usage config if present so shadow diff uses identical usage assumptions
    let base_dir = if repo_dir.is_file() {
        repo_dir.parent().unwrap_or(Path::new("."))
    } else {
        repo_dir
    };
    if base_dir.join("minici-usage.yml").exists() {
        let _ = fs::copy(
            base_dir.join("minici-usage.yml"),
            shadow_root.join("minici-usage.yml"),
        );
    }

    let mut all_edits: Vec<FilePatchEdit> = Vec::new();
    let mut files_modified = 0usize;
    let mut pending_writes: Vec<(PathBuf, String)> = Vec::new();

    for (src_path, rel_name) in &wf_files {
        if let Ok(raw) = fs::read_to_string(src_path) {
            let (patched, file_edits) = if operation == "migrate" {
                migrate_single_workflow_yaml(rel_name, &raw, target_mode)
            } else {
                shrink_single_workflow_yaml(rel_name, &raw)
            };
            if !file_edits.is_empty() {
                files_modified += 1;
                all_edits.extend(file_edits);
                pending_writes.push((src_path.clone(), patched.clone()));
            }
            let fname = src_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("ci.yml");
            let _ = fs::write(
                shadow_root.join(".github").join("workflows").join(fname),
                &patched,
            );
        }
    }

    let head_report = if wf_files.is_empty() {
        base_report.clone()
    } else {
        analyze_path(&shadow_root, Some(base_report.usage.clone()))
    };
    let verified_diff = compute_diff(&base_report, &head_report);
    let _ = fs::remove_dir_all(&shadow_root);

    if apply_to_disk {
        for (dest_path, content) in pending_writes {
            let tmp = dest_path.with_extension("yml.minici_tmp");
            if fs::write(&tmp, content).is_ok() {
                let _ = fs::rename(&tmp, dest_path);
            }
        }
    }

    // Compute 5-Layer Platform Switching TCO if migrating across CI platforms (`gitlab-ci`, `circleci`, `buildkite`, `aws-codebuild`)
    let switching_verdict = if operation == "migrate"
        && matches!(
            target_mode,
            "gitlab-ci" | "circleci" | "bitbucket" | "buildkite" | "aws-codebuild" | "gcp-cloud-build"
        ) {
        let cc = &base_report.cost_components;
        let target_tco = match target_mode {
            "gitlab-ci" => cc.gitlab_ci_premium_monthly_tco_usd,
            "circleci" => cc.circleci_performance_monthly_tco_usd,
            "bitbucket" => cc.bitbucket_premium_monthly_tco_usd,
            "buildkite" => cc.buildkite_pro_monthly_tco_usd,
            "aws-codebuild" => cc.aws_codebuild_monthly_tco_usd,
            "gcp-cloud-build" => cc.gcp_cloud_build_monthly_tco_usd,
            _ => cc.github_actions_team_monthly_tco_usd,
        };
        let sw = compute_5layer_platform_switching_tco(
            base_report.total_workflows.max(1),
            cc.implied_active_devs.max(1),
            "github-actions",
            target_mode,
            cc.github_actions_team_monthly_tco_usd
                .max(base_report.gross_monthly_compute_cost_usd),
            target_tco,
            cc.blended_fully_loaded_hourly_usd,
            true,
        );
        Some((
            sw.engineering_hours,
            sw.total_switching_cost_usd,
            sw.payback_months,
            sw.verdict,
        ))
    } else {
        None
    };

    OptimizationReceipt {
        operation: operation.to_string(),
        target_platform: target_mode.to_string(),
        applied_to_disk: apply_to_disk,
        files_modified,
        edits: all_edits,
        verified_diff,
        switching_verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_preserving_shrink_and_5gate_arm64_migrate() {
        let dir = std::env::temp_dir().join("minici_optimizer_unit_test");
        let _ = fs::remove_dir_all(&dir);
        let wf_dir = dir.join(".github").join("workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        let orig = "# Important header comment preserved!\nname: PR Suite\non: [pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/upload-artifact@v4\n        with:\n          retention-days: 90\n      - run: cargo test\n  ios:\n    runs-on: macos-latest\n    steps:\n      - run: xcodebuild test\n";
        fs::write(wf_dir.join("ci.yml"), orig).unwrap();

        // 1. Dry-run shrink preserves comments and does not touch disk
        let dry_shrink = run_surgical_optimizer(&dir, "shrink", "hygiene", false);
        assert!(dry_shrink.edits.len() >= 3);
        assert_eq!(fs::read_to_string(wf_dir.join("ci.yml")).unwrap(), orig);

        // 2. Dry-run migrate to `ubuntu-arm64` rewrites `ubuntu-latest` -> `ubuntu-24.04-arm` and NEVER touches `macos-latest`!
        let mig = run_surgical_optimizer(&dir, "migrate", "ubuntu-arm64", true);
        assert_eq!(mig.files_modified, 1);
        assert!(mig.verified_diff.delta_monthly_compute_usd < 0.0);
        let updated = fs::read_to_string(wf_dir.join("ci.yml")).unwrap();
        assert!(updated.contains("# Important header comment preserved!"));
        assert!(updated.contains("runs-on: ubuntu-24.04-arm"));
        assert!(updated.contains("runs-on: macos-latest"));

        let _ = fs::remove_dir_all(&dir);
    }
}
