"""Canonical sourced CI pricing loader and CSV exporter (`pricing/<platform>.json` & `.csv`).

Single source of truth for CI runner, plan, storage, and cache rates across `engine/`.

CLI usage:
  python3 -m engine.pricing --sync-csv
  python3 -m engine.pricing --check
"""
from __future__ import annotations

import argparse
import csv
import io
import json
import math
import os
import re
import sys
import time
import shutil
import subprocess
import urllib.request
from pathlib import Path
from typing import Any, Iterable, Sequence

ROOT = Path(__file__).resolve().parent.parent
PRICING_DIR = ROOT / "pricing"
INFRACOST_BIN = shutil.which("infracost") or "/opt/homebrew/bin/infracost"
INFRACOST_GRAPHQL_URL = "https://pricing.api.infracost.io/graphql"

PLATFORMS = tuple(
    p
    for p in ("github", "gitlab", "circleci", "bitbucket", "aws", "gcp", "azure")
    if (PRICING_DIR / f"{p}.json").exists()
) or ("github", "gitlab")

CSV_COLUMNS = (
    "platform",
    "sku",
    "os",
    "arch",
    "cores",
    "gpu",
    "unit",
    "usd",
    "plan",
    "included_qty",
    "uses_allowance",
    "rounding",
    "source_url",
    "checked_on",
    "status",
)

STANDARD_OS_ORDER = ("linux", "windows", "macos")
ENGINE_PLAN_ORDER = ("free", "team", "enterprise")
LARGER_LINUX_ENGINE_KEYS = ("linux_4", "linux_8", "linux_16", "linux_32", "linux_64")


def pricing_json_path(platform: str = "github") -> Path:
    return PRICING_DIR / f"{platform}.json"


def pricing_csv_path(platform: str = "github") -> Path:
    return PRICING_DIR / f"{platform}.csv"


def load_platform_pricing(platform: str = "github") -> dict:
    path = pricing_json_path(platform)
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    validate_platform_pricing(data, expected_platform=platform)
    return data


def validate_platform_pricing(data: dict, expected_platform: str | None = None) -> None:
    if not isinstance(data, dict):
        raise ValueError("pricing data must be a JSON object")
    platform = data.get("platform")
    if expected_platform and platform != expected_platform:
        raise ValueError(f"expected platform={expected_platform!r}, got {platform!r}")
    for req_top in ("platform", "source_label", "source_url", "checked_on", "plans", "skus"):
        if req_top not in data:
            raise ValueError(f"pricing/{platform}.json missing top-level key {req_top!r}")
    skus = data["skus"]
    if not isinstance(skus, list) or not skus:
        raise ValueError(f"pricing/{platform}.json must have a non-empty 'skus' list")
    seen_skus = set()
    for idx, row in enumerate(skus):
        missing = [c for c in CSV_COLUMNS if c not in row]
        if missing:
            raise ValueError(f"pricing/{platform}.json sku[{idx}] missing columns: {missing}")
        if list(row.keys())[: len(CSV_COLUMNS)] != list(CSV_COLUMNS):
            raise ValueError(
                f"pricing/{platform}.json sku[{idx}] ({row.get('sku')}) first 15 keys must be in exact order {CSV_COLUMNS}"
            )
        if row["platform"] != platform:
            raise ValueError(f"pricing/{platform}.json sku[{idx}] has platform={row['platform']!r}")
        sku_id = row["sku"]
        if sku_id in seen_skus:
            raise ValueError(f"pricing/{platform}.json duplicate sku {sku_id!r}")
        seen_skus.add(sku_id)
        if not isinstance(row["gpu"], bool):
            raise ValueError(f"pricing/{platform}.json sku {sku_id!r} gpu must be bool")
        if not isinstance(row["uses_allowance"], bool):
            raise ValueError(f"pricing/{platform}.json sku {sku_id!r} uses_allowance must be bool")
        if not isinstance(row["cores"], int):
            raise ValueError(f"pricing/{platform}.json sku {sku_id!r} cores must be int")
        if not isinstance(row["usd"], (int, float)):
            raise ValueError(f"pricing/{platform}.json sku {sku_id!r} usd must be numeric")
        if not str(row["source_url"]).startswith("https://"):
            raise ValueError(f"pricing/{platform}.json sku {sku_id!r} invalid source_url")


def _format_csv_val(val) -> str:
    if isinstance(val, bool):
        return "true" if val else "false"
    if isinstance(val, float):
        return format(val, "g")
    return str(val)


def render_csv(platform: str = "github") -> str:
    data = load_platform_pricing(platform)
    buf = io.StringIO(newline="")
    writer = csv.writer(buf, lineterminator="\n")
    writer.writerow(CSV_COLUMNS)
    for row in data["skus"]:
        writer.writerow([_format_csv_val(row[col]) for col in CSV_COLUMNS])
    return buf.getvalue()


def export_csv(platform: str = "github") -> Path:
    content = render_csv(platform)
    out = pricing_csv_path(platform)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(content, encoding="utf-8")
    return out


def verify_csv_sync(platform: str = "github") -> bool:
    expected = render_csv(platform)
    csv_path = pricing_csv_path(platform)
    if not csv_path.exists():
        raise AssertionError(f"missing CSV file: {csv_path}")
    actual = csv_path.read_text(encoding="utf-8")
    if actual != expected:
        raise AssertionError(f"{csv_path} is out of sync with {pricing_json_path(platform)}")
    reader = list(csv.reader(io.StringIO(actual)))
    if not reader or tuple(reader[0]) != CSV_COLUMNS:
        raise AssertionError(f"{csv_path} header {reader[0] if reader else None} != {CSV_COLUMNS}")
    return True


def standard_hosted_os(platform: str = "github") -> tuple[str, ...]:
    data = load_platform_pricing(platform)
    found = []
    for row in data["skus"]:
        ek = row.get("engine_key")
        if ek in STANDARD_OS_ORDER and row.get("uses_allowance"):
            found.append(ek)
    return tuple(k for k in STANDARD_OS_ORDER if k in found)


def larger_linux_rates(platform: str = "github") -> dict[str, float]:
    data = load_platform_pricing(platform)
    by_key = {}
    for row in data["skus"]:
        ek = row.get("engine_key")
        if ek in LARGER_LINUX_ENGINE_KEYS:
            by_key[ek] = float(row["usd"])
    return {k: by_key[k] for k in LARGER_LINUX_ENGINE_KEYS if k in by_key}


def standard_runner_rates(platform: str = "github") -> dict[str, float]:
    data = load_platform_pricing(platform)
    by_key = {}
    for row in data["skus"]:
        ek = row.get("engine_key")
        if ek in STANDARD_OS_ORDER:
            by_key[ek] = float(row["usd"])
    return {k: by_key[k] for k in STANDARD_OS_ORDER if k in by_key}


def plan_included_minutes(platform: str = "github", plans: tuple[str, ...] | None = ENGINE_PLAN_ORDER) -> dict[str, int]:
    data = load_platform_pricing(platform)
    plans_dict = data["plans"]
    if plans is None:
        return {k: int(v["included_minutes"]) for k, v in plans_dict.items()}
    return {k: int(plans_dict[k]["included_minutes"]) for k in plans if k in plans_dict}


def default_engine_rates(platform: str = "github") -> dict:
    std_rates = standard_runner_rates(platform)
    larger_rates = larger_linux_rates(platform)
    std_os = list(standard_hosted_os(platform))
    inc_min = plan_included_minutes(platform, plans=ENGINE_PLAN_ORDER)
    return {
        "runner_os": "linux",
        "runner_usd_per_min": {
            **std_rates,
            **larger_rates,
        },
        "included_minutes_apply": std_os,
        "plan": "team",
        "included_minutes": inc_min,
        "self_hosted_usd_per_month": None,
        "public_repo": False,
        "developer_usd_per_hour": 40,
        "productivity_loss": 0.4,
        "dev_rate_sensitivity": [25, 40, 75, 100],
        "map_rebuilds_per_week": 7,
        "trace_overhead": 0.33,
        "fast_selection_pct": 10,
        "weeks_per_year": 52,
    }


def default_email_rates(platform: str = "github") -> dict:
    std_rates = standard_runner_rates(platform)
    return {
        "runner_os": "linux",
        "runner_usd_per_min": dict(std_rates),
        "public_repo": False,
        "developer_usd_per_hour": 40,
        "productivity_loss": 0.4,
        "weeks_per_year": 52,
    }


HOURS_PER_MONTH = 744.0
INSIDE_ALLOWANCE_NOTE = "$0 compute saved (usage is within plan's included minutes allowance)"
PUBLIC_REPO_NOTE = "$0 compute saved (public repository: standard hosted runners are free; list-price reference shown)"

SKU_ALIASES = {
    "github": {
        "ubuntu-latest": "linux",
        "ubuntu-24.04": "linux",
        "ubuntu-22.04": "linux",
        "Actions Linux": "linux",
        "windows-latest": "windows",
        "macos-latest": "macos",
        "ubuntu-slim": "actions_linux_slim",
        "linux_slim": "actions_linux_slim",
        "linux_2": "linux",
        "linux_96": "linux_x64_96core",
        "linux_arm64": "linux_arm64_2core",
        "windows_2": "windows",
        "windows_4": "windows_x64_4core",
        "windows_8": "windows_x64_8core",
        "windows_16": "windows_x64_16core",
        "windows_32": "windows_x64_32core",
        "windows_64": "windows_x64_64core",
        "windows_96": "windows_x64_96core",
        "windows_arm64": "windows_arm64_2core",
        "macos_m1": "macos",
        "macos_intel_12": "macos_12core_intel",
        "macos_m2pro": "macos_large_m2pro",
        "gpu_linux": "linux_gpu_t4_4core",
        "gpu_windows": "windows_gpu_t4_4core",
        "blacksmith-2vcpu-ubuntu-2204": "blacksmith_2vcpu",
        "blacksmith-4vcpu-ubuntu-2204": "blacksmith_4vcpu",
        "blacksmith-8vcpu-ubuntu-2204": "blacksmith_8vcpu",
        "blacksmith-16vcpu-ubuntu-2204": "blacksmith_16vcpu",
        "blacksmith-32vcpu-ubuntu-2204": "blacksmith_32vcpu",
    },
    "gitlab": {
        "linux_2": "linux",
        "linux_small": "linux_x64_small",
        "linux_medium": "linux_x64_medium",
        "linux_large": "linux_x64_large",
        "linux_xlarge": "linux_x64_xlarge",
        "linux_2xlarge": "linux_x64_2xlarge",
        "gpu_linux": "linux_gpu_standard",
    },
    "aws": {
        "general1.small": "BUILD_GENERAL1_SMALL",
        "general1.medium": "BUILD_GENERAL1_MEDIUM",
        "general1.large": "BUILD_GENERAL1_LARGE",
        "general1.xlarge": "BUILD_GENERAL1_XLARGE",
        "general1.2xlarge": "BUILD_GENERAL1_2XLARGE",
        "arm1.small": "BUILD_ARM1_SMALL",
        "arm1.medium": "BUILD_ARM1_MEDIUM",
        "arm1.large": "BUILD_ARM1_LARGE",
        "windows1.medium": "WINDOWS_MEDIUM",
        "windows1.large": "WINDOWS_LARGE",
        "lambda.arm.1GB": "BUILD_LAMBDA_ARM_1GB",
        "lambda.x86-64.1GB": "BUILD_LAMBDA_X86_1GB",
    },
    "gcp": {
        "default": "e2-standard-2",
        "n2d-highmem-2": "private-n2d-highmem-2",
        "n2d-highmem-8": "private-n2d-highmem-8",
        "c3d-standard-8": "private-c3d-standard-8",
        "t2d-standard-8": "private-t2d-standard-8",
    },
    "azure": {
        "ms_hosted": "ms_hosted_parallel_job",
        "self_hosted_slot": "self_hosted_parallel_job",
        "Standard_D2ads_v5": "mdp_Standard_D2ads_v5",
        "Standard_D4ads_v5": "mdp_Standard_D4ads_v5",
    },
}


def ceil_job_minutes(duration_s: float | int) -> int:
    """Round a single job's duration in seconds up to a whole minute."""
    if duration_s <= 0:
        return 0
    return math.ceil(max(float(duration_s), 1.0) / 60.0)


def ceil_job_duration_minutes(duration_min: float | int) -> int:
    """Round a single job's duration in minutes up to a whole minute."""
    if duration_min <= 0:
        return 0
    return math.ceil(float(duration_min))


def billed_minutes_for_jobs(
    jobs: Iterable[float | int | dict],
    unit: str = "minutes",
    *,
    rounding: str = "ceil_minute_per_job",
) -> int | float:
    """Sum per-job billed minutes for an iterable of job durations or job dicts.

    Supports provider/SKU-specific rounding rules:
      - `ceil_minute_per_job` (default, GitHub Actions & AWS CodeBuild EC2):
        rounds each job's duration up to a whole minute (`math.ceil(max(1.0, sec) / 60.0)`).
      - `exact_seconds` (GitLab CI, CircleCI, GCP Cloud Build, AWS CodeBuild Lambda):
        bills exact seconds (`sec / 60.0`) with zero per-job ceiling penalty.
      - `ceil_minute_per_step` (Bitbucket Pipelines):
        divides job duration across `steps` / `steps_per_job` and ceil-rounds each step to a whole minute.
    """
    total_float = 0.0
    total_int = 0
    used_exact = rounding == "exact_seconds"

    for item in jobs:
        if isinstance(item, dict):
            if str(item.get("conclusion", "success")).lower() == "skipped":
                continue
            raw_count = item.get("shards", item.get("count", item.get("jobs", 1)))
            count = max(0, int(raw_count))
            if count <= 0:
                continue
            item_rounding = str(item.get("rounding") or rounding)
            if item_rounding == "exact_seconds":
                used_exact = True
            steps = max(1, int(item.get("steps", item.get("steps_per_job", 1))))

            if "duration_s" in item or "duration_seconds" in item or "seconds" in item:
                sec = max(0.0, float(item.get("duration_s", item.get("duration_seconds", item.get("seconds", 0.0)))))
                if sec <= 1.0 and bool(item.get("is_gatekeeper_skip", False)):
                    continue
                if item_rounding == "exact_seconds":
                    val_min = sec / 60.0
                    total_float += count * val_min
                elif item_rounding == "ceil_minute_per_step":
                    sec_per_step = (sec / steps) if sec > 0 else 0.0
                    step_mins = steps * ceil_job_minutes(sec_per_step) if sec > 0 else 0
                    total_int += count * step_mins
                    total_float += count * step_mins
                else:
                    cmin = ceil_job_minutes(sec)
                    total_int += count * cmin
                    total_float += count * cmin
            elif "duration_min" in item or "minutes" in item:
                mins = max(0.0, float(item.get("duration_min", item.get("minutes", 0.0))))
                if mins * 60.0 <= 1.0 and bool(item.get("is_gatekeeper_skip", False)):
                    continue
                if item_rounding == "exact_seconds":
                    total_float += count * mins
                elif item_rounding == "ceil_minute_per_step":
                    sec = mins * 60.0
                    sec_per_step = (sec / steps) if sec > 0 else 0.0
                    step_mins = steps * ceil_job_minutes(sec_per_step) if sec > 0 else 0
                    total_int += count * step_mins
                    total_float += count * step_mins
                else:
                    cmin = ceil_job_duration_minutes(mins)
                    total_int += count * cmin
                    total_float += count * cmin
            else:
                raise ValueError(f"job dict missing duration_s or duration_min: {item!r}")
        else:
            val = max(0.0, float(item))
            if unit in ("s", "sec", "seconds"):
                if rounding == "exact_seconds":
                    total_float += val / 60.0
                else:
                    cmin = ceil_job_minutes(val)
                    total_int += cmin
                    total_float += cmin
            elif unit in ("m", "min", "minutes"):
                if rounding == "exact_seconds":
                    total_float += val
                else:
                    cmin = ceil_job_duration_minutes(val)
                    total_int += cmin
                    total_float += cmin
            else:
                raise ValueError(f"unknown duration unit {unit!r}")

    return round(total_float, 6) if used_exact else total_int


THIRD_PARTY_GITHUB_RUNNER_SKUS: tuple[dict[str, Any], ...] = (
    {
        "platform": "github",
        "sku": "blacksmith_2vcpu",
        "engine_key": "blacksmith_2vcpu",
        "os": "linux",
        "arch": "x64",
        "cores": 2,
        "gpu": False,
        "unit": "minute",
        "usd": 0.004,
        "plan": "all",
        "included_qty": 0,
        "uses_allowance": False,
        "rounding": "ceil_minute_per_job",
        "source_url": "https://www.blacksmith.sh/pricing",
        "checked_on": "2026-11-12",
        "status": "third_party_runner",
    },
    {
        "platform": "github",
        "sku": "blacksmith_4vcpu",
        "engine_key": "blacksmith_4vcpu",
        "os": "linux",
        "arch": "x64",
        "cores": 4,
        "gpu": False,
        "unit": "minute",
        "usd": 0.008,
        "plan": "all",
        "included_qty": 0,
        "uses_allowance": False,
        "rounding": "ceil_minute_per_job",
        "source_url": "https://www.blacksmith.sh/pricing",
        "checked_on": "2026-11-12",
        "status": "third_party_runner",
    },
    {
        "platform": "github",
        "sku": "blacksmith_8vcpu",
        "engine_key": "blacksmith_8vcpu",
        "os": "linux",
        "arch": "x64",
        "cores": 8,
        "gpu": False,
        "unit": "minute",
        "usd": 0.016,
        "plan": "all",
        "included_qty": 0,
        "uses_allowance": False,
        "rounding": "ceil_minute_per_job",
        "source_url": "https://www.blacksmith.sh/pricing",
        "checked_on": "2026-11-12",
        "status": "third_party_runner",
    },
    {
        "platform": "github",
        "sku": "blacksmith_16vcpu",
        "engine_key": "blacksmith_16vcpu",
        "os": "linux",
        "arch": "x64",
        "cores": 16,
        "gpu": False,
        "unit": "minute",
        "usd": 0.032,
        "plan": "all",
        "included_qty": 0,
        "uses_allowance": False,
        "rounding": "ceil_minute_per_job",
        "source_url": "https://www.blacksmith.sh/pricing",
        "checked_on": "2026-11-12",
        "status": "third_party_runner",
    },
    {
        "platform": "github",
        "sku": "blacksmith_32vcpu",
        "engine_key": "blacksmith_32vcpu",
        "os": "linux",
        "arch": "x64",
        "cores": 32,
        "gpu": False,
        "unit": "minute",
        "usd": 0.064,
        "plan": "all",
        "included_qty": 0,
        "uses_allowance": False,
        "rounding": "ceil_minute_per_job",
        "source_url": "https://www.blacksmith.sh/pricing",
        "checked_on": "2026-11-12",
        "status": "third_party_runner",
    },
)


def get_sku_map(platform: str | dict = "github") -> dict[str, dict]:
    """Return a lookup dict mapping SKU ids, engine_keys, and aliases to their canonical SKU row."""
    data = load_platform_pricing(platform) if isinstance(platform, str) else platform
    plat = data["platform"]
    by_id: dict[str, dict] = {}
    for row in data["skus"]:
        by_id[row["sku"]] = row
        ek = row.get("engine_key")
        if ek and ek not in by_id:
            by_id[ek] = row
    if plat == "github":
        for tp_row in THIRD_PARTY_GITHUB_RUNNER_SKUS:
            by_id[tp_row["sku"]] = tp_row
    for alias, target in SKU_ALIASES.get(plat, {}).items():
        if target in by_id and alias not in by_id:
            by_id[alias] = by_id[target]
    return by_id


def resolve_sku_via_ollama(
    raw_label: str,
    platform: str = "github",
    model: str = "qwen2.5-coder:7b",
    ollama_url: str = "http://127.0.0.1:11434/api/generate",
) -> dict[str, Any] | None:
    """Use local Ollama LLM to classify an unrecognized runner label into a canonical SKU."""
    sku_map = get_sku_map(platform)
    valid_skus = sorted({r.get("engine_key") or r["sku"] for r in sku_map.values()})
    prompt = (
        f"You are a CI/CD FinOps SKU classifier for platform={platform!r}.\n"
        f"Map the runner label {raw_label!r} to the single closest SKU from this exact list:\n"
        f"{valid_skus}\n"
        'Respond ONLY with valid JSON: {"sku": "<chosen_sku>", "confidence": 0.95, "reason": "<short reason>"}'
    )
    try:
        req = urllib.request.Request(
            ollama_url,
            data=json.dumps({"model": model, "prompt": prompt, "stream": False, "format": "json"}).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=20) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            parsed = json.loads(body.get("response", "{}"))
            chosen = parsed.get("sku")
            if chosen and chosen in sku_map:
                return {
                    "row": sku_map[chosen],
                    "resolution_mode": "ai_ollama_fallback",
                    "confidence": float(parsed.get("confidence", 0.8)),
                    "reason": str(parsed.get("reason", "Mapped by local Ollama LLM")),
                }
    except Exception:
        return None
    return None


LEARNED_SKU_REGISTRY_PATH = PRICING_DIR / "learned_sku_registry.json"
_LEARNED_SKU_CACHE: dict[str, dict[str, Any]] | None = None


def load_learned_sku_registry(*, force_reload: bool = False) -> dict[str, dict[str, Any]]:
    """Load the persistent learned SKU alias registry (`pricing/learned_sku_registry.json`) with in-memory caching."""
    global _LEARNED_SKU_CACHE
    if _LEARNED_SKU_CACHE is not None and not force_reload:
        return _LEARNED_SKU_CACHE
    if not LEARNED_SKU_REGISTRY_PATH.exists():
        _LEARNED_SKU_CACHE = {}
        return _LEARNED_SKU_CACHE
    try:
        _LEARNED_SKU_CACHE = json.loads(LEARNED_SKU_REGISTRY_PATH.read_text(encoding="utf-8"))
    except Exception:
        _LEARNED_SKU_CACHE = {}
    return _LEARNED_SKU_CACHE


def save_learned_sku_entry(platform: str, raw_label: str, meta: dict[str, Any]) -> None:
    """Persist a newly discovered runner label mapping into `pricing/learned_sku_registry.json`."""
    reg = load_learned_sku_registry()
    plat_reg = reg.setdefault(platform, {})
    if raw_label not in plat_reg or plat_reg[raw_label].get("resolved_sku") != meta.get("resolved_sku"):
        plat_reg[raw_label] = meta
        try:
            LEARNED_SKU_REGISTRY_PATH.write_text(
                json.dumps(reg, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
        except OSError:
            # Read-only filesystem or sandboxed environment: keep in-memory cache without crashing
            pass


def _normalized_rate_per_min(row: dict[str, Any]) -> float:
    """Return a SKU row's effective per-minute USD rate for metadata display."""
    if row.get("sku") == "self_hosted":
        return float(row.get("effective_usd", 0.0))
    raw_unit_rate = float(row.get("usd", 0.0))
    sku_unit = str(row.get("unit", "minute")).lower()
    if sku_unit == "hour":
        return round(raw_unit_rate / 60.0, 10)
    if sku_unit == "second":
        return round(raw_unit_rate * 60.0, 10)
    if sku_unit == "1000_compute_minutes":
        return round(raw_unit_rate / 1000.0, 10)
    return round(raw_unit_rate, 10)


def resolve_sku_row(
    raw_key: str,
    platform: str = "github",
    sku_map: dict[str, dict] | None = None,
    *,
    allow_heuristic: bool = True,
    use_ai_fallback: bool = False,
    persist_learned: bool = True,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """4-Stage SKU Resolver with Persistent Learned Registry:
      Stage 1A: Exact catalog / alias lookup (`exact_catalog`)
      Stage 1B: Persistent Learned Registry lookup (`pricing/learned_sku_registry.json`)
      Stage 2:  Deterministic regex/pattern extraction (`heuristic_pattern`) -> auto-saved to registry
      Stage 3:  Local Ollama AI fallback (`ai_ollama_fallback`) -> auto-saved to registry
    """
    smap = sku_map if sku_map is not None else get_sku_map(platform)
    if raw_key in smap:
        row = smap[raw_key]
        return row, {
            "raw_label": raw_key,
            "resolved_sku": row.get("engine_key") or row["sku"],
            "resolution_mode": "exact_catalog",
            "rate_usd_per_min": _normalized_rate_per_min(row),
        }

    # Stage 1B: Check Persistent Learned Registry
    learned = load_learned_sku_registry().get(platform, {}).get(raw_key)
    if learned and learned.get("resolved_sku") in smap:
        row = smap[learned["resolved_sku"]]
        cached_meta = dict(learned)
        cached_meta["resolution_mode"] = "learned_registry_cache"
        return row, cached_meta

    if not allow_heuristic:
        raise ValueError(f"unknown SKU {raw_key!r} for platform {platform!r}")

    low = str(raw_key).strip().lower()

    # Stage 2A: Self-hosted / Kubernetes ARC / custom runner pools without cloud size
    if any(tok in low for tok in ("self-hosted", "self_hosted", "arc-runner", "k8s-runner", "actuated")):
        if "self_hosted" in smap:
            row = smap["self_hosted"]
            meta = {
                "raw_label": raw_key,
                "resolved_sku": row.get("engine_key") or row["sku"],
                "resolution_mode": "heuristic_pattern",
                "rate_usd_per_min": float(row.get("effective_usd", 0.0)),
                "reason": "Matched self-hosted / Kubernetes ARC runner pattern",
            }
            if persist_learned:
                save_learned_sku_entry(platform, raw_key, meta)
            return row, meta

    # Detect OS, Arch, GPU
    is_win = any(w in low for w in ("windows", "win-", "win_", "win10", "win11", "win20"))
    is_mac = any(m in low for m in ("macos", "osx", "mac-"))
    os_name = "windows" if is_win else ("macos" if is_mac else "linux")
    is_arm = any(a in low for a in ("arm64", "aarch64", "_arm", "-arm", "m1", "m2", "graviton"))
    arch = "arm64" if is_arm else "x64"
    is_gpu = any(g in low for g in ("gpu", "t4", "a10", "l4", "cuda"))
    has_xlarge = bool(re.search(r"(?:^|[-_])xlarge(?:$|[-_])", low))
    has_2xlarge = bool(re.search(r"(?:^|[-_])2xlarge(?:$|[-_])", low))
    has_large = bool(re.search(r"(?:^|[-_])large(?:$|[-_])", low))
    has_medium = bool(re.search(r"(?:^|[-_])medium(?:$|[-_])", low))
    has_small = bool(re.search(r"(?:^|[-_])small(?:$|[-_])", low))

    # Strip canonical OS release versions BEFORE extracting bare `-(\d+)` core counts so that
    # `windows-2022`, `windows-2025`, `windows-11-arm64`, `macos-14`, `macos-15`, and `ubuntu-24.04`
    # never misparse OS version numbers as CPU core counts.
    cleaned_for_cores = re.sub(
        r"(?:^|(?<=[-_ ]))(?:"
        r"ubuntu[-_ ]?(?:latest|(?:18|2\d)\.?\d{2})"
        r"|windows[-_ ]?(?:latest|20\d{2}|1[01])"
        r"|macos[-_ ]?(?:latest|1[1-9]|2\d)"
        r")(?=$|[-_ ])",
        lambda m: m.group(0).split("-")[0].split("_")[0].split(" ")[0],
        low,
    )

    # Extract core count from patterns like: `-8-core`, `-8core`, `-8vcpu`, `-x86_64-4`, `-16x`, `8cpu`
    core_match = (
        re.search(r"(?:^|[-_=])(\d+)[-_ ]*(?:v?cpus?|cores?)(?:$|[-_])", cleaned_for_cores)
        or re.search(r"-(\d+)x(?:$|[-_])", cleaned_for_cores)
        or re.search(
            r"(?:x86_64|x64|arm64|aarch64|ubuntu|linux|windows|macos)-(\d+)(?:$|[-_])",
            cleaned_for_cores,
        )
    )
    if core_match:
        cores = int(core_match.group(1))
    elif is_mac and (has_xlarge or has_large):
        cores = 12
    elif has_2xlarge:
        cores = 16
    elif has_xlarge:
        cores = 8
    elif has_large:
        cores = 4
    else:
        cores = 2

    # Stage 2B: Third-party microVM providers (blacksmith, warpbuild, buildjet, depot, namespace, runs-on)
    if platform == "github" and any(
        p in low for p in ("blacksmith", "warpbuild", "buildjet", "depot", "namespace", "runs-on")
    ):
        snapped = min((2, 4, 8, 16, 32), key=lambda c: abs(c - cores))
        target_key = f"blacksmith_{snapped}vcpu"
        if target_key in smap:
            row = smap[target_key]
            meta = {
                "raw_label": raw_key,
                "resolved_sku": target_key,
                "resolution_mode": "heuristic_pattern",
                "rate_usd_per_min": _normalized_rate_per_min(row),
                "reason": f"Matched 3rd-party microVM runner pattern ({snapped} vCPU)",
            }
            if persist_learned:
                save_learned_sku_entry(platform, raw_key, meta)
            return row, meta

    has_known_heuristic_signal = bool(
        core_match
        or is_win
        or is_mac
        or is_arm
        or is_gpu
        or has_small
        or has_medium
        or has_large
        or has_xlarge
        or has_2xlarge
        or any(
            tok in low
            for tok in (
                "ubuntu",
                "linux",
                "slim",
                "runner",
                "docker",
                "general1",
                "arm1",
                "lambda",
                "e2-",
                "n2d-",
                "c3d-",
                "t2d-",
                "standard_",
                "mdp_",
            )
        )
    )

    if has_known_heuristic_signal:
        row = None
        if platform == "github":
            if is_gpu:
                gk = "windows_gpu_t4_4core" if is_win else "linux_gpu_t4_4core"
                row = smap.get(gk)
            elif is_mac:
                if has_xlarge or (cores >= 12 and is_arm):
                    row = smap.get("macos_large_m2pro")
                elif has_large or cores >= 12:
                    row = smap.get("macos_12core_intel")
                else:
                    row = smap.get("macos")
            elif is_win:
                if cores <= 2:
                    row = smap.get("windows_arm64_2core") if is_arm else smap.get("windows")
                else:
                    snapped = min((4, 8, 16, 32, 64, 96), key=lambda c: abs(c - cores))
                    wk = f"windows_{'arm64' if is_arm else 'x64'}_{snapped}core"
                    row = smap.get(wk, smap.get("windows"))
            else:
                if "slim" in low or cores == 1:
                    row = smap.get("actions_linux_slim")
                elif cores <= 2 and not is_arm and not has_large and not has_xlarge:
                    row = smap.get("linux")
                elif cores <= 2 and is_arm:
                    row = smap.get("linux_arm64_2core")
                else:
                    snapped = min((4, 8, 16, 32, 64, 96), key=lambda c: abs(c - cores))
                    lk = f"linux_arm64_{snapped}core" if is_arm else f"linux_{snapped}"
                    row = smap.get(lk) or smap.get(f"linux_x64_{snapped}core") or smap.get("linux")
        elif platform == "gitlab":
            if is_gpu:
                row = smap.get("linux_gpu_standard")
            elif is_mac:
                row = smap.get("macos")
            elif is_win:
                row = smap.get("windows")
            else:
                snapped = min((2, 4, 8, 16, 32, 64), key=lambda c: abs(c - cores))
                row = smap.get(f"linux_{snapped}") or smap.get("linux")
        elif platform == "circleci":
            if is_gpu:
                row = smap.get("gpu_nvidia_t4") or smap.get("linux")
            elif is_mac:
                row = smap.get("macos_m2pro_medium") if (has_xlarge or has_large) else smap.get("macos")
            elif is_win:
                row = smap.get("windows")
            elif is_arm:
                row = smap.get("linux_arm64_large") if cores >= 4 else smap.get("linux_arm64_medium")
            else:
                snapped = min((2, 4, 8, 16, 32), key=lambda c: abs(c - cores))
                row = smap.get(f"linux_{snapped}") or smap.get("linux")
        elif platform == "bitbucket":
            snapped = min((2, 4, 8, 16, 32), key=lambda c: abs(c - cores))
            row = smap.get(f"linux_{snapped}") or smap.get("linux")
        elif platform == "aws":
            if "lambda" in low:
                row = smap.get("BUILD_LAMBDA_ARM_1GB") if is_arm else smap.get("BUILD_LAMBDA_X86_1GB")
            elif is_win:
                row = smap.get("WINDOWS_LARGE") if cores >= 8 else smap.get("windows")
            elif is_arm:
                row = smap.get("BUILD_ARM1_LARGE") if cores >= 8 else smap.get("BUILD_ARM1_SMALL")
            else:
                snapped = min((2, 4, 8, 16, 32, 64), key=lambda c: abs(c - cores))
                row = smap.get(f"linux_{snapped}") or smap.get("linux")
        elif platform == "gcp":
            snapped = min((2, 4, 8, 16, 32, 64), key=lambda c: abs(c - cores))
            row = smap.get(f"linux_{snapped}") or smap.get("linux")
        elif platform == "azure":
            if cores >= 4 or "d4" in low or "mdp" in low:
                row = smap.get(f"linux_{min((4, 8, 16, 32), key=lambda c: abs(c - max(4, cores)))}") or smap.get("linux_4")
            else:
                row = smap.get("linux")

        if row is not None:
            meta = {
                "raw_label": raw_key,
                "resolved_sku": row.get("engine_key") or row["sku"],
                "resolution_mode": "heuristic_pattern",
                "rate_usd_per_min": _normalized_rate_per_min(row),
                "reason": f"Parsed os={os_name}, arch={arch}, cores={cores}, gpu={is_gpu}",
            }
            if persist_learned:
                save_learned_sku_entry(platform, raw_key, meta)
            return row, meta

    # Stage 3: Optional Local Ollama AI Fallback
    if use_ai_fallback:
        ai_res = resolve_sku_via_ollama(raw_key, platform=platform)
        if ai_res is not None:
            row = ai_res["row"]
            meta = {
                "raw_label": raw_key,
                "resolved_sku": row.get("engine_key") or row["sku"],
                "resolution_mode": "ai_ollama_fallback",
                "rate_usd_per_min": _normalized_rate_per_min(row),
                "reason": ai_res["reason"],
            }
            if persist_learned:
                save_learned_sku_entry(platform, raw_key, meta)
            return row, meta

    # Stage 4: Final safe fallback to default compute SKU (`linux`) on platform (never a seat/plan SKU!)
    default_row = (
        smap.get("linux")
        or next(
            (r for r in smap.values() if str(r.get("unit", "")).lower() in ("minute", "action_minute", "second", "hour")),
            next(iter(smap.values())),
        )
    )
    return default_row, {
        "raw_label": raw_key,
        "resolved_sku": default_row.get("engine_key") or default_row["sku"],
        "resolution_mode": "default_fallback",
        "rate_usd_per_min": _normalized_rate_per_min(default_row),
        "reason": f"Fallback to default compute SKU for platform={platform}",
    }


def _effective_sku_rounding(platform: str, row: dict[str, Any], rules: dict[str, Any] | None = None) -> str:
    """Determine the canonical rounding rule for a resolved SKU on a given platform."""
    sku_rounding = str(row.get("rounding") or (rules or {}).get("job_rounding") or "")
    if platform in ("gitlab", "circleci", "gcp") or sku_rounding == "exact_seconds":
        return "exact_seconds"
    if platform == "bitbucket" or sku_rounding == "ceil_minute_per_step":
        return "ceil_minute_per_step"
    return "ceil_minute_per_job"


def _extract_sku_minutes(val, *, rounding: str = "ceil_minute_per_job") -> float:
    """Extract monthly billed minutes from a value (number, job list, or audit/job dict)."""
    if val is None:
        return 0.0
    if isinstance(val, (int, float)):
        return max(0.0, float(val))
    if isinstance(val, (list, tuple)):
        return float(billed_minutes_for_jobs(val, unit="minutes", rounding=rounding))
    if isinstance(val, dict):
        if "billed" in val:
            return max(0.0, float(val["billed"]))
        if "job_durations_s" in val:
            return float(billed_minutes_for_jobs(val["job_durations_s"], unit="seconds", rounding=rounding))
        if "job_durations_min" in val:
            return float(billed_minutes_for_jobs(val["job_durations_min"], unit="minutes", rounding=rounding))
        if ("jobs" in val or "count" in val or "shards" in val) and (
            "duration_s" in val or "duration_seconds" in val or "seconds" in val or "duration_min" in val or "minutes" in val
        ):
            return float(billed_minutes_for_jobs([val], rounding=rounding))
    raise ValueError(f"cannot extract billed minutes from {val!r}")


NON_COMPUTE_SKU_UNITS = frozenset(
    {
        "gb_month",
        "gb_hour",
        "10gb_year",
        "100gb_year",
        "100gb_month",
        "user_month",
        "pipeline_month",
        "25000_credits",
        "gb",
        "job",
    }
)


def calculate_monthly_cost(
    minutes_by_sku: dict[str, float | int | dict | list] | None = None,
    *,
    platform: str = "github",
    plan: str = "team",
    seats: int = 0,
    public_repo: bool = False,
    public_larger_runners_free: bool = True,
    storage_gb: float = 0.0,
    cache_gb: float = 0.0,
    egress_gb: float = 0.0,
    dlc_enabled: bool = False,
    dlc_job_runs: int | None = None,
    sku_order: Sequence[str] | None = None,
    allowance_mode: str = "ordered",
    include_self_hosted_announced: bool = False,
    self_hosted_usd_per_month: float | None = None,
    gitlab_pack_rounding: bool = False,
    storage_hours_per_month: float = HOURS_PER_MONTH,
    rate_overrides: dict[str, float] | None = None,
    included_minutes_override: float | None = None,
    allowance_skus_override: Iterable[str] | None = None,
    jobs: Sequence[dict] | None = None,
    workflows: Sequence[dict] | None = None,
) -> dict:
    """Exact monthly CI cost calculator across seats, runner SKUs, allowance, storage, cache, egress, and DLC.

    Formula:
      cost = seats_usd
           + sum_sku(max(0, billed_min_sku - allowance_covered_min_sku) * rate_sku)
           + storage_over_included + cache_over_included + egress_usd + dlc_usd
    """
    data = load_platform_pricing(platform)
    plans_dict = data["plans"]
    rules_dict = data.get("rules", {})
    plan_aliases = {
        "free_tier": "free",
        "default_pool": "free",
        "default": "free",
        "basic": "team",
        "pro": "team",
    }
    if plan not in plans_dict and plan in plan_aliases and plan_aliases[plan] in plans_dict:
        plan = plan_aliases[plan]
    if plan not in plans_dict:
        raise ValueError(f"unknown plan {plan!r} for platform {platform!r} (expected one of {sorted(plans_dict)})")
    plan_info = plans_dict[plan]
    sku_map = get_sku_map(data)

    # Clamp negative storage / cache / egress inputs to 0.0
    storage_gb = max(0.0, float(storage_gb))
    cache_gb = max(0.0, float(cache_gb))
    egress_gb = max(0.0, float(egress_gb))

    # 1. Aggregate per-SKU billed minutes (preserving caller / sku_order)
    aggregated: dict[str, float] = {}
    order_list: list[str] = []
    per_workflow_breakdown: dict[str, dict[str, Any]] = {}
    sku_resolutions: dict[str, dict[str, Any]] = {}
    inferred_job_executions = 0
    t_start_ns = time.perf_counter_ns()

    def _resolve_and_get_rounding(raw_key: str) -> tuple[dict[str, Any], dict[str, Any], str]:
        canonical_row, res_meta = resolve_sku_row(raw_key, platform=platform, sku_map=sku_map)
        sku_resolutions[raw_key] = res_meta
        eff_rounding = _effective_sku_rounding(platform, canonical_row, rules_dict)
        return canonical_row, res_meta, eff_rounding

    def _add_sku_min(raw_key: str, mins: float) -> str:
        if raw_key in ("billed_total", "raw", "raw_total"):
            return raw_key
        canonical_row, _res_meta, _eff_rounding = _resolve_and_get_rounding(raw_key)
        canon_key = canonical_row.get("engine_key") or canonical_row["sku"]
        if canon_key not in sku_map:
            sku_map[canon_key] = canonical_row
        if canon_key not in order_list:
            order_list.append(canon_key)
        aggregated[canon_key] = aggregated.get(canon_key, 0.0) + max(0.0, float(mins))
        return canon_key

    for k, v in (minutes_by_sku or {}).items():
        if k in ("billed_total", "raw", "raw_total"):
            continue
        _row, _meta, eff_rounding = _resolve_and_get_rounding(k)
        _add_sku_min(k, _extract_sku_minutes(v, rounding=eff_rounding))

    for job in (jobs or ()):
        if str(job.get("conclusion", "success")).lower() == "skipped":
            continue
        sku_k = (
            job.get("sku")
            or job.get("os")
            or job.get("runs_on")
            or (job.get("labels", ["linux"])[0] if isinstance(job.get("labels"), (list, tuple)) and job.get("labels") else job.get("labels"))
            or "linux"
        )
        _row, _meta, eff_rounding = _resolve_and_get_rounding(str(sku_k))
        j_mins = float(billed_minutes_for_jobs([job], rounding=eff_rounding))
        if j_mins > 0:
            inferred_job_executions += max(0, int(job.get("shards", job.get("count", job.get("jobs", 1)))))
        _add_sku_min(str(sku_k), j_mins)

    for wf in (workflows or ()):
        wf_name = str(wf.get("workflow_name") or wf.get("name") or "Workflow")
        raw_monthly = wf.get("monthly_runs", wf.get("daily_runs", 0.0) * 30.0)
        monthly_runs = max(0.0, float(raw_monthly))
        if "monthly_skipped_runs" in wf:
            raw_skipped = float(wf.get("monthly_skipped_runs", 0.0))
        else:
            raw_skipped = float(wf.get("skipped_runs_pct", 0.0)) * monthly_runs
        skipped_runs = min(monthly_runs, max(0.0, raw_skipped))
        active_runs = max(0.0, monthly_runs - skipped_runs)
        wf_billed_per_run = 0.0
        wf_sku_mins: dict[str, float] = {}
        wf_shard_tax_per_run = 0.0
        active_job_wall_mins: list[float] = []

        for j in wf.get("jobs", ()):
            if str(j.get("conclusion", "success")).lower() == "skipped":
                continue
            dur_s = float(
                j.get(
                    "duration_s",
                    j.get("duration_seconds", j.get("seconds", float(j.get("duration_min", j.get("minutes", 0.0))) * 60.0)),
                )
            )
            if dur_s <= 1.0 and bool(j.get("is_gatekeeper_skip", False)):
                continue
            raw_labels = j.get("labels")
            first_label = raw_labels[0] if isinstance(raw_labels, (list, tuple)) and raw_labels else (raw_labels if isinstance(raw_labels, str) else None)
            sku_k = str(j.get("sku") or j.get("runs_on") or first_label or "linux")
            _row, _meta, eff_rounding = _resolve_and_get_rounding(sku_k)
            j_ceil_min = float(billed_minutes_for_jobs([j], rounding=eff_rounding))
            if j_ceil_min <= 0.0 and dur_s <= 0.0:
                continue
            shards = max(0, int(j.get("shards", j.get("count", j.get("jobs", 1)))))
            if shards <= 0:
                continue
            setup_min = max(0.0, float(j.get("setup_min_per_shard", 0.0)))
            if shards > 1 and setup_min > 0.0:
                wf_shard_tax_per_run += (shards - 1) * setup_min
            active_job_wall_mins.append(max(0.0, dur_s / 60.0))
            inferred_job_executions += int(round(active_runs * shards))
            wf_billed_per_run += j_ceil_min
            m_mins = active_runs * j_ceil_min
            canon_k = _add_sku_min(sku_k, m_mins)
            wf_sku_mins[canon_k] = wf_sku_mins.get(canon_k, 0.0) + m_mins

        wf_crit_wall = max(0.0, float(wf.get("critical_path_wall_min", 0.0)))
        if wf_crit_wall <= 0.0 and active_job_wall_mins:
            wf_crit_wall = max(active_job_wall_mins)

        per_workflow_breakdown[wf_name] = {
            "monthly_triggered_runs": monthly_runs,
            "monthly_active_runs": active_runs,
            "monthly_skipped_runs": skipped_runs,
            "billed_minutes_per_active_run": wf_billed_per_run,
            "critical_path_wall_min": round(wf_crit_wall, 4),
            "shard_setup_duplication_min_per_run": round(wf_shard_tax_per_run, 4),
            "monthly_billed_minutes": active_runs * wf_billed_per_run,
            "minutes_by_sku": wf_sku_mins,
        }

    if sku_order is not None:
        explicit_order: list[str] = []
        for k in sku_order:
            if k in sku_map:
                ck = sku_map[k].get("engine_key") or sku_map[k]["sku"]
                if ck not in explicit_order:
                    explicit_order.append(ck)
        for ck in order_list:
            if ck not in explicit_order:
                explicit_order.append(ck)
        order_list = explicit_order

    # 2. Seats cost (Azure Basic plan includes first 5 seats free; CircleCI Performance includes $15 base for 5 users)
    seat_rate_usd = float(plan_info.get("seat_usd_per_month", 0.0))
    included_seats = int(plan_info.get("included_seats", 5 if platform == "azure" else 0))
    clamped_seats = max(0, int(seats))
    billable_seats = max(0, clamped_seats - included_seats)
    seats_usd = billable_seats * seat_rate_usd
    list_seats_usd = clamped_seats * seat_rate_usd

    # 3. Allowance pool and per-SKU compute cost
    included_allowance = float(
        included_minutes_override
        if included_minutes_override is not None
        else plan_info.get("included_minutes", 0.0)
    )
    allowance_remaining = max(0.0, included_allowance)
    allowance_override_set = (
        {sku_map[k].get("engine_key") or sku_map[k]["sku"] for k in allowance_skus_override if k in sku_map}
        if allowance_skus_override is not None
        else None
    )

    overrides = dict(rate_overrides or {})
    sku_breakdown: dict[str, dict] = {}
    total_billed_minutes = 0.0
    total_core_minutes = 0.0
    total_allowance_pool_consumed = 0.0
    total_allowance_covered_minutes = 0.0
    total_overage_minutes = 0.0
    total_overage_compute_units = 0.0
    private_compute_usd = 0.0
    list_price_compute_usd = 0.0
    public_billed_compute_usd = 0.0
    self_hosted_billed_minutes = 0.0

    def _sku_multiplier(row: dict[str, Any]) -> float:
        if platform == "gitlab":
            return float(row.get("cost_factor") or plan_info.get("allowance_os_multipliers", {}).get(row["os"], 1))
        return float(row.get("allowance_multiplier") or plan_info.get("allowance_os_multipliers", {}).get(row["os"], 1))

    # Pre-compute total allowance-eligible pool units (weighted by multiplier) for 'blended' mode
    total_eligible_blended_pool = 0.0
    if allowance_mode == "blended":
        for ck in order_list:
            row = sku_map[ck]
            uses_allow = (ck in allowance_override_set) if allowance_override_set is not None else bool(row.get("uses_allowance", False))
            if uses_allow:
                total_eligible_blended_pool += aggregated.get(ck, 0.0) * _sku_multiplier(row)

    for ck in order_list:
        row = sku_map[ck]
        billed_min = max(0.0, float(aggregated.get(ck, 0.0)))
        cores = int(row.get("cores", 0))
        core_min = billed_min * cores
        total_billed_minutes += billed_min
        total_core_minutes += core_min

        is_self_hosted = row["sku"] == "self_hosted"
        if is_self_hosted:
            self_hosted_billed_minutes += billed_min
        sku_unit = str(row.get("unit", "minute")).lower()
        is_parallel_slot = sku_unit == "parallel_slot_month"

        if is_self_hosted:
            rate_usd = float(row["usd"]) if include_self_hosted_announced else float(row.get("effective_usd", 0.0))
        elif ck in overrides:
            rate_usd = float(overrides[ck])
            is_parallel_slot = False
        elif row["sku"] in overrides:
            rate_usd = float(overrides[row["sku"]])
            is_parallel_slot = False
        else:
            if sku_unit in NON_COMPUTE_SKU_UNITS:
                raise ValueError(
                    f"SKU {row['sku']!r} has non-compute unit={sku_unit!r} and cannot be billed as runner compute minutes"
                )
            rate_usd = _normalized_rate_per_min(row)

        uses_allow = (ck in allowance_override_set) if allowance_override_set is not None else bool(row.get("uses_allowance", False))
        multiplier = _sku_multiplier(row)

        if allowance_mode == "ordered":
            if uses_allow and allowance_remaining > 0.0 and billed_min > 0.0:
                pool_needed = billed_min * multiplier
                pool_used = min(allowance_remaining, pool_needed)
                covered_min = pool_used / multiplier
                overage_min = max(0.0, billed_min - covered_min)
                allowance_remaining -= pool_used
            else:
                pool_used = 0.0
                covered_min = 0.0
                overage_min = billed_min
        elif allowance_mode == "blended":
            if uses_allow and total_eligible_blended_pool > 0.0 and billed_min > 0.0:
                pool_needed = billed_min * multiplier
                share = pool_needed / total_eligible_blended_pool
                pool_used = min(pool_needed, included_allowance * share)
                covered_min = pool_used / multiplier
                overage_min = max(0.0, billed_min - covered_min)
                allowance_remaining = max(0.0, included_allowance - min(included_allowance, total_eligible_blended_pool))
            else:
                pool_used = 0.0
                covered_min = 0.0
                overage_min = billed_min
        else:
            raise ValueError(f"unknown allowance_mode {allowance_mode!r} (expected 'ordered' or 'blended')")

        if is_parallel_slot:
            slot_price_usd = float(row["usd"])
            slots_needed = 0 if overage_min <= 0.0 else max(1, math.ceil(billed_min / 7200.0))
            list_slots = 0 if billed_min <= 0.0 else max(1, math.ceil(billed_min / 7200.0))
            sku_private_usd = slots_needed * slot_price_usd
            sku_list_usd = list_slots * slot_price_usd
            rate_usd = round(slot_price_usd / 7200.0, 10)
        else:
            sku_private_usd = overage_min * rate_usd
            sku_list_usd = billed_min * rate_usd

        is_free_on_public_repo = bool(public_repo) and (
            public_larger_runners_free
            or (ck in STANDARD_OS_ORDER and bool(row.get("uses_allowance", False)))
        )
        sku_billed_usd = 0.0 if is_free_on_public_repo else sku_private_usd

        total_allowance_pool_consumed += pool_used
        total_allowance_covered_minutes += covered_min
        total_overage_minutes += overage_min
        if uses_allow:
            total_overage_compute_units += overage_min * multiplier
        private_compute_usd += sku_private_usd
        list_price_compute_usd += sku_list_usd
        public_billed_compute_usd += sku_billed_usd

        sku_breakdown[ck] = {
            "sku": row["sku"],
            "engine_key": ck,
            "os": row["os"],
            "arch": row["arch"],
            "cores": cores,
            "gpu": bool(row.get("gpu", False)),
            "billed_minutes": billed_min,
            "core_minutes": core_min,
            "rate_usd_per_min": rate_usd,
            "uses_allowance": uses_allow,
            "allowance_multiplier": multiplier,
            "allowance_consumed": covered_min,
            "allowance_covered_minutes": covered_min,
            "allowance_pool_consumed": pool_used,
            "overage_minutes": overage_min,
            "billed_usd": sku_billed_usd,
            "private_usd": sku_private_usd,
            "list_usd": sku_list_usd,
            "status": row.get("status", "official"),
        }

    if platform == "gitlab" and gitlab_pack_rounding and total_overage_compute_units > 0.0:
        pack_row = sku_map.get("compute_pack_1000")
        pack_qty = float(pack_row["included_qty"]) if pack_row else 1000.0
        pack_usd = float(pack_row["usd"]) if pack_row else 10.0
        packs_needed = math.ceil(total_overage_compute_units / pack_qty)
        non_allow_usd = sum(v["private_usd"] for v in sku_breakdown.values() if not v["uses_allowance"])
        private_compute_usd = packs_needed * pack_usd + non_allow_usd
        allowance_value_usd = (total_allowance_pool_consumed / pack_qty) * pack_usd
        list_price_compute_usd = max(list_price_compute_usd, private_compute_usd + allowance_value_usd)
        if not public_repo:
            public_billed_compute_usd = private_compute_usd

    sh_infra_usd = max(0.0, float(self_hosted_usd_per_month)) if self_hosted_usd_per_month is not None else 0.0
    private_compute_usd += sh_infra_usd
    list_price_compute_usd += sh_infra_usd
    if public_repo:
        compute_usd = (0.0 if public_larger_runners_free else public_billed_compute_usd) + sh_infra_usd
    else:
        compute_usd = private_compute_usd

    # 4. Storage, Cache, Egress, and Docker Layer Caching (DLC) across all 7 providers
    included_storage_gb = float(plan_info.get("included_storage_gb", 0.0))
    storage_over_gb = max(0.0, storage_gb - included_storage_gb)
    included_cache_gb = float(plan_info.get("included_cache_gb_per_repo", rules_dict.get("included_cache_gb_per_repo", 0.0)))
    cache_over_gb = max(0.0, cache_gb - included_cache_gb)
    egress_overage_usd = 0.0
    dlc_overage_usd = 0.0

    if platform == "github":
        storage_row = sku_map.get("actions_storage_gb_hour")
        gb_hour_rate = float(storage_row["usd"]) if storage_row else 0.0
        storage_overage_usd = storage_over_gb * float(storage_hours_per_month) * gb_hour_rate
        storage_overage_monthly_list_usd = storage_over_gb * float(
            storage_row.get("usd_per_gb_month", gb_hour_rate * HOURS_PER_MONTH) if storage_row else 0.0
        )
        cache_row = sku_map.get("actions_cache_gb_hour")
        cache_gb_hour_rate = float(cache_row["usd"]) if cache_row else 0.0
        cache_overage_usd = cache_over_gb * float(storage_hours_per_month) * cache_gb_hour_rate
    elif platform == "gitlab":
        storage_row = sku_map.get("storage_10gb_year")
        gb_month_rate = float(storage_row.get("usd_per_gb_month", 0.5)) if storage_row else 0.5
        if gitlab_pack_rounding and storage_over_gb > 0.0:
            pack_gb = float(storage_row["included_qty"]) if storage_row else 10.0
            pack_month_usd = (float(storage_row["usd"]) / 12.0) if storage_row else 5.0
            storage_overage_usd = math.ceil(storage_over_gb / pack_gb) * pack_month_usd
        else:
            storage_overage_usd = storage_over_gb * gb_month_rate
        storage_overage_monthly_list_usd = storage_overage_usd
        cache_overage_usd = 0.0
        if egress_gb > 0.0:
            egress_overage_usd = (math.ceil(egress_gb / 100.0) * 10.0) if gitlab_pack_rounding else (egress_gb * 0.10)
    elif platform == "circleci":
        storage_row = sku_map.get("storage_gb_month")
        storage_rate = float(storage_row["usd"]) if storage_row else 0.252
        storage_overage_usd = storage_over_gb * storage_rate
        storage_overage_monthly_list_usd = storage_overage_usd
        cache_overage_usd = cache_over_gb * storage_rate
        net_row = sku_map.get("network_egress_gb")
        net_rate = float(net_row["usd"]) if net_row else 0.252
        egress_overage_usd = egress_gb * net_rate
        if dlc_enabled:
            dlc_row = sku_map.get("docker_layer_caching")
            dlc_rate = float(dlc_row["usd"]) if dlc_row else 0.12
            eff_runs = dlc_job_runs if dlc_job_runs is not None else inferred_job_executions
            dlc_overage_usd = max(0, int(eff_runs)) * dlc_rate
    elif platform == "bitbucket":
        storage_row = sku_map.get("lfs_storage_100gb_month")
        pack_usd = float(storage_row["usd"]) if storage_row else 10.0
        storage_overage_usd = (math.ceil(storage_over_gb / 100.0) * pack_usd) if storage_over_gb > 0.0 else 0.0
        storage_overage_monthly_list_usd = storage_overage_usd
        cache_overage_usd = 0.0
    elif platform == "aws":
        s3_row = sku_map.get("S3_Artifact_Cache_GB")
        s3_rate = float(s3_row["usd"]) if s3_row else 0.023
        storage_overage_usd = storage_over_gb * s3_rate
        storage_overage_monthly_list_usd = storage_overage_usd
        cache_overage_usd = cache_over_gb * s3_rate
        nat_row = sku_map.get("NAT_Gateway_Data_Processed_GB")
        nat_rate = float(nat_row["usd"]) if nat_row else 0.045
        egress_overage_usd = egress_gb * nat_rate
    elif platform == "gcp":
        ar_row = sku_map.get("artifact_registry_gb")
        ar_rate = float(ar_row["usd"]) if ar_row else 0.10
        storage_overage_usd = storage_over_gb * ar_rate
        storage_overage_monthly_list_usd = storage_overage_usd
        gcs_row = sku_map.get("gcs_cache_gb")
        gcs_rate = float(gcs_row["usd"]) if gcs_row else 0.020
        cache_overage_usd = cache_over_gb * gcs_rate
        egress_overage_usd = egress_gb * 0.045
    elif platform == "azure":
        art_row = sku_map.get("azure_artifacts_gib")
        art_rate = float(art_row["usd"]) if art_row else 2.0
        art_free = float(art_row.get("included_qty", 2.0)) if art_row else 2.0
        eff_free = max(included_storage_gb, art_free)
        storage_over_gb = max(0.0, storage_gb - eff_free)
        storage_overage_usd = storage_over_gb * art_rate
        storage_overage_monthly_list_usd = storage_overage_usd
        cache_overage_usd = 0.0
    else:
        storage_overage_usd = 0.0
        storage_overage_monthly_list_usd = 0.0
        cache_overage_usd = 0.0

    hidden_tax_usd = storage_overage_usd + cache_overage_usd + egress_overage_usd + dlc_overage_usd
    total_usd = seats_usd + compute_usd + hidden_tax_usd
    list_price_total_usd = list_seats_usd + list_price_compute_usd + hidden_tax_usd
    within_allowance = (not public_repo) and (private_compute_usd == 0.0) and (total_billed_minutes > 0.0)

    unpriced_self_hosted_warning = None
    self_hosted_cloud_equivalent_usd = 0.0
    if self_hosted_billed_minutes > 0.0 and self_hosted_usd_per_month is None and not include_self_hosted_announced:
        std_linux_rate = float(sku_map["linux"]["usd"]) if "linux" in sku_map and str(sku_map["linux"].get("unit", "minute")) == "minute" else 0.006
        self_hosted_cloud_equivalent_usd = round(self_hosted_billed_minutes * std_linux_rate, 2)
        unpriced_self_hosted_warning = (
            f"{self_hosted_billed_minutes:,.1f} self-hosted runner minutes detected with $0.00 metered software fee; "
            f"pass self_hosted_usd_per_month or run_infracost_breakdown() for underlying cloud VM cost "
            f"(standard cloud equivalent: ${self_hosted_cloud_equivalent_usd:,.2f}/mo)."
        )

    return {
        "platform": platform,
        "plan": plan,
        "public_repo": bool(public_repo),
        "public_larger_runners_free": bool(public_larger_runners_free),
        "allowance_mode": allowance_mode,
        "seats": clamped_seats,
        "seat_rate_usd": seat_rate_usd,
        "seats_usd": seats_usd,
        "included_allowance": included_allowance,
        "allowance_consumed": total_allowance_pool_consumed,
        "allowance_pool_consumed": total_allowance_pool_consumed,
        "allowance_covered_minutes": total_allowance_covered_minutes,
        "allowance_remaining": allowance_remaining,
        "within_allowance": within_allowance,
        "total_billed_minutes": total_billed_minutes,
        "total_core_minutes": total_core_minutes,
        "overage_minutes_total": total_overage_minutes,
        "compute_usd": compute_usd,
        "private_plan_compute_usd": private_compute_usd,
        "list_price_compute_usd": list_price_compute_usd,
        "storage_gb": storage_gb,
        "included_storage_gb": included_storage_gb,
        "storage_over_gb": storage_over_gb,
        "storage_overage_usd": storage_overage_usd,
        "storage_overage_monthly_list_usd": storage_overage_monthly_list_usd,
        "cache_gb": cache_gb,
        "included_cache_gb": included_cache_gb,
        "cache_over_gb": cache_over_gb,
        "cache_overage_usd": cache_overage_usd,
        "egress_gb": egress_gb,
        "egress_overage_usd": egress_overage_usd,
        "dlc_enabled": bool(dlc_enabled),
        "dlc_overage_usd": dlc_overage_usd,
        "hidden_tax_usd": hidden_tax_usd,
        "self_hosted_usd": sh_infra_usd,
        "self_hosted_billed_minutes": self_hosted_billed_minutes,
        "self_hosted_cloud_equivalent_usd": self_hosted_cloud_equivalent_usd,
        "unpriced_self_hosted_warning": unpriced_self_hosted_warning,
        "gross_consumed_usd": list_price_total_usd,
        "discount_allowance_usd": max(0.0, list_price_total_usd - total_usd),
        "net_billable_usd": total_usd,
        "total_usd": total_usd,
        "list_price_total_usd": list_price_total_usd,
        "sku_breakdown": sku_breakdown,
        "per_workflow_breakdown": per_workflow_breakdown,
        "sku_resolutions": sku_resolutions,
        "execution_time_ms": round((time.perf_counter_ns() - t_start_ns) / 1_000_000.0, 4),
        "source_label": data["source_label"],
        "source_url": data["source_url"],
        "checked_on": data["checked_on"],
    }


def calculate_savings(
    current_minutes_by_sku: dict[str, float | int | dict | list] | None = None,
    selected_minutes_by_sku: dict[str, float | int | dict | list] | None = None,
    *,
    platform: str = "github",
    plan: str = "team",
    seats: int = 0,
    public_repo: bool = False,
    public_larger_runners_free: bool = True,
    storage_gb: float = 0.0,
    selected_storage_gb: float | None = None,
    cache_gb: float = 0.0,
    selected_cache_gb: float | None = None,
    egress_gb: float = 0.0,
    selected_egress_gb: float | None = None,
    dlc_enabled: bool = False,
    selected_dlc_enabled: bool | None = None,
    sku_order: Sequence[str] | None = None,
    allowance_mode: str = "ordered",
    include_self_hosted_announced: bool = False,
    self_hosted_usd_per_month: float | None = None,
    gitlab_pack_rounding: bool = False,
    storage_hours_per_month: float = HOURS_PER_MONTH,
    rate_overrides: dict[str, float] | None = None,
    included_minutes_override: float | None = None,
    allowance_skus_override: Iterable[str] | None = None,
    current_jobs: Sequence[dict] | None = None,
    selected_jobs: Sequence[dict] | None = None,
    current_workflows: Sequence[dict] | None = None,
    selected_workflows: Sequence[dict] | None = None,
    runs_per_month: float = 0.0,
    current_wall_min_per_run: float | None = None,
    selected_wall_min_per_run: float | None = None,
    current_wall_seconds_per_run: float | None = None,
    selected_wall_seconds_per_run: float | None = None,
    misses: int = 0,
    defect_commits: int = 0,
) -> dict:
    """Compute monthly savings as cost(current) - cost(selected), honoring plan allowance & DORA metrics."""
    common_kw = dict(
        platform=platform,
        plan=plan,
        seats=seats,
        public_repo=public_repo,
        public_larger_runners_free=public_larger_runners_free,
        sku_order=sku_order,
        allowance_mode=allowance_mode,
        include_self_hosted_announced=include_self_hosted_announced,
        self_hosted_usd_per_month=self_hosted_usd_per_month,
        gitlab_pack_rounding=gitlab_pack_rounding,
        storage_hours_per_month=storage_hours_per_month,
        rate_overrides=rate_overrides,
        included_minutes_override=included_minutes_override,
        allowance_skus_override=allowance_skus_override,
    )
    current = calculate_monthly_cost(
        current_minutes_by_sku,
        storage_gb=storage_gb,
        cache_gb=cache_gb,
        egress_gb=egress_gb,
        dlc_enabled=dlc_enabled,
        jobs=current_jobs,
        workflows=current_workflows,
        **common_kw,
    )
    selected = calculate_monthly_cost(
        selected_minutes_by_sku,
        storage_gb=storage_gb if selected_storage_gb is None else selected_storage_gb,
        cache_gb=cache_gb if selected_cache_gb is None else selected_cache_gb,
        egress_gb=egress_gb if selected_egress_gb is None else selected_egress_gb,
        dlc_enabled=dlc_enabled if selected_dlc_enabled is None else selected_dlc_enabled,
        jobs=selected_jobs,
        workflows=selected_workflows,
        **common_kw,
    )

    saved_compute_usd = current["compute_usd"] - selected["compute_usd"]
    saved_total_usd = current["total_usd"] - selected["total_usd"]
    saved_list_compute_usd = current["list_price_compute_usd"] - selected["list_price_compute_usd"]
    saved_gross_consumed_usd = current["gross_consumed_usd"] - selected["gross_consumed_usd"]
    saved_discount_allowance_usd = current["discount_allowance_usd"] - selected["discount_allowance_usd"]
    saved_net_billable_usd = current["net_billable_usd"] - selected["net_billable_usd"]
    saved_billed_minutes = current["total_billed_minutes"] - selected["total_billed_minutes"]
    saved_core_minutes = current["total_core_minutes"] - selected["total_core_minutes"]

    inside_allowance = (not public_repo) and (current["compute_usd"] == 0.0) and (current["total_billed_minutes"] > 0.0)
    crossed_allowance_boundary = (
        (not public_repo)
        and (current["compute_usd"] > 0.0)
        and (selected["allowance_pool_consumed"] > 0.0 or selected["total_billed_minutes"] == 0.0)
        and (
            selected["compute_usd"] == 0.0
            or abs(saved_compute_usd - saved_list_compute_usd) > 1e-6
        )
    )

    if public_repo and current["compute_usd"] == 0.0:
        savings_note = PUBLIC_REPO_NOTE
    elif inside_allowance:
        savings_note = INSIDE_ALLOWANCE_NOTE
    elif crossed_allowance_boundary:
        savings_note = (
            f"${saved_compute_usd:.2f} compute saved "
            f"(crossed plan's {int(current['included_allowance']):,}-minute allowance boundary; "
            f"list-price reference ${saved_list_compute_usd:.2f})"
        )
    else:
        savings_note = f"${saved_compute_usd:.2f} compute saved"

    # Auto-infer runs_per_month and critical-path wall minutes from workflows if not explicitly passed
    eff_runs_per_month = float(runs_per_month)
    if eff_runs_per_month <= 0.0 and current["per_workflow_breakdown"]:
        eff_runs_per_month = sum(w["monthly_active_runs"] for w in current["per_workflow_breakdown"].values())

    # Developer wait: critical path per run (never sum of parallel job minutes) -> DORA lead time for changes
    if current_wall_seconds_per_run is not None:
        cur_wall_s = float(current_wall_seconds_per_run)
        cur_wall_m = cur_wall_s / 60.0
    elif current_wall_min_per_run is not None:
        cur_wall_m = float(current_wall_min_per_run)
        cur_wall_s = cur_wall_m * 60.0
    elif current["per_workflow_breakdown"]:
        cur_wall_m = max((w.get("critical_path_wall_min", 0.0) for w in current["per_workflow_breakdown"].values()), default=0.0)
        cur_wall_s = cur_wall_m * 60.0
    else:
        cur_wall_s = 0.0
        cur_wall_m = 0.0

    if selected_wall_seconds_per_run is not None:
        sel_wall_s = float(selected_wall_seconds_per_run)
        sel_wall_m = sel_wall_s / 60.0
    elif selected_wall_min_per_run is not None:
        sel_wall_m = float(selected_wall_min_per_run)
        sel_wall_s = sel_wall_m * 60.0
    elif selected["per_workflow_breakdown"]:
        sel_wall_m = max((w.get("critical_path_wall_min", 0.0) for w in selected["per_workflow_breakdown"].values()), default=0.0)
        sel_wall_s = sel_wall_m * 60.0
    else:
        sel_wall_s = cur_wall_s
        sel_wall_m = cur_wall_m

    saved_wall_s_per_run = max(0.0, cur_wall_s - sel_wall_s)
    saved_wall_min_per_run = saved_wall_s_per_run / 60.0
    saved_wait_hours_per_month = (eff_runs_per_month * saved_wall_min_per_run) / 60.0

    # Miss rate -> DORA change failure rate
    miss_rate = (float(misses) / float(defect_commits)) if defect_commits > 0 else 0.0

    return {
        "platform": platform,
        "plan": plan,
        "public_repo": bool(public_repo),
        "current": current,
        "selected": selected,
        "saved_compute_usd": saved_compute_usd,
        "saved_total_usd": saved_total_usd,
        "saved_list_compute_usd": saved_list_compute_usd,
        "saved_gross_consumed_usd": saved_gross_consumed_usd,
        "saved_discount_allowance_usd": saved_discount_allowance_usd,
        "saved_net_billable_usd": saved_net_billable_usd,
        "saved_billed_minutes": saved_billed_minutes,
        "total_core_minutes": current["total_core_minutes"],
        "selected_core_minutes": selected["total_core_minutes"],
        "saved_core_minutes": saved_core_minutes,
        "inside_allowance": inside_allowance,
        "crossed_allowance_boundary": crossed_allowance_boundary,
        "savings_note": savings_note,
        "wait_metrics": {
            "dora_metric": "lead_time_for_changes",
            "runs_per_month": eff_runs_per_month,
            "current_wall_seconds_per_run": cur_wall_s,
            "selected_wall_seconds_per_run": sel_wall_s,
            "saved_wall_seconds_per_run": saved_wall_s_per_run,
            "current_wall_min_per_run": cur_wall_m,
            "selected_wall_min_per_run": sel_wall_m,
            "saved_wall_min_per_run": saved_wall_min_per_run,
            "saved_wait_hours_per_month": saved_wait_hours_per_month,
        },
        "safety_metrics": {
            "dora_metric": "change_failure_rate",
            "misses": int(misses),
            "defect_commits": int(defect_commits),
            "miss_rate": miss_rate,
            "miss_pct": miss_rate * 100.0,
        },
    }


MULTI_CLOUD_PRICING_SOURCES: dict[str, dict[str, str]] = {
    "infracost": {
        "name": "Infracost CLI & Cloud Pricing GraphQL API (v2.19.0)",
        "url": "https://pricing.api.infracost.io/graphql",
        "supports": "AWS, GCP, Azure On-Demand & Spot instances (purchaseOption='on_demand'|'spot'), aws_codebuild_project, S3/GCS/Blob, NAT",
    },
    "gcosts": {
        "name": "Cyclenerd/google-cloud-pricing-cost-calculator (gcosts)",
        "url": "https://github.com/Cyclenerd/google-cloud-pricing-cost-calculator",
        "supports": "GCP Compute Engine (On-Demand & Spot/Preemptible), Persistent Disk SSD, GCS, Cloud NAT via pricing.yml",
    },
    "economize": {
        "name": "Economize.cloud Multi-Cloud FinOps Calculators & Benchmarks",
        "url": "https://www.economize.cloud/",
        "supports": "Multi-cloud AWS EC2/CodeBuild, GCP Compute/Cloud Build, Azure VM On-Demand vs Spot & commitment cross-checks",
    },
    "vantage": {
        "name": "Vantage.sh Cloud Instance & Spot Reference (instances.vantage.sh)",
        "url": "https://instances.vantage.sh/",
        "supports": "AWS EC2, GCP GCE, Azure VM live On-Demand & Spot market rates",
    },
    "azure_retail_api": {
        "name": "Azure Retail Prices Unauthenticated REST API",
        "url": "https://prices.azure.com/api/retail/prices",
        "supports": "Azure DevOps parallel jobs ($40/$15), Basic/Test Plans seats, Managed DevOps Pool VMs & Spot priority VMs",
    },
    "gcp_pricing_calculator": {
        "name": "Google Cloud Pricing Calculator (UI + pricelist.json backend)",
        "url": "https://cloud.google.com/products/calculator",
        "backend_json": "https://cloudpricingcalculator.appspot.com/static/data/pricelist.json",
        "supports": "Back-of-house browser/Puppeteer/browser-use & JSON verification for Cloud Build, Cloud Deploy, GCE, GCS",
    },
    "aws_pricing_calculator": {
        "name": "AWS Pricing Calculator (calculator.aws + Bulk Price List JSON API)",
        "url": "https://calculator.aws/",
        "backend_json": "https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AWSCodeBuild/current/us-east-1/index.json",
        "supports": "Back-of-house browser/Puppeteer/browser-use & JSON verification for AWS CodeBuild, CodePipeline, EC2, S3, NAT",
    },
    "azure_pricing_calculator": {
        "name": "Azure Pricing Calculator (UI + Retail Prices JSON API)",
        "url": "https://azure.microsoft.com/en-us/pricing/calculator/",
        "backend_json": "https://prices.azure.com/api/retail/prices?$filter=serviceName eq 'Azure DevOps'",
        "supports": "Back-of-house browser/Puppeteer/browser-use & JSON verification for Azure DevOps Pipelines, Artifacts, VMs",
    },
}


def run_infracost_breakdown(
    terraform_path: str | Path,
    usage_file: str | Path | None = None,
) -> dict[str, Any]:
    """Delegate dynamic cloud CI/CD & runner infrastructure pricing (including Spot instances) to `/opt/homebrew/bin/infracost`.

    Supports `aws_codebuild_project`, self-hosted EC2/EKS (On-Demand & Spot), S3, GCP worker pools (Standard & Spot), and Azure MDP VMs.
    """
    cmd = [INFRACOST_BIN, "breakdown", "--path", str(terraform_path), "--format", "json"]
    if usage_file is not None:
        cmd.extend(["--usage-file", str(usage_file)])
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(f"infracost breakdown failed ({proc.returncode}): {proc.stderr.strip()}")
    return json.loads(proc.stdout)


def query_infracost_cloud_sku(
    vendor_name: str,
    service: str,
    region: str,
    attribute_filters: dict[str, str],
    purchase_option: str = "on_demand",
    api_key: str | None = None,
) -> list[dict[str, Any]]:
    """Query the Infracost Cloud Pricing GraphQL API (`https://pricing.api.infracost.io/graphql`) for On-Demand or Spot SKUs.

    Pass `purchase_option="spot"` to retrieve live Spot / Preemptible instance rates directly from Infracost.
    """
    key = api_key or os.environ.get("INFRACOST_API_KEY", "")
    if not key:
        raise ValueError("INFRACOST_API_KEY is required to query pricing.api.infracost.io/graphql")
    if purchase_option not in ("on_demand", "spot", "all"):
        raise ValueError(f"Unsupported purchase_option={purchase_option!r}; expected 'on_demand', 'spot', or 'all'")
    filters_gql = ", ".join(
        f'{{key: {json.dumps(k)}, value: {json.dumps(v)}}}' for k, v in attribute_filters.items()
    )
    price_filter = "" if purchase_option == "all" else f'(filter: {{ purchaseOption: {json.dumps(purchase_option)} }})'
    query = f"""
    {{
      products(
        filter: {{
          vendorName: {json.dumps(vendor_name)}
          service: {json.dumps(service)}
          region: {json.dumps(region)}
          attributeFilters: [{filters_gql}]
        }}
      ) {{
        productFamily
        attributes {{ key value }}
        prices{price_filter} {{
          purchaseOption
          USD
          unit
          description
        }}
      }}
    }}
    """
    req = urllib.request.Request(
        INFRACOST_GRAPHQL_URL,
        data=json.dumps({"query": query}).encode("utf-8"),
        headers={"Content-Type": "application/json", "X-Api-Key": key},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        payload = json.loads(resp.read().decode("utf-8"))
    return payload.get("data", {}).get("products", [])


def audit_receipt_with_ollama(
    receipt: dict[str, Any],
    context_note: str = "",
    model: str = "qwen2.5-coder:7b",
    ollama_url: str = "http://127.0.0.1:11434/api/generate",
) -> dict[str, Any]:
    """Adversarial AI Auditor: sends the engine's deterministic receipt to local Ollama to question and verify the math."""
    compact = {
        "platform": receipt.get("platform"),
        "plan": receipt.get("plan"),
        "public_repo": receipt.get("public_repo"),
        "total_billed_minutes": receipt.get("total_billed_minutes"),
        "gross_consumed_usd": receipt.get("gross_consumed_usd"),
        "discount_allowance_usd": receipt.get("discount_allowance_usd"),
        "net_billable_usd": receipt.get("net_billable_usd"),
        "sku_resolutions": receipt.get("sku_resolutions"),
        "sku_breakdown": receipt.get("sku_breakdown"),
    }
    prompt = (
        "You are an independent FinOps QA Auditor. Question and verify this CI/CD cost receipt:\n"
        f"Context: {context_note}\n"
        f"Receipt JSON: {json.dumps(compact)}\n"
        "Verify: (1) gross_consumed_usd - discount_allowance_usd == net_billable_usd, "
        "(2) per-SKU billed_minutes * rate_usd_per_min == list_usd, "
        "(3) any custom runner labels in sku_resolutions are mapped sensibly.\n"
        'Respond ONLY in JSON: {"verdict": "PASS" or "FAIL", "math_verified": true, "findings": "<1-2 sentence audit summary>"}'
    )
    t0 = time.perf_counter_ns()
    try:
        req = urllib.request.Request(
            ollama_url,
            data=json.dumps({"model": model, "prompt": prompt, "stream": False, "format": "json"}).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=20) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            parsed = json.loads(body.get("response", "{}"))
            parsed["model"] = model
            parsed["audit_latency_ms"] = round((time.perf_counter_ns() - t0) / 1_000_000.0, 2)
            return parsed
    except Exception as exc:
        return {
            "verdict": "UNAVAILABLE",
            "math_verified": False,
            "model": model,
            "error": str(exc),
            "audit_latency_ms": round((time.perf_counter_ns() - t0) / 1_000_000.0, 2),
        }


def check_all() -> dict:
    summary = {}
    for p in PLATFORMS:
        data = load_platform_pricing(p)
        verify_csv_sync(p)
        summary[p] = {
            "checked_on": data["checked_on"],
            "plans": len(data["plans"]),
            "skus": len(data["skus"]),
            "source_url": data["source_url"],
        }
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate or export CI pricing tables")
    parser.add_argument("--sync-csv", action="store_true", help="Regenerate pricing/*.csv from pricing/*.json")
    parser.add_argument("--check", action="store_true", help="Validate pricing/*.json and verify pricing/*.csv match")
    args = parser.parse_args(argv)

    if args.sync_csv:
        for p in PLATFORMS:
            out = export_csv(p)
            print(f"wrote {out.relative_to(ROOT)}")
    summary = check_all()
    if args.check or not args.sync_csv:
        for p, info in summary.items():
            print(f"pricing/{p}: {info['skus']} SKUs, {info['plans']} plans, checked_on={info['checked_on']} OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
