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
