# FinOps & CI/CD Pricing Engine — Lessons Learned & Calculation Rules

This document is the permanent engineering log of calculation errors discovered during real-world repository and organization audits, the root causes behind those discrepancies, and the mandatory calculation rules now enforced in [`engine/pricing.py`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py) and [`engine/billing_ingestor.py`](file:///Users/araza/personal/cicd-pricing-engine/engine/billing_ingestor.py).

---

## 1. Never Multiply Total Repository Workflow Runs by the Primary CI Pipeline Duration

### What Went Wrong
In our initial repository-level estimator, we counted the total number of daily workflow runs across the entire repository (`~418 runs/day` = `9,268–12,552 runs/month`) and multiplied that aggregate count by the billed duration of the primary test workflow (`51.8 billed mins/run`), producing an inflated estimate of `480,082 billed mins/month`.

### What Cross-Checking `/actions/metrics/performance` Proved
A repository rarely runs a single workflow. When we cross-checked the repository's actual `/actions/metrics/performance` and `/actions/metrics/usage` breakdowns:
- **Primary CI Workflow (`Continuous Integration - QA`):** Accounted for only **`26.3%`** of workflow runs (`~110 runs/day` = `3,301 runs/month` at `~61.6 ceil mins/run` = `203,342 mins/month`).
- **Short Compliance Workflows (`License Check`):** Accounted for **`27.5%`** of workflow runs (`~115 runs/day`), but each run took only **`1.2 minutes`** (`~4,140 mins/month`).
- **Conditional / Gatekeeper Workflows (`Preview Deploy`, `Release`, `CodeQL`):** Accounted for the remaining **`46.2%`** of triggered runs, where **`97%` (`245 of 252`)** exited in `0–1 seconds` as `skipped` (`0 billed minutes`).
- **True Monthly Billed Minutes:** **`209,850 ceil mins/month`** — a **`-56.3%` correction** compared to the naive aggregate multiplication.

### Mandatory Engine Rule
Always group workflow runs **by `workflow_id` / `workflow_name`** before extrapolating monthly compute minutes:
$$\text{Monthly Billed Minutes} = \sum_{w \in \text{Workflows}} \left( \text{Active Runs}_{w} \times \text{Avg Per-Job Ceil Minutes}_{w} \right)$$
Skipped runs (`conclusion == "skipped"` or `0s` job duration) must be excluded from active run averages (`0 billed minutes`).

---

## 2. Always Output Three Financial Numbers: Gross Consumed, Discounts/Allowances, and Net Billable Spend

### What Went Wrong
Estimators that only multiply `total_minutes * sku_rate` output **Gross List Consumption** and mistake it for what the customer actually pays on their invoice (**Net Billable Spend**).

### What Cross-Checking `Eazy-Ops` Official GitHub Billing Proved
When we audited the `Eazy-Ops` organization (`23 repositories`, `GitHub Free` plan) via [`https://github.com/organizations/Eazy-Ops/settings/billing`](https://github.com/organizations/Eazy-Ops/settings/billing) and `gh api /orgs/Eazy-Ops/settings/billing/usage`:
- **November 1–30, 2026 MTD:**
  - **Gross Metered Usage:** **`$0.14`** (`23.0 minutes` of `Actions Linux` $\times$ `$0.006/min` = `$0.138`)
  - **Included Usage Discount:** **`-$0.14`** (`-$0.138` covered by the `2,000 mins/month` GitHub Free allowance)
  - **Net Billable Due:** **`$0.00`**
- **Last Year Actions Usage (`period=6`):**
  - **Gross Consumed Usage:** **`$45.26`** (`$45.264` across `Backend $27.54`, `frontend $7.25`, `quotas $4.60`, `discovery $4.13`, `website $1.12`, `All other usage $0.62`)
  - **Discounts (Public Repositories + Included Monthly Minutes/Storage):** **`-$37.42`** (`-$37.416`, an **`82.7%` discount**)
  - **Net Billable Spend:** **`$7.85`** (`$7.848`)

### Mandatory Engine Rule
Every cost calculation and verification report must output and reconcile all three tiers:
1. **`gross_consumed_usd` (List / Metered Gross Usage):** Total usage at catalog list rates before plan allowances or public-repo exemptions.
2. **`discount_allowance_usd` (Included Plan Allowance + Public Repo Exemption):** Dollars zeroed out by `public_repo=True` or `included_minutes` / `included_storage_gb`.
3. **`net_billable_usd` (Actual Invoice Overage):** $\max(0, \text{Gross Consumed USD} - \text{Discounts USD})$.

---

## 3. Track Effective-Date Rate Changes in Official Billing History (`$0.008/min` in 2025 vs. `$0.006/min` in 2026)

### What We Discovered in `Eazy-Ops` API Telemetry
Querying `/orgs/Eazy-Ops/settings/billing/usage?year=2025` vs. `?year=2026` revealed that GitHub's historical billing line items carry different `pricePerUnit` values for the exact same `"Actions Linux"` SKU:
- **March–December 2025 (`year=2025`):** `sku: "Actions Linux"` was billed at **`$0.008/minute`** (`5,658 minutes` = `$45.264` gross).
- **January–November 2026 (`year=2026`):** `sku: "Actions Linux"` was billed at **`$0.006/minute`** (reflecting GitHub's `25%` hosted runner price reduction effective January 1, 2026).

### Mandatory Engine Rule
When reconciling historical billing exports or multi-month windows (`Last year` / `period=6`), never apply today's static rate (`$0.006/min`) blindly to 2025 minutes. Always honor the line-item `pricePerUnit` from the billing API/CSV when present, and date-bound catalog rate lookups for historical backtests.

---

## 4. Handle GitHub Enhanced Billing API Aggregation Quirks (`?year=YYYY` vs. Per-Month / UI Export)

### What We Discovered in `Eazy-Ops` API Telemetry
When querying `GET /orgs/{org}/settings/billing/usage?year=2025` without a `month` filter, GitHub's Enhanced Billing API returns **one aggregated row per `(month, product, sku)`** (summing to `$45.264` gross across the 10 active months), but populates the `repositoryName` field with a single sample repository from that month (`quotas` or `discovery`). Meanwhile, the GitHub Billing UI (`group=2&period=6&query=product:actions`) groups across the full underlying ledger by repository:
- `Backend`: **`$27.54`** (`60.85%`)
- `frontend`: **`$7.25`** (`16.02%`)
- `quotas`: **`$4.60`** (`10.16%`)
- `discovery`: **`$4.13`** (`9.13%`)
- `website`: **`$1.12`** (`2.47%`)
- `All other usage`: **`$0.62`** (`1.37%`)
- **Total Gross:** **`$45.26`**

### Mandatory Engine Rule
Do not trust a full-year ungrouped `/settings/billing/usage?year=YYYY` response for **per-repository** attribution unless queried at fine granularity or paired with the exported GitHub Billing CSV / UI repository breakdown.

---

## 5. Detect Third-Party Runner Labels (`blacksmith-*`, `warpbuild-*`, `buildjet-*`) and Cache Interception

### What Went Wrong
In repositories using third-party Firecracker/microVM runners (such as `blacksmith-8vcpu-ubuntu-2204` at `$0.016/min` and `blacksmith-4vcpu-ubuntu-2204` at `$0.008/min`), two things happen simultaneously:
1. Compute minutes are billed on the **third-party runner provider's invoice** (~28%–50% cheaper than GitHub's `$0.022/min` 2026 / `$0.032/min` 2025 8-core and `$0.012/min` 2026 / `$0.016/min` 2025 4-core rates), **not** on GitHub's metered Actions bill.
2. Third-party runners intercept `actions/cache` onto local co-located NVMe storage, causing GitHub's `/repos/{owner}/{repo}/actions/cache/usage` API to report `active_caches_size_in_bytes: 0`.

### Mandatory Engine Rule
Always inspect `job["labels"]` (from `/actions/runs/{id}/jobs`) and `runs-on:` in `.github/workflows/*.yml`:
- Classify labels matching `blacksmith-*`, `warpbuild-*`, `buildjet-*`, `namespace-*`, or `self-hosted` separately from GitHub-hosted standard/larger runners.
- Report both **GitHub Metered Invoice USD** and **Third-Party / Self-Hosted Runner Infrastructure USD** so the user sees why GitHub's billing page shows `$0` while the third-party runner invoice carries the compute spend.

---

## 6. Measure the "Sharding Setup Duplication Tax" and Critical-Path Wall Time Using `/graphs/traffic` & `/actions/metrics/performance`

### What We Learned
When an engineering team splits a slow test job into $N$ parallel shards (e.g., expanding 1 test job + 1 compatibility job into `4 + 4 + 2 = 10` parallel matrix containers):
1. **Git Clones Spike (`+172%` in `/graphs/traffic`):** Every parallel shard container runs `actions/checkout` and boots its own database/service containers. We observed daily Git clones jump from `~1,247 clones/day` to **`3,389 clones/day`** immediately after 4-way test sharding was merged.
2. **Setup Duplication Tax (`+29.3%` Billed Compute Minutes):** Because each of the 8 new shard containers pays a fixed `~2.0m` setup tax (`checkout` + `pnpm install` + Postgres/Redis boot + DB migrations) before running tests, sharding added **`~16.0 billed minutes/run`** (`+29.3%` compute cost per CI run).
3. **Critical-Path Wall Time Masking:** Despite reducing individual test shard duration from `~9m` to `~5.3m`, total workflow median wall time barely moved (`10.2m` $\rightarrow$ `9.75m`) because another job on the DAG (`Docker Build Validate` taking `6.4m` after a `4.0m` `Build` step = `10.4m`) became the **critical path**.

### Mandatory Engine KPIs
Our repository KPI analyzer must track and report:
- **`clones_per_ci_run`**: $\frac{\text{Daily Git Clones (`/traffic/clones`)}}{\text{Daily Active CI Runs}}$ — detects matrix fan-out and uncached checkout proliferation.
- **`shard_setup_duplication_minutes`**: $(N_{\text{shards}} - 1) \times t_{\text{setup}}$ — quantifies how many billed minutes are burned repeating identical container/database setup across shards.
- **`critical_path_wall_minutes` vs. `total_parallel_billed_minutes`**: Distinguishes developer wait time (DORA Lead Time for Changes, governed by the slowest sequential chain in the DAG) from FinOps billable compute (the sum of `ceil(duration_min)` across all parallel jobs).

---

## 7. Enforce a Generic 3-Tier Data Ingestion Architecture (Never Hardcode to One Repository)

To ensure our pricing and verification engine works out-of-the-box for **any** organization or repository regardless of permission level, [`engine/billing_ingestor.py`](file:///Users/araza/personal/cicd-pricing-engine/engine/billing_ingestor.py) implements a strict **3-Tier Ingestion Waterfall**:

| Tier | Mode | Data Source | When Used | Accuracy |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 1** | `official_api` | GitHub Enhanced Billing REST API (`/orgs/{org}/settings/billing/usage`) | User is Org Admin / Billing Manager (e.g., `Eazy-Ops`) | **100% Exact Invoice Reconciliation** (`Gross`, `Discount`, `Net`) |
| **Tier 2** | `billing_csv` | User-uploaded GitHub/Cloud Billing CSV or UI Summary Table (`--billing-csv`) | Finance/Admin exports CSV or pastes billing table | **100% Exact Invoice + Full Per-Repo Attribution** |
| **Tier 3** | `repo_telemetry` | Read-only Repo APIs (`/actions/runs`, `/actions/runs/{id}/jobs`, `/traffic/clones`, `/stats/commit_activity`, `/pulse`) | Developer has read-only repo access (`/settings/billing` returns `403/404`) | **Per-Workflow Ceil-Rounded Reconstruction** (filters skipped runs & classifies 3rd-party runners) |

---

## 8. Normalize Sign Conventions & Guard Against Falsy `0.0` in Billing APIs and CSV Exports (QA Sessions #1 & #2)

### What Went Wrong
1. GitHub's Billing UI and exported CSVs often format discount columns as negative strings (`"-$37.42"`, `"-0.37"`), while the REST API returns positive `discountAmount: 0.37`. Subtracting a negative discount (`gross - (-37.42)`) inverted the discount and double-charged the customer.
2. Using Python's truthiness operator (`item.get("netAmount") or max(0.0, gross - discount)`) treated a legitimate `netAmount = 0.0` as falsy (`False`), falling through to `gross - discount` and failing whenever `discount` was negative or omitted.
3. Summary CSV exports with multiple rows for the same repository (`Backend`) overwrote previous rows instead of accumulating, and zero-quantity placeholder rows (`quantity == 0`) polluted `historical_rate_shifts`.

### Mandatory Engine Rule
- Always normalize `discountAmount` with `abs(float(...))` so discounts represent positive dollar reductions.
- Never use `or` fallback chains on numeric financial fields where `0.0` is a valid terminal value; check `if parsed_net is not None` explicitly.
- Only record `pricePerUnit` in `observed_rates_by_sku` when `quantity > 0` and `pricePerUnit > 0`, and always accumulate duplicate repository rows (`+=`) in summary CSVs.

---

## 9. Enforce Identical Workflow Schema Semantics Across Tier 3 Telemetry and `calculate_monthly_cost(workflows=...)` (QA Session #3)

### What Went Wrong
[`estimate_from_repo_telemetry()`](file:///Users/araza/personal/cicd-pricing-engine/engine/billing_ingestor.py#L438) honored `shards`, `is_gatekeeper_skip`, `daily_runs`, and `skipped_runs_pct`, whereas [`calculate_monthly_cost(workflows=...)`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L818) initially read only `count`/`jobs` and `monthly_runs`/`monthly_skipped_runs`, causing the two entry points to diverge on sharded or gatekeeper-heavy workflows. Additionally, fallback `critical_path_wall_min` included skipped jobs when `critical_path_wall_min` was omitted.

### Mandatory Engine Rule
- Both [`estimate_from_repo_telemetry()`](file:///Users/araza/personal/cicd-pricing-engine/engine/billing_ingestor.py#L438) and [`calculate_monthly_cost(workflows=...)`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L818) must use the exact same job multiplier resolution (`shards` $\rightarrow$ `count` $\rightarrow$ `jobs`), gatekeeper skip filter (`dur_s <= 1.0 and is_gatekeeper_skip`), run clamping (`0 <= skipped_runs <= monthly_runs`), and active-job-only `critical_path_wall_min` fallback.

---

## 10. Propagate Provider & SKU Rounding Rules Into Every Job Duration Aggregator (QA Session #4)

### What Went Wrong
[`billed_minutes_for_jobs()`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L320) unconditionally applied `ceil_job_minutes()` (`ceil_minute_per_job`) even when called for `gitlab`, `circleci`, `gcp`, or AWS CodeBuild Lambda (`BUILD_LAMBDA_*`), which bill `exact_seconds`, or `bitbucket`, which bills `ceil_minute_per_step`. Furthermore, `0`-second jobs in `qa_verifier.py` were clamped to `max(1.0, duration_s)` (`1` billed minute instead of `0`).

### Mandatory Engine Rule
- Always resolve the target SKU's rounding mode (`exact_seconds`, `ceil_minute_per_step`, or `ceil_minute_per_job`) before aggregating job durations, and guard `duration_s <= 0 -> 0.0` prior to any `max(1.0, duration_s)` ceiling operation.

---

## 11. Strip OS Release Versions Before Extracting Runner Core Counts & Never Fall Back to Seat/Plan SKUs (QA Sessions #5 & #6)

### What Went Wrong
1. In Stage 2 heuristic regex parsing, bare OS version suffixes in `windows-2022`, `windows-2025`, `windows-11-arm64`, `macos-14`, and `macos-15` matched `(?:windows|macos)-(\d+)`, misparsing `2022` or `14` as CPU core counts (`windows_x64_96core` at `$0.552/min` — a **55.2x overcharge**!). Meanwhile, `macos-15-xlarge` and `macos-latest-xlarge` missed Apple Silicon M2 Pro (`macos_large_m2pro`, `$0.102/min`).
2. For non-GitHub platforms (`circleci`, `azure`), unrecognized runner labels fell through to `next(iter(smap.values()))`, which was `plan_free` (`$0.00`) on CircleCI and `user_basic_plan` (`$6.00/user_month` seat SKU!) on Azure.

### Mandatory Engine Rule
- Strip canonical OS release tokens (`ubuntu-(?:latest|(?:18|2\d)\.?\d{2})`, `windows-(?:latest|20\d{2}|1[01])`, `macos-(?:latest|1[1-9]|2\d)`) **before** running bare `-(\d+)` core-count regex extraction.
- Explicitly map macOS `-xlarge` to `macos_large_m2pro` (`12-core M2 Pro`, `$0.102/min`) and `-large` to `macos_12core_intel` (`12-core Intel`, `$0.077/min`).
- Stage 4 default fallback must always select a compute SKU (`smap.get("linux")`), never a subscription/seat/storage SKU, and `save_learned_sku_entry()` must cache in memory and guard disk writes with `try/except OSError`.

---

## 12. Weight `blended` Allowance Mode by OS Multipliers & Gate Non-Time SKU Units (QA Session #7)

### What Went Wrong
1. `allowance_mode="blended"` prorated `included_allowance` using raw unweighted minutes (`billed_min`) instead of multiplier-weighted pool units (`billed_min * multiplier`), ignoring GitHub's `2x` Windows and `10x` macOS multipliers and GitLab's `cost_factor`.
2. Passing `{"linux": 2000}` on `platform="azure"` resolved to `ms_hosted_parallel_job` (`unit="parallel_slot_month"`, `usd=40.0`) and multiplied `$40.00` by `200 overage minutes` (`$8,000.00` instead of `1 slot * $40.00 = $40.00`).

### Mandatory Engine Rule
- Every `allowance_mode` (`ordered` and `blended`) must operate strictly in **allowance pool units** (`pool_needed = billed_min * multiplier`, `covered_min = pool_used / multiplier`) so that `sum(pool_used) <= included_allowance`.
- Handle `unit == "parallel_slot_month"` via Azure's slot step-function (`0` slots when `overage_min <= 0`, else `max(1, ceil(billed_min / 7200)) * $40`), and raise a `ValueError` if non-compute SKUs (`gb_month`, `user_month`, `pipeline_month`, etc.) are passed into `minutes_by_sku`.

---

## 13. Compute Storage, Cache, Egress, and DLC Hidden Taxes Across All 7 Providers (QA Session #8)

### What Went Wrong
[`calculate_monthly_cost()`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L744) initially computed storage and cache overages only for `github` and `gitlab`, returning `$0.00` for `circleci`, `bitbucket`, `aws`, `gcp`, and `azure` and omitting `egress_gb` and `dlc_enabled` (which were only implemented in `qa_verifier.py`).

### Mandatory Engine Rule
- [`calculate_monthly_cost()`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L744) is the single source of truth and must compute `storage_overage_usd`, `cache_overage_usd`, `egress_overage_usd`, `dlc_overage_usd`, and `hidden_tax_usd` across all 7 providers directly from the catalog SKUs.

---

## 14. Expose 3-Tier Savings Deltas, Workflow-Level DORA Wait Inference, and Outlier Guards (QA Sessions #9 & #10)

### Mandatory Engine Rules
1. **3-Tier Savings Deltas:** [`calculate_savings()`](file:///Users/araza/personal/cicd-pricing-engine/engine/pricing.py#L1120) must output `saved_gross_consumed_usd`, `saved_discount_allowance_usd`, and `saved_net_billable_usd`, accept `current_workflows` / `selected_workflows`, and detect `crossed_allowance_boundary` even when `selected` drops to `0` minutes.
2. **Public Repo Larger Runner Billing Flag (`public_larger_runners_free`):** Standard GitHub-hosted runners (`linux`, `windows`, `macos` with `uses_allowance=True`) are always free on public repositories, whereas Larger/GPU/3rd-party runners (`uses_allowance=False`) are billed on un-sponsored public repos when `public_larger_runners_free=False`.
3. **Unpriced Self-Hosted Runner Warning:** Whenever `self_hosted` minutes $> 0$ and `self_hosted_usd_per_month` is not provided, emit `unpriced_self_hosted_warning` and `self_hosted_cloud_equivalent_usd` so `$0.00` software fees are never mistaken for zero infrastructure cost.
4. **Hard Non-Negativity & Reconciliation Invariants:** Clamp all negative inputs (`duration_s`, `count`, `shards`, `seats`, `storage_gb`, `cache_gb`, `egress_gb`, `skipped_runs`) at `0`, cap `skipped_runs <= monthly_runs`, and guarantee `round(gross_consumed_usd - discount_allowance_usd, 2) == round(net_billable_usd, 2)` on every receipt.

---

## 15. Nov 16, 2026 — 10-Repository (3 Internal + 7 Fresh OSS) × 3-Mode Pre-Lock Validation Findings (`minici-cost-action` v0.2.0)

Before locking Feature 1 (`minici-cost-action` / `minici` v0.2.0), we executed [`scripts/validate_minici_cost_10_repos.sh`](file:///Users/araza/personal/minici-cost-action/scripts/validate_minici_cost_10_repos.sh) and [`tools/verify_10_repos_all_modes.py`](file:///Users/araza/personal/minici-cost-action/tools/verify_10_repos_all_modes.py) across **10 primary repositories** (`MaximVote/maxim`, `mgjartproject/FineArts-BE`, `Eazy-Ops/Backend`, `astral-sh/uv`, `denoland/deno`, `ggml-org/llama.cpp`, `immich-app/immich`, `langgenius/dify`, `open-webui/open-webui`, `mendableai/firecrawl`) plus **7 additional internal repositories** (`MaximVote/tf`, `mgjartproject/gfa-data-pipelines`, `mgjartproject/frontend`, `mgjartproject/FineArt_AI_Pipeline`, `Eazy-Ops/frontend`, `Eazy-Ops/multi-cloud-finops-mcp-server`, `Eazy-Ops/Automation`).

### 15.1 Static Rust Catalog vs. JSON Pricing Drift (Resolved)
- **What Happened:** `pricing/github-actions.json` carried GitHub's January 1, 2026 post-cut rates (`$0.006/min` 2-vCPU Linux, `$0.005/min` ARM64, `$0.010/min` Windows, `$0.062/min` macOS), whereas [`crates/minici/src/catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs) initially retained 2025 rates (`$0.008`, `$0.016`, `$0.080`).
- **Resolution:** Added [`tools/sync_rust_sku_catalog.py`](file:///Users/araza/personal/minici-cost-action/tools/sync_rust_sku_catalog.py) (`--check` and `--apply-overrides`) into Gate 7 of [`scripts/qa_closed_loop.sh`](file:///Users/araza/personal/minici-cost-action/scripts/qa_closed_loop.sh) and `minici self-update`, enforcing **24/24 canonical SKU parity (`0.00%` drift)**.

### 15.2 `ggml-org/llama.cpp`: Self-Hosted macOS & Windows Labels (`self-hosted-macOS-ARM64`, `self-hosted-Windows-X64-Intel`)
- **What We Found:** `ggml-org/llama.cpp` defines both Linux self-hosted runners (`self-hosted-Linux-NVIDIA`, `self-hosted-Linux-Intel-OpenVINO`) and macOS/Windows self-hosted runners (`self-hosted-macOS-ARM64`, `self-hosted-llama-server-macOS-ARM64`, `self-hosted-Windows-X64-Intel`). Because `lookup_sku()` in [`crates/minici/src/main.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/main.rs) checked `lower.contains("macos")` and `lower.contains("windows")` *before* `lower.contains("self-hosted")`, `self-hosted-macOS-ARM64` resolved to GitHub SaaS macOS (`$0.0620/min`) instead of self-hosted compute (`$0.0024/min`).
- **Rule / Action Item:** Check `lower.contains("self-hosted")` *before* generic OS SaaS branches in `lookup_sku()`, while preserving the detected OS (`macOS` / `Windows` / `Linux`) for metadata accuracy.

### 15.3 `astral-sh/uv` & `langgenius/dify`: Third-Party Runner Trailing vCPU Suffixes (`depot-ubuntu-24.04-4/8/16`, `namespace-profile-macos-15`)
- **What We Found:**
  1. `astral-sh/uv` and `langgenius/dify` use Depot runners where the vCPU count is a trailing numeric suffix (`depot-ubuntu-24.04-4`, `depot-ubuntu-24.04-8`, `depot-ubuntu-24.04-16`, `depot-ubuntu-latest-4`) rather than `-4vcpu` or `-4core`. Matching `depot-` to a flat 2-vCPU rate (`$0.0040/min`) under-prices 4/8/16-vCPU Depot runners (`$0.0080/min`, `$0.0160/min`, `$0.0320/min`).
  2. `astral-sh/uv` uses `namespace-profile-macos-15` and `namespace-profile-windows-2022-x86-64-16x32`, which matched `namespace-` as 2-vCPU Linux (`$0.0040/min`) instead of macOS/Windows Namespace profiles.

### 15.4 `astral-sh/uv`, `denoland/deno`, `ggml-org/llama.cpp`: Custom Organization Runner Scale Sets (`github-ubuntu-24.04-aarch64-4`, `ubuntu-24.04-xl`, `ah-ubuntu_24_04-c8g_8x`, `hf-jobs-t4-medium:cuda13`)
- **What We Found:**
  - `astral-sh/uv` names its GitHub Larger Runner groups `github-ubuntu-24.04-aarch64-2`, `github-ubuntu-24.04-aarch64-4`, `github-ubuntu-24.04-x86_64-8`, and `github-windows-11-aarch64-8` (using `aarch64` instead of `-arm`, and trailing `-2`/`-4`/`-8` instead of `-4core`/`-8core`).
  - `denoland/deno` uses `ubuntu-24.04-xl`, and `ggml-org/llama.cpp` uses AWS Graviton (`ah-ubuntu_24_04-c8g_8x`) and HuggingFace GPU/CPU (`hf-jobs-t4-medium:cuda13`) runner scale sets. Without trailing-core heuristics or `.minici.toml` `[sku_overrides]`, these fall back to default 2-vCPU Linux (`$0.0060/min`).

### 15.5 `Eazy-Ops/Automation` & `mgjartproject/FineArt_AI_Pipeline`: Nested Monorepo Workflows & Zero-Workflow Repositories
- **What We Found:**
  - In `Eazy-Ops/Automation`, 5 GitHub Actions workflows (`api.yml`, `cronjob-evening.yml`, `cronjob-morning.yml`, `frontEnd.yml`, `qa-cronjob-flow.yml`) reside in `cron/.github/workflows/` rather than root `.github/workflows/`.
  - In `mgjartproject/FineArt_AI_Pipeline`, `mgjartproject/terraform-infra`, and `Eazy-Ops/control-panel`, there are `0` CI workflow files. `minici` handles zero-workflow repositories gracefully (`✓ minici: Skipped (0 CI workflow files found)`, exit code `0` in `~0.1 ms`).
  - In `Eazy-Ops/multi-cloud-finops-mcp-server`, gross compute is `$10.39/mo` (`1,732 mins/mo`, `86.6%` of the `2,000 mins/mo` free quota), and `minici` accurately reports **`$0.00/mo` net billed** (`free (within included quota)`).

### 15.6 Non-Interactive Subprocess Invocation of `--hook pre-tool-use`
- **What We Found:** Because `minici breakdown --hook pre-tool-use` reads Claude Code's `PreToolUse` JSON payload from `stdin`, invoking it from a test runner with an open inherited `stdin` blocks waiting for EOF. Automated scripts must always pass `input='{"tool_name":"Bash","tool_input":{"command":"git push"}}\n'` (or `stdin=subprocess.DEVNULL`).


