#!/usr/bin/env bash
# ==============================================================================
# MiniCI Cost Action (Feature 1) — 10-Repository x 3-Mode Standalone Validator
# ==============================================================================
# Usage:
#   bash /Users/araza/personal/minici-cost-action/scripts/validate_minici_cost_10_repos.sh
#
# Validates `minici` (v0.2.0) across 10 real-world repositories:
#   1. MaximVote/maxim              (Internal Private • 41 workflows, 70 jobs)
#   2. mgjartproject/FineArts-BE    (Internal Private • Grassy Fine Arts Backend)
#   3. Eazy-Ops/Backend             (Internal Private • Eazy-Ops Backend)
#   4. astral-sh/uv                 (Fresh OSS • 47 workflows, 243 jobs)
#   5. denoland/deno                (Fresh OSS • 11 workflows, 162 jobs)
#   6. ggml-org/llama.cpp           (Fresh OSS • 62 workflows, 157 jobs)
#   7. immich-app/immich            (Fresh OSS • 27 workflows, 87 jobs)
#   8. langgenius/dify              (Fresh OSS • 41 workflows, 128 jobs)
#   9. open-webui/open-webui        (Fresh OSS • 7 workflows, 18 jobs)
#  10. mendableai/firecrawl         (Fresh OSS • 36 workflows, 86 jobs)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CACHE_DIR="${HOME}/.minici/10-repos-cache"

echo "▶ [1/3] Checking 2026 Canonical SKU Sync (catalog.rs <-> pricing/*.json)..."
python3 "${REPO_ROOT}/tools/sync_rust_sku_catalog.py" --check

echo ""
echo "▶ [2/3] Running 10-Repository x 3-Mode Validation (CLI, GitHub Workflow, MCP/Agent + Head-to-Head vs cicdcost.com & Infracost)..."
python3 "${REPO_ROOT}/tools/verify_10_repos_all_modes.py" --cache-dir "${CACHE_DIR}"

echo ""
echo "▶ [3/3] Verifying CLI Catalog & Benchmark Subcommands..."
"${REPO_ROOT}/target/release/minici" catalog | head -n 18
echo "✓ All 10 repositories validated across CLI, GitHub Workflow, and AI Agent (MCP/Hook) modes."
