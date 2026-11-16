# Locked 27-Repository Golden Baseline: KPIs, 6 `cicdcost.com` Components & 3-Mode Latency (`v0.3.0`)

**Locked Version:** `minici 0.3.0` (`tag: v0.3.0`)  
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

If any commit alters job expansion, 1-minute ceiling rounding, SKU pricing, or any of the 6 `cicdcost.com` invoice components on any of the 27 repositories, `verify_10_repos_all_modes.py` exits non-zero and prints the exact field delta.

---

## Part 1: 10 Primary Repositories (`3 Internal + 7 Modern OSS`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`MaximVote/maxim`** | Internal (MaximVote • Private) | `41` / `45` / `57` | `195.00m` | **`$1,009.41/mo`** | `4221.8%` | `$4,269.38/mo` (`9.1m`) | `5.83` | `31.33` | `7.52` | `81.11` |
| **`mgjartproject/FineArts-BE`** | Internal (Grassy Fine Arts • Private) | `2` / `6` / `6` | `18.00m` | **`$46.76/mo`** | `389.7%` | `$2,979.04/mo` (`6.35m`) | `0.57` | `23.82` | `7.83` | `55.55` |
| **`Eazy-Ops/Backend`** | Internal (Eazy-Ops • Private) | `5` / `5` / `5` | `28.00m` | **`$72.74/mo`** | `606.2%` | `$8,374.22/mo` (`17.85m`) | `1.41` | `24.72` | `8.47` | `59.16` |
| **`astral-sh/uv`** | Fresh OSS (Rust/Python Packaging) | `47` / `223` / `243` | `746.25m` | **`$3,464.00/mo`** | `16156.3%` | `$6,122.62/mo` (`13.05m`) | `8.35` | `34.12` | `10.25` | `114.01` |
| **`denoland/deno`** | Fresh OSS (JS/TS Runtime) | `11` / `59` / `162` | `1860.00m` | **`$20,206.81/mo`** | `40269.0%` | `$13,015.98/mo` (`27.75m`) | `5.66` | `27.18` | `8.41` | `69.35` |
| **`ggml-org/llama.cpp`** | Fresh OSS (C++ LLM Inference) | `62` / `121` / `152` | `459.85m` | **`$1,930.75/mo`** | `9955.8%` | `$6,378.09/mo` (`13.6m`) | `7.47` | `29.66` | `11.59` | `113.17` |
| **`immich-app/immich`** | Fresh OSS (Photo/Video Cloud) | `27` / `76` / `85` | `269.00m` | **`$819.67/mo`** | `5823.9%` | `$4,879.91/mo` (`10.4m`) | `4.61` | `22.90` | `8.51` | `64.06` |
| **`langgenius/dify`** | Fresh OSS (Agentic AI Platform) | `41` / `84` / `117` | `393.50m` | **`$1,153.51/mo`** | `8519.3%` | `$4,853.93/mo` (`10.35m`) | `6.43` | `29.02` | `8.90` | `105.27` |
| **`open-webui/open-webui`** | Fresh OSS (AI Web Interface) | `7` / `13` / `18` | `87.00m` | **`$226.03/mo`** | `1883.6%` | `$4,455.57/mo` (`9.5m`) | `0.61` | `6.98` | `5.07` | `62.78` |
| **`mendableai/firecrawl`** | Fresh OSS (AI Web Scraper) | `36` / `71` / `86` | `626.85m` | **`$1,978.81/mo`** | `13571.3%` | `$39,615.17/mo` (`84.45m`) | `3.74` | `11.61` | `9.25` | `147.79` |

### Part 1B: 10 Primary Repositories — 6 `cicdcost.com` Line-by-Line Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Infracost EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`Datadog 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
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

---

## Part 2: 10 Fresh World-Scale GitHub Repositories (`/Users/araza/personal/top10-github-repos`) — Core KPIs & 3-Mode Performance

| Repository | Category | WF / Def / Exp Jobs | Billed Mins/Run | Gross Monthly Compute | Free-Tier Burn % | Monthly Dev Wait (`$65/hr`) | Engine (`ms`) | CLI (`ms`) | Workflow (`ms`) | MCP 4-RPC (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`microsoft/vscode`** | World-Scale OSS (VS Code IDE) | `5` / `26` / `36` | `127.00m` | **`$2,103.51/mo`** | `2749.6%` | `$3,680.50/mo` (`7.85m`) | `1.46` | `10.83` | `15.83` | `200.25` |
| **`flutter/flutter`** | World-Scale OSS (Flutter SDK) | `6` / `11` / `11` | `20.00m` | **`$76.21/mo`** | `433.0%` | `$1,337.97/mo` (`2.85m`) | `0.47` | `7.42` | `16.11` | `195.16` |
| **`ollama/ollama`** | World-Scale OSS (Local LLM Runner) | `4` / `18` / `47` | `149.00m` | **`$1,266.09/mo`** | `3225.9%` | `$3,563.59/mo` (`7.6m`) | `0.69` | `6.25` | `9.30` | `235.69` |
| **`huggingface/transformers`** | World-Scale OSS (ML Transformers) | `7` / `37` / `57` | `217.00m` | **`$563.77/mo`** | `4698.1%` | `$3,563.59/mo` (`7.6m`) | `1.09` | `6.11` | `7.58` | `243.54` |
| **`vercel/next.js`** | World-Scale OSS (Next.js Framework) | `5` / `38` / `45` | `132.00m` | **`$919.69/mo`** | `2857.8%` | `$2,511.40/mo` (`5.35m`) | `1.15` | `7.06` | `6.60` | `222.41` |
| **`langchain-ai/langchain`** | World-Scale OSS (Agentic Framework) | `27` / `41` / `43` | `105.00m` | **`$272.79/mo`** | `2273.3%` | `$3,095.95/mo` (`6.6m`) | `4.56` | `10.27` | `10.24` | `233.10` |
| **`n8n-io/n8n`** | World-Scale OSS (Workflow Automation) | `12` / `38` / `44` | `198.85m` | **`$636.25/mo`** | `4305.1%` | `$4,853.93/mo` (`10.35m`) | `0.97` | `6.88` | `13.22` | `216.52` |
| **`supabase/supabase`** | World-Scale OSS (Postgres Cloud) | `12` / `17` / `25` | `68.85m` | **`$268.46/mo`** | `1490.6%` | `$2,554.70/mo` (`5.45m`) | `3.79` | `10.54` | `8.61` | `95.58` |
| **`home-assistant/core`** | World-Scale OSS (Home Automation) | `19` / `66` / `87` | `561.00m` | **`$1,403.79/mo`** | `12145.7%` | `$22,139.29/mo` (`47.2m`) | `15.38` | `22.11` | `8.30` | `93.68` |
| **`pydantic/pydantic`** | World-Scale OSS (Python Validation) | `12` / `66` / `175` | `930.70m` | **`$4,759.88/mo`** | `20149.7%` | `$5,958.08/mo` (`12.7m`) | `7.70` | `17.53` | `5.60` | `87.59` |

### Part 2B: 10 Fresh World-Scale Repositories — 6 `cicdcost.com` Line-by-Line Invoice Components (`$/mo`)

| Repository | C1: Rounding Tax | C2: Infracost EC2 (`Spot $/mo`) | C3: Artifact (`90d` -> `S3 14d`) | C4: Parallelism Tax | C5: Log Profile (`Datadog 90d`) | C6: Egress (`NAT Worst` -> `VPC+Cache`) |
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

---

## Part 3: 7 Internal Edge-Case Repositories — KPIs & Latency

| Repository | Edge-Case Scenario | WF / Exp Jobs | Gross Monthly Compute | Net Billed Monthly | CLI Latency (`ms`) |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **`MaximVote/tf`** | Internal (6 Terragrunt VPN/GCP workflows) | `6` / `8` | `$70.15/mo` | **`$70.15/mo`** | `5.86` |
| **`mgjartproject/gfa-data-pipelines`** | Internal (17x matrix scraper pipeline) | `2` / `23` | `$127.30/mo` | **`$127.30/mo`** | `5.23` |
| **`mgjartproject/frontend`** | Internal (GFA Frontend WIF + Security) | `2` / `6` | `$44.17/mo` | **`$44.17/mo`** | `4.76` |
| **`mgjartproject/FineArt_AI_Pipeline`** | Internal Edge Case (0 workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `4.76` |
| **`Eazy-Ops/frontend`** | Internal (Eazy-Ops Frontend Docker) | `2` / `2` | `$20.78/mo` | **`$20.78/mo`** | `5.35` |
| **`Eazy-Ops/multi-cloud-finops-mcp-server`** | Internal (Within 2,000m Free Tier -> $0 net) | `1` / `1` | `$5.20/mo` | **`$0.00/mo`** | `4.54` |
| **`Eazy-Ops/Automation`** | Internal Edge Case (Nested cron/.github/workflows) | `0` / `0` | `$0.00/mo` | **`$0.00/mo`** | `4.88` |
