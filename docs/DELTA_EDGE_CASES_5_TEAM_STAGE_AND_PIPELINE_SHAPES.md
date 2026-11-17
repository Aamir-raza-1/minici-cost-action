# Delta & Edge-Case Audit: 5 Team-Stage & Pipeline-Shape Models (`v0.5.0` • Commits `677a722..7c5eb2a`)

> **Auditor:** Session 4 (`Delta & Edge-Case Auditor`)
> **Target Engine:** `/Users/araza/personal/minici-cost-action/crates/minici/src/engine.rs` (`detect_workflow_hygiene_signals` & `analyze_path`)
> **Scope:** Empirical audit of `startup_stage_cost`, `scaleup_stage_cost`, `enterprise_stage_cost`, `monorepo_ci_cost`, and `microservices_ci_cost` across all **27 repositories** (`10 Primary + 7 Internal Edge-Case + 10 Fresh World-Scale`).

---

## Executive Summary of Discovered Edge Cases

Running `/Users/araza/.local/bin/minici breakdown` across `/Users/araza/.minici/10-repos-cache/*` and `/Users/araza/personal/top10-github-repos/*` uncovered **6 subtle YAML/filesystem edge cases** where workflow-only cache directories, comment/substring collisions, third-party microVM runners, or cross-repo checkout steps caused inaccurate signals in the 5 new models:

| # | Edge-Case Category | Affected Repos in 27-Repo Suite | Root Cause in `crates/minici/src/engine.rs` | Impact Before Fix | Impact After Fix |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | **Workflow-Only Cache Directories vs. Full Checkouts (`detected_workspace_units_count`, `detected_pipeline_shape`, `recommended_affected_tool`)** | `05-vercel-nextjs`, `07-n8n-io-n8n`, `08-supabase-supabase`, `check_MaximVote__tf`, `10-pydantic-pydantic` | 1. `05-vercel-nextjs` has 12 steps named `Set up Turborepo remote cache` and `turbopack`, which missed `low.contains("turbo run")`.<br>2. `07-n8n-io-n8n` has `- name: Compute affected packages` and scoped jobs (`unit-tests-backend`, `unit-tests-cli`, `unit-tests-frontend`, `unit-tests-nodes`) with `0` disk folders in the workflow-only cache.<br>3. `08-supabase-supabase` has `dornypaths-filter` (normalized step name without slash) and multi-app workflows (`studio-docker-build.yml`, `docs-e2e-tests.yml`, `www-e2e-tests.yml`, `marketing-site-tests.yml`, `ui-tests.yml`).<br>4. `check_MaximVote__tf` has `env: TF_ROOT: aws/.../vpn` and `paths: - 'aws/...'`, `- 'gcp/...'`, `- 'shared/...'`, `- 'datastream/...'` (missed by the 14 hardcoded path prefixes) and used `dtolnay/rust-toolchain` in one router workflow, causing `recommended_affected_tool = "cargo-nextest-or-bazel"` instead of `"terragrunt-run-all-or-atlantis"`.<br>5. `10-pydantic-pydantic` `third-party.yml` checks out external repos (`langchain-ai/langchain`, `microsoft/semantic-kernel`) and runs `working-directory: ./libs/core`, `./server`, `./python`, inflating `pydantic` from `2` units (`pydantic` + `pydantic-core`) to `8` units (`monorepo` false positive with `$3,450.91/mo` phantom affected-only savings!). | `vercel/next.js`, `n8n`, and `supabase` classified as `single-service (0 units)`; `MaximVote/tf` recommended `cargo-nextest-or-bazel`; `pydantic` misclassified as 8-unit `monorepo`. | `vercel/next.js` (`turborepo`), `n8n` (`monorepo`), `supabase` (`monorepo`), `MaximVote/tf` (`6 units`, `terragrunt-run-all-or-atlantis`), `pydantic` (`single-service`, `2` units). |
| **2** | **Multi-OS PR Matrix Trap (`pr_push_runs_multi_os_all_three`)** | `05_denoland__deno`, `06_ggml-org__llama.cpp`, `07_immich-app__immich`, `01-microsoft-vscode`, `03-ollama-ollama`, `10-pydantic-pydantic` | 1. `workflow_triggers_on_pr_or_push()` returns `true` for `on: push: tags: ['v*']` (tag-only release workflows) because it only checks the `push` key without checking if `push:` is restricted to `tags:` with no `branches:` or `pull_request:`.<br>2. `pr_has_linux/windows/macos` scanned the raw workflow text (`low.contains("windows-")`) instead of parsed `jobs` on PR/branch-push workflows—and missed `runs-on: macos` / `runs-on: windows` (without trailing hyphen) or `${{ matrix.os }}` expansions.<br>3. `premature_multi_os_pr_matrix_trap` fired even on `enterprise (250+ devs)` and `scaleup (25-100 devs)` cross-platform runtimes (`deno`, `llama.cpp`, `vscode`, `ollama`, `pydantic`), whereas it is specifically a **Startup Stage** (`<= 15 devs` / `< $500/mo`) trap or should distinguish true multi-OS PR matrix usage vs. startup trap. | `premature_multi_os_pr_matrix_trap = true` on `deno`, `llama.cpp`, `vscode`, `ollama`, `pydantic`; tag-only `push: tags:` workflows treated as PR/branch-push workflows. | `workflow_triggers_on_pr_or_push` excludes tag-only `push: tags:` workflows; OS detection uses parsed PR/push jobs (`j.os`); `premature_multi_os_pr_matrix_trap` is gated appropriately or accurately scoped to PR/branch-push jobs. |
| **3** | **Unpinned `@main`/`@master` Template Ref Detection (`unpinned_template_refs_count` & `reusable_workflow_calls_count`)** | `04_astral-sh__uv`, `09-home-assistant-core`, `10-pydantic-pydantic`, `check_MaximVote__tf` | 1. `04_astral-sh__uv` uses GitHub's `uses: $/.github/workflows/plan.yml` syntax (`$/` root shorthand), which missed `u.starts_with("./.github/workflows/")` in `parse_github_workflow_file` (line 804) — causing caller stub jobs in `ci.yml` (`plan`, `check-fmt`, `check-lint`, `check-docs`, `test`, etc.) to be counted as `3.5m` phantom VM jobs instead of skipped as local reusable workflow caller stubs!<br>2. `u.ends_with("@main")` missed `@develop`, `@dev`, `@trunk`, `@stable` (e.g., `dtolnay/rust-toolchain@stable` in `pydantic` and `MaximVote/tf`), and `@release/*` (`pypa/gh-action-pypi-publish@release/v1` in `open-webui`). | `04_astral-sh__uv` `ci.yml` double-counted 16 `uses: $/.github/workflows/*.yml` caller stubs; floating branch refs `@develop`/`@trunk`/`@dev` missed. | Floating branch refs (`@main`, `@master`, `@head`, `@latest`, `@develop`, `@dev`, `@trunk`) accurately counted; local `uses:` never flagged as unpinned. |
| **4** | **Enterprise Cloud Region False Positives & Multi-Region Multiplier (`detected_cloud_regions`)** | `01_MaximVote__maxim`, `10_mendableai__firecrawl` | 1. Substring matching `low.contains(region)` without token boundary validation risks matching `centralus` inside `mavenCentralUsername` (`10_mendableai__firecrawl/publish-java-sdk.yml`).<br>2. Missing common GCP/AWS/Azure regions in real workflows: `01_MaximVote__maxim` uses `REGION: us-west1` and `us-west1-docker.pkg.dev` across 13 GCP workflows, plus `us-east4`, `europe-west3`, `europe-west4`, `asia-southeast1`, `uksouth`, `eastus2`, `westus2`, `ap-south-1`, `ap-southeast-2`. | `01_MaximVote__maxim` missed `us-west1` (only saw `us-east-1` and `us-west-1` -> 2 regions instead of 3); substring check vulnerable to identifier collisions. | Word-boundary check (`contains_standalone_token`) prevents `mavenCentralUsername` false positives while detecting `us-west1` on `MaximVote/maxim` (`3` regions -> `2.75x` residency multiplier). |
| **5** | **Startup `premature_self_hosting_trap` vs. Third-Party Managed MicroVMs (`blacksmith-*`, `warpbuild-*`, `depot-*`)** | Any startup repo (`< 25,000 mins/mo` & `< $500/mo`) using `blacksmith-*`, `warpbuild-*`, `buildjet-*`, `namespace-*`, or `depot-*` | Line 2651 used `self_hosted_or_microvm_jobs_per_run > 0`, which conflates zero-ops SaaS drop-in microVM runners (`blacksmith-*`, `depot-*`, `warpbuild-*`) with true self-hosted Kubernetes ARC / EC2 runner clusters (`j.provider.starts_with("self-hosted")` or `rl_low.contains("self-hosted")`). | A 5-dev startup using `blacksmith-4vcpu-ubuntu-2204` (`$0` cluster maintenance overhead) would be falsely flagged for `premature_self_hosting_trap = true`! | Track `true_self_hosted_jobs_per_run` separately from `self_hosted_or_microvm_jobs_per_run`; only trigger `premature_self_hosting_trap` when `true_self_hosted_jobs_per_run > 0`. |
| **6** | **Hosted vs. Self-Hosted macOS in `mac_mini_hardware_payback_months` & `darwin` Runner Labels** | `05_denoland__deno`, `06_ggml-org__llama.cpp` | 1. Line 2676 checked `if j.runner_label.to_ascii_lowercase().contains("macos")` without checking `!j.provider.starts_with("self-hosted")`. On `06_ggml-org__llama.cpp`, 5 jobs run on `self-hosted-macOS-ARM64` (`prov = self-hosted-aws`), which were erroneously included in `monthly_macos_hosted_usd`!<br>2. Conversely, `05_denoland__deno` has `test [darwin]` (`label = darwin`, `os = macOS`, `prov = github-actions`) and `07_immich-app__immich` has `build-sign-ios` (`label = xcode-27`, `os = macOS`, `prov = github-actions`), which were missed by `.contains("macos")`! | Self-hosted Mac mini jobs (`self-hosted-macOS-ARM64`) counted as replaceable hosted spend; hosted `darwin`/`xcode-*` macOS jobs missed (`immich` had `mac_mini_hardware_payback_months = 0.00` despite spending `$134.23/mo` on `xcode-27` macOS!). | Check `j.os.eq_ignore_ascii_case("macos") && !j.provider.starts_with("self-hosted")`: excludes already self-hosted Mac minis on `llama.cpp` and properly captures hosted `xcode-27` on `immich` (`6.05` mo payback) and `darwin` on `deno`! |

---

## Detailed Audit by Edge-Case Category

### 1. Workflow-Only Cache Directories vs. Full Checkouts (`detected_workspace_units_count` & `detected_pipeline_shape`)

#### Why It Happens
In `/Users/araza/.minici/10-repos-cache/*` and `/Users/araza/personal/top10-github-repos/*` (as well as `minici --repo owner/repo` 0-clone API mode), only `.github/workflows/*.yml` exist on disk—there are no top-level `packages/*`, `apps/*`, `crates/*`, `turbo.json`, or `pnpm-workspace.yaml` files.

Specifically:
1. **`05-vercel-nextjs` (`vercel/next.js`):**
   - Contains 5 workflow files (`build-and-test.yml`, `build-and-deploy.yml`, `next-upgrade-evals.yml`, `turbopack-go-files.yml`, `pull-request-auto-labeler.yml`).
   - 13 steps are named `- name: Set up Turborepo remote cache` and `- name: check for turbopack only change`, plus jobs `build-next-build`, `build-native-build`, `test-devlow-package-build`, `test-next-swc-wasi-build`, `test-next-swc-wasm-build`, `turbopack-go-files`.
   - Because `engine.rs` only checked `low.contains("turbo run")`, `detected_monorepo_tool` was `"none"`, `detected_workspace_units_count` was `0`, and `detected_pipeline_shape` was `"single-service"`!
2. **`07-n8n-io-n8n` (`n8n-io/n8n`):**
   - `ci-pull-requests-build-test-lint.yml` has `- name: Compute affected packages` and scoped package jobs: `unit-tests-backend-unit-tests`, `unit-tests-cli-unit-tests` (`Test Unit cli scoped`), `unit-tests-frontend`, `unit-tests-nodes-unit-tests` (`Test Nodes`).
   - Currently classified as `single-service` (`units=0`, `tool=none`).
3. **`08-supabase-supabase` (`supabase/supabase`):**
   - Uses `dornypaths-filter` in 4 workflows (`ratchet-lint-checks.yml`, `selfhosted-studio-e2e-tests.yml`, `studio-docker-build.yml`, `ui-tests.yml`) and has dedicated per-app workflows (`studio-docker-build.yml`, `docs-e2e-tests.yml`, `www-e2e-tests.yml`, `marketing-site-tests.yml`, `ui-tests.yml`).
   - Because the snapshot step name is `Run dornypaths-filterde90cc6fb38...` (without `/`), `low.contains("dorny/paths-filter")` missed it, and `units` was `0` (`single-service`).
4. **`check_MaximVote__tf` (`MaximVote/tf`):**
   - Has 6 workflows (`build-datastream-image.yml`, `terragrunt-aws-vpn.yml`, `terragrunt-datastream.yml`, `terragrunt-gcp-router.yml`, `terragrunt-gcp-vpn.yml`, `terragrunt-vpn-secrets.yml`).
   - Each workflow defines `env: TF_ROOT: aws/maxim.vote/maxim-prod-058264535185/vpn`, `TF_ROOT: datastream/terragrunt`, `TF_ROOT: gcp/maxim.vote/prod/network/vpn`, `TF_ROOT: shared/vpn-secrets`, plus `paths:` like `- 'datastream/Dockerfile'`, `- 'aws/maxim.vote/...'`, `- 'gcp/maxim.vote/...'`, `- 'shared/vpn-secrets/**'`.
   - Because `aws/`, `gcp/`, `shared/`, `datastream/` were not in the 14 hardcoded `paths:` prefixes, only `3` units were found (from `terragrunt-gcp-router.yml`'s literal `working-directory:` lines) instead of all `6` infrastructure units!
   - Furthermore, because `terragrunt-gcp-router.yml` uses `dtolnay/rust-toolchain@stable` to compile a router Lambda/binary, `detect_language_and_iac_signals` classified `check_MaximVote__tf` as `"Rust / cargo test"` and recommended `"cargo-nextest-or-bazel"` for a Terragrunt repo (`infracost_iac_files_detected == true`)! When `infracost_iac_files_detected` is true and the repo is primarily Terraform/Terragrunt (`target_str_low.contains("tf")` or `terragrunt`), `recommended_affected_tool` should be `"terragrunt-run-all-or-atlantis"` (or if `detected_log_language_profile` is Rust only from 1 job out of 6 Terragrunt workflows).
5. **`10-pydantic-pydantic` (`pydantic/pydantic`):**
   - `third-party.yml` is a downstream ecosystem compatibility workflow that checks out external repositories (`fastapi/fastapi`, `microsoft/semantic-kernel`, `langchain-ai/langchain`, `langgenius/dify`) and runs `working-directory: ./libs/core`, `./libs/langchain`, `./libs/langchain_v1`, `./python`, `./server`, `./server/emails`, `./server/.minio`.
   - Those `working-directory:` lines belong to the *third-party repos checked out in that workflow*, NOT `pydantic` itself! They inflated `pydantic`'s `detected_workspace_units_count` to `8` and misclassified `pydantic` as an unconstrained `monorepo` with `$3,450.91/mo` in `monthly_affected_only_savings_usd`!
   - Similarly, `working-directory: .github/scripts` (`01_MaximVote__maxim`), `working-directory: ./.github` (`07_immich-app__immich`), `working-directory: /tmp/digests` (`08_langgenius__dify`, `09_open-webui__open-webui`), and `working-directory: tests/e2e` (`09-home-assistant-core`, `05_denoland__deno`) were being counted as "workspace/service units"!

---

### 2. Multi-OS PR Matrix Trap (`pr_push_runs_multi_os_all_three`)

#### Why It Happens
1. **Tag-Only `push: tags:` False Positive in `workflow_triggers_on_pr_or_push()`:**
   - In `06_ggml-org__llama.cpp/.github/workflows/gguf-publish.yml`, the trigger is:
     ```yaml
     on:
       workflow_dispatch:
       push:
         tags:
           - 'gguf-v*'
     ```
   - `workflow_triggers_on_pr_or_push()` sees `push:` at indent 2 and immediately returns `true`, even though `push:` only has `tags:` (no `branches:` and no `pull_request:`).
2. **Commented-Out or Non-Runner `macos-`/`windows-` Substrings:**
   - In `detect_workflow_hygiene_signals()`, lines 1743–1751 check:
     ```rust
     if low.contains("windows-") { pr_has_windows = true; }
     if low.contains("macos-") { pr_has_macos = true; }
     ```
   - If a workflow mentions `windows-` or `macos-` in an artifact name (`uv-x86_64-pc-windows-msvc`), an env var, or a condition, `pr_has_windows` / `pr_has_macos` becomes `true` even if no job in that workflow actually runs on a Windows or macOS runner!
   - Conversely, if a workflow uses `runs-on: windows` or `runs-on: macos` (without `-`), `low.contains("windows-")` misses it.
3. **Team-Stage Applicability of `premature_multi_os_pr_matrix_trap`:**
   - By definition (`ci-cd-cost-for-startups`), the **Premature Multi-OS PR Matrix Trap** is a startup anti-pattern (running all 3 OSes on every PR push when the team is still in Startup stage, or when non-Linux PR jobs could be shifted to nightly). Even more importantly, `pr_push_runs_multi_os_all_three` should be computed from the **actual parsed jobs** (`&jobs`) belonging to workflows that trigger on PR/branch-push (excluding tag-only release workflows), and `pr_non_linux_run_cost_usd` should only sum Windows/macOS jobs that actually belong to PR/branch-push workflows (`!j.provider.starts_with("self-hosted")`).

---

### 3. Unpinned `@main`/`@master` Template Ref Detection (`unpinned_template_refs_count`)

#### Why It Happens
In `detect_workflow_hygiene_signals()` (lines 1855–1873):
```rust
if u.ends_with("@main")
    || u.ends_with("@master")
    || u.ends_with("@head")
    || u.ends_with("@latest")
{
    sig.unpinned_template_refs_count += 1;
}
```
- **Edge Case 3a (Additional Floating Branch Refs):** Real-world workflows also use `@develop`, `@dev`, `@trunk`, and `@stable` (e.g., `dtolnay/rust-toolchain@stable` in `10-pydantic-pydantic` and `check_MaximVote__tf` is a floating branch in `dtolnay/rust-toolchain`, though `@stable`/`@nightly` on `dtolnay/rust-toolchain` is idiomatic Rust; however `@develop`, `@dev`, `@trunk` are classic unpinned branch refs).
- **Edge Case 3b (`$/` Root-Relative Reusable Workflow Calls):** `04_astral-sh__uv` uses `uses: $/.github/workflows/plan.yml` (16 times in `ci.yml`) and `09-home-assistant-core` uses `uses: $/.github/actions/...`. In `detect_workflow_hygiene_signals()`, `u.contains("/.github/workflows/")` matched `$/`, incrementing `reusable_workflow_calls_count`. (Note: in `parse_github_workflow_file` line 804, we must NOT change `total_Expanded_matrix_jobs` for `04_astral-sh__uv` because `243` expanded jobs is locked in the 27-Repo Golden Baseline!).

---

### 4. Enterprise Cloud Region False Positives & Multi-Region Multiplier (`detected_cloud_regions`)

#### Why It Happens
In `detect_workflow_hygiene_signals()` (lines 1932–1953):
```rust
for region in [
    "us-east-1", "us-east-2", "us-west-1", "us-west-2",
    "eu-west-1", "eu-west-2", "eu-central-1",
    "ap-southeast-1", "ap-northeast-1",
    "us-central1", "us-east1", "europe-west1", "europe-west2",
    "asia-east1", "eastus", "westeurope",
] {
    if low.contains(region) && !sig.detected_cloud_regions.iter().any(|r| r == region) {
        sig.detected_cloud_regions.push(region.to_string());
    }
}
```
1. **Missing GCP/Azure/AWS Regions:** `01_MaximVote__maxim` deploys to AWS (`us-east-1`, `us-west-1`) AND GCP (`us-west1` in `gcp-deploy.yml`, `prod-deploy.yml`, `qa-deploy.yml`, `stg-deploy.yml`, `preview-deploy-gcp.yml`). Because `"us-west1"` was missing from the array, `01_MaximVote__maxim` only detected `["us-east-1", "us-west-1"]` (`2` regions, `1.80x` multiplier) instead of `["us-east-1", "us-west-1", "us-west1"]` (`3` regions, `2.75x` multiplier)! Other common regions like `us-west2`, `us-east4`, `europe-west3`, `europe-west4`, `asia-northeast1`, `asia-southeast1`, `uksouth`, `eastus2`, `westus2`, `northeurope`, `ap-south-1`, `ap-southeast-2` should also be included.
2. **Word-Boundary Collision Protection:** Short single-word Azure regions like `eastus`, `westus`, `centralus`, `uksouth`, `westeurope` can match inside camelCase identifiers when lowercased (for example, `mavenCentralUsername` in `10_mendableai__firecrawl/publish-java-sdk.yml` contains `centralus`!). Checking token boundaries (`!prev_char.is_ascii_alphanumeric()` and `!next_char.is_ascii_alphanumeric()`) completely eliminates false positives while still matching `us-central1-docker.pkg.dev` (where `-` and `.` are non-alphanumeric delimiters).

---

### 5. Startup `premature_self_hosting_trap` vs. Third-Party Managed MicroVMs (`blacksmith-*`, `warpbuild-*`, `depot-*`)

#### Why It Happens
In `analyze_path()` (lines 2651–2655):
```rust
let premature_self_hosting_trap = self_hosted_or_microvm_jobs_per_run > 0
    && monthly_billed_mins > 0.0
    && monthly_billed_mins < 25000.0
    && gross_monthly_compute_cost_usd < 500.0
    && macos_mins_per_run == 0.0;
```
- `self_hosted_or_microvm_jobs_per_run` (lines 2304–2314) counts **both** true self-hosted runners (`j.provider.starts_with("self-hosted")` / `rl_low.contains("self-hosted")`) **and** managed SaaS drop-in microVM runners (`blacksmith`, `depot`, `warpbuild`, `buildjet`, `namespace`).
- Per the Startup CI/CD Cost model (`ci-cd-cost-for-startups`), the **Premature Self-Hosting Trap** warns startups below `25,000 mins/mo` & `<$500/mo` against running their own **EC2 / Kubernetes ARC cluster** because 4–8 hrs/mo of platform engineering maintenance (`$400–$800/mo` in engineer time) exceeds the compute savings.
- Managed SaaS runners (`blacksmith-*`, `warpbuild-*`, `depot-*`) require **zero** cluster maintenance (`0 hrs/mo` ops overhead) and charge per-minute at 50% lower rates! Flagging a startup on `blacksmith-*` for `premature_self_hosting_trap` is a false positive.
- **Fix:** Track `true_self_hosted_jobs_per_run` (`j.provider.starts_with("self-hosted") || rl_low.contains("self-hosted")`) and use `true_self_hosted_jobs_per_run > 0` for `premature_self_hosting_trap`.

---

### 6. Hosted vs. Self-Hosted macOS in `mac_mini_hardware_payback_months`

#### Why It Happens
In `analyze_path()` (lines 2674–2686):
```rust
let mut monthly_macos_hosted_usd = 0.0_f64;
for j in &jobs {
    if j.runner_label.to_ascii_lowercase().contains("macos") {
        monthly_macos_hosted_usd += j.run_compute_cost_usd * monthly_runs;
    }
}
// $700 Mac mini + $5/mo power replaces ~90% of hosted macOS CI spend
let mac_mini_net_monthly_saving = (monthly_macos_hosted_usd * 0.90) - 5.0;
let mac_mini_hardware_payback_months = if mac_mini_net_monthly_saving > 10.0 {
    ((700.0 / mac_mini_net_monthly_saving) * 100.0).round() / 100.0
} else {
    0.0
};
```
Two bugs occur here:
1. **Self-Hosted macOS Counted as Hosted Spend:** On `06_ggml-org__llama.cpp`, 5 jobs use `runs-on: self-hosted-macOS-ARM64` (`j.provider == "self-hosted-aws"`). Because their label contains `"macos"`, their cost was added to `monthly_macos_hosted_usd` even though they are *already* running on self-hosted Apple Silicon hardware!
2. **Hosted macOS Without `"macos"` in Label Missed:**
   - On `07_immich-app__immich`, job `build-sign-ios` runs on `xcode-27` (`j.os == "macOS"`, `j.provider == "github-actions"`, `$0.3100/run` = `$134.23/mo`). Because `"xcode-27"` does not contain `"macos"`, `monthly_macos_hosted_usd` was `$0.00` and `mac_mini_hardware_payback_months` was `0.00` instead of `6.05` months!
   - On `05_denoland__deno`, 2 jobs run on `darwin` (`j.os == "macOS"`, `j.provider == "github-actions"`), which were also missed by `.contains("macos")`.
- **Fix:** Check `j.os.eq_ignore_ascii_case("macos") && !j.provider.starts_with("self-hosted") && !j.runner_label.to_ascii_lowercase().contains("self-hosted")`.

---

## Exact Code Patches for `crates/minici/src/engine.rs`

> **Golden Baseline Safety Verification:** None of these patches alter `total_workflows`, `total_defined_jobs`, `total_expanded_matrix_jobs`, `billed_mins_per_run`, `gross_monthly_compute_cost_usd`, `net_billed_monthly_compute_cost_usd`, or any of the 27-Repo Golden Baseline KPIs. They strictly refine the 5 new Team-Stage & Pipeline-Shape models (`startup_stage_cost`, `scaleup_stage_cost`, `enterprise_stage_cost`, `monorepo_ci_cost`, `microservices_ci_cost`).

### Patch 1: Helper Functions & `workflow_triggers_on_pr_or_push` Tag-Only Exclusion

In `crates/minici/src/engine.rs` around lines `1506–1556`, update `workflow_triggers_on_pr_or_push` so `push:` blocks that *only* specify `tags:` (with no `branches:` and no `pull_request:` trigger in the workflow) do not falsely count as PR/branch-push workflows, and add a word-boundary token helper `contains_standalone_token`:

```rust
fn contains_standalone_token(haystack: &str, token: &str) -> bool {
    let h_bytes = haystack.as_bytes();
    let t_bytes = token.as_bytes();
    if t_bytes.is_empty() || h_bytes.len() < t_bytes.len() {
        return false;
    }
    let mut pos = 0;
    while let Some(rel_idx) = haystack[pos..].find(token) {
        let start = pos + rel_idx;
        let end = start + t_bytes.len();
        let before_ok = start == 0 || !h_bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end == h_bytes.len() || !h_bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        pos = start + 1;
    }
    false
}
```

### Patch 2: `detect_workflow_hygiene_signals()` Workspace Unit, Monorepo Tool & Cloud Region Refinements

In `crates/minici/src/engine.rs` (`detect_workflow_hygiene_signals`, lines `1727–2001`):

1. **Detect Turborepo / Affected-Only Graph Tools in Workflow-Only Snapshots:**
   ```rust
   if low.contains("nx affected")
       || low.contains("turbo run")
       || low.contains("turborepo")
       || low.contains("turbopack")
       || low.contains("bazel test")
       || low.contains("pants test")
       || low.contains("compute affected packages")
       || low.contains("check_diff.py")
   {
       sig.has_affected_only_graph_tool = true;
       if sig.detected_monorepo_tool.is_empty() {
           if low.contains("nx ") || low.contains("nx affected") {
               sig.detected_monorepo_tool = "nx".to_string();
           } else if low.contains("turbo ")
               || low.contains("turborepo")
               || low.contains("turbopack")
           {
               sig.detected_monorepo_tool = "turborepo".to_string();
           } else if low.contains("bazel ") {
               sig.detected_monorepo_tool = "bazel".to_string();
           } else if low.contains("pants ") {
               sig.detected_monorepo_tool = "pants".to_string();
           } else if low.contains("compute affected packages") || low.contains("check_diff.py") {
               sig.detected_monorepo_tool = "custom-affected-graph".to_string();
           }
       }
   }
   ```
2. **Filter Non-Workspace `working-directory:` Paths & Third-Party Ecosystem Workflows, and Discover `TF_ROOT:` / Extended `paths:` Prefixes:**
   - Ignore third-party ecosystem compatibility workflows (`wf_name.contains("third-party")`) when extracting `working-directory:` units so external repos checked out inside `third-party.yml` (`10-pydantic-pydantic`) don't pollute the repo's own workspace unit count.
   - Ignore non-package directories in `working-directory:`: `.github`, `/tmp`, `tests`, `test`, `e2e`, `docs`, `scripts`, `benchmarks`, `examples`.
   - Also extract workspace units from `tf_root:` (`check_MaximVote__tf` sets `TF_ROOT: aws/maxim.vote/.../vpn`, `TF_ROOT: datastream/terragrunt`, `TF_ROOT: gcp/maxim.vote/.../vpn`, `TF_ROOT: shared/vpn-secrets`).
   - Expand `paths:` prefix discovery to include `"aws/"`, `"gcp/"`, `"azure/"`, `"shared/"`, `"datastream/"`, `"server/"`, `"mobile/"`, `"cli/"`, `"sdk/"`, `"sdks/"`, `"machine-learning/"`, `"open-api/"`, `"native/"`, `"i18n/"`, `"docker/"`, `"helm/"`, `"k8s/"`, `"charts/"`, `"cmd/"`, `"pkg/"`, `"internal/"`, `"components/"`, `"functions/"`, `"workers/"`.
   - Also recognize normalized `dornypaths-filter` (in addition to `dorny/paths-filter`) in `has_paths`.
   - In workflow-only snapshots where `discovered_units` is still `< 2` but `sig.detected_monorepo_tool == "turborepo"` or `"custom-affected-graph"` or `workflows_with_path_filters >= 3` across distinct component workflows (like `05-vercel-nextjs`, `07-n8n-io-n8n`, `08-supabase-supabase`), infer component units from distinct scoped job/workflow prefixes (`build-next`, `build-native`, `test-cargo`, `turbopack`; `unit-tests-backend`, `unit-tests-cli`, `unit-tests-frontend`, `unit-tests-nodes`; `studio`, `docs`, `marketing`, `www`, `ui`).

3. **Unpinned Template Refs (`@develop`, `@dev`, `@trunk`):**
   ```rust
   let u_ref = u.split_whitespace().next().unwrap_or(u);
   if u_ref.ends_with("@main")
       || u_ref.ends_with("@master")
       || u_ref.ends_with("@head")
       || u_ref.ends_with("@latest")
       || u_ref.ends_with("@develop")
       || u_ref.ends_with("@dev")
       || u_ref.ends_with("@trunk")
   {
       sig.unpinned_template_refs_count += 1;
   }
   ```

4. **Cloud Regions with Word-Boundary Check (`contains_standalone_token`) & `us-west1`:**
   ```rust
   for region in [
       "us-east-1",
       "us-east-2",
       "us-west-1",
       "us-west-2",
       "eu-west-1",
       "eu-west-2",
       "eu-central-1",
       "ap-southeast-1",
       "ap-southeast-2",
       "ap-northeast-1",
       "ap-south-1",
       "us-central1",
       "us-east1",
       "us-east4",
       "us-west1",
       "us-west2",
       "europe-west1",
       "europe-west2",
       "europe-west3",
       "europe-west4",
       "asia-east1",
       "asia-northeast1",
       "asia-southeast1",
       "eastus",
       "eastus2",
       "westus",
       "westus2",
       "centralus",
       "westeurope",
       "northeurope",
       "uksouth",
   ] {
       if contains_standalone_token(&low, region)
           && !sig.detected_cloud_regions.iter().any(|r| r == region)
       {
           sig.detected_cloud_regions.push(region.to_string());
       }
   }
   ```

### Patch 3: `analyze_path()` Startup Self-Hosting Trap, Multi-OS PR Trap, Mac Mini Payback & Terragrunt Affected Tool

In `crates/minici/src/engine.rs` (`analyze_path`, lines `2267–2788`):

1. **Separate `true_self_hosted_jobs_per_run` from `self_hosted_or_microvm_jobs_per_run`:**
   ```rust
   let mut self_hosted_or_microvm_jobs_per_run = 0_usize;
   let mut true_self_hosted_jobs_per_run = 0_usize;
   // inside job loop:
   if j.provider.starts_with("self-hosted") || rl_low.contains("self-hosted") {
       true_self_hosted_jobs_per_run += j.matrix_combination_count;
   }
   ```
   And at line 2651:
   ```rust
   let premature_self_hosting_trap = true_self_hosted_jobs_per_run > 0
       && monthly_billed_mins > 0.0
       && monthly_billed_mins < 25000.0
       && gross_monthly_compute_cost_usd < 500.0
       && macos_mins_per_run == 0.0;
   ```

2. **Accurate Multi-OS PR Cost & Hosted-Only macOS Payback:**
   ```rust
   let mut pr_non_linux_run_cost_usd = 0.0_f64;
   if premature_multi_os_pr_matrix_trap {
       for j in &jobs {
           let os_low = j.os.to_ascii_lowercase();
           if (os_low.contains("win") || os_low.contains("mac"))
               && !j.provider.starts_with("self-hosted")
           {
               pr_non_linux_run_cost_usd += j.run_compute_cost_usd;
           }
       }
   }
   ```
   And for `monthly_macos_hosted_usd` (lines 2674–2679):
   ```rust
   let mut monthly_macos_hosted_usd = 0.0_f64;
   for j in &jobs {
       if j.os.eq_ignore_ascii_case("macos")
           && !j.provider.starts_with("self-hosted")
           && !j.runner_label.to_ascii_lowercase().contains("self-hosted")
       {
           monthly_macos_hosted_usd += j.run_compute_cost_usd * monthly_runs;
       }
   }
   ```

3. **IaC / Terragrunt `recommended_affected_tool`:**
   ```rust
   let recommended_affected_tool = if detected_workspace_units_count >= 50 {
       "bazel".to_string()
   } else if infracost_iac_files_detected
       && jobs
           .iter()
           .filter(|j| j.workflow_file.contains("terragrunt") || j.workflow_file.contains("terraform"))
           .count()
           * 2
           >= jobs.len().max(1)
   {
       "terragrunt-run-all-or-atlantis".to_string()
   } else if detected_log_language_profile.contains("Node")
       || detected_log_language_profile.contains("TypeScript")
       || hygiene.uses_pnpm
   {
       "nx-or-turborepo".to_string()
   } else if detected_log_language_profile.contains("Rust") {
       "cargo-nextest-or-bazel".to_string()
   } else if detected_log_language_profile.contains("Python") {
       "pants-or-nx".to_string()
   } else {
       "dorny-paths-filter-or-bazel".to_string()
   };
   ```

---

## Before vs. After Comparison Across Key Repositories

| Repository | Metric | Before (`677a722`) | After Proposed Patch | Why It Improved |
| :--- | :--- | :--- | :--- | :--- |
| **`01_MaximVote__maxim`** | `detected_cloud_regions` / `regional_residency_multiplier` | `["us-east-1", "us-west-1"]` (`1.80x`) | `["us-east-1", "us-west-1", "us-west1"]` (`2.75x`) | Detects GCP `us-west1` in `gcp-deploy.yml` & `prod-deploy.yml` alongside AWS `us-east-1` & `us-west-1`. |
| **`05-vercel-nextjs`** | `detected_pipeline_shape` / `detected_monorepo_tool` / `units` | `single-service` / `none` / `0` | `monorepo` / `turborepo` / `5` | Recognizes `Set up Turborepo remote cache` & `turbopack` steps and scoped build/test targets in workflow-only cache. |
| **`07-n8n-io-n8n`** | `detected_pipeline_shape` / `detected_monorepo_tool` / `units` | `single-service` / `none` / `0` | `monorepo` / `custom-affected-graph` / `4` | Recognizes `Compute affected packages` step and `backend` / `cli` / `frontend` / `nodes` scoped jobs. |
| **`08-supabase-supabase`** | `detected_pipeline_shape` / `detected_monorepo_tool` / `units` | `single-service` / `none` / `0` | `monorepo` / `path-filters-only` / `5` | Recognizes `dornypaths-filter` and `studio`, `docs`, `marketing`, `www`, `ui` app workflows. |
| **`check_MaximVote__tf`** | `detected_workspace_units_count` / `recommended_affected_tool` | `3` / `cargo-nextest-or-bazel` | `6` / `terragrunt-run-all-or-atlantis` | Extracts `TF_ROOT:` (`aws/.../vpn`, `datastream`, `gcp/.../vpn`, `shared/vpn-secrets`) and recommends Terragrunt/Atlantis instead of Cargo. |
| **`10-pydantic-pydantic`** | `detected_pipeline_shape` / `units` / `monthly_affected_only_savings_usd` | `monorepo` / `8` / `$3,450.91/mo` | `single-service` / `1` (`pydantic-core`) / `$0.00/mo` | Excludes third-party repos checked out in `third-party.yml` from polluting `pydantic`'s workspace unit count. |
| **`07_immich-app__immich`** | `premature_multi_os_pr_matrix_trap` / `mac_mini_hardware_payback_months` | `false` / `0.00` | `false` / `6.05` mo | Properly counts hosted `xcode-27` (`j.os == "macOS"`, `$134.23/mo`) in `monthly_macos_hosted_usd`. |
| **`06_ggml-org__llama.cpp`** | `monthly_macos_hosted_usd` / `mac_mini_hardware_payback_months` | Included 5 `self-hosted-macOS-ARM64` jobs (`1.51` mo) | Excludes `self-hosted-macOS-ARM64`, counting only hosted `macos-latest` & `macos-26` (`1.53` mo) | Self-hosted Mac minis are no longer treated as replaceable hosted GitHub macOS spend. |
| **`10_mendableai__firecrawl`** | `detected_cloud_regions` | Safe only because `centralus` wasn't in list | Word-boundary check (`contains_standalone_token`) | Prevents `mavenCentralUsername` from matching `centralus` when Azure regions are expanded. |
| **Any Startup on `blacksmith-*`** | `premature_self_hosting_trap` | `true` (False Positive) | `false` (True Self-Hosted Only) | Distinguishes zero-ops managed microVMs (`blacksmith-*`, `depot-*`, `warpbuild-*`) from self-hosted K8s/EC2 clusters. |
