# Locked 27-Repository Golden Baseline: KPIs, 20 Unified CI/CD Cost Models & 3-Mode Latency (`v0.7.0`)

**Locked Version:** `minici 0.7.0` (`tag: v0.7.0`)  
**Total Repositories Locked:** `27` (`10` Primary + `7` Internal Edge-Case + `10` Fresh World-Scale OSS)  
**Execution Modes Verified on Every Repo:** `Mode 1: CLI` • `Mode 2: GitHub Action Workflow` • `Mode 3: MCP JSON-RPC 2.0 + Shift-Left Hook`  
**Machine-Readable Companion Lock:** [`tests/golden_baseline_10_repos.json`](../tests/golden_baseline_10_repos.json)

## How to Re-Verify Zero Deviation in Future Sessions

```bash
# Run the 27-repository parallel verification gate (< 3s):
bash scripts/validate_minici_cost_10_repos.sh

# Or invoke the verifier directly:
python3 tools/verify_10_repos_all_modes.py
```

If any commit alters job expansion, 1-minute ceiling rounding, SKU pricing, or any of the 20 unified CI/CD cost models (`6` invoice components + `5` hidden payroll/pipeline models + `5` team-stage & pipeline-shape models + `4` multi-vendor calculator & stack intelligence models) on any of the 27 repositories, `verify_10_repos_all_modes.py` exits non-zero and prints the exact field delta.

---

## Part 1: 10 Primary Repositories (`3 Internal + 7 Modern OSS`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | Internal (MaximVote • Private) | `41` / `45` / `57` | `195.00m` | **`$1,009.41/mo`** | `4221.8%` | `$4,269.38/mo` (`9.1m`) | `14.14` | `45.01` | `11.54` | `82.76` |
| **`mgjartproject/FineArts-BE`** | Internal (Grassy Fine Arts • Private) | `2` / `6` / `6` | `18.00m` | **`$46.76/mo`** | `389.7%` | `$2,979.04/mo` (`6.35m`) | `1.24` | `33.04` | `6.11` | `59.08` |
| **`Eazy-Ops/Backend`** | Internal (Eazy-Ops • Private) | `5` / `5` / `5` | `28.00m` | **`$72.74/mo`** | `606.2%` | `$8,374.22/mo` (`17.85m`) | `1.54` | `34.12` | `6.51` | `59.39` |
| **`astral-sh/uv`** | Fresh OSS (Rust/Python Packaging) | `47` / `223` / `243` | `746.25m` | **`$3,464.00/mo`** | `16156.3%` | `$6,122.62/mo` (`13.05m`) | `27.52` | `58.63` | `28.90` | `132.86` |
| **`denoland/deno`** | Fresh OSS (JS/TS Runtime) | `11` / `59` / `162` | `1860.00m` | **`$20,206.81/mo`** | `40269.0%` | `$13,015.98/mo` (`27.75m`) | `13.27` | `42.59` | `16.65` | `95.36` |
| **`ggml-org/llama.cpp`** | Fresh OSS (C++ LLM Inference) | `62` / `121` / `152` | `459.85m` | **`$1,930.75/mo`** | `9955.8%` | `$6,378.09/mo` (`13.6m`) | `20.74` | `50.29` | `17.56` | `104.39` |
| **`immich-app/immich`** | Fresh OSS (Photo/Video Cloud) | `27` / `76` / `85` | `269.00m` | **`$819.67/mo`** | `5823.9%` | `$4,879.91/mo` (`10.4m`) | `9.65` | `37.84` | `12.68` | `78.66` |
| **`langgenius/dify`** | Fresh OSS (Agentic AI Platform) | `41` / `84` / `117` | `393.50m` | **`$1,153.51/mo`** | `8519.3%` | `$4,853.93/mo` (`10.35m`) | `15.59` | `40.63` | `15.63` | `99.47` |
| **`open-webui/open-webui`** | Fresh OSS (AI Web Interface) | `7` / `13` / `18` | `87.00m` | **`$226.03/mo`** | `1883.6%` | `$4,455.57/mo` (`9.5m`) | `3.15` | `7.68` | `6.34` | `55.83` |
| **`mendableai/firecrawl`** | Fresh OSS (AI Web Scraper) | `36` / `71` / `86` | `626.85m` | **`$1,978.81/mo`** | `13571.3%` | `$39,615.17/mo` (`84.45m`) | `10.35` | `15.13` | `12.60` | `81.02` |

### Part 1B: 10 Primary Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$27.44` | `c5.2xlarge (8 vCPU / 16 GB)` (`$128.29`) | `$243.56` -> `$2.73` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$63.76`) | `$742.96` -> `$16.51` |
| **`mgjartproject/FineArts-BE`** | `$11.69` | `t3.medium (2 vCPU / 4 GB)` (`$36.97`) | `$8.12` -> `$0.09` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$10.07`) | `$63.13` -> `$1.40` |
| **`Eazy-Ops/Backend`** | `$6.37` | `t3.medium (2 vCPU / 4 GB)` (`$43.25`) | `$8.12` -> `$0.09` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$5.61`) | `$52.61` -> `$1.17` |
| **`astral-sh/uv`** | `$461.87` | `c5.4xlarge (16 vCPU / 32 GB)` (`$628.93`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$597.91`) | `$3,600.24` -> `$80.01` |
| **`denoland/deno`** | `$913.65` | `c5.4xlarge (16 vCPU / 32 GB)` (`$6,783.54`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$398.59`) | `$4,221.04` -> `$93.80` |
| **`ggml-org/llama.cpp`** | `$258.64` | `c5.4xlarge (16 vCPU / 32 GB)` (`$498.59`) | `$243.56` -> `$2.73` | `$510.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$1,530.05`) | `$2,438.16` -> `$54.18` |
| **`immich-app/immich`** | `$104.82` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$198.52`) | `$243.56` -> `$2.73` | `$510.00` | iOS / xcodebuild (2.0-8.0 MB/build) (`$855.60`) | `$1,058.04` -> `$23.51` |
| **`langgenius/dify`** | `$40.01` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$278.02`) | `$243.56` -> `$2.73` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$196.28`) | `$1,926.68` -> `$42.82` |
| **`open-webui/open-webui`** | `$12.60` | `t3.medium (2 vCPU / 4 GB)` (`$72.61`) | `$162.38` -> `$1.82` | `$0.00` | Python / pytest (0.3-1.2 MB/build) (`$30.18`) | `$230.31` -> `$5.12` |
| **`mendableai/firecrawl`** | `$64.08` | `m5.xlarge (4 vCPU / 16 GB)` (`$345.09`) | `$243.56` -> `$2.73` | `$100.00` | Rust / cargo test (0.6-2.0 MB/build) (`$211.60`) | `$1,089.02` -> `$24.20` |

### Part 1C: 10 Primary Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `$1,707.42/mo` | `$487.21/mo` | `42/57` (`$26.69/mo`) | `$1,461.63/mo` | `20` wf (`$12.16/mo`) | `13` wf (`$55.70/mo`) |
| **`mgjartproject/FineArts-BE`** | `$1,191.58/mo` | `$190.31/mo` | `2/6` (`$3.64/mo`) | `$570.93/mo` | `2` wf (`$7.01/mo`) | `2` wf (`$18.71/mo`) |
| **`Eazy-Ops/Backend`** | `$3,349.32/mo` | `$191.68/mo` | `1/5` (`$3.51/mo`) | `$575.04/mo` | `5` wf (`$10.91/mo`) | `5` wf (`$29.10/mo`) |
| **`astral-sh/uv`** | `$2,448.68/mo` | `$759.94/mo` | `129/243` (`$566.23/mo`) | `$2,279.82/mo` | `4` wf (`$3.51/mo`) | `4` wf (`$9.35/mo`) |
| **`denoland/deno`** | `$5,206.76/mo` | `$2,620.25/mo` | `148/162` (`$123.84/mo`) | `$7,860.75/mo` | `1` wf (`$0.39/mo`) | `2` wf (`$7,726.63/mo`) |
| **`ggml-org/llama.cpp`** | `$2,551.90/mo` | `$589.58/mo` | `94/152` (`$177.46/mo`) | `$1,768.74/mo` | `8` wf (`$77.03/mo`) | `5` wf (`$4.16/mo`) |
| **`immich-app/immich`** | `$1,951.30/mo` | `$466.12/mo` | `13/85` (`$142.72/mo`) | `$1,398.36/mo` | `10` wf (`$11.30/mo`) | `17` wf (`$183.59/mo`) |
| **`langgenius/dify`** | `$1,941.94/mo` | `$503.22/mo` | `62/117` (`$43.40/mo`) | `$1,509.66/mo` | `9` wf (`$6.08/mo`) | `6` wf (`$64.05/mo`) |
| **`open-webui/open-webui`** | `$1,782.56/mo` | `$400.16/mo` | `6/18` (`$34.81/mo`) | `$1,200.48/mo` | `2` wf (`$2.34/mo`) | `4` wf (`$78.98/mo`) |
| **`mendableai/firecrawl`** | `$15,845.70/mo` | `$594.92/mo` | `67/86` (`$36.63/mo`) | `$1,784.76/mo` | `30` wf (`$62.72/mo`) | `2` wf (`$593.87/mo`) |

### Part 1D: 10 Primary Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `scaleup (25-100 devs)` (`53` devs • `0.00m`) | `$1,045.27/mo` (`6.33%`) | `2` (`2.75x` • `$205.82/mo`) | `monorepo` (`9` • `turborepo` • `$0.00/mo`) | `37.6%` (`$227.48/mo` • `0`) |
| **`mgjartproject/FineArts-BE`** | `startup-seed-series-a (6-15 devs)` (`5` devs • `0.00m`) | `$539.08/mo` (`3.27%`) | `3` (`1.00x` • `$11.83/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$12.62/mo` • `0`) |
| **`Eazy-Ops/Backend`** | `startup-seed-series-a (6-15 devs)` (`8` devs • `0.00m`) | `$1,201.66/mo` (`7.28%`) | `2` (`1.00x` • `$6.59/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `39.1%` (`$17.08/mo` • `1`) |
| **`astral-sh/uv`** | `enterprise (250+ devs)` (`202` devs • `0.83m`) | `$1,997.64/mo` (`12.11%`) | `3` (`1.00x` • `$701.89/mo`) | `monorepo` (`5` • `path-filters-only` • `$1,212.40/mo`) | `45.0%` (`$935.28/mo` • `0`) |
| **`denoland/deno`** | `enterprise (250+ devs)` (`504` devs • `0.05m`) | `$11,771.12/mo` (`71.34%`) | `2` (`1.00x` • `$467.91/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `11.4%` (`$1,382.15/mo` • `0`) |
| **`ggml-org/llama.cpp`** | `scaleup (25-100 devs)` (`125` devs • `1.54m`) | `$1,554.84/mo` (`9.42%`) | `1` (`1.00x` • `$1,796.15/mo`) | `multi-service-microservices` (`2` • `path-filters-only` • `$0.00/mo`) | `45.0%` (`$521.30/mo` • `0`) |
| **`immich-app/immich`** | `scaleup (25-100 devs)` (`73` devs • `6.04m`) | `$1,342.51/mo` (`8.14%`) | `3` (`1.00x` • `$1,004.40/mo`) | `monorepo` (`20` • `path-filters-only` • `$286.88/mo`) | `45.0%` (`$221.31/mo` • `2`) |
| **`langgenius/dify`** | `scaleup (25-100 devs)` (`107` devs • `5.00m`) | `$1,149.01/mo` (`6.96%`) | `1` (`1.00x` • `$230.42/mo`) | `monorepo` (`67` • `path-filters-only` • `$403.73/mo`) | `38.5%` (`$266.46/mo` • `0`) |
| **`open-webui/open-webui`** | `scaleup (25-100 devs)` (`24` devs • `0.00m`) | `$1,011.04/mo` (`6.13%`) | `3` (`1.00x` • `$35.42/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `43.8%` (`$59.43/mo` • `1`) |
| **`mendableai/firecrawl`** | `scaleup (25-100 devs)` (`170` devs • `0.00m`) | `$5,982.36/mo` (`36.26%`) | `2` (`1.00x` • `$248.40/mo`) | `monorepo` (`16` • `path-filters-only` • `$692.58/mo`) | `17.7%` (`$210.39/mo` • `0`) |

### Part 1E: 10 Primary Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | `gcp-cloud-build` (`$245.80/mo` • 2x:`False`) | `dependency_install` (`$373.99/mo`) | `io-bound` (`0` • `$10.39/mo` • `standard-2-core`) | `node-typescript-vitest` (`$302.82/mo`) |
| **`mgjartproject/FineArts-BE`** | `gcp-cloud-build` (`$15.88/mo` • 2x:`False`) | `dependency_install` (`$27.17/mo`) | `balanced-web-backend` (`0` • `$7.79/mo` • `larger-16-core`) | `python-uv-pytest` (`$11.69/mo`) |
| **`Eazy-Ops/Backend`** | `gcp-cloud-build` (`$28.87/mo` • 2x:`False`) | `dependency_install` (`$36.37/mo`) | `balanced-web-backend` (`0` • `$12.12/mo` • `larger-16-core`) | `node-typescript-vitest` (`$21.82/mo`) |
| **`astral-sh/uv`** | `azure-pipelines` (`$1,720.00/mo` • 2x:`False`) | `dependency_install` (`$1,417.55/mo`) | `compile-heavy` (`18` • `$310.91/mo` • `larger-4-core`) | `rust-cargo` (`$519.60/mo`) |
| **`denoland/deno`** | `azure-pipelines` (`$4,280.00/mo` • 2x:`True`) | `dependency_install` (`$10,962.19/mo`) | `compile-heavy` (`42` • `$751.53/mo` • `standard-2-core`) | `rust-cargo` (`$9,093.06/mo`) |
| **`ggml-org/llama.cpp`** | `azure-pipelines` (`$1,040.00/mo` • 2x:`False`) | `dependency_install` (`$756.57/mo`) | `compile-heavy` (`25` • `$248.72/mo` • `larger-4-core`) | `cpp-cmake-ninja` (`$347.54/mo`) |
| **`immich-app/immich`** | `gcp-cloud-build` (`$484.82/mo` • 2x:`False`) | `dependency_install` (`$392.60/mo`) | `compile-heavy` (`2` • `$119.42/mo` • `larger-8-core`) | `flutter-dart-mobile` (`$286.88/mo`) |
| **`langgenius/dify`** | `gcp-cloud-build` (`$675.12/mo` • 2x:`False`) | `dependency_install` (`$556.12/mo`) | `io-bound` (`0` • `$55.42/mo` • `standard-2-core`) | `python-uv-pytest` (`$288.38/mo`) |
| **`open-webui/open-webui`** | `gcp-cloud-build` (`$105.51/mo` • 2x:`False`) | `dependency_install` (`$109.06/mo`) | `test-heavy` (`0` • `$37.67/mo` • `larger-16-core`) | `python-uv-pytest` (`$56.51/mo`) |
| **`mendableai/firecrawl`** | `gcp-cloud-build` (`$806.78/mo` • 2x:`False`) | `dependency_install` (`$938.49/mo`) | `compile-heavy` (`0` • `$37.24/mo` • `larger-16-core`) | `rust-cargo` (`$890.46/mo`) |

---

## Part 2: 10 Fresh World-Scale GitHub Repositories (`/Users/araza/personal/top10-github-repos`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | World-Scale OSS (VS Code IDE) | `5` / `26` / `36` | `127.00m` | **`$2,103.51/mo`** | `2749.6%` | `$3,680.50/mo` (`7.85m`) | `2.81` | `11.50` | `8.19` | `42.00` |
| **`flutter/flutter`** | World-Scale OSS (Flutter SDK) | `6` / `11` / `11` | `20.00m` | **`$76.21/mo`** | `433.0%` | `$1,337.97/mo` (`2.85m`) | `2.54` | `7.74` | `7.16` | `40.05` |
| **`ollama/ollama`** | World-Scale OSS (Local LLM Runner) | `4` / `18` / `47` | `149.00m` | **`$1,266.09/mo`** | `3225.9%` | `$3,563.59/mo` (`7.6m`) | `2.11` | `7.86` | `8.61` | `36.30` |
| **`huggingface/transformers`** | World-Scale OSS (ML Transformers) | `7` / `37` / `57` | `217.00m` | **`$563.77/mo`** | `4698.1%` | `$3,563.59/mo` (`7.6m`) | `1.89` | `8.81` | `8.24` | `38.20` |
| **`vercel/next.js`** | World-Scale OSS (Next.js Framework) | `5` / `38` / `45` | `132.00m` | **`$919.69/mo`** | `2857.8%` | `$2,511.40/mo` (`5.35m`) | `2.25` | `8.66` | `5.49` | `36.25` |
| **`langchain-ai/langchain`** | World-Scale OSS (Agentic Framework) | `27` / `41` / `43` | `105.00m` | **`$272.79/mo`** | `2273.3%` | `$3,095.95/mo` (`6.6m`) | `15.00` | `22.66` | `12.33` | `44.44` |
| **`n8n-io/n8n`** | World-Scale OSS (Workflow Automation) | `12` / `38` / `44` | `198.85m` | **`$636.25/mo`** | `4305.1%` | `$4,853.93/mo` (`10.35m`) | `2.98` | `9.10` | `6.47` | `33.81` |
| **`supabase/supabase`** | World-Scale OSS (Postgres Cloud) | `12` / `17` / `25` | `68.85m` | **`$268.46/mo`** | `1490.6%` | `$2,554.70/mo` (`5.45m`) | `2.51` | `7.49` | `5.76` | `31.55` |
| **`home-assistant/core`** | World-Scale OSS (Home Automation) | `19` / `66` / `87` | `561.00m` | **`$1,403.79/mo`** | `12145.7%` | `$22,139.29/mo` (`47.2m`) | `17.14` | `23.99` | `16.41` | `57.55` |
| **`pydantic/pydantic`** | World-Scale OSS (Python Validation) | `12` / `66` / `175` | `930.70m` | **`$4,759.88/mo`** | `20149.7%` | `$5,958.08/mo` (`12.7m`) | `5.41` | `13.40` | `7.80` | `38.21` |

### Part 2B: 10 Fresh World-Scale Repositories — 6 Core CI/CD Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Self-Hosted EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`External 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$134.99` | `c5.4xlarge (16 vCPU / 32 GB)` (`$421.43`) | `$8.12` -> `$0.09` | `$510.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$40.25`) | `$378.79` -> `$8.42` |
| **`flutter/flutter`** | `$13.91` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$46.06`) | `$8.12` -> `$0.09` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$12.28`) | `$115.74` -> `$2.57` |
| **`ollama/ollama`** | `$93.47` | `c5.4xlarge (16 vCPU / 32 GB)` (`$415.80`) | `$243.56` -> `$2.73` | `$510.00` | Go / go test (0.1-0.4 MB/build) (`$26.27`) | `$760.50` -> `$16.90` |
| **`huggingface/transformers`** | `$62.87` | `t3.medium (2 vCPU / 4 GB)` (`$129.89`) | `$81.19` -> `$0.91` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$95.63`) | `$620.21` -> `$13.78` |
| **`vercel/next.js`** | `$95.80` | `c5.4xlarge (16 vCPU / 32 GB)` (`$271.35`) | `$8.12` -> `$0.09` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$50.32`) | `$473.49` -> `$10.52` |
| **`langchain-ai/langchain`** | `$51.57` | `t3.medium (2 vCPU / 4 GB)` (`$74.04`) | `$81.19` -> `$0.91` | `$100.00` | Python / pytest (0.3-1.2 MB/build) (`$72.13`) | `$472.90` -> `$10.51` |
| **`n8n-io/n8n`** | `$23.04` | `c5.2xlarge (8 vCPU / 16 GB)` (`$128.64`) | `$8.12` -> `$0.09` | `$100.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$49.22`) | `$462.96` -> `$10.29` |
| **`supabase/supabase`** | `$6.43` | `c5.2xlarge (8 vCPU / 16 GB)` (`$64.81`) | `$162.38` -> `$1.82` | `$0.00` | Node / npm / Vitest (0.2-0.8 MB/build) (`$27.97`) | `$303.97` -> `$6.75` |
| **`home-assistant/core`** | `$113.73` | `t3.medium (2 vCPU / 4 GB)` (`$291.49`) | `$243.56` -> `$2.73` | `$510.00` | Python / pytest (0.3-1.2 MB/build) (`$145.96`) | `$1,713.32` -> `$38.07` |
| **`pydantic/pydantic`** | `$377.44` | `m6g.xlarge (Arm 4 vCPU / 16 GB)` (`$1,369.18`) | `$243.56` -> `$2.73` | `$510.00` | Rust / cargo test (0.6-2.0 MB/build) (`$430.61`) | `$3,130.27` -> `$69.56` |

### Part 2C: 10 Fresh World-Scale Repositories — 5 Hidden Payroll & Pipeline Hygiene Cost Models (`$/mo`)

| Repository | H1: 40% Context-Switch Payroll Loss | H2: Rerun Flake Cost (`12%`) | H3: Cache Coverage (`Unrealized Savings`) | H4: Combined Flake Tax (`+ Triage`) | H5: Missing `cancel-in-progress` Waste | H5: Missing Path-Filter Savings |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `$1,472.90/mo` | `$608.77/mo` | `2/36` (`$469.78/mo`) | `$1,826.31/mo` | `5` wf (`$315.53/mo`) | `4` wf (`$82.44/mo`) |
| **`flutter/flutter`** | `$534.82/mo` | `$191.86/mo` | `0/11` (`$5.26/mo`) | `$575.58/mo` | `6` wf (`$11.43/mo`) | `4` wf (`$9.35/mo`) |
| **`ollama/ollama`** | `$1,426.10/mo` | `$254.49/mo` | `14/47` (`$43.26/mo`) | `$763.47/mo` | `4` wf (`$189.91/mo`) | `3` wf (`$178.74/mo`) |
| **`huggingface/transformers`** | `$1,426.10/mo` | `$217.52/mo` | `1/57` (`$97.17/mo`) | `$652.56/mo` | `7` wf (`$84.56/mo`) | `7` wf (`$225.51/mo`) |
| **`vercel/next.js`** | `$1,003.86/mo` | `$236.25/mo` | `9/45` (`$97.04/mo`) | `$708.75/mo` | `5` wf (`$137.95/mo`) | `4` wf (`$154.84/mo`) |
| **`langchain-ai/langchain`** | `$1,238.38/mo` | `$405.36/mo` | `23/43` (`$8.44/mo`) | `$1,216.08/mo` | `6` wf (`$3.51/mo`) | `7` wf (`$28.06/mo`) |
| **`n8n-io/n8n`** | `$1,941.94/mo` | `$445.74/mo` | `0/44` (`$131.47/mo`) | `$1,337.22/mo` | `12` wf (`$95.44/mo`) | `11` wf (`$242.03/mo`) |
| **`supabase/supabase`** | `$1,022.58/mo` | `$404.88/mo` | `0/25` (`$36.71/mo`) | `$1,214.64/mo` | `12` wf (`$40.27/mo`) | `11` wf (`$85.56/mo`) |
| **`home-assistant/core`** | `$8,856.38/mo` | `$531.03/mo` | `49/87` (`$31.18/mo`) | `$1,593.09/mo` | `0` wf (`$0.00/mo`) | `0` wf (`$0.00/mo`) |
| **`pydantic/pydantic`** | `$2,382.90/mo` | `$438.37/mo` | `159/175` (`$84.30/mo`) | `$1,315.11/mo` | `3` wf (`$545.24/mo`) | `5` wf (`$1,828.09/mo`) |

### Part 2D: 10 Fresh World-Scale Repositories — 5 Dynamic Team-Stage & Pipeline-Shape Cost Models

| Repository | S1: Inferred Team Stage (`Devs` • `Mac mini Payback`) | S2: Recoverable Waste (`Platform Eng ROI %`) | S3: Compliance Controls (`Residency` • `1yr SIEM`) | S4: Pipeline Shape (`Units` • `Tool` • `Affected Savings`) | S5: Setup Share (`Shared Base Savings` • `Unpinned`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `scaleup (25-100 devs)` (`35` devs • `0.99m`) | `$1,857.51/mo` (`11.26%`) | `0` (`1.00x` • `$47.25/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$567.95/mo` • `0`) |
| **`flutter/flutter`** | `startup-seed-series-a (6-15 devs)` (`6` devs • `36.53m`) | `$339.98/mo` (`2.06%`) | `0` (`1.00x` • `$14.42/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$20.57/mo` • `0`) |
| **`ollama/ollama`** | `scaleup (25-100 devs)` (`41` devs • `0.89m`) | `$1,043.33/mo` (`6.32%`) | `1` (`1.00x` • `$30.83/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$341.84/mo` • `0`) |
| **`huggingface/transformers`** | `scaleup (25-100 devs)` (`59` devs • `0.00m`) | `$1,009.09/mo` (`6.12%`) | `0` (`1.00x` • `$112.27/mo`) | `single-service` (`0` • `none` • `$0.00/mo`) | `45.0%` (`$152.22/mo` • `0`) |
| **`vercel/next.js`** | `scaleup (25-100 devs)` (`36` devs • `0.00m`) | `$879.99/mo` (`5.33%`) | `0` (`1.00x` • `$59.08/mo`) | `monorepo` (`5` • `turborepo` • `$0.00/mo`) | `45.0%` (`$248.32/mo` • `0`) |
| **`langchain-ai/langchain`** | `scaleup (25-100 devs)` (`29` devs • `0.00m`) | `$776.34/mo` (`4.71%`) | `2` (`1.00x` • `$84.67/mo`) | `monorepo` (`5` • `custom-affected-graph` • `$0.00/mo`) | `45.0%` (`$73.66/mo` • `0`) |
| **`n8n-io/n8n`** | `scaleup (25-100 devs)` (`54` devs • `0.00m`) | `$1,452.69/mo` (`8.80%`) | `0` (`1.00x` • `$57.78/mo`) | `monorepo` (`4` • `custom-affected-graph` • `$0.00/mo`) | `45.0%` (`$171.79/mo` • `0`) |
| **`supabase/supabase`** | `scaleup (25-100 devs)` (`19` devs • `0.00m`) | `$833.70/mo` (`5.05%`) | `0` (`1.00x` • `$32.83/mo`) | `monorepo` (`5` • `path-filters-only` • `$93.96/mo`) | `45.0%` (`$72.49/mo` • `0`) |
| **`home-assistant/core`** | `scaleup (25-100 devs)` (`152` devs • `0.00m`) | `$3,166.02/mo` (`19.19%`) | `5` (`1.00x` • `$171.34/mo`) | `multi-service-microservices` (`0` • `none` • `$0.00/mo`) | `21.1%` (`$177.64/mo` • `5`) |
| **`pydantic/pydantic`** | `enterprise (250+ devs)` (`252` devs • `0.32m`) | `$3,523.20/mo` (`21.35%`) | `2` (`1.00x` • `$505.49/mo`) | `single-service` (`1` • `none` • `$0.00/mo`) | `25.5%` (`$729.12/mo` • `0`) |

### Part 2E: 10 Fresh World-Scale Repositories — 4 Calculator & Stack Intelligence Pillars (`P17–P20`)

| Repository | P17: Cheapest Vendor TCO (`2x Migration Rule`) | P18: Best First Phase (`Reducible $/mo`) | P19: Compute Profile (`Larger` • `ARM64 Savings` • `Best Tier`) | P20: Stack Archetype (`Stack Savings $/mo`) |
| :--- | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | `azure-pipelines` (`$280.00/mo` • 2x:`True`) | `dependency_install` (`$1,201.10/mo`) | `io-bound` (`34` • `$754.25/mo` • `standard-2-core`) | `node-typescript-vitest` (`$631.05/mo`) |
| **`flutter/flutter`** | `azure-pipelines` (`$40.00/mo` • 2x:`False`) | `dependency_install` (`$42.68/mo`) | `compile-heavy` (`0` • `$8.23/mo` • `larger-16-core`) | `flutter-dart-mobile` (`$26.67/mo`) |
| **`ollama/ollama`** | `azure-pipelines` (`$320.00/mo` • 2x:`True`) | `dependency_install` (`$576.07/mo`) | `compile-heavy` (`3` • `$81.39/mo` • `larger-4-core`) | `go-mod-build-cache` (`$379.83/mo`) |
| **`huggingface/transformers`** | `gcp-cloud-build` (`$274.38/mo` • 2x:`False`) | `dependency_install` (`$315.71/mo`) | `compile-heavy` (`0` • `$93.96/mo` • `larger-8-core`) | `python-uv-pytest` (`$140.94/mo`) |
| **`vercel/next.js`** | `azure-pipelines` (`$280.00/mo` • 2x:`False`) | `dependency_install` (`$418.46/mo`) | `compile-heavy` (`15` • `$249.36/mo` • `larger-4-core`) | `node-typescript-vitest` (`$275.91/mo`) |
| **`langchain-ai/langchain`** | `gcp-cloud-build` (`$128.90/mo` • 2x:`False`) | `dependency_install` (`$153.51/mo`) | `test-heavy` (`0` • `$45.47/mo` • `larger-8-core`) | `python-uv-pytest` (`$68.20/mo`) |
| **`n8n-io/n8n`** | `gcp-cloud-build` (`$250.81/mo` • 2x:`False`) | `dependency_install` (`$358.85/mo`) | `io-bound` (`0` • `$19.05/mo` • `standard-2-core`) | `node-typescript-vitest` (`$190.88/mo`) |
| **`supabase/supabase`** | `gcp-cloud-build` (`$81.94/mo` • 2x:`False`) | `dependency_install` (`$155.98/mo`) | `io-bound` (`0` • `$4.76/mo` • `larger-4-core`) | `node-typescript-vitest` (`$80.54/mo`) |
| **`home-assistant/core`** | `gcp-cloud-build` (`$721.24/mo` • 2x:`False`) | `dependency_install` (`$701.17/mo`) | `io-bound` (`0` • `$242.91/mo` • `larger-4-core`) | `python-uv-pytest` (`$350.95/mo`) |
| **`pydantic/pydantic`** | `azure-pipelines` (`$2,120.00/mo` • 2x:`True`) | `dependency_install` (`$2,165.75/mo`) | `compile-heavy` (`0` • `$313.06/mo` • `standard-2-core`) | `rust-cargo` (`$713.98/mo`) |

---

## Part 3: 7 Internal Edge-Case Repositories — KPIs & Latency

| Repository | Edge-Case Scenario | WF / Exp Jobs | Gross Monthly Compute | Net Billed Monthly | CLI Latency (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/tf`** | Internal (6 Terragrunt VPN/GCP workflows) | `6` / `8` | `$70.15/mo` | **`$70.15/mo`** | `7.99` |
| **`mgjartproject/gfa-data-pipelines`** | Internal (17x matrix scraper pipeline) | `2` / `23` | `$127.30/mo` | **`$127.30/mo`** | `6.52` |
| **`mgjartproject/frontend`** | Internal (GFA Frontend WIF + Security) | `2` / `6` | `$44.17/mo` | **`$44.17/mo`** | `5.81` |
| **`mgjartproject/FineArt_AI_Pipeline`** | Internal Edge Case (0 workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `4.47` |
| **`Eazy-Ops/frontend`** | Internal (Eazy-Ops Frontend Docker) | `2` / `2` | `$20.78/mo` | **`$20.78/mo`** | `4.73` |
| **`Eazy-Ops/multi-cloud-finops-mcp-server`** | Internal (Within 2,000m Free Tier -> $0 net) | `1` / `1` | `$5.20/mo` | **`$0.00/mo`** | `5.82` |
| **`Eazy-Ops/Automation`** | Internal Edge Case (Nested cron/.github/workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `5.32` |
