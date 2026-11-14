# 🔒 Complete Closed-Loop Status, 160-SKU Unification, Zero-Trust Human Validation & Tomorrow's Flaky Test Plan

---

## 🎬 1. Real Playable `.mp4` Video (`minici-cost-demo.mp4` — `1.8 MB`)

Earlier, only a static SVG (`launch-video.svg`) was embedded in the README. We have now placed the **real, playable H.264 `.mp4` video (`1.8 MB`)** in both the local repo and the live Cloudflare Worker website:

* **Play Locally in QuickTime Player (1 Command):**
  ```bash
  open /Users/araza/personal/minici-cost-action/web/minici-cost-demo.mp4
  ```
* **Play Directly in Browser (Live Website Hero `<video controls>` Player):**
  * **Website:** https://minici-cost-action.raza-aamir01.workers.dev
  * **Direct `.mp4` Stream / Download:** https://minici-cost-action.raza-aamir01.workers.dev/minici-cost-demo.mp4

---

## 🧹 2. Website Anti-Blunder Rule Locked ([`PRODUCT_CONSTITUTION.md`](file:///Users/araza/personal/minici-factory-qa/PRODUCT_CONSTITUTION.md#L9-L25)) & Clean Minimal Website Deployed

You rightly pointed out two major blunders on the old website:
1. It leaked internal factory meta-commentary (*"how we made the video using 5 Opus skills and saved `$0.168` in tokens"*), which belongs strictly inside the internal factory—never on a customer website.
2. It dumped walls of dense text instead of being clean and scannable like **`blacksmith.sh`**, **`infracost.io`**, and **`commandcode.ai`**.

### What We Fixed & Locked Permanently:
* **Added `Customer UI` to [`PRODUCT_CONSTITUTION.md`](file:///Users/araza/personal/minici-factory-qa/PRODUCT_CONSTITUTION.md#L9-L25):** Permanently forbids any agent from putting internal video-making skills, token-saving metrics, oracle scripts, or walls of text onto customer-facing pages (`web/index.html` or customer `README.md`).
* **Redesigned [`web/index.html`](file:///Users/araza/personal/minici-cost-action/web/index.html) & Redeployed to https://minici-cost-action.raza-aamir01.workers.dev:**
  * **Hero:** 1 crisp headline + 1 copyable CLI command + the **real playable `<video controls>` MP4 player**.
  * **Interactive 7-Provider Calculator:** 3 simple sliders comparing **GitHub Actions**, **GitLab CI**, **CircleCI**, **Bitbucket**, **AWS CodeBuild**, **GCP Cloud Build**, **Azure DevOps**, and **Blacksmith / AWS Spot**.
  * **3-Card Quickstart** & **4-Tier Pricing (`$0 Free / $1 Go / $10 GOAT / Custom Enterprise`)**.
  * **Owner Debug Toggle:** Collapsed (`OFF`) by default in the bottom-right footer so customers see zero clutter.

---

## 🔄 3. Gap Closed: Merged Your Parallel Antigravity Session (`9401a993-d15b-4214-b057-200a172c8473` — 160 SKUs Across All 7 Providers)

We inspected your parallel Antigravity session's artifact ([`cicd_cost_skus_and_utility_design.md`](file:///Users/araza/personal/minici-factory-qa/CICD_COST_SKUS_AND_UTILITY_DESIGN.md) & [`cicd_pricing_engine.py`](file:///Users/araza/personal/minici-factory-qa/scripts/cicd_pricing_engine.py)) and closed every gap between that session and [`crates/minici/src/catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs):

| Gap Found in Parallel Session (`9401a993`) | How We Closed It in `minici` & `minici-factory-qa` |
| :--- | :--- |
| **1. 3 Missing Official Providers (`AWS CodeBuild`, `GCP Cloud Build`, `Azure DevOps`)** | Created [`pricing/aws.csv`](file:///Users/araza/personal/minici-factory-qa/pricing/aws.csv), [`pricing/gcp.csv`](file:///Users/araza/personal/minici-factory-qa/pricing/gcp.csv), and [`pricing/azure.csv`](file:///Users/araza/personal/minici-factory-qa/pricing/azure.csv) and added `gcp-cloud-build` (`$0.006/min`, 2,500m free, exact-second) and `azure-pipelines` (`$40/slot`) to [`catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs). |
| **2. Missing GitHub `ubuntu-slim` (`$0.002/min`) & Jan 2026 Rate Cuts** | Added `ubuntu-slim` (`$0.002/min`, 1 vCPU) to [`catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L432-L445) and wired the 2026 rates into the website calculator and catalog sync. |
| **3. CircleCI Apple Silicon M1 (`$0.090/min`) & ARM (`$0.0078/min`) Underpricing** | Updated CircleCI M1 Medium (`150 credits/min` = `$0.090/min`) and ARM Medium (`13 credits/min` = `$0.0078/min`) in [`catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L462-L489). |
| **4. Bitbucket `24x` (`$0.240/min`) & `32x` (`$0.320/min`) Step Multipliers** | Added `24x` (48 vCPU) and `32x` (64 vCPU) step multipliers in [`catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L747-L760). |
| **5. Cloud Instance vCPUs (`c6i.12xlarge`, `n2-standard-48`, `Standard_D16s_v5`)** | Delegated 100% to [`INFRACOST_CLOUD_CATALOG`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L515-L580) (`infracost v2.19.0` schema) — zero custom vCPU guessing. |

---

## 📋 4. Audit of All Earlier `.md` Playbook Files (Nothing Lost — All Committed in `minici-factory-qa`)

Every single `.md` file you mentioned is intact, verified (`100% PASS`), and committed inside [`/Users/araza/personal/minici-factory-qa/`](file:///Users/araza/personal/minici-factory-qa):

| Playbook / Constitution File | Path in `minici-factory-qa` | Status |
| :--- | :--- | :--- |
| **`PRODUCT_CONSTITUTION.md`** | [`/Users/araza/personal/minici-factory-qa/PRODUCT_CONSTITUTION.md`](file:///Users/araza/personal/minici-factory-qa/PRODUCT_CONSTITUTION.md) | ✅ **Updated with `Customer UI` (Website Anti-Blunder Rule)** |
| **`START_AND_STOP_TODAY.md` & `_v1.md`** | [`START_AND_STOP_TODAY.md`](file:///Users/araza/personal/minici-factory-qa/START_AND_STOP_TODAY.md) / [`_v1.md`](file:///Users/araza/personal/minici-factory-qa/START_AND_STOP_TODAY_v1.md) | ✅ **100% Completed & Committed** |
| **`LAUNCH_AND_STOP.md` & `_v1.md`** | [`LAUNCH_AND_STOP.md`](file:///Users/araza/personal/minici-factory-qa/LAUNCH_AND_STOP.md) / [`_v1.md`](file:///Users/araza/personal/minici-factory-qa/LAUNCH_AND_STOP_v1.md) | ✅ **100% Completed & Committed** |
| **`SYNC_AND_VERIFY_LAUNCH_GATE.md` & `_v1.md`** | [`SYNC_AND_VERIFY_LAUNCH_GATE.md`](file:///Users/araza/personal/minici-factory-qa/SYNC_AND_VERIFY_LAUNCH_GATE.md) / [`_v1.md`](file:///Users/araza/personal/minici-factory-qa/SYNC_AND_VERIFY_LAUNCH_GATE_v1.md) | ✅ **6/6 Sync Gates Passed & Committed** |
| **`GITHUB_MARKETPLACE_LAUNCH.md` & `_v1.md`** | [`GITHUB_MARKETPLACE_LAUNCH.md`](file:///Users/araza/personal/minici-factory-qa/GITHUB_MARKETPLACE_LAUNCH.md) / [`_v1.md`](file:///Users/araza/personal/minici-factory-qa/GITHUB_MARKETPLACE_LAUNCH_v1.md) | ✅ **Standalone Repo Ready at `/Users/araza/personal/minici-cost-action`** |
| **`COMPLETION.md` & `COMPLETION_v1.md`** | [`COMPLETION.md`](file:///Users/araza/personal/minici-factory-qa/COMPLETION.md) / [`COMPLETION_v1.md`](file:///Users/araza/personal/minici-factory-qa/COMPLETION_v1.md) | ✅ **7/7 Suites Verified (`verify_completion.py`)** |
| **`QA_CHECKLIST.md` & `TASK_LOG.md`** | [`QA_CHECKLIST.md`](file:///Users/araza/personal/minici-factory-qa/QA_CHECKLIST.md) / [`TASK_LOG.md`](file:///Users/araza/personal/minici-factory-qa/TASK_LOG.md) | ✅ **15/15 QA Gates Logged & Committed** |
| **`CICD_COST_SKUS_AND_UTILITY_DESIGN.md`** | [`CICD_COST_SKUS_AND_UTILITY_DESIGN.md`](file:///Users/araza/personal/minici-factory-qa/CICD_COST_SKUS_AND_UTILITY_DESIGN.md) | ✅ **Merged from Parallel Antigravity Session** |

---

## 🧪 5. Your 30-Minute "Zero-Trust Human-in-the-Loop" Validation Guide (On Any Other Machine + Real GitHub Actions Runner)

Here is how you can test and prove `minici` **100% by yourself on a second machine**, using a real GitHub repository and real GitHub Actions runners—**without relying on any AI claims**:

### Step 1: Install `minici` on the Second Machine (2 Mins)
```bash
git clone https://github.com/Aamir-raza-1/minici-cost-action.git
cd minici-cost-action
cargo build --release --manifest-path crates/minici/Cargo.toml
sudo cp crates/minici/target/release/minici /usr/local/bin/minici
minici --version
```

### Step 2: Create a Fresh Test Repo with 3 Workflow Files (5 Mins)
Create a new repo `my-ci-bill-test` on GitHub and add 3 real `.github/workflows/*.yml` files with known `sleep` durations so you know the exact real runtime down to the second:
1. `.github/workflows/unit.yml`: `runs-on: ubuntu-latest`, step runs `sleep 75` (`1m 15s` raw $\rightarrow$ GitHub bills **`2m`** ceiling).
2. `.github/workflows/matrix.yml`: `strategy.matrix.os: [ubuntu-latest, windows-latest, macos-14]`, step runs `sleep 40` (`0m 40s` raw $\rightarrow$ GitHub bills **`1m` each** across 3 OS runners).
3. `.github/workflows/minici-cost.yml`: generated by running `minici init-hooks --path .`.

### Step 3: Run `minici breakdown` Before Pushing & Write Down the Prediction (2 Mins)
```bash
minici breakdown --path . --customer-mode
```
* Look at the exact `Billed Mins / Run` and `Cost / Run` printed by `minici`.

### Step 4: Push to GitHub, Let the Real GitHub Actions Runners Execute & Modify a File in a PR (15 Mins)
1. Commit and push `main` to GitHub, then open the **GitHub Actions** tab and watch `unit.yml` and `matrix.yml` run on real GitHub-hosted VMs.
2. Create a branch ` git checkout -b test-matrix-spike`, edit `.github/workflows/matrix.yml` to add `node: [18, 20, 22]` (multiplying the matrix $3 \times$!), and open a Pull Request.
3. Watch two things happen on your real GitHub PR:
   * The **MiniCI Action** posts the sticky **PR Cost & Matrix Diff Receipt** directly on the PR within seconds.
   * Check your actual GitHub Billing / Workflow Run **Billable Time** tab on GitHub (`Settings -> Billing -> Usage` or the workflow run summary):
     * Verify that the `1m 15s` job was billed by GitHub as **`2m`** (1-minute ceiling rounding).
     * Verify that the $3 \times 3 = 9$ matrix combinations match `minici diff --path . --compare-to main` down to the exact minute and cent!

---

## 🌅 6. Tomorrow Morning's Plan: Feature 2 — Flaky Test & CI Retry Waste Detector (`minici flaky`)

When you start fresh tomorrow morning, we are building **Feature 2 ([`TOMORROW.md`](file:///Users/araza/personal/minici-factory-qa/TOMORROW.md))**:

### What Problem Feature 2 Solves:
When a developer clicks **"Re-run failed jobs"** on GitHub Actions (or `pytest-rerunfailures` / Playwright retries a flaky test), GitHub charges **100% full runner minutes + developer wait time** for every retry attempt (`run_attempt: 2, 3, 4`). Most teams silently waste **12%–25% of their monthly CI bill** on flaky retries without knowing which jobs or tests are the culprits.

### Exact Architecture We Will Build Tomorrow (`< 1 Hour`, Zero Wheel-Reinventing):
1. **Subcommand 1 — Workflow & Run-Attempt Retry Auditor (`minici flaky --repo <owner/repo>` or `--path .`):**
   * Queries recent workflow runs via the GitHub API (`/repos/{owner}/{repo}/actions/runs?per_page=100`, checking `run_attempt > 1`) OR parses JUnit XML test reports (`junit.xml` `<flakyFailure>` / `<rerunFailure>`).
2. **Quantifies Exact "Flaky Retry Tax ($/mo)":**
   * Multiplies each retried job's duration by its official SKU rate from `catalog.rs` (`$0.006/min` Linux, `$0.062/min` macOS, or Infracost cloud rate) + Developer Wait-Time burn.
3. **Outputs the "Top 5 Flaky Money-Pit Jobs" Table + 1-Line Auto-Quarantine Fix:**
   * Prints the top 5 flaky jobs/tests burning the most dollars per month and adds a collapsible **"🔥 Flaky Retry Waste Receipt"** section directly into the sticky PR comment!
