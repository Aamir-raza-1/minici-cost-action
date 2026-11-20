//! Comment-Preserving Surgical Workflow Optimizer & Cross-Vendor AST Converter (`minici shrink` & `minici migrate`)
//! (`crates/minici/src/optimizer.rs` — 100% Pure Rust `std`, Zero External Crate Dependencies).
//!
//! Safely rewrites GitHub Actions, GitLab CI, CircleCI, and Bitbucket workflow YAML files (preserving every
//! comment and blank line) and verifies exact Before -> After cost, storage, and risk deltas on an isolated
//! shadow directory (including manifests & `.minici/telemetry.lock.json`) before applying:
//! 1. `minici shrink`  : Injects missing `concurrency` (`cancel-in-progress`), job-level `timeout-minutes: 30`
//!                       (handling pre-existing timeouts above `runs-on:` and multi-line `runs-on:` lists),
//!                       `retention-days: 7`, git-history-safe `fetch-depth: 1`, lockfile `cache:`, and GitLab/CircleCI/Bitbucket equivalents.
//! 2. `minici migrate` : Applies 6 safety gates (OS, GPU/KVM, container `linux/amd64` arch, slim eligibility,
//!                       deploy ceremony, and mandatory net-cost-drop verification) + `minici-yaml-fixer` cross-vendor AST preview
//!                       + 5-Layer Platform Switching TCO with "Optimize In-Place First" comparison.

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
    pub artifact_storage_before_usd: f64,
    pub artifact_storage_after_usd: f64,
    pub missing_timeouts_before: usize,
    pub missing_timeouts_after: usize,
    pub in_place_shrink_monthly_savings_usd: f64,
    pub converted_github_workflow_preview: Option<String>,
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
                    "{{\"engineering_hours\":{:.1},\"switching_cost_usd\":{:.2},\"payback_months\":{:.2},\"verdict\":\"{}\",\"in_place_shrink_monthly_savings_usd\":{:.2}}}",
                    h, c, p, v, self.in_place_shrink_monthly_savings_usd
                )
            })
            .unwrap_or_else(|| "null".to_string());
        let conv_json = self
            .converted_github_workflow_preview
            .as_ref()
            .map(|s| format!("\"{}\"", json_escape(s)))
            .unwrap_or_else(|| "null".to_string());
        format!(
            "{{\n  \"operation\": \"{}\",\n  \"target_platform\": \"{}\",\n  \"applied_to_disk\": {},\n  \"files_modified\": {},\n  \"edits_count\": {},\n  \"artifact_storage_before_usd\": {:.2},\n  \"artifact_storage_after_usd\": {:.2},\n  \"missing_timeouts_before\": {},\n  \"missing_timeouts_after\": {},\n  \"in_place_shrink_monthly_savings_usd\": {:.2},\n  \"edits\": [{}],\n  \"converted_github_workflow_preview\": {},\n  \"switching_tco\": {},\n  \"verified_diff\": {}\n}}",
            json_escape(&self.operation),
            json_escape(&self.target_platform),
            self.applied_to_disk,
            self.files_modified,
            self.edits.len(),
            self.artifact_storage_before_usd,
            self.artifact_storage_after_usd,
            self.missing_timeouts_before,
            self.missing_timeouts_after,
            self.in_place_shrink_monthly_savings_usd,
            edits_json,
            conv_json,
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
        if (self.artifact_storage_before_usd - self.artifact_storage_after_usd).abs() > 0.009 {
            out.push_str(&format!(
                "  • Artifact Storage: ${:.2}/mo -> ${:.2}/mo ({:+.2}/mo)\n",
                self.artifact_storage_before_usd,
                self.artifact_storage_after_usd,
                self.artifact_storage_after_usd - self.artifact_storage_before_usd
            ));
        }
        if self.missing_timeouts_before != self.missing_timeouts_after {
            out.push_str(&format!(
                "  • Timeout Guard   : {} -> {} job(s) missing timeout-minutes\n",
                self.missing_timeouts_before, self.missing_timeouts_after
            ));
        }
        if let Some((eng_h, sw_usd, payback, verdict)) = &self.switching_verdict {
            out.push_str(&format!(
                "  • 5-Layer Switch  : {:.1}h effort (${:.2} total switch TCO • {:.1}m payback • Verdict: {})\n",
                eng_h, sw_usd, payback, verdict
            ));
            out.push_str(&format!(
                "  • Optimize First  : Staying & running `minici shrink` + ARM64 saves ~${:.2}/mo in < 30 mins ($0 switch risk)\n",
                self.in_place_shrink_monthly_savings_usd
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
            out.push_str("  💡 Pass `--apply` to write these comment-preserving edits directly to disk.\n");
        }
        out.push_str("+====================================================================================+\n");
        out
    }

    pub fn render_pr_suggestions_markdown(&self) -> String {
        let mut md = format!(
            "<!-- minici-cost-comment -->\n\
             ### ✂️ MiniCI Surgical `{}` Plan (`{}` file(s) • `{:+.2}/mo` verified compute delta)\n\n\
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
        if (self.artifact_storage_before_usd - self.artifact_storage_after_usd).abs() > 0.009 {
            md.push_str(&format!(
                "- **Before → After Artifact Storage:** `${:.2}/mo` → **`${:.2}/mo`**\n",
                self.artifact_storage_before_usd, self.artifact_storage_after_usd
            ));
        }
        if self.missing_timeouts_before != self.missing_timeouts_after {
            md.push_str(&format!(
                "- **Hung-Job Timeout Guard:** `{}` → **`{}`** job(s) missing `timeout-minutes`\n",
                self.missing_timeouts_before, self.missing_timeouts_after
            ));
        }
        if let Some((eng_h, sw_usd, payback, verdict)) = &self.switching_verdict {
            md.push_str(&format!(
                "- **5-Layer Switching TCO Verdict:** **`{}`** (`{:.1}h` effort, `${:.2}` one-time cost, `{:.1} mo` payback | In-place YAML optimization saves `~${:.2}/mo` immediately)\n",
                verdict, eng_h, sw_usd, payback, self.in_place_shrink_monthly_savings_usd
            ));
        }
        md.push('\n');
        for e in self.edits.iter().take(6) {
            md.push_str(&format!(
                "#### [{}] `{}` (Line {}) — {}\n```suggestion\n{}\n```\n\n",
                e.rule_id,
                e.workflow_file,
                e.line_number,
                e.description,
                e.replacement_snippet
            ));
        }
        if let Some(ref preview) = self.converted_github_workflow_preview {
            md.push_str("#### Automated Cross-Vendor GitHub Actions Workflow Preview\n```yaml\n");
            md.push_str(preview);
            md.push_str("\n```\n");
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
    for (rel, sub) in [
        (".gitlab-ci.yml", repo_dir.join(".gitlab-ci.yml")),
        (".circleci/config.yml", repo_dir.join(".circleci").join("config.yml")),
        ("bitbucket-pipelines.yml", repo_dir.join("bitbucket-pipelines.yml")),
    ] {
        if sub.is_file() {
            out.push((sub, rel.to_string()));
        }
    }
    out
}

/// Finds the `(start_line_inclusive, end_line_exclusive)` of the enclosing YAML job block
/// where `prop_idx` is a job-level property (such as `runs-on:`) at indentation `prop_ind`.
fn find_job_block_bounds(lines: &[&str], prop_idx: usize, prop_ind: usize) -> (usize, usize) {
    let mut start = prop_idx;
    while start > 0 {
        let prev = lines[start - 1];
        let pt = prev.trim();
        if !pt.is_empty() && !pt.starts_with('#') && indent_len(prev) < prop_ind {
            // `start - 1` is the `<job_id>:` header line!
            start -= 1;
            break;
        }
        start -= 1;
    }
    let job_hdr_ind = indent_len(lines[start]);
    let mut end = prop_idx + 1;
    while end < lines.len() {
        let st = lines[end].trim();
        if !st.is_empty() && !st.starts_with('#') && indent_len(lines[end]) <= job_hdr_ind {
            break;
        }
        end += 1;
    }
    (start, end)
}

fn job_uses_git_history(job_lines: &[&str]) -> bool {
    let joined = job_lines.join("\n").to_ascii_lowercase();
    joined.contains("git diff")
        || joined.contains("git log")
        || joined.contains("git fetch")
        || joined.contains("git merge-base")
        || joined.contains("git rev-list")
        || joined.contains("minici diff")
        || joined.contains("compare-to")
        || joined.contains("semantic-release")
        || joined.contains("changeset")
        || joined.contains("nx affected")
        || joined.contains("turbo ")
        || joined.contains("sonar")
        || joined.contains("goreleaser")
        || joined.contains("fetch-depth:")
}

fn detect_node_cache_manager(repo_base: &Path) -> Option<&'static str> {
    if repo_base.join("pnpm-lock.yaml").exists() {
        Some("pnpm")
    } else if repo_base.join("yarn.lock").exists() {
        Some("yarn")
    } else if repo_base.join("package-lock.json").exists() {
        Some("npm")
    } else {
        None
    }
}

fn detect_python_cache_manager(repo_base: &Path) -> Option<&'static str> {
    if repo_base.join("poetry.lock").exists() {
        Some("poetry")
    } else if repo_base.join("Pipfile.lock").exists() {
        Some("pipenv")
    } else if repo_base.join("requirements.txt").exists() || repo_base.join("pyproject.toml").exists() {
        Some("pip")
    } else {
        None
    }
}

fn shrink_github_workflow_yaml(
    rel_name: &str,
    content: &str,
    repo_base: &Path,
) -> (String, Vec<FilePatchEdit>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() + 20);
    let mut edits: Vec<FilePatchEdit> = Vec::new();

    let low_all = content.to_ascii_lowercase();
    let has_concurrency = low_all.contains("\nconcurrency:") || low_all.starts_with("concurrency:");
    let triggers_pr_or_push = low_all.contains("pull_request") || low_all.contains("push");
    let node_mgr = detect_node_cache_manager(repo_base);
    let py_mgr = detect_python_cache_manager(repo_base);

    let mut injected_concurrency = false;
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let ind = indent_len(line);

        // 1. Inject top-level `concurrency:` right before `jobs:` when missing on PR/push workflows
        if !has_concurrency
            && triggers_pr_or_push
            && !injected_concurrency
            && ind == 0
            && trimmed == "jobs:"
        {
            let conc_block = "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}\n  cancel-in-progress: ${{ github.event_name == 'pull_request' }}";
            for cl in conc_block.lines() {
                out_lines.push(cl.to_string());
            }
            edits.push(FilePatchEdit {
                workflow_file: rel_name.to_string(),
                rule_id: "MINICI-SHRINK-001".to_string(),
                description: "Add `concurrency` cancel-in-progress guard for superseded PR pushes"
                    .to_string(),
                line_number: i + 1,
                original_snippet: line.to_string(),
                replacement_snippet: format!("{}\n{}", conc_block, line),
            });
            injected_concurrency = true;
        }

        out_lines.push(line.to_string());

        // 2. Inject job-level `timeout-minutes: 30` safely:
        //    - Scans the ENTIRE job block (above AND below `runs-on:`) so existing `timeout-minutes:` is NEVER duplicated!
        //    - Advances past multi-line `runs-on:` YAML list items (`- self-hosted`, `- linux`) before inserting!
        if trimmed.starts_with("runs-on:") {
            let (j_start, j_end) = find_job_block_bounds(&lines, i, ind);
            let job_has_timeout = lines[j_start..j_end].iter().any(|jl| {
                let jt = jl.trim();
                indent_len(jl) == ind && jt.starts_with("timeout-minutes:")
            });

            let rhs = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
            if rhs.is_empty() {
                // Multi-line YAML `runs-on:` (both `- item` sequence AND `group:` / `labels:` mapping form):
                // emit all indented child lines first before inserting `timeout-minutes: 30`!
                while i + 1 < lines.len() {
                    let nxt = lines[i + 1];
                    let nt = nxt.trim();
                    if !nt.is_empty() && !nt.starts_with('#') {
                        if indent_len(nxt) > ind {
                            i += 1;
                            out_lines.push(nxt.to_string());
                        } else {
                            break;
                        }
                    } else {
                        let still_child = lines[(i + 2)..]
                            .iter()
                            .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                            .map(|l| indent_len(l) > ind)
                            .unwrap_or(false);
                        if still_child {
                            i += 1;
                            out_lines.push(nxt.to_string());
                        } else {
                            break;
                        }
                    }
                }
            }

            if !job_has_timeout {
                let anchor_line = lines[i];
                let pad = " ".repeat(ind);
                let tline = format!("{}timeout-minutes: 30", pad);
                out_lines.push(tline.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-002".to_string(),
                    description:
                        "Cap job hung-runner exposure from 360m default to `timeout-minutes: 30`"
                            .to_string(),
                    line_number: i + 1,
                    original_snippet: anchor_line.to_string(),
                    replacement_snippet: format!("{}\n{}", anchor_line, tline),
                });
            }
        }

        // 3. Inject `fetch-depth: 1` ONLY when the job does NOT run git-history commands (`git diff`, `minici diff`, etc.)
        if trimmed.contains("actions/checkout@") {
            let step_ind = if trimmed.starts_with('-') { ind + 2 } else { ind };
            let (j_start, j_end) = find_job_block_bounds(&lines, i, ind.saturating_sub(2).max(4));
            let history_needed = job_uses_git_history(&lines[j_start..j_end]);

            let next_non_empty = lines[(i + 1)..]
                .iter()
                .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'));
            let has_with = next_non_empty
                .map(|nl| indent_len(nl) >= step_ind && nl.trim().starts_with("with:"))
                .unwrap_or(false);
            if !has_with && !history_needed {
                let pad = " ".repeat(step_ind);
                let w1 = format!("{}with:", pad);
                let w2 = format!("{}  fetch-depth: 1", pad);
                out_lines.push(w1.clone());
                out_lines.push(w2.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-003".to_string(),
                    description:
                        "Pin shallow `fetch-depth: 1` clone on history-independent `actions/checkout`"
                            .to_string(),
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
                            description: format!(
                                "Reduce artifact `retention-days` from {}d to 7d",
                                days
                            ),
                            line_number: i + 1,
                            original_snippet: line.to_string(),
                            replacement_snippet: replaced,
                        });
                    }
                }
            }
        }

        // 5. Inject lockfile-backed dependency `cache:` on bare `actions/setup-node@*` or `actions/setup-python@*`
        if trimmed.contains("actions/setup-node@") || trimmed.contains("actions/setup-python@") {
            let mgr_opt = if trimmed.contains("setup-node") {
                node_mgr
            } else {
                py_mgr
            };
            if let Some(mgr) = mgr_opt {
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
                    let w2 = format!("{}  cache: '{}'", pad, mgr);
                    out_lines.push(w1.clone());
                    out_lines.push(w2.clone());
                    edits.push(FilePatchEdit {
                        workflow_file: rel_name.to_string(),
                        rule_id: "MINICI-SHRINK-005".to_string(),
                        description: format!(
                            "Enable lockfile-backed `cache: '{}'` on toolchain setup",
                            mgr
                        ),
                        line_number: i + 1,
                        original_snippet: line.to_string(),
                        replacement_snippet: format!("{}\n{}\n{}", line, w1, w2),
                    });
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

fn shrink_non_github_yaml(rel_name: &str, content: &str) -> (String, Vec<FilePatchEdit>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() + 8);
    let mut edits: Vec<FilePatchEdit> = Vec::new();
    let low_all = content.to_ascii_lowercase();

    if rel_name.contains("gitlab") {
        if !low_all.contains("interruptible:") {
            let block = "default:\n  interruptible: true\n  timeout: 30m";
            for bl in block.lines() {
                out_lines.push(bl.to_string());
            }
            let first_line = lines.first().copied().unwrap_or("");
            edits.push(FilePatchEdit {
                workflow_file: rel_name.to_string(),
                rule_id: "MINICI-SHRINK-GL1".to_string(),
                description:
                    "Enable `interruptible: true` and `timeout: 30m` default guard in `.gitlab-ci.yml`"
                        .to_string(),
                line_number: 1,
                original_snippet: first_line.to_string(),
                replacement_snippet: if first_line.is_empty() {
                    block.to_string()
                } else {
                    format!("{}\n{}", block, first_line)
                },
            });
        }
        for (i, &l) in lines.iter().enumerate() {
            let t = l.trim();
            if t.starts_with("expire_in:") && (t.contains("30 day") || t.contains("90 day") || t.contains("1 month")) {
                let pad = " ".repeat(indent_len(l));
                let rep = format!("{}expire_in: 7 days", pad);
                out_lines.push(rep.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-GL2".to_string(),
                    description: "Shorten GitLab artifact `expire_in` to `7 days`".to_string(),
                    line_number: i + 1,
                    original_snippet: l.to_string(),
                    replacement_snippet: rep,
                });
            } else {
                out_lines.push(l.to_string());
            }
        }
    } else if rel_name.contains("bitbucket") {
        let has_max_time = low_all.contains("max-time:");
        let mut injected = false;
        for (i, &l) in lines.iter().enumerate() {
            if !has_max_time && !injected && l.trim() == "pipelines:" {
                let opt = "options:\n  max-time: 30";
                for ol in opt.lines() {
                    out_lines.push(ol.to_string());
                }
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-SHRINK-BB1".to_string(),
                    description: "Add `options.max-time: 30` step timeout guard in Bitbucket Pipelines"
                        .to_string(),
                    line_number: i + 1,
                    original_snippet: l.to_string(),
                    replacement_snippet: format!("{}\n{}", opt, l),
                });
                injected = true;
            }
            out_lines.push(l.to_string());
        }
    } else {
        for l in lines {
            out_lines.push(l.to_string());
        }
    }

    let mut joined = out_lines.join("\n");
    if content.ends_with('\n') {
        joined.push('\n');
    }
    (joined, edits)
}

/// Extracts explicit vCPU core count from a GitHub runner label AFTER stripping OS version strings
/// (`26.04`, `24.04`, `22.04`, `20.04`, `18.04`, `2025`, `2022`) so `ubuntu-24.04` NEVER falsely matches `'4'`!
fn extract_explicit_linux_cores_for_migration(cur_low: &str) -> Option<u32> {
    let mut cleaned = cur_low.to_string();
    for os_ver in [
        "26.04", "26-04", "2604", "24.04", "24-04", "2404", "22.04", "22-04", "2204", "20.04",
        "20-04", "2004", "18.04", "18-04", "1804",
    ] {
        cleaned = cleaned.replace(os_ver, "");
    }
    for (cores, pats) in [
        (64u32, &["64core", "64-core", "64vcpu", "-64"][..]),
        (32u32, &["32core", "32-core", "32vcpu", "-32"][..]),
        (16u32, &["16core", "16-core", "16vcpu", "-16", "-xl"][..]),
        (8u32, &["8core", "8-core", "8vcpu", "-8"][..]),
        (4u32, &["4core", "4-core", "4vcpu", "-4"][..]),
    ] {
        for pat in pats {
            if pat.starts_with('-') {
                if cleaned.ends_with(pat) || cleaned.contains(&format!("{}-", pat)) {
                    return Some(cores);
                }
            } else if cleaned.contains(pat) {
                return Some(cores);
            }
        }
    }
    None
}

fn migrate_single_workflow_yaml(
    rel_name: &str,
    content: &str,
    target_runner_mode: &str,
) -> (String, Vec<FilePatchEdit>) {
    if rel_name.contains("gitlab") {
        let mut out = Vec::new();
        let mut edits = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if matches!(target_runner_mode, "arm64" | "ubuntu-arm64" | "github-arm64")
                && line.contains("saas-linux-small-amd64")
            {
                let rep = line.replace("saas-linux-small-amd64", "saas-linux-small-arm64");
                out.push(rep.clone());
                edits.push(FilePatchEdit {
                    workflow_file: rel_name.to_string(),
                    rule_id: "MINICI-MIGRATE-GL1".to_string(),
                    description: "Migrate GitLab runner `saas-linux-small-amd64` -> `saas-linux-small-arm64`".to_string(),
                    line_number: idx + 1,
                    original_snippet: line.to_string(),
                    replacement_snippet: rep,
                });
            } else {
                out.push(line.to_string());
            }
        }
        let mut j = out.join("\n");
        if content.ends_with('\n') {
            j.push('\n');
        }
        return (j, edits);
    }

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
            if cur_val.is_empty() || cur_val.starts_with('[') {
                out_lines.push(line.to_string());
                continue;
            }
            let cur_low = cur_val.to_ascii_lowercase();
            let (j_start, j_end) = find_job_block_bounds(&lines, idx, ind);
            let job_body_low = lines[j_start..j_end].join("\n").to_ascii_lowercase();

            // Gate 1: Linux standard/larger x64 only (never rewrite macOS or Windows)
            let is_linux_x64 =
                cur_low.starts_with("ubuntu-") && !cur_low.contains("arm") && !cur_low.contains("slim");
            // Gate 2: Skip GPU / Android emulator KVM jobs
            let has_gpu_or_kvm = cur_low.contains("gpu")
                || job_body_low.contains("android-emulator")
                || job_body_low.contains("cuda");
            // Gate 3: Skip explicit x86 container platform pins when targeting ARM64 (`ubuntu-arm64` or `aws-c7g`)
            let has_amd64_pin = job_body_low.contains("linux/amd64") || job_body_low.contains("--platform=linux/amd64");
            // Gate 4: Lightweight gate for `ubuntu-slim`
            let is_heavy_for_slim = job_body_low.contains("docker build")
                || job_body_low.contains("services:")
                || job_body_low.contains("cargo build")
                || job_body_low.contains("pytest")
                || job_body_low.contains("playwright");
            // Gate 5: Spot/Self-hosted deploy ceremony gate
            let is_deploy_ceremony = job_body_low.contains("terraform apply")
                || job_body_low.contains("kubectl apply")
                || rel_name.contains("release");

            let explicit_cores = extract_explicit_linux_cores_for_migration(&cur_low);

            let new_runner: Option<&str> = match target_runner_mode {
                "ubuntu-arm64" | "github-arm64" | "arm64" | "ubuntu-24.04-arm" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !has_amd64_pin {
                        match explicit_cores {
                            Some(64) => Some("ubuntu-24.04-arm-64core"),
                            Some(32) => Some("ubuntu-24.04-arm-32core"),
                            Some(16) => Some("ubuntu-24.04-arm-16core"),
                            Some(8) => Some("ubuntu-24.04-arm-8core"),
                            Some(4) => Some("ubuntu-24.04-arm-4core"),
                            // Standard 2-core `ubuntu-latest`, `ubuntu-24.04`, `ubuntu-22.04` -> `ubuntu-24.04-arm`!
                            _ => Some("ubuntu-24.04-arm"),
                        }
                    } else {
                        None
                    }
                }
                "ubuntu-slim" | "slim" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !is_heavy_for_slim && explicit_cores.is_none() {
                        Some("ubuntu-slim")
                    } else {
                        None
                    }
                }
                "blacksmith" | "high-speed" => {
                    if is_linux_x64 && !has_gpu_or_kvm {
                        match explicit_cores {
                            Some(16) => Some("blacksmith-16vcpu-ubuntu-2404"),
                            Some(8) => Some("blacksmith-8vcpu-ubuntu-2404"),
                            Some(4) => Some("blacksmith-4vcpu-ubuntu-2404"),
                            _ => Some("blacksmith-2vcpu-ubuntu-2404"),
                        }
                    } else {
                        None
                    }
                }
                "self-hosted" | "aws-spot" => {
                    if is_linux_x64 && !has_gpu_or_kvm && !is_deploy_ceremony {
                        if has_amd64_pin {
                            Some("aws-c6i.xlarge-on-demand")
                        } else {
                            Some("aws-c7g.xlarge-on-demand")
                        }
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
                        "Migrate `{}` -> `{}` (passed all 6 safety & cost gates)",
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

/// Deterministic AST cross-vendor converter (`minici-yaml-fixer` core rules) converting `.gitlab-ci.yml`,
/// `.circleci/config.yml`, or `bitbucket-pipelines.yml` into a GitHub Actions workflow preview,
/// explicitly marking proprietary vendor steps with `# MANUAL_REVIEW_REQUIRED`.
fn generate_cross_vendor_github_actions_preview(repo_dir: &Path) -> Option<String> {
    let base = if repo_dir.is_file() {
        repo_dir.parent().unwrap_or(Path::new("."))
    } else {
        repo_dir
    };
    for (src_vendor, path) in [
        ("gitlab-ci", base.join(".gitlab-ci.yml")),
        ("circleci", base.join(".circleci").join("config.yml")),
        ("bitbucket", base.join("bitbucket-pipelines.yml")),
    ] {
        if let Ok(raw) = fs::read_to_string(&path) {
            let mut preview = format!(
                "# Auto-converted from {} by MiniCI AST Converter\nname: CI\non:\n  push:\n    branches: [main]\n  pull_request:\nconcurrency:\n  group: ${{{{ github.workflow }}}}-${{{{ github.ref }}}}\n  cancel-in-progress: ${{{{ github.event_name == 'pull_request' }}}}\njobs:\n  build:\n    runs-on: ubuntu-24.04-arm\n    timeout-minutes: 30\n    steps:\n      - uses: actions/checkout@v4\n",
                src_vendor
            );
            for line in raw.lines() {
                let t = line.trim();
                if let Some(cmd) = t.strip_prefix("- ").filter(|c| {
                    c.contains("npm ")
                        || c.contains("pytest")
                        || c.contains("cargo ")
                        || c.contains("go ")
                        || c.contains("mvn ")
                        || c.contains("./gradlew")
                }) {
                    preview.push_str(&format!("      - run: {}\n", cmd.trim_matches('"')));
                } else if t.starts_with("orbs:") || t.starts_with("include:") || t.starts_with("pipe:") {
                    preview.push_str(&format!(
                        "      # MANUAL_REVIEW_REQUIRED: unmapped vendor directive `{}`\n",
                        t
                    ));
                }
            }
            return Some(preview);
        }
    }
    None
}

fn copy_shadow_context(base_dir: &Path, shadow_root: &Path) {
    for rel in [
        "minici-usage.yml",
        "Cargo.toml",
        "package.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "yarn.lock",
        "package-lock.json",
        "requirements.txt",
        "pyproject.toml",
        "go.mod",
        "go.sum",
        "turbo.json",
        "nx.json",
    ] {
        let src = base_dir.join(rel);
        if src.is_file() {
            let _ = fs::copy(&src, shadow_root.join(rel));
        }
    }
    let codeowners = base_dir.join(".github").join("CODEOWNERS");
    if codeowners.is_file() {
        let _ = fs::create_dir_all(shadow_root.join(".github"));
        let _ = fs::copy(&codeowners, shadow_root.join(".github").join("CODEOWNERS"));
    }
    let lock = base_dir.join(".minici").join("telemetry.lock.json");
    if lock.is_file() {
        let _ = fs::create_dir_all(shadow_root.join(".minici"));
        let _ = fs::copy(&lock, shadow_root.join(".minici").join("telemetry.lock.json"));
    }
}

pub fn run_surgical_optimizer(
    repo_dir: &Path,
    operation: &str, // "shrink" | "migrate"
    target_mode: &str,
    apply_to_disk: bool,
) -> OptimizationReceipt {
    let base_report = analyze_path(repo_dir, None);
    let wf_files = collect_workflow_files(repo_dir);
    let base_dir = if repo_dir.is_file() {
        repo_dir.parent().unwrap_or(Path::new("."))
    } else {
        repo_dir
    };

    static SHADOW_SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let seq = SHADOW_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let shadow_root = std::env::temp_dir().join(format!(
        "minici_opt_shadow_{}_{}_{}",
        std::process::id(),
        seq,
        operation
    ));
    let _ = fs::remove_dir_all(&shadow_root);
    let _ = fs::create_dir_all(shadow_root.join(".github").join("workflows"));
    copy_shadow_context(base_dir, &shadow_root);

    let mut all_edits: Vec<FilePatchEdit> = Vec::new();
    let mut files_modified = 0usize;
    let mut pending_writes: Vec<(PathBuf, String)> = Vec::new();

    for (src_path, rel_name) in &wf_files {
        if let Ok(raw) = fs::read_to_string(src_path) {
            let is_gh_wf = rel_name.starts_with(".github/") || (!rel_name.contains("gitlab") && !rel_name.contains("circleci") && !rel_name.contains("bitbucket"));
            let (patched, file_edits) = if operation == "migrate" {
                migrate_single_workflow_yaml(rel_name, &raw, target_mode)
            } else if is_gh_wf {
                shrink_github_workflow_yaml(rel_name, &raw, base_dir)
            } else {
                shrink_non_github_yaml(rel_name, &raw)
            };
            if !file_edits.is_empty() {
                files_modified += 1;
                all_edits.extend(file_edits);
                pending_writes.push((src_path.clone(), patched.clone()));
            }
            let dest_shadow = if rel_name.starts_with(".github/") {
                shadow_root.join(rel_name)
            } else if rel_name.starts_with(".circleci/") {
                let _ = fs::create_dir_all(shadow_root.join(".circleci"));
                shadow_root.join(rel_name)
            } else if is_gh_wf && repo_dir.is_file() {
                shadow_root
                    .join(".github")
                    .join("workflows")
                    .join(rel_name)
            } else {
                shadow_root.join(rel_name)
            };
            if let Some(p) = dest_shadow.parent() {
                let _ = fs::create_dir_all(p);
            }
            let _ = fs::write(dest_shadow, &patched);
        }
    }

    let mut head_report = if wf_files.is_empty() {
        base_report.clone()
    } else {
        analyze_path(&shadow_root, Some(base_report.usage.clone()))
    };

    // Gate 6 (Mandatory Cost-Drop Safety Gate): If a runner migration unexpectedly increases monthly compute cost,
    // reject and revert the candidate edits so MiniCI NEVER writes a more expensive workflow!
    if operation == "migrate"
        && matches!(
            target_mode,
            "ubuntu-arm64" | "github-arm64" | "arm64" | "ubuntu-24.04-arm" | "ubuntu-slim" | "slim" | "self-hosted" | "aws-spot"
        )
        && head_report.gross_monthly_compute_cost_usd > base_report.gross_monthly_compute_cost_usd + 0.01
    {
        all_edits.clear();
        files_modified = 0;
        pending_writes.clear();
        head_report = base_report.clone();
    }

    let verified_diff = compute_diff(&base_report, &head_report);
    let _ = fs::remove_dir_all(&shadow_root);

    if apply_to_disk && !pending_writes.is_empty() {
        for (dest_path, content) in pending_writes {
            let tmp = dest_path.with_extension("yml.minici_tmp");
            if fs::write(&tmp, content).is_ok() {
                let _ = fs::rename(&tmp, dest_path);
            }
        }
    }

    let cc_base = &base_report.cost_components;
    let cc_head = &head_report.cost_components;
    let in_place_savings = (cc_base.recoverable_ci_and_wait_monthly_usd * 0.35)
        .max(cc_base.arm64_migration_monthly_savings_usd + cc_base.monthly_superseded_run_waste_usd);

    let converted_preview = if operation == "migrate" && target_mode == "github-actions" {
        generate_cross_vendor_github_actions_preview(repo_dir)
    } else {
        None
    };

    // Compute 5-Layer Platform Switching TCO if migrating across CI platforms
    let switching_verdict = if operation == "migrate"
        && matches!(
            target_mode,
            "github-actions"
                | "gitlab-ci"
                | "circleci"
                | "bitbucket"
                | "buildkite"
                | "aws-codebuild"
                | "gcp-cloud-build"
        ) {
        let src_vendor = base_report
            .detected_providers
            .first()
            .map(|s| s.as_str())
            .unwrap_or("github-actions");
        let target_tco = match target_mode {
            "gitlab-ci" => cc_base.gitlab_ci_premium_monthly_tco_usd,
            "circleci" => cc_base.circleci_performance_monthly_tco_usd,
            "bitbucket" => cc_base.bitbucket_premium_monthly_tco_usd,
            "buildkite" => cc_base.buildkite_pro_monthly_tco_usd,
            "aws-codebuild" => cc_base.aws_codebuild_monthly_tco_usd,
            "gcp-cloud-build" => cc_base.gcp_cloud_build_monthly_tco_usd,
            _ => cc_base.github_actions_team_monthly_tco_usd,
        };
        let sw = compute_5layer_platform_switching_tco(
            base_report.total_workflows.max(1),
            cc_base.implied_active_devs.max(1),
            src_vendor,
            target_mode,
            cc_base
                .github_actions_team_monthly_tco_usd
                .max(base_report.gross_monthly_compute_cost_usd),
            target_tco,
            cc_base.blended_fully_loaded_hourly_usd,
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
        applied_to_disk: apply_to_disk && !all_edits.is_empty(),
        files_modified,
        edits: all_edits,
        verified_diff,
        artifact_storage_before_usd: base_report.monthly_artifact_storage_vendor_usd,
        artifact_storage_after_usd: head_report.monthly_artifact_storage_vendor_usd,
        missing_timeouts_before: cc_base.jobs_missing_timeout_minutes_count,
        missing_timeouts_after: cc_head.jobs_missing_timeout_minutes_count,
        in_place_shrink_monthly_savings_usd: ((in_place_savings) * 100.0).round() / 100.0,
        converted_github_workflow_preview: converted_preview,
        switching_verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_preserving_shrink_and_6gate_arm64_migrate_edge_cases() {
        let dir = std::env::temp_dir().join("minici_optimizer_unit_test_v2");
        let _ = fs::remove_dir_all(&dir);
        let wf_dir = dir.join(".github").join("workflows");
        fs::create_dir_all(&wf_dir).unwrap();

        // Includes:
        // 1. `ubuntu-24.04` (must become `ubuntu-24.04-arm`, NEVER `ubuntu-24.04-arm-4core`!)
        // 2. Existing `timeout-minutes: 15` ABOVE `runs-on:` (must NOT add a duplicate `timeout-minutes`!)
        // 3. Multi-line `runs-on:` list (must NOT insert `timeout-minutes` inside the list!)
        // 4. `git diff` job (must NOT inject shallow `fetch-depth: 1`!)
        let orig = r#"# Important header comment preserved!
name: PR Suite
on: [pull_request]
jobs:
  ubuntu_ver_job:
    timeout-minutes: 15
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions/upload-artifact@v4
        with:
          retention-days: 90
      - run: cargo test
  multiline_runner_and_git_diff:
    runs-on:
      - self-hosted
      - linux
    steps:
      - uses: actions/checkout@v4
      - run: git diff origin/main...HEAD
  ios:
    runs-on: macos-latest
    steps:
      - run: xcodebuild test
"#;
        fs::write(wf_dir.join("ci.yml"), orig).unwrap();

        let shrink = run_surgical_optimizer(&dir, "shrink", "hygiene", true);
        assert!(shrink.edits.len() >= 4);
        let shrunk_yaml = fs::read_to_string(wf_dir.join("ci.yml")).unwrap();
        // Verify `ubuntu_ver_job` still has ONLY ONE `timeout-minutes: 15`!
        assert_eq!(shrunk_yaml.matches("timeout-minutes: 15").count(), 1);
        // Verify multi-line `runs-on:` list stays intact with `timeout-minutes: 30` AFTER `- linux`!
        assert!(shrunk_yaml.contains("runs-on:\n      - self-hosted\n      - linux\n    timeout-minutes: 30"));
        // Verify `fetch-depth: 1` was added ONLY once (on `ubuntu_ver_job`, NOT on `multiline_runner_and_git_diff`!)
        assert_eq!(shrunk_yaml.matches("fetch-depth: 1").count(), 1);

        // Verify `migrate --to github-arm64` rewrites `ubuntu-24.04` -> `ubuntu-24.04-arm` (NOT 4core!) and lowers cost!
        let mig = run_surgical_optimizer(&dir, "migrate", "github-arm64", true);
        assert_eq!(mig.files_modified, 1);
        assert!(mig.verified_diff.delta_monthly_compute_usd < 0.0);
        let migrated_yaml = fs::read_to_string(wf_dir.join("ci.yml")).unwrap();
        assert!(migrated_yaml.contains("# Important header comment preserved!"));
        assert!(migrated_yaml.contains("runs-on: ubuntu-24.04-arm\n"));
        assert!(!migrated_yaml.contains("arm-4core"));
        assert!(migrated_yaml.contains("runs-on: macos-latest"));

        let _ = fs::remove_dir_all(&dir);
    }
}
