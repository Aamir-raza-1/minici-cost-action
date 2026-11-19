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
| **`MaximVote/maxim`** | Internal (MaximVote • Private) | `41` / `45` / `57` | `382.25m` | **`$2,282.43/mo`** | `519.6%` | `$11,842.55/mo` (`25.25m`) | `14.28` | `22.34` | `12.78` | `93.94` |
| **`mgjartproject/FineArts-BE`** | Internal (Grassy Fine Arts • Private) | `2` / `6` / `6` | `18.00m` | **`$46.76/mo`** | `389.7%` | `$2,979.04/mo` (`6.35m`) | `1.56` | `10.43` | `6.69` | `64.98` |
| **`Eazy-Ops/Backend`** | Internal (Eazy-Ops • Private) | `5` / `5` / `5` | `32.00m` | **`$83.14/mo`** | `692.8%` | `$9,898.38/mo` (`21.1m`) | `1.71` | `7.86` | `4.87` | `65.80` |
| **`astral-sh/uv`** | Fresh OSS (Rust/Python Packaging) | `47` / `224` / `243` | `764.00m` | **`$3,652.79/mo`** | `19441.7%` | `$11,725.64/mo` (`25.0m`) | `29.84` | `37.36` | `29.53` | `139.32` |
| **`denoland/deno`** | Fresh OSS (JS/TS Runtime) | `11` / `59` / `162` | `1870.00m` | **`$20,232.79/mo`** | `171099.9%` | `$21,368.55/mo` (`45.55m`) | `15.58` | `22.30` | `20.98` | `99.31` |
| **`ggml-org/llama.cpp`** | Fresh OSS (C++ LLM Inference) | `62` / `121` / `152` | `1005.85m` | **`$5,046.36/mo`** | `37346.3%` | `$12,266.89/mo` (`26.15m`) | `20.21` | `25.82` | `23.29` | `112.42` |
| **`immich-app/immich`** | Fresh OSS (Photo/Video Cloud) | `27` / `76` / `85` | `316.00m` | **`$1,205.04/mo`** | `10067.3%` | `$7,599.15/mo` (`16.2m`) | `10.86` | `16.23` | `12.06` | `77.79` |
| **`langgenius/dify`** | Fresh OSS (Agentic AI Platform) | `41` / `84` / `117` | `460.00m` | **`$1,327.58/mo`** | `5066.1%` | `$7,975.86/mo` (`17.0m`) | `15.85` | `20.57` | `18.40` | `109.72` |
| **`open-webui/open-webui`** | Fresh OSS (AI Web Interface) | `7` / `13` / `18` | `123.00m` | **`$319.55/mo`** | `2663.0%` | `$15,219.95/mo` (`32.45m`) | `3.05` | `7.35` | `7.19` | `63.26` |
| **`mendableai/firecrawl`** | Fresh OSS (AI Web Scraper) | `36` / `71` / `86` | `672.35m` | **`$2,121.70/mo`** | `1926.9%` | `$41,256.24/mo` (`87.95m`) | `11.72` | `17.26` | `18.30` | `85.24` |

### Part 1B: 10 Primary Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$31.65` | `c5.2xlarge (8 vCPU / 16 GB)` (`$705.44`) | `$243.56` -> `$2.73` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$63.76`) | `$346.72` -> `$16.51` |
| **`mgjartproject/FineArts-BE`** | `$11.69` | `t3.medium (2 vCPU / 4 GB)` (`$36.97`) | `$0.00` -> `$0.00` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$10.07`) | `$29.46` -> `$1.40` |
| **`Eazy-Ops/Backend`** | `$8.31` | `t3.medium (2 vCPU / 4 GB)` (`$44.94`) | `$0.00` -> `$0.00` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$5.61`) | `$24.55` -> `$1.17` |
| **`astral-sh/uv`** | `$474.53` | `c5.4xlarge (16 vCPU / 32 GB)` (`$1,139.43`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$597.91`) | `$1,680.11` -> `$80.01` |
| **`denoland/deno`** | `$915.34` | `c5.4xlarge (16 vCPU / 32 GB)` (`$6,788.48`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$398.59`) | `$1,969.82` -> `$93.80` |
| **`ggml-org/llama.cpp`** | `$314.06` | `c5.4xlarge (16 vCPU / 32 GB)` (`$1,436.56`) | `$243.56` -> `$2.73` | `$510.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$1,530.05`) | `$1,137.81` -> `$54.18` |
| **`immich-app/immich`** | `$132.13` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$316.30`) | `$243.56` -> `$2.73` | `$510.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$855.60`) | `$493.75` -> `$23.51` |
| **`langgenius/dify`** | `$53.10` | `m5.xlarge (4 vCPU / 16 GB)` (`$377.44`) | `$243.56` -> `$2.73` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$196.28`) | `$899.12` -> `$42.82` |
| **`open-webui/open-webui`** | `$21.04` | `t3.medium (2 vCPU / 4 GB)` (`$89.62`) | `$162.38` -> `$1.82` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$30.18`) | `$107.48` -> `$5.12` |
| **`mendableai/firecrawl`** | `$62.48` | `m5.xlarge (4 vCPU / 16 GB)` (`$632.95`) | `$243.56` -> `$2.73` | `$100.00` | Rust / cargo test (0.6-2.0 MB/build) (`$211.60`) | `$508.21` -> `$24.20` |

### Part 1C: 10 Primary Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$4,737.72/mo` | `$628.65/mo` | `40/57` (`$28.33/mo`) | `$1,885.95/mo` | `20` wf (`$15.54/mo`) | `13` wf (`$73.71/mo`) |
| **`mgjartproject/FineArts-BE`** | `$1,191.58/mo` | `$190.31/mo` | `0/6` (`$5.20/mo`) | `$570.93/mo` | `2` wf (`$7.01/mo`) | `2` wf (`$18.71/mo`) |
| **`Eazy-Ops/Backend`** | `$3,959.02/mo` | `$192.23/mo` | `1/5` (`$3.51/mo`) | `$576.69/mo` | `5` wf (`$12.47/mo`) | `5` wf (`$33.25/mo`) |
| **`astral-sh/uv`** | `$4,690.92/mo` | `$780.92/mo` | `81/243` (`$799.13/mo`) | `$2,342.76/mo` | `4` wf (`$3.51/mo`) | `4` wf (`$9.35/mo`) |
| **`denoland/deno`** | `$8,546.72/mo` | `$2,623.14/mo` | `130/162` (`$485.35/mo`) | `$7,869.42/mo` | `1` wf (`$0.39/mo`) | `2` wf (`$7,737.02/mo`) |
| **`ggml-org/llama.cpp`** | `$4,906.72/mo` | `$935.76/mo` | `88/152` (`$1,018.17/mo`) | `$2,807.28/mo` | `8` wf (`$200.05/mo`) | `5` wf (`$4.16/mo`) |
| **`immich-app/immich`** | `$3,039.66/mo` | `$508.94/mo` | `13/85` (`$148.95/mo`) | `$1,526.82/mo` | `10` wf (`$11.30/mo`) | `17` wf (`$317.30/mo`) |
| **`langgenius/dify`** | `$3,189.68/mo` | `$522.56/mo` | `59/117` (`$76.58/mo`) | `$1,567.68/mo` | `9` wf (`$6.99/mo`) | `6` wf (`$86.91/mo`) |
| **`open-webui/open-webui`** | `$6,088.68/mo` | `$410.56/mo` | `1/18` (`$55.08/mo`) | `$1,231.68/mo` | `2` wf (`$3.51/mo`) | `4` wf (`$113.27/mo`) |
| **`mendableai/firecrawl`** | `$16,502.46/mo` | `$610.79/mo` | `52/86` (`$117.98/mo`) | `$1,832.37/mo` | `30` wf (`$75.06/mo`) | `2` wf (`$618.12/mo`) |

### Part 1D: 10 Primary Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `scaleup (25-100 devs)` (`104` devs • `0.00m`) | `$2,091.48/mo` (`12.68%`) | `2` (`2.75x` • `$17.53/mo`) | `monorepo` (`9` • `turborepo` • `$0.00/mo`) | `18.9%` (`$258.83/mo` • `0`) |
| **`mgjartproject/FineArts-BE`** | `startup-seed-series-a (6-15 devs)` (`5` devs • `0.00m`) | `$534.21/mo` (`3.24%`) | `3` (`1.00x` • `$1.01/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$12.62/mo` • `0`) |
| **`Eazy-Ops/Backend`** | `startup-seed-series-a (6-15 devs)` (`9` devs • `0.00m`) | `$1,382.22/mo` (`8.38%`) | `2` (`1.00x` • `$0.56/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `34.7%` (`$17.32/mo` • `1`) |
| **`astral-sh/uv`** | `enterprise (250+ devs)` (`207` devs • `0.80m`) | `$2,838.36/mo` (`17.20%`) | `3` (`1.00x` • `$59.79/mo`) | `monorepo` (`5` • `path-filters-only` • `$1,278.48/mo`) | `45.0%` (`$986.26/mo` • `0`) |
| **`denoland/deno`** | `enterprise (250+ devs)` (`507` devs • `0.05m`) | `$12,158.15/mo` (`73.69%`) | `2` (`1.00x` • `$39.86/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `11.3%` (`$1,376.64/mo` • `0`) |
| **`ggml-org/llama.cpp`** | `enterprise (250+ devs)` (`273` devs • `0.39m`) | `$3,377.14/mo` (`20.47%`) | `1` (`1.00x` • `$153.01/mo`) | `multi-service-microservices` (`2` • `path-filters-only` • `$0.00/mo`) | `20.1%` (`$609.80/mo` • `0`) |
| **`immich-app/immich`** | `scaleup (25-100 devs)` (`86` devs • `1.83m`) | `$1,760.84/mo` (`10.67%`) | `3` (`1.00x` • `$85.56/mo`) | `monorepo` (`20` • `path-filters-only` • `$421.76/mo`) | `45.0%` (`$325.36/mo` • `2`) |
| **`langgenius/dify`** | `scaleup (25-100 devs)` (`125` devs • `5.00m`) | `$1,575.38/mo` (`9.55%`) | `1` (`1.00x` • `$19.63/mo`) | `monorepo` (`67` • `path-filters-only` • `$464.65/mo`) | `33.1%` (`$263.81/mo` • `0`) |
| **`open-webui/open-webui`** | `scaleup (25-100 devs)` (`34` devs • `0.00m`) | `$2,331.74/mo` (`14.13%`) | `3` (`1.00x` • `$3.02/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `31.3%` (`$60.07/mo` • `1`) |
| **`mendableai/firecrawl`** | `scaleup (25-100 devs)` (`182` devs • `0.00m`) | `$6,177.22/mo` (`37.44%`) | `2` (`1.00x` • `$21.16/mo`) | `monorepo` (`16` • `path-filters-only` • `$742.59/mo`) | `16.5%` (`$209.66/mo` • `0`) |

### Part 1E: 10 Primary Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `aws-codebuild` (`$827.07/mo` • 2x:`False`) | `clone_checkout` (`$622.47/mo`) | `io-bound` (`0` • `$10.39/mo` • `larger-16-core`) | `node-typescript-vitest` (`$684.73/mo`) |
| **`mgjartproject/FineArts-BE`** | `buildkite-pro-byo` (`$30.00/mo` • 2x:`False`) | `dependency_install` (`$27.17/mo`) | `balanced-web-backend` (`0` • `$7.79/mo` • `larger-16-core`) | `python-uv-pytest` (`$11.69/mo`) |
| **`Eazy-Ops/Backend`** | `azure-pipelines` (`$40.00/mo` • 2x:`False`) | `dependency_install` (`$37.00/mo`) | `balanced-web-backend` (`0` • `$13.86/mo` • `larger-16-core`) | `node-typescript-vitest` (`$24.94/mo`) |
| **`astral-sh/uv`** | `azure-pipelines` (`$1,760.00/mo` • 2x:`False`) | `dependency_install` (`$1,323.51/mo`) | `compile-heavy` (`10` • `$267.63/mo` • `larger-8-core`) | `rust-cargo` (`$547.92/mo`) |
| **`denoland/deno`** | `azure-pipelines` (`$4,280.00/mo` • 2x:`True`) | `dependency_install` (`$8,909.65/mo`) | `compile-heavy` (`1` • `$183.31/mo` • `larger-8-core`) | `rust-cargo` (`$9,104.76/mo`) |
| **`ggml-org/llama.cpp`** | `azure-pipelines` (`$2,320.00/mo` • 2x:`False`) | `dependency_install` (`$1,631.25/mo`) | `compile-heavy` (`12` • `$359.63/mo` • `larger-8-core`) | `cpp-cmake-ninja` (`$908.34/mo`) |
| **`immich-app/immich`** | `azure-pipelines` (`$720.00/mo` • 2x:`False`) | `dependency_install` (`$540.25/mo`) | `compile-heavy` (`0` • `$115.18/mo` • `larger-8-core`) | `flutter-dart-mobile` (`$421.76/mo`) |
| **`langgenius/dify`** | `azure-pipelines` (`$1,040.00/mo` • 2x:`False`) | `dependency_install` (`$533.68/mo`) | `io-bound` (`0` • `$70.15/mo` • `larger-4-core`) | `python-uv-pytest` (`$331.90/mo`) |
| **`open-webui/open-webui`** | `aws-codebuild` (`$265.80/mo` • 2x:`False`) | `dependency_install` (`$151.25/mo`) | `test-heavy` (`0` • `$53.26/mo` • `larger-16-core`) | `python-uv-pytest` (`$79.89/mo`) |
| **`mendableai/firecrawl`** | `aws-codebuild` (`$1,455.14/mo` • 2x:`False`) | `dependency_install` (`$856.63/mo`) | `compile-heavy` (`0` • `$38.54/mo` • `larger-32-core`) | `rust-cargo` (`$954.77/mo`) |

### Part 1F: 10 Primary Repositories — 4 FinOps Unit Economics, Anomaly/Timeout, Registry & Fleet SLA Models (`M21–M24`)

| Repository | M21: Cost / Merged PR (`$/Dev/Mo`) | M22: Projected EOM (`Missing Timeouts`) | M23: Container Images (`Registry $/mo`) | M24: Est. P95 Queue (`Idle Warm Pool $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$21.08` (`$21.95/dev`) | `$2,282.43/mo` (`28` jobs) | `10` (`$68.10/mo`) | `8.0s` (`$0.00/mo`) |
| **`mgjartproject/FineArts-BE`** | `$0.43` (`$9.35/dev`) | `$46.76/mo` (`6` jobs) | `1` (`$6.72/mo`) | `8.0s` (`$0.00/mo`) |
| **`Eazy-Ops/Backend`** | `$0.77` (`$9.24/dev`) | `$83.14/mo` (`5` jobs) | `6` (`$40.82/mo`) | `8.0s` (`$0.00/mo`) |
| **`astral-sh/uv`** | `$33.74` (`$17.65/dev`) | `$3,652.79/mo` (`83` jobs) | `7` (`$47.64/mo`) | `1183.0s` (`$0.00/mo`) |
| **`denoland/deno`** | `$186.91` (`$39.91/dev`) | `$20,232.79/mo` (`10` jobs) | `0` (`$0.00/mo`) | `877.0s` (`$0.00/mo`) |
| **`ggml-org/llama.cpp`** | `$46.62` (`$18.48/dev`) | `$5,046.36/mo` (`118` jobs) | `9` (`$61.28/mo`) | `709.0s` (`$2,410.80/mo`) |
| **`immich-app/immich`** | `$11.13` (`$14.01/dev`) | `$1,205.04/mo` (`76` jobs) | `10` (`$68.10/mo`) | `415.0s` (`$0.00/mo`) |
| **`langgenius/dify`** | `$12.26` (`$10.62/dev`) | `$1,327.58/mo` (`45` jobs) | `9` (`$61.28/mo`) | `133.0s` (`$0.00/mo`) |
| **`open-webui/open-webui`** | `$2.95` (`$9.40/dev`) | `$319.55/mo` (`8` jobs) | `11` (`$74.92/mo`) | `8.0s` (`$0.00/mo`) |
| **`mendableai/firecrawl`** | `$19.60` (`$11.66/dev`) | `$2,121.70/mo` (`47` jobs) | `6` (`$40.82/mo`) | `169.0s` (`$0.00/mo`) |

---

## Part 2: 10 Fresh World-Scale GitHub Repositories (`/Users/araza/personal/top10-github-repos`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | World-Scale OSS (VS Code IDE) | `5` / `26` / `36` | `242.00m` | **`$4,316.14/mo`** | `129.9%` | `$5,208.99/mo` (`11.1m`) | `2.71` | `8.92` | `6.28` | `44.51` |
| **`flutter/flutter`** | World-Scale OSS (Flutter SDK) | `6` / `11` / `11` | `27.00m` | **`$94.39/mo`** | `779.4%` | `$4,620.11/mo` (`9.85m`) | `2.21` | `9.16` | `7.02` | `37.93` |
| **`ollama/ollama`** | World-Scale OSS (Local LLM Runner) | `4` / `18` / `47` | `413.00m` | **`$2,606.66/mo`** | `18554.1%` | `$7,949.88/mo` (`16.95m`) | `2.22` | `7.29` | `6.39` | `37.41` |
| **`huggingface/transformers`** | World-Scale OSS (ML Transformers) | `7` / `37` / `57` | `223.00m` | **`$579.35/mo`** | `4828.0%` | `$3,563.59/mo` (`7.6m`) | `2.55` | `7.60` | `6.98` | `39.24` |
| **`vercel/next.js`** | World-Scale OSS (Next.js Framework) | `5` / `38` / `45` | `144.00m` | **`$1,137.92/mo`** | `2663.0%` | `$2,979.04/mo` (`6.35m`) | `2.00` | `8.74` | `6.19` | `38.13` |
| **`langchain-ai/langchain`** | World-Scale OSS (Agentic Framework) | `27` / `41` / `43` | `113.00m` | **`$293.57/mo`** | `2446.5%` | `$7,668.43/mo` (`16.35m`) | `14.38` | `20.25` | `11.43` | `46.09` |
| **`n8n-io/n8n`** | World-Scale OSS (Workflow Automation) | `12` / `38` / `44` | `258.35m` | **`$866.61/mo`** | `952.6%` | `$6,495.00/mo` (`13.85m`) | `3.85` | `9.16` | `6.75` | `34.52` |
| **`supabase/supabase`** | World-Scale OSS (Postgres Cloud) | `12` / `17` / `25` | `73.35m` | **`$284.05/mo`** | `238.2%` | `$3,611.22/mo` (`7.7m`) | `2.69` | `7.61` | `5.81` | `33.76` |
| **`home-assistant/core`** | World-Scale OSS (Home Automation) | `19` / `66` / `87` | `570.00m` | **`$1,427.17/mo`** | `12340.5%` | `$43,624.75/mo` (`93.0m`) | `18.71` | `27.93` | `18.22` | `60.04` |
| **`pydantic/pydantic`** | World-Scale OSS (Python Validation) | `12` / `67` / `175` | `933.70m` | **`$5,395.87/mo`** | `44620.7%` | `$10,249.11/mo` (`21.85m`) | `6.09` | `12.17` | `8.55` | `39.79` |

### Part 2B: 10 Fresh World-Scale Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$346.00` | `c5.4xlarge (16 vCPU / 32 GB)` (`$928.09`) | `$0.00` -> `$0.00` | `$510.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$40.25`) | `$176.77` -> `$8.42` |
| **`flutter/flutter`** | `$12.76` | `m5.xlarge (4 vCPU / 16 GB)` (`$49.70`) | `$0.00` -> `$0.00` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$12.28`) | `$54.01` -> `$2.57` |
| **`ollama/ollama`** | `$100.98` | `c5.4xlarge (16 vCPU / 32 GB)` (`$771.34`) | `$243.56` -> `$2.73` | `$510.00` | Go / go test (0.1-0.4 MB/build) (`$26.27`) | `$354.90` -> `$16.90` |
| **`huggingface/transformers`** | `$63.52` | `t3.medium (2 vCPU / 4 GB)` (`$132.88`) | `$81.19` -> `$0.91` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$95.63`) | `$289.43` -> `$13.78` |
| **`vercel/next.js`** | `$120.51` | `c5.4xlarge (16 vCPU / 32 GB)` (`$292.44`) | `$0.00` -> `$0.00` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$50.32`) | `$220.96` -> `$10.52` |
| **`langchain-ai/langchain`** | `$52.87` | `t3.medium (2 vCPU / 4 GB)` (`$77.93`) | `$81.19` -> `$0.91` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$72.13`) | `$220.69` -> `$10.51` |
| **`n8n-io/n8n`** | `$24.15` | `c5.2xlarge (8 vCPU / 16 GB)` (`$275.47`) | `$0.00` -> `$0.00` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$49.22`) | `$216.05` -> `$10.29` |
| **`supabase/supabase`** | `$6.39` | `c5.2xlarge (8 vCPU / 16 GB)` (`$111.32`) | `$162.38` -> `$1.82` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$27.97`) | `$141.85` -> `$6.75` |
| **`home-assistant/core`** | `$113.80` | `t3.medium (2 vCPU / 4 GB)` (`$296.17`) | `$243.56` -> `$2.73` | `$510.00` | Python / pytest (0.3-1.2 MB/build) (`$145.96`) | `$799.55` -> `$38.07` |
| **`pydantic/pydantic`** | `$423.60` | `m5.xlarge (4 vCPU / 16 GB)` (`$1,602.48`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$430.61`) | `$1,460.79` -> `$69.56` |

### Part 2C: 10 Fresh World-Scale Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$2,082.86/mo` | `$854.62/mo` | `0/36` (`$971.13/mo`) | `$2,563.86/mo` | `5` wf (`$647.42/mo`) | `4` wf (`$120.55/mo`) |
| **`flutter/flutter`** | `$1,848.08/mo` | `$192.82/mo` | `0/11` (`$9.35/mo`) | `$578.46/mo` | `6` wf (`$14.16/mo`) | `4` wf (`$16.63/mo`) |
| **`ollama/ollama`** | `$3,180.32/mo` | `$325.04/mo` | `5/47` (`$380.87/mo`) | `$975.12/mo` | `4` wf (`$391.00/mo`) | `3` wf (`$608.28/mo`) |
| **`huggingface/transformers`** | `$1,426.10/mo` | `$218.34/mo` | `0/57` (`$101.84/mo`) | `$655.02/mo` | `7` wf (`$86.90/mo`) | `7` wf (`$231.74/mo`) |
| **`vercel/next.js`** | `$1,191.58/mo` | `$247.74/mo` | `1/45` (`$155.49/mo`) | `$743.22/mo` | `5` wf (`$170.69/mo`) | `4` wf (`$242.13/mo`) |
| **`langchain-ai/langchain`** | `$3,067.74/mo` | `$407.67/mo` | `17/43` (`$21.11/mo`) | `$1,223.01/mo` | `6` wf (`$3.51/mo`) | `7` wf (`$28.06/mo`) |
| **`n8n-io/n8n`** | `$2,598.70/mo` | `$471.34/mo` | `0/44` (`$183.30/mo`) | `$1,414.02/mo` | `12` wf (`$129.99/mo`) | `11` wf (`$334.17/mo`) |
| **`supabase/supabase`** | `$1,444.82/mo` | `$406.61/mo` | `0/25` (`$39.05/mo`) | `$1,219.83/mo` | `12` wf (`$42.61/mo`) | `11` wf (`$91.80/mo`) |
| **`home-assistant/core`** | `$17,449.90/mo` | `$533.62/mo` | `37/87` (`$49.88/mo`) | `$1,600.86/mo` | `0` wf (`$0.00/mo`) | `0` wf (`$0.00/mo`) |
| **`pydantic/pydantic`** | `$4,099.68/mo` | `$471.84/mo` | `98/175` (`$480.11/mo`) | `$1,415.52/mo` | `3` wf (`$639.47/mo`) | `5` wf (`$2,082.49/mo`) |

### Part 2D: 10 Fresh World-Scale Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `scaleup (25-100 devs)` (`66` devs • `0.42m`) | `$2,791.08/mo` (`16.92%`) | `0` (`1.00x` • `$4.02/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `32.4%` (`$837.76/mo` • `0`) |
| **`flutter/flutter`** | `startup-seed-series-a (6-15 devs)` (`8` devs • `36.53m`) | `$742.05/mo` (`4.50%`) | `0` (`1.00x` • `$1.23/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$25.49/mo` • `0`) |
| **`ollama/ollama`** | `scaleup (25-100 devs)` (`112` devs • `0.52m`) | `$2,321.68/mo` (`14.07%`) | `1` (`1.00x` • `$2.63/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `23.7%` (`$370.36/mo` • `0`) |
| **`huggingface/transformers`** | `scaleup (25-100 devs)` (`61` devs • `0.00m`) | `$924.18/mo` (`5.60%`) | `0` (`1.00x` • `$9.56/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$156.43/mo` • `0`) |
| **`vercel/next.js`** | `scaleup (25-100 devs)` (`39` devs • `0.00m`) | `$1,015.98/mo` (`6.16%`) | `0` (`1.00x` • `$5.03/mo`) | `monorepo` (`5` • `turborepo` • `$0.00/mo`) | `45.0%` (`$307.24/mo` • `0`) |
| **`langchain-ai/langchain`** | `scaleup (25-100 devs)` (`31` devs • `0.00m`) | `$1,332.31/mo` (`8.07%`) | `2` (`1.00x` • `$7.21/mo`) | `monorepo` (`5` • `custom-affected-graph` • `$0.00/mo`) | `45.0%` (`$79.27/mo` • `0`) |
| **`n8n-io/n8n`** | `scaleup (25-100 devs)` (`70` devs • `0.00m`) | `$1,662.60/mo` (`10.08%`) | `0` (`1.00x` • `$4.92/mo`) | `monorepo` (`4` • `custom-affected-graph` • `$0.00/mo`) | `35.0%` (`$182.20/mo` • `0`) |
| **`supabase/supabase`** | `scaleup (25-100 devs)` (`20` devs • `0.00m`) | `$928.19/mo` (`5.63%`) | `0` (`1.00x` • `$2.80/mo`) | `monorepo` (`5` • `path-filters-only` • `$99.42/mo`) | `45.0%` (`$76.69/mo` • `0`) |
| **`home-assistant/core`** | `scaleup (25-100 devs)` (`155` devs • `0.00m`) | `$5,760.12/mo` (`34.91%`) | `5` (`1.00x` • `$14.60/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `20.7%` (`$177.51/mo` • `5`) |
| **`pydantic/pydantic`** | `enterprise (250+ devs)` (`253` devs • `0.25m`) | `$4,227.26/mo` (`25.62%`) | `2` (`1.00x` • `$43.06/mo`) | `single-service` (`1` • `none` • `$0.00/mo`) | `25.4%` (`$822.98/mo` • `0`) |

### Part 2E: 10 Fresh World-Scale Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `azure-pipelines` (`$520.00/mo` • 2x:`True`) | `dependency_install` (`$2,464.52/mo`) | `io-bound` (`34` • `$431.27/mo` • `standard-2-core`) | `node-typescript-vitest` (`$1,294.84/mo`) |
| **`flutter/flutter`** | `azure-pipelines` (`$40.00/mo` • 2x:`False`) | `dependency_install` (`$52.86/mo`) | `compile-heavy` (`0` • `$11.26/mo` • `larger-32-core`) | `flutter-dart-mobile` (`$33.04/mo`) |
| **`ollama/ollama`** | `azure-pipelines` (`$920.00/mo` • 2x:`False`) | `dependency_install` (`$1,151.75/mo`) | `compile-heavy` (`1` • `$101.76/mo` • `larger-8-core`) | `go-mod-build-cache` (`$782.00/mo`) |
| **`huggingface/transformers`** | `azure-pipelines` (`$480.00/mo` • 2x:`False`) | `dependency_install` (`$324.44/mo`) | `compile-heavy` (`0` • `$96.56/mo` • `larger-8-core`) | `python-uv-pytest` (`$144.84/mo`) |
| **`vercel/next.js`** | `azure-pipelines` (`$320.00/mo` • 2x:`False`) | `dependency_install` (`$513.31/mo`) | `compile-heavy` (`15` • `$198.85/mo` • `larger-4-core`) | `node-typescript-vitest` (`$341.38/mo`) |
| **`langchain-ai/langchain`** | `azure-pipelines` (`$240.00/mo` • 2x:`False`) | `dependency_install` (`$135.37/mo`) | `test-heavy` (`0` • `$48.93/mo` • `larger-16-core`) | `python-uv-pytest` (`$73.39/mo`) |
| **`n8n-io/n8n`** | `aws-codebuild` (`$558.83/mo` • 2x:`False`) | `dependency_install` (`$488.77/mo`) | `io-bound` (`0` • `$19.05/mo` • `larger-8-core`) | `node-typescript-vitest` (`$259.98/mo`) |
| **`supabase/supabase`** | `aws-codebuild` (`$158.30/mo` • 2x:`False`) | `dependency_install` (`$165.03/mo`) | `io-bound` (`0` • `$4.76/mo` • `larger-16-core`) | `node-typescript-vitest` (`$85.22/mo`) |
| **`home-assistant/core`** | `aws-codebuild` (`$1,233.55/mo` • 2x:`False`) | `dependency_install` (`$617.62/mo`) | `io-bound` (`0` • `$246.81/mo` • `larger-4-core`) | `python-uv-pytest` (`$356.79/mo`) |
| **`pydantic/pydantic`** | `azure-pipelines` (`$2,120.00/mo` • 2x:`True`) | `dependency_install` (`$2,039.64/mo`) | `compile-heavy` (`0` • `$299.20/mo` • `larger-8-core`) | `rust-cargo` (`$809.38/mo`) |

### Part 2F: 10 Fresh World-Scale Repositories — 4 FinOps Unit Economics, Anomaly/Timeout, Registry & Fleet SLA Models (`M21–M24`)

| Repository | M21: Cost / Merged PR (`$/Dev/Mo`) | M22: Projected EOM (`Missing Timeouts`) | M23: Container Images (`Registry $/mo`) | M24: Est. P95 Queue (`Idle Warm Pool $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$39.87` (`$65.40/dev`) | `$4,316.14/mo` (`26` jobs) | `0` (`$0.00/mo`) | `121.0s` (`$0.00/mo`) |
| **`flutter/flutter`** | `$0.87` (`$11.80/dev`) | `$94.39/mo` (`11` jobs) | `0` (`$0.00/mo`) | `8.0s` (`$0.00/mo`) |
| **`ollama/ollama`** | `$24.08` (`$23.27/dev`) | `$2,606.66/mo` (`18` jobs) | `0` (`$0.00/mo`) | `187.0s` (`$0.00/mo`) |
| **`huggingface/transformers`** | `$5.35` (`$9.50/dev`) | `$579.35/mo` (`37` jobs) | `0` (`$0.00/mo`) | `247.0s` (`$0.00/mo`) |
| **`vercel/next.js`** | `$10.51` (`$29.18/dev`) | `$1,137.92/mo` (`38` jobs) | `0` (`$0.00/mo`) | `175.0s` (`$0.00/mo`) |
| **`langchain-ai/langchain`** | `$2.71` (`$9.47/dev`) | `$293.57/mo` (`29` jobs) | `0` (`$0.00/mo`) | `163.0s` (`$0.00/mo`) |
| **`n8n-io/n8n`** | `$8.01` (`$12.38/dev`) | `$866.61/mo` (`38` jobs) | `0` (`$0.00/mo`) | `43.0s` (`$0.00/mo`) |
| **`supabase/supabase`** | `$2.62` (`$14.20/dev`) | `$284.05/mo` (`17` jobs) | `1` (`$6.72/mo`) | `8.0s` (`$0.00/mo`) |
| **`home-assistant/core`** | `$13.18` (`$9.21/dev`) | `$1,427.17/mo` (`31` jobs) | `5` (`$34.00/mo`) | `427.0s` (`$0.00/mo`) |
| **`pydantic/pydantic`** | `$49.85` (`$21.33/dev`) | `$5,395.87/mo` (`64` jobs) | `0` (`$0.00/mo`) | `901.0s` (`$641.42/mo`) |

---

## Part 3: 7 Internal Edge-Case Repositories — KPIs & Latency

| Repository | Edge-Case Scenario | WF / Exp Jobs | Gross Monthly Compute | Net Billed Monthly | CLI Latency (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/tf`** | Internal (6 Terragrunt VPN/GCP workflows) | `6` / `8` | `$77.94/mo` | **`$65.94/mo`** | `7.63` |
| **`mgjartproject/gfa-data-pipelines`** | Internal (17x matrix scraper pipeline) | `2` / `23` | `$127.30/mo` | **`$115.30/mo`** | `6.38` |
| **`mgjartproject/frontend`** | Internal (GFA Frontend WIF + Security) | `2` / `6` | `$44.17/mo` | **`$32.17/mo`** | `5.42` |
| **`mgjartproject/FineArt_AI_Pipeline`** | Internal Edge Case (0 workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `5.11` |
| **`Eazy-Ops/frontend`** | Internal (Eazy-Ops Frontend Docker) | `2` / `2` | `$20.78/mo` | **`$8.78/mo`** | `4.91` |
| **`Eazy-Ops/multi-cloud-finops-mcp-server`** | Internal (Within 2,000m Free Tier -> $0 net) | `1` / `1` | `$5.20/mo` | **`$0.00/mo`** | `4.66` |
| **`Eazy-Ops/Automation`** | Internal Edge Case (Nested cron/.github/workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `5.07` |
