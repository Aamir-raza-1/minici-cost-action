# Infracost + MiniCI: Complete Cloud Infrastructure & CI/CD Pipeline Cost Visibility in Every Pull Request

> **SEO & AI Search Metadata (GEO / AIO Ready)**
> - **Suggested URL Slug:** `/blog/infracost-and-minici-full-stack-finops-pull-request-costs`
> - **Meta Title:** `Infracost + MiniCI: Full Cloud IaC & CI/CD Cost Visibility`
> - **Meta Description:** `Compare what Infracost (Terraform/IaC cloud costs) and MiniCI (CI/CD workflow & matrix costs) measure, and run both in one Pull Request for 100% FinOps visibility.`
> - **Target Keywords:** `Infracost GitHub Actions`, `shift-left FinOps`, `Terraform and CI/CD cost calculator`, `Pull Request cost estimation`, `Infracost vs CI/CD cost`, `GitHub Actions PR cost comment`, `MiniCI`

---

## Every Engineering Repository Generates Two Separate Monthly Bills

When a Pull Request changes infrastructure and deployment code, it affects **two completely distinct monthly invoices**:

1. **The Provisioned Cloud Infrastructure Bill (`AWS` / `GCP` / `Azure`):** What the databases, compute instances, Kubernetes clusters, storage buckets, and load balancers defined in Terraform, OpenTofu, Terragrunt, or CloudFormation cost to run 24/7 in staging and production.
2. **The CI/CD Build & Delivery Pipeline Bill (`GitHub Actions` / `GitLab CI` / `CircleCI`):** What the CI runners, test matrices, container builds, artifact storage, and developer wait-time defined in `.github/workflows/*.yml` cost every time engineers push commits and open Pull Requests.

**Infracost** is purpose-built for the first bill. **[MiniCI](https://github.com/minici-dev/minici-cost-action)** (`minici v0.9.0`) is purpose-built for the second. They do not overlap, replace, or wrap one another—running **both side-by-side on the same repository** gives engineering and FinOps teams `100%` shift-left cost visibility before merge.

---

## Infracost vs. MiniCI: Clear Side-by-Side Comparison

| Capability | Infracost (`infracost scan` / `diff`) | MiniCI (`minici breakdown` / `diff`) | Combined Outcome |
| :--- | :--- | :--- | :--- |
| **Primary Domain** | **Provisioned Cloud Infrastructure (IaC)** | **CI/CD Pipeline Execution & Workflow Hygiene** | **100% Engineering Run-Rate Visibility** |
| **Files Scanned** | `*.tf`, `*.tfvars`, `terragrunt.hcl`, CloudFormation | `.github/workflows/*.yml`, `.gitlab-ci.yml`, `.circleci/config.yml`, `azure-pipelines.yml`, `bitbucket-pipelines.yml`, `buildspec.yml`, `cloudbuild.yaml` | Every infra + pipeline file in a monorepo or service repo is covered |
| **Cost Drivers Captured** | EC2/GCE/Azure VMs, RDS/Cloud SQL, EKS/GKE, NAT Gateways, S3/GCS buckets, FinOps tagging policies | `strategy.matrix` fan-out, Linux/Windows/macOS runner SKUs, `ceil(60s)` whole-minute rounding, `upload-artifact` storage, superseded PR runs & critical-path wait | Catches both an oversized `m6i.xlarge` in Terraform **and** a runaway `6-cell` matrix in GitHub Actions |
| **Pure App Repo (`0` `.tf` files)** | Reports `$0.00/mo` (`0` IaC resources) | Captures full CI/CD spend (e.g., `$4,316.14/mo` on `microsoft/vscode`) | No blind spots on frontend/SDK/mobile repos |
| **Execution Model** | Queries Infracost Cloud Pricing API (`infracost auth login` / `INFRACOST_API_KEY`) | `100%` offline static Rust binary (`< 5 ms`, `~3 MiB` RAM, zero API keys) | Fast, non-blocking sticky PR comments side-by-side |

---

## Live Walkthrough: Running Both on One Full-Stack Repository

Consider a typical full-stack repository containing **both** Terraform infrastructure (`infra/main.tf`) and a GitHub Actions build + Terraform plan pipeline (`.github/workflows/deploy.yml`):

```text
fullstack-finops-repo/
├── infra/
│   └── main.tf                    # AWS RDS Postgres + EC2 API worker (Scanned by Infracost)
└── .github/
    └── workflows/
        └── deploy.yml             # Unit test matrix + E2E + Terraform Plan (Scanned by MiniCI)
```

### Step 1: Baseline Audit on `main`

On `main`, the CI workflow runs `1x ubuntu-4core` (`unit-tests`) and `1x ubuntu-latest` (`terraform-plan` uploading `tfplan` as an artifact). Running `minici breakdown` locally executes in **`1.11 ms`**:

```text
$ minici breakdown --path .

Project: ./fullstack-finops-repo (1 workflow(s), 2 expanded jobs • 1.11 ms)

 WORKFLOW / JOB                              MATRIX × RUNNER           BILLED MINS    RATE/MIN    COST/RUN   MONTHLY COST
 ──────────────────────────────────────────  ────────────────────────  ───────────  ──────────  ──────────  ─────────────
 deploy.yml • unit-tests                     1x ubuntu-4core                  4.0m   $0.0120/m      $0.048      $20.78/mo
 deploy.yml • terraform-plan                 1x ubuntu-latest                 2.0m   $0.0060/m      $0.012       $5.20/mo
 ──────────────────────────────────────────  ────────────────────────  ───────────  ──────────  ──────────  ─────────────
 OVERALL TOTAL (433 runs/mo • paid tier)     2 expanded job(s)                6.0m           —      $0.060      $25.98/mo

 • Free-Tier Quota  : 2598 / 2000 mins (43.3% of monthly quota) • Net Billed: $20.78/mo • Rounding Overhead: 0.9m/run ($4.11/mo)
 • Dev Wait-Time    : 3.6m critical path ($1.56/run • $675.48/mo at 40% DORA context-switch loss)
 • Artifact Storage : 1 upload job(s) • 90d retention ($81.19/mo, 324.75 GB steady-state) (14d retention reduces storage to $12.63/mo)
 • Pipeline Hygiene : Cache 0/2 jobs (caching 1 uncached saves ~$4.68/mo) • Cancel-in-progress missing on 1 wf (~$3.90/mo superseded waste) • 1 larger runner job(s) ($20.78/mo metered from 1st sec; never draw free mins) • 2 job(s) missing timeout-minutes ($6.48 6h hung-run exposure)
 • Unit Economics   : $0.24/merged PR • $6.50/active dev/mo
 • Recommendation   : Stay on hosted runners (2598 / 5,000 mins/mo break-even); set artifact `retention-days: 14` to reduce storage to $12.63/mo
```

Notice what `minici` immediately catches on `deploy.yml • terraform-plan`: uploading `tfplan` via `actions/upload-artifact@v4` without setting `retention-days` defaults to `90 days` (`$81.19/mo` steady-state storage—more than `3x` the `$25.98/mo` compute bill!).

---

### Step 2: A Pull Request Scales Both Infra (`main.tf`) and CI (`deploy.yml`)

Now an engineer opens branch `feat/multi-region-e2e` with two changes in the same PR:
1. **In `infra/main.tf`:** Upgrades `aws_instance.api_worker` from `m6i.large` to `m6i.xlarge` — **Infracost** (`infracost scan` / `infracost diff`) catches the exact monthly EC2 infrastructure increase and FinOps policy checks.
2. **In `.github/workflows/deploy.yml`:** Expands `unit-tests` to a `[ubuntu-4core, ubuntu-8core] × [1, 2, 3]` shard matrix and adds an `ios-safari-e2e` job on `macos-latest` — **MiniCI** catches the exact CI/CD cost jump in **`230 µs`**:

```text
$ minici diff --path . --compare-to main

+====================================================================================+
| MINICI — PULL REQUEST COST DIFF (230 µs • Exit Code 0 Informational)                |
+====================================================================================+
Comparing            : `git-ref:main` (Base) -> `./fullstack-finops-repo` (PR Head)
1. Matrix Jobs       : 2 jobs -> 8 jobs (+6 jobs)
2. Billed Mins / Run : 6.0m -> 30.0m (+24.0m / run)
3. Monthly CI Cost   : $25.98/mo -> $289.24/mo (+$263.26/mo • +1013.3%)
4. Dev Wait Cost ($) : $675.48/mo -> $675.48/mo (+$0.00/mo) [Assumed Rate Configurable]
5. Key Change        : Added job `unit-tests [ubuntu-4core]` (`.github/workflows/deploy.yml`): `3x ubuntu-4core` (+$0.144/run) (total changes: 4)
+====================================================================================+
```

Before merging, running `minici shrink --path .` previews surgical, comment-preserving YAML fixes—dropping that `$81.19/mo` `tfplan` artifact bill down to **`$6.32/mo` (`-$74.87/mo`)** and adding `cancel-in-progress` and `30m` hung-runner timeouts:

```text
$ minici shrink --path .

+====================================================================================+
| MINICI SURGICAL WORKFLOW SHRINK RECEIPT (1 file(s), 8 edit(s) • DRY-RUN) |
+====================================================================================+
  • Monthly Compute : $289.24/mo -> $289.24/mo (+0.00/mo • +0.0%)
  • Billed Mins/Run : 30.0m -> 30.0m (+0.0m/run)
  • Artifact Storage: $81.19/mo -> $6.32/mo (-74.87/mo)
  • Timeout Guard   : 4 -> 0 job(s) missing timeout-minutes
  1. [MINICI-SHRINK-001] .github/workflows/deploy.yml:L7 — Add `concurrency` cancel-in-progress guard for superseded PR pushes
  2. [MINICI-SHRINK-002] .github/workflows/deploy.yml:L13 — Cap job hung-runner exposure from 360m default to `timeout-minutes: 30`
  ...
  8. [MINICI-SHRINK-004] .github/workflows/deploy.yml:L31 — Set explicit `retention-days: 7` on `actions/upload-artifact` (replaces 90d default)
+====================================================================================+
```

---

## Drop-In GitHub Actions Setup: Run Infracost + MiniCI Together

Add `.github/workflows/finops-visibility.yml` so every Pull Request gets **Infracost** for Cloud IaC (`*.tf`) and **MiniCI** for CI/CD Pipelines (`*.yml`):

```yaml
name: Full-Stack FinOps (Cloud IaC + CI/CD)
on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  cloud-and-ci-costs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Needed for branch diffing against base branch

      # 1. Cloud Infrastructure Cost Visibility (Terraform / OpenTofu / Terragrunt)
      - name: Setup Infracost
        uses: infracost/actions/setup@v3
        with:
          api-key: ${{ secrets.INFRACOST_API_KEY }}

      - name: Infracost Cloud IaC Breakdown & PR Comment
        run: |
          infracost breakdown --path . --format json --out-file infracost.json
          infracost comment github --path infracost.json \
            --repo "$GITHUB_REPOSITORY" \
            --pull-request "${{ github.event.pull_request.number }}" \
            --github-token "${{ secrets.GITHUB_TOKEN }}" \
            --behavior update

      # 2. CI/CD Workflow & Matrix Cost Visibility (< 5 ms, zero API keys)
      - name: MiniCI Workflow Cost Breakdown & PR Diff
        uses: minici-dev/minici-cost-action@v1
        with:
          compare-to: "origin/${{ github.base_ref }}"
          post-pr-comment: "true"
          suggest-fixes: "true"
```

---

## Frequently Asked Questions (FAQ)

### Does Infracost calculate GitHub Actions or GitLab CI workflow costs?
No. Infracost scans Infrastructure-as-Code files (`*.tf`, `terragrunt.hcl`, CloudFormation) to estimate provisioned AWS, GCP, and Azure cloud resource bills. It does not parse `.github/workflows/*.yml` or `.gitlab-ci.yml` CI/CD pipelines. Pairing Infracost with MiniCI covers both deployed cloud resources and CI/CD runner spend.

### Does MiniCI replace Infracost for Terraform projects?
No. MiniCI focuses strictly on CI/CD workflow execution, runner SKUs, matrix expansion, CI artifact storage, and PR wait-time, whereas Infracost prices the cloud infrastructure resources provisioned by Terraform/OpenTofu. Using both together gives engineering leads full visibility into total monthly engineering run-rate.

### Do Infracost and MiniCI conflict when posting Pull Request comments?
No. Each tool uses its own idempotent HTML comment marker (`<!-- minici-cost-comment -->` for MiniCI) so each updates its own single sticky Pull Request comment cleanly without duplicating comments or blocking builds.
