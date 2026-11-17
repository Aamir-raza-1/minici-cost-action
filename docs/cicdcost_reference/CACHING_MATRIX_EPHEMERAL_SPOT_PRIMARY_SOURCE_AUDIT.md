# Step 0: Independent Primary-Source Fact-Check & Reference Audit — Caching, Matrix, Ephemeral Runners, Spot Instances & Build-Minute Mechanics

**Audited & Cross-Checked Against Primary Sources:**
1. **AWS Live Spot Instance Advisor Feed:** `https://spot-bid-advisor.s3.amazonaws.com/spot-advisor-data.json` (`us-east-1` Linux live JSON feed)
2. **AWS Official Service Pricing (`us-east-1`):**
   - **Amazon EC2 On-Demand:** `t3.medium` (`$0.0416/hr`), `m5.large` (`$0.096/hr`), `m5.xlarge` (`$0.192/hr`), `c5.2xlarge` (`$0.340/hr`), `c5.4xlarge` (`$0.680/hr`), `c6g.xlarge` (`$0.136/hr`), `c7g.2xlarge` (`$0.290/hr`), `c7a.xlarge` (`$0.2052/hr`), `c6a.2xlarge` (`$0.306/hr`).
   - **Amazon EKS Control Plane:** `$0.10/hr` = **`$73.00/mo`** (`730 hrs`) under Standard Support (`14` months per K8s minor version), jumping **6x to `$0.60/hr` (`$438.00/mo`)** under Extended Support. Compare against **GKE Free Tier** (`$74.40/mo` zonal cluster management fee waived for 1 cluster = `$0/mo` control plane) and **Azure AKS Free Tier** (`$0/mo` control plane; `$0.10/hr` only with Uptime SLA).
   - **Amazon ECR & Pull-Through Cache:** `$0.10/GB-month` compressed image storage, **`$0.00/GB` same-region data transfer** to EC2/EKS/ECS/Fargate runners, eliminating `$0.045/GB` NAT Gateway data processing fees and avoiding Docker Hub anonymous (`100 pulls / 6h`) and authenticated free (`200 pulls / 6h`) rate limits.
3. **Vercel Turborepo Remote Cache & Nx Cloud Official Pricing:**
   - **Primary-Source Correction vs. `cicdcost.com`:** `cicdcost.com/ci-cd-caching-strategies` claims managed remote caches (`Turborepo Remote`, `Nx Cloud`, `BuildBuddy`) all cost `$25–$100/seat/mo`. In reality, **Vercel's Managed Turborepo Remote Cache is `$0/mo` (Free)** across Hobby (`100 GB/mo` artifact upload limit), Pro (`1 TB/mo` limit), and Enterprise (`4 TB/mo` limit) with **`$0/GB` egress fees**. Self-hosted S3-backed remote cache (`duckturbocache` / `turborepo-remote-cache` / `sccache` S3) costs **`$5–$25/mo`** at typical scale (`$0.023/GB-mo` S3 Standard), while full Bazel remote execution/cache clusters (`BuildBuddy` / self-hosted EC2+S3) cost `$50–$200/mo`.
4. **GitHub Actions, GitLab CI & CircleCI Official 2026 Billing Docs:**
   - GitHub Actions 1-minute per-job ceiling rounding (`ceil(raw_duration_mins)`), `2,000` Free / `3,000` Team Linux mins, `10 GB` per-repo cache limit with `7-day` unused LRU eviction, `$0.006/min` Linux 2-core, `$0.010/min` Windows 4-core (`1.67x`), `$0.062/min` macOS 3-core M1 (`10.33x`), and `$0.008/GB-day` (`~$0.24–$0.25/GB-mo`) artifact storage overage beyond `500 MB` (Free) / `2 GB` (Team).

---

## 1. Primary-Source Discrepancies Caught vs. `cicdcost.com`

| Topic / Metric | `cicdcost.com` Claim | Official Primary-Source Reality (Verified) | How `minici` Models It |
| :--- | :--- | :--- | :--- |
| **AWS Spot Interruption Rates (`us-east-1` Linux)** | Claims `m5/m6i` has `3–7%` interruption, `c5/c6i` has `5–10%`, and `t3.medium` has `<5%` low interruption. | Live AWS `spot-advisor-data.json` (`us-east-1` Linux) shows single-family **`m5.large`, `m5.xlarge`, `m6i.large`, `m6i.xlarge`, `c5.2xlarge`, `c5.4xlarge`, and `t3.medium` all sit in the `>20%` interruption band** (`58–68%` savings). Low interruption (`<5%` to `5–10%`) in `us-east-1` is achieved by **ARM64 Graviton3 (`c7g.large/xlarge/2xlarge` `5–10%`, `c7g.4xlarge` `<5%`)**, **AMD EPYC (`c7a.large/2xlarge/4xlarge` & `c6a.*` `5–10%`)**, or **multi-family Karpenter Spot NodePools** (`c7a + c6a + c7g + c5` across 3+ AZs -> `<5%` effective pool interruption). | Reports both the single-family Spot risk and the **diversified Karpenter multi-family/Graviton `<5%` interruption model**, plus a `1.25%–3.75%` retry compute margin (`interruption_rate * 0.25..0.50`). |
| **Turborepo / Distributed Remote Cache Cost** | Claims managed remote cache services (`Turborepo Remote`, `Nx Cloud`, `BuildBuddy`) cost `$25–$100/seat/mo` (`$50–$200/mo` self-hosted). | **Vercel Turborepo Remote Cache is `$0/mo` (Free)** up to `100 GB/mo` (Hobby) / `1 TB/mo` (Pro) with `$0` egress; S3-backed `sccache`/`ccache`/Gradle remote cache costs **`$5–$25/mo`** (`$0.023/GB-mo`), while Bazel/Nx Cloud Enterprise/dedicated cache servers cost `$50–$200/mo`. | Models **`$0/mo`** for `turborepo` remote cache (within `100 GB–1 TB/mo`), **`$15/mo`** for S3-backed compiler/Gradle cache, and **`$125/mo`** (`$50–$200/mo` midpoint) for Bazel/monorepo cluster cache. |
| **Kubernetes Ephemeral Runner Cluster Overhead** | Assumes a mandatory `$73/mo` EKS control plane + `$30–$50/mo` K8s ops overhead (`~$113/mo` fixed floor) on every ephemeral cluster. | AWS EKS charges **`$73/mo` (`$0.10/hr`)** Standard Support (`$438/mo` Extended Support), whereas **GKE (1 zonal cluster free tier)** and **AKS (Free tier)** charge **`$0/mo`** for the control plane, and serverless EC2 runners (`RunsOn` / Lambda-SQS / CodeBuild) have **`$0/mo` idle cluster fee**. | Models both **EKS-backed ARC + Karpenter** (`$73/mo` control plane + `$40/mo` ops = `$113/mo` fixed floor, crossover at `~40–50%` always-on utilization) and **zero-idle ephemeral runners** (`GKE/AKS/Serverless EC2`). |

---

## 2. Verified Formulas & Rules Across All 5 Pages

### Page A: `/ci-cd-caching-strategies` (7-Layer Caching Strategy & Poisoning Guard)
1. **7-Layer Investment Ordering (by ROI & Effort):**
   - **Layer 1 — Dependency Cache (`npm`/`pnpm`/`uv`/`pip`/`Maven`/`Cargo`):** `1–2 days` effort; `60–90%` off cold install (`90s -> 5s`), **`15–30%` off total monthly bill**.
   - **Layer 2 — Docker Layer Cache (`BuildKit` registry/GHA export):** `3–5 days` effort (`--cache-to=type=registry` / `type=gha`); **`30–70%` off Docker build time** (`10m -> 2m`).
   - **Layer 3 — Test Result Cache (`Jest --cache` / `pytest --lf` / `Gradle --build-cache`):** `1 week` effort; **`20–40%` off stable test suites**.
   - **Layer 4 — Compiler / Build Artifact Cache (`sccache` / `ccache` / `Gradle` / `Bazel`):** **`30–60%` off compile-heavy builds** (`70–90%` C++/Rust recompile reduction).
   - **Layer 5 — Distributed Remote Build Cache (`Turborepo` / `Nx Cloud` / `Bazel Remote` / `Develocity`):** `2–4 weeks` effort; **`50–90%` saving on monorepos** (`10+` units or `100k+` LOC).
   - **Layer 6 — Container Base-Image Pull-Through Mirror (`ECR Pull-Through Cache` / `Harbor` / `GHCR`):** **`10–25%` off Docker build time** + `$0.00/GB` same-region transfer (`$0.10/GB-mo` ECR storage).
   - **Layer 7 — Package Registry Mirror (`Artifactory` / `Nexus` / `Verdaccio`):** **`5–15%` off dependency install time** + upstream registry outage immunity (recommended at `100+` active devs).
2. **Cache Failure Modes & Write-Guard Hygiene:**
   - **Stale cache hit:** Missing lockfile hash (`hashFiles(...)`) in cache key.
   - **PR cache poisoning:** PR jobs writing directly to shared caches without `lookup-only: true` on PRs or `main`-branch write restriction.
   - **Runaway cache size:** Exceeding GitHub's `10 GB` per-repo cache limit (`LRU` eviction thrashing).

### Page B: `/matrix-build-cost-strategy` (Library vs. Application & Conditional PR Matrix)
1. **Library vs. Application Repo Classification:**
   - **Library / SDK / CLI Runtime (`library-or-sdk`):** Detected via multi-OS/multi-version release artifacts, `Cargo.toml` `[lib]`, `pyproject.toml` wheel builds (`maturin`/`cibuildwheel`), `package.json` `"exports"`/`"main"` without app deploy steps, or cross-platform binary release workflows. Full compatibility matrix is legitimate on `main`/nightly.
   - **Deployable Application (`application`):** Ships 1 production OS + 1 runtime version (Docker/K8s/Terraform/Vercel/cloud deploy). Multi-OS or multi-runtime matrices on an application repo are cargo-culted waste (e.g., `3 OS × 5 Node = 15 cells` where `14/15 = 93.3%` cells are redundant).
2. **Conditional PR Matrix Pattern (`80–89%` PR Matrix Savings):**
   - Running `1` representative cell (`ubuntu-latest` + primary runtime) on every `pull_request` (`~80–85%` of monthly run volume) and running the full `N`-cell matrix only on `push` to `main` and nightly `schedule` cuts matrix compute spend by **`~80%`** (e.g., `9 cells -> 1 cell` on PRs = `88.9%` per-PR reduction, `$200/mo -> $40/mo`).

### Page C: `/ephemeral-runner-cost` (Always-On Utilization Crossover & Cold-Start Wait Tax)
1. **Always-On vs. Ephemeral Utilization Crossover (`40–50%` Rule):**
   - Always-on EC2 runner pays for `720 hrs/mo` (`m5.large` = `$69.12/mo` per node).
   - `always_on_runner_utilization_pct = (monthly_billed_mins / 60.0) / (required_runner_nodes * 720.0) * 100.0`.
   - When `always_on_runner_utilization_pct < 45.0%`, **ephemeral (`ARC + Karpenter` or `GitLab K8s Executor`)** eliminates `55–88%` idle capacity waste.
2. **Cold-Start Wait-Time Payroll Tax (`90–180s` Slow Path vs. `10–30s` Warm Path):**
   - Slow path (new EC2 node via Karpenter: `30–60s` EC2 boot + `5–15s` Kubelet/CNI + `5–30s` image pull + `5–10s` registration) = **`90–180s` (`1.5m` typical)** extra queue wait on `~10%` of burst jobs.
   - Warm path (pod on existing warm node) = **`10–30s` (`0.25m`)**.
   - Pure scale-to-zero cold-start wait tax = `(monthly_runs * 0.10 * 1.5 mins / 60.0) * dev_hourly_rate_usd`.
   - **Winning Hybrid Pattern (for `25–200` devs):** Keep a small **1–2 node warm pool** for the morning burst + **Spot-backed Karpenter ephemeral pool** for bulk PR checks (`50–70%` compute savings with `$0` cold-start payroll tax).

### Page D: `/spot-instance-ci-cost` (Spot Interruption Margin & On-Demand Pinning for High-Stakes Jobs)
1. **Spot Interruption Retry Overhead (`1.25%`–`3.75%`):**
   - Diversified Graviton/AMD Karpenter Spot pool (`c7g` + `c7a` + `c6a`) has `~5%` monthly interruption rate (`<5%` to `5–10%` verified in AWS Spot Advisor).
   - Wasted compute per interrupted job = `25–50%` of job duration -> **`1.25%–2.5%` effective compute retry overhead** (`$48.00/mo -> $48.60/mo`).
2. **3 Categories of Spot-Ineligible Jobs (Must Pin to On-Demand):**
   - **Category 1 — Release / Publish Ceremony Jobs:** Jobs matching `release`, `publish`, `goreleaser`, `semantic-release`, `changesets`.
   - **Category 2 — Long-Running Integration / E2E Jobs (`> 30 mins`):** Probability of interruption scales with duration; a `60m` job interrupted at `minute 50` wastes `50m`.
   - **Category 3 — Non-Idempotent Deploy / State-Mutating Jobs:** Jobs running `terraform apply`, `terragrunt apply`, `kubectl apply`, `helm upgrade`, `aws ecs update-service`, database migrations, or production deploys.
   - **Hybrid Spot/On-Demand Split:** Eligible PR/test jobs (`typically 80–90%` of minutes) run on Spot (`60–71%` off), while Spot-ineligible jobs (`10–20%`) stay pinned to On-Demand.

### Page E: `/cost-per-build-minute` (Cost Per Deployment Translation & Billing Mechanics)
1. **Cost Per Deployment / Merge Translation:**
   - Translates monthly compute into **Cost Per PR / Deployment** (`gross_compute_cost_per_run_usd`) and **Annualized CI Run Rate** (`gross_monthly_compute_cost_usd * 12.0`).
