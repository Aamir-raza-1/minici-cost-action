# Locked 27-Repository Golden Baseline: KPIs, 24 Unified CI/CD Cost Models & 3-Mode Latency (`v0.8.0`)

**Locked Version:** `minici 0.8.0` (`tag: v0.8.0`)  
**Total Repositories Locked:** `27` (`10` Primary + `7` Internal Edge-Case + `10` Fresh World-Scale OSS)  
**Execution Modes Verified on Every Repo:** `Mode 1: CLI` • `Mode 2: GitHub Action Workflow` • `Mode 3: MCP JSON-RPC 2.0 (7 Tools) + Shift-Left Hook`  
**Machine-Readable Companion Lock:** [`tests/golden_baseline_10_repos.json`](../tests/golden_baseline_10_repos.json)

## How to Re-Verify Zero Deviation in Future Sessions

```bash
# Run the 27-repository parallel verification gate (< 3s):
bash scripts/validate_minici_cost_10_repos.sh

# Or invoke the verifier directly:
python3 tools/verify_10_repos_all_modes.py
```

If any commit alters job expansion, 1-minute ceiling rounding, SKU pricing, or any of the 24 unified CI/CD cost models (`6` invoice components + `5` hidden payroll/pipeline models + `5` team-stage & pipeline-shape models + `4` multi-vendor calculator & stack intelligence models + `4` unit economics, anomaly/timeout, container registry & fleet SLA models) on any of the 27 repositories, `verify_10_repos_all_modes.py` exits non-zero and prints the exact field delta.

---

## Part 1: 10 Primary Repositories (`3 Internal + 7 Modern OSS`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | Internal (MaximVote • Private) | `41` / `45` / `57` | `395.00m` | **`$2,347.38/mo`** | `519.6%` | `$3,788.75/mo` (`25.25m`) | `16.06` | `24.82` | `15.23` | `82.07` |
| **`mgjartproject/FineArts-BE`** | Internal (Grassy Fine Arts • Private) | `2` / `6` / `6` | `18.00m` | **`$46.76/mo`** | `389.7%` | `$1,190.75/mo` (`6.35m`) | `2.54` | `10.16` | `5.81` | `59.55` |
| **`Eazy-Ops/Backend`** | Internal (Eazy-Ops • Private) | `5` / `5` / `5` | `32.00m` | **`$83.14/mo`** | `692.8%` | `$3,957.62/mo` (`21.1m`) | `3.05` | `10.94` | `6.95` | `55.95` |
| **`astral-sh/uv`** | Fresh OSS (Rust/Python Packaging) | `47` / `227` / `243` | `764.00m` | **`$3,737.66/mo`** | `20177.8%` | `$2,156.34/mo` (`25.0m`) | `29.27` | `36.18` | `31.79` | `135.23` |
| **`denoland/deno`** | Fresh OSS (JS/TS Runtime) | `11` / `59` / `162` | `1870.00m` | **`$13,179.22/mo`** | `110891.3%` | `$8,547.42/mo` (`45.55m`) | `16.26` | `22.08` | `20.64` | `95.64` |
| **`ggml-org/llama.cpp`** | Fresh OSS (C++ LLM Inference) | `62` / `129` / `148` | `982.55m` | **`$5,302.09/mo`** | `39446.3%` | `$3,433.69/mo` (`26.15m`) | `20.72` | `27.09` | `20.96` | `113.55` |
| **`immich-app/immich`** | Fresh OSS (Photo/Video Cloud) | `27` / `79` / `85` | `316.00m` | **`$1,444.06/mo`** | `9807.5%` | `$3,039.66/mo` (`16.2m`) | `10.83` | `15.28` | `11.12` | `75.58` |
| **`langgenius/dify`** | Fresh OSS (Agentic AI Platform) | `41` / `84` / `117` | `460.00m` | **`$1,327.58/mo`** | `5066.1%` | `$3,134.92/mo` (`17.0m`) | `15.44` | `21.02` | `17.39` | `101.36` |
| **`open-webui/open-webui`** | Fresh OSS (AI Web Interface) | `7` / `13` / `18` | `123.00m` | **`$319.55/mo`** | `2663.0%` | `$6,087.98/mo` (`32.45m`) | `2.83` | `7.24` | `6.94` | `57.45` |
| **`mendableai/firecrawl`** | Fresh OSS (AI Web Scraper) | `36` / `75` / `82` | `659.15m` | **`$2,103.69/mo`** | `1234.1%` | `$16,501.63/mo` (`87.95m`) | `11.79` | `16.23` | `14.40` | `78.59` |

### Part 1B: 10 Primary Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$31.50` | `c5.2xlarge (8 vCPU / 16 GB)` (`$724.92`) | `$243.56` -> `$2.73` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`mgjartproject/FineArts-BE`** | `$11.69` | `t3.medium (2 vCPU / 4 GB)` (`$36.97`) | `$0.00` -> `$0.00` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`Eazy-Ops/Backend`** | `$8.31` | `t3.medium (2 vCPU / 4 GB)` (`$44.94`) | `$0.00` -> `$0.00` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`astral-sh/uv`** | `$485.55` | `c5.4xlarge (16 vCPU / 32 GB)` (`$1,169.14`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$0.00`) | `$1,680.11` -> `$80.01` |
| **`denoland/deno`** | `$596.24` | `c5.4xlarge (16 vCPU / 32 GB)` (`$4,087.95`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$0.00`) | `$1,969.82` -> `$93.80` |
| **`ggml-org/llama.cpp`** | `$320.54` | `c5.4xlarge (16 vCPU / 32 GB)` (`$1,537.19`) | `$243.56` -> `$2.73` | `$510.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$0.00`) | `$1,099.07` -> `$52.34` |
| **`immich-app/immich`** | `$158.34` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$321.11`) | `$243.56` -> `$2.73` | `$100.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`langgenius/dify`** | `$53.10` | `m5.xlarge (4 vCPU / 16 GB)` (`$377.44`) | `$243.56` -> `$2.73` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`open-webui/open-webui`** | `$21.04` | `t3.medium (2 vCPU / 4 GB)` (`$89.62`) | `$162.38` -> `$1.82` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`mendableai/firecrawl`** | `$41.49` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$636.98`) | `$243.56` -> `$2.73` | `$100.00` | Rust / cargo test (0.6-2.0 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |

### Part 1C: 10 Primary Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$3,788.75/mo` | `$635.87/mo` | `40/57` (`$28.33/mo`) | `$1,907.61/mo` | `20` wf (`$15.54/mo`) | `13` wf (`$83.07/mo`) |
| **`mgjartproject/FineArts-BE`** | `$1,190.75/mo` | `$190.31/mo` | `0/6` (`$5.20/mo`) | `$570.93/mo` | `2` wf (`$7.01/mo`) | `2` wf (`$18.71/mo`) |
| **`Eazy-Ops/Backend`** | `$3,957.62/mo` | `$192.23/mo` | `1/5` (`$3.51/mo`) | `$576.69/mo` | `5` wf (`$12.47/mo`) | `5` wf (`$33.25/mo`) |
| **`astral-sh/uv`** | `$2,156.34/mo` | `$790.35/mo` | `63/243` (`$1,091.59/mo`) | `$2,371.05/mo` | `4` wf (`$3.51/mo`) | `4` wf (`$9.35/mo`) |
| **`denoland/deno`** | `$8,547.42/mo` | `$1,839.41/mo` | `130/162` (`$485.35/mo`) | `$5,518.23/mo` | `1` wf (`$0.39/mo`) | `2` wf (`$4,915.59/mo`) |
| **`ggml-org/llama.cpp`** | `$3,433.69/mo` | `$964.17/mo` | `77/148` (`$1,044.03/mo`) | `$2,892.51/mo` | `8` wf (`$243.24/mo`) | `5` wf (`$4.16/mo`) |
| **`immich-app/immich`** | `$3,039.66/mo` | `$535.50/mo` | `13/85` (`$148.95/mo`) | `$1,606.50/mo` | `10` wf (`$11.30/mo`) | `17` wf (`$317.30/mo`) |
| **`langgenius/dify`** | `$3,134.92/mo` | `$522.56/mo` | `53/117` (`$96.37/mo`) | `$1,567.68/mo` | `9` wf (`$6.99/mo`) | `6` wf (`$86.91/mo`) |
| **`open-webui/open-webui`** | `$6,087.98/mo` | `$410.56/mo` | `1/18` (`$55.08/mo`) | `$1,231.68/mo` | `2` wf (`$3.51/mo`) | `4` wf (`$113.27/mo`) |
| **`mendableai/firecrawl`** | `$16,501.63/mo` | `$608.79/mo` | `28/82` (`$117.98/mo`) | `$1,826.37/mo` | `30` wf (`$75.63/mo`) | `2` wf (`$618.12/mo`) |

### Part 1D: 10 Primary Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `scaleup (25-100 devs)` (`107` devs • `0.00m`) | `$1,821.59/mo` (`11.04%`) | `2` (`2.75x` • `$17.53/mo`) | `monorepo` (`9` • `turborepo` • `$0.00/mo`) | `18.3%` (`$257.46/mo` • `0`) |
| **`mgjartproject/FineArts-BE`** | `startup-seed-series-a (6-15 devs)` (`5` devs • `0.00m`) | `$533.96/mo` (`3.24%`) | `3` (`1.00x` • `$1.01/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$12.62/mo` • `0`) |
| **`Eazy-Ops/Backend`** | `startup-seed-series-a (6-15 devs)` (`9` devs • `0.00m`) | `$1,381.80/mo` (`8.37%`) | `2` (`1.00x` • `$0.56/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `34.7%` (`$17.32/mo` • `1`) |
| **`astral-sh/uv`** | `enterprise (250+ devs)` (`207` devs • `0.74m`) | `$2,348.84/mo` (`14.24%`) | `3` (`1.00x` • `$59.79/mo`) | `monorepo` (`5` • `path-filters-only` • `$1,308.18/mo`) | `45.0%` (`$1,009.17/mo` • `0`) |
| **`denoland/deno`** | `enterprise (250+ devs)` (`507` devs • `0.09m`) | `$8,917.83/mo` (`54.05%`) | `2` (`1.00x` • `$39.86/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `11.3%` (`$896.71/mo` • `0`) |
| **`ggml-org/llama.cpp`** | `enterprise (250+ devs)` (`266` devs • `0.33m`) | `$3,016.17/mo` (`18.28%`) | `1` (`1.00x` • `$148.98/mo`) | `multi-service-microservices` (`2` • `path-filters-only` • `$0.00/mo`) | `20.0%` (`$637.52/mo` • `0`) |
| **`immich-app/immich`** | `scaleup (25-100 devs)` (`86` devs • `1.83m`) | `$1,791.13/mo` (`10.86%`) | `3` (`1.00x` • `$85.56/mo`) | `monorepo` (`20` • `path-filters-only` • `$505.42/mo`) | `45.0%` (`$389.90/mo` • `2`) |
| **`langgenius/dify`** | `scaleup (25-100 devs)` (`125` devs • `5.00m`) | `$1,575.51/mo` (`9.55%`) | `1` (`1.00x` • `$19.63/mo`) | `monorepo` (`67` • `path-filters-only` • `$464.65/mo`) | `33.1%` (`$263.81/mo` • `0`) |
| **`open-webui/open-webui`** | `scaleup (25-100 devs)` (`34` devs • `0.00m`) | `$2,331.53/mo` (`14.13%`) | `3` (`1.00x` • `$3.02/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `31.3%` (`$60.07/mo` • `1`) |
| **`mendableai/firecrawl`** | `scaleup (25-100 devs)` (`179` devs • `0.00m`) | `$6,175.06/mo` (`37.42%`) | `2` (`1.00x` • `$20.18/mo`) | `monorepo` (`16` • `path-filters-only` • `$736.29/mo`) | `15.9%` (`$200.19/mo` • `0`) |

### Part 1E: 10 Primary Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `azure-pipelines` (`$880.00/mo` • 2x:`False`) | `clone_checkout` (`$625.34/mo`) | `io-bound` (`0` • `$10.39/mo` • `larger-16-core`) | `node-typescript-vitest` (`$704.21/mo`) |
| **`mgjartproject/FineArts-BE`** | `buildkite-pro-byo` (`$30.00/mo` • 2x:`False`) | `dependency_install` (`$24.88/mo`) | `balanced-web-backend` (`0` • `$7.79/mo` • `larger-16-core`) | `python-uv-pytest` (`$11.69/mo`) |
| **`Eazy-Ops/Backend`** | `azure-pipelines` (`$40.00/mo` • 2x:`False`) | `dependency_install` (`$34.14/mo`) | `balanced-web-backend` (`0` • `$13.86/mo` • `larger-16-core`) | `node-typescript-vitest` (`$24.94/mo`) |
| **`astral-sh/uv`** | `azure-pipelines` (`$1,760.00/mo` • 2x:`False`) | `dependency_install` (`$1,355.99/mo`) | `compile-heavy` (`10` • `$263.30/mo` • `larger-4-core`) | `rust-cargo` (`$560.65/mo`) |
| **`denoland/deno`** | `azure-pipelines` (`$4,280.00/mo` • 2x:`True`) | `clone_checkout` (`$5,661.82/mo`) | `compile-heavy` (`1` • `$483.38/mo` • `larger-8-core`) | `rust-cargo` (`$5,930.65/mo`) |
| **`ggml-org/llama.cpp`** | `azure-pipelines` (`$2,240.00/mo` • 2x:`False`) | `dependency_install` (`$1,811.10/mo`) | `compile-heavy` (`12` • `$326.72/mo` • `larger-4-core`) | `cpp-cmake-ninja` (`$954.38/mo`) |
| **`immich-app/immich`** | `azure-pipelines` (`$720.00/mo` • 2x:`False`) | `dependency_install` (`$585.57/mo`) | `compile-heavy` (`4` • `$109.98/mo` • `larger-8-core`) | `flutter-dart-mobile` (`$505.42/mo`) |
| **`langgenius/dify`** | `azure-pipelines` (`$1,040.00/mo` • 2x:`False`) | `dependency_install` (`$526.63/mo`) | `io-bound` (`0` • `$70.15/mo` • `larger-4-core`) | `python-uv-pytest` (`$331.90/mo`) |
| **`open-webui/open-webui`** | `aws-codebuild` (`$265.80/mo` • 2x:`False`) | `dependency_install` (`$149.33/mo`) | `test-heavy` (`0` • `$53.26/mo` • `larger-16-core`) | `python-uv-pytest` (`$79.89/mo`) |
| **`mendableai/firecrawl`** | `azure-pipelines` (`$1,520.00/mo` • 2x:`False`) | `dependency_install` (`$810.15/mo`) | `compile-heavy` (`0` • `$24.68/mo` • `larger-64-core`) | `rust-cargo` (`$946.66/mo`) |

### Part 1F: 10 Primary Repositories — 4 FinOps Unit Economics, Anomaly/Timeout, Registry & Fleet SLA Models (`M21–M24`)

| Repository | M21: Cost / Merged PR (`$/Dev/Mo`) | M22: Projected EOM (`Missing Timeouts`) | M23: Container Images (`Registry $/mo`) | M24: Est. P95 Queue (`Idle Warm Pool $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$2.97` (`$3.01/dev`) | `$2,347.38/mo` (`28` jobs) | `10` (`$68.10/mo`) | `0.0s` (`$0.00/mo`) |
| **`mgjartproject/FineArts-BE`** | `$0.43` (`$9.35/dev`) | `$46.76/mo` (`6` jobs) | `1` (`$6.72/mo`) | `0.0s` (`$0.00/mo`) |
| **`Eazy-Ops/Backend`** | `$0.77` (`$9.24/dev`) | `$83.14/mo` (`5` jobs) | `6` (`$40.82/mo`) | `0.0s` (`$0.00/mo`) |
| **`astral-sh/uv`** | `$3.02` (`$1.58/dev`) | `$3,737.66/mo` (`79` jobs) | `7` (`$47.64/mo`) | `115.0s` (`$0.00/mo`) |
| **`denoland/deno`** | `$113.77` (`$24.29/dev`) | `$13,179.22/mo` (`12` jobs) | `0` (`$0.00/mo`) | `589.0s` (`$0.00/mo`) |
| **`ggml-org/llama.cpp`** | `$34.99` (`$14.24/dev`) | `$5,302.09/mo` (`125` jobs) | `9` (`$61.28/mo`) | `523.0s` (`$2,521.05/mo`) |
| **`immich-app/immich`** | `$12.70` (`$15.98/dev`) | `$1,444.06/mo` (`79` jobs) | `10` (`$68.10/mo`) | `235.0s` (`$0.00/mo`) |
| **`langgenius/dify`** | `$3.85` (`$3.34/dev`) | `$1,327.58/mo` (`63` jobs) | `9` (`$61.28/mo`) | `0.0s` (`$0.00/mo`) |
| **`open-webui/open-webui`** | `$2.90` (`$9.25/dev`) | `$319.55/mo` (`8` jobs) | `11` (`$74.92/mo`) | `0.0s` (`$0.00/mo`) |
| **`mendableai/firecrawl`** | `$19.08` (`$11.54/dev`) | `$2,103.69/mo` (`73` jobs) | `6` (`$40.82/mo`) | `37.0s` (`$0.00/mo`) |

---

## Part 2: 10 Fresh World-Scale GitHub Repositories (`/Users/araza/personal/top10-github-repos`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | World-Scale OSS (VS Code IDE) | `5` / `26` / `36` | `242.00m` | **`$4,316.14/mo`** | `129.9%` | `$2,082.73/mo` (`11.1m`) | `2.84` | `10.75` | `6.97` | `44.28` |
| **`flutter/flutter`** | World-Scale OSS (Flutter SDK) | `6` / `11` / `11` | `27.00m` | **`$94.39/mo`** | `779.4%` | `$1,848.91/mo` (`9.85m`) | `2.16` | `6.90` | `5.95` | `41.32` |
| **`ollama/ollama`** | World-Scale OSS (Local LLM Runner) | `4` / `18` / `47` | `413.00m` | **`$2,606.66/mo`** | `18554.1%` | `$3,178.22/mo` (`16.95m`) | `1.99` | `7.31` | `7.11` | `39.50` |
| **`huggingface/transformers`** | World-Scale OSS (ML Transformers) | `7` / `37` / `57` | `223.00m` | **`$579.35/mo`** | `4828.0%` | `$1,424.57/mo` (`7.6m`) | `1.98` | `6.79` | `6.62` | `45.85` |
| **`vercel/next.js`** | World-Scale OSS (Next.js Framework) | `5` / `38` / `45` | `144.00m` | **`$1,137.92/mo`** | `2663.0%` | `$1,190.75/mo` (`6.35m`) | `1.92` | `9.89` | `6.98` | `43.71` |
| **`langchain-ai/langchain`** | World-Scale OSS (Agentic Framework) | `27` / `41` / `43` | `113.00m` | **`$293.57/mo`** | `2446.5%` | `$1,238.38/mo` (`16.35m`) | `15.15` | `23.59` | `12.88` | `45.92` |
| **`n8n-io/n8n`** | World-Scale OSS (Workflow Automation) | `12` / `38` / `44` | `258.35m` | **`$866.61/mo`** | `952.6%` | `$2,598.00/mo` (`13.85m`) | `2.57` | `10.31` | `6.12` | `32.60` |
| **`supabase/supabase`** | World-Scale OSS (Postgres Cloud) | `12` / `17` / `25` | `73.35m` | **`$284.05/mo`** | `238.2%` | `$1,446.22/mo` (`7.7m`) | `2.87` | `9.91` | `5.26` | `32.50` |
| **`home-assistant/core`** | World-Scale OSS (Home Automation) | `19` / `67` / `83` | `558.00m` | **`$1,394.26/mo`** | `12080.7%` | `$17,449.90/mo` (`93.0m`) | `18.62` | `26.11` | `18.77` | `65.23` |
| **`pydantic/pydantic`** | World-Scale OSS (Python Validation) | `12` / `67` / `175` | `933.70m` | **`$5,395.87/mo`** | `44620.7%` | `$3,217.19/mo` (`21.85m`) | `6.79` | `13.93` | `10.07` | `39.41` |

### Part 2B: 10 Fresh World-Scale Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$346.00` | `c5.4xlarge (16 vCPU / 32 GB)` (`$928.09`) | `$0.00` -> `$0.00` | `$510.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`flutter/flutter`** | `$12.76` | `m5.xlarge (4 vCPU / 16 GB)` (`$49.70`) | `$0.00` -> `$0.00` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`ollama/ollama`** | `$100.98` | `c5.4xlarge (16 vCPU / 32 GB)` (`$771.34`) | `$243.56` -> `$2.73` | `$510.00` | Go / go test (0.1-0.4 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`huggingface/transformers`** | `$63.52` | `t3.medium (2 vCPU / 4 GB)` (`$132.88`) | `$81.19` -> `$0.91` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`vercel/next.js`** | `$120.51` | `c5.4xlarge (16 vCPU / 32 GB)` (`$292.44`) | `$0.00` -> `$0.00` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`langchain-ai/langchain`** | `$52.87` | `t3.medium (2 vCPU / 4 GB)` (`$77.93`) | `$81.19` -> `$0.91` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`n8n-io/n8n`** | `$24.15` | `c5.2xlarge (8 vCPU / 16 GB)` (`$275.47`) | `$0.00` -> `$0.00` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`supabase/supabase`** | `$6.39` | `c5.2xlarge (8 vCPU / 16 GB)` (`$111.32`) | `$162.38` -> `$1.82` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`home-assistant/core`** | `$105.82` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$291.10`) | `$243.56` -> `$2.73` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$0.00`) | `$0.00` -> `$0.00` |
| **`pydantic/pydantic`** | `$423.60` | `m5.xlarge (4 vCPU / 16 GB)` (`$1,602.48`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$0.00`) | `$1,460.79` -> `$69.56` |

### Part 2C: 10 Fresh World-Scale Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$2,082.73/mo` | `$854.62/mo` | `0/36` (`$971.13/mo`) | `$2,563.86/mo` | `5` wf (`$647.42/mo`) | `4` wf (`$120.55/mo`) |
| **`flutter/flutter`** | `$1,848.91/mo` | `$192.82/mo` | `0/11` (`$9.35/mo`) | `$578.46/mo` | `6` wf (`$14.16/mo`) | `4` wf (`$16.63/mo`) |
| **`ollama/ollama`** | `$3,178.22/mo` | `$325.04/mo` | `5/47` (`$380.87/mo`) | `$975.12/mo` | `4` wf (`$391.00/mo`) | `3` wf (`$608.28/mo`) |
| **`huggingface/transformers`** | `$1,424.57/mo` | `$218.34/mo` | `0/57` (`$101.84/mo`) | `$655.02/mo` | `7` wf (`$86.90/mo`) | `7` wf (`$231.74/mo`) |
| **`vercel/next.js`** | `$1,190.75/mo` | `$247.74/mo` | `1/45` (`$155.49/mo`) | `$743.22/mo` | `5` wf (`$170.69/mo`) | `4` wf (`$242.13/mo`) |
| **`langchain-ai/langchain`** | `$1,238.38/mo` | `$407.67/mo` | `7/43` (`$39.69/mo`) | `$1,223.01/mo` | `6` wf (`$3.51/mo`) | `7` wf (`$28.06/mo`) |
| **`n8n-io/n8n`** | `$2,598.00/mo` | `$471.34/mo` | `0/44` (`$183.30/mo`) | `$1,414.02/mo` | `12` wf (`$129.99/mo`) | `11` wf (`$334.17/mo`) |
| **`supabase/supabase`** | `$1,446.22/mo` | `$406.61/mo` | `0/25` (`$39.05/mo`) | `$1,219.83/mo` | `12` wf (`$42.61/mo`) | `11` wf (`$91.80/mo`) |
| **`home-assistant/core`** | `$17,449.90/mo` | `$529.97/mo` | `37/83` (`$43.30/mo`) | `$1,589.91/mo` | `0` wf (`$0.00/mo`) | `0` wf (`$0.00/mo`) |
| **`pydantic/pydantic`** | `$3,217.19/mo` | `$471.84/mo` | `69/175` (`$703.02/mo`) | `$1,415.52/mo` | `3` wf (`$639.47/mo`) | `5` wf (`$2,082.49/mo`) |

### Part 2D: 10 Fresh World-Scale Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `scaleup (25-100 devs)` (`66` devs • `0.42m`) | `$2,791.04/mo` (`16.92%`) | `0` (`1.00x` • `$0.00/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `32.4%` (`$837.76/mo` • `0`) |
| **`flutter/flutter`** | `startup-seed-series-a (6-15 devs)` (`8` devs • `36.53m`) | `$742.30/mo` (`4.50%`) | `0` (`1.00x` • `$0.00/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$25.49/mo` • `0`) |
| **`ollama/ollama`** | `scaleup (25-100 devs)` (`112` devs • `0.52m`) | `$2,321.05/mo` (`14.07%`) | `1` (`1.00x` • `$2.63/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `23.7%` (`$370.36/mo` • `0`) |
| **`huggingface/transformers`** | `scaleup (25-100 devs)` (`61` devs • `0.00m`) | `$923.72/mo` (`5.60%`) | `0` (`1.00x` • `$0.00/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$156.43/mo` • `0`) |
| **`vercel/next.js`** | `scaleup (25-100 devs)` (`39` devs • `0.00m`) | `$1,015.73/mo` (`6.16%`) | `0` (`1.00x` • `$0.00/mo`) | `monorepo` (`5` • `turborepo` • `$0.00/mo`) | `45.0%` (`$307.24/mo` • `0`) |
| **`langchain-ai/langchain`** | `scaleup (25-100 devs)` (`31` devs • `0.00m`) | `$798.45/mo` (`4.84%`) | `2` (`1.00x` • `$7.21/mo`) | `monorepo` (`5` • `custom-affected-graph` • `$0.00/mo`) | `45.0%` (`$79.27/mo` • `0`) |
| **`n8n-io/n8n`** | `scaleup (25-100 devs)` (`70` devs • `0.00m`) | `$1,662.39/mo` (`10.08%`) | `0` (`1.00x` • `$0.00/mo`) | `monorepo` (`4` • `custom-affected-graph` • `$0.00/mo`) | `35.0%` (`$182.20/mo` • `0`) |
| **`supabase/supabase`** | `scaleup (25-100 devs)` (`20` devs • `0.00m`) | `$928.61/mo` (`5.63%`) | `0` (`1.00x` • `$0.00/mo`) | `monorepo` (`5` • `path-filters-only` • `$99.42/mo`) | `45.0%` (`$76.69/mo` • `0`) |
| **`home-assistant/core`** | `scaleup (25-100 devs)` (`152` devs • `0.00m`) | `$5,750.91/mo` (`34.85%`) | `5` (`1.00x` • `$13.92/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `20.1%` (`$168.32/mo` • `5`) |
| **`pydantic/pydantic`** | `enterprise (250+ devs)` (`253` devs • `0.25m`) | `$4,078.09/mo` (`24.72%`) | `2` (`1.00x` • `$43.06/mo`) | `single-service` (`1` • `none` • `$0.00/mo`) | `40.7%` (`$1,317.02/mo` • `0`) |

### Part 2E: 10 Fresh World-Scale Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `azure-pipelines` (`$520.00/mo` • 2x:`False`) | `dependency_install` (`$2,464.52/mo`) | `io-bound` (`34` • `$431.27/mo` • `standard-2-core`) | `node-typescript-vitest` (`$1,294.84/mo`) |
| **`flutter/flutter`** | `azure-pipelines` (`$40.00/mo` • 2x:`False`) | `dependency_install` (`$46.47/mo`) | `compile-heavy` (`0` • `$11.26/mo` • `larger-32-core`) | `flutter-dart-mobile` (`$33.04/mo`) |
| **`ollama/ollama`** | `azure-pipelines` (`$920.00/mo` • 2x:`False`) | `dependency_install` (`$1,177.86/mo`) | `compile-heavy` (`1` • `$101.76/mo` • `larger-8-core`) | `go-mod-build-cache` (`$782.00/mo`) |
| **`huggingface/transformers`** | `azure-pipelines` (`$480.00/mo` • 2x:`False`) | `dependency_install` (`$315.95/mo`) | `compile-heavy` (`0` • `$96.56/mo` • `larger-8-core`) | `python-uv-pytest` (`$144.84/mo`) |
| **`vercel/next.js`** | `azure-pipelines` (`$320.00/mo` • 2x:`False`) | `dependency_install` (`$505.49/mo`) | `compile-heavy` (`15` • `$198.85/mo` • `larger-4-core`) | `node-typescript-vitest` (`$341.38/mo`) |
| **`langchain-ai/langchain`** | `azure-pipelines` (`$240.00/mo` • 2x:`False`) | `dependency_install` (`$141.63/mo`) | `test-heavy` (`0` • `$48.93/mo` • `larger-8-core`) | `python-uv-pytest` (`$73.39/mo`) |
| **`n8n-io/n8n`** | `azure-pipelines` (`$560.00/mo` • 2x:`False`) | `dependency_install` (`$481.62/mo`) | `io-bound` (`0` • `$19.05/mo` • `larger-8-core`) | `node-typescript-vitest` (`$259.98/mo`) |
| **`supabase/supabase`** | `azure-pipelines` (`$160.00/mo` • 2x:`False`) | `dependency_install` (`$162.42/mo`) | `io-bound` (`0` • `$4.76/mo` • `larger-16-core`) | `node-typescript-vitest` (`$85.22/mo`) |
| **`home-assistant/core`** | `aws-codebuild` (`$1,207.57/mo` • 2x:`False`) | `dependency_install` (`$566.82/mo`) | `io-bound` (`0` • `$239.88/mo` • `larger-4-core`) | `python-uv-pytest` (`$348.57/mo`) |
| **`pydantic/pydantic`** | `azure-pipelines` (`$2,120.00/mo` • 2x:`False`) | `dependency_install` (`$1,956.89/mo`) | `compile-heavy` (`0` • `$299.20/mo` • `larger-4-core`) | `rust-cargo` (`$809.38/mo`) |

### Part 2F: 10 Fresh World-Scale Repositories — 4 FinOps Unit Economics, Anomaly/Timeout, Registry & Fleet SLA Models (`M21–M24`)

| Repository | M21: Cost / Merged PR (`$/Dev/Mo`) | M22: Projected EOM (`Missing Timeouts`) | M23: Container Images (`Registry $/mo`) | M24: Est. P95 Queue (`Idle Warm Pool $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$39.87` (`$65.40/dev`) | `$4,316.14/mo` (`26` jobs) | `0` (`$0.00/mo`) | `121.0s` (`$0.00/mo`) |
| **`flutter/flutter`** | `$0.87` (`$11.80/dev`) | `$94.39/mo` (`11` jobs) | `0` (`$0.00/mo`) | `0.0s` (`$0.00/mo`) |
| **`ollama/ollama`** | `$24.08` (`$23.27/dev`) | `$2,606.66/mo` (`18` jobs) | `0` (`$0.00/mo`) | `187.0s` (`$0.00/mo`) |
| **`huggingface/transformers`** | `$5.35` (`$9.50/dev`) | `$579.35/mo` (`37` jobs) | `0` (`$0.00/mo`) | `247.0s` (`$0.00/mo`) |
| **`vercel/next.js`** | `$10.51` (`$29.18/dev`) | `$1,137.92/mo` (`38` jobs) | `0` (`$0.00/mo`) | `175.0s` (`$0.00/mo`) |
| **`langchain-ai/langchain`** | `$0.83` (`$2.88/dev`) | `$293.57/mo` (`30` jobs) | `0` (`$0.00/mo`) | `0.0s` (`$0.00/mo`) |
| **`n8n-io/n8n`** | `$8.01` (`$12.38/dev`) | `$866.61/mo` (`38` jobs) | `0` (`$0.00/mo`) | `43.0s` (`$0.00/mo`) |
| **`supabase/supabase`** | `$2.62` (`$14.20/dev`) | `$284.05/mo` (`17` jobs) | `1` (`$6.72/mo`) | `0.0s` (`$0.00/mo`) |
| **`home-assistant/core`** | `$6.89` (`$4.91/dev`) | `$1,394.26/mo` (`49` jobs) | `5` (`$34.00/mo`) | `0.0s` (`$0.00/mo`) |
| **`pydantic/pydantic`** | `$48.13` (`$20.59/dev`) | `$5,395.87/mo` (`64` jobs) | `0` (`$0.00/mo`) | `709.0s` (`$641.42/mo`) |

---

## Part 3: 7 Internal Edge-Case Repositories — KPIs & Latency

| Repository | Edge-Case Scenario | WF / Exp Jobs | Gross Monthly Compute | Net Billed Monthly | CLI Latency (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/tf`** | Internal (6 Terragrunt VPN/GCP workflows) | `6` / `8` | `$77.94/mo` | **`$65.94/mo`** | `8.12` |
| **`mgjartproject/gfa-data-pipelines`** | Internal (17x matrix scraper pipeline) | `2` / `23` | `$127.30/mo` | **`$115.30/mo`** | `6.97` |
| **`mgjartproject/frontend`** | Internal (GFA Frontend WIF + Security) | `2` / `6` | `$44.17/mo` | **`$32.17/mo`** | `5.26` |
| **`mgjartproject/FineArt_AI_Pipeline`** | Internal Edge Case (0 workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `5.67` |
| **`Eazy-Ops/frontend`** | Internal (Eazy-Ops Frontend Docker) | `2` / `2` | `$20.78/mo` | **`$8.78/mo`** | `5.80` |
| **`Eazy-Ops/multi-cloud-finops-mcp-server`** | Internal (Within 2,000m Free Tier -> $0 net) | `1` / `1` | `$5.20/mo` | **`$0.00/mo`** | `4.95` |
| **`Eazy-Ops/Automation`** | Internal Edge Case (Nested cron/.github/workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `5.33` |
