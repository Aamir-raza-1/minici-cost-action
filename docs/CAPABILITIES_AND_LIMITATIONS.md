# 🔬 MiniCI (`minici-cost-action`) — Capabilities, Math Models & Known Limitations

This document provides complete engineering transparency into:
1. **What `minici` computes** (all 20 unified CI/CD FinOps & engineering velocity models).
2. **How build duration is calculated** (exact static step-level formula + overrides).
3. **Standalone architecture proof** (zero runtime third-party CLIs or external pricing APIs).
4. **Exact crash & failure boundaries** (where `minici` will fail or under/over-estimate).

---

## 1. Zero External Runtime Dependencies (100% Standalone Rust)

* **Zero External Crates:** `crates/minici/Cargo.toml` has `[dependencies]` completely empty (`std` only).
* **Zero Runtime Third-Party Binaries or Cloud APIs:** `minici` **never** shells out to third-party cost CLIs and **never** queries external GraphQL/REST pricing endpoints during `breakdown`, `diff`, `comment`, `catalog`, or `mcp`.
* **Compiled Multi-Vendor SKU Catalog (`crates/minici/src/catalog.rs`):** All `160+` runner and cloud instance rates across `GitHub Actions`, `GitLab CI`, `CircleCI`, `Bitbucket Pipelines`, `AWS CodeBuild`, `Google Cloud Build`, `Azure Pipelines`, `AWS EC2` (On-Demand & Spot), `GCP Compute Engine`, `Azure Virtual Machines`, `S3`, and `NAT Gateway` are compiled directly into the binary and verified against `pricing/*.json` at `0.00%` drift (`tools/sync_rust_sku_catalog.py --check`).
  * *Schema Note:* JSON output (`--format json`) preserves legacy field keys under `cost_components` for backwards compatibility with existing automation consumers, populated purely from `minici`'s internal Rust catalog.

---

## 2. How `minici` Calculates Build Time / Job Duration

Because `minici` executes locally in `< 2 ms` without requiring CI API tokens or historical run logs, each job's raw runtime (`est_raw_mins`) is computed deterministically in `crates/minici/src/engine.rs`:

### Step-Level Static Duration Formula

Unless a job specifies an explicit `est-minutes: <N>` or `estimated-minutes: <N>` annotation:

| Detected Job / Step Signal | Duration Added (`minutes`) | Notes |
| :--- | :---: | :--- |
| **Base Runner VM Boot & Provisioning** | `+0.50m` | Applied to every job with a `steps:` block |
| **Fallback Job Without `steps:`** | `3.50m` | Applied only when a non-reusable job omits `steps:` |
| **Local Reusable Caller (`uses: ./.github/workflows/...`)** | `0.00m` | Orchestrator stub (`0` runner VMs allocated; child workflow holds actual jobs) |
| **`actions/checkout`** | `+0.35m` | Discounted by `-0.30m` (floored at `0.50m` job minimum) when `fetch-depth: 1`, `sparse-checkout`, or `--filter=` is used |
| **`upload-artifact`** | `+0.35m` | Covers archive compression and upload I/O |
| **Test Runner Command** (`pytest`, `cargo test`, `npm test`, `vitest`, `go test`, `mvn test`) | `+2.50m` | Added per matching test invocation line |
| **Heavy Compile / Container Build** (`docker build`, `cargo build --release`) | `+3.50m` | Added per matching build invocation line |
| **Standard Step** (`- name:`, `- run:`, `- uses:`) | `+0.25m` | Added per step entry |

### Provider Billing Rounding & Matrix Expansion

1. **Matrix Expansion (`strategy.matrix`):** Multiplies list dimensions (`os × version × shard`), adds explicit `include:` entries, and subtracts matching `exclude:` combinations using overflow-safe saturating arithmetic.
2. **Billing Rounding:**
   * **GitHub Actions & Bitbucket Pipelines:** Every expanded matrix job is rounded up to the nearest whole minute (`raw_mins.ceil().max(1.0)`). Ten `15-second` matrix shards bill `10.0m`, not `2.5m`—`minici` captures this exact **Sub-Minute Ceiling Rounding Tax**.
   * **GitLab CI, CircleCI & Google Cloud Build:** Billed at exact fractional minutes (`raw_mins.max(0.1)`).
3. **Critical-Path Wait Time:** Tracks `needs:` DAG dependencies across jobs inside each workflow so parallel jobs use `max(duration)` while sequential stages accumulate `parent_finish + job_duration`.

### Pinning Exact Historical Runtime (`est-minutes`)

When a 1-line script wraps a long-running integration suite (`- run: ./scripts/e2e_45min.sh`), override the static heuristic directly in YAML:

```yaml
jobs:
  heavy-e2e:
    runs-on: ubuntu-latest
    # est-minutes: 45
    steps:
      - uses: actions/checkout@v4
      - run: ./scripts/e2e_45min.sh
```

---

## 3. All 20 Unified Cost & Engineering Velocity Models

`minici` evaluates 20 models simultaneously and applies **Strict Applicability Filtering** in CLI, Markdown, and HTML views (non-applicable `$0` hypothetical items are never dumped onto unrelated repositories; full telemetry is always available via `--format json`):

### Core CI/CD Invoice Components (`C1–C6`)
1. **Build Minutes & Sub-Minute Rounding Tax (`build_minutes_cost`):** Linux/Windows/macOS minute split, OS free-tier multipliers (`1x` Linux, `2x` Windows, `10x` macOS), and whole-minute ceiling overhead.
2. **Runner Compute & Self-Hosted Breakeven (`runner_cost`):** Hosted SaaS spend vs. right-sized cloud Spot/On-Demand VMs, always-on idle utilization %, and `5,000 mins/mo` self-hosted breakeven check.
3. **Artifact Storage (`artifact_storage_cost`):** Steady-state GB accumulation using the workflow's actual `retention-days` (`$0.25/GB-month` hosted vs. object-storage lifecycle retention). Shown in summary only when `jobs_uploading_artifacts > 0`.
4. **Concurrency & Parallelism Cap Tax (`parallelism_cost`):** Detects when concurrent matrix fan-out approaches or exceeds plan runner slot caps.
5. **Log Retention & External SIEM Shipping (`log_retention_cost`):** Native vendor retention (`$0/mo`) vs. external log ingestion when log-shipping steps are detected.
6. **Cloud VPC & NAT Gateway Egress (`cross_region_egress_cost`):** Shown only when self-hosted cloud VPC runners (`self-hosted`, `aws-*`, `gcp-*`, `azure-*`, `codebuild`) or cloud registry transfers are active (`$0/mo` on standard hosted SaaS runners).

### Hidden Engineering Payroll & Hygiene Models (`H1–H5`)
7. **Developer Wait-Time & Context-Switch Loss (`developer_wait_time_cost`):** Critical-path wait cost at `--dev-hourly-rate` plus `40%` productive context-switch recovery loss.
8. **Immediate Failed-Build Rerun Cost (`failed_build_rerun_cost`):** Compute rerun waste + `8-minute` engineer triage cost per flaky failure.
9. **Unrealized Dependency & Build Cache Savings (`build_cache_savings`):** Identifies uncached setup/build jobs and estimates monthly cache hit savings.
10. **Combined Flaky Test Velocity Drag (`flaky_test_ci_cd_tax`):** Immediate rerun waste + `2x` downstream merge-queue/review drag.
11. **Branch Protection & Trigger Hygiene (`branch_protection_cost`):** Quantifies superseded PR push waste (missing `concurrency.cancel-in-progress: true`) and unfiltered docs-only push waste (missing `paths-ignore`).

### Dynamic Team-Stage, Pipeline-Shape & Stack Intelligence (`S1–S5`, `P17–P20`)
12. **Startup Stage & Hardware Payback (`startup_stage_cost`):** Implied active developer count and local Apple Silicon Mac mini runner payback months for macOS-heavy pipelines.
13. **Scale-Up Recoverable Waste & Platform ROI (`scaleup_stage_cost`):** Total recoverable compute + wait-time dollars vs. dedicated platform engineering investment.
14. **Enterprise Compliance & Audit Controls (`enterprise_compliance_cost`):** Regional data-residency multipliers and 1-year audit retention sizing.
15. **Monorepo Pipeline Shape & Affected-Only Graph (`monorepo_ci_cost`):** Detects `pnpm`/`nx`/`turborepo`/`cargo`/`go.work` workspaces and models affected-only graph execution savings.
16. **Microservices Shared Setup Overhead (`microservices_ci_cost`):** Measures duplicated container/setup bootstrap share across jobs and unpinned template refs.
17. **Multi-Vendor TCO Comparison (`vendor_tco_comparison`):** Compares monthly TCO across CI platforms and checks the `2x` annual engineering migration payback rule.
18. **6-Phase Build Breakdown (`build_phase_breakdown`):** Splits spend across Checkout, Dependencies, Compile, Test, Container/Package, and Deploy to rank the highest-ROI first optimization phase.
19. **Workload Compute Profile & ARM64 Explorer (`compute_class_explorer`):** Classifies workloads (`compile-heavy`, `test-heavy`, `balanced-web-backend`, `io-bound`) and calculates eligible Linux `x86_64 -> ARM64` savings (`-37.5%` on GitHub Actions).
20. **Stack-Specific Economics (`stack_specific_economics`):** Detects primary language/build stack (`Rust`, `Node/TS`, `Python`, `Go`, `Java/JVM`, `Docker`, etc.) and recommends stack-native speed/cost levers.

---

## 4. Crash Resilience & Non-Working Boundaries

`minici` contains **zero `.unwrap()` or `.expect()` calls** in non-test production Rust code and has been verified against corrupt non-UTF-8 binary `.yml` files, `0`-byte files, unclosed YAML brackets, multibyte Unicode/emoji comments, and `10^25` matrix overflows (`rc=0`, zero panics).

However, users should be aware of the following **hard operational and static-analysis boundaries**:

| Scenario | What Happens | Mitigation |
| :--- | :--- | :--- |
| **Minimal Docker CI runner without `cargo`** | Composite `action.yml` runs `cargo build --release` when `target/release/minici` is not cached; fails on minimal images (`alpine`, `node:slim`) lacking Rust. | Run on `ubuntu-latest` (Rust preinstalled) or pre-copy the static `minici` binary into `/usr/local/bin`. |
| **`actions/checkout@v4` without `fetch-depth: 0` during `minici diff`** | Shallow clones omit the `main` Git ref; `git show main:...` finds `0` base workflows and reports the entire repo as newly added PR cost. | Always set `fetch-depth: 0` when comparing branches in CI. |
| **Dynamic runtime matrices (`strategy: matrix: ${{ fromJSON(...) }}`)** | Static parser cannot evaluate runtime GitHub Actions JSON outputs from prior jobs; counts dynamic matrix as `1x` job. | Pass explicit static matrix lists where possible or review job count in `--format markdown`. |
| **YAML anchors & merge keys (`&base` / `<<: *base`)** | Line-stream YAML scanner does not expand anchor aliases; inherited `runs-on:` inside an anchor defaults to `ubuntu-latest`. | Specify `runs-on:` explicitly per job when accurate SKU mapping is required. |
| **Cross-repository reusable workflows (`uses: org/other-repo/.github/workflows/ci.yml@v1`)** | Local reusable workflows (`uses: ./.github/workflows/...`) are resolved cleanly, but external repository workflows are not fetched over the network. | Run `minici breakdown` directly inside the shared workflow repository as well. |
| **Opaque wrapper scripts (`- run: ./ci.sh`)** | A single shell wrapper hiding a 90-minute test suite looks like a `1.0m` generic job to static step heuristics. | Add `# est-minutes: 90` or `est-minutes: 90` to the job definition. |
| **Unauthenticated `--repo owner/repo` bursts** | Uses `curl` against GitHub's public API (`60 requests/hr/IP` unauthenticated limit) and fetches `.github/workflows/` only. | Export `GITHUB_TOKEN` (`5,000 req/hr`) or clone non-GitHub repositories locally. |
