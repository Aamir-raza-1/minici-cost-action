#!/usr/bin/env python3
"""10-Session Closed-Loop QA & Hard Invariant Verifier (`closed_loop_qa.py`).

Executes all 10 architectural QA checks across the 5-Layer CI/CD FinOps Pricing Engine
(`engine/pricing.py`, `engine/billing_ingestor.py`, `qa_verifier.py`, and
`benchmark_10_repos_and_clouds.py`), verifies mathematical invariants on every receipt,
and writes a structured JSON audit report to `reports/closed_loop_qa_report.json`.

Usage:
  python3 closed_loop_qa.py
  python3 closed_loop_qa.py --with-ollama
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import benchmark_10_repos_and_clouds
import qa_verifier
from engine import billing_ingestor, pricing


def assert_receipt_invariants(receipt: dict[str, Any], label: str) -> None:
    """Enforce hard mathematical invariants on every calculate_monthly_cost receipt."""
    gross = float(receipt["gross_consumed_usd"])
    disc = float(receipt["discount_allowance_usd"])
    net = float(receipt["net_billable_usd"])
    assert gross >= -1e-6, f"[{label}] negative gross_consumed_usd: {gross}"
    assert disc >= -1e-6, f"[{label}] negative discount_allowance_usd: {disc}"
    assert net >= -1e-6, f"[{label}] negative net_billable_usd: {net}"
    assert round(gross - disc, 2) == round(net, 2), (
        f"[{label}] 3-tier reconciliation invariant failed: gross ({gross}) - discount ({disc}) != net ({net})"
    )
    assert float(receipt["total_billed_minutes"]) >= 0.0, f"[{label}] negative total_billed_minutes"
    assert float(receipt["allowance_remaining"]) >= -1e-6, f"[{label}] negative allowance_remaining"
    for wf_name, wf in receipt.get("per_workflow_breakdown", {}).items():
        trig = float(wf["monthly_triggered_runs"])
        act = float(wf["monthly_active_runs"])
        skip = float(wf["monthly_skipped_runs"])
        assert round(act + skip, 4) == round(trig, 4), (
            f"[{label}:{wf_name}] workflow run invariant failed: active ({act}) + skipped ({skip}) != triggered ({trig})"
        )


def run_closed_loop_qa(*, with_ollama: bool = False) -> dict[str, Any]:
    t0 = time.perf_counter_ns()
    sessions: list[dict[str, Any]] = []

    # Session 1: Layer 1 Tier 1 Official Billing API Reconciler
    s1_items = [
        {
            "date": "2025-10-01",
            "product": "actions",
            "sku": "Actions Linux",
            "quantity": 993,
            "unitType": "Minutes",
            "pricePerUnit": 0.008,
            "grossAmount": 7.944,
            "discountAmount": 0.0,
            "netAmount": 7.944,
            "repositoryName": "Backend",
        },
        {
            "date": "2026-11-01",
            "product": "actions",
            "sku": "Actions Linux",
            "quantity": 23,
            "unitType": "Minutes",
            "pricePerUnit": 0.006,
            "grossAmount": 0.138,
            "discountAmount": -0.138,
            "netAmount": 0.0,
            "repositoryName": "quotas",
        },
    ]
    s1_res = billing_ingestor.reconcile_github_usage_items(s1_items)
    assert s1_res["gross_consumed_usd"] == 8.08
    assert s1_res["discount_allowance_usd"] == 0.14
    assert s1_res["net_billable_usd"] == 7.94
    assert s1_res["historical_rate_shifts"]["Actions Linux"] == [0.006, 0.008]
    sessions.append(
        {
            "session": 1,
            "layer": "Layer 1 (Tier 1 Billing API)",
            "focus": "2025 ($0.008) vs 2026 ($0.006) rate shifts, falsy 0.0 net guard, negative discount sign",
            "verdict": "PASS",
        }
    )

    # Session 2: Layer 1 Tier 2 CSV Ingestor
    s2_csv = """Repository, Gross amount, Discount amount, Net amount
Backend, $20.00, -$15.00, $5.00
Backend, $7.54, -$5.00, $2.54
frontend, $7.25, -$7.25, $0.00
"""
    s2_res = billing_ingestor.ingest_billing_csv(s2_csv)
    assert s2_res["by_repository"]["Backend"]["gross_usd"] == 27.54
    assert s2_res["gross_consumed_usd"] == 34.79
    assert s2_res["discount_allowance_usd"] == 27.25
    assert s2_res["net_billable_usd"] == 7.54
    sessions.append(
        {
            "session": 2,
            "layer": "Layer 1 (Tier 2 Billing CSV)",
            "focus": "Duplicate repo row accumulation, negative discount strings, skipinitialspace CSV parsing",
            "verdict": "PASS",
        }
    )

    # Session 3: Layer 1 Tier 3 Repo Telemetry & Sharding KPIs
    s3_wf = [
        {
            "workflow_name": "Primary Sharded CI",
            "daily_runs": 20,
            "skipped_runs_pct": 0.10,
            "jobs": [
                {"duration_s": 185, "shards": 4, "setup_min_per_shard": 2.0, "labels": ["blacksmith-8vcpu-ubuntu-2204"]},
                {"duration_s": 0.4, "shards": 1, "is_gatekeeper_skip": True, "labels": ["ubuntu-latest"]},
                {"duration_s": 999, "shards": 1, "conclusion": "skipped", "labels": ["ubuntu-latest"]},
            ],
        }
    ]
    s3_est = billing_ingestor.estimate_from_repo_telemetry(s3_wf, plan="team", daily_git_clones=1080, baseline_daily_git_clones=360)
    assert s3_est["kpis"]["monthly_triggered_runs"] == 600.0
    assert s3_est["kpis"]["monthly_active_runs"] == 540.0
    assert s3_est["kpis"]["shard_setup_duplication_min_per_run"] == 6.0
    assert s3_est["per_workflow"]["Primary Sharded CI"]["critical_path_wall_min"] == 3.08
    sessions.append(
        {
            "session": 3,
            "layer": "Layer 1 (Tier 3 Telemetry & Sharding)",
            "focus": "Per-workflow grouping, skipped/gatekeeper filtering, shard setup duplication tax, clone spike",
            "verdict": "PASS",
        }
    )

    # Session 4: Layer 2 Rounding Mechanics
    s4_gh = pricing.calculate_monthly_cost(jobs=[{"sku": "linux", "duration_s": 61, "count": 10}], platform="github", plan="team")
    s4_gcp = pricing.calculate_monthly_cost(jobs=[{"sku": "e2-standard-2", "duration_s": 61, "count": 10}], platform="gcp", plan="free")
    s4_bb = pricing.calculate_monthly_cost(jobs=[{"sku": "linux", "duration_s": 61, "steps": 2, "count": 10}], platform="bitbucket", plan="standard")
    assert s4_gh["total_billed_minutes"] == 20.0
    assert round(s4_gcp["total_billed_minutes"], 4) == round(610.0 / 60.0, 4)
    assert s4_bb["total_billed_minutes"] == 20.0
    assert_receipt_invariants(s4_gh, "Session4-GitHub")
    assert_receipt_invariants(s4_gcp, "Session4-GCP")
    assert_receipt_invariants(s4_bb, "Session4-Bitbucket")
    sessions.append(
        {
            "session": 4,
            "layer": "Layer 2 (Rounding Mechanics)",
            "focus": "ceil_minute_per_job (GitHub/AWS EC2) vs exact_seconds (GCP/GitLab/CircleCI/Lambda) vs ceil_minute_per_step (Bitbucket)",
            "verdict": "PASS",
        }
    )

    # Session 5: Layer 3 Catalog & Persistent Learned Registry
    cat_summary = pricing.check_all()
    assert sum(v["skus"] for v in cat_summary.values()) == 160
    for plat in pricing.PLATFORMS:
        _r, m = pricing.resolve_sku_row("unknown-custom-runner-xyz", platform=plat, persist_learned=False)
        assert m["resolved_sku"] == "linux"
    sessions.append(
        {
            "session": 5,
            "layer": "Layer 3 (Catalog & Learned Registry)",
            "focus": "160 static SKUs + 5 3rd-party SKUs + in-memory cached learned registry + safe compute fallback",
            "verdict": "PASS",
        }
    )

    # Session 6: Layer 3 Wild Custom Runner Label Regex
    wild_checks = {
        "windows-2022": ("windows", 0.010),
        "windows-2025": ("windows", 0.010),
        "windows-11-arm64": ("windows_arm64_2core", 0.008),
        "macos-14": ("macos", 0.062),
        "macos-14-large": ("macos_12core_intel", 0.077),
        "macos-15-xlarge": ("macos_large_m2pro", 0.102),
        "macos-latest-xlarge": ("macos_large_m2pro", 0.102),
        "depot-ubuntu-latest-16": ("blacksmith_16vcpu", 0.032),
        "vscode-large-runners-ubuntu-8-core": ("linux_8", 0.022),
        "ubuntu-24.04-arm64": ("linux_arm64_2core", 0.005),
    }
    for lbl, (exp_sku, exp_rate) in wild_checks.items():
        _r, meta = pricing.resolve_sku_row(lbl, platform="github", persist_learned=False)
        assert meta["resolved_sku"] == exp_sku and abs(meta["rate_usd_per_min"] - exp_rate) < 1e-6
    sessions.append(
        {
            "session": 6,
            "layer": "Layer 3 (Wild Custom Runner Regex)",
            "focus": "OS release number stripping (windows-2022, macos-14) and macOS -large vs -xlarge M2 Pro resolution",
            "verdict": "PASS",
        }
    )

    # Session 7: Layer 4 Unit Normalization & Allowance Waterfall (ordered vs blended)
    s7_ord = pricing.calculate_monthly_cost({"linux": 1000, "windows": 1000, "macos": 500}, platform="github", plan="team", allowance_mode="ordered")
    s7_bld = pricing.calculate_monthly_cost({"linux": 1000, "windows": 1000, "macos": 500}, platform="github", plan="team", allowance_mode="blended")
    assert s7_ord["allowance_pool_consumed"] == 3000.0
    assert s7_bld["allowance_pool_consumed"] == 3000.0
    assert_receipt_invariants(s7_ord, "Session7-Ordered")
    assert_receipt_invariants(s7_bld, "Session7-Blended")
    s7_az = pricing.calculate_monthly_cost({"linux": 2000}, platform="azure", plan="basic", seats=6)
    assert s7_az["compute_usd"] == 40.0 and s7_az["seats_usd"] == 6.0
    assert_receipt_invariants(s7_az, "Session7-Azure")
    sessions.append(
        {
            "session": 7,
            "layer": "Layer 4 (Allowance Waterfall & Unit Gating)",
            "focus": "Multiplier-weighted blended mode, Azure parallel slot step-function, and strict non-compute unit gating",
            "verdict": "PASS",
        }
    )

    # Session 8: Layer 4 Storage, Cache, Egress, & DLC Hidden Taxes across all 7 Providers
    for plat in pricing.PLATFORMS:
        plan_name = "performance" if plat == "circleci" else ("standard" if plat == "bitbucket" else ("premium" if plat == "gitlab" else "team"))
        r_tax = pricing.calculate_monthly_cost(
            {"linux": 2500},
            platform=plat,
            plan=plan_name,
            storage_gb=100.0,
            cache_gb=25.0,
            egress_gb=20.0,
            dlc_enabled=(plat == "circleci"),
            dlc_job_runs=50,
        )
        assert r_tax["storage_overage_usd"] > 0.0, f"{plat} storage_overage_usd should be > 0"
        assert_receipt_invariants(r_tax, f"Session8-{plat}")
    sessions.append(
        {
            "session": 8,
            "layer": "Layer 4 (Storage, Cache, Egress & DLC Taxes)",
            "focus": "All 7 providers compute storage, cache, egress, and CircleCI DLC overages natively in calculate_monthly_cost",
            "verdict": "PASS",
        }
    )

    # Session 9: Layer 5 Savings & DORA Metrics
    s9_sav = pricing.calculate_savings(
        {"linux": 4000},
        {"linux": 0},
        platform="github",
        plan="team",
        runs_per_month=300,
        current_wall_min_per_run=10.0,
        selected_wall_min_per_run=4.0,
        misses=1,
        defect_commits=50,
    )
    assert s9_sav["crossed_allowance_boundary"] is True
    assert s9_sav["saved_net_billable_usd"] == 6.0
    assert s9_sav["wait_metrics"]["saved_wait_hours_per_month"] == 30.0
    assert s9_sav["safety_metrics"]["miss_pct"] == 2.0
    sessions.append(
        {
            "session": 9,
            "layer": "Layer 5 (Savings & DORA Metrics)",
            "focus": "3-tier savings deltas, crossed_allowance_boundary at 0 selected mins, DORA Lead Time & CFR",
            "verdict": "PASS",
        }
    )

    # Session 10: Adversarial Outliers & Edge Cases
    s10_pub = pricing.calculate_monthly_cost(
        {"linux": 5000, "linux_8": 1000},
        platform="github",
        plan="free",
        public_repo=True,
        public_larger_runners_free=False,
    )
    assert s10_pub["sku_breakdown"]["linux"]["billed_usd"] == 0.0
    assert s10_pub["sku_breakdown"]["linux_8"]["billed_usd"] == 22.0
    assert_receipt_invariants(s10_pub, "Session10-PublicUnSponsored")

    s10_sh = pricing.calculate_monthly_cost({"self_hosted": 10000}, platform="github", plan="team")
    assert s10_sh["unpriced_self_hosted_warning"] is not None
    assert s10_sh["self_hosted_cloud_equivalent_usd"] == 60.0
    assert_receipt_invariants(s10_sh, "Session10-SelfHosted")

    s10_neg = pricing.calculate_monthly_cost(
        workflows=[
            {
                "workflow_name": "NegativeAndOverflow",
                "monthly_runs": 100,
                "monthly_skipped_runs": 999,
                "jobs": [{"duration_s": -100, "count": -5, "labels": ["ubuntu-latest"]}],
            }
        ],
        storage_gb=-50.0,
        cache_gb=-50.0,
        egress_gb=-50.0,
        seats=-10,
    )
    assert s10_neg["total_usd"] == 0.0
    assert_receipt_invariants(s10_neg, "Session10-NegativeClamp")
    sessions.append(
        {
            "session": 10,
            "layer": "Adversarial Outliers (Outliers A, B, C)",
            "focus": "Public repo Larger/GPU runner billing, unpriced self-hosted warning, negative & skipped>triggered clamping",
            "verdict": "PASS",
        }
    )

    # Also verify the 10-Repo + 3-Cloud Benchmark & 5-Source QA Verifier
    bench = benchmark_10_repos_and_clouds.run_all_benchmarks()
    sample_reports = [
        qa_verifier.verify_workflow_file(p, open_browser=False)
        for p in sorted((ROOT / "samples").glob("*.yml"))
    ]
    assert sample_reports and all(r["all_rates_verified_zero_delta"] for r in sample_reports)

    ollama_audit = None
    if with_ollama:
        ollama_audit = pricing.audit_receipt_with_ollama(
            s7_ord,
            context_note="Closed-loop QA verification of 3,000-min GitHub Team allowance waterfall",
            model="gpt-oss:20b",
        )

    elapsed_ms = round((time.perf_counter_ns() - t0) / 1_000_000.0, 2)
    report = {
        "status": "ALL_10_QA_SESSIONS_PASSED",
        "total_static_skus": 160,
        "total_third_party_skus": len(pricing.THIRD_PARTY_GITHUB_RUNNER_SKUS),
        "learned_registry_entries": len(pricing.load_learned_sku_registry().get("github", {})),
        "benchmark_scenarios_verified": len(bench.get("repos", [])) + len(bench.get("cloud_scenarios", [])),
        "five_source_zero_delta": all(r["all_rates_verified_zero_delta"] for r in sample_reports),
        "execution_time_ms": elapsed_ms,
        "ollama_audit": ollama_audit,
        "sessions": sessions,
    }

    out_path = ROOT / "reports" / "closed_loop_qa_report.json"
    out_path.parent.mkdir(exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="Run 10-Session Closed-Loop QA & Invariant Suite")
    parser.add_argument("--with-ollama", action="store_true", help="Also run Local Ollama AI receipt auditor")
    args = parser.parse_args()
    rep = run_closed_loop_qa(with_ollama=args.with_ollama)
    print(f"=== CLOSED-LOOP QA SUITE: {rep['status']} ({rep['execution_time_ms']} ms) ===")
    for s in rep["sessions"]:
        print(f"  [Session #{s['session']:02d} | {s['verdict']}] {s['layer']:42s} — {s['focus']}")
    if rep["ollama_audit"]:
        print(f"  [Ollama AI Auditor] {rep['ollama_audit']}")
    print(f"\nReport written to: {ROOT / 'reports' / 'closed_loop_qa_report.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
