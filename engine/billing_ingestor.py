"""Generic 3-Tier CI/CD Billing & Repository Telemetry Ingestor (`engine/billing_ingestor.py`).

Implements the 3-tier waterfall documented in `LESSONS_LEARNED.md`:
  - Tier 1 (`official_api`): GitHub Enhanced Billing REST API (`/orgs/{org}/settings/billing/usage`)
  - Tier 2 (`billing_csv`): Exported GitHub/Cloud Billing CSV or UI repository summary table
  - Tier 3 (`repo_telemetry`): Read-only repository workflow runs + traffic clones + commit activity
    with strict per-workflow grouping, skipped-run filtering, third-party runner detection,
    and sharding setup duplication / critical-path KPI calculation.
"""
from __future__ import annotations

import csv
import io
import json
import math
import subprocess
from pathlib import Path
from typing import Any, Iterable, Sequence

from engine.pricing import (
    billed_minutes_for_jobs,
    calculate_monthly_cost,
    ceil_job_minutes,
    get_sku_map,
    resolve_sku_row,
)

THIRD_PARTY_RUNNER_PREFIXES: tuple[str, ...] = (
    "blacksmith-",
    "warpbuild-",
    "buildjet-",
    "depot-",
    "namespace-",
    "runs-on-",
)

THIRD_PARTY_RUNNER_RATES_USD_PER_MIN: dict[str, float] = {
    "blacksmith-2vcpu": 0.004,
    "blacksmith-4vcpu": 0.008,
    "blacksmith-8vcpu": 0.016,
    "blacksmith-16vcpu": 0.032,
    "blacksmith-32vcpu": 0.064,
    "warpbuild-2x": 0.004,
    "warpbuild-4x": 0.008,
    "warpbuild-8x": 0.016,
    "warpbuild-16x": 0.032,
    "warpbuild-32x": 0.064,
    "buildjet-2vcpu": 0.004,
    "buildjet-4vcpu": 0.008,
    "buildjet-8vcpu": 0.016,
    "buildjet-16vcpu": 0.032,
    "buildjet-32vcpu": 0.064,
}


def _parse_currency_float(val: Any, default: float | None = 0.0) -> float | None:
    """Safely parse a numeric or currency string (`'$27.54'`, `'-$37.42'`, `'0'`, `0.0`) without falsy `0.0` bugs."""
    if val is None:
        return default
    if isinstance(val, (int, float)):
        return float(val)
    s = str(val).strip().replace("$", "").replace(",", "")
    if not s:
        return default
    return float(s)


def classify_runner_label(labels: Sequence[str] | str) -> dict[str, Any]:
    """Classify a job's `runs-on` labels into GitHub-hosted, third-party microVM, or self-hosted."""
    label_list = [labels] if isinstance(labels, str) else list(labels or [])
    normalized = [str(lbl).strip().lower() for lbl in label_list if str(lbl).strip()]

    for lbl in normalized:
        if any(lbl.startswith(p) or p.rstrip("-") in lbl for p in THIRD_PARTY_RUNNER_PREFIXES):
            provider = next(
                (p.rstrip("-") for p in THIRD_PARTY_RUNNER_PREFIXES if lbl.startswith(p) or p.rstrip("-") in lbl),
                "third_party",
            )
            row, meta = resolve_sku_row(lbl, platform="github")
            rate = float(meta["rate_usd_per_min"])
            for key, val in THIRD_PARTY_RUNNER_RATES_USD_PER_MIN.items():
                if lbl.startswith(key):
                    rate = val
                    break
            return {
                "billing_target": "third_party_runner",
                "provider": provider,
                "sku": lbl,
                "resolved_sku": meta["resolved_sku"],
                "rate_usd_per_min": rate,
                "intercepts_actions_cache": True,
            }

    if any(
        tok in lbl
        for lbl in normalized
        for tok in ("self-hosted", "self_hosted", "arc-runner", "k8s-runner", "actuated")
    ):
        return {
            "billing_target": "self_hosted",
            "provider": "self_hosted",
            "sku": "self_hosted",
            "rate_usd_per_min": 0.0,
            "intercepts_actions_cache": False,
        }

    # Default to 4-Stage GitHub-hosted SKU mapping via pricing.resolve_sku_row
    first_lbl = normalized[0] if normalized else "linux"
    row, meta = resolve_sku_row(first_lbl, platform="github")
    return {
        "billing_target": "github_hosted",
        "provider": "github",
        "sku": meta["resolved_sku"],
        "rate_usd_per_min": float(meta["rate_usd_per_min"]),
        "resolution_mode": meta["resolution_mode"],
        "intercepts_actions_cache": False,
    }


def reconcile_github_usage_items(
    usage_items: Sequence[dict[str, Any]],
    *,
    product_filter: str | None = "actions",
    repo_gross_overrides: dict[str, float] | None = None,
) -> dict[str, Any]:
    """Tier 1 & Tier 2 core reconciler for GitHub Enhanced Billing `usageItems`.

    Always outputs the 3 mandatory financial numbers:
      - `gross_consumed_usd`
      - `discount_allowance_usd`
      - `net_billable_usd`
    plus historical rate shift detection (`$0.008/min` in 2025 vs `$0.006/min` in 2026).
    """
    filtered = [
        item
        for item in usage_items
        if product_filter is None or str(item.get("product", "")).strip().lower() == product_filter.lower()
    ]

    gross_total = 0.0
    discount_total = 0.0
    net_total = 0.0
    by_sku: dict[str, dict[str, Any]] = {}
    by_repo: dict[str, dict[str, float]] = {}
    observed_rates_by_sku: dict[str, set[float]] = {}
    dates_seen: list[str] = []

    for item in filtered:
        sku_name = str(item.get("sku") or item.get("SKU") or "Unknown").strip()
        repo_name = str(
            item.get("repositoryName") or item.get("repository_name") or item.get("Repository") or "All other usage"
        ).strip()
        qty = max(0.0, float(_parse_currency_float(item.get("quantity", item.get("Quantity")), 0.0) or 0.0))
        unit_type = str(item.get("unitType") or item.get("unit_type") or "Minutes").strip()
        price_per_unit = max(
            0.0, float(_parse_currency_float(item.get("pricePerUnit", item.get("price_per_unit")), 0.0) or 0.0)
        )
        gross = max(
            0.0, float(_parse_currency_float(item.get("grossAmount", item.get("gross_amount")), 0.0) or 0.0)
        )
        raw_disc = item.get("discountAmount") if "discountAmount" in item else item.get("discount_amount")
        discount = abs(float(_parse_currency_float(raw_disc, 0.0) or 0.0))

        raw_net = item.get("netAmount") if "netAmount" in item else item.get("net_amount")
        parsed_net = _parse_currency_float(raw_net, default=None)
        if parsed_net is not None:
            net = max(0.0, float(parsed_net))
        else:
            net = max(0.0, gross - discount)

        date_str = str(item.get("date") or item.get("Date") or "").strip()
        if date_str:
            dates_seen.append(date_str[:10])

        gross_total += gross
        discount_total += discount
        net_total += net

        if price_per_unit > 0.0 and qty > 0.0:
            observed_rates_by_sku.setdefault(sku_name, set()).add(round(price_per_unit, 8))

        sku_bucket = by_sku.setdefault(
            sku_name,
            {
                "sku": sku_name,
                "unit_type": unit_type,
                "quantity": 0.0,
                "gross_usd": 0.0,
                "discount_usd": 0.0,
                "net_usd": 0.0,
                "observed_rates": [],
            },
        )
        sku_bucket["quantity"] += qty
        sku_bucket["gross_usd"] += gross
        sku_bucket["discount_usd"] += discount
        sku_bucket["net_usd"] += net

        repo_bucket = by_repo.setdefault(
            repo_name,
            {"gross_usd": 0.0, "discount_usd": 0.0, "net_usd": 0.0, "quantity": 0.0},
        )
        repo_bucket["gross_usd"] += gross
        repo_bucket["discount_usd"] += discount
        repo_bucket["net_usd"] += net
        repo_bucket["quantity"] += qty

    for sku_name, rates in observed_rates_by_sku.items():
        by_sku[sku_name]["observed_rates"] = sorted(rates)

    # Detect if historical rate shift occurred (e.g. 2025 $0.008/min vs 2026 $0.006/min)
    rate_shifts = {
        sku_name: sorted(rates)
        for sku_name, rates in observed_rates_by_sku.items()
        if len(rates) > 1
    }

    # If UI / CSV per-repository gross breakdown was supplied, attach it cleanly
    if repo_gross_overrides is not None:
        by_repo = {
            r: {
                "gross_usd": float(g),
                "share_pct": (float(g) / gross_total * 100.0) if gross_total > 0 else 0.0,
            }
            for r, g in repo_gross_overrides.items()
        }

    discount_pct = (discount_total / gross_total * 100.0) if gross_total > 0 else 0.0

    return {
        "tier": "official_api",
        "gross_consumed_usd": round(gross_total, 2),
        "gross_consumed_exact_usd": round(gross_total, 6),
        "discount_allowance_usd": round(discount_total, 2),
        "discount_allowance_exact_usd": round(discount_total, 6),
        "net_billable_usd": round(net_total, 2),
        "net_billable_exact_usd": round(net_total, 6),
        "discount_pct": round(discount_pct, 2),
        "items_count": len(filtered),
        "date_range": (min(dates_seen), max(dates_seen)) if dates_seen else None,
        "by_sku": by_sku,
        "by_repository": by_repo,
        "historical_rate_shifts": rate_shifts,
    }


def fetch_github_official_billing(
    org: str,
    *,
    year: int | None = None,
    month: int | None = None,
) -> dict[str, Any]:
    """Tier 1: Query `/orgs/{org}/settings/billing/usage` via `gh api` when Org Admin / Billing access exists."""
    endpoint = f"/orgs/{org}/settings/billing/usage"
    params: list[str] = []
    if year is not None:
        params.append(f"year={int(year)}")
    if month is not None:
        params.append(f"month={int(month)}")
    if params:
        endpoint = f"{endpoint}?{'&'.join(params)}"

    proc = subprocess.run(
        ["gh", "api", endpoint],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise PermissionError(
            f"GitHub Billing API {endpoint} unavailable ({proc.returncode}): {proc.stderr.strip()}"
        )
    payload = json.loads(proc.stdout)
    items = payload.get("usageItems", [])
    result = reconcile_github_usage_items(items)
    result["organization"] = org
    result["endpoint"] = endpoint
    return result


def ingest_billing_csv(
    csv_text_or_path: str | Path,
    *,
    discount_usd: float | None = None,
    net_usd: float | None = None,
) -> dict[str, Any]:
    """Tier 2: Parse a user-uploaded GitHub Enhanced Billing CSV export or repository summary CSV."""
    if isinstance(csv_text_or_path, Path) or (
        isinstance(csv_text_or_path, str)
        and "\n" not in csv_text_or_path
        and Path(csv_text_or_path).exists()
    ):
        raw_text = Path(csv_text_or_path).read_text(encoding="utf-8")
    else:
        raw_text = str(csv_text_or_path)

    reader = list(csv.DictReader(io.StringIO(raw_text.strip()), skipinitialspace=True))
    if not reader:
        raise ValueError("Billing CSV is empty")

    first_row = reader[0]
    lower_keys = {k.strip().lower(): k for k in first_row if k is not None}

    # Case A: Full GitHub Enhanced Billing usage report CSV (has sku / quantity / gross_amount)
    if any(k in lower_keys for k in ("sku", "grossamount", "gross_amount", "priceperunit", "price_per_unit")):
        normalized_items = []
        for row in reader:
            lrow = {k.strip().lower(): v for k, v in row.items() if k is not None}
            raw_disc = lrow.get("discountamount") if "discountamount" in lrow else lrow.get("discount_amount")
            raw_net = lrow.get("netamount") if "netamount" in lrow else lrow.get("net_amount")
            parsed_net = _parse_currency_float(raw_net, default=None)
            item_dict: dict[str, Any] = {
                "date": lrow.get("date", ""),
                "product": lrow.get("product", "actions"),
                "sku": lrow.get("sku", "Actions Linux"),
                "quantity": float(_parse_currency_float(lrow.get("quantity"), 0.0) or 0.0),
                "unitType": lrow.get("unittype") or lrow.get("unit_type") or "Minutes",
                "pricePerUnit": float(
                    _parse_currency_float(lrow.get("priceperunit", lrow.get("price_per_unit")), 0.0) or 0.0
                ),
                "grossAmount": float(
                    _parse_currency_float(lrow.get("grossamount", lrow.get("gross_amount")), 0.0) or 0.0
                ),
                "discountAmount": abs(float(_parse_currency_float(raw_disc, 0.0) or 0.0)),
                "repositoryName": (
                    lrow.get("repositoryname")
                    or lrow.get("repository_name")
                    or lrow.get("repository")
                    or "All other usage"
                ),
            }
            if parsed_net is not None:
                item_dict["netAmount"] = max(0.0, float(parsed_net))
            normalized_items.append(item_dict)
        res = reconcile_github_usage_items(normalized_items, product_filter=None)
        res["tier"] = "billing_csv"
        return res

    # Case B: UI Summary table by Repository (Repository, Gross amount [, Discount amount, Net amount])
    if "repository" in lower_keys and any(k in lower_keys for k in ("gross amount", "gross_amount", "gross")):
        repo_key = lower_keys["repository"]
        gross_key = (
            lower_keys.get("gross amount")
            or lower_keys.get("gross_amount")
            or lower_keys["gross"]
        )
        disc_col_key = (
            lower_keys.get("discount amount")
            or lower_keys.get("discount_amount")
            or lower_keys.get("discount")
        )
        net_col_key = (
            lower_keys.get("net amount")
            or lower_keys.get("net_amount")
            or lower_keys.get("net")
        )
        by_repo: dict[str, dict[str, float]] = {}
        gross_total = 0.0
        csv_disc_total = 0.0
        csv_net_total = 0.0
        has_csv_disc = False
        has_csv_net = False

        for row in reader:
            rname = str(row.get(repo_key) or "All other usage").strip()
            gval = max(0.0, float(_parse_currency_float(row.get(gross_key), 0.0) or 0.0))
            gross_total += gval
            prev_gross = by_repo.get(rname, {}).get("gross_usd", 0.0)
            by_repo[rname] = {"gross_usd": round(prev_gross + gval, 2)}

            if disc_col_key and row.get(disc_col_key) is not None and str(row.get(disc_col_key)).strip() != "":
                has_csv_disc = True
                csv_disc_total += abs(float(_parse_currency_float(row.get(disc_col_key), 0.0) or 0.0))
            if net_col_key and row.get(net_col_key) is not None and str(row.get(net_col_key)).strip() != "":
                has_csv_net = True
                csv_net_total += max(0.0, float(_parse_currency_float(row.get(net_col_key), 0.0) or 0.0))

        for rname, rinfo in by_repo.items():
            rinfo["share_pct"] = round((rinfo["gross_usd"] / gross_total * 100.0) if gross_total > 0 else 0.0, 2)

        if discount_usd is not None:
            disc_val = abs(float(discount_usd))
        elif has_csv_disc:
            disc_val = csv_disc_total
        else:
            disc_val = 0.0

        if net_usd is not None:
            net_val = max(0.0, float(net_usd))
        elif has_csv_net:
            net_val = csv_net_total
        else:
            net_val = round(max(0.0, gross_total - disc_val), 2)

        return {
            "tier": "billing_csv",
            "gross_consumed_usd": round(gross_total, 2),
            "discount_allowance_usd": round(disc_val, 2),
            "net_billable_usd": round(net_val, 2),
            "discount_pct": round((disc_val / gross_total * 100.0) if gross_total > 0 else 0.0, 2),
            "by_repository": by_repo,
        }

    raise ValueError(f"Unrecognized billing CSV columns: {list(first_row.keys())}")


def estimate_from_repo_telemetry(
    workflows_telemetry: Sequence[dict[str, Any]],
    *,
    plan: str = "free",
    public_repo: bool = False,
    public_larger_runners_free: bool = True,
    daily_git_clones: float | None = None,
    baseline_daily_git_clones: float | None = None,
    active_cache_gb: float = 0.0,
    storage_gb: float = 0.0,
) -> dict[str, Any]:
    """Tier 3: Reconstruct CI/CD cost and KPIs from read-only repository telemetry.

    Enforces Lessons Learned 1, 2, 5, and 6:
      1. Groups strictly per workflow (`workflow_name`), never multiplying total repo runs
         by the primary CI workflow's duration.
      2. Filters out skipped / 0-second gatekeeper runs before averaging workflow duration,
         and clamps `0 <= skipped_runs <= monthly_runs`.
      3. Separates GitHub-hosted SKUs, third-party microVM runners (`blacksmith-*`, etc.),
         and self-hosted runners (with explicit cloud-equivalent estimate and warning).
      4. Computes sharding setup duplication tax, critical-path wall time (across active jobs only),
         and clone-to-run ratio.
    """
    per_workflow_summary: dict[str, dict[str, Any]] = {}
    github_minutes_by_sku: dict[str, float] = {}
    third_party_minutes_by_sku: dict[str, float] = {}
    self_hosted_minutes_by_sku: dict[str, float] = {}
    third_party_cost_usd = 0.0
    self_hosted_cloud_equiv_usd = 0.0
    total_monthly_triggered_runs = 0.0
    total_monthly_active_runs = 0.0
    total_monthly_skipped_runs = 0.0
    total_monthly_billed_minutes = 0.0
    total_shard_setup_duplication_minutes_per_run = 0.0
    intercepts_cache = False

    for wf in workflows_telemetry:
        wf_name = str(wf.get("workflow_name") or wf.get("name") or "Unnamed Workflow")
        raw_monthly = wf.get("monthly_runs", wf.get("daily_runs", 0.0) * 30.0)
        monthly_runs = max(0.0, float(raw_monthly))
        if "monthly_skipped_runs" in wf:
            raw_skipped = float(wf.get("monthly_skipped_runs", 0.0))
        else:
            raw_skipped = float(wf.get("skipped_runs_pct", 0.0)) * monthly_runs
        skipped_runs = min(monthly_runs, max(0.0, raw_skipped))
        active_runs = max(0.0, monthly_runs - skipped_runs)

        jobs = wf.get("jobs", [])
        wf_billed_min_per_active_run = 0.0
        wf_critical_path_wall_min = max(0.0, float(wf.get("critical_path_wall_min", 0.0)))
        wf_shard_tax_min_per_run = 0.0
        active_job_wall_mins: list[float] = []

        for job in jobs:
            if str(job.get("conclusion", "success")).lower() == "skipped":
                continue
            dur_s = max(
                0.0,
                float(
                    job.get(
                        "duration_s",
                        job.get("duration_seconds", job.get("seconds", float(job.get("duration_min", 0.0)) * 60.0)),
                    )
                ),
            )
            if dur_s <= 1.0 and bool(job.get("is_gatekeeper_skip", False)):
                continue
            shards = max(0, int(job.get("shards", job.get("count", job.get("jobs", 1)))))
            if shards <= 0:
                continue
            job_ceil_min = ceil_job_minutes(dur_s)
            active_job_wall_mins.append(dur_s / 60.0)
            setup_min_per_shard = max(0.0, float(job.get("setup_min_per_shard", 0.0)))
            if shards > 1 and setup_min_per_shard > 0.0:
                wf_shard_tax_min_per_run += (shards - 1) * setup_min_per_shard

            total_job_ceil_min_per_run = shards * job_ceil_min
            wf_billed_min_per_active_run += total_job_ceil_min_per_run

            runner_info = classify_runner_label(job.get("labels") or job.get("runs_on") or job.get("sku") or "ubuntu-latest")
            monthly_job_mins = active_runs * total_job_ceil_min_per_run
            if runner_info["intercepts_actions_cache"]:
                intercepts_cache = True

            if runner_info["billing_target"] == "github_hosted":
                sku_key = runner_info["sku"]
                github_minutes_by_sku[sku_key] = github_minutes_by_sku.get(sku_key, 0.0) + monthly_job_mins
            elif runner_info["billing_target"] == "third_party_runner":
                sku_key = runner_info["sku"]
                third_party_minutes_by_sku[sku_key] = third_party_minutes_by_sku.get(sku_key, 0.0) + monthly_job_mins
                third_party_cost_usd += monthly_job_mins * float(runner_info["rate_usd_per_min"])
            elif runner_info["billing_target"] == "self_hosted":
                sku_key = runner_info["sku"]
                self_hosted_minutes_by_sku[sku_key] = self_hosted_minutes_by_sku.get(sku_key, 0.0) + monthly_job_mins
                self_hosted_cloud_equiv_usd += monthly_job_mins * 0.006

        if wf_critical_path_wall_min <= 0.0 and active_job_wall_mins:
            wf_critical_path_wall_min = max(active_job_wall_mins)

        wf_monthly_billed_mins = active_runs * wf_billed_min_per_active_run
        total_monthly_triggered_runs += monthly_runs
        total_monthly_active_runs += active_runs
        total_monthly_skipped_runs += skipped_runs
        total_monthly_billed_minutes += wf_monthly_billed_mins
        total_shard_setup_duplication_minutes_per_run += wf_shard_tax_min_per_run

        per_workflow_summary[wf_name] = {
            "monthly_triggered_runs": round(monthly_runs, 1),
            "monthly_active_runs": round(active_runs, 1),
            "monthly_skipped_runs": round(skipped_runs, 1),
            "billed_min_per_active_run": round(wf_billed_min_per_active_run, 2),
            "critical_path_wall_min": round(wf_critical_path_wall_min, 2),
            "monthly_billed_minutes": round(wf_monthly_billed_mins, 1),
            "shard_setup_duplication_min_per_run": round(wf_shard_tax_min_per_run, 2),
        }

    # Naive wrong calculation (for diagnostic comparison: total triggered runs * max workflow duration)
    max_wf_duration = max(
        (w["billed_min_per_active_run"] for w in per_workflow_summary.values()),
        default=0.0,
    )
    naive_wrong_monthly_minutes = total_monthly_triggered_runs * max_wf_duration

    gh_bill = calculate_monthly_cost(
        github_minutes_by_sku,
        platform="github",
        plan=plan,
        public_repo=public_repo,
        public_larger_runners_free=public_larger_runners_free,
        storage_gb=storage_gb,
        cache_gb=active_cache_gb,
    )

    gross_github_usd = float(gh_bill["list_price_total_usd"])
    net_github_usd = float(gh_bill["total_usd"])
    discount_github_usd = max(0.0, gross_github_usd - net_github_usd)

    daily_active_runs = total_monthly_active_runs / 30.0 if total_monthly_active_runs > 0 else 0.0
    clones_per_active_run = (
        (float(daily_git_clones) / daily_active_runs)
        if (daily_git_clones is not None and daily_active_runs > 0)
        else None
    )
    clone_spike_pct = (
        ((float(daily_git_clones) - float(baseline_daily_git_clones)) / float(baseline_daily_git_clones) * 100.0)
        if (daily_git_clones is not None and baseline_daily_git_clones and baseline_daily_git_clones > 0)
        else None
    )

    total_self_hosted_mins = sum(self_hosted_minutes_by_sku.values())
    unpriced_sh_warning = (
        f"{total_self_hosted_mins:,.1f} self-hosted runner minutes detected ($0.00 GitHub software fee; "
        f"cloud equivalent reference: ${self_hosted_cloud_equiv_usd:,.2f}/mo)."
        if total_self_hosted_mins > 0
        else None
    )

    return {
        "tier": "repo_telemetry",
        "gross_consumed_usd": round(gross_github_usd + third_party_cost_usd, 2),
        "discount_allowance_usd": round(discount_github_usd, 2),
        "net_billable_usd": round(net_github_usd + third_party_cost_usd, 2),
        "github_metered_bill": {
            "gross_consumed_usd": round(gross_github_usd, 2),
            "discount_allowance_usd": round(discount_github_usd, 2),
            "net_billable_usd": round(net_github_usd, 2),
            "minutes_by_sku": github_minutes_by_sku,
        },
        "third_party_runner_bill": {
            "net_billable_usd": round(third_party_cost_usd, 2),
            "minutes_by_sku": third_party_minutes_by_sku,
            "intercepts_actions_cache": intercepts_cache,
        },
        "self_hosted_runner_bill": {
            "net_billable_usd": 0.0,
            "cloud_equivalent_usd": round(self_hosted_cloud_equiv_usd, 2),
            "minutes_by_sku": self_hosted_minutes_by_sku,
            "unpriced_self_hosted_warning": unpriced_sh_warning,
        },
        "kpis": {
            "monthly_triggered_runs": round(total_monthly_triggered_runs, 1),
            "monthly_active_runs": round(total_monthly_active_runs, 1),
            "monthly_skipped_runs": round(total_monthly_skipped_runs, 1),
            "monthly_billed_minutes": round(total_monthly_billed_minutes, 1),
            "naive_aggregate_minutes_avoided": round(naive_wrong_monthly_minutes, 1),
            "overestimation_avoided_pct": round(
                ((naive_wrong_monthly_minutes - total_monthly_billed_minutes) / naive_wrong_monthly_minutes * 100.0)
                if naive_wrong_monthly_minutes > 0
                else 0.0,
                2,
            ),
            "shard_setup_duplication_min_per_run": round(total_shard_setup_duplication_minutes_per_run, 2),
            "clones_per_active_run": round(clones_per_active_run, 2) if clones_per_active_run is not None else None,
            "daily_git_clone_spike_pct": round(clone_spike_pct, 2) if clone_spike_pct is not None else None,
        },
        "per_workflow": per_workflow_summary,
    }
