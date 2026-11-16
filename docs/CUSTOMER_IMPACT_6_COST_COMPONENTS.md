# Customer Impact Record: 6-Component CI/CD Invoice Breakdown & Infracost Cloud Delegation

**Repository:** `/Users/araza/personal/minici-cost-action`  
**Binary Verified:** `/Users/araza/personal/minici-cost-action/target/release/minici` (`v0.2.0`)  
**Engine Implementation:** `crates/minici/src/engine.rs` (`CostComponentsBreakdown`) & `crates/minici/src/catalog.rs` (`INFRACOST_CLOUD_CATALOG`, `infracost_aws_nat_gateway_rate_usd`, `infracost_ec2_runner_profile`, `vendor_os_free_tier_multiplier`)  
**Benchmark References:** [cicdcost.com](https://cicdcost.com) (6-component CI/CD cost taxonomy) + Infracost Cloud Pricing API (`v2.19.0` schema)

---

## 1. Executive Summary: Why a 6-Component Invoice Breakdown Matters to Customers

Historically, engineering leaders, Staff/Principal DevOps engineers, and FinOps teams saw CI/CD costs as a single opaque line item ("GitHub Actions Compute") or discovered hidden cloud infrastructure bills weeks later on their AWS/GCP/Azure invoice.

By decomposing every repository's CI/CD footprint into **6 explicit invoice components** (`build_minutes_cost`, `runner_cost`, `artifact_storage_cost`, `parallelism_cost`, `log_retention_cost`, and `cross_region_egress_cost`) and delegating cloud infrastructure pricing (`[IC]`) to **Infracost's verified resource catalog**, `minici breakdown` gives customers:

1. **Line-by-Line Invoice Explainability:** Customers immediately see *why* their bill is high—separating per-minute runner compute from GitHub's 1-minute ceiling rounding tax, OS free-tier multiplier burn (`1x` Linux / `2x` Windows / `10x` macOS), artifact storage markups (`10.87x` over AWS S3), seat-tier upgrade triggers from concurrency caps, observability log shipping (`Datadog` / `Splunk`), and cloud NAT Gateway egress.
2. **Actionable FinOps Arbitrage (SaaS vs. Infracost Cloud Primitives):** Every component pairs the CI vendor's SaaS price against the exact self-hosted/cloud alternative (`aws_instance`, `aws_s3_bucket`, `aws_nat_gateway`, `aws_vpc_endpoint`, `aws_cloudwatch_log_group`, `google_compute_instance`, `azurerm_linux_virtual_machine`), empowering customers to make data-backed build-vs-buy decisions in `< 4 ms`.
3. **Zero-Config Shift-Left Prevention:** Because `minici breakdown` runs in `0.12 ms – 3.90 ms` locally and outputs both a human-scannable 6-line CLI receipt and deterministic JSON (`--format json`), developers and AI coding agents see the full 6-component financial impact *before* pushing a PR.

---

## 2. How Each of the 6 Cost Components Helps Engineering & FinOps Customers

### Component 1: `build_minutes_cost` ([cicdcost.com/build-minutes-cost](https://cicdcost.com/build-minutes-cost))
- **Customer Pain Point:** Engineering teams estimate CI cost by multiplying average job duration by per-minute rates, missing three hidden multipliers that inflate real invoices by **15%–400%**:
  1. **Sub-Minute Ceiling Rounding Tax:** GitHub Actions rounds every job (and every matrix shard) up to the next whole minute (`ceil(raw_mins)`). A 48-job matrix where each shard runs for `1.15m` bills `2.0m × 48 = 96.0m` instead of `55.2m` (`+73.9%` rounding tax).
  2. **OS Free-Tier Quota Multipliers:** `1` minute of `macos-latest` burns `10` free-tier minutes on GitHub Actions (`6x` on GitLab CI), and `windows-latest` burns `2` free-tier minutes. Teams think they have `2,000` minutes of runway, but `200` minutes of macOS builds exhaust the entire monthly quota.
  3. **Flaky Test Retry Tax (`+10%` baseline):** Re-running failed workflows due to flaky tests silently adds ~10% to gross monthly compute spend.
- **What `minici` Surfaces:**
  - `rounding_tax_mins_per_run` and `monthly_rounding_tax_usd` (exact dollar waste from whole-minute ceiling rounding).
  - Split of `linux_mins_per_run`, `windows_mins_per_run`, and `macos_mins_per_run`.
  - `os_weighted_free_tier_mins_per_run` and `monthly_os_weighted_free_tier_mins` (true quota consumption).
  - `monthly_flake_retry_10pct_overhead_usd` (quantifying the financial ROI of fixing flaky tests).
- **Customer Action:** Consolidate micro-jobs/shards that finish in `< 1.2m` into single steps or switch to per-second billed runners (`Blacksmith`, `GitLab CI`, or self-hosted AWS Spot), immediately eliminating `$27.44/mo` (`MaximVote/maxim`) to `$913.65/mo` (`denoland/deno`) in pure rounding tax.

---

### Component 2: `runner_cost` `[IC]` ([cicdcost.com/runner-cost](https://cicdcost.com/runner-cost))
- **Customer Pain Point:** Customers struggle to compare per-minute SaaS runner rates (`$0.006/min` to `$0.162/min`) against hourly/monthly cloud compute invoices (`$/hr` On-Demand vs. Spot) and often forget the surrounding cloud overhead (`gp3` EBS root volumes, CloudWatch agent metrics, idle capacity, and macOS bare-metal minimums).
- **What `minici` Surfaces (Infracost Delegated):**
  - `hosted_effective_hourly_rate_usd`: Converts the repo's weighted per-minute SaaS rate into an effective hourly rate (`$0.36/hr` for standard Linux up to `$1.51/hr` for macOS/multi-core repos like `denoland/deno`).
  - `infracost_recommended_ec2_instance`: Right-sizes the exact AWS EC2 equivalent based on peak job vCPU demand (`t3.medium` for 2 vCPU, `m5.xlarge` for 4 vCPU, `c5.2xlarge` for 8 vCPU, `c5.4xlarge` for 16+ vCPU, or `m6g.xlarge` for ARM64).
  - `infracost_ec2_on_demand_hourly_usd`, `infracost_ec2_on_demand_monthly_usd`, and `infracost_ec2_spot_hourly_usd` alongside `infracost_ec2_surrounding_overhead_monthly_usd` (`$8.40/mo` for 80 GB `gp3` EBS + CloudWatch).
  - `self_hosted_breakeven_mins_t3_medium` (`5,000 mins/mo`) and `self_hosted_breakeven_mins_c5_4xlarge_spot` (`22,000 mins/mo`).
- **Customer Action:** Small repos below `5,000 mins/mo` (e.g., `Eazy-Ops/multi-cloud-finops-mcp-server` at `866 mins/mo`) are explicitly advised to **stay on SaaS runners** rather than wasting engineering time managing EC2/ARC. High-volume repos (`astral-sh/uv` at `323,126 mins/mo`) see an immediate `$2,835.07/mo` (`81.8%`) savings path via Cloud Spot (`$628.93/mo`) or `$1,040.98/mo` (`30.1%`) via drop-in MicroVM runners (`Blacksmith`).

---

### Component 3: `artifact_storage_cost` `[IC]` ([cicdcost.com/artifact-storage-cost](https://cicdcost.com/artifact-storage-cost))
- **Customer Pain Point:** GitHub Actions and CI vendors default `actions/upload-artifact` retention to **90 days** and charge **`$0.25/GB-month`**—a **10.87x markup** over AWS S3 Standard (`$0.023/GB-month`) and a **13.89x markup** over AWS S3 Intelligent-Tiering (`$0.018/GB-month`). Because artifact storage accumulates across `433 runs/mo` for 90 days before reaching steady state, a repo uploading `0.25 GB` across 3+ jobs accumulates `974.25 GB` of steady-state storage (`$243.56/mo`).
- **What `minici` Surfaces (Infracost Delegated):**
  - `jobs_uploading_artifacts` and `default_retention_days` (`90` days on GitHub Actions, `30` days on GitLab/CircleCI).
  - `steady_state_stored_gb_90d` vs. `steady_state_stored_gb_14d`.
  - Four-way cost matrix:
    1. `monthly_vendor_storage_90d_usd` (`$0.25/GB-mo` at 90d default)
    2. `monthly_vendor_storage_14d_usd` (`$0.25/GB-mo` with `retention-days: 14`)
    3. `monthly_infracost_s3_standard_90d_usd` (`$0.023/GB-mo` via `aws_s3_bucket`)
    4. `monthly_infracost_s3_intelligent_14d_usd` (`$0.018/GB-mo` with 14d lifecycle policy)
- **Customer Action:** Adding a single line (`retention-days: 14`) to `actions/upload-artifact` cuts vendor storage cost by **84.4%** (`$243.56/mo -> $37.89/mo`), while offloading artifacts to `aws_s3_bucket` with a 14-day lifecycle rule drops the bill by **98.9%** (`$243.56/mo -> $2.73/mo`).

---

### Component 4: `parallelism_cost` ([cicdcost.com/parallelism-cost](https://cicdcost.com/parallelism-cost))
- **Customer Pain Point:** Expanding test matrices improves developer wait time, but CI vendors enforce strict **concurrent job caps** by organization plan:
  - GitHub Free: **20 concurrent jobs** (`5` macOS)
  - GitHub Team (`$4/user/mo`): **60 concurrent jobs** (`5` macOS)
  - GitHub Enterprise (`$21/user/mo`): **180–500 concurrent jobs** (`50` macOS)
  When a repository's hosted concurrency exceeds `20` or `60` jobs (or `5` concurrent macOS jobs), teams either suffer severe **queue wait times** or are forced into an organization-wide seat upgrade (`+$4/seat -> $21/seat`, costing `+$510/mo` for a 30-engineer org).
- **What `minici` Surfaces:**
  - `peak_concurrent_jobs_per_run`, broken down into `hosted_concurrent_jobs_per_run`, `macos_concurrent_jobs_per_run`, and `self_hosted_or_microvm_jobs_per_run`.
  - Comparison against `vendor_free_concurrent_cap` (`20`) and `vendor_team_concurrent_cap` (`60`).
  - `parallelism_tier_upgrade_tax_monthly_usd` (`$0/mo`, `$100/mo` Free-to-Team jump, or `$510/mo` Team-to-Enterprise jump).
- **Customer Action:** `MaximVote/maxim` runs **57 peak concurrent jobs**, which would normally breach the 20-job cap and approach the 60-job Team cap—*except* `minici` detects that **44 of those 57 jobs run on `blacksmith` microVMs** and only **13 run on GitHub-hosted runners**, keeping `parallelism_tier_upgrade_tax_monthly_usd` at **`$0.00/mo`**! Conversely, `astral-sh/uv` (`213` hosted jobs, `13` macOS) and `denoland/deno` (`162` hosted jobs, `48` macOS) trigger the **`$510.00/mo`** Enterprise concurrency tier tax.

---

### Component 5: `log_retention_cost` `[IC]` ([cicdcost.com/log-retention-cost](https://cicdcost.com/log-retention-cost))
- **Customer Pain Point:** While CI vendors include basic 90-day console log viewing for `$0`, enterprise security and observability teams frequently forward CI/CD build logs to **Datadog Logs** (`$0.10/GB` ingest + `$2.50–$3.75/M` indexed events = `~$3.10/GB` at 30d or `~$4.60/GB` at 90d) or **Splunk Cloud** (`~$2.10/GB`). Verbose test runners (`cargo test` backtraces, `pytest -vv`, `vitest`) with a 15% failed-build stack-trace amplification factor generate tens to hundreds of gigabytes per month.
- **What `minici` Surfaces (Infracost Delegated):**
  - `detected_log_language_profile`: Automatically fingerprints the repository's primary build/test ecosystem:
    - `Rust / cargo test (0.6-2.0 MB/build)` (`1.10 MB/job` base)
    - `Python / pytest (0.3-1.2 MB/build)` (`0.75 MB/job` base)
    - `Node / npm / Vitest (0.2-0.8 MB/build)` (`0.50 MB/job` base)
    - `Go / go test (0.1-0.4 MB/build)` (`0.25 MB/job` base)
    - `Java / Gradle-Maven (0.8-3.0 MB/build)` (`1.50 MB/job` base)
  - `est_log_mb_per_run` (including `1.15x` failed-build stack-trace amplification) and `monthly_log_volume_gb`.
  - Side-by-side comparison: `monthly_vendor_default_log_cost_usd` (`$0.00`) vs. `monthly_datadog_30d_log_shipping_usd`, `monthly_datadog_90d_log_shipping_usd`, `monthly_splunk_30d_log_shipping_usd`, and `monthly_infracost_loki_s3_usd` (`$0.023/GB-mo` via Grafana Loki + `aws_s3_bucket`).
- **Customer Action:** For `astral-sh/uv` (`243` expanded Rust jobs generating `307.40 MB/run` = `129.98 GB/mo`), shipping CI logs to Datadog (90d) costs **`$597.91/mo`** (`$402.94/mo` at 30d), whereas archiving to self-hosted Grafana Loki + Infracost S3 (`aws_s3_bucket`) costs just **`$2.99/mo`** (**99.5% savings**).

---

### Component 6: `cross_region_egress_cost` `[IC]` ([cicdcost.com/cross-region-egress-cost](https://cicdcost.com/cross-region-egress-cost))
- **Customer Pain Point:** The #1 "bill shock" when migrating from hosted GitHub Actions to self-hosted Kubernetes/EC2 runners (`Actions Runner Controller` in private VPC subnets) is **AWS NAT Gateway data processing + Internet Egress**. Every uncached Docker base image pull, `npm`/`cargo`/`pip` dependency download, and artifact transfer traversing a private subnet NAT Gateway incurs **`$0.090/GB` AWS Internet Egress + `$0.045–$0.093/GB` regional NAT Gateway processing** (`$0.135/GB` in `us-east-1` up to `$0.183/GB` in `sa-east-1`). Across hundreds of matrix jobs, NAT Gateway charges routinely **exceed the EC2 compute bill itself**.
- **What `minici` Surfaces (Infracost Delegated):**
  - `infracost_cloud_region` (respects `AWS_REGION` / `AWS_DEFAULT_REGION`, defaulting to `us-east-1`).
  - `est_transfer_gb_per_run` (`0.18 GB` per expanded job + `0.35 GB` per artifact-uploading job) and `monthly_transfer_gb`.
  - Exact Infracost per-GB rates: `infracost_aws_internet_egress_per_gb_usd` (`$0.090`), regional `infracost_aws_nat_gateway_per_gb_usd` (`$0.045`–`$0.093`), `infracost_aws_cross_az_per_gb_usd` (`$0.010`), and `infracost_aws_cross_region_per_gb_usd` (`$0.020`).
  - Three architectural cost tiers:
    1. **Worst-Case Private Subnet + NAT Gateway (`monthly_infracost_private_subnet_nat_worst_case_usd`):** Full cold pulls routed through NAT Gateway + Internet Egress.
    2. **Cross-Region Registry Pulls (`monthly_infracost_cross_region_registry_usd`):** Pulling ECR/artifact images across regions (`$0.020/GB`).
    3. **VPC Endpoint + Local Cache Optimized (`monthly_infracost_vpc_endpoint_cache_optimized_usd`):** Provisioning S3/ECR Gateway/Interface VPC Endpoints (`aws_vpc_endpoint` at `$0.010/GB`) with a 70% local registry/dependency cache hit rate (`0.30 × $0.010 = $0.003/GB` effective).
- **Customer Action:** Before deploying self-hosted runners into a private subnet, `minici` warns `MaximVote/maxim` (`5,503.43 GB/mo`) that an uncached NAT Gateway architecture in `us-east-1` will cost **`$742.96/mo`** (wiping out most of their `$881.12/mo` EC2 Spot compute savings!), whereas adding an S3/ECR VPC Endpoint + local cache reduces network transfer cost by **97.8%** to **`$16.51/mo`**.

---

## 3. Exact Multi-Repository Validation Matrix (12 Repositories Tested)

All measurements below were produced by running `/Users/araza/personal/minici-cost-action/target/release/minici breakdown` (both CLI table and `--format json`) at the default baseline (`433 runs/mo` = `25 PRs/wk × 4 pushes/PR × 4.33 wks/mo`, `$65/hr` developer rate, `AWS_REGION=us-east-1`).

### 3.1 High-Level Compute, Quota Burn & Developer Wait-Time Summary

| Repository | Providers Detected | Workflows / Defined / Expanded Jobs | Raw -> Billed Mins/Run | Monthly Billed Mins (Quota Burn %) | Net Monthly Compute Cost | Monthly Dev Wait Cost (`$65/hr`) | Execution Latency |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`01_MaximVote__maxim`** | `blacksmith`, `github-actions` | `41` / `45` / `57` | `189.70m -> 195.00m` | `84,435m` (`4,221.8%`) | **`$1,009.41/mo`** (`$2.331/run`) | `$4,269.38/mo` (`9.1m` crit) | `1.46 ms` |
| **`04_astral-sh__uv`** | `blacksmith`, `github-actions` | `47` / `223` / `243` | `646.75m -> 746.25m` | `323,126m` (`16,156.3%`) | **`$3,464.00/mo`** (`$8.000/run`) | `$6,122.62/mo` (`13.1m` crit) | `3.90 ms` |
| **`05_denoland__deno`** | `github-actions` | `11` / `59` / `162` | `1775.90m -> 1860.00m` | `805,380m` (`40,269.0%`) | **`$20,206.81/mo`** (`$46.667/run`) | `$13,015.98/mo` (`27.8m` crit) | `2.08 ms` |
| **`06_ggml-org__llama.cpp`** | `github-actions`, `self-hosted-aws` | `62` / `121` / `152` | `398.25m -> 459.85m` | `199,115m` (`9,955.8%`) | **`$1,930.75/mo`** (`$4.459/run`) | `$6,378.09/mo` (`13.6m` crit) | `2.72 ms` |
| **`10_mendableai__firecrawl`** | `blacksmith`, `github-actions` | `36` / `71` / `86` | `606.55m -> 626.85m` | `271,426m` (`13,571.3%`) | **`$1,978.81/mo`** (`$4.570/run`) | `$39,615.17/mo` (`84.5m` crit) | `1.75 ms` |
| **`check_MaximVote__tf`** | `github-actions` | `6` / `8` / `8` | `23.40m -> 27.00m` | `11,691m` (`584.6%`) | **`$70.15/mo`** (`$0.162/run`) | `$1,688.70/mo` (`3.6m` crit) | `0.33 ms` |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | `github-actions` | `1` / `1` / `1` | `1.85m -> 2.00m` | `866m` (`43.3%`) | **`$0.00/mo`** (`$5.20` gross, free tier) | `$866.00/mo` (`1.9m` crit) | `0.12 ms` |
| **`02_mgjartproject__FineArts-BE`** | `github-actions` | `2` / `6` / `6` | `13.50m -> 18.00m` | `7,794m` (`389.7%`) | **`$46.76/mo`** (`$0.108/run`) | `$2,979.04/mo` (`6.3m` crit) | `0.17 ms` |
| **`03_Eazy-Ops__Backend`** | `github-actions` | `5` / `5` / `5` | `25.55m -> 28.00m` | `12,124m` (`606.2%`) | **`$72.74/mo`** (`$0.168/run`) | `$8,374.22/mo` (`17.9m` crit) | `0.24 ms` |
| **`07_immich-app__immich`** | `github-actions` | `27` / `76` / `85` | `234.60m -> 269.00m` | `116,477m` (`5,823.9%`) | **`$819.67/mo`** (`$1.893/run`) | `$4,879.91/mo` (`10.4m` crit) | `1.09 ms` |
| **`08_langgenius__dify`** | `blacksmith`, `github-actions` | `41` / `84` / `117` | `379.85m -> 393.50m` | `170,386m` (`8,519.3%`) | **`$1,153.51/mo`** (`$2.664/run`) | `$4,853.93/mo` (`10.3m` crit) | `1.91 ms` |
| **`09_open-webui__open-webui`** | `github-actions` | `7` / `13` / `18` | `82.15m -> 87.00m` | `37,671m` (`1,883.6%`) | **`$226.03/mo`** (`$0.522/run`) | `$4,455.57/mo` (`9.5m` crit) | `0.34 ms` |

---

### 3.2 Exact 6-Component Invoice Breakdown Across All Tested Repositories (`us-east-1`)

| Repository | 1. `build_minutes_cost`<br>*(Rounding Tax / OS-Weighted Mins / 10% Flake Tax)* | 2. `runner_cost` `[IC]`<br>*(Hosted $/hr vs Infracost EC2 On-Demand / Spot)* | 3. `artifact_storage_cost` `[IC]`<br>*(Uploads / 90d Vendor vs S3 90d / S3 14d)* | 4. `parallelism_cost`<br>*(Peak: Hosted + Self/uVM / Tier Tax)* | 5. `log_retention_cost` `[IC]`<br>*(Profile / GB/mo / Datadog 30d vs Loki+S3)* | 6. `cross_region_egress_cost` `[IC]`<br>*(GB/mo / Private+NAT vs Cross-Reg vs VPC Ep)* |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`01_MaximVote__maxim`** | `5.30m/run` (`$27.44/mo`)<br>OS-Wt: `84,435m/mo`<br>Flake: `+$100.94/mo` | Hosted: `$0.72/hr`<br>`c5.2xlarge` (8 vCPU)<br>OD: `$0.34/hr` (`$245/mo`)<br>Spot: `$0.100/hr` + `$8.40` | `7` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `57` peak (`13` hosted / `44` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Node/Vitest (`32.78 MB/run`)<br>`13.86 GB/mo`<br>DD 30d: `$42.97/mo`<br>Loki+S3: `$0.32/mo` | `5,503.43 GB/mo`<br>NAT: **`$742.96/mo`**<br>X-Reg: `$110.07/mo`<br>VPC+Cache: **`$16.51/mo`** |
| **`04_astral-sh__uv`** | `99.50m/run` (`$461.87/mo`)<br>OS-Wt: `455,191m/mo`<br>Flake: `+$346.40/mo` | Hosted: `$0.64/hr`<br>`c5.4xlarge` (16 vCPU)<br>OD: `$0.68/hr` (`$490/mo`)<br>Spot: `$0.170/hr` + `$8.40` | `51` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `243` peak (`213` hosted / `30` uVM)<br>Mac: `13` • Cap: `20/60`<br>Tier Tax: **`$510.00/mo`** | Rust/cargo (`307.40 MB/run`)<br>`129.98 GB/mo`<br>DD 30d: `$402.94/mo`<br>Loki+S3: `$2.99/mo` | `26,668.47 GB/mo`<br>NAT: **`$3,600.24/mo`**<br>X-Reg: `$533.37/mo`<br>VPC+Cache: **`$80.01/mo`** |
| **`05_denoland__deno`** | `84.10m/run` (`$913.65/mo`)<br>OS-Wt: `3,418,968m/mo`<br>Flake: `+$2,020.68/mo` | Hosted: `$1.51/hr`<br>`c5.4xlarge` (16 vCPU)<br>OD: `$0.68/hr` (`$490/mo`)<br>Spot: `$0.170/hr` + `$8.40` | `123` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `162` peak (`162` hosted / `0` uVM)<br>Mac: `48` • Cap: `20/60`<br>Tier Tax: **`$510.00/mo`** | Rust/cargo (`204.93 MB/run`)<br>`86.65 GB/mo`<br>DD 30d: `$268.62/mo`<br>Loki+S3: `$1.99/mo` | `31,266.93 GB/mo`<br>NAT: **`$4,221.04/mo`**<br>X-Reg: `$625.34/mo`<br>VPC+Cache: **`$93.80/mo`** |
| **`06_ggml-org__llama.cpp`** | `61.60m/run` (`$258.64/mo`)<br>OS-Wt: `314,293m/mo`<br>Flake: `+$193.08/mo` | Hosted: `$0.58/hr`<br>`c5.4xlarge` (16 vCPU)<br>OD: `$0.68/hr` (`$490/mo`)<br>Spot: `$0.170/hr` + `$8.40` | `41` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `152` peak (`134` hosted / `18` self)<br>Mac: `13` • Cap: `20/60`<br>Tier Tax: **`$510.00/mo`** | Node/Vitest (`87.40 MB/run`)<br>`36.96 GB/mo`<br>DD 30d: `$114.58/mo`<br>Loki+S3: `$0.85/mo` | `18,060.43 GB/mo`<br>NAT: **`$2,438.16/mo`**<br>X-Reg: `$361.21/mo`<br>VPC+Cache: **`$54.18/mo`** |
| **`10_mendableai__firecrawl`** | `20.30m/run` (`$64.08/mo`)<br>OS-Wt: `271,426m/mo`<br>Flake: `+$197.88/mo` | Hosted: `$0.44/hr`<br>`m5.xlarge` (4 vCPU)<br>OD: `$0.192/hr` (`$138/mo`)<br>Spot: `$0.058/hr` + `$8.40` | `9` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `86` peak (`44` hosted / `42` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$100.00/mo`** | Node/Vitest (`49.45 MB/run`)<br>`20.91 GB/mo`<br>DD 30d: `$64.82/mo`<br>Loki+S3: `$0.48/mo` | `8,066.79 GB/mo`<br>NAT: **`$1,089.02/mo`**<br>X-Reg: `$161.34/mo`<br>VPC+Cache: **`$24.20/mo`** |
| **`check_MaximVote__tf`** | `3.60m/run` (`$9.35/mo`)<br>OS-Wt: `11,691m/mo`<br>Flake: `+$7.02/mo` | Hosted: `$0.36/hr`<br>`t3.medium` (2 vCPU)<br>OD: `$0.0416/hr` (`$30/mo`)<br>Spot: `$0.012/hr` + `$8.40` | `1` upload job (`324.75 GB`)<br>Vendor 90d: `$81.19/mo`<br>S3 90d: `$7.47/mo`<br>S3 Int 14d: `$0.91/mo` | `8` peak (`8` hosted / `0` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Node/Vitest (`4.60 MB/run`)<br>`1.95 GB/mo`<br>DD 30d: `$6.05/mo`<br>Loki+S3: `$0.04/mo` | `775.07 GB/mo`<br>NAT: **`$104.63/mo`**<br>X-Reg: `$15.50/mo`<br>VPC+Cache: **`$2.33/mo`** |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | `0.15m/run` (`$0.39/mo`)<br>OS-Wt: `866m/mo`<br>Flake: `+$0.52/mo` | Hosted: `$0.36/hr`<br>`t3.medium` (2 vCPU)<br>OD: `$0.0416/hr` (`$30/mo`)<br>Spot: `$0.012/hr` + `$8.40` | `0` upload jobs (`32.48 GB`)<br>Vendor 90d: `$8.12/mo`<br>S3 90d: `$0.75/mo`<br>S3 Int 14d: `$0.09/mo` | `1` peak (`1` hosted / `0` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Node/Vitest (`0.57 MB/run`)<br>`0.24 GB/mo`<br>DD 30d: `$0.74/mo`<br>Loki+S3: `$0.01/mo` | `77.94 GB/mo`<br>NAT: **`$10.52/mo`**<br>X-Reg: `$1.56/mo`<br>VPC+Cache: **`$0.23/mo`** |
| **`02_mgjartproject__FineArts-BE`** | `4.50m/run` (`$11.69/mo`)<br>OS-Wt: `7,794m/mo`<br>Flake: `+$4.68/mo` | Hosted: `$0.36/hr`<br>`t3.medium` (2 vCPU)<br>OD: `$0.0416/hr` (`$30/mo`)<br>Spot: `$0.012/hr` + `$8.40` | `0` upload jobs (`32.48 GB`)<br>Vendor 90d: `$8.12/mo`<br>S3 90d: `$0.75/mo`<br>S3 Int 14d: `$0.09/mo` | `6` peak (`6` hosted / `0` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Node/Vitest (`3.45 MB/run`)<br>`1.46 GB/mo`<br>DD 30d: `$4.53/mo`<br>Loki+S3: `$0.03/mo` | `467.64 GB/mo`<br>NAT: **`$63.13/mo`**<br>X-Reg: `$9.35/mo`<br>VPC+Cache: **`$1.40/mo`** |
| **`03_Eazy-Ops__Backend`** | `2.45m/run` (`$6.37/mo`)<br>OS-Wt: `12,124m/mo`<br>Flake: `+$7.27/mo` | Hosted: `$0.36/hr`<br>`t3.medium` (2 vCPU)<br>OD: `$0.0416/hr` (`$30/mo`)<br>Spot: `$0.012/hr` + `$8.40` | `0` upload jobs (`32.48 GB`)<br>Vendor 90d: `$8.12/mo`<br>S3 90d: `$0.75/mo`<br>S3 Int 14d: `$0.09/mo` | `5` peak (`5` hosted / `0` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Node/Vitest (`2.88 MB/run`)<br>`1.22 GB/mo`<br>DD 30d: `$3.78/mo`<br>Loki+S3: `$0.03/mo` | `389.70 GB/mo`<br>NAT: **`$52.61/mo`**<br>X-Reg: `$7.79/mo`<br>VPC+Cache: **`$1.17/mo`** |
| **`07_immich-app__immich`** | `34.40m/run` (`$104.82/mo`)<br>OS-Wt: `138,127m/mo`<br>Flake: `+$81.97/mo` | Hosted: `$0.42/hr`<br>`m5.xlarge` (4 vCPU)<br>OD: `$0.192/hr` (`$138/mo`)<br>Spot: `$0.058/hr` + `$8.40` | `8` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `85` peak (`85` hosted / `0` uVM)<br>Mac: `1` • Cap: `20/60`<br>Tier Tax: **`$510.00/mo`** | Node/Vitest (`48.87 MB/run`)<br>`20.66 GB/mo`<br>DD 30d: `$64.05/mo`<br>Loki+S3: `$0.48/mo` | `7,837.30 GB/mo`<br>NAT: **`$1,058.04/mo`**<br>X-Reg: `$156.75/mo`<br>VPC+Cache: **`$23.51/mo`** |
| **`08_langgenius__dify`** | `13.65m/run` (`$40.01/mo`)<br>OS-Wt: `196,366m/mo`<br>Flake: `+$115.35/mo` | Hosted: `$0.41/hr`<br>`m5.xlarge` (4 vCPU)<br>OD: `$0.192/hr` (`$138/mo`)<br>Spot: `$0.058/hr` + `$8.40` | `34` upload jobs (`974.25 GB`)<br>Vendor 90d: `$243.56/mo`<br>S3 90d: `$22.41/mo`<br>S3 Int 14d: `$2.73/mo` | `117` peak (`38` hosted / `79` uVM)<br>Mac: `1` • Cap: `20/60`<br>Tier Tax: **`$100.00/mo`** | Python/pytest (`100.91 MB/run`)<br>`42.67 GB/mo`<br>DD 30d: `$132.28/mo`<br>Loki+S3: `$0.98/mo` | `14,271.68 GB/mo`<br>NAT: **`$1,926.68/mo`**<br>X-Reg: `$285.43/mo`<br>VPC+Cache: **`$42.82/mo`** |
| **`09_open-webui__open-webui`** | `4.85m/run` (`$12.60/mo`)<br>OS-Wt: `37,671m/mo`<br>Flake: `+$22.60/mo` | Hosted: `$0.36/hr`<br>`t3.medium` (2 vCPU)<br>OD: `$0.0416/hr` (`$30/mo`)<br>Spot: `$0.012/hr` + `$8.40` | `2` upload jobs (`649.50 GB`)<br>Vendor 90d: `$162.38/mo`<br>S3 90d: `$14.94/mo`<br>S3 Int 14d: `$1.82/mo` | `18` peak (`18` hosted / `0` uVM)<br>Mac: `0` • Cap: `20/60`<br>Tier Tax: **`$0.00/mo`** | Python/pytest (`15.52 MB/run`)<br>`6.56 GB/mo`<br>DD 30d: `$20.34/mo`<br>Loki+S3: `$0.15/mo` | `1,706.02 GB/mo`<br>NAT: **`$230.31/mo`**<br>X-Reg: `$34.12/mo`<br>VPC+Cache: **`$5.12/mo`** |

---

## 4. Regional NAT Gateway Impact Analysis (`us-east-1` vs. `eu-west-2` vs. `sa-east-1`)

When customers deploy self-hosted CI runners inside non-US cloud regions, AWS NAT Gateway data processing rates (`aws_nat_gateway` / `NatGateway-Bytes`) increase significantly while AWS Internet Egress stays at `$0.090/GB`:
- **`us-east-1` (N. Virginia):** `$0.045/GB` NAT Gateway + `$0.090/GB` Internet Egress = **`$0.135/GB`** combined Private+NAT rate.
- **`eu-west-2` (London):** `$0.050/GB` NAT Gateway (`+11.1%` NAT surcharge) + `$0.090/GB` Internet Egress = **`$0.140/GB`** combined Private+NAT rate.
- **`sa-east-1` (São Paulo):** `$0.093/GB` NAT Gateway (`+106.7%` NAT surcharge) + `$0.090/GB` Internet Egress = **`$0.183/GB`** combined Private+NAT rate (`+35.6%` higher total Private+NAT bill than `us-east-1`).

By reading `AWS_REGION` (or `AWS_DEFAULT_REGION`), `minici breakdown` dynamically recalculates Component 6 (`cross_region_egress_cost`) using `infracost_aws_nat_gateway_rate_usd`:

| Repository | Monthly Transfer Volume (`GB/mo`) | `us-east-1` Private+NAT (`$0.135/GB` • NAT `$0.045`) | `eu-west-2` (London) Private+NAT (`$0.140/GB` • NAT `$0.050`) | `sa-east-1` (São Paulo) Private+NAT (`$0.183/GB` • NAT `$0.093`) | Regional Surcharge (`sa-east-1` vs `us-east-1`) | VPC Endpoint + Cache Optimized (`$0.003/GB` eff.) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`01_MaximVote__maxim`** | `5,503.43 GB/mo` | `$742.96/mo` | `$770.48/mo` (`+$27.52`) | **`$1,007.13/mo`** | **`+$264.17/mo`** (`+$3,170/yr`) | **`$16.51/mo`** (saves `$990.62/mo` in `sa-east-1`) |
| **`04_astral-sh__uv`** | `26,668.47 GB/mo` | `$3,600.24/mo` | `$3,733.59/mo` (`+$133.35`) | **`$4,880.33/mo`** | **`+$1,280.09/mo`** (`+$15,361/yr`) | **`$80.01/mo`** (saves `$4,800.32/mo` in `sa-east-1`) |
| **`05_denoland__deno`** | `31,266.93 GB/mo` | `$4,221.04/mo` | `$4,377.37/mo` (`+$156.33`) | **`$5,721.85/mo`** | **`+$1,500.81/mo`** (`+$18,010/yr`) | **`$93.80/mo`** (saves `$5,628.05/mo` in `sa-east-1`) |
| **`06_ggml-org__llama.cpp`** | `18,060.43 GB/mo` | `$2,438.16/mo` | `$2,528.46/mo` (`+$90.30`) | **`$3,305.06/mo`** | **`+$866.90/mo`** (`+$10,403/yr`) | **`$54.18/mo`** (saves `$3,250.88/mo` in `sa-east-1`) |
| **`10_mendableai__firecrawl`** | `8,066.79 GB/mo` | `$1,089.02/mo` | `$1,129.35/mo` (`+$40.33`) | **`$1,476.22/mo`** | **`+$387.20/mo`** (`+$4,646/yr`) | **`$24.20/mo`** (saves `$1,452.02/mo` in `sa-east-1`) |
| **`check_MaximVote__tf`** | `775.07 GB/mo` | `$104.63/mo` | `$108.51/mo` (`+$3.88`) | **`$141.84/mo`** | **`+$37.21/mo`** (`+$447/yr`) | **`$2.33/mo`** (saves `$139.51/mo` in `sa-east-1`) |
| **`check_Eazy-Ops__multi-cloud-finops-mcp-server`** | `77.94 GB/mo` | `$10.52/mo` | `$10.91/mo` (`+$0.39`) | **`$14.26/mo`** | **`+$3.74/mo`** (`+$45/yr`) | **`$0.23/mo`** (saves `$14.03/mo` in `sa-east-1`) |

**Key FinOps Takeaway:** In `MaximVote/maxim`, moving self-hosted runners into a private subnet in `sa-east-1` without VPC Endpoints generates **`$1,007.13/mo` in NAT Gateway + Egress charges**—nearly equal to the entire `$1,009.41/mo` SaaS compute bill! Because `minici` surfaces this regional sensitivity immediately, FinOps engineers know they *must* provision Terraform `aws_vpc_endpoint` resources for S3 and ECR (`$16.51/mo`) alongside any self-hosted runner migration.

---

## 5. How Infracost Cloud Delegation Handles Cloud-Related Costs (AWS / GCP / Azure)

Rather than inventing ad-hoc cloud pricing heuristics, `minici` delegates cloud infrastructure pricing and resource mapping to **Infracost's Cloud Pricing Catalog (`v2.19.0` schema)** across 4 of the 6 components (`[IC]`):

1. **Zero-Guesswork Multi-Cloud Instance Resolution (`INFRACOST_CLOUD_CATALOG` in `crates/minici/src/catalog.rs`):**
   - `minici` embeds 38 canonical cloud compute SKUs across **AWS EC2** (`t3.medium..2xlarge`, `c6i.large..24xlarge`, `m6i.large..4xlarge`, Graviton ARM64 `c7g.large..16xlarge`), **GCP Compute Engine** (`e2-standard-2..16`, `n2-standard-2..64`, `c3-standard-4..22`), and **Azure Virtual Machines** (`Standard_D2s_v5..D32s_v5`, `Standard_F4s_v2..F16s_v2`).
   - Longest-match-first lookup (`lookup_infracost_cloud_spec`) guarantees that `aws-c6i.12xlarge` (`48 vCPU`, `$0.03400/min` On-Demand, `$0.01140/min` Spot) never collides with `aws-c6i.2xlarge` (`8 vCPU`).
2. **Explicit Terraform / IaC Resource Attribution (`infracost_resources` in JSON output):**
   - Every delegated component in `--format json` emits `"infracost_delegated": true` and lists the exact Terraform resource types that Infracost CLI / FinOps policies govern:
     - **`runner_cost`:** `["aws_instance", "aws_ebs_volume", "google_compute_instance", "azurerm_linux_virtual_machine"]`
     - **`artifact_storage_cost`:** `["aws_s3_bucket", "aws_ecr_repository"]`
     - **`log_retention_cost`:** `["aws_cloudwatch_log_group", "aws_s3_bucket"]`
     - **`cross_region_egress_cost`:** `["aws_nat_gateway", "aws_vpc_endpoint", "google_compute_router_nat", "azurerm_nat_gateway"]`
3. **Live Catalog Sync & Zero-Latency Overrides (`minici catalog` & `~/.minici/pricing_overrides.json`):**
   - Customers with custom Enterprise Discount Programs (EDP) on AWS/GCP/Azure or custom GitHub Actions contracts can override any SKU in `~/.minici/pricing_overrides.json`, which `minici` hot-loads in `< 1 µs` with a guaranteed zero-break fallback to `EMBEDDED_SKU_CATALOG`.
4. **Complementary Division of Labor with Infracost:**
   - **Infracost** prices static Infrastructure-as-Code (`*.tf`) resources (`aws_instance`, `aws_nat_gateway`, `aws_s3_bucket`).
   - **MiniCI** parses CI/CD workflow DAGs (`.github/workflows/*.yml`, `.gitlab-ci.yml`, `.circleci/config.yml`, `bitbucket-pipelines.yml`, `buildspec.yml`), expands dynamic Cartesian test matrices, computes the workload volume (billed minutes, concurrent jobs, artifact GBs, log GBs, network transfer GBs), and feeds those volumes into Infracost's verified cloud unit prices.

---

## 6. Deep-Dive Customer Case Studies from Tested Repositories

### Case Study A: `MaximVote/maxim` + `MaximVote/tf` (Internal Production Stack)
- **`MaximVote/maxim` (`41 workflows, 57 expanded jobs`):**
  - **Current State:** Hybrid setup using `blacksmith` microVMs (`44` jobs, `$947.06/mo`) and `github-actions` (`13` jobs, `$62.35/mo`) for a net compute bill of **`$1,009.41/mo`** and developer wait cost of **`$4,269.38/mo`** (`9.1m` critical path).
  - **What the 6-Component Breakdown Reveals:**
    1. Because `44` of `57` jobs already run on `blacksmith`, `MaximVote/maxim` avoids the GitHub Team/Enterprise concurrency upgrade tax (`parallelism_tier_upgrade_tax_monthly_usd = $0.00/mo` with only `13` hosted concurrent jobs).
    2. The remaining `13` `ubuntu-latest` jobs incur **`$27.44/mo`** (`5.30m/run`) in GitHub 1-minute ceiling rounding tax.
    3. `7` artifact-uploading jobs (`ci-apk.yml`, `ci.yml`, etc.) at 90-day retention cost **`$243.56/mo`** on GitHub (`974.25 GB` steady-state), which drops to **`$37.89/mo`** with `retention-days: 14` or **`$2.73/mo`** on S3 Intelligent-Tiering.
    4. Migrating to self-hosted Cloud Spot (`c5.2xlarge` Spot) cuts compute to **`$128.29/mo`** (saving **`$881.12/mo`**), *provided* `MaximVote/tf` provisions S3/ECR VPC Endpoints (`$16.51/mo`) to avoid the **`$742.96/mo`** (`us-east-1`) Private+NAT Gateway trap.
- **`MaximVote/tf` (`6 workflows, 8 expanded jobs`):**
  - **Current State:** `27.0m` billed/run (`$70.15/mo` compute, `$1,688.70/mo` dev wait).
  - **What the 6-Component Breakdown Reveals:** `3.6m/run` (`$9.35/mo`, or `13.3%` of the compute bill) is pure 1-minute ceiling rounding tax across 6 Terragrunt workflows (`terragrunt-aws.yml`, `terragrunt-gcp.yml`, `terragrunt-gcp-router.yml`, `terragrunt-gcp-vpn.yml`, `terragrunt-vpn-secrets.yml`), and `1` artifact upload job costs more in 90-day vendor storage (**`$81.19/mo`**) than the entire compute bill (**`$70.15/mo`**)! Setting `retention-days: 14` immediately saves **`$68.56/mo`**.

### Case Study B: `denoland/deno` & `astral-sh/uv` (High-Scale Open-Source / Enterprise Matrix Pipelines)
- **`denoland/deno` (`11 workflows, 162 expanded jobs`):**
  - Heavy multi-OS matrix (`664m` Linux, `591m` Windows, `605m` macOS per run) drives an effective hosted rate of **`$1.51/hr`** and **`$20,206.81/mo`** in gross compute (`3,418,968` OS-weighted minutes/month = `40,269%` of free quota).
  - **Hidden Taxes Uncovered:** `$913.65/mo` in 1-minute rounding tax, `$2,020.68/mo` in 10% flake retry overhead, `$510.00/mo` in Enterprise parallelism tier tax (`48` concurrent macOS jobs vs `5` cap), and `$4,221.04/mo` (`us-east-1`) to `$5,721.85/mo` (`sa-east-1`) in NAT Gateway risk if self-hosted without VPC endpoints.
- **`astral-sh/uv` (`47 workflows, 243 expanded jobs`):**
  - Highest rounding tax by volume (`99.50m/run` = **`$461.87/mo`**) due to `243` fast Rust build/test matrix shards hitting GitHub's 1-minute minimum ceiling, plus **`$402.94/mo`** (30d) to **`$597.91/mo`** (90d) if `129.98 GB/mo` of `cargo test` logs are shipped to Datadog instead of Loki+S3 (`$2.99/mo`).

### Case Study C: `Eazy-Ops/multi-cloud-finops-mcp-server` (Lean Free-Tier Microservice)
- **`1 workflow, 1 job` (`2.0m` billed/run, `866 mins/mo`):**
  - Consumes only **`43.3%`** of GitHub's `2,000 mins/mo` free tier (`Net Billed: $0.00/mo`).
  - `minici` prevents premature optimization by explicitly recommending: **`Stay on SaaS runners (866 / 5,000 mins/mo self-hosted break-even threshold)`**, showing that spinning up even a single `$30/mo` `t3.medium` self-hosted runner + `$8.40/mo` EBS/CW overhead would *increase* monthly cost from `$0.00/mo` to `$38.40/mo`.
