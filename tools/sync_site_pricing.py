#!/usr/bin/env python3
"""Sync canonical pricing/*.csv files into site/pricing/*.csv and verify byte-for-byte parity.

Usage:
  /usr/bin/python3 tools/sync_site_pricing.py          # copy pricing/*.csv -> site/pricing/*.csv and verify
  /usr/bin/python3 tools/sync_site_pricing.py --check  # verify without writing
"""
from __future__ import annotations

import argparse
import csv
import io
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PRICING_DIR = ROOT / "pricing"
SITE_PRICING_DIR = ROOT / "site" / "pricing"

REQUIRED_COLUMNS = (
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

DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def sync_site_pricing() -> list[Path]:
    src_files = sorted(PRICING_DIR.glob("*.csv"))
    if not src_files:
        raise FileNotFoundError(f"no pricing/*.csv files found in {PRICING_DIR}")
    SITE_PRICING_DIR.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    for src in src_files:
        dst = SITE_PRICING_DIR / src.name
        shutil.copyfile(src, dst)
        written.append(dst)
    return written


def check_site_pricing() -> dict[str, dict]:
    src_files = sorted(PRICING_DIR.glob("*.csv"))
    if not src_files:
        raise AssertionError(f"no pricing/*.csv files found in {PRICING_DIR}")
    if not SITE_PRICING_DIR.is_dir():
        raise AssertionError(f"missing directory: {SITE_PRICING_DIR.relative_to(ROOT)}")

    summary: dict[str, dict] = {}
    checked_on_idx = REQUIRED_COLUMNS.index("checked_on")
    source_url_idx = REQUIRED_COLUMNS.index("source_url")
    platform_idx = REQUIRED_COLUMNS.index("platform")

    for src in src_files:
        dst = SITE_PRICING_DIR / src.name
        rel_src = src.relative_to(ROOT)
        rel_dst = dst.relative_to(ROOT)
        if not dst.is_file():
            raise AssertionError(f"missing site pricing CSV: {rel_dst}")

        src_bytes = src.read_bytes()
        dst_bytes = dst.read_bytes()
        if not dst_bytes:
            raise AssertionError(f"empty site pricing CSV: {rel_dst}")
        if src_bytes != dst_bytes:
            raise AssertionError(f"{rel_dst} does not match {rel_src} byte-for-byte")

        text = dst_bytes.decode("utf-8")
        rows = list(csv.reader(io.StringIO(text)))
        if len(rows) < 2:
            raise AssertionError(f"{rel_dst} must contain a header and at least 1 SKU row")

        header = tuple(rows[0])
        if len(header) != 15 or header != REQUIRED_COLUMNS:
            raise AssertionError(
                f"{rel_dst} header mismatch: got {header}, expected {REQUIRED_COLUMNS}"
            )

        expected_platform = src.stem
        checked_dates: set[str] = set()
        for row_num, row in enumerate(rows[1:], start=2):
            if len(row) != len(REQUIRED_COLUMNS):
                raise AssertionError(
                    f"{rel_dst}:{row_num} has {len(row)} columns, expected {len(REQUIRED_COLUMNS)}"
                )
            if row[platform_idx] != expected_platform:
                raise AssertionError(
                    f"{rel_dst}:{row_num} platform={row[platform_idx]!r} != {expected_platform!r}"
                )
            checked_on = row[checked_on_idx].strip()
            if not DATE_RE.match(checked_on):
                raise AssertionError(
                    f"{rel_dst}:{row_num} invalid checked_on={checked_on!r} (expected YYYY-MM-DD)"
                )
            checked_dates.add(checked_on)
            source_url = row[source_url_idx].strip()
            if not source_url.startswith("https://"):
                raise AssertionError(
                    f"{rel_dst}:{row_num} invalid source_url={source_url!r} (expected https://...)"
                )

        summary[src.name] = {
            "rows": len(rows) - 1,
            "bytes": len(dst_bytes),
            "checked_on": sorted(checked_dates),
        }

    extra_site_files = {p.name for p in SITE_PRICING_DIR.glob("*.csv")} - {p.name for p in src_files}
    if extra_site_files:
        raise AssertionError(f"orphan CSV files in site/pricing/: {sorted(extra_site_files)}")

    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Sync pricing/*.csv to site/pricing/*.csv and verify parity")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verify site/pricing/*.csv matches pricing/*.csv byte-for-byte without writing",
    )
    args = parser.parse_args(argv)

    if not args.check:
        written = sync_site_pricing()
        for dst in written:
            print(f"synced {dst.relative_to(ROOT)}")

    summary = check_site_pricing()
    for name, info in summary.items():
        dates = ",".join(info["checked_on"])
        print(
            f"site/pricing/{name}: {info['rows']} rows, {info['bytes']} bytes, "
            f"15 cols, checked_on={dates} OK"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
