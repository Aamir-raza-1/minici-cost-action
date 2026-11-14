"""Verification suite for all 7 CI/CD providers + Infracost delegation (`engine/pricing.py`)."""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from engine import billing_ingestor, pricing


def test_all_7_providers_load_and_sync_csv() -> None:
    assert pricing.PLATFORMS == ("github", "gitlab", "circleci", "bitbucket", "aws", "gcp", "azure")
    summary = pricing.check_all()
    assert summary["github"]["skus"] == 45
    assert summary["gitlab"]["skus"] == 24
    assert summary["circleci"]["skus"] == 32
    assert summary["bitbucket"]["skus"] == 13
    assert summary["aws"]["skus"] == 21
    assert summary["gcp"]["skus"] == 12
    assert summary["azure"]["skus"] == 13
    assert sum(v["skus"] for v in summary.values()) == 160


def test_side_by_side_qa_fixes_applied() -> None:
    gh = pricing.get_sku_map("github")
    assert gh["actions_linux_slim"]["usd"] == 0.002
    assert gh["ubuntu-slim"]["usd"] == 0.002
    assert gh["actions_cache_gb_hour"]["usd"] == 0.000094
    assert gh["actions_cache_gb_hour"]["usd_per_gb_month"] == 0.07
    assert gh["actions_custom_image_storage"]["usd"] == 0.07

    cci = pricing.get_sku_map("circleci")
    assert cci["linux_arm64_medium"]["usd"] == 0.0078
    assert cci["linux_arm64_medium"]["credits_per_min"] == 13
    assert cci["macos_m1_medium"]["usd"] == 0.09
    assert cci["macos_m1_medium"]["credits_per_min"] == 150
    assert cci["macos_m2pro_medium"]["usd"] == 0.12
    assert cci["docker_layer_caching"]["usd"] == 0.12

    bb = pricing.get_sku_map("bitbucket")
    assert bb["linux_24x"]["usd"] == 0.24
    assert bb["linux_32x"]["usd"] == 0.32

    gl = pricing.get_sku_map("gitlab")
    assert gl["oss_public_project_factor"]["cost_factor"] == 0.5
    assert gl["public_fork_mr_factor"]["cost_factor"] == 0.008


def test_calculate_monthly_cost_across_all_7_providers() -> None:
    for platform in pricing.PLATFORMS:
        plan = "performance" if platform == "circleci" else ("standard" if platform == "bitbucket" else ("premium" if platform == "gitlab" else "team"))
        res = pricing.calculate_monthly_cost({"linux": 5000}, platform=platform, plan=plan, seats=5)
        assert res["platform"] == platform
        assert res["total_usd"] >= 0.0


def test_infracost_adapter_configured() -> None:
    assert Path(pricing.INFRACOST_BIN).name == "infracost"
    aws_data = pricing.load_platform_pricing("aws")
    assert aws_data["rules"]["infracost_delegated"] is True
    assert "aws_codebuild_project" in aws_data["rules"]["infracost_terraform_resources"]
    assert set(pricing.MULTI_CLOUD_PRICING_SOURCES.keys()) == {
        "infracost",
        "gcosts",
        "economize",
        "vantage",
        "azure_retail_api",
        "gcp_pricing_calculator",
        "aws_pricing_calculator",
        "azure_pricing_calculator",
    }


def test_tier1_and_tier2_eazyops_billing_reconciliation() -> None:
    # 1. Nov 2026 MTD from /orgs/Eazy-Ops/settings/billing/usage (23 mins @ $0.006 = $0.138 -> $0.14)
    nov_items = [
        {
            "date": "2026-11-01T00:00:00Z",
            "product": "actions",
            "sku": "Actions Linux",
            "quantity": 23.0,
            "unitType": "Minutes",
            "pricePerUnit": 0.006,
            "grossAmount": 0.138,
            "discountAmount": 0.138,
            "netAmount": 0.0,
            "organizationName": "Eazy-Ops",
            "repositoryName": "quotas",
        }
    ]
    nov_res = billing_ingestor.reconcile_github_usage_items(nov_items)
    assert nov_res["gross_consumed_usd"] == 0.14
    assert nov_res["discount_allowance_usd"] == 0.14
    assert nov_res["net_billable_usd"] == 0.00

    # 2. Tier 2 CSV / UI Repository Table for Eazy-Ops Last Year ($45.26 gross, -$37.42 discount, $7.85 net)
    csv_summary = """Repository,Gross amount
Backend,$27.54
frontend,$7.25
quotas,$4.60
discovery,$4.13
website,$1.12
All other usage,$0.62
"""
    yr_res = billing_ingestor.ingest_billing_csv(csv_summary, discount_usd=37.42, net_usd=7.85)
    assert yr_res["gross_consumed_usd"] == 45.26
    assert yr_res["discount_allowance_usd"] == 37.42
    assert yr_res["net_billable_usd"] == 7.85
    assert yr_res["discount_pct"] == 82.68
    assert yr_res["by_repository"]["Backend"]["gross_usd"] == 27.54
    assert yr_res["by_repository"]["frontend"]["gross_usd"] == 7.25


def test_tier3_per_workflow_telemetry_estimator_avoids_aggregate_trap() -> None:
    # Simulates a multi-workflow monorepo where 1 primary CI workflow runs on third-party 8vcpu/4vcpu runners,
    # 1 short compliance workflow takes 1.2m, and 1 deploy workflow is 97% skipped.
    workflows = [
        {
            "workflow_name": "Continuous Integration - QA",
            "monthly_runs": 3301,
            "monthly_skipped_runs": 0,
            "critical_path_wall_min": 9.75,
            "jobs": [
                {"duration_s": 240, "shards": 1, "labels": ["blacksmith-8vcpu-ubuntu-2204"]},
                {"duration_s": 318, "shards": 4, "setup_min_per_shard": 2.0, "labels": ["blacksmith-8vcpu-ubuntu-2204"]},
                {"duration_s": 225, "shards": 4, "setup_min_per_shard": 2.0, "labels": ["blacksmith-8vcpu-ubuntu-2204"]},
                {"duration_s": 384, "shards": 1, "labels": ["blacksmith-4vcpu-ubuntu-2204"]},
            ],
        },
        {
            "workflow_name": "License Check",
            "monthly_runs": 3450,
            "monthly_skipped_runs": 0,
            "critical_path_wall_min": 1.2,
            "jobs": [
                {"duration_s": 72, "shards": 1, "labels": ["blacksmith-4vcpu-ubuntu-2204"]},
            ],
        },
        {
            "workflow_name": "Preview Deploy",
            "monthly_runs": 5801,
            "monthly_skipped_runs": 5640,
            "critical_path_wall_min": 5.0,
            "jobs": [
                {"duration_s": 300, "shards": 1, "labels": ["blacksmith-4vcpu-ubuntu-2204"]},
            ],
        },
    ]
    est = billing_ingestor.estimate_from_repo_telemetry(
        workflows,
        plan="team",
        public_repo=False,
        daily_git_clones=3389,
        baseline_daily_git_clones=1247,
    )
    assert est["tier"] == "repo_telemetry"
    assert est["third_party_runner_bill"]["intercepts_actions_cache"] is True
    assert est["github_metered_bill"]["net_billable_usd"] == 0.0
    assert est["kpis"]["shard_setup_duplication_min_per_run"] == 12.0
    assert est["kpis"]["overestimation_avoided_pct"] > 50.0
    assert est["kpis"]["daily_git_clone_spike_pct"] == 171.77


def test_qa_sessions_1_and_2_falsy_zero_and_negative_discounts() -> None:
    # Session #1 & #2: negative discount string ("-$37.42") + explicit netAmount=0.0 must never double-charge
    csv_negative_disc = """date,product,sku,quantity,unitType,pricePerUnit,grossAmount,discountAmount,netAmount,repositoryName
2026-11-01,actions,Actions Linux,23,Minutes,$0.006,$0.14,-$0.14,$0.00,quotas
2026-11-02,actions,Actions Linux,0,Minutes,$0.999,$0.00,$0.00,$0.00,quotas
"""
    res = billing_ingestor.ingest_billing_csv(csv_negative_disc)
    assert res["gross_consumed_usd"] == 0.14
    assert res["discount_allowance_usd"] == 0.14
    assert res["net_billable_usd"] == 0.00
    # Zero-quantity placeholder row ($0.999) must not pollute observed_rates
    assert res["by_sku"]["Actions Linux"]["observed_rates"] == [0.006]

    # Summary CSV with duplicate repo rows and inline Discount/Net columns
    csv_dup_repos = """Repository, Gross amount, Discount amount, Net amount
Backend, $20.00, -$15.00, $5.00
Backend, $7.54, -$5.00, $2.54
frontend, $7.25, -$7.25, $0.00
"""
    res_b = billing_ingestor.ingest_billing_csv(csv_dup_repos)
    assert res_b["by_repository"]["Backend"]["gross_usd"] == 27.54
    assert res_b["gross_consumed_usd"] == 34.79
    assert res_b["discount_allowance_usd"] == 27.25
    assert res_b["net_billable_usd"] == 7.54


def test_qa_session_3_and_4_workflow_parity_and_rounding_modes() -> None:
    # Session #3: calculate_monthly_cost(workflows=...) and estimate_from_repo_telemetry must agree on shards & gatekeeper skips
    wf_spec = [
        {
            "workflow_name": "Sharded CI",
            "daily_runs": 10,
            "skipped_runs_pct": 0.2,
            "jobs": [
                {"duration_s": 125, "shards": 4, "setup_min_per_shard": 1.0, "labels": ["ubuntu-latest"]},
                {"duration_s": 0.5, "shards": 1, "is_gatekeeper_skip": True, "labels": ["ubuntu-latest"]},
                {"duration_s": 900, "shards": 1, "conclusion": "skipped", "labels": ["ubuntu-latest"]},
            ],
        }
    ]
    c_res = pricing.calculate_monthly_cost(workflows=wf_spec, platform="github", plan="team")
    t_res = billing_ingestor.estimate_from_repo_telemetry(wf_spec, plan="team")
    # 10/day * 30 = 300 triggered, 60 skipped, 240 active; 125s -> 3 ceil min * 4 shards = 12 mins/run -> 2,880 mins
    assert c_res["total_billed_minutes"] == 2880.0
    assert t_res["kpis"]["monthly_billed_minutes"] == 2880.0
    assert t_res["per_workflow"]["Sharded CI"]["critical_path_wall_min"] == 2.08

    # Session #4: Exact seconds on GCP/GitLab/CircleCI/AWS Lambda vs ceil_minute_per_step on Bitbucket
    gcp_res = pricing.calculate_monthly_cost(
        jobs=[{"sku": "e2-standard-2", "duration_s": 90, "count": 10}],
        platform="gcp",
        plan="free",
    )
    assert gcp_res["total_billed_minutes"] == 15.0  # 10 * 1.5m exact_seconds

    bb_res = pricing.calculate_monthly_cost(
        jobs=[{"sku": "linux", "duration_s": 90, "steps": 3, "count": 10}],
        platform="bitbucket",
        plan="standard",
    )
    assert bb_res["total_billed_minutes"] == 30.0  # 3 steps of 30s -> 3 ceil mins * 10 = 30m

    aws_lambda = pricing.calculate_monthly_cost(
        jobs=[{"sku": "BUILD_LAMBDA_ARM_1GB", "duration_s": 30, "count": 10}],
        platform="aws",
        plan="free",
    )
    assert aws_lambda["total_billed_minutes"] == 5.0  # 10 * 0.5m exact_seconds


def test_qa_session_5_and_6_wild_runner_labels_and_os_version_stripping() -> None:
    # Session #6: OS release numbers in windows-2022, windows-2025, windows-11-arm64, macos-14, macos-15
    # must NEVER be parsed as CPU core counts, and macos-15-xlarge must map to macos_large_m2pro
    expected_labels = {
        "windows-2022": ("windows", 0.010),
        "windows-2025": ("windows", 0.010),
        "windows-11-arm64": ("windows_arm64_2core", 0.008),
        "macos-14": ("macos", 0.062),
        "macos-15": ("macos", 0.062),
        "macos-14-large": ("macos_12core_intel", 0.077),
        "macos-15-xlarge": ("macos_large_m2pro", 0.102),
        "macos-latest-large": ("macos_12core_intel", 0.077),
        "macos-latest-xlarge": ("macos_large_m2pro", 0.102),
        "depot-ubuntu-latest-16": ("blacksmith_16vcpu", 0.032),
        "vscode-large-runners-ubuntu-8-core": ("linux_8", 0.022),
        "ubuntu-24.04-arm64": ("linux_arm64_2core", 0.005),
    }
    for raw_label, (exp_sku, exp_rate) in expected_labels.items():
        _row, meta = pricing.resolve_sku_row(raw_label, platform="github", persist_learned=False)
        assert meta["resolved_sku"] == exp_sku, f"{raw_label}: got {meta['resolved_sku']}, expected {exp_sku}"
        assert abs(meta["rate_usd_per_min"] - exp_rate) < 1e-6

    # Session #5: Unrecognized label on CircleCI/Azure must fall back to 'linux' compute SKU, never a seat/plan SKU
    for plat in ("circleci", "azure", "gitlab", "gcp"):
        _r, m = pricing.resolve_sku_row("completely-unknown-xyz-token", platform=plat, persist_learned=False)
        assert m["resolved_sku"] == "linux"


def test_qa_session_7_and_8_blended_multipliers_azure_slots_and_7_provider_taxes() -> None:
    # Session #7: blended mode with 2x Windows and 10x macOS must consume multiplier-weighted pool units
    workload = {"linux": 1000, "windows": 1000, "macos": 500}  # 1000*1 + 1000*2 + 500*10 = 8000 pool units
    ord_res = pricing.calculate_monthly_cost(workload, platform="github", plan="team", allowance_mode="ordered")
    bld_res = pricing.calculate_monthly_cost(workload, platform="github", plan="team", allowance_mode="blended")
    assert ord_res["allowance_pool_consumed"] == 3000.0
    assert bld_res["allowance_pool_consumed"] == 3000.0
    assert bld_res["allowance_remaining"] == 0.0
    assert bld_res["compute_usd"] > 0.0

    # Session #7: Azure parallel slot step-function ($0 for <=1800m, $40/slot for >1800m)
    az_free = pricing.calculate_monthly_cost({"linux": 1500}, platform="azure", plan="basic", seats=5)
    assert az_free["compute_usd"] == 0.0
    assert az_free["seats_usd"] == 0.0
    az_paid = pricing.calculate_monthly_cost({"linux": 2000}, platform="azure", plan="basic", seats=6)
    assert az_paid["compute_usd"] == 40.0
    assert az_paid["seats_usd"] == 6.0

    # Session #8: All 7 providers compute storage, cache, egress, and DLC hidden taxes
    cci_tax = pricing.calculate_monthly_cost(
        {"linux": 1000},
        platform="circleci",
        plan="performance",
        storage_gb=20.0,
        cache_gb=10.0,
        egress_gb=5.0,
        dlc_enabled=True,
        dlc_job_runs=100,
    )
    assert cci_tax["storage_overage_usd"] > 0.0
    assert cci_tax["cache_overage_usd"] > 0.0
    assert cci_tax["egress_overage_usd"] > 0.0
    assert cci_tax["dlc_overage_usd"] == 12.0


def test_qa_session_9_and_10_savings_dora_and_adversarial_outliers() -> None:
    # Session #9: 3-tier savings keys and crossed_allowance_boundary when selected drops to 0
    sav = pricing.calculate_savings(
        {"linux": 4000},
        {"linux": 0},
        platform="github",
        plan="team",
        runs_per_month=300,
        current_wall_min_per_run=10.0,
        selected_wall_min_per_run=4.0,
    )
    assert sav["crossed_allowance_boundary"] is True
    assert sav["saved_net_billable_usd"] == 6.0
    assert sav["saved_gross_consumed_usd"] == 24.0
    assert sav["saved_discount_allowance_usd"] == 18.0
    assert sav["wait_metrics"]["saved_wait_hours_per_month"] == 30.0

    # Session #10 Outlier A: public_repo=True with public_larger_runners_free=False bills Larger/GPU runners
    pub_unsponsored = pricing.calculate_monthly_cost(
        {"linux": 5000, "linux_8": 1000},
        platform="github",
        plan="free",
        public_repo=True,
        public_larger_runners_free=False,
    )
    assert pub_unsponsored["sku_breakdown"]["linux"]["billed_usd"] == 0.0
    assert pub_unsponsored["sku_breakdown"]["linux_8"]["billed_usd"] == 22.0
    assert pub_unsponsored["net_billable_usd"] == 22.0

    # Session #10 Outlier B: Self-hosted runner emits explicit warning and cloud-equivalent USD
    sh_res = pricing.calculate_monthly_cost({"self_hosted": 10000}, platform="github", plan="team")
    assert sh_res["unpriced_self_hosted_warning"] is not None
    assert sh_res["self_hosted_cloud_equivalent_usd"] == 60.0

    # Session #10 Outlier C: Negative inputs clamped to 0 and skipped_runs capped at monthly_runs
    adv_res = pricing.calculate_monthly_cost(
        workflows=[
            {
                "workflow_name": "Adversarial",
                "monthly_runs": 100,
                "monthly_skipped_runs": 500,  # > monthly_runs
                "jobs": [{"duration_s": -50, "count": -3, "labels": ["ubuntu-latest"]}],
            }
        ],
        storage_gb=-10.0,
        cache_gb=-20.0,
        seats=-5,
    )
    assert adv_res["total_usd"] == 0.0
    assert adv_res["per_workflow_breakdown"]["Adversarial"]["monthly_active_runs"] == 0.0
    assert adv_res["per_workflow_breakdown"]["Adversarial"]["monthly_skipped_runs"] == 100.0


if __name__ == "__main__":
    import sys

    passed = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"PASS: {name}")
            passed += 1
    print(f"\nAll {passed} test suites passed.")
    sys.exit(0)



