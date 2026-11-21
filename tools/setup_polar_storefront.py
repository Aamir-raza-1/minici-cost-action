#!/usr/bin/env python3
"""
Automates Polar.sh Storefront & Recurring Tier setup for `minici` via Polar's
official REST API (`https://api.polar.sh/v1`).

Usage:
  POLAR_OAT="polar_oat_..." python3 tools/setup_polar_storefront.py
"""
from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request

API_BASE = "https://api.polar.sh/v1"
ORG_SLUG = os.environ.get("POLAR_ORG_SLUG", "minici")


def api_call(method: str, path: str, token: str, payload: dict | None = None) -> dict:
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(
        f"{API_BASE}{path}",
        data=data,
        method=method,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": "minici-polar-setup/0.9.0",
        },
    )
    with urllib.request.urlopen(req, timeout=20) as resp:
        return json.loads(resp.read().decode("utf-8"))


def main() -> None:
    token = os.environ.get("POLAR_OAT") or os.environ.get("POLAR_ACCESS_TOKEN") or ""
    if not token.strip():
        print(
            "Error: Set POLAR_OAT='polar_oat_...' (create in https://polar.sh/dashboard/minici/settings -> Developers).",
            file=sys.stderr,
        )
        sys.exit(1)

    orgs = api_call("GET", f"/organizations/?slug={ORG_SLUG}", token)
    items = orgs.get("items") or []
    if not items:
        print(f"Error: Organization '{ORG_SLUG}' not found for this token.", file=sys.stderr)
        sys.exit(1)
    org_id = items[0]["id"]
    print(f"Found Polar organization '{ORG_SLUG}' (id={org_id})")

    # 1. Enable public storefront (`https://polar.sh/minici`)
    api_call(
        "PATCH",
        f"/organizations/{org_id}",
        token,
        {
            "profile_settings": {
                "enabled": True,
                "description": "Sub-millisecond CI/CD Cost Breakdown, Pull Request Cost Diff Action & 8-Tool MCP FinOps Engine.",
            }
        },
    )
    print("Enabled Polar public storefront -> https://polar.sh/minici")

    # 2. Ensure recurring Sponsor ($15/mo) and Team Pro ($79/mo) products exist
    existing = api_call("GET", f"/products/?organization_id={org_id}", token).get("items") or []
    existing_names = {p.get("name") for p in existing}

    tiers = [
        {
            "name": "MiniCI Open-Source Sponsor",
            "description": "Support open-source Rust std-only CI/CD FinOps research, 7-provider SKU catalog updates, and benchmark studies.",
            "recurring_interval": "month",
            "prices": [{"amount_type": "fixed", "price_amount": 1500, "price_currency": "usd"}],
        },
        {
            "name": "MiniCI Team Pro (Private Repos & Slack FinOps Bot)",
            "description": "Priority support, Slack FinOps Bot integration, FOCUS v1.1 CSV reconciliation & multi-repo org governance.",
            "recurring_interval": "month",
            "prices": [{"amount_type": "fixed", "price_amount": 7900, "price_currency": "usd"}],
        },
    ]

    for tier in tiers:
        if tier["name"] in existing_names:
            print(f"Product already exists: {tier['name']}")
            continue
        created = api_call("POST", "/products/", token, {**tier, "organization_id": org_id})
        print(f"Created Polar product: {created.get('name')} (id={created.get('id')})")

    print("Done! Verify live storefront: curl -sI https://polar.sh/minici")


if __name__ == "__main__":
    try:
        main()
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        print(f"Polar API HTTP {exc.code}: {body}", file=sys.stderr)
        sys.exit(1)
