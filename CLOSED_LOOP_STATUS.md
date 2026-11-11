# 🔒 MiniCI (`Skipwise` • `taste-ci-1`) — Closed-Loop Status, Golden Paths & Next-Days Roadmap

> [!IMPORTANT]
> **Live Cloudflare Worker Website Deployed:** **https://minici-cost-action.raza-aamir01.workers.dev**  
> Includes the interactive **"👁️ Owner Debug View: ON (Click for Customer View)"** toggle button in the top navbar, plus `/customer-diagram.svg`, `/launch-video.svg`, and `/health`.

---

## 1. Two Cleanly Separated Standalone Git Repositories (Never Mixed with `ci-casestudies`)

| Repository | Local Path | Git Branch & Pre-Commit Gate | What Lives Inside |
| :--- | :--- | :--- | :--- |
| **Repo 1: Customer Product, CLI, GitHub Action & Website** | [`/Users/araza/personal/minici-cost-action`](file:///Users/araza/personal/minici-cost-action) | `main` (`91c942c`)<br/>`.git/hooks/pre-commit` runs `cargo test` + `minici breakdown --quiet` | • [`crates/minici/`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/main.rs) (`573 KB` Pure Rust source)<br/>• [`action.yml`](file:///Users/araza/personal/minici-cost-action/action.yml) & [`.github/workflows/minici-cost.yml`](file:///Users/araza/personal/minici-cost-action/.github/workflows/minici-cost.yml)<br/>• [`README.md`](file:///Users/araza/personal/minici-cost-action/README.md) (embeds [`web/launch-video.svg`](file:///Users/araza/personal/minici-cost-action/web/launch-video.svg) & [`web/customer-diagram.svg`](file:///Users/araza/personal/minici-cost-action/web/customer-diagram.svg) at the top!)<br/>• [`web/index.html`](file:///Users/araza/personal/minici-cost-action/web/index.html) (with **Owner Debug View ON/OFF** toggle)<br/>• [`.claude/settings.json`](file:///Users/araza/personal/minici-cost-action/.claude/settings.json) (`wozcode` + `PreToolUse`/`PostToolUse`) & [`.mcp.json`](file:///Users/araza/personal/minici-cost-action/.mcp.json)<br/>• [`wrangler.jsonc`](file:///Users/araza/personal/minici-cost-action/wrangler.jsonc) & [`worker.js`](file:///Users/araza/personal/minici-cost-action/worker.js) |
| **Repo 2: Internal Factory, QA Checklists & Oracles** | [`/Users/araza/personal/minici-factory-qa`](file:///Users/araza/personal/minici-factory-qa) | `main`<br/>`.git/hooks/pre-commit` runs `minici self-update --quiet` | • [`CLOSED_LOOP_STATUS.md`](file:///Users/araza/personal/minici-factory-qa/CLOSED_LOOP_STATUS.md), [`QA_CHECKLIST.md`](file:///Users/araza/personal/minici-factory-qa/QA_CHECKLIST.md), [`TASK_LOG.md`](file:///Users/araza/personal/minici-factory-qa/TASK_LOG.md), [`COMPLETION.md`](file:///Users/araza/personal/minici-factory-qa/COMPLETION.md)<br/>• [`FACTORY_FROM_SCRATCH.md`](file:///Users/araza/personal/minici-factory-qa/FACTORY_FROM_SCRATCH.md), [`MANUAL_HOW_TO_RUN.md`](file:///Users/araza/personal/minici-factory-qa/MANUAL_HOW_TO_RUN.md)<br/>• [`MONDAY.md`](file:///Users/araza/personal/minici-factory-qa/MONDAY.md)..[`SATURDAY.md`](file:///Users/araza/personal/minici-factory-qa/SATURDAY.md), [`TODAY.md`](file:///Users/araza/personal/minici-factory-qa/TODAY.md), [`TOMORROW.md`](file:///Users/araza/personal/minici-factory-qa/TOMORROW.md)<br/>• [`scripts/verify_5_oracles.py`](file:///Users/araza/personal/minici-factory-qa/scripts/verify_5_oracles.py), [`scripts/independent_calculator_audit.py`](file:///Users/araza/personal/minici-factory-qa/scripts/independent_calculator_audit.py)<br/>• [`pricing/`](file:///Users/araza/personal/minici-factory-qa/pricing/github.json) & [`tools/pricing_watch.py`](file:///Users/araza/personal/minici-factory-qa/tools/pricing_watch.py) |

---

## 2. All 8 Golden Paths — Verified Working End-to-End (`Exit Code 0`)

| Golden Path | Exact Command to Run | Verified Output & Latency | Status |
| :--- | :--- | :--- | :--- |
| **GP-1: Owner / Internal Verbose Debug View (Default ON for You)** | `minici breakdown --path .` | Displays all 8 `[ON] feat.*` toggles, `infracost version v2.19.0` delegation, and `[MATH TRACE]` equations in **`0.13 ms`** | ✅ **WORKING** |
| **GP-2: Customer Launch View (Hides Owner Internal Flags)** | `minici breakdown --path . --customer-mode` | Hides the `OWNER INTERNAL DEBUG & FEATURE FLAGS` panel cleanly for external customers in **`0.10 ms`** | ✅ **WORKING** |
| **GP-3: Non-Interactive Quiet Mode (`--quiet` / `-q`)** | `minici breakdown --path . --quiet` | Prints 1-line receipt: `minici: $6.93/mo ($0.016/run) \| Quota: 43.3% \| DevWait: $796.72/mo \| 1 workflows (1 expanded jobs) in 0.12 ms` | ✅ **WORKING** |
| **GP-4: Zero-Clone Public GitHub URL Audit + `< 1.5 ms` Cache** | `minici breakdown --repo https://github.com/vercel/next.js` | Fetches only `.github/workflows/*.yml` via GitHub API on 1st run and hits `~/.minici/cache/repos/` in **`1.46 ms`** on 2nd run | ✅ **WORKING** |
| **GP-5: Infracost Cloud Instance Engine (Zero Custom vCPU Guessing)** | `cargo test --manifest-path crates/minici/Cargo.toml` | Uses [`INFRACOST_CLOUD_CATALOG`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L515-L580) (`infracost v2.19.0` schema) for AWS `c6i.12xlarge` (`48 vCPUs`), `c6i.24xlarge` (`96 vCPUs`), GCP `n2-standard-48` (`48 vCPUs`), Azure `Standard_D16s_v5` (`16 vCPUs`) | ✅ **WORKING** |
| **GP-6: Zero-Break Self-Update & Live Pricing Override** | `minici self-update` / `minici catalog set <sku> <rate>` | Syncs `71` SaaS + Infracost Cloud SKUs into `~/.minici/pricing_overrides.json` (`OnceLock` `< 1 µs` memory cache) while preserving custom user rates | ✅ **WORKING** |
| **GP-7: Multi-Format Executive Reports (`.html`, `.json`, `.csv`, `.md`)** | `minici --report report.html` | Auto-detects format from file extension, verifies disk write, and generates standalone dark-mode HTML/JSON/CSV/MD reports | ✅ **WORKING** |
| **GP-8: Claude Code + Wozcode Token-Savings Hooks & MCP** | `minici init-hooks --path .` | Writes `.claude/settings.json` (`woz@wozcode-marketplace`, `PreToolUse`, `PostToolUse`), `.mcp.json`, and `.github/workflows/minici-cost.yml` | ✅ **WORKING** |

---

## 3. Claude Code Independent Review (`claude -p`) — All Findings Fixed & Verified

When you activated `/opt/homebrew/bin/claude`, we ran an independent code review of [`catalog.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs) and [`main.rs`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/main.rs) and fixed every finding:
1. **Rely 100% on Infracost for Cloud Instances (`INFRACOST_CLOUD_CATALOG`):** Removed our custom vCPU string-guessing heuristic and replaced it with [`lookup_infracost_cloud_spec`](file:///Users/araza/personal/minici-cost-action/crates/minici/src/catalog.rs#L575-L581) backed by Infracost `v2.19.0` instance specs (`c6i.12xlarge` = 48 vCPUs, `c6i.24xlarge` = 96 vCPUs, `n2-standard-48` = 48 vCPUs, `n2-standard-2` = 2 vCPUs, `c6i.large` = 2 vCPUs, `Standard_D16s_v5` = 16 vCPUs).
2. **Multi-CI Vendor Precedence:** `bitbucket-16x`, `saas-linux-xlarge-amd64`, `docker-xlarge`, `BUILD_GENERAL1_LARGE`, and `macos-15-xlarge` (`$0.160/min`) now resolve to their exact vendor family before generic size labels.
3. **In-Memory `OnceLock` Override Cache (`< 1 µs`):** `~/.minici/pricing_overrides.json` is loaded once into memory via `std::sync::OnceLock` instead of reading from disk on every lookup.
4. **Custom Override Preservation & Safe Self-Copy in `minici self-update`:** `self-update` preserves custom rates set via `minici catalog set` and checks `.canonicalize()` before copying the binary to `~/.local/bin/minici`.

---

## 4. Optional 10-Second GitHub Remote Push (Whenever You Want)

Both repositories are committed locally with pre-commit hooks installed. To push them to GitHub under `Aamir-raza-1`:

```bash
gh auth login -h github.com
gh repo create Aamir-raza-1/minici-cost-action --public --source=/Users/araza/personal/minici-cost-action --push
gh repo create Aamir-raza-1/minici-factory-qa --private --source=/Users/araza/personal/minici-factory-qa --push
```

---

## 5. Clear Day-by-Day Roadmap (Stop Today & Resume Tomorrow on Feature 2)

| Day | Focus | Deliverable | Status |
| :--- | :--- | :--- | :--- |
| **Today (Day 1 — CLOSED)** | **Feature 1: Pre-Merge CI/CD Matrix Cost, Quota & Dev Wait-Time Receipt (`minici`)** | `573 KB` Pure Rust CLI + Infracost Cloud Engine + Owner/Customer View Toggle + 2 Standalone Git Repos + Live Cloudflare Worker (`minici-cost-action.raza-aamir01.workers.dev`) | ✅ **100% DONE — STOP HERE** |
| **Tomorrow (Day 2)** | **Feature 2: Flaky Test & CI Retry Waste Detector (`minici flaky`)** | Detects jobs that pass on re-run (`run_attempt > 1`), quantifies monthly `$` burned on flaky retries, and posts a "Top 5 Flaky Money Pit Jobs" section in the PR receipt | ⏭️ **Ready for Tomorrow ([`TOMORROW.md`](file:///Users/araza/personal/minici-factory-qa/TOMORROW.md))** |
| **Day After Tomorrow (Day 3)** | **Feature 3: Path-Filter & Skip-Redundant-Matrix Auto-Patcher (`minici optimize`)** | Generates ready-to-apply `.github/workflows/*.yml` diffs (`paths-ignore`, `concurrency.cancel-in-progress`, and ARM64/Blacksmith runner swaps) | 📅 **Scheduled for Day 3 ([`WEDNESDAY.md`](file:///Users/araza/personal/minici-factory-qa/WEDNESDAY.md))** |
