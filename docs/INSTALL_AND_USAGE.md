# 📘 MiniCI (`minici-cost-action`) — Installation & Complete Usage Guide

This guide walks through every way to install and run `minici` (`v0.7.0`) as a **Local CLI**, **GitHub Action**, **Git Pre-Push Hook**, and **MCP Server for AI Coding Agents**.

* ▶️ **Video Walkthrough (`.mp4`):** [`../web/how-to-install-and-use.mp4`](../web/how-to-install-and-use.mp4)
* 📼 **Asciinema Recording (`.cast`):** [`../web/how-to-install-and-use.cast`](../web/how-to-install-and-use.cast)

---

## 1. Local CLI Installation

`crates/minici` uses **100% Rust standard library (`std`) with zero external crate dependencies**. It compiles cleanly in `~1.5 seconds` even in offline or air-gapped environments.

### Option A: Build from Source (Recommended)

```bash
git clone https://github.com/Aamir-raza-1/minici-cost-action.git
cd minici-cost-action
cargo build --release --manifest-path crates/minici/Cargo.toml

mkdir -p ~/.local/bin
cp crates/minici/target/release/minici ~/.local/bin/minici
minici --version
# -> minici 0.7.0
```

### Option B: Install via `cargo install`

```bash
cargo install --git https://github.com/Aamir-raza-1/minici-cost-action.git minici
```

---

## 2. Everyday CLI Workflows

### A. Repository Cost & Quota Receipt (`minici breakdown`)

Run inside any repository containing `.github/workflows/*.yml`, `.gitlab-ci.yml`, `.circleci/config.yml`, `bitbucket-pipelines.yml`, `buildspec.yml`, `cloudbuild.yaml`, or `azure-pipelines.yml`:

```bash
minici breakdown --path .
```

Customize team activity assumptions to match your engineering organization:

```bash
minici breakdown \
  --path . \
  --tier paid \
  --prs-per-week 40 \
  --pushes-per-pr 3 \
  --dev-hourly-rate 85
```

* `--tier <auto|free|paid>`: `auto` checks whether monthly OS-weighted minutes exceed your provider's free-tier quota (`2,000 mins/mo` on GitHub Free) and switches to paid rates automatically when exceeded.
* `--prs-per-week <NUM>` & `--pushes-per-pr <NUM>`: Controls monthly CI run projection (`prs_per_week * pushes_per_pr * 4.33`).
* `--dev-hourly-rate <USD>`: Engineering hourly rate for critical-path wait-time & context-switch payroll calculations (`0` disables wait-time math).

### B. Pull Request / Branch Cost Delta (`minici diff`)

Before pushing a workflow change, compare your working tree against `main`, `origin/main`, `HEAD~1`, or another directory:

```bash
# Compare uncommitted/committed branch changes against `main`
minici diff --path . --compare-to main

# Compare against the previous commit
minici diff --path . --compare-to HEAD~1

# Emit a sticky-ready GitHub Pull Request Markdown diff table
minici diff --path . --compare-to main --format markdown --out-file pr-cost-diff.md
```

### C. Zero-Clone Public GitHub Repository Audit (`--repo`)

Inspect any public GitHub repository's CI spend without cloning its source tree:

```bash
minici breakdown --repo open-webui/open-webui
minici breakdown --repo https://github.com/denoland/deno

# Force refresh cached workflows (~/.minici/cache/repos/)
minici breakdown --repo open-webui/open-webui --refresh
```

> **Tip:** Export `GITHUB_TOKEN=ghp_...` to raise GitHub REST API rate limits from `60 req/hr` to `5,000 req/hr` and access private repositories you have permission to read.

### D. Multi-Format Reports (`html`, `json`, `markdown`, `csv`)

```bash
minici breakdown --path . --report ci-executive-report.html
minici breakdown --path . --report ci-breakdown.json --quiet
minici breakdown --path . --report ci-jobs.csv
minici breakdown --path . --report ci-summary.md
```

### E. Pricing SKU Catalog & Live Override Sync (`catalog` & `self-update`)

```bash
# Print all compiled SaaS & Cloud runner SKUs as JSON
minici catalog

# Verify 0.00% catalog drift and refresh local override cache (~/.minici/pricing_overrides.json)
minici self-update
```

---

## 3. GitHub Action Integration (`action.yml`)

Add `.github/workflows/minici-cost.yml` to post job summaries and expose structured FinOps outputs to downstream workflow steps:

```yaml
name: MiniCI Cost Receipt
on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read
  pull-requests: write

jobs:
  cost-receipt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Compute CI Cost Breakdown
        id: cost
        uses: Aamir-raza-1/minici-cost-action@v1
        with:
          path: "."
          tier: "auto"
          prs-per-week: "25"
          pushes-per-pr: "4"
          dev-hourly-rate: "65"

      - name: Print Key FinOps Outputs
        run: |
          echo "Gross Monthly Compute ($): ${{ steps.cost.outputs.gross_monthly_compute_cost_usd }}"
          echo "Net Billed Monthly ($):    ${{ steps.cost.outputs.net_billed_monthly_compute_cost_usd }}"
          echo "ARM64 Savings ($/mo):      ${{ steps.cost.outputs.arm64_migration_monthly_savings_usd }}"
          echo "Detected Stack:            ${{ steps.cost.outputs.detected_stack_archetype }}"
```

### Action Inputs

| Input | Default | Description |
| :--- | :---: | :--- |
| `path` | `.` | Repository root or workflow directory |
| `compare-to` | `""` | Optional base Git ref (e.g., `origin/main`) to also append a PR cost delta table |
| `tier` | `auto` | Pricing tier mode (`auto`, `free`, or `paid`) |
| `prs-per-week` | `25` | Pull requests per week for monthly run projection |
| `pushes-per-pr` | `4` | Average pushes per pull request |
| `dev-hourly-rate` | `65` | Developer hourly rate in USD for wait-time calculations |

---

## 4. Shift-Left Git Pre-Push Hook & MCP Server

Run `minici init-hooks --path .` inside any repository to scaffold:
1. `.git/hooks/pre-push` — Prints a non-blocking `< 2 ms` cost delta receipt whenever `.github/workflows/` changes before `git push`.
2. `.mcp.json` — Registers the stdio JSON-RPC 2.0 MCP server (`minici mcp --path .`) so AI agents can call `minici_breakdown`, `minici_diff`, and `minici_catalog` directly during workflow edits.

Test the MCP server directly from your shell:

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"minici_breakdown"}}\n' | minici mcp --path .
```

---

## 5. Pairing with `minici-yaml-fixer`

Once `minici breakdown` highlights missing `cancel-in-progress`, uncached dependencies, bloated `fetch-depth: 0` clones, or eligible `x86_64 -> ARM64` runner savings, run [`minici-yaml-fixer`](https://github.com/Aamir-raza-1/minici-yaml-fixer) to patch the YAML automatically while preserving every `#` comment:

```bash
minici-yaml-fixer fix --workflow .github/workflows
git apply .minici/workflow-fixes.patch
minici diff --path . --compare-to HEAD
```
