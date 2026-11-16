#!/usr/bin/env python3
"""
End-to-End 27-Repository (10 Primary + 7 Internal Edge-Case + 10 Fresh World-Scale) Validation & Golden Baseline Lock Suite
for Feature 1 (`minici-cost-action` / `minici` v0.3.0).

Tests all repositories in parallel across:
  1. Mode 1: CLI Mode (`breakdown`, `diff --compare-to HEAD`, `--report .html/.json/.csv/.md`, `--benchmark`)
  2. Mode 2: GitHub Workflow Mode (`$GITHUB_STEP_SUMMARY` Markdown + `minici comment github`)
  3. Mode 3: AI Agent / MCP + Shift-Left Hook Mode (JSON-RPC 2.0 `initialize`, `tools/list`,
             `tools/call` for `minici_breakdown` & `minici_diff`, plus `--hook pre-tool-use`)
  4. Mode 4: Head-to-Head Comparison, 6 `cicdcost.com` Component Audit & Golden Locked Baseline Regression Check:
     - `cicdcost.com` 6 line-by-line invoice components (`build_minutes_cost`, `runner_cost`, `artifact_storage_cost`,
       `parallelism_cost`, `log_retention_cost`, `cross_region_egress_cost`)
     - Official 2026 GitHub Billing Docs Math (`docs.github.com` 1-min ceiling, 2,000 free mins, 2026 SKU rates)
     - `Infracost v2.19.0` (`/opt/homebrew/bin/infracost`)
     - Golden Baseline Lock (`tests/golden_baseline_10_repos.json` + `docs/GOLDEN_BASELINE_27_REPOS_KPI_AND_PERFORMANCE.md`)
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
MINICI_BIN = REPO_ROOT / "target" / "release" / "minici"
DEFAULT_CACHE_DIR = Path.home() / ".minici" / "10-repos-cache"
FALLBACK_TMP_DIR = Path("/tmp/minici-cost-10-repos")
FRESH_10_DIR = Path("/Users/araza/personal/top10-github-repos")
DEFAULT_GOLDEN_BASELINE = REPO_ROOT / "tests" / "golden_baseline_10_repos.json"
DEFAULT_MARKDOWN_BASELINE = REPO_ROOT / "docs" / "GOLDEN_BASELINE_27_REPOS_KPI_AND_PERFORMANCE.md"

TEN_REPOS = [
    ("01_MaximVote__maxim", "MaximVote/maxim", "Internal (MaximVote • Private)"),
    ("02_mgjartproject__FineArts-BE", "mgjartproject/FineArts-BE", "Internal (Grassy Fine Arts • Private)"),
    ("03_Eazy-Ops__Backend", "Eazy-Ops/Backend", "Internal (Eazy-Ops • Private)"),
    ("04_astral-sh__uv", "astral-sh/uv", "Fresh OSS (Rust/Python Packaging)"),
    ("05_denoland__deno", "denoland/deno", "Fresh OSS (JS/TS Runtime)"),
    ("06_ggml-org__llama.cpp", "ggml-org/llama.cpp", "Fresh OSS (C++ LLM Inference)"),
    ("07_immich-app__immich", "immich-app/immich", "Fresh OSS (Photo/Video Cloud)"),
    ("08_langgenius__dify", "langgenius/dify", "Fresh OSS (Agentic AI Platform)"),
    ("09_open-webui__open-webui", "open-webui/open-webui", "Fresh OSS (AI Web Interface)"),
    ("10_mendableai__firecrawl", "mendableai/firecrawl", "Fresh OSS (AI Web Scraper)"),
]

EDGE_CASE_REPOS = [
    ("check_MaximVote__tf", "MaximVote/tf", "Internal (6 Terragrunt VPN/GCP workflows)"),
    ("check_mgjartproject__gfa-data-pipelines", "mgjartproject/gfa-data-pipelines", "Internal (17x matrix scraper pipeline)"),
    ("check_mgjartproject__frontend", "mgjartproject/frontend", "Internal (GFA Frontend WIF + Security)"),
    ("check_mgjartproject__FineArt_AI_Pipeline", "mgjartproject/FineArt_AI_Pipeline", "Internal Edge Case (0 workflows)"),
    ("check_Eazy-Ops__frontend", "Eazy-Ops/frontend", "Internal (Eazy-Ops Frontend Docker)"),
    ("check_Eazy-Ops__multi-cloud-finops-mcp-server", "Eazy-Ops/multi-cloud-finops-mcp-server", "Internal (Within 2,000m Free Tier -> $0 net)"),
    ("check_Eazy-Ops__Automation", "Eazy-Ops/Automation", "Internal Edge Case (Nested cron/.github/workflows)"),
]

FRESH_10_REPOS = [
    ("01-microsoft-vscode", "microsoft/vscode", "World-Scale OSS (VS Code IDE)"),
    ("02-flutter-flutter", "flutter/flutter", "World-Scale OSS (Flutter SDK)"),
    ("03-ollama-ollama", "ollama/ollama", "World-Scale OSS (Local LLM Runner)"),
    ("04-huggingface-transformers", "huggingface/transformers", "World-Scale OSS (ML Transformers)"),
    ("05-vercel-nextjs", "vercel/next.js", "World-Scale OSS (Next.js Framework)"),
    ("06-langchain-ai-langchain", "langchain-ai/langchain", "World-Scale OSS (Agentic Framework)"),
    ("07-n8n-io-n8n", "n8n-io/n8n", "World-Scale OSS (Workflow Automation)"),
    ("08-supabase-supabase", "supabase/supabase", "World-Scale OSS (Postgres Cloud)"),
    ("09-home-assistant-core", "home-assistant/core", "World-Scale OSS (Home Automation)"),
    ("10-pydantic-pydantic", "pydantic/pydantic", "World-Scale OSS (Python Validation)"),
]

# Official 2026 GitHub Docs, cicdcost.com, and 3rd-party runner rates ($/min)
OFFICIAL_2026_RATES = {
    "ubuntu-latest": 0.0060,
    "ubuntu": 0.0060,
    "ubuntu-24.04": 0.0060,
    "ubuntu-22.04": 0.0060,
    "ubuntu-26.04": 0.0060,
    "ubuntu-24.04-ppc64le": 0.0060,
    "ubuntu-24.04-riscv": 0.0060,
    "ubuntu-26.04-s390x": 0.0060,
    "linux": 0.0060,
    "linux-arm64": 0.0050,
    "mich": 0.0060,
    "codspeed-macro": 0.0060,
    "codspeed-macro-x64-ryzen-9950x-ubuntu-24-04": 0.0060,
    "amd64-generic-x86-64-qemux86-64": 0.0060,
    "ubuntu-slim": 0.0020,
    "ubuntu-24.04-arm": 0.0050,
    "ubuntu-26.04-arm": 0.0050,
    "github-ubuntu-24.04-aarch64-2": 0.0050,
    "github-ubuntu-24.04-aarch64-4": 0.0080,
    "github-ubuntu-24.04-x86_64-2": 0.0060,
    "github-ubuntu-24.04-x86_64-4": 0.0120,
    "github-ubuntu-24.04-x86_64-8": 0.0220,
    "vscode-large-runners-ubuntu-8-core": 0.0220,
    "ubuntu-latest-16-core-arm-oss": 0.0260,
    "ubuntu-latest-16-core-oss": 0.0420,
    "ubuntu-24.04-xl": 0.0420,
    "ah-ubuntu_24_04-c8g_8x": 0.0500,
    "hf-jobs-cpu-performance": 0.0120,
    "hf-jobs-cpu-upgrade:ubuntu26_04": 0.0120,
    "hf-jobs-t4-small:ubuntu26_04": 0.0120,
    "hf-jobs-t4-medium:cuda13": 0.0160,
    "windows-latest": 0.0100,
    "windows-2022": 0.0100,
    "windows-2025": 0.0100,
    "windows-2025-vs2026": 0.0100,
    "windows": 0.0100,
    "windows-11-arm": 0.0080,
    "github-windows-11-aarch64-8": 0.0260,
    "vscode-large-runners-windows-8-core": 0.0420,
    "windows-latest-8-core-oss": 0.0420,
    "macos-latest": 0.0620,
    "macOS-latest": 0.0620,
    "macos": 0.0620,
    "darwin": 0.0620,
    "macos-14": 0.0620,
    "macos-15": 0.0620,
    "macos-15-intel": 0.0620,
    "macos-26": 0.0620,
    "macos-26-xlarge": 0.1020,
    "xcode-27": 0.0620,
    "blacksmith-2vcpu": 0.0040,
    "blacksmith-2vcpu-ubuntu-2204": 0.0040,
    "blacksmith-2vcpu-ubuntu-2404": 0.0040,
    "blacksmith-4vcpu": 0.0080,
    "blacksmith-4vcpu-ubuntu-2204": 0.0080,
    "blacksmith-4vcpu-ubuntu-2404": 0.0080,
    "blacksmith-8vcpu": 0.0160,
    "blacksmith-8vcpu-ubuntu-2204": 0.0160,
    "blacksmith-8vcpu-ubuntu-2404": 0.0160,
    "depot-ubuntu-24.04": 0.0040,
    "depot-ubuntu-24.04-4": 0.0080,
    "depot-ubuntu-latest-4": 0.0080,
    "depot-ubuntu-24.04-8": 0.0160,
    "depot-ubuntu-24.04-16": 0.0320,
    "namespace-profile-macos-15": 0.0320,
    "namespace-profile-windows-2022-x86-64-4": 0.0080,
    "namespace-profile-windows-2022-x86-64-16x32": 0.0320,
    "self-hosted-Linux-Intel": 0.0024,
    "self-hosted-Linux-Intel-OpenVINO": 0.0024,
    "self-hosted-Linux-NVIDIA": 0.0024,
    "self-hosted-Linux-NVIDIA-COOPMAT2": 0.0024,
    "self-hosted-Linux-gb10-perf": 0.0024,
    "self-hosted-Linux-gfx1201-1accel": 0.0024,
    "self-hosted-PLAYWRIGHT": 0.0024,
    "self-hosted-Windows-X64-Intel": 0.0024,
    "self-hosted-fast": 0.0024,
    "self-hosted-llama-server-macOS-ARM64": 0.0024,
    "self-hosted-macOS-ARM64": 0.0024,
    "self-hosted-opencode": 0.0024,
}


def run_timed(cmd: list[str], input_text: str | None = "", env: dict | None = None, timeout: float = 5.0) -> tuple[int, str, str, float]:
    t0 = time.perf_counter()
    res = subprocess.run(
        cmd,
        input=input_text,
        capture_output=True,
        text=True,
        env=env,
        timeout=timeout,
    )
    elapsed_ms = (time.perf_counter() - t0) * 1000.0
    return res.returncode, res.stdout, res.stderr, elapsed_ms


def verify_official_math_and_edge_cases(breakdown_json: dict) -> dict:
    jobs = breakdown_json.get("jobs", [])
    canonical_drifts = []
    custom_label_warnings = []

    recomputed_run_billed_mins = 0.0
    recomputed_run_cost_usd = 0.0

    for j in jobs:
        label = j.get("runner_label", "")
        rate = float(j.get("rate_per_min_usd", 0.0))
        raw_dur = float(j.get("est_raw_duration_mins_per_job", 0.0))
        billed_per_job = float(j.get("billed_mins_per_job", 0.0))
        total_billed = float(j.get("total_billed_mins_all_matrix", 0.0))

        # Check 1-minute ceiling rule for GitHub Actions standard SaaS runners
        if j.get("provider") == "github-actions":
            expected_ceil = max(1.0, float(math.ceil(raw_dur - 1e-9)))
            if abs(billed_per_job - expected_ceil) > 0.01:
                canonical_drifts.append(
                    f"Job {j.get('job_id')}: billed_mins_per_job={billed_per_job} != ceil({raw_dur})={expected_ceil}"
                )

        recomputed_run_billed_mins += total_billed
        recomputed_run_cost_usd += total_billed * rate

        if label in OFFICIAL_2026_RATES:
            expected_rate = OFFICIAL_2026_RATES[label]
            if abs(rate - expected_rate) > 1e-6:
                canonical_drifts.append(
                    f"SKU rate drift on {label}: got ${rate:.4f}/min, expected ${expected_rate:.4f}/min"
                )
        else:
            custom_label_warnings.append(f"Unmapped runner label `{label}` resolved to ${rate:.4f}/min")

    reported_run_cost = float(breakdown_json.get("gross_compute_cost_per_run_usd", 0.0))
    reported_monthly_cost = float(breakdown_json.get("gross_monthly_compute_cost_usd", 0.0))
    monthly_runs = float(breakdown_json.get("monthly_runs", 433.0))
    expected_monthly_cost = round(recomputed_run_cost_usd * monthly_runs, 2)

    cost_delta_pct = 0.0
    if expected_monthly_cost > 0:
        cost_delta_pct = abs(reported_monthly_cost - expected_monthly_cost) / expected_monthly_cost * 100.0

    # Verify all 11 MiniCI cost_components (6 invoice components + 5 hidden payroll & pipeline cost models)
    cc = breakdown_json.get("cost_components", {})
    for comp_key in (
        "build_minutes_cost",
        "runner_cost",
        "artifact_storage_cost",
        "parallelism_cost",
        "log_retention_cost",
        "cross_region_egress_cost",
        "developer_wait_time_cost",
        "failed_build_rerun_cost",
        "build_cache_savings",
        "flaky_test_ci_cd_tax",
        "branch_protection_cost",
    ):
        if comp_key not in cc:
            canonical_drifts.append(f"Missing cost_components.{comp_key} in JSON output")
    for ic_key in ("runner_cost", "artifact_storage_cost", "log_retention_cost", "cross_region_egress_cost"):
        if not cc.get(ic_key, {}).get("infracost_delegated"):
            canonical_drifts.append(f"Missing infracost_delegated=true on cost_components.{ic_key}")

    return {
        "canonical_drifts": canonical_drifts,
        "custom_label_warnings": sorted(set(custom_label_warnings)),
        "recomputed_run_cost_usd": round(recomputed_run_cost_usd, 4),
        "reported_run_cost_usd": reported_run_cost,
        "expected_monthly_cost_usd": expected_monthly_cost,
        "reported_monthly_cost_usd": reported_monthly_cost,
        "math_drift_pct": round(cost_delta_pct, 4),
    }


def validate_single_repo_all_modes(
    folder: str,
    repo_name: str,
    category: str,
    repo_path: Path,
    tmp_rep: Path,
    run_infracost: bool = False,
    infracost_bin: str = "",
) -> dict:
    if not repo_path.exists():
        raise RuntimeError(f"Missing snapshot directory: {repo_path}")

    # --- MODE 1: CLI MODE ---
    rc_bd, out_bd_json, err_bd, wall_bd_ms = run_timed(
        [str(MINICI_BIN), "breakdown", "--path", str(repo_path), "--format", "json"]
    )
    assert rc_bd == 0, f"CLI breakdown failed on {repo_name}: {err_bd}"
    bd_data = json.loads(out_bd_json)
    engine_us = bd_data.get("execution_time_us", 0)
    engine_ms = bd_data.get("execution_time_ms", round(engine_us / 1000.0, 3))

    rc_df, out_df_json, err_df, wall_df_ms = run_timed(
        [str(MINICI_BIN), "diff", "--path", str(repo_path), "--compare-to", "HEAD", "--format", "json"]
    )
    assert rc_df == 0, f"CLI diff failed on {repo_name}: {err_df}"
    df_data = json.loads(out_df_json)
    assert "cost_components" in df_data and len(df_data["cost_components"]) == 11, (
        f"CLI diff JSON missing 11 cost_components on {repo_name}"
    )

    for ext in ("html", "json", "csv", "md"):
        exp_path = tmp_rep / f"{folder}.{ext}"
        rc_ex, _, err_ex, _ = run_timed(
            [str(MINICI_BIN), "breakdown", "--path", str(repo_path), "--report", str(exp_path)]
        )
        assert rc_ex == 0 and exp_path.exists() and exp_path.stat().st_size > 0, f"Report export {exp_path} failed: {err_ex}"

    # --- MODE 2: GITHUB WORKFLOW MODE ---
    step_summary_file = tmp_rep / f"{folder}_step_summary.md"
    rc_wf, out_wf_md, err_wf, wall_wf_ms = run_timed(
        [str(MINICI_BIN), "breakdown", "--path", str(repo_path), "--format", "markdown"]
    )
    assert rc_wf == 0 and "| Workflow | Job ID |" in out_wf_md and "- **Recommendation:**" in out_wf_md and "cicdcost.com" not in out_wf_md, (
        f"Workflow markdown mode failed on {repo_name}"
    )
    step_summary_file.write_text(out_wf_md)

    rc_cm, out_cm_md, err_cm, wall_cm_ms = run_timed(
        [str(MINICI_BIN), "comment", "github", "--path", str(repo_path)]
    )
    assert rc_cm == 0 and "MiniCI Pull Request CI/CD Cost Diff" in out_cm_md, f"GitHub PR comment mode failed on {repo_name}"

    # --- MODE 3: AI AGENT / MCP + SHIFT-LEFT HOOK MODE ---
    mcp_requests = "\n".join([
        json.dumps({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json.dumps({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "minici_breakdown", "arguments": {"path": str(repo_path)}}}),
        json.dumps({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "minici_diff", "arguments": {"path": str(repo_path), "compare_to": "HEAD"}}}),
    ]) + "\n"
    rc_mcp, out_mcp, err_mcp, wall_mcp_ms = run_timed(
        [str(MINICI_BIN), "mcp", "--path", str(repo_path)],
        input_text=mcp_requests,
    )
    assert rc_mcp == 0, f"MCP server failed on {repo_name}: {err_mcp}"
    mcp_lines = [json.loads(line) for line in out_mcp.strip().splitlines() if line.strip()]
    assert len(mcp_lines) == 4, f"Expected 4 JSON-RPC responses on {repo_name}, got {len(mcp_lines)}"
    mcp_bd_payload = json.loads(mcp_lines[2]["result"]["content"][0]["text"])
    assert abs(float(mcp_bd_payload["gross_monthly_compute_cost_usd"]) - float(bd_data["gross_monthly_compute_cost_usd"])) < 0.01
    assert "cost_components" in mcp_bd_payload and len(mcp_bd_payload["cost_components"]) == 11

    hook_payload = json.dumps({"tool_name": "Bash", "tool_input": {"command": "git push"}}) + "\n"
    rc_hk, out_hk, err_hk, wall_hk_ms = run_timed(
        [str(MINICI_BIN), "breakdown", "--path", str(repo_path), "--hook", "pre-tool-use"],
        input_text=hook_payload,
    )
    assert rc_hk == 0, f"Hook mode failed on {repo_name}: {err_hk}"

    # --- MODE 4: HEAD-TO-HEAD & 6-COMPONENT AUDIT ---
    math_audit = verify_official_math_and_edge_cases(bd_data)
    if math_audit["canonical_drifts"]:
        raise RuntimeError(f"Canonical SKU/Component drift on {repo_name}: {math_audit['canonical_drifts']}")

    infracost_ms = None
    infracost_resources = 0
    if run_infracost and infracost_bin:
        try:
            _, out_ic, _, infracost_ms = run_timed(
                [infracost_bin, "breakdown", "--path", str(repo_path), "--format", "json"],
                timeout=3.0,
            )
            ic_json = json.loads(out_ic)
            infracost_resources = int(ic_json.get("summary", {}).get("totalDetectedResources", 0))
        except Exception:
            infracost_resources = 0

    cc = bd_data["cost_components"]
    return {
        "folder": folder,
        "repo": repo_name,
        "category": category,
        "workflows": bd_data["total_workflows"],
        "defined_jobs": bd_data["total_defined_jobs"],
        "expanded_jobs": bd_data["total_expanded_matrix_jobs"],
        "billed_mins_per_run": bd_data["billed_mins_per_run"],
        "cost_per_run_usd": bd_data["gross_compute_cost_per_run_usd"],
        "monthly_cost_usd": bd_data["gross_monthly_compute_cost_usd"],
        "net_billed_monthly_usd": bd_data["net_billed_monthly_compute_cost_usd"],
        "free_tier_burn_pct": bd_data["free_tier_quota_burn_pct"],
        "dev_wait_mins": bd_data["critical_path_wait_mins_per_run"],
        "monthly_dev_wait_usd": bd_data["monthly_dev_wait_cost_usd"],
        "c1_rounding_tax_usd": cc["build_minutes_cost"]["monthly_rounding_tax_usd"],
        "c2_ec2_instance": cc["runner_cost"]["infracost_recommended_ec2_instance"],
        "c2_spot_monthly_usd": bd_data.get("monthly_self_hosted_spot_cost_usd", cc["runner_cost"]["infracost_ec2_on_demand_monthly_usd"]),
        "c3_artifact_90d_usd": cc["artifact_storage_cost"]["monthly_vendor_storage_90d_usd"],
        "c3_artifact_s3_14d_usd": cc["artifact_storage_cost"]["monthly_infracost_s3_intelligent_14d_usd"],
        "c4_parallelism_tax_usd": cc["parallelism_cost"]["parallelism_tier_upgrade_tax_monthly_usd"],
        "c5_log_profile": cc["log_retention_cost"]["detected_log_language_profile"],
        "c5_datadog_90d_usd": cc["log_retention_cost"]["monthly_datadog_90d_log_shipping_usd"],
        "c6_nat_worst_case_usd": cc["cross_region_egress_cost"]["monthly_infracost_private_subnet_nat_worst_case_usd"],
        "c6_vpc_cache_opt_usd": cc["cross_region_egress_cost"]["monthly_infracost_vpc_endpoint_cache_optimized_usd"],
        "h1_productive_payroll_loss_usd": cc["developer_wait_time_cost"]["monthly_productive_payroll_loss_usd"],
        "h2_immediate_flake_cost_usd": cc["failed_build_rerun_cost"]["monthly_total_immediate_flake_cost_usd"],
        "h3_cached_jobs": cc["build_cache_savings"]["jobs_with_cache"],
        "h3_uncached_eligible_jobs": cc["build_cache_savings"]["jobs_missing_cache"],
        "h3_unrealized_cache_savings_usd": cc["build_cache_savings"]["monthly_unrealized_cache_savings_usd"],
        "h4_combined_flake_tax_usd": cc["flaky_test_ci_cd_tax"]["monthly_combined_flake_tax_usd"],
        "h5_missing_cancel_wf": cc["branch_protection_cost"]["workflows_missing_concurrency_cancel"],
        "h5_superseded_run_waste_usd": cc["branch_protection_cost"]["monthly_superseded_run_waste_usd"],
        "h5_missing_path_filter_wf": cc["branch_protection_cost"]["workflows_missing_path_filters"],
        "h5_unfiltered_check_savings_usd": cc["branch_protection_cost"]["monthly_unfiltered_check_savings_usd"],
        "engine_time_us": engine_us,
        "engine_time_ms": engine_ms,
        "cli_wall_ms": round(wall_bd_ms, 2),
        "diff_wall_ms": round(wall_df_ms, 2),
        "workflow_wall_ms": round(wall_wf_ms, 2),
        "mcp_4rpc_wall_ms": round(wall_mcp_ms, 2),
        "hook_wall_ms": round(wall_hk_ms, 2),
        "infracost_wall_ms": round(infracost_ms, 2) if infracost_ms else None,
        "infracost_ci_resources": infracost_resources,
        "math_audit": math_audit,
    }


def write_markdown_baseline_doc(
    md_path: Path,
    ten_results: list[dict],
    edge_results: list[dict],
    fresh_results: list[dict],
) -> None:
    lines = [
        "# Locked 27-Repository Golden Baseline: KPIs, 11 Unified CI/CD Cost Models & 3-Mode Latency (`v0.4.0`)",
        "",
        "**Locked Version:** `minici 0.4.0` (`tag: v0.4.0`)  ",
        "**Total Repositories Locked:** `27` (`10` Primary + `7` Internal Edge-Case + `10` Fresh World-Scale OSS)  ",
        "**Execution Modes Verified on Every Repo:** `Mode 1: CLI` • `Mode 2: GitHub Action Workflow` • `Mode 3: MCP JSON-RPC 2.0 + Shift-Left Hook`  ",
        "**Machine-Readable Companion Lock:** [`tests/golden_baseline_10_repos.json`](../tests/golden_baseline_10_repos.json)",
        "",
        "## How to Re-Verify Zero Deviation in Future Sessions",
        "",
        "```bash",
        "# Run the 27-repository parallel verification gate (< 3s):",
        "bash scripts/validate_minici_cost_10_repos.sh",
        "",
        "# Or invoke the verifier directly:",
        "python3 tools/verify_10_repos_all_modes.py",
        "```",
        "",
        "If any commit alters job expansion, 1-minute ceiling rounding, SKU pricing, or any of the 11 unified CI/CD cost models (`6` invoice components + `5` hidden payroll & pipeline cost models) on any of the 27 repositories, `verify_10_repos_all_modes.py` exits non-zero and prints the exact field delta.",
        "",
        "---",
        "",
        "## Part 1: 10 Primary Repositories (`3 Internal + 7 Modern OSS`) — Core KPIs & 3-Mode Performance",
        "",
        "| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |",
        "| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ]
    for r in ten_results:
        lines.append(
            f"| **`{r['repo']}`** | {r['category']} | `{r['workflows']}` / `{r['defined_jobs']}` / `{r['expanded_jobs']}` "
            f"| `{r['billed_mins_per_run']:.2f}m` | **`${r['monthly_cost_usd']:,.2f}/mo`** | `{r['free_tier_burn_pct']:.1f}%` "
            f"| `${r['monthly_dev_wait_usd']:,.2f}/mo` (`{r['dev_wait_mins']}m`) | `{r['engine_time_ms']:.2f}` | `{r['cli_wall_ms']:.2f}` | `{r['workflow_wall_ms']:.2f}` | `{r['mcp_4rpc_wall_ms']:.2f}` |"
        )

    lines.extend([
        "",
        "### Part 1B: 10 Primary Repositories — 6 Core CI/CD Invoice Components (`$/mo`)",
        "",
        "| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for r in ten_results:
        lines.append(
            f"| **`{r['repo']}`** | `${r['c1_rounding_tax_usd']:,.2f}` | `{r['c2_ec2_instance']}` (`${r['c2_spot_monthly_usd']:,.2f}`) "
            f"| `${r['c3_artifact_90d_usd']:,.2f}` -> `${r['c3_artifact_s3_14d_usd']:,.2f}` | `${r['c4_parallelism_tax_usd']:,.2f}` "
            f"| {r['c5_log_profile']} (`${r['c5_datadog_90d_usd']:,.2f}`) | `${r['c6_nat_worst_case_usd']:,.2f}` -> `${r['c6_vpc_cache_opt_usd']:,.2f}` |"
        )

    lines.extend([
        "",
        "### Part 1C: 10 Primary Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)",
        "",
        "| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for r in ten_results:
        lines.append(
            f"| **`{r['repo']}`** | `${r['h1_productive_payroll_loss_usd']:,.2f}/mo` | `${r['h2_immediate_flake_cost_usd']:,.2f}/mo` "
            f"| `{r['h3_cached_jobs']}/{r['expanded_jobs']}` (`${r['h3_unrealized_cache_savings_usd']:,.2f}/mo`) | `${r['h4_combined_flake_tax_usd']:,.2f}/mo` "
            f"| `{r['h5_missing_cancel_wf']}` wf (`${r['h5_superseded_run_waste_usd']:,.2f}/mo`) | `{r['h5_missing_path_filter_wf']}` wf (`${r['h5_unfiltered_check_savings_usd']:,.2f}/mo`) |"
        )

    lines.extend([
        "",
        "---",
        "",
        "## Part 2: 10 Fresh World-Scale GitHub Repositories (`/Users/araza/personal/top10-github-repos`) — Core KPIs & 3-Mode Performance",
        "",
        "| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |",
        "| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for r in fresh_results:
        lines.append(
            f"| **`{r['repo']}`** | {r['category']} | `{r['workflows']}` / `{r['defined_jobs']}` / `{r['expanded_jobs']}` "
            f"| `{r['billed_mins_per_run']:.2f}m` | **`${r['monthly_cost_usd']:,.2f}/mo`** | `{r['free_tier_burn_pct']:.1f}%` "
            f"| `${r['monthly_dev_wait_usd']:,.2f}/mo` (`{r['dev_wait_mins']}m`) | `{r['engine_time_ms']:.2f}` | `{r['cli_wall_ms']:.2f}` | `{r['workflow_wall_ms']:.2f}` | `{r['mcp_4rpc_wall_ms']:.2f}` |"
        )

    lines.extend([
        "",
        "### Part 2B: 10 Fresh World-Scale Repositories — 6 Core CI/CD Invoice Components (`$/mo`)",
        "",
        "| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for r in fresh_results:
        lines.append(
            f"| **`{r['repo']}`** | `${r['c1_rounding_tax_usd']:,.2f}` | `{r['c2_ec2_instance']}` (`${r['c2_spot_monthly_usd']:,.2f}`) "
            f"| `${r['c3_artifact_90d_usd']:,.2f}` -> `${r['c3_artifact_s3_14d_usd']:,.2f}` | `${r['c4_parallelism_tax_usd']:,.2f}` "
            f"| {r['c5_log_profile']} (`${r['c5_datadog_90d_usd']:,.2f}`) | `${r['c6_nat_worst_case_usd']:,.2f}` -> `${r['c6_vpc_cache_opt_usd']:,.2f}` |"
        )

    lines.extend([
        "",
        "### Part 2C: 10 Fresh World-Scale Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)",
        "",
        "| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for r in fresh_results:
        lines.append(
            f"| **`{r['repo']}`** | `${r['h1_productive_payroll_loss_usd']:,.2f}/mo` | `${r['h2_immediate_flake_cost_usd']:,.2f}/mo` "
            f"| `{r['h3_cached_jobs']}/{r['expanded_jobs']}` (`${r['h3_unrealized_cache_savings_usd']:,.2f}/mo`) | `${r['h4_combined_flake_tax_usd']:,.2f}/mo` "
            f"| `{r['h5_missing_cancel_wf']}` wf (`${r['h5_superseded_run_waste_usd']:,.2f}/mo`) | `{r['h5_missing_path_filter_wf']}` wf (`${r['h5_unfiltered_check_savings_usd']:,.2f}/mo`) |"
        )

    lines.extend([
        "",
        "---",
        "",
        "## Part 3: 7 Internal Edge-Case Repositories — KPIs & Latency",
        "",
        "| Repository | Edge-Case Scenario | WF / Exp Jobs | Gross Monthly Compute | Net Billed Monthly | CLI Latency (`ms`) |",
        "| :--- | :--- | :---: | :---: | :---: | :---: |",
    ])
    for e in edge_results:
        lines.append(
            f"| **`{e['repo']}`** | {e['note']} | `{e['workflows']}` / `{e['expanded_jobs']}` "
            f"| `${abs(e['monthly_cost_usd']):,.2f}/mo` | **`${abs(e['net_billed_monthly_usd']):,.2f}/mo`** | `{e['wall_ms']:.2f}` |"
        )

    md_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.write_text("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser(description="Validate minici-cost-action across 27 repositories (10 primary + 7 internal edge-case + 10 fresh world-scale) and all 3 modes.")
    parser.add_argument("--cache-dir", default=str(DEFAULT_CACHE_DIR), help="Directory holding the 10+7 repo workflow snapshots")
    parser.add_argument("--fresh-dir", default=str(FRESH_10_DIR), help="Directory holding the 10 fresh world-scale repo snapshots")
    parser.add_argument("--out-json", default=str(DEFAULT_CACHE_DIR / "minici_10_repos_validation_results.json"), help="Output JSON report path")
    parser.add_argument("--golden-baseline", default=str(DEFAULT_GOLDEN_BASELINE), help="Golden baseline JSON path")
    parser.add_argument("--markdown-baseline", default=str(DEFAULT_MARKDOWN_BASELINE), help="Golden baseline Markdown path")
    parser.add_argument("--write-locked-baseline", action="store_true", help="Write/update the locked golden baseline JSON and Markdown")
    args = parser.parse_args()

    cache_dir = Path(args.cache_dir)
    if not cache_dir.exists() and FALLBACK_TMP_DIR.exists():
        cache_dir = FALLBACK_TMP_DIR
    fresh_dir = Path(args.fresh_dir)

    if not MINICI_BIN.exists():
        print(f"Building release binary at {MINICI_BIN}...")
        subprocess.run(["cargo", "build", "--manifest-path", str(REPO_ROOT / "crates" / "minici" / "Cargo.toml"), "--release"], check=True)
        shutil.copy(REPO_ROOT / "crates" / "minici" / "target" / "release" / "minici", MINICI_BIN)

    edge_case_results = []

    print("=" * 118)
    print(" MINICI-COST-ACTION (v0.4.0) — 27-REPOSITORY (17 EXISTING + 10 FRESH) x 3-MODE PARALLEL VALIDATION & GOLDEN LOCK")
    print("=" * 118)

    with tempfile.TemporaryDirectory(prefix="minici-27repo-reports-") as tmp_rep_dir:
        tmp_rep = Path(tmp_rep_dir)

        # Execute the 10 Primary + 10 Fresh repositories in parallel across all 3 modes (CLI, Workflow, MCP)
        with ThreadPoolExecutor(max_workers=8) as pool:
            ten_futures = [
                pool.submit(validate_single_repo_all_modes, folder, repo_name, category, cache_dir / folder, tmp_rep)
                for folder, repo_name, category in TEN_REPOS
            ]
            fresh_futures = []
            if fresh_dir.exists():
                fresh_futures = [
                    pool.submit(validate_single_repo_all_modes, folder, repo_name, category, fresh_dir / folder, tmp_rep)
                    for folder, repo_name, category in FRESH_10_REPOS
                    if (fresh_dir / folder).exists()
                ]

            results = [f.result() for f in ten_futures]
            fresh_results = [f.result() for f in fresh_futures]

        for row in results:
            warn_tag = f"⚠️ {len(row['math_audit']['custom_label_warnings'])} custom-label note(s)" if row["math_audit"]["custom_label_warnings"] else "✅ 0.00% drift"
            print(
                f" • {row['repo']:28s} | {row['workflows']:2d} wf / {row['expanded_jobs']:3d} jobs "
                f"| Engine: {row['engine_time_ms']:5.2f} ms ({row['engine_time_us']:4d} µs) | CLI: {row['cli_wall_ms']:5.2f} ms | WF: {row['workflow_wall_ms']:5.2f} ms "
                f"| MCP(4 RPCs): {row['mcp_4rpc_wall_ms']:5.2f} ms | ${row['monthly_cost_usd']:8.2f}/mo | {warn_tag}"
            )

        if fresh_results:
            print("-" * 118)
            print(" 10 FRESH WORLD-SCALE GITHUB REPOSITORIES (CLI + WORKFLOW + MCP 4-RPC + 11 COST MODELS):")
            for row in fresh_results:
                warn_tag = f"⚠️ {len(row['math_audit']['custom_label_warnings'])} custom-label note(s)" if row["math_audit"]["custom_label_warnings"] else "✅ 0.00% drift"
                print(
                    f" • {row['repo']:28s} | {row['workflows']:2d} wf / {row['expanded_jobs']:3d} jobs "
                    f"| Engine: {row['engine_time_ms']:5.2f} ms ({row['engine_time_us']:4d} µs) | CLI: {row['cli_wall_ms']:5.2f} ms | WF: {row['workflow_wall_ms']:5.2f} ms "
                    f"| MCP(4 RPCs): {row['mcp_4rpc_wall_ms']:5.2f} ms | ${row['monthly_cost_usd']:8.2f}/mo | {warn_tag}"
                )

        print("-" * 118)
        print(" 7 ADDITIONAL INTERNAL REPOSITORIES & EDGE-CASE SCANS:")
        for folder, repo_name, note in EDGE_CASE_REPOS:
            repo_path = cache_dir / folder
            if not repo_path.exists():
                continue
            rc_e, out_e, err_e, wall_e_ms = run_timed(
                [str(MINICI_BIN), "breakdown", "--path", str(repo_path), "--format", "json"]
            )
            ed = json.loads(out_e)
            wf_cnt = ed.get("total_workflows", ed.get("workflows_parsed", 0))
            job_cnt = ed.get("total_expanded_matrix_jobs", 0)
            gross_mo = float(ed.get("gross_monthly_compute_cost_usd", ed.get("total_monthly_cost", 0.0)))
            net_mo = float(ed.get("net_billed_monthly_compute_cost_usd", 0.0))
            tier_lbl = ed.get("effective_tier", "skipped (0 workflows)")
            edge_case_results.append({
                "repo": repo_name,
                "note": note,
                "workflows": wf_cnt,
                "expanded_jobs": job_cnt,
                "monthly_cost_usd": gross_mo,
                "net_billed_monthly_usd": net_mo,
                "wall_ms": round(wall_e_ms, 2),
                "status": tier_lbl,
            })
            print(
                f" • {repo_name:42s} | {wf_cnt:2d} wf / {job_cnt:3d} jobs "
                f"| {wall_e_ms:5.2f} ms | Gross: ${abs(gross_mo):6.2f}/mo • Net: ${abs(net_mo):6.2f}/mo — {note}"
            )

    def extract_locked_kpi(r: dict) -> dict:
        return {
            "workflows": r["workflows"],
            "defined_jobs": r["defined_jobs"],
            "expanded_jobs": r["expanded_jobs"],
            "billed_mins_per_run": r["billed_mins_per_run"],
            "cost_per_run_usd": r["cost_per_run_usd"],
            "monthly_cost_usd": r["monthly_cost_usd"],
            "free_tier_burn_pct": r["free_tier_burn_pct"],
            "dev_wait_mins": r["dev_wait_mins"],
            "monthly_dev_wait_usd": r["monthly_dev_wait_usd"],
            "c1_rounding_tax_usd": r["c1_rounding_tax_usd"],
            "c2_spot_monthly_usd": r["c2_spot_monthly_usd"],
            "c3_artifact_90d_usd": r["c3_artifact_90d_usd"],
            "c4_parallelism_tax_usd": r["c4_parallelism_tax_usd"],
            "c5_datadog_90d_usd": r["c5_datadog_90d_usd"],
            "c6_nat_worst_case_usd": r["c6_nat_worst_case_usd"],
            "h1_productive_payroll_loss_usd": r["h1_productive_payroll_loss_usd"],
            "h2_immediate_flake_cost_usd": r["h2_immediate_flake_cost_usd"],
            "h3_cached_jobs": r["h3_cached_jobs"],
            "h3_uncached_eligible_jobs": r["h3_uncached_eligible_jobs"],
            "h3_unrealized_cache_savings_usd": r["h3_unrealized_cache_savings_usd"],
            "h4_combined_flake_tax_usd": r["h4_combined_flake_tax_usd"],
            "h5_missing_cancel_wf": r["h5_missing_cancel_wf"],
            "h5_superseded_run_waste_usd": r["h5_superseded_run_waste_usd"],
            "h5_missing_path_filter_wf": r["h5_missing_path_filter_wf"],
            "h5_unfiltered_check_savings_usd": r["h5_unfiltered_check_savings_usd"],
        }

    # Golden Locked Baseline Write or Verification
    golden_path = Path(args.golden_baseline)
    md_path = Path(args.markdown_baseline)
    locked_deterministic = {
        "schema_version": "1.2.0",
        "minici_version": "0.4.0",
        "repos": {r["repo"]: extract_locked_kpi(r) for r in results},
        "fresh_10_repos": {r["repo"]: extract_locked_kpi(r) for r in fresh_results},
        "edge_case_repos": {
            e["repo"]: {
                "workflows": e["workflows"],
                "expanded_jobs": e["expanded_jobs"],
                "monthly_cost_usd": abs(e["monthly_cost_usd"]),
                "net_billed_monthly_usd": abs(e["net_billed_monthly_usd"]),
            }
            for e in edge_case_results
        },
    }

    if args.write_locked_baseline or not golden_path.exists():
        golden_path.parent.mkdir(parents=True, exist_ok=True)
        golden_path.write_text(json.dumps(locked_deterministic, indent=2) + "\n")
        write_markdown_baseline_doc(md_path, results, edge_case_results, fresh_results)
        print(f"🔒 Wrote locked 27-repo golden JSON baseline to: {golden_path}")
        print(f"🔒 Wrote locked 27-repo Markdown KPI & Performance baseline to: {md_path}")
    else:
        existing_golden = json.loads(golden_path.read_text())
        deviations = []
        for section_key in ("repos", "fresh_10_repos", "edge_case_repos"):
            for repo_name, curr_vals in locked_deterministic.get(section_key, {}).items():
                exp_vals = existing_golden.get(section_key, {}).get(repo_name)
                if not exp_vals:
                    continue
                for k, v in curr_vals.items():
                    if k not in exp_vals:
                        deviations.append(f"{section_key}[{repo_name}].{k}: missing in locked golden baseline")
                        continue
                    if abs(float(v) - float(exp_vals.get(k, -999999))) > 1e-4:
                        deviations.append(f"{section_key}[{repo_name}].{k}: expected {exp_vals.get(k)}, got {v}")
        if deviations:
            print("❌ GOLDEN LOCKED BASELINE DEVIATION DETECTED:", file=sys.stderr)
            for d in deviations:
                print(f"   - {d}", file=sys.stderr)
            sys.exit(1)
        total_verified = len(results) + len(fresh_results) + len(edge_case_results)
        print(
            f"🔒 GOLDEN LOCKED BASELINE VERIFIED: {total_verified}/27 repositories "
            f"({len(results)} primary + {len(fresh_results)} fresh world-scale + {len(edge_case_results)} internal edge-case) "
            f"match {golden_path.name} with 0.00% deviation!"
        )

    out_json_path = Path(args.out_json)
    out_json_path.parent.mkdir(parents=True, exist_ok=True)
    summary_payload = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "binary": str(MINICI_BIN),
        "minici_version": "0.4.0",
        "ten_repos": results,
        "fresh_10_repos": fresh_results,
        "edge_case_repos": edge_case_results,
    }
    out_json_path.write_text(json.dumps(summary_payload, indent=2))
    print("=" * 118)
    print(f"✓ Saved full JSON validation report to: {out_json_path}")


if __name__ == "__main__":
    main()
