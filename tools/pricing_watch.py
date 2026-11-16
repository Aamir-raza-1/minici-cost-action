#!/usr/bin/env python3
"""Weekly CI/CD pricing watch and offline verifier (PRICE2).

Fetches rank-1 official pricing pages (or runs offline against embedded doc
fixtures in --self-test / --check mode), parses runner rate tables for GitHub
Actions, GitLab CI/CD, CircleCI, and Bitbucket Pipelines, diffs against
pricing/*.json, bumps checked_on + syncs CSVs when unchanged, and emits a
structured diff / PR / issue payload when rates change or a parser fails.

Usage:
  /usr/bin/python3 tools/pricing_watch.py --self-test
  /usr/bin/python3 tools/pricing_watch.py --check
  /usr/bin/python3 tools/pricing_watch.py --fetch [--bump-checked-on] [--out pricing_diff.json]
"""

from __future__ import annotations

import argparse
import datetime as dt
import html.parser
import json
import os
import re
import shutil
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from engine import pricing  # noqa: E402

PRICING_DIR = ROOT / "pricing"
SITE_PRICING_DIR = ROOT / "site" / "pricing"
ISO_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")

ALL_KNOWN_PLATFORMS = ("github", "gitlab", "circleci", "bitbucket")


class ParserError(RuntimeError):
    """Raised when an official pricing page cannot be parsed into SKU rates."""


# Embedded official doc table snippets for deterministic offline verification
# (--self-test / --check) across all 4 platforms.
EMBEDDED_DOC_SNIPPETS: dict[str, str] = {
    "github": """
    <html><body>
      <h1>About billing for GitHub Actions</h1>
      <h2>Included minutes and storage</h2>
      <table>
        <tr><th>Product</th><th>Included minutes</th></tr>
        <tr><td>GitHub Free</td><td>2,000</td></tr>
        <tr><td>GitHub Pro</td><td>3,000</td></tr>
        <tr><td>GitHub Team</td><td>3,000</td></tr>
        <tr><td>GitHub Enterprise Cloud</td><td>50,000</td></tr>
      </table>
      <h2>Per-minute rates for standard and larger runners (Jan 2026)</h2>
      <table>
        <thead><tr><th>SKU</th><th>Runner</th><th>Per-minute rate (USD)</th></tr></thead>
        <tbody>
          <tr data-sku="linux"><td>linux</td><td>Linux 2-core (standard)</td><td>$0.006</td></tr>
          <tr data-sku="linux_x64_2core"><td>linux_x64_2core</td><td>Linux 2-core x64</td><td>$0.006</td></tr>
          <tr data-sku="linux_arm64_2core"><td>linux_arm64_2core</td><td>Linux ARM64 2-core</td><td>$0.005</td></tr>
          <tr data-sku="windows"><td>windows</td><td>Windows 2-core (standard)</td><td>$0.010</td></tr>
          <tr data-sku="windows_x64_2core"><td>windows_x64_2core</td><td>Windows 2-core x64</td><td>$0.010</td></tr>
          <tr data-sku="windows_arm64_2core"><td>windows_arm64_2core</td><td>Windows ARM64 2-core</td><td>$0.008</td></tr>
          <tr data-sku="macos"><td>macos</td><td>macOS 3-core (M1 / standard)</td><td>$0.062</td></tr>
          <tr data-sku="macos_3_4core"><td>macos_3_4core</td><td>macOS 3/4-core standard</td><td>$0.062</td></tr>
          <tr data-sku="linux_x64_4core"><td>linux_x64_4core</td><td>Linux 4-core</td><td>$0.012</td></tr>
          <tr data-sku="linux_x64_8core"><td>linux_x64_8core</td><td>Linux 8-core</td><td>$0.022</td></tr>
          <tr data-sku="linux_x64_16core"><td>linux_x64_16core</td><td>Linux 16-core</td><td>$0.042</td></tr>
          <tr data-sku="linux_x64_32core"><td>linux_x64_32core</td><td>Linux 32-core</td><td>$0.082</td></tr>
          <tr data-sku="linux_x64_64core"><td>linux_x64_64core</td><td>Linux 64-core</td><td>$0.162</td></tr>
          <tr data-sku="linux_x64_96core"><td>linux_x64_96core</td><td>Linux 96-core</td><td>$0.252</td></tr>
          <tr data-sku="linux_arm64_4core"><td>linux_arm64_4core</td><td>Linux ARM64 4-core</td><td>$0.008</td></tr>
          <tr data-sku="linux_arm64_8core"><td>linux_arm64_8core</td><td>Linux ARM64 8-core</td><td>$0.014</td></tr>
          <tr data-sku="linux_arm64_16core"><td>linux_arm64_16core</td><td>Linux ARM64 16-core</td><td>$0.026</td></tr>
          <tr data-sku="linux_arm64_32core"><td>linux_arm64_32core</td><td>Linux ARM64 32-core</td><td>$0.050</td></tr>
          <tr data-sku="linux_arm64_64core"><td>linux_arm64_64core</td><td>Linux ARM64 64-core</td><td>$0.098</td></tr>
          <tr data-sku="windows_x64_4core"><td>windows_x64_4core</td><td>Windows 4-core</td><td>$0.022</td></tr>
          <tr data-sku="windows_x64_8core"><td>windows_x64_8core</td><td>Windows 8-core</td><td>$0.042</td></tr>
          <tr data-sku="windows_x64_16core"><td>windows_x64_16core</td><td>Windows 16-core</td><td>$0.082</td></tr>
          <tr data-sku="windows_x64_32core"><td>windows_x64_32core</td><td>Windows 32-core</td><td>$0.162</td></tr>
          <tr data-sku="windows_x64_64core"><td>windows_x64_64core</td><td>Windows 64-core</td><td>$0.322</td></tr>
          <tr data-sku="windows_x64_96core"><td>windows_x64_96core</td><td>Windows 96-core</td><td>$0.552</td></tr>
          <tr data-sku="windows_arm64_4core"><td>windows_arm64_4core</td><td>Windows ARM64 4-core</td><td>$0.014</td></tr>
          <tr data-sku="windows_arm64_8core"><td>windows_arm64_8core</td><td>Windows ARM64 8-core</td><td>$0.026</td></tr>
          <tr data-sku="windows_arm64_16core"><td>windows_arm64_16core</td><td>Windows ARM64 16-core</td><td>$0.050</td></tr>
          <tr data-sku="windows_arm64_32core"><td>windows_arm64_32core</td><td>Windows ARM64 32-core</td><td>$0.098</td></tr>
          <tr data-sku="windows_arm64_64core"><td>windows_arm64_64core</td><td>Windows ARM64 64-core</td><td>$0.194</td></tr>
          <tr data-sku="macos_12core_intel"><td>macos_12core_intel</td><td>macOS 12-core Intel</td><td>$0.077</td></tr>
          <tr data-sku="macos_large_m2pro"><td>macos_large_m2pro</td><td>macOS 12-core M2 Pro</td><td>$0.102</td></tr>
          <tr data-sku="linux_gpu_t4_4core"><td>linux_gpu_t4_4core</td><td>GPU Linux 4-core T4</td><td>$0.052</td></tr>
          <tr data-sku="windows_gpu_t4_4core"><td>windows_gpu_t4_4core</td><td>GPU Windows 4-core T4</td><td>$0.102</td></tr>
        </tbody>
      </table>
    </body></html>
    """,
    "gitlab": """
    <html><body>
      <h1>GitLab CI/CD compute minutes</h1>
      <p>Additional compute minutes cost $10 per 1,000 compute minutes ($0.010 per compute minute).</p>
      <table>
        <thead><tr><th>SKU</th><th>Runner size</th><th>Cost factor</th><th>Effective USD/min</th></tr></thead>
        <tbody>
          <tr data-sku="linux"><td>linux</td><td>saas-linux-small-amd64 (2 vCPU)</td><td>1</td><td>$0.010</td></tr>
          <tr data-sku="linux_x64_small"><td>linux_x64_small</td><td>saas-linux-small-amd64</td><td>1</td><td>$0.010</td></tr>
          <tr data-sku="linux_x64_medium"><td>linux_x64_medium</td><td>saas-linux-medium-amd64 (4 vCPU)</td><td>2</td><td>$0.020</td></tr>
          <tr data-sku="linux_x64_large"><td>linux_x64_large</td><td>saas-linux-large-amd64 (8 vCPU)</td><td>3</td><td>$0.030</td></tr>
          <tr data-sku="linux_x64_xlarge"><td>linux_x64_xlarge</td><td>saas-linux-xlarge-amd64 (16 vCPU)</td><td>6</td><td>$0.060</td></tr>
          <tr data-sku="linux_x64_2xlarge"><td>linux_x64_2xlarge</td><td>saas-linux-2xlarge-amd64 (32 vCPU)</td><td>12</td><td>$0.120</td></tr>
          <tr data-sku="linux_arm64_small"><td>linux_arm64_small</td><td>saas-linux-small-arm64</td><td>1</td><td>$0.010</td></tr>
          <tr data-sku="linux_arm64_medium"><td>linux_arm64_medium</td><td>saas-linux-medium-arm64</td><td>2</td><td>$0.020</td></tr>
          <tr data-sku="linux_arm64_large"><td>linux_arm64_large</td><td>saas-linux-large-arm64</td><td>3</td><td>$0.030</td></tr>
          <tr data-sku="macos"><td>macos</td><td>saas-macos-medium-m1 (4 vCPU)</td><td>6</td><td>$0.060</td></tr>
          <tr data-sku="macos_medium_m1"><td>macos_medium_m1</td><td>saas-macos-medium-m1 (4 vCPU)</td><td>6</td><td>$0.060</td></tr>
          <tr data-sku="macos_large_m2pro"><td>macos_large_m2pro</td><td>saas-macos-large-m2pro (12 vCPU)</td><td>12</td><td>$0.120</td></tr>
          <tr data-sku="windows"><td>windows</td><td>windows-shared (2 vCPU)</td><td>1</td><td>$0.010</td></tr>
          <tr data-sku="windows_standard"><td>windows_standard</td><td>windows-shared standard</td><td>1</td><td>$0.010</td></tr>
          <tr data-sku="linux_gpu_standard"><td>linux_gpu_standard</td><td>saas-linux-medium-amd64-gpu-standard</td><td>7</td><td>$0.070</td></tr>
          <tr data-sku="self_hosted"><td>self_hosted</td><td>Self-managed runner</td><td>0</td><td>$0.000</td></tr>
        </tbody>
      </table>
    </body></html>
    """,
    "circleci": """
    <html><body>
      <h1>CircleCI Pricing &amp; Resource Classes</h1>
      <p>Credits cost $0.0006 per credit ($15 per 25,000 credits).</p>
      <table>
        <thead><tr><th>SKU</th><th>Resource class</th><th>Credits/min</th><th>Effective USD/min</th></tr></thead>
        <tbody>
          <tr data-sku="linux_x64_small"><td>linux_x64_small</td><td>Docker small (1 vCPU)</td><td>5</td><td>$0.003</td></tr>
          <tr data-sku="linux_x64_medium"><td>linux_x64_medium</td><td>Docker medium (2 vCPU)</td><td>10</td><td>$0.006</td></tr>
          <tr data-sku="linux_x64_medium_plus"><td>linux_x64_medium_plus</td><td>Docker medium+ (3 vCPU)</td><td>15</td><td>$0.009</td></tr>
          <tr data-sku="linux_x64_large"><td>linux_x64_large</td><td>Docker large (4 vCPU)</td><td>20</td><td>$0.012</td></tr>
          <tr data-sku="linux_x64_xlarge"><td>linux_x64_xlarge</td><td>Docker xlarge (8 vCPU)</td><td>40</td><td>$0.024</td></tr>
          <tr data-sku="linux_x64_2xlarge"><td>linux_x64_2xlarge</td><td>Docker 2xlarge (16 vCPU)</td><td>80</td><td>$0.048</td></tr>
          <tr data-sku="linux_x64_2xlarge_plus"><td>linux_x64_2xlarge_plus</td><td>Docker 2xlarge+ (20 vCPU)</td><td>100</td><td>$0.060</td></tr>
          <tr data-sku="linux_arm64_medium"><td>linux_arm64_medium</td><td>Arm64 medium (2 vCPU)</td><td>13</td><td>$0.0078</td></tr>
          <tr data-sku="linux_arm64_large"><td>linux_arm64_large</td><td>Arm64 large (4 vCPU)</td><td>26</td><td>$0.0156</td></tr>
          <tr data-sku="macos_m1_medium"><td>macos_m1_medium</td><td>macOS M1 medium (4 vCPU)</td><td>150</td><td>$0.090</td></tr>
          <tr data-sku="macos_m1_large"><td>macos_m1_large</td><td>macOS M1 large (8 vCPU)</td><td>300</td><td>$0.180</td></tr>
          <tr data-sku="macos_m2pro_medium"><td>macos_m2pro_medium</td><td>macOS M2 Pro medium (8 vCPU)</td><td>200</td><td>$0.120</td></tr>
          <tr data-sku="macos_m2pro_large"><td>macos_m2pro_large</td><td>macOS M2 Pro large (12 vCPU)</td><td>300</td><td>$0.180</td></tr>
          <tr data-sku="windows_medium"><td>windows_medium</td><td>Windows medium (4 vCPU)</td><td>40</td><td>$0.024</td></tr>
          <tr data-sku="windows_large"><td>windows_large</td><td>Windows large (8 vCPU)</td><td>80</td><td>$0.048</td></tr>
          <tr data-sku="linux_gpu_nvidia_t4"><td>linux_gpu_nvidia_t4</td><td>GPU Linux NVIDIA T4</td><td>160</td><td>$0.096</td></tr>
          <tr data-sku="self_hosted"><td>self_hosted</td><td>Self-hosted runner</td><td>0</td><td>$0.000</td></tr>
        </tbody>
      </table>
    </body></html>
    """,
    "bitbucket": """
    <html><body>
      <h1>Bitbucket Pipelines build minutes</h1>
      <p>Additional build minutes cost $10 per 1,000 build minutes ($0.010 per build minute).</p>
      <table>
        <thead><tr><th>SKU</th><th>Size</th><th>Multiplier</th><th>Effective USD/min</th></tr></thead>
        <tbody>
          <tr data-sku="linux_1x"><td>linux_1x</td><td>1x Linux (2 vCPU, 4 GB)</td><td>1x</td><td>$0.010</td></tr>
          <tr data-sku="linux_2x"><td>linux_2x</td><td>2x Linux (4 vCPU, 8 GB)</td><td>2x</td><td>$0.020</td></tr>
          <tr data-sku="linux_4x"><td>linux_4x</td><td>4x Linux (8 vCPU, 16 GB)</td><td>4x</td><td>$0.040</td></tr>
          <tr data-sku="linux_8x"><td>linux_8x</td><td>8x Linux (16 vCPU, 32 GB)</td><td>8x</td><td>$0.080</td></tr>
          <tr data-sku="linux_16x"><td>linux_16x</td><td>16x Linux (32 vCPU, 64 GB)</td><td>16x</td><td>$0.160</td></tr>
          <tr data-sku="self_hosted"><td>self_hosted</td><td>Self-hosted runner</td><td>0x</td><td>$0.000</td></tr>
        </tbody>
      </table>
    </body></html>
    """,
}

EXPECTED_ANCHOR_RATES: dict[str, dict[str, float]] = {
    "github": {
        "linux": 0.006,
        "windows": 0.010,
        "macos": 0.062,
        "linux_x64_4core": 0.012,
        "linux_x64_8core": 0.022,
        "linux_x64_16core": 0.042,
        "linux_x64_32core": 0.082,
        "linux_x64_64core": 0.162,
    },
    "gitlab": {
        "linux": 0.010,
        "linux_x64_small": 0.010,
        "linux_x64_medium": 0.020,
        "linux_x64_large": 0.030,
        "linux_x64_xlarge": 0.060,
        "linux_x64_2xlarge": 0.120,
        "macos_medium_m1": 0.060,
        "windows": 0.010,
    },
    "circleci": {
        "linux_x64_small": 0.003,
        "linux_x64_medium": 0.006,
        "linux_x64_large": 0.012,
        "macos_m1_medium": 0.090,
        "windows_medium": 0.024,
    },
    "bitbucket": {
        "linux_1x": 0.010,
        "linux_2x": 0.020,
        "linux_4x": 0.040,
        "linux_8x": 0.080,
        "linux_16x": 0.160,
    },
}


def _baseline_minute_skus(baseline_data: dict[str, Any] | None) -> dict[str, float]:
    """Extract {sku_name: usd_per_min} for minute-billed runner SKUs from pricing/*.json."""
    if not baseline_data:
        return {}
    skus = baseline_data.get("skus")
    out: dict[str, float] = {}
    if isinstance(skus, list):
        for row in skus:
            if row.get("unit") != "minute":
                continue
            sku_id = str(row["sku"])
            if sku_id == "self_hosted" and "effective_usd" in row:
                continue
            out[sku_id] = float(row["usd"])
    elif isinstance(skus, dict):
        for sku_id, meta in skus.items():
            if isinstance(meta, dict) and ("usd" in meta or "usd_per_min" in meta):
                out[sku_id] = float(meta.get("usd", meta.get("usd_per_min", 0.0)))
    return out


class _TableExtractor(html.parser.HTMLParser):
    """Lightweight stdlib HTML table parser extracting rows with optional data-sku."""

    def __init__(self) -> None:
        super().__init__()
        self.rows: list[tuple[str | None, list[str]]] = []
        self._in_tr = False
        self._tr_sku: str | None = None
        self._in_cell = False
        self._cell_buf: list[str] = []
        self._current_row: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        t = tag.lower()
        if t == "tr":
            self._in_tr = True
            self._current_row = []
            attr_map = dict(attrs)
            self._tr_sku = attr_map.get("data-sku")
        elif t in ("td", "th") and self._in_tr:
            self._in_cell = True
            self._cell_buf = []

    def handle_endtag(self, tag: str) -> None:
        t = tag.lower()
        if t in ("td", "th") and self._in_cell:
            self._in_cell = False
            self._current_row.append(" ".join("".join(self._cell_buf).split()))
        elif t == "tr" and self._in_tr:
            self._in_tr = False
            if self._current_row:
                self.rows.append((self._tr_sku, self._current_row))
            self._tr_sku = None

    def handle_data(self, data: str) -> None:
        if self._in_cell:
            self._cell_buf.append(data)


def _extract_rows(text: str) -> list[tuple[str | None, list[str]]]:
    parser = _TableExtractor()
    try:
        parser.feed(text)
    except Exception:
        pass
    rows = list(parser.rows)
    # Also support Markdown tables: | col1 | col2 | ... |
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("|") and s.endswith("|") and "---" not in s:
            cells = [c.strip() for c in s.strip("|").split("|")]
            if cells:
                rows.append((None, cells))
    return rows


USD_RE = re.compile(r"\$\s*(\d+(?:\.\d+)?)")


def _extract_usd_values(cells: list[str]) -> list[float]:
    vals: list[float] = []
    for cell in cells:
        for m in USD_RE.finditer(cell):
            vals.append(float(m.group(1)))
    return vals


def parse_pricing_doc(
    platform: str,
    text: str,
    baseline_data: dict[str, Any] | None = None,
) -> dict[str, float]:
    """Parse SKU USD/min rates from an HTML or Markdown pricing document.

    Raises ParserError if no recognizable rate table or rates are found.
    """
    if not text or not text.strip():
        raise ParserError(f"Empty pricing document for platform '{platform}'")

    rows = _extract_rows(text)
    known_skus = set(_baseline_minute_skus(baseline_data).keys()) | set(
        EXPECTED_ANCHOR_RATES.get(platform, {}).keys()
    )
    parsed: dict[str, float] = {}

    # 1. Direct row matching via data-sku or first-column SKU identifier
    for tr_sku, cells in rows:
        usd_vals = _extract_usd_values(cells)
        if not usd_vals:
            continue
        rate = usd_vals[-1]
        if tr_sku:
            parsed[tr_sku] = rate
            continue
        first_col = cells[0].strip().strip("`")
        if first_col in known_skus:
            parsed[first_col] = rate

    # 2. Heuristic table row matching for live vendor docs without data-sku
    for _, cells in rows:
        row_text = " ".join(cells).lower()
        usd_vals = _extract_usd_values(cells)
        if platform == "github" and usd_vals:
            rate = usd_vals[-1]
            if "linux" in row_text and "2-core" in row_text and "arm" not in row_text:
                parsed.setdefault("linux", rate)
                parsed.setdefault("linux_x64_2core", rate)
            elif "windows" in row_text and "2-core" in row_text and "arm" not in row_text:
                parsed.setdefault("windows", rate)
                parsed.setdefault("windows_x64_2core", rate)
            elif ("macos" in row_text or "mac" in row_text) and ("3-core" in row_text or "standard" in row_text):
                parsed.setdefault("macos", rate)
                parsed.setdefault("macos_3_4core", rate)
            elif "linux" in row_text and "4-core" in row_text and "arm" not in row_text and "gpu" not in row_text:
                parsed.setdefault("linux_x64_4core", rate)
            elif "linux" in row_text and "8-core" in row_text and "arm" not in row_text:
                parsed.setdefault("linux_x64_8core", rate)
            elif "linux" in row_text and "16-core" in row_text and "arm" not in row_text:
                parsed.setdefault("linux_x64_16core", rate)
            elif "linux" in row_text and "32-core" in row_text and "arm" not in row_text:
                parsed.setdefault("linux_x64_32core", rate)
            elif "linux" in row_text and "64-core" in row_text and "arm" not in row_text:
                parsed.setdefault("linux_x64_64core", rate)
        elif platform == "gitlab":
            if "saas-linux-small-amd64" in row_text:
                if usd_vals:
                    parsed.setdefault("linux", usd_vals[-1])
                    parsed.setdefault("linux_x64_small", usd_vals[-1])
            elif "saas-linux-medium-amd64" in row_text and "gpu" not in row_text:
                if usd_vals:
                    parsed.setdefault("linux_x64_medium", usd_vals[-1])
            elif "saas-linux-large-amd64" in row_text:
                if usd_vals:
                    parsed.setdefault("linux_x64_large", usd_vals[-1])
            elif "saas-linux-xlarge-amd64" in row_text:
                if usd_vals:
                    parsed.setdefault("linux_x64_xlarge", usd_vals[-1])
            elif "saas-linux-2xlarge-amd64" in row_text:
                if usd_vals:
                    parsed.setdefault("linux_x64_2xlarge", usd_vals[-1])
            elif "saas-macos-medium-m1" in row_text:
                if usd_vals:
                    parsed.setdefault("macos", usd_vals[-1])
                    parsed.setdefault("macos_medium_m1", usd_vals[-1])
            elif "windows-shared" in row_text:
                if usd_vals:
                    parsed.setdefault("windows", usd_vals[-1])
                    parsed.setdefault("windows_standard", usd_vals[-1])
        elif platform == "circleci" and usd_vals:
            rate = usd_vals[-1]
            if "docker" in row_text and "small" in row_text:
                parsed.setdefault("linux_x64_small", rate)
            elif "docker" in row_text and "medium+" in row_text:
                parsed.setdefault("linux_x64_medium_plus", rate)
            elif "docker" in row_text and "medium" in row_text:
                parsed.setdefault("linux_x64_medium", rate)
            elif "docker" in row_text and "xlarge" in row_text:
                parsed.setdefault("linux_x64_xlarge", rate)
            elif "docker" in row_text and "large" in row_text:
                parsed.setdefault("linux_x64_large", rate)
        elif platform == "bitbucket" and usd_vals:
            rate = usd_vals[-1]
            for mult in ("1x", "2x", "4x", "8x", "16x"):
                if re.search(rf"\b{mult}\b", row_text):
                    parsed.setdefault(f"linux_{mult}", rate)

    if not parsed:
        raise ParserError(
            f"Parser failed for platform '{platform}': no runner SKU rates found in document"
        )

    anchors = EXPECTED_ANCHOR_RATES.get(platform, {})
    if anchors and not any(k in parsed for k in anchors):
        raise ParserError(
            f"Parser failed for platform '{platform}': none of the expected anchor SKUs "
            f"({', '.join(sorted(anchors))}) were found"
        )

    return parsed


def diff_platform_rates(
    platform: str,
    baseline_data: dict[str, Any],
    parsed_rates: dict[str, float],
) -> dict[str, Any]:
    """Compare parsed SKU rates against baseline pricing/<platform>.json."""
    baseline_skus = _baseline_minute_skus(baseline_data)
    diffs: list[dict[str, Any]] = []
    compared_count = 0

    for sku, old_rate in baseline_skus.items():
        if sku not in parsed_rates:
            continue
        compared_count += 1
        old_val = round(float(old_rate), 6)
        new_val = round(float(parsed_rates[sku]), 6)
        if abs(old_val - new_val) > 1e-7:
            diffs.append(
                {
                    "sku": sku,
                    "field": "usd",
                    "old": old_val,
                    "new": new_val,
                }
            )

    if compared_count == 0:
        raise ParserError(
            f"Parser extracted {len(parsed_rates)} rates for '{platform}', "
            f"but 0 matched SKUs in pricing/{platform}.json"
        )

    return {
        "platform": platform,
        "source_label": baseline_data.get("source_label", platform),
        "source_url": baseline_data.get("source_url", ""),
        "checked_on": baseline_data.get("checked_on", ""),
        "status": "changed" if diffs else "unchanged",
        "compared_skus": compared_count,
        "diffs": diffs,
        "error": None,
    }


def format_markdown_summary(report: dict[str, Any]) -> str:
    """Render a human-readable Markdown summary for PR or GitHub issue bodies."""
    lines = [
        f"# Weekly CI/CD Price Watch (`{report['run_date']}`)",
        "",
        f"- **Overall status:** `{report['status']}`",
        "",
        "| Platform | Status | Checked On | Compared SKUs | Diffs | Source |",
        "|---|---|---|---|---|---|",
    ]
    for p in report.get("platforms", []):
        src = p.get("source_url") or ""
        src_link = f"[{p.get('source_label', p['platform'])}]({src})" if src else p["platform"]
        lines.append(
            f"| `{p['platform']}` | `{p['status']}` | `{p.get('checked_on', '')}` | "
            f"{p.get('compared_skus', 0)} | {len(p.get('diffs', []))} | {src_link} |"
        )

    changed = [p for p in report.get("platforms", []) if p["status"] == "changed"]
    if changed:
        lines.extend(["", "## Rate Changes Detected", ""])
        for p in changed:
            lines.append(f"### `{p['platform']}` ({p.get('source_url', '')})")
            lines.append("")
            lines.append("| SKU | Field | Old Rate (USD/min) | New Rate (USD/min) |")
            lines.append("|---|---|---|---|")
            for d in p["diffs"]:
                lines.append(
                    f"| `{d['sku']}` | `{d['field']}` | `${d['old']:.4f}` | `${d['new']:.4f}` |"
                )
            lines.append("")

    errors = [p for p in report.get("platforms", []) if p["status"] == "parser_error"]
    if errors:
        lines.extend(["", "## Parser Errors", ""])
        for p in errors:
            lines.append(
                f"- **`{p['platform']}`** ({p.get('source_url', '')}): `{p.get('error', 'unknown error')}`"
            )

    return "\n".join(lines).rstrip() + "\n"


def bump_checked_on_and_sync(
    checked_date: str,
    pricing_dir: Path = PRICING_DIR,
    site_pricing_dir: Path = SITE_PRICING_DIR,
) -> list[str]:
    """Update checked_on in pricing/*.json and regenerate pricing/*.csv + site/pricing/*.csv."""
    if not ISO_DATE_RE.match(checked_date):
        raise ValueError(f"Invalid ISO date for checked_on: {checked_date!r}")

    site_pricing_dir.mkdir(parents=True, exist_ok=True)
    updated: list[str] = []
    for json_path in sorted(pricing_dir.glob("*.json")):
        if json_path.name == "learned_sku_registry.json":
            continue
        platform = json_path.stem
        data = json.loads(json_path.read_text(encoding="utf-8"))
        data["checked_on"] = checked_date
        if isinstance(data.get("skus"), list):
            for row in data["skus"]:
                if isinstance(row, dict) and "checked_on" in row:
                    row["checked_on"] = checked_date
        json_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

        old_dir = pricing.PRICING_DIR
        try:
            pricing.PRICING_DIR = pricing_dir
            csv_path = pricing.export_csv(platform)
            shutil.copyfile(csv_path, site_pricing_dir / f"{platform}.csv")
        finally:
            pricing.PRICING_DIR = old_dir

        updated.append(platform)
    return updated


def _fetch_url(url: str, timeout: int = 20) -> str:
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "Skipwise-PricingWatch/1.0 (+https://skipwise.raza-aamir01.workers.dev/pricing/)",
            "Accept": "text/html,application/xhtml+xml,text/plain;q=0.9,*/*;q=0.8",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        charset = resp.headers.get_content_charset() or "utf-8"
        return resp.read().decode(charset, errors="replace")


def run_fetch(
    bump_checked_on: bool = False,
    fixture_dir: Path | None = None,
    out_path: Path | None = None,
    markdown_out: Path | None = None,
) -> dict[str, Any]:
    """Fetch official pricing docs (or fixture files) for all platforms in pricing/*.json."""
    today = dt.datetime.now(dt.timezone.utc).date().isoformat()
    platform_results: list[dict[str, Any]] = []

    for json_path in sorted(PRICING_DIR.glob("*.json")):
        if json_path.name == "learned_sku_registry.json":
            continue
        platform = json_path.stem
        baseline = json.loads(json_path.read_text(encoding="utf-8"))
        source_url = str(baseline.get("source_url") or "")
        try:
            if fixture_dir is not None:
                fx_file = fixture_dir / f"{platform}.html"
                if not fx_file.exists():
                    raise ParserError(f"Missing fixture file {fx_file}")
                doc_text = fx_file.read_text(encoding="utf-8")
            else:
                if not source_url.startswith("https://"):
                    raise ParserError(f"Invalid source_url for {platform}: {source_url!r}")
                doc_text = _fetch_url(source_url)

            parsed_rates = parse_pricing_doc(platform, doc_text, baseline_data=baseline)
            res = diff_platform_rates(platform, baseline, parsed_rates)
            platform_results.append(res)
        except (ParserError, urllib.error.URLError, OSError) as exc:
            platform_results.append(
                {
                    "platform": platform,
                    "source_label": baseline.get("source_label", platform),
                    "source_url": source_url,
                    "checked_on": baseline.get("checked_on", ""),
                    "status": "parser_error",
                    "compared_skus": 0,
                    "diffs": [],
                    "error": str(exc),
                }
            )

    statuses = {p["status"] for p in platform_results}
    if "parser_error" in statuses:
        overall = "parser_error"
    elif "changed" in statuses:
        overall = "changed"
    else:
        overall = "unchanged"

    bumped_platforms: list[str] = []
    if overall == "unchanged" and bump_checked_on:
        bumped_platforms = bump_checked_on_and_sync(today)
        for p in platform_results:
            p["checked_on"] = today

    report: dict[str, Any] = {
        "run_date": today,
        "status": overall,
        "bumped_checked_on": bumped_platforms,
        "platforms": platform_results,
    }

    if out_path is not None:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    if markdown_out is not None:
        markdown_out.parent.mkdir(parents=True, exist_ok=True)
        markdown_out.write_text(format_markdown_summary(report), encoding="utf-8")

    gh_output = os.environ.get("GITHUB_OUTPUT")
    if gh_output:
        with open(gh_output, "a", encoding="utf-8") as f:
            f.write(f"status={overall}\n")
            f.write(f"run_date={today}\n")

    return report


def run_self_test() -> None:
    """Deterministic offline self-test of pricing/*.json, parsers, diffing, and CSV sync."""
    # 1. Validate existing pricing/*.json and CSV sync via engine.pricing
    summary = pricing.check_all()
    assert isinstance(summary, dict) and "github" in summary and "gitlab" in summary, (
        f"engine.pricing.check_all failed: {summary}"
    )

    json_files = [
        p for p in sorted(PRICING_DIR.glob("*.json")) if p.name != "learned_sku_registry.json"
    ]
    assert len(json_files) >= 2, "Expected at least github.json and gitlab.json in pricing/"

    for json_path in json_files:
        p = json_path.stem
        data = json.loads(json_path.read_text(encoding="utf-8"))
        assert str(data.get("source_url", "")).startswith("https://"), f"{p}: invalid source_url"
        assert ISO_DATE_RE.match(str(data.get("checked_on", ""))), f"{p}: invalid checked_on"
        assert str(data.get("source_label", "")).strip(), f"{p}: missing source_label"

        csv_path = PRICING_DIR / f"{p}.csv"
        site_csv_path = SITE_PRICING_DIR / f"{p}.csv"
        expected_csv = pricing.render_csv(p)
        assert csv_path.exists() and csv_path.read_text(encoding="utf-8") == expected_csv, (
            f"{csv_path} is out of sync with {json_path}"
        )
        assert site_csv_path.exists() and site_csv_path.read_text(encoding="utf-8") == expected_csv, (
            f"{site_csv_path} is out of sync with {json_path}"
        )

    # 2. Verify HTML/Markdown table parser on all 4 platforms (github, gitlab, circleci, bitbucket)
    for platform in ALL_KNOWN_PLATFORMS:
        snippet = EMBEDDED_DOC_SNIPPETS[platform]
        baseline_path = PRICING_DIR / f"{platform}.json"
        baseline = (
            json.loads(baseline_path.read_text(encoding="utf-8"))
            if baseline_path.exists()
            else None
        )
        parsed = parse_pricing_doc(platform, snippet, baseline_data=baseline)
        for anchor_sku, expected_rate in EXPECTED_ANCHOR_RATES[platform].items():
            assert anchor_sku in parsed, f"{platform}: missing anchor SKU {anchor_sku}"
            assert abs(parsed[anchor_sku] - expected_rate) < 1e-7, (
                f"{platform}.{anchor_sku}: expected {expected_rate}, got {parsed[anchor_sku]}"
            )
        if baseline is not None:
            diff_res = diff_platform_rates(platform, baseline, parsed)
            assert diff_res["status"] == "unchanged" and not diff_res["diffs"], (
                f"Expected unchanged rates for {platform}, got: {diff_res}"
            )

    # 3. Verify rate-change detection on a mutated snippet
    gh_baseline = json.loads((PRICING_DIR / "github.json").read_text(encoding="utf-8"))
    mutated_gh_html = EMBEDDED_DOC_SNIPPETS["github"].replace(
        '<tr data-sku="linux"><td>linux</td><td>Linux 2-core (standard)</td><td>$0.006</td></tr>',
        '<tr data-sku="linux"><td>linux</td><td>Linux 2-core (standard)</td><td>$0.008</td></tr>',
    )
    parsed_mutated = parse_pricing_doc("github", mutated_gh_html, baseline_data=gh_baseline)
    diff_mutated = diff_platform_rates("github", gh_baseline, parsed_mutated)
    assert diff_mutated["status"] == "changed", "Expected status='changed' for mutated rate"
    assert len(diff_mutated["diffs"]) == 1 and diff_mutated["diffs"][0] == {
        "sku": "linux",
        "field": "usd",
        "old": 0.006,
        "new": 0.008,
    }, f"Unexpected diff payload: {diff_mutated}"

    md_summary = format_markdown_summary(
        {
            "run_date": "2026-11-01",
            "status": "changed",
            "platforms": [diff_mutated],
        }
    )
    assert "$0.0060" in md_summary and "$0.0080" in md_summary and "docs.github.com" in md_summary

    # 4. Verify parser-error detection on a corrupted/empty page
    try:
        parse_pricing_doc("github", "<html><body><h1>Page Moved</h1></body></html>")
        raise AssertionError("Expected ParserError on page without pricing table")
    except ParserError:
        pass

    # 5. Verify bump_checked_on_and_sync in an isolated temp directory
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        tmp_pricing = tmp_path / "pricing"
        tmp_site_pricing = tmp_path / "site_pricing"
        shutil.copytree(PRICING_DIR, tmp_pricing)
        shutil.copytree(SITE_PRICING_DIR, tmp_site_pricing)

        updated = bump_checked_on_and_sync("2026-11-08", tmp_pricing, tmp_site_pricing)
        assert "github" in updated and "gitlab" in updated
        gh_updated = json.loads((tmp_pricing / "github.json").read_text(encoding="utf-8"))
        assert gh_updated["checked_on"] == "2026-11-08"
        assert "2026-11-08" in (tmp_pricing / "github.csv").read_text(encoding="utf-8")
        assert "2026-11-08" in (tmp_site_pricing / "github.csv").read_text(encoding="utf-8")

    # 6. Verify report.py and email_report.py include price provenance and CSV link (if present)
    fx_path = ROOT / "tools" / "fixtures" / "report_inputs.json"
    if fx_path.exists():
        try:
            from engine import email_report, report  # noqa: PLC0415

            fx = json.loads(fx_path.read_text(encoding="utf-8"))
            rates, given = report.load_rates()
            doc, _ = report.build(fx["audit"], fx["flaky"], fx["selection"], fx["safety"], rates, given)
            md_text = report.markdown(doc)
            html_text = report.html(doc, "CI Rules Audit")
            email_html = email_report.render_email(fx["audit"], fx["flaky"])
            expected_prov = f"Prices: {gh_baseline['source_label']}, checked {gh_baseline['checked_on']}"
            for label, content in (("report.md", md_text), ("report.html", html_text), ("email.html", email_html)):
                assert expected_prov in content, f"{label} missing provenance string {expected_prov!r}"
                assert "https://skipwise.raza-aamir01.workers.dev/pricing/github.csv" in content, f"{label} missing CSV URL"
            assert not re.search(r"<(script|link|img)\b|src=|url\(", html_text), "V26 self-contained HTML violated"
        except ImportError:
            pass

    print(
        json.dumps(
            {
                "status": "PASS",
                "checked_platforms": [p.stem for p in json_files],
                "tested_parsers": list(ALL_KNOWN_PLATFORMS),
            }
        )
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="CI/CD pricing watch and self-test tool")
    parser.add_argument(
        "--self-test",
        "--check",
        dest="self_test",
        action="store_true",
        help="Run deterministic offline self-test of pricing/*.json, parsers, and diffing",
    )
    parser.add_argument(
        "--fetch",
        action="store_true",
        help="Fetch live rank-1 official pricing pages and diff against pricing/*.json",
    )
    parser.add_argument(
        "--bump-checked-on",
        action="store_true",
        help="When --fetch finds all rates unchanged, bump checked_on to today and sync CSVs",
    )
    parser.add_argument(
        "--fixture-dir",
        type=Path,
        default=None,
        help="Optional directory of <platform>.html fixtures for offline --fetch testing",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Write structured JSON diff report to this file (e.g. pricing_diff.json)",
    )
    parser.add_argument(
        "--markdown-out",
        type=Path,
        default=None,
        help="Write Markdown summary report to this file (e.g. pricing_diff.md)",
    )
    parser.add_argument(
        "--strict-exit",
        action="store_true",
        help="Exit non-zero when status is parser_error (exit 2) or changed (exit 1)",
    )
    args = parser.parse_args()

    if args.self_test or not args.fetch:
        run_self_test()
        return

    report = run_fetch(
        bump_checked_on=args.bump_checked_on,
        fixture_dir=args.fixture_dir,
        out_path=args.out,
        markdown_out=args.markdown_out,
    )
    print(json.dumps(report, indent=2))
    if args.strict_exit:
        if report["status"] == "parser_error":
            sys.exit(2)
        if report["status"] == "changed":
            sys.exit(1)


if __name__ == "__main__":
    main()
