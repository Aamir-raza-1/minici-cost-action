# Customer Impact & Multi-Repository Analysis: 5 Team-Stage & Pipeline-Shape Cost Models

**Repository:** `/Users/araza/personal/minici-cost-action` (`v0.3.0` • commit `677a722`)  
**Engine:** `minici` Zero-Dependency Rust Static FinOps Engine (`< 3.5 ms` execution per repository)  
**Scope:** 13 Representative Production & World-Scale Repositories across Startup, Scaleup, and Enterprise Team Stages (`single-service`, `monorepo`, and `multi-service-microservices` pipeline shapes), validated against the full 27-repository benchmark.

---

## 1. Executive Summary

Engineering organizations face fundamentally different CI/CD cost traps as they scale from a 4-developer pre-seed team to a 250+ developer enterprise, and as their repository topology evolves from a single service into a multi-package monorepo or distributed microservices fleet.

`minici` dynamically evaluates five first-party organizational and architectural cost models directly from repository manifests, workspace configs, and workflow AST signals—**with zero hardcoded repository names or manual configuration required**:

1. **`startup_stage_cost` (Pre-Seed to Series A • 1–15 Devs):** Tracks 2,000-minute free-tier burn rate, the 80% quota upgrade trigger, premature self-hosting traps below the 25,000 mins/mo ($500/mo) threshold, multi-OS (`linux` + `windows` + `macos`) PR matrix traps, and dedicated Mac mini hardware payback months (`$700` cap-ex + `$5/mo` power replacing 90% of hosted macOS spend).
2. **`scaleup_stage_cost` (Growth Stage • 25–100 Devs):** Evaluates the 25,000 mins/mo ($500/mo) self-hosted break-even threshold, the `+$17/seat/mo` Team-to-Enterprise seat licensing jump (`$4/seat` to `$21/seat`), quarterly required-check audit savings (`20%–35%` reduction when `> 6` required checks accumulate), and Platform Engineer ROI coverage against a `$16,500/mo` fully loaded benchmark.
3. **`enterprise_stage_cost` (Enterprise • 250+ Devs):** Audits 6 automated compliance and security governance controls (`oidc-federated-auth`, `deployment-environment-approvals`, `sast-codeql-semgrep`, `sca-container-vulnerability-scan`, `secret-scanning`, `sbom-slsa-provenance`), multi-region cloud residency multipliers (`1.0x` single-region, `1.8x` dual-region, `2.75x` tri-region), 250-seat Enterprise licensing (`$5,250/mo`), and 1-year rolling SIEM audit log retention costs (`SOC 2` / `ISO 27001`).
4. **`monorepo_ci_cost` (Workspace & Affected-Graph Economics):** Dynamically detects workspace packages (`pnpm-workspace.yaml`, `turbo.json`, `nx.json`, `Cargo.toml`, `pyproject.toml`, and path-filter prefixes), identifies active graph tools vs. coarse path-filters-only, recommends stack-specific affected-only runners (`nx-or-turborepo`, `cargo-nextest-or-bazel`, `pants-or-nx`), and quantifies monthly savings (`35%` incremental over coarse path filters; `72.5%` midpoint on unconstrained monorepos).
5. **`microservices_ci_cost` (Per-Job Setup Overhead, Template Governance & Shared Cache):** Quantifies per-job container/cache/init setup tax (`75s` cached vs. `120s` uncached, capped at `45%` of raw runtime), shared base-image and pre-warmed runner savings (`60%` setup reduction), reusable workflow call volume, unpinned `@main`/`@master`/`@head`/`@latest` template blast-radius exposure, cross-repository dispatch triggers, and aggregate cache footprint against the `10 GB` repository eviction ceiling.

---

## 2. Dynamic Classification Architecture (Zero Hardcoding)

### 2.1 Dynamic `inferred_team_stage` Classification

`minici` infers active developer headcount (`implied_active_devs`) and organizational stage from workload volume (`monthly_billed_mins`) and gross monthly compute spend (`gross_monthly_compute_cost_usd`), using `max(ceil(runs_per_day / 5.0), ceil(monthly_billed_mins / 1600.0))`:

| Inferred Team Stage | Dynamic Threshold Rule | Primary FinOps Focus |
| :--- | :--- | :--- |
| **`startup-pre-seed (1-5 devs)`** | `monthly_billed_mins <= 3,000` **and** `gross_monthly_compute_cost_usd <= $50.00` | Stay within 2,000-min free tier (`$0.00/mo` net billed); avoid premature self-hosting ops burden |
| **`startup-seed-series-a (6-15 devs)`** | `monthly_billed_mins <= 25,000` **and** `gross_monthly_compute_cost_usd <= $500.00` | Manage post-free-tier transition, add path filters & `cancel-in-progress`, avoid premature multi-OS PR matrices |
| **`scaleup (25-100 devs)`** | `monthly_billed_mins <= 150,000` **and** `gross_monthly_compute_cost_usd <= $12,000.00` | Cross self-hosted/custom runner break-even, prune required checks quarterly, evaluate platform engineering ROI |
| **`enterprise (250+ devs)`** | `monthly_billed_mins > 150,000` **or** `gross_monthly_compute_cost_usd > $12,000.00` | Automate compliance gates, manage multi-region data residency, budget 1-yr SIEM audit log retention |

### 2.2 Dynamic `detected_pipeline_shape` Classification

`minici` inspects repository manifests and workflow YAML structure to classify every codebase into one of three architectural shapes:

| Detected Pipeline Shape | AST & Manifest Detection Signal | Primary Architectural Optimization |
| :--- | :--- | :--- |
| **`single-service`** | `< 3` workspace units, no monorepo graph tool, and `< 4` multi-service workflows | Job-level dependency caching, `cancel-in-progress`, and right-sized runner SKUs |
| **`monorepo`** | Explicit monorepo graph tool (`turbo`, `nx`, `bazel`, `pants`, `lerna`) **or** `>= 3` distinct workspace units discovered across manifests/path filters | Affected-only target execution (`nx affected`, `turbo run --filter`, `cargo nextest`) + nightly full-suite safety net |
| **`multi-service-microservices`** | `>= 4` workflows combining reusable workflow calls (`workflow_call` / `/.github/workflows/`), multi-service path filters (`>= 2`), or multiple service units | Pre-baked shared container base images, SHA-pinned reusable workflow templates, and shared remote cache backends |

---

## 3. Master Multi-Repository Benchmark (13 Diverse Repositories)

The table below summarizes the deterministic outputs produced by `/Users/araza/.local/bin/minici breakdown --path <repo> --format json` across all 13 target repositories spanning Startup, Scaleup, and Enterprise cohorts:

| Repository | Cohort | Workflows / Jobs (Expanded) | Monthly Billed Mins | Gross Compute (`$/mo`) | Net Billed (`$/mo`) | Dev Wait Gross (`$/mo`) | `inferred_team_stage` | `Implied Devs` | `detected_pipeline_shape` | Workspace Units / Tool |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :--- | :---: | :--- | :--- |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | Startup | 1 / 1 (1) | 866.0m | $5.20 | **$0.00** | $866.00 | `startup-pre-seed (1-5 devs)` | 4 | `single-service` | 0 (`none`) |
| **`02_mgjartproject__FineArts-BE`** | Startup | 2 / 6 (6) | 7,794.0m | $46.76 | $46.76 | $2,979.04 | `startup-seed-series-a (6-15 devs)` | 5 | `single-service` | 0 (`none`) |
| **`03_Eazy-Ops__Backend`** | Startup | 5 / 5 (5) | 12,124.0m | $72.74 | $72.74 | $8,374.22 | `startup-seed-series-a (6-15 devs)` | 8 | `single-service` | 0 (`none`) |
| **`check_MaximVote__tf`** | Startup | 6 / 8 (8) | 11,691.0m | $70.15 | $70.15 | $1,688.70 | `startup-seed-series-a (6-15 devs)` | 8 | `monorepo` | 3 (`path-filters-only`) |
| **`01_MaximVote__maxim`** | Scaleup | 41 / 45 (57) | 84,435.0m | $1,009.41 | $1,009.41 | $4,269.38 | `scaleup (25-100 devs)` | 53 | `monorepo` | 10 (`turborepo`) |
| **`07_immich-app__immich`** | Scaleup | 27 / 76 (85) | 116,477.0m | $819.67 | $819.67 | $4,879.91 | `scaleup (25-100 devs)` | 73 | `monorepo` | 17 (`path-filters-only`) |
| **`08_langgenius__dify`** | Scaleup | 41 / 84 (117) | 170,385.5m | $1,153.51 | $1,153.51 | $4,853.93 | `enterprise (250+ devs)` | 107 | `monorepo` | 45 (`path-filters-only`) |
| **`10_mendableai__firecrawl`** | Scaleup | 36 / 71 (86) | 271,426.1m | $1,978.81 | $1,978.81 | $39,615.17 | `enterprise (250+ devs)` | 170 | `monorepo` | 16 (`path-filters-only`) |
| **`fresh_05_vercel__next.js`** | Enterprise | 5 / 38 (45) | 57,156.0m | $919.69 | $919.69 | $2,511.40 | `scaleup (25-100 devs)` | 36 | `single-service` | 0 (`none`) |
| **`06_ggml-org__llama.cpp`** | Enterprise | 62 / 121 (152) | 199,115.1m | $1,930.75 | $1,930.75 | $6,378.09 | `enterprise (250+ devs)` | 125 | `multi-service-microservices` | 1 (`none`) |
| **`04_astral-sh__uv`** | Enterprise | 47 / 223 (243) | 323,126.3m | $3,464.00 | $3,464.00 | $6,122.62 | `enterprise (250+ devs)` | 202 | `monorepo` | 5 (`path-filters-only`) |
| **`fresh_10_pydantic__pydantic`** | Enterprise | 12 / 66 (175) | 402,993.1m | $4,759.88 | $4,759.88 | $5,958.08 | `enterprise (250+ devs)` | 252 | `monorepo` | 8 (`none`) |
| **`05_denoland__deno`** | Enterprise | 11 / 59 (162) | 805,380.0m | $20,206.81 | $20,206.81 | $13,015.98 | `enterprise (250+ devs)` | 504 | `multi-service-microservices` | 1 (`none`) |

---

## 4. Deep-Dive Findings by Cost Model

### 4.1 Model 1: `startup_stage_cost` — Free-Tier Quota, Multi-OS Matrix Traps & Mac mini Payback

| Repository | Inferred Stage | Free-Tier Burn (`%`) | 80% Upgrade Trigger | Multi-OS PR Matrix Trap | Shift Non-Linux PR Matrix to Nightly Savings (`$/mo`) | Mac mini ($700) Hardware Payback |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | `startup-pre-seed (1-5 devs)` | **43.3%** (`866 / 2,000m`) | `false` | `false` | $0.00/mo | — (`0.0 mo`) |
| **`02_mgjartproject__FineArts-BE`** | `startup-seed-series-a (6-15 devs)` | **389.7%** (`7,794m`) | `true` | `false` | $0.00/mo | — (`0.0 mo`) |
| **`03_Eazy-Ops__Backend`** | `startup-seed-series-a (6-15 devs)` | **606.2%** (`12,124m`) | `true` | `false` | $0.00/mo | — (`0.0 mo`) |
| **`check_MaximVote__tf`** | `startup-seed-series-a (6-15 devs)` | **584.6%** (`11,691m`) | `true` | `false` | $0.00/mo | — (`0.0 mo`) |
| **`08_langgenius__dify`** | `enterprise (250+ devs)` | **8,519.3%** (`170,386m`) | `true` | `false` | $0.00/mo | **5.00 months** |
| **`06_ggml-org__llama.cpp`** | `enterprise (250+ devs)` | **9,955.8%** (`199,115m`) | `true` | **`true`** | **$867.74/mo** | **1.51 months** |
| **`04_astral-sh__uv`** | `enterprise (250+ devs)` | **16,156.3%** (`323,126m`) | `true` | `false` | $0.00/mo | **0.83 months (~25 days)** |
| **`fresh_10_pydantic__pydantic`** | `enterprise (250+ devs)` | **20,149.7%** (`402,993m`) | `true` | **`true`** | **$2,640.46/mo** | **0.32 months (~10 days)** |
| **`05_denoland__deno`** | `enterprise (250+ devs)` | **40,269.0%** (`805,380m`) | `true` | **`true`** | **$16,975.97/mo** | **0.05 months (~1.5 days)** |

#### Customer Impact Highlights (`startup_stage_cost`)
* **True Free-Tier Enforcement & Anti-Premature Self-Hosting (`check_Eazy-Ops__multi-cloud-finops-mcp-server`):** At `866 mins/mo` (`43.3%` of the 2,000-minute allowance), `minici` bills **$0.00/mo net** (`$5.20/mo` gross covered by free tier), keeps `upgrade_trigger_80pct_reached = false`, and explicitly recommends in CLI: `Stay on hosted runners (866 / 5,000 mins/mo self-hosted break-even threshold)`.
* **Seed/Series-A Quota Exhaustion (`FineArts-BE`, `Eazy-Ops/Backend`, `MaximVote/tf`):** All three 5–8 developer repositories have crossed the `80%` free-tier trigger (`389.7%` to `606.2%` quota burn, `$46.76/mo` to `$72.74/mo`), yet remain well below the `25,000 mins/mo` (`$500/mo`) self-hosted infrastructure break-even where DevOps maintenance overhead outweighs compute savings.
* **Multi-OS PR Matrix Trap (`deno`, `pydantic`, `llama.cpp`):** Running full 3-OS (`linux` + `windows` + `macos`) matrices on every PR push instead of gating PRs on Linux and shifting full cross-OS verification to a 30-run/month nightly schedule (`shift_ratio = (433 - 30) / 433 = 93.07%`) costs:
  * **`05_denoland__deno`:** **$16,975.97/mo** in avoidable non-Linux PR matrix spend (out of `$20,206.81/mo` total compute).
  * **`fresh_10_pydantic__pydantic`:** **$2,640.46/mo** in avoidable non-Linux PR matrix spend (out of `$4,759.88/mo` total compute).
  * **`06_ggml-org__llama.cpp`:** **$867.74/mo** in avoidable non-Linux PR matrix spend (out of `$1,930.75/mo` total compute).
* **Dedicated Mac mini Hardware Payback (`$700` Cap-Ex + `$5/mo` Power):** Because hosted macOS runners bill at `$0.0620/min` (`10.3x` Linux rates), replacing 90% of hosted macOS minutes with self-hosted Apple Silicon Mac mini hardware pays back in:
  * **`05_denoland__deno`:** **0.05 months (~1.5 days)**
  * **`fresh_10_pydantic__pydantic`:** **0.32 months (~9.6 days)**
  * **`04_astral-sh__uv`:** **0.83 months (~25 days)**
  * **`06_ggml-org__llama.cpp`:** **1.51 months (~45 days)**
  * **`08_langgenius__dify`:** **5.00 months**

---

### 4.2 Model 2: `scaleup_stage_cost` — Self-Hosted Break-Even, Quarterly Check Audits & Platform Engineer ROI

| Repository | Self-Hosted Break-Even (`>=25k mins` or `>=$500`) | Team $\to$ Enterprise Seat Jump (`+$17/dev/mo`) | Quarterly Required-Check Audit Savings (`$/mo`) | Recoverable CI + Wait Waste (`$/mo`) | Platform Engineer (`$16.5k/mo`) ROI Coverage |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | `false` | $425.00/mo (25 min seats) | $0.00/mo (1 check) | $255.41/mo | **1.55%** |
| **`02_mgjartproject__FineArts-BE`** | `false` | $425.00/mo (25 min seats) | $0.00/mo (6 checks) | $539.08/mo | **3.27%** |
| **`03_Eazy-Ops__Backend`** | `false` | $425.00/mo (25 min seats) | $0.00/mo (5 checks) | $1,201.66/mo | **7.28%** |
| **`check_MaximVote__tf`** | `false` | $425.00/mo (25 min seats) | **$14.03/mo** (8 checks) | $587.73/mo | **3.56%** |
| **`fresh_05_vercel__next.js`** | **`true`** | $612.00/mo (36 devs) | **$321.89/mo** (45 checks) | $879.99/mo | **5.33%** |
| **`01_MaximVote__maxim`** | **`true`** | $901.00/mo (53 devs) | **$353.29/mo** (57 checks) | $1,045.27/mo | **6.33%** |
| **`07_immich-app__immich`** | **`true`** | $1,241.00/mo (73 devs) | **$286.88/mo** (85 checks) | $1,342.51/mo | **8.14%** |
| **`08_langgenius__dify`** | **`true`** | $1,819.00/mo (107 devs) | **$230.70/mo** (117 checks) | $1,149.01/mo | **6.96%** |
| **`06_ggml-org__llama.cpp`** | **`true`** | $2,125.00/mo (125 devs) | **$386.15/mo** (152 checks) | $1,554.84/mo | **9.42%** |
| **`04_astral-sh__uv`** | **`true`** | $3,434.00/mo (202 devs) | **$1,212.40/mo** (243 checks) | $1,997.64/mo | **12.11%** |
| **`fresh_10_pydantic__pydantic`** | **`true`** | $4,284.00/mo (252 devs) | **$1,665.96/mo** (175 checks) | $3,523.20/mo | **21.35%** |
| **`10_mendableai__firecrawl`** | **`true`** | $2,890.00/mo (170 devs) | **$395.76/mo** (86 checks) | **$5,982.36/mo** | **36.26%** |
| **`05_denoland__deno`** | **`true`** | $8,568.00/mo (504 devs) | **$7,072.38/mo** (162 checks) | **$11,771.12/mo** | **71.34%** |

#### Customer Impact Highlights (`scaleup_stage_cost`)
* **Self-Hosted & High-Speed Runner Break-Even Validation:** Every scaleup and enterprise repository (`>= 25,000 mins/mo` or `>= $500/mo`) triggers `self_hosted_breakeven_reached = true`. Notably, `01_MaximVote__maxim` (`84,435 mins/mo`, `$1,009.41/mo`) and `08_langgenius__dify` (`170,386 mins/mo`, `$1,153.51/mo`) have already migrated `44/57` and `79/117` jobs respectively onto high-speed custom runners—which `minici` detects and acknowledges without emitting redundant runner migration noise.
* **Quarterly Required-Check Audit Savings:** As repositories scale past 6 required checks, legacy blocking checks accumulate on every PR push. Auditing and demoting low-signal checks saves `20%` (when path filters are already dominant, e.g., `dify` saving **$230.70/mo** and `firecrawl` saving **$395.76/mo**) to `35%` (when unfiltered workflows dominate, e.g., `maxim` saving **$353.29/mo**, `uv` saving **$1,212.40/mo**, `pydantic` saving **$1,665.96/mo**, and `deno` saving **$7,072.38/mo**).
* **Platform Engineer ROI Coverage (`$16,500/mo` Benchmark):** Combining unrealized cache savings, superseded run cancellation, path filtering, flake reduction to 1%, and a 30% reduction in DORA context-switch wait loss yields **$5,982.36/mo** (`36.26%` of a full-time Platform Engineer's cost) on a single repository like `10_mendableai__firecrawl` (`84.5m` critical path) and **$11,771.12/mo** (`71.34%` FTE coverage) on `05_denoland__deno`. Across an organization managing 2–3 such repositories, a dedicated CI/CD Platform Engineer is `> 100%` self-funding.

---

### 4.3 Model 3: `enterprise_stage_cost` — Governance Controls, Cloud Regions & 1-Year SIEM Audit Log Retention

| Repository | Compliance Controls Count | Detected Compliance Controls | Detected Cloud Regions | Residency Multiplier | 250-Seat Enterprise Licence (`$/mo`) | 1-Yr SIEM Audit Log Retention (`$/mo`) |
| :--- | :---: | :--- | :--- | :---: | :---: | :---: |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | 0 | `[]` | `[]` | `1.0x` | $5,250.00 | $1.94/mo |
| **`fresh_05_vercel__next.js`** | 0 | `[]` | `[]` | `1.0x` | $5,250.00 | $59.08/mo |
| **`08_langgenius__dify`** | 1 | `oidc-federated-auth` | `[]` | `1.0x` | $5,250.00 | $230.42/mo |
| **`06_ggml-org__llama.cpp`** | 1 | `oidc-federated-auth` | `[]` | `1.0x` | $5,250.00 | **$1,796.15/mo** |
| **`03_Eazy-Ops__Backend`** | 2 | `sast-codeql-semgrep`, `sca-container-vulnerability-scan` | `us-central1` | `1.0x` | $5,250.00 | $6.59/mo |
| **`check_MaximVote__tf`** | 2 | `oidc-federated-auth`, `deployment-environment-approvals` | `us-west-1` | `1.0x` | $5,250.00 | $23.11/mo |
| **`01_MaximVote__maxim`** | 2 | `oidc-federated-auth`, `deployment-environment-approvals` | `us-east-1`, `us-west-1` | **`1.8x`** | $5,250.00 | **$134.72/mo** |
| **`10_mendableai__firecrawl`** | 2 | `deployment-environment-approvals`, `oidc-federated-auth` | `[]` | `1.0x` | $5,250.00 | $248.40/mo |
| **`05_denoland__deno`** | 2 | `oidc-federated-auth`, `deployment-environment-approvals` | `[]` | `1.0x` | $5,250.00 | $467.91/mo |
| **`fresh_10_pydantic__pydantic`** | 2 | `oidc-federated-auth`, `deployment-environment-approvals` | `[]` | `1.0x` | $5,250.00 | **$505.49/mo** |
| **`02_mgjartproject__FineArts-BE`** | 3 | `sca-container-vulnerability-scan`, `secret-scanning`, `oidc-federated-auth` | `us-central1` | `1.0x` | $5,250.00 | $11.83/mo |
| **`07_immich-app__immich`** | 3 | `deployment-environment-approvals`, `oidc-federated-auth`, `sast-codeql-semgrep` | `[]` | `1.0x` | $5,250.00 | **$1,004.40/mo** |
| **`04_astral-sh__uv`** | 3 | `oidc-federated-auth`, `deployment-environment-approvals`, `sbom-slsa-provenance` | `us-east-1` | `1.0x` | $5,250.00 | **$701.89/mo** |

*(Note: Across the broader 27-repository validation suite, `09-home-assistant-core` reaches **5 of 6** compliance controls simultaneously, and `04_astral-sh__uv` demonstrates cryptographic supply-chain attestation via `sbom-slsa-provenance`.)*

#### Customer Impact Highlights (`enterprise_stage_cost`)
* **Zero-Config Security & Compliance Posture Auditing:** Without requiring external scanners, `minici` parses workflow ASTs to verify which repositories enforce keyless OIDC authentication (`id-token: write`, `google-github-actions/auth`, `configure-aws-credentials`), environment approval gates (`environment:`), SAST (`codeql-action`, `semgrep`), SCA (`trivy`, `dependency-review`, `cargo-audit`), secret scanning (`gitleaks`, `trufflehog`), and SLSA/SBOM provenance (`attest-build-provenance`, `cosign`).
* **Multi-Region Data Residency Multiplier (`01_MaximVote__maxim`):** Because `01_MaximVote__maxim` deploys across both `us-east-1` and `us-west-1`, `minici` automatically applies a **`1.8x` regional residency multiplier** to account for dual-region log replication and compliance boundary isolation (`$134.72/mo` 1-yr SIEM retention). Single-region deployments (`FineArts-BE` and `Eazy-Ops/Backend` in `us-central1`, `MaximVote/tf` in `us-west-1`, `uv` in `us-east-1`) retain a `1.0x` multiplier.
* **1-Year Rolling SIEM Audit Log Retention (`SOC 2` / `ISO 27001`):** Native vendor log retention expires after 90 days on GitHub Actions. Retaining 12 months of indexed CI/CD audit logs (`monthly_log_volume_gb * 12 * $0.45/GB * regional_residency_multiplier`) creates a hidden enterprise compliance obligation of **$1,796.15/mo** on `06_ggml-org__llama.cpp`, **$1,004.40/mo** on `07_immich-app__immich`, **$701.89/mo** on `04_astral-sh__uv`, and **$505.49/mo** on `fresh_10_pydantic__pydantic`.

---

### 4.4 Model 4: `monorepo_ci_cost` — Workspace Discovery & Affected-Only Dependency Graph Savings

| Repository | `detected_pipeline_shape` | Workspace Units | `detected_monorepo_tool` | `recommended_affected_tool` | Nightly Full-Suite Safety Net | Affected-Only Savings (`$/mo`) |
| :--- | :--- | :---: | :--- | :--- | :---: | :---: |
| **`01_MaximVote__maxim`** | **`monorepo`** | **10** | **`turborepo`** | `nx-or-turborepo` | `true` | **$0.00/mo** *(Already optimized via Turborepo)* |
| **`check_MaximVote__tf`** | **`monorepo`** | **3** | `path-filters-only` | `cargo-nextest-or-bazel` | `false` | **$24.55/mo** (`35%` incremental) |
| **`07_immich-app__immich`** | **`monorepo`** | **17** | `path-filters-only` | `nx-or-turborepo` | `true` | **$286.88/mo** (`35%` incremental) |
| **`08_langgenius__dify`** | **`monorepo`** | **45** | `path-filters-only` | `nx-or-turborepo` | `true` | **$403.73/mo** (`35%` incremental) |
| **`10_mendableai__firecrawl`** | **`monorepo`** | **16** | `path-filters-only` | `nx-or-turborepo` | `true` | **$692.58/mo** (`35%` incremental) |
| **`04_astral-sh__uv`** | **`monorepo`** | **5** | `path-filters-only` | `cargo-nextest-or-bazel` | `true` | **$1,212.40/mo** (`35%` incremental) |
| **`fresh_10_pydantic__pydantic`** | **`monorepo`** | **8** | `none` | `nx-or-turborepo` | `true` | **$3,450.91/mo** (`72.5%` unconstrained) |

*(Note: `check_Eazy-Ops__multi-cloud-finops-mcp-server`, `02_mgjartproject__FineArts-BE`, `03_Eazy-Ops__Backend`, and `fresh_05_vercel__next.js` are dynamically classified as `single-service` with `$0.00/mo` monorepo affected-only savings; `05_denoland__deno` and `06_ggml-org__llama.cpp` are classified as `multi-service-microservices`.)*

#### Customer Impact Highlights (`monorepo_ci_cost`)
* **Strict Applicability When Graph Tools Are Already Active (`01_MaximVote__maxim`):** `minici` detects `turborepo` across 10 workspace units in `01_MaximVote__maxim` (`has_affected_only_graph_tool = true`). Instead of fabricating phantom savings, `minici` reports `$0.00/mo` in `monthly_affected_only_savings_usd` and displays `Monorepo (10 units • turborepo)` cleanly in CLI output.
* **Coarse Path Filters vs. True Dependency Graph (`immich`, `dify`, `firecrawl`, `uv`, `MaximVote/tf`):** Many growing monorepos rely on YAML `paths:` filters (`detected_monorepo_tool = "path-filters-only"`). While better than running everything, coarse directory filters still trigger full workspace rebuilds whenever shared libraries change. Upgrading from coarse path filters to fine-grained target graph execution (`nx-or-turborepo` or `cargo-nextest-or-bazel`) yields an additional **`35%` compute reduction**:
  * **`04_astral-sh__uv` (5 units):** Saves **$1,212.40/mo**
  * **`10_mendableai__firecrawl` (16 units):** Saves **$692.58/mo**
  * **`08_langgenius__dify` (45 units):** Saves **$403.73/mo**
  * **`07_immich-app__immich` (17 units):** Saves **$286.88/mo**
  * **`check_MaximVote__tf` (3 units):** Saves **$24.55/mo**
* **Unconstrained Monorepo Blast Radius (`fresh_10_pydantic__pydantic`):** With 8 workspace units and `detected_monorepo_tool = "none"` (0 path-filtered workflows), every PR push triggers the full 175-job matrix (`$4,759.88/mo`). Introducing affected-only graph execution (`60%–85%` reduction, `72.5%` midpoint) unlocks **$3,450.91/mo** (`$41,410.92/yr`) in direct compute savings while retaining its existing scheduled nightly suite (`has_nightly_full_suite_safety_net = true`).

---

### 4.5 Model 5: `microservices_ci_cost` — Per-Job Setup Overhead, Unpinned `@main/@master` Blast Radius & Cache Eviction

| Repository | Setup Overhead (`sec/job`) | Monthly Setup Hours | Setup Share (`%`) | Setup Compute Tax (`$/mo`) | Shared Base-Image Savings (`$/mo`) | Reusable Workflow Calls | Unpinned `@main/@master` Refs | Cross-Repo Triggers | Est. Cache (`GB`) / `>10GB` Limit |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | 75s | 6.01 hrs | 45.00% | $2.34/mo | $1.40/mo | 0 | 0 | 0 | 0.5 GB (`false`) |
| **`02_mgjartproject__FineArts-BE`** | 75s | 43.84 hrs | 45.00% | $21.04/mo | $12.62/mo | 0 | 0 | 0 | 1.0 GB (`false`) |
| **`03_Eazy-Ops__Backend`** | **120s** | 72.17 hrs | 39.14% | $28.47/mo | $17.08/mo | 0 | **1** | 0 | 2.5 GB (`false`) |
| **`check_MaximVote__tf`** | **120s** | 75.99 hrs | 45.00% | $31.57/mo | $18.94/mo | 0 | 0 | 0 | 3.0 GB (`false`) |
| **`fresh_05_vercel__next.js`** | **120s** | 384.02 hrs | 45.00% | $413.86/mo | $248.32/mo | 0 | 0 | 0 | 2.5 GB (`false`) |
| **`01_MaximVote__maxim`** | 75s | 514.19 hrs | 37.56% | $379.13/mo | **$227.48/mo** | **31** | 0 | 0 | **20.5 GB (`true`)** |
| **`07_immich-app__immich`** | **120s** | 761.86 hrs | 45.00% | $368.85/mo | **$221.31/mo** | **10** | **2** | **1** | **13.5 GB (`true`)** |
| **`10_mendableai__firecrawl`** | 75s | 775.79 hrs | 17.72% | $350.65/mo | **$210.39/mo** | **6** | 0 | **2** | **18.0 GB (`true`)** |
| **`08_langgenius__dify`** | 75s | 1,055.44 hrs | 38.50% | $444.10/mo | **$266.46/mo** | **25** | 0 | **9** | **22.5 GB (`true`)** |
| **`06_ggml-org__llama.cpp`** | 75s | 1,293.32 hrs | 45.00% | $868.84/mo | **$521.30/mo** | **16** | 0 | **1** | **31.0 GB (`true`)** |
| **`05_denoland__deno`** | 75s | 1,461.38 hrs | 11.40% | **$2,303.58/mo** | **$1,382.15/mo** | **3** | 0 | 0 | 5.5 GB (`false`) |
| **`fresh_10_pydantic__pydantic`** | 75s | 1,578.65 hrs | 25.53% | **$1,215.20/mo** | **$729.12/mo** | **1** | 0 | **2** | 6.0 GB (`false`) |
| **`04_astral-sh__uv`** | 75s | **2,100.32 hrs** | 45.00% | **$1,558.80/mo** | **$935.28/mo** | **73** | 0 | **1** | **23.5 GB (`true`)** |

#### Customer Impact Highlights (`microservices_ci_cost`)
* **Per-Job Setup Tax & Pre-Warmed Base Image Savings:** Every expanded job pays a recurring setup tax (`75s` when majority-cached; `120s` when uncached jobs dominate, as in `Eazy-Ops/Backend`, `MaximVote/tf`, `immich`, and `next.js`) for container pulls, toolchain bootstrapping, and cache restoration. Across 243 expanded jobs in `04_astral-sh__uv`, setup overhead consumes **2,100.32 hours/month** (**$1,558.80/mo** compute tax). Adopting pre-baked shared container images and pre-warmed runner caches cuts setup overhead by `60%`, saving **$1,382.15/mo** on `deno`, **$935.28/mo** on `uv`, **$729.12/mo** on `pydantic`, and **$521.30/mo** on `llama.cpp`.
* **Unpinned `@main`/`@master` Template Blast-Radius Detection:** Referencing actions or reusable workflows via mutable branches (`@main`, `@master`, `@head`, `@latest`) exposes pipelines to supply-chain injection and simultaneous cross-service breakage. `minici` flags:
  * **`03_Eazy-Ops__Backend`:** **1 unpinned `@main/@master` ref** (surfaced directly in the CLI `Pipeline Hygiene` line).
  * **`07_immich-app__immich`:** **2 unpinned `@main/@master` refs** alongside 10 reusable workflow calls and 1 cross-repo trigger.
  * *(Across the full 27-repo benchmark, `09_open-webui__open-webui` and `check_Eazy-Ops__frontend` each have **1** unpinned ref, and `09-home-assistant-core` has **5** unpinned refs.)*
* **Reusable Workflow Sprawl & 10 GB Cache Eviction Thrashing:** High-velocity repositories with dozens of workflow units (`uv` with **73** reusable workflow calls and `23.5 GB` aggregate cache; `maxim` with **31** reusable calls and `20.5 GB` cache; `dify` with **25** reusable calls, **9** cross-repo triggers, and `22.5 GB` cache; `llama.cpp` with **16** reusable calls and `31.0 GB` cache) exceed GitHub Actions' native `10 GB` per-repository cache limit (`exceeds_10gb_repo_cache_limit = true`), explaining why LRU cache eviction degrades hit rates unless backed by external object-store caching.

---

## 5. Unified CLI Output Verification (`--format table`)

In compliance with the **Unified Product Output & Strict Applicability Rule**, `minici` integrates the actionable signals from all 5 models directly into the single unified `• Pipeline Hygiene` and `• Recommendation` bullets in CLI table mode—with zero bolted-on banners, zero hypothetical cost lines, and zero external branding:

### 5.1 Pre-Seed Startup (`check_Eazy-Ops__multi-cloud-finops-mcp-server`)
```text
 OVERALL TOTAL (433 runs/mo • free tier)     1 expanded job(s)                2.0m           —      $0.012       $5.20/mo

 • Free-Tier Quota  : 866 / 2000 mins (43.3% of monthly quota) • Net Billed: $0.00/mo • Rounding Overhead: 0.1m/run ($0.39/mo)
 • Dev Wait-Time    : 1.9m critical path ($2.00/run • $866.00/mo gross | $347.10/mo at 40% context-switch loss)
 • Pipeline Hygiene : Cache 1/1 jobs (75% est hit rate)
 • Recommendation   : Stay on hosted runners (866 / 5,000 mins/mo self-hosted break-even threshold)
```

### 5.2 Series-A Single-Service with Unpinned Ref (`03_Eazy-Ops__Backend`)
```text
 OVERALL TOTAL (433 runs/mo • paid tier)     5 expanded job(s)               28.0m           —      $0.168      $72.74/mo

 • Free-Tier Quota  : 12124 / 2000 mins (606.2% of monthly quota) • Net Billed: $72.74/mo • Rounding Overhead: 2.5m/run ($6.37/mo)
 • Dev Wait-Time    : 17.9m critical path ($19.34/run • $8374.22/mo gross | $3349.32/mo at 40% context-switch loss)
 • Pipeline Hygiene : Cache 1/5 jobs (caching 1 uncached saves ~$3.51/mo) • Cancel-in-progress missing on 5 wf (~$10.91/mo superseded waste) • Path filters missing on 5 wf (~$29.10/mo selective savings) • 1 unpinned @main/@master ref(s)
 • Recommendation   : Right-size heavy jobs to ARM64 / High-Speed Runners ($44.25/mo, saves $28.49/mo) or Cloud Spot ($43.25/mo, saves $29.49/mo)
```

### 5.3 Scaleup Monorepo Already Optimized with Turborepo & Custom Runners (`01_MaximVote__maxim`)
```text
 OVERALL TOTAL (433 runs/mo • paid tier)     57 expanded job(s)             195.0m           —      $2.331    $1009.41/mo

 • Free-Tier Quota  : 84435 / 2000 mins (4221.8% of monthly quota) • Net Billed: $1009.41/mo • Rounding Overhead: 5.3m/run ($27.44/mo)
 • Dev Wait-Time    : 9.1m critical path ($9.86/run • $4269.38/mo gross | $1707.42/mo at 40% context-switch loss) • Concurrency: 57 peak jobs (13 hosted / 44 custom)
 • Artifact Storage : 7 upload job(s) • 1d retention ($2.71/mo, 10.83 GB steady-state)
 • Pipeline Hygiene : Monorepo (10 units • turborepo) • Cache 42/57 jobs (caching 4 uncached saves ~$26.69/mo) • Cancel-in-progress missing on 20 wf (~$12.16/mo superseded waste) • Path filters missing on 13 wf (~$55.70/mo selective savings)
 • Recommendation   : Pipeline already optimized (44/57 jobs on custom runners; remaining 13 lightweight hosted job(s) are optimal as-is)
```

### 5.4 Scaleup Monorepo Needing Affected-Only Graph & Pinning (`07_immich-app__immich`)
```text
 OVERALL TOTAL (433 runs/mo • paid tier)     85 expanded job(s)             269.0m           —      $1.893     $819.67/mo

 • Free-Tier Quota  : 116477 / 2000 mins (5823.9% of monthly quota) • Net Billed: $819.67/mo • Rounding Overhead: 34.4m/run ($104.82/mo)
 • Dev Wait-Time    : 10.4m critical path ($11.27/run • $4879.91/mo gross | $1951.30/mo at 40% context-switch loss) • Concurrency: 85 peak jobs (85 hosted > 20 cap, +$510.00/mo tier upgrade)
 • Artifact Storage : 8 upload job(s) • 1d retention ($2.71/mo, 10.83 GB steady-state)
 • Pipeline Hygiene : Monorepo (17 units, nx-or-turborepo affected-only saves ~$286.88/mo) • Cache 13/85 jobs (caching 29 uncached saves ~$142.72/mo) • Cancel-in-progress missing on 10 wf (~$11.30/mo superseded waste) • Path filters missing on 17 wf (~$183.59/mo selective savings) • 2 unpinned @main/@master ref(s)
 • Recommendation   : Right-size heavy jobs to ARM64 / High-Speed Runners ($472.66/mo, saves $347.01/mo) or Cloud Spot ($198.52/mo, saves $621.15/mo)
```

### 5.5 World-Scale Multi-OS Multi-Service Pipeline (`05_denoland__deno`)
```text
 OVERALL TOTAL (433 runs/mo • paid tier)     162 expanded job(s)           1860.0m           —     $46.667   $20206.81/mo

 • Free-Tier Quota  : 805380 / 2000 mins (40269.0% of monthly quota) • Net Billed: $20206.81/mo • Rounding Overhead: 84.1m/run ($913.65/mo)
 • Dev Wait-Time    : 27.8m critical path ($30.06/run • $13015.98/mo gross | $5206.76/mo at 40% context-switch loss) • Concurrency: 162 peak jobs (162 hosted > 20 cap, +$510.00/mo tier upgrade)
 • Artifact Storage : 123 upload job(s) • 3d retention ($8.12/mo, 32.48 GB steady-state)
 • Pipeline Hygiene : Multi-service (11 units, 11% setup overhead) • Cache 148/162 jobs (caching 7 uncached saves ~$123.84/mo) • Path filters missing on 2 wf (~$7726.63/mo selective savings)
 • Recommendation   : Right-size heavy jobs to ARM64 / High-Speed Runners ($11853.12/mo, saves $8353.69/mo) or Cloud Spot ($6783.54/mo, saves $13423.27/mo)
```

---

## 6. Strategic Customer Impact Summary

1. **Zero-Config Stage & Topology Awareness:** `minici` classifies every repository's team stage (`startup-pre-seed`, `startup-seed-series-a`, `scaleup`, `enterprise`) and pipeline shape (`single-service`, `monorepo`, `multi-service-microservices`) purely from static AST and manifest signals in `< 3.5 ms`.
2. **Immediate Hardware & Matrix ROI for macOS/Multi-OS Teams:** On macOS-intensive repositories, `minici` proves that a `$700` Mac mini pays for itself in **1.5 days** on `denoland/deno`, **10 days** on `pydantic/pydantic`, and **25 days** on `astral-sh/uv`, while shifting non-Linux PR matrices to nightly runs saves **$867.74/mo to $16,975.97/mo**.
3. **Quantified Justification for Platform Engineering & Monorepo Tooling:** Rather than relying on intuition, engineering leaders get exact dollar figures for adopting affected-only graph execution (**$286.88/mo** on `immich` to **$3,450.91/mo** on `pydantic`), quarterly required-check pruning (**$230.70/mo to $7,072.38/mo**), and Platform Engineer FTE payback (**36.26% to 71.34% coverage** from a single repository).
4. **Enterprise Governance & Supply-Chain Hygiene:** CISOs and VP Eng teams gain automated visibility into compliance control coverage (`0 to 5` controls), multi-region residency cost multipliers (`1.8x` on dual-region `MaximVote/maxim`), 1-year SIEM audit log retention budgets (up to **$1,796.15/mo** on `llama.cpp`), and unpinned `@main/@master` template vulnerabilities.
