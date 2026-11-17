# Delta & Edge-Case Audit: 4 Calculator & Stack Intelligence Pillars (`v0.6.0` • Commits `b9f35f2..5e04008`)

**Audited Files:**
- `crates/minici/src/engine.rs` (lines `987–1031`, `1435–2435`, `3157–3700`)
- `crates/minici/src/catalog.rs` (lines `167–310`, `784–1160`)
- `crates/minici/src/renderer.rs` (lines `300–395`, `529–598`, `685–783`)
- Reference: `docs/cicdcost_reference/ALL_CALCULATOR_AND_STACK_MODELS_VERIFIED_CONTENT.md`
- Golden Baseline Lock: `tests/golden_baseline_10_repos.json` (`27` repos × `16` locked prior components)

---

## Executive Summary of Discovered Deltas & Edge Cases

| # | Pillar | Severity | Edge Case / Mathematical Delta | Impact if Unfixed |
| :--- | :--- | :---: | :--- | :--- |
| **D1** | `compute_class_explorer` & `vendor_tco_comparison` | **Critical** | `is_gh_larger` checks `j.vcpus > 2`, which falsely classifies standard `windows-latest` (`4 vCPU`) and `macos-latest` (`3 vCPU`) as GitHub Larger Runners. | 1) Double-charges Windows & macOS minutes in `gh_tco` (`gh_std_overage + larger_runner_monthly_usd`).<br>2) Falsely prints `larger runner job(s) ... never draw free mins` in CLI/Markdown/HTML on repos using only standard `windows-latest` / `macos-latest`. |
| **D2** | `vendor_tco_comparison` | **High** | `current_tco` is hardcoded to `gh_tco.max(gross_monthly_compute_cost_usd)` instead of reflecting the repo's actual `primary_provider` (`gitlab-ci`, `circleci`, `bitbucket`, `aws-codebuild`, `jenkins`, etc.), and treats `gross_monthly_compute_cost_usd > gh_tco` on a GitHub repo as a paid migration to `github-actions-team`. | 1) On non-GitHub repos (`.gitlab-ci.yml`, `Jenkinsfile`, `.circleci/config.yml`), savings from migrating to a cheaper vendor are miscalculated.<br>2) On GitHub Actions repos where Team's 3,000 free mins make `gh_tco < gross_monthly_compute_cost_usd`, it recommends a `44h` migration to `github-actions-team` when the repo is *already* on GitHub Actions. |
| **D3** | `vendor_tco_comparison` | **High** | `Jenkinsfile` and `.buildkite/pipeline.yml` are not parsed as source CI configs, and migration hours use a flat `40h + 4h/wf` formula instead of path-aware calculation (`60h + 6h/wf` from Jenkins; `50h + 5h/wf` to Buildkite BYO; `0h` when `best_vendor == current_vendor`). | Misses Jenkins-sourced repos and underestimates Jenkins migration effort (`132h` / `$13,200` for 12 pipelines in `calc_migration.html` lines `937–970`). |
| **D4** | `vendor_tco_comparison` | **Medium** | Bitbucket Pipelines (`bitbucket-premium`) has **no hosted macOS runner** (`n/a hosted`, `calc_comparison.html` lines `51, 145`), yet `candidates` allows `bitbucket-premium` to win `cheapest_vendor_platform` on macOS/iOS workloads. | Violates reference guidance (*"Avoid Bitbucket Pipelines for iOS-heavy work; it doesn't offer native macOS runners"*). |
| **D5** | `vendor_tco_comparison` | **Medium** | CircleCI credit math (`cci_credits`) assumes all Linux minutes burn `10 credits/min` (2-core Medium) regardless of job `vcpus`, and Buildkite Free tier checks `peak_concurrent_jobs_per_run <= 3` instead of the published `<= 10 concurrent jobs` (`calc_comparison.html` line `55`, `calc_ios.html` line `523`). | Understates CircleCI credit burn on `4/8/16-core` Linux jobs and overstates Buildkite seat/agent costs on small teams (`<= 5 devs, <= 10 jobs`). |
| **D6** | `build_phase_breakdown` | **High** | 1) Shallow-clone detection (`trimmed.ends_with('1')`) fails on quoted `fetch-depth: "1"` / `'1'`, false-positives on `fetch-depth: 21`, and uses `.all()` instead of job-weighted ratio.<br>2) `hygiene.has_deploy_or_release_steps` is detected at line `1912` but never used.<br>3) Rust/Java/C++ repos with Docker steps drop `compile_typecheck` share from `30%` to `18%`. | Inaccurate phase shares and reducible clone/compile savings; must also enforce an exact floating-point sum guard (`sum == 100.0`). |
| **D7** | `compute_class_explorer` | **High** | 1) Missing `32-core` (`$0.082/min`, `1.84x` balanced speedup) and `64-core` (`$0.162/min`, `1.89x` balanced speedup) Amdahl projections.<br>2) `std_2c_mo` uses raw `gross_monthly_compute_cost_usd` without normalizing jobs that *already* run on `4/8/16/32/64-core` runners, multiplying their larger-runner rate a second time.<br>3) On Free-tier / public repos, moving from free standard runners (`$0` net) to larger runners (always metered from 1st second) must compare against actual net billed baseline.<br>4) GPU larger runners (`Linux 4-core GPU` @ `$0.052/min`, `Windows 4-core GPU` @ `$0.102/min`) are missing in `resolve_runner_sku`. | Over-projects `4/8/16-core` costs by up to `7x` on repos already using larger runners, omits `32/64-core` and GPU SKUs, and understates the free-to-metered jump on Free-tier repos. |
| **D8** | `stack_specific_economics` | **High** | 1) `else if detected_log_language_profile.contains("iOS") || macos_mins_per_run > 0.0` is evaluated first, so cross-platform Rust/Go/Python/Node repos (e.g., `astral-sh/uv`, `denoland/deno`, `ollama/ollama`) with even 1 `macos-latest` job are misclassified as `"ios-macos-xcode"` (`xcode-deriveddata-spm-cache`).<br>2) Missing `flutter-dart-mobile` (`pubspec.yaml` / `flutter test`), `cpp-cmake-ninja` (`CMakeLists.txt` / `llama.cpp` / `ccache`), and `gradle-multi-module` (`settings.gradle` / `settings.gradle.kts`) detection.<br>3) `ios_buildkite_mac_mini_fleet_monthly_tco_usd` uses `$30/mini` instead of `$25/mini` (`$899 / 36 mos`, `calc_ios.html` lines `521, 558, 579, 600`). | Misclassifies cross-platform CLI/backend repos as iOS Xcode apps, and misclassifies C++ (`llama.cpp`) and Flutter (`flutter/flutter`) as `"node-typescript-vitest"`. |

> [!IMPORTANT]
> **Golden Baseline Preservation Constraint (`0.00%` KPI Deviation on Locked Fields):**
> `tests/golden_baseline_10_repos.json` locks `c5_datadog_90d_usd`, `h3_unrealized_cache_savings_usd`, `s2_recoverable_ci_and_wait_usd`, and `s3_siem_1yr_retention_usd`, which depend on `detect_language_and_iac_signals()`. Therefore, **do not alter the return values of `detect_language_and_iac_signals()`**. All new stack/build signals (`has_flutter_dart`, `has_cpp_cmake`, `has_gradle_multi_module`, `has_jenkinsfile`) must live in `WorkflowHygieneSignals` and feed Pillars 17–20 directly.

---

## Detailed Audit by Pillar & Exact Code Improvements

### 1. Pillar 17: `vendor_tco_comparison`

#### 1.1 False-Positive Larger Runner Classification & Double-Counting in `gh_tco`
In `crates/minici/src/engine.rs` (lines `3166–3191`):
- `windows-latest` has `vcpus = 4` (`catalog.rs` line `272`) and `macos-latest` has `vcpus = 3` (`catalog.rs` line `286`). Both are **standard** GitHub-hosted runners that draw from plan-included minutes (`free_tier_mins_per_month: 2000.0`).
- Because `is_gh_larger` checks `j.vcpus > 2`, every standard `windows-latest` and `macos-latest` job is treated as a GitHub Larger Runner (`larger_runners_detected_count += ...`, `larger_runner_monthly_usd += ...`).
- Then at lines `3229–3232`:
  ```rust
  let gh_std_overage = (monthly_std_linux_mins - 3000.0).max(0.0) * 0.006
      + (monthly_win_mins * 0.010)
      + (monthly_mac_mins * 0.062);
  let gh_tco = ((seats_f * 4.0 + gh_std_overage + larger_runner_monthly_usd) * 100.0).round() / 100.0;
  ```
  `monthly_win_mins` and `monthly_mac_mins` include all Windows/macOS jobs, while `larger_runner_monthly_usd` *also* includes them—charging Windows and macOS twice in `gh_tco`!

**Exact Fix:**
Classify a GitHub runner as a **GitHub Larger Runner** strictly when it is an actual GitHub-hosted non-standard SKU (never draws free minutes per `calc_larger_runners.html` lines `280–397`):
- Linux x64 larger (`vcpus > 2`, e.g., `4/8/16/32/64/96-core`),
- Linux arm64 larger (`2/4/8/16/32/64-core` — note: per `calc_larger_runners.html` lines `292–295` & `427–436`, GitHub arm64 runners are part of the larger-runner SKU family and never draw free minutes),
- Windows larger (`vcpus > 4`, or explicitly `windows-4core` / `windows-4-core`, or `windows-11-arm`),
- macOS larger (`macos-large`, `macos-xlarge`, `macos-14-large`, `macos-15-large`, `macos-15-xlarge`, `macos-26-xlarge`, `m2-pro` where `vcpus > 4`),
- GPU larger (`gpu` in label).
And separate standard vs. larger minutes by OS so `gh_std_overage` only bills standard Linux/Windows/macOS minutes!

#### 1.2 `current_tco`, Source Provider Awareness, Jenkinsfile Support & Migration Hours
In `crates/minici/src/engine.rs` (lines `3241–3330`):
1. **CircleCI Credit Math (`calc_comparison.html` lines `40–45`, `calc_ios.html` lines `477–485`, `calc_android.html` lines `710–713`):**
   - Linux credits scale with vCPUs (`10 credits/min` per 2 vCPUs: `2-core = 10`, `4-core = 20`, `8-core = 40`, `16-core = 80`).
   - Windows Medium (`4-core`) = `40 credits/min`.
   - macOS M4 Pro Medium = `200 credits/min` (`$0.120/min` at `$0.0006/credit`).
   - Free plan includes `30,000 credits/mo` (`$0` for `<= 5` users if total credits `<= 30,000`); Performance plan charges `$15/user/mo` (with 5 users included in the `$15` base or `$15/seat` past 5 users) plus `(cci_credits - 30_000.0).max(0.0) * 0.0006`.
2. **Bitbucket Pipelines macOS Disqualification (`calc_comparison.html` lines `51, 145`):**
   - Bitbucket Pipelines has **no native hosted macOS runners** (`n/a hosted`). When `monthly_mac_mins > 0.0`, `bitbucket-premium` should not be selected as `cheapest_vendor_platform` ahead of platforms that natively support the workload.
3. **Buildkite Free Tier & Agent Sizing (`calc_comparison.html` line `55`, `calc_ios.html` lines `523, 558–610`):**
   - Buildkite Free covers `<= 5 users` and `<= 10 concurrent jobs` (currently `<= 3` in `engine.rs` line `3264`).
   - Agent count should scale by sustained monthly volume (`(monthly_billed_mins / 8500.0).ceil().max(1.0)`) bounded by `peak_concurrent_jobs_per_run`, rather than assuming every concurrent matrix leg requires a dedicated 24/7 monthly VM (`peak_concurrent_jobs_per_run.min(16.0)`).
4. **Actual `current_tco` & Migration Path Hours (`calc_migration.html` lines `937–1007`, `1047–1049`):**
   - Map `primary_provider` to its corresponding vendor TCO (`"github-actions" => gh_tco`, `"gitlab-ci" => gl_tco`, `"circleci" => cci_tco`, `"bitbucket" => bb_tco`, `"aws-codebuild" => cb_tco`, `"gcp-cloud-build" => gcp_tco`, `"azure-pipelines" => az_tco`, `"jenkins" => jenkins_tco`, `"buildkite" => bk_tco`).
   - If `best_vendor` matches the repo's current provider (e.g., `primary_provider == "github-actions"` and `best_vendor == "github-actions-team"`), `monthly_tco_saving = 0.0`, `mig_hours = 0.0`, `mig_cost_usd = 0.0`, `mig_payback_mo = 0.0`, `mig_worth_2x = false`.
   - When migrating from Jenkins (`primary_provider == "jenkins"` or `hygiene.has_jenkinsfile`): use `60.0 + (workflow_count as f64) * 6.0` hours (`calc_migration.html` lines `948–958`: `setup_hours: 60`, `per_pipeline_hours: 6`).
   - When migrating to `buildkite-pro-byo`: use `50.0 + (workflow_count as f64) * 5.0` hours (standing up BYO agent fleet, `calc_migration.html` lines `1003–1007`).
   - Otherwise (YAML-to-YAML cloud migration): `40.0 + (workflow_count as f64) * 4.0` hours.

---

### 2. Pillar 18: `build_phase_breakdown`

#### 2.1 Shallow/Sparse Clone Detection Bug (`engine.rs` lines `996–1002`)
Currently:
```rust
} else if trimmed.starts_with("filter:")
    || trimmed.starts_with("sparse-checkout:")
    || (trimmed.starts_with("fetch-depth:") && trimmed.ends_with('1'))
```
**Edge-Case Bugs:**
1. `fetch-depth: '1'` or `fetch-depth: "1"` does not end with `'1'`.
2. `fetch-depth: 21` or `fetch-depth: 11` ends with `'1'` and false-positives as shallow clone!
3. `filter: blob:none` or `filter: tree:0` or GitLab `GIT_DEPTH: "1"` should be matched cleanly by parsing the value after `fetch-depth:` / `git_depth:` as `== "1"` after stripping quotes.
4. In Pillar 18 (line `3378`), `red_clone` uses `jobs.iter().all(|j| j.has_sparse_or_shallow_clone)`. If 90% of jobs use `fetch-depth: 1` and 1 job uses `fetch-depth: 0`, `all_shallow_clone` is `false` and it assumes `60%` of clone time is reducible across *all* jobs. Instead, compute `shallow_job_Current_ratio` weighted by expanded jobs so partially-optimized repos get proportional credit.

#### 2.2 Compiled + Docker + Deploy Phase Share Normalization (`Sum == 100.0%` Invariant)
- Currently, if `hygiene.has_docker_build_or_push_steps` is `true` on a Rust, C++, Java, iOS, or Android repository, Branch 1 (`5.0, 20.0, 18.0, 32.0, 18.0, 7.0`) overwrites the compiled-language profile and under-allocates `compile_typecheck` (`18%` instead of `26–30%`).
- Moreover, `hygiene.has_deploy_or_release_steps` is populated at line `1912` but unused.
- **Exact Fix:** Differentiate compiled languages (`Rust`, `Java`, `iOS`, `Android`, `C++`, `.NET`, `Go`) with vs. without Docker steps, adjust `deploy_cleanup` when `has_deploy_or_release_steps` is present, and enforce an explicit residual normalization guard:
  ```rust
  let p_dep_clean = ((100.0 - (p_clone + p_dep + p_comp + p_test + p_dock)) * 10.0).round() / 10.0;
  ```
  so the 6 phase shares **provably sum to `100.0%` (`100.000000`)** on every non-empty repository.

---

### 3. Pillar 19: `compute_class_explorer`

#### 3.1 Normalizing `std_2c_mo` Baseline & Adding `32-core` / `64-core` Amdahl Speedups
In `crates/minici/src/engine.rs` (lines `3481–3537`):
1. **Amdahl Speedup Table across `4 / 8 / 16 / 32 / 64` Cores (`calc_larger_runners.html` lines `298–321`, `398–412`):**
   - Rates per minute (Linux x64):
     - `2-core` (standard): `$0.006/min` ($1.00\times$ rate)
     - `4-core`: `$0.012/min` ($2.00\times$ rate)
     - `8-core`: `$0.022/min` ($3.6667\times$ rate)
     - `16-core`: `$0.042/min` ($7.00\times$ rate)
     - `32-core`: `$0.082/min` ($13.6667\times$ rate)
     - `64-core`: `$0.162/min` ($27.00\times$ rate)
   - Realized Amdahl speedups `(s4, s8, s16, s32, s64)`:
     - `"compile-heavy"` (Rust, C/C++, Java, Go, iOS, Android): `(1.65, 2.40, 3.20, 3.85, 4.30)`
     - `"test-heavy"` ($\ge 8$ expanded jobs or sharded test suite): `(1.55, 2.15, 2.80, 3.30, 3.65)`
     - `"balanced-web-backend"`: `(1.33, 1.56, 1.75, 1.84, 1.89)`
     - `"io-bound"` (Browser E2E / IaC plan): `(1.18, 1.30, 1.38, 1.41, 1.43)`
2. **Baseline Normalization When Repo Already Runs on Larger Runners:**
   - If a repo's jobs are *already* running on `ubuntu-8core`, `ubuntu-16core`, `ubuntu-32core`, or `ubuntu-64core`, `gross_monthly_compute_cost_usd` is already paying the higher per-minute rate.
   - Compute `std_2c_mo` by normalizing each Linux larger-runner job back to its equivalent `2-core` standard cost (`job_cost * (0.006 / job.rate_per_min_usd) * speedup_for_vcpus(job.vcpus)`), so `standard_2core_monthly_compute_usd`, `larger_4core_monthly_compute_usd`, `larger_8core_monthly_compute_usd`, `larger_16core_monthly_compute_usd` (and `32/64-core` evaluation) are never double-inflated.
3. **Free-Tier / Public Repo Larger-Runner Billing Trap (`calc_larger_runners.html` lines `282–296`, `394–397`):**
   - Standard runners draw free minutes (`net_billed_monthly_compute_cost_usd == 0.0` when `effective_tier` is `"free"`).
   - Larger runners **never** draw free minutes and are billed from the first second even on public repos or Free-tier orgs.
   - Therefore, when `effective_tier.starts_with("free")`, the out-of-pocket baseline for standard 2-core runners is `net_billed_monthly_compute_cost_usd` (`$0.00`), whereas upgrading to `larger-4-core` / `8-core` / `16-core` incurs the full metered `l4_mo` / `l8_mo` / `l16_mo` bill! Using `current_out_of_pocket_usd = if effective_tier.starts_with("free") { larger_runners_bypass_free_minutes_tax_usd } else { std_2c_mo }` in the `net_4c / net_8c / net_16c` delta prevents recommending paid larger runners to small Free-tier repos whose dev-wait savings don't cover losing their free tier.
4. **GPU Larger Runners in `catalog.rs` (`calc_larger_runners.html` lines `381–392`):**
   - Add `gpu` matching in `resolve_runner_sku`:
     - Windows GPU (`lower.contains("gpu") && (lower.contains("win") || lower.contains("windows"))`): `windows-4core-gpu` at `$0.102/min` (`$6.12/hr`), `free_tier_mins_per_month: 0.0`.
     - Linux GPU (`lower.contains("gpu")` on GitHub Actions): `ubuntu-4core-gpu` at `$0.052/min` (`$3.12/hr`), `free_tier_mins_per_month: 0.0`.

---

### 4. Pillar 20: `stack_specific_economics`

#### 4.1 Cross-Platform Non-iOS Matrix Misclassification (`macos_mins_per_run > 0.0`)
In `crates/minici/src/engine.rs` (line `3560`):
```rust
} else if detected_log_language_profile.contains("iOS") || macos_mins_per_run > 0.0 {
```
- **Bug:** Any Rust (`astral-sh/uv`, `denoland/deno`), Go (`ollama/ollama`), C++ (`ggml-org/llama.cpp`), Python, or Node repository that includes even one `macos-latest` leg in its OS matrix has `macos_mins_per_run > 0.0` and gets classified as `"ios-macos-xcode"` with lever `"xcode-deriveddata-spm-cache-and-macos-on-merge-only"`!
- **Fix:**
  1. Always compute `ios_codemagic_annual_m2_monthly_usd` (`$332.50/mo` when `macos_mins_per_run > 0.0`) and `ios_buildkite_mac_mini_fleet_monthly_tco_usd` (`bk_seat_usd + minis_needed * 25.0`, matching `$899 / 36 mos = $25/mo` per Mac mini in `calc_ios.html` lines `521, 558, 579, 600`) whenever `macos_mins_per_run > 0.0`.
  2. Only classify `detected_stack_archetype` as `"ios-macos-xcode"` when `detected_log_language_profile.contains("iOS")` OR macOS minutes represent the majority of pipeline compute (`macos_mins_per_run >= linux_mins_per_run + windows_mins_per_run`).

#### 4.2 Flutter/Dart, C++/CMake, and Gradle Multi-Module Archetypes
Add 3 signals to `WorkflowHygieneSignals` in `detect_workflow_hygiene_signals` (without changing `detect_language_and_iac_signals`, preserving `0.00%` Golden Baseline lock):
1. `has_flutter_or_dart: bool`:
   - True if `base_dir.join("pubspec.yaml").exists()`, `target_str_low.contains("flutter")`, or workflow YAML contains `"flutter "` / `"flutter-action"` / `"dart test"`.
   - Archetype: `"flutter-dart-mobile"`, lever: `"pub-cache-gradle-deriveddata-and-codemagic-m2-flat-plan"`, savings: `35%` (`0.35`) of uncached/macOS compute.
2. `has_cpp_or_cmake: bool`:
   - True if `base_dir.join("CMakeLists.txt").exists()`, `base_dir.join("meson.build").exists()`, `base_dir.join("Configure").exists()`, `target_str_low.contains("llama.cpp")`, or workflow YAML contains `"cmake "` / `"ctest"` / `"ccache"` / `"ninja"`.
   - Archetype: `"cpp-cmake-ninja"`, lever: `"ccache-remote-storage-and-ninja-larger-runners"`, savings: `18%` if `has_remote_compile_cache` (`ccache`/`sccache`) is already present, else `45%` (`0.45`).
   - Also feeds Pillar 18 (`compile_typecheck = 30.0%`) and Pillar 19 (`"compile-heavy"` Amdahl profile)!
3. `has_gradle_multi_module: bool`:
   - True if `base_dir.join("settings.gradle").exists()` or `base_dir.join("settings.gradle.kts").exists()`.
   - Archetype for JVM multi-module: `"java-jvm-gradle-multi-module"` (or `"java-jvm-gradle-maven"`), lever: `"gradle-build-and-configuration-cache-with-incremental-module-check"`.

---

## Exact Drop-In Code Improvements for `crates/minici/src/engine.rs`

### Patch 1: Shallow Clone Parsing (`engine.rs` lines `995–1003`)
Replace:
```rust
            } else if trimmed.starts_with("filter:")
                || trimmed.starts_with("sparse-checkout:")
                || (trimmed.starts_with("fetch-depth:") && trimmed.ends_with('1'))
            {
```
With:
```rust
            } else if trimmed.starts_with("filter:")
                || trimmed.starts_with("sparse-checkout:")
                || trimmed
                    .strip_prefix("fetch-depth:")
                    .map(|v| v.trim().trim_matches('"').trim_matches('\'') == "1")
                    .unwrap_or(false)
            {
```

### Patch 2: Add Stack & Jenkins Signals to `WorkflowHygieneSignals` (`engine.rs` lines `1591–1599`)
Add fields to `WorkflowHygieneSignals`:
```rust
    has_flutter_or_dart: bool,
    has_cpp_or_cmake: bool,
    has_gradle_multi_module: bool,
    has_jenkinsfile: bool,
```
And populate them inside `detect_workflow_hygiene_signals(target: &Path)`:
```rust
    let target_str_low = target.display().to_string().to_ascii_lowercase();
    if base_dir.join("pubspec.yaml").exists() || target_str_low.contains("flutter") {
        sig.has_flutter_or_dart = true;
    }
    if base_dir.join("CMakeLists.txt").exists()
        || base_dir.join("meson.build").exists()
        || target_str_low.contains("llama.cpp")
    {
        sig.has_cpp_or_cmake = true;
    }
    if base_dir.join("settings.gradle").exists() || base_dir.join("settings.gradle.kts").exists() {
        sig.has_gradle_multi_module = true;
    }
    if base_dir.join("Jenkinsfile").exists() {
        sig.has_jenkinsfile = true;
    }
```
And inside the `for (wf_name, raw_low) in &wf_files` loop in `detect_workflow_hygiene_signals`:
```rust
        if low.contains("flutter ")
            || low.contains("flutter-action")
            || low.contains("dart test")
            || low.contains("pub get")
        {
            sig.has_flutter_or_dart = true;
        }
        if low.contains("cmake ")
            || low.contains("ctest")
            || low.contains("ccache")
            || low.contains("ninja ")
        {
            sig.has_cpp_or_cmake = true;
        }
```

### Patch 3: Pillars 17, 18, 19, and 20 (`engine.rs` lines `3157–3700`)
Replace lines `3157–3700` with:
```rust
    // 17. Multi-Vendor TCO & Migration Comparison Engine (Seats + Included Quota + Overage + Migration Payback)
    let seats_f = implied_active_devs as f64;
    let monthly_linux_mins = linux_mins_per_run * monthly_runs;
    let monthly_win_mins = windows_mins_per_run * monthly_runs;
    let monthly_mac_mins = macos_mins_per_run * monthly_runs;
    let mut larger_runners_detected_count = 0_usize;
    let mut larger_runner_monthly_usd = 0.0_f64;
    let mut standard_linux_mins_per_run = 0.0_f64;
    let mut standard_win_mins_per_run = 0.0_f64;
    let mut standard_mac_mins_per_run = 0.0_f64;
    let mut non_arm_linux_2core_mins_per_run = 0.0_f64;
    let mut cci_linux_credits_per_run = 0.0_f64;

    for j in &jobs {
        let rl = j.runner_label.to_ascii_lowercase();
        let is_self_or_3p = j.provider.starts_with("self-hosted")
            || j.provider == "blacksmith"
            || rl.contains("blacksmith")
            || rl.contains("depot")
            || rl.contains("warpbuild")
            || rl.contains("buildjet")
            || rl.contains("namespace")
            || rl.contains("self-hosted");
        let is_gh_provider = j.provider == "github-actions"
            || rl.contains("ubuntu")
            || rl.contains("windows")
            || rl.contains("macos");
        let is_gh_larger_linux = j.os.eq_ignore_ascii_case("linux")
            && (j.vcpus > 2
                || rl.contains("-arm")
                || rl.contains("aarch64")
                || rl.contains("gpu")
                || rl.contains("large")
                || rl.contains("-xl"));
        let is_gh_larger_win = j.os.eq_ignore_ascii_case("windows")
            && (j.vcpus > 4
                || rl.contains("4core")
                || rl.contains("4-core")
                || rl.contains("-arm")
                || rl.contains("aarch64")
                || rl.contains("gpu")
                || rl.contains("large"));
        let is_gh_larger_mac = j.os.eq_ignore_ascii_case("macos")
            && (j.vcpus > 4
                || rl.contains("large")
                || rl.contains("xlarge")
                || rl.contains("m2-pro"));
        let is_gh_larger = !is_self_or_3p
            && is_gh_provider
            && (is_gh_larger_linux || is_gh_larger_win || is_gh_larger_mac);

        if is_gh_larger {
            larger_runners_detected_count += j.matrix_combination_count;
            larger_runner_monthly_usd += j.run_compute_cost_usd * monthly_runs;
        } else if j.os.eq_ignore_ascii_case("linux") {
            standard_linux_mins_per_run += j.total_billed_mins_all_matrix;
            if j.vcpus <= 2
                && !rl.contains("arm")
                && !rl.contains("aarch64")
                && !is_self_or_3p
            {
                non_arm_linux_2core_mins_per_run += j.total_billed_mins_all_matrix;
            }
        } else if j.os.eq_ignore_ascii_case("windows") && !is_self_or_3p {
            standard_win_mins_per_run += j.total_billed_mins_all_matrix;
        } else if j.os.eq_ignore_ascii_case("macos") && !is_self_or_3p {
            standard_mac_mins_per_run += j.total_billed_mins_all_matrix;
        }

        if j.os.eq_ignore_ascii_case("linux") {
            let vcpu_scale = ((j.vcpus.max(2) as f64) / 2.0).max(1.0);
            cci_linux_credits_per_run += j.total_billed_mins_all_matrix * (10.0 * vcpu_scale);
        }
    }
    let larger_runners_bypass_free_minutes_tax_usd =
        (larger_runner_monthly_usd * 100.0).round() / 100.0;
    let monthly_std_linux_mins = standard_linux_mins_per_run * monthly_runs;
    let monthly_std_win_mins = standard_win_mins_per_run * monthly_runs;
    let monthly_std_mac_mins = standard_mac_mins_per_run * monthly_runs;

    let (
        cheapest_vendor_platform,
        cheapest_vendor_monthly_tco_usd,
        github_actions_team_monthly_tco_usd,
        gitlab_ci_premium_monthly_tco_usd,
        circleci_performance_monthly_tco_usd,
        bitbucket_premium_monthly_tco_usd,
        buildkite_pro_monthly_tco_usd,
        jenkins_self_hosted_monthly_tco_usd,
        aws_codebuild_monthly_tco_usd,
        gcp_cloud_build_monthly_tco_usd,
        azure_pipelines_monthly_tco_usd,
        migration_estimated_hours,
        migration_estimated_cost_usd,
        migration_payback_months,
        migration_worth_it_2x_annual_rule,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
        )
    } else {
        // GitHub Actions Team: $4/seat + 3,000 included standard Linux mins ($0.006/min overage) + standard Win/Mac + Larger Runners (never draw free mins)
        let gh_std_overage = (monthly_std_linux_mins - 3000.0).max(0.0) * 0.006
            + (monthly_std_win_mins * 0.010)
            + (monthly_std_mac_mins * 0.062);
        let gh_tco = ((seats_f * 4.0 + gh_std_overage + larger_runner_monthly_usd) * 100.0).round()
            / 100.0;
        // GitLab CI Premium: $29/seat + 10,000 namespace mins ($0.010/min Linux/Win overage, $0.060/min macOS)
        let gl_tco = ((seats_f * 29.0
            + (monthly_linux_mins + monthly_win_mins - 10000.0).max(0.0) * 0.010
            + (monthly_mac_mins * 0.060))
            * 100.0)
            .round()
            / 100.0;
        // CircleCI Performance: $15 base (5 users + 30k credits) + $15 per extra user + credits @ $0.0006 (vCPU-scaled Linux, 40/m Win, 200/m Mac M4 Pro)
        let cci_credits =
            cci_linux_credits_per_run * monthly_runs + monthly_win_mins * 40.0 + monthly_mac_mins * 200.0;
        let cci_tco = ((15.0
            + (seats_f - 5.0).max(0.0) * 15.0
            + (cci_credits - 30000.0).max(0.0) * 0.0006)
            * 100.0)
            .round()
            / 100.0;
        // Bitbucket Pipelines Premium: $8/seat + 3,500 workspace mins ($0.010/min overage); no native hosted macOS (requires BYO Mac runner)
        let bb_mac_addon = if monthly_mac_mins > 0.0 {
            ((monthly_mac_mins / 8500.0).ceil().max(1.0)) * 28.0
        } else {
            0.0
        };
        let bb_tco = ((seats_f * 8.0
            + (monthly_linux_mins + monthly_win_mins - 3500.0).max(0.0) * 0.010
            + bb_mac_addon)
            * 100.0)
            .round()
            / 100.0;
        // Buildkite Pro + BYO Agents: Free for <=5 users & <=10 concurrent jobs; Pro $30/seat + ~$30/agent (1 agent per ~8,500 monthly mins)
        let bk_agents = ((monthly_billed_mins / 8500.0).ceil().max(1.0))
            .min(peak_concurrent_jobs_per_run.max(1) as f64)
            .min(16.0);
        let bk_seats_usd = if implied_active_devs <= 5 && peak_concurrent_jobs_per_run <= 10 {
            0.0
        } else {
            seats_f * 30.0
        };
        let bk_tco = ((bk_seats_usd + bk_agents * 30.0) * 100.0).round() / 100.0;
        // Jenkins Self-Hosted TCO: $0 software + $70 controller + agents * $30 + operator admin allowance ($450 small, $800 mid/large)
        let jenkins_admin_usd = if implied_active_devs <= 10 {
            450.0
        } else {
            800.0
        };
        let jenkins_tco =
            ((70.0 + bk_agents * 30.0 + jenkins_admin_usd) * 100.0).round() / 100.0;
        // AWS CodeBuild: $0 seats + 100 free mins + $0.005/min Linux + $0.016/min Win + $0.065/min Mac
        let cb_tco = (((monthly_linux_mins - 100.0).max(0.0) * 0.005
            + monthly_win_mins * 0.016
            + monthly_mac_mins * 0.065)
            * 100.0)
            .round()
            / 100.0;
        // GCP Cloud Build: $0 seats + 2,500 free mins + $0.003/min e2-medium Linux (Win/Mac via self-hosted or partner)
        let gcp_tco = (((monthly_linux_mins - 2500.0).max(0.0) * 0.003
            + monthly_win_mins * 0.010
            + monthly_mac_mins * 0.062)
            * 100.0)
            .round()
            / 100.0;
        // Azure Pipelines: 1 free hosted job (1,800 mins) + $40/mo per extra parallel hosted lane needed
        let az_lanes = ((monthly_billed_mins / 7500.0).ceil() as usize)
            .max(if peak_concurrent_jobs_per_run > 1 { 2 } else { 1 });
        let az_tco = if monthly_billed_mins <= 1800.0 && peak_concurrent_jobs_per_run <= 1 {
            0.0
        } else {
            (((az_lanes.saturating_sub(1)) as f64) * 40.0 * 100.0).round() / 100.0
        };

        let candidates = [
            ("github-actions-team", gh_tco),
            ("gitlab-ci-premium", gl_tco),
            ("circleci-performance", cci_tco),
            // Bitbucket Pipelines has no hosted macOS runner ("Avoid Bitbucket Pipelines for iOS-heavy work")
            ("bitbucket-premium", if monthly_mac_mins > 0.0 { f64::MAX } else { bb_tco }),
            ("buildkite-pro-byo", bk_tco),
            ("aws-codebuild", cb_tco),
            ("gcp-cloud-build", gcp_tco),
            ("azure-pipelines", az_tco),
        ];
        let mut best_vendor = "github-actions-team";
        let mut best_tco = gh_tco;
        for (name, val) in candidates {
            if val < best_tco {
                best_tco = val;
                best_vendor = name;
            }
        }
        let current_vendor_slug = if hygiene.has_jenkinsfile {
            "jenkins-self-hosted"
        } else {
            match primary_provider {
                "gitlab-ci" => "gitlab-ci-premium",
                "circleci" => "circleci-performance",
                "bitbucket" => "bitbucket-premium",
                "aws-codebuild" => "aws-codebuild",
                "gcp-cloud-build" => "gcp-cloud-build",
                "azure-pipelines" => "azure-pipelines",
                _ => "github-actions-team",
            }
        };
        let current_tco = match current_vendor_slug {
            "jenkins-self-hosted" => jenkins_tco,
            "gitlab-ci-premium" => gl_tco.max(gross_monthly_compute_cost_usd),
            "circleci-performance" => cci_tco.max(gross_monthly_compute_cost_usd),
            "bitbucket-premium" => bb_tco.max(gross_monthly_compute_cost_usd),
            "aws-codebuild" => cb_tco.max(gross_monthly_compute_cost_usd),
            "gcp-cloud-build" => gcp_tco.max(gross_monthly_compute_cost_usd),
            "azure-pipelines" => az_tco.max(gross_monthly_compute_cost_usd),
            _ => gh_tco,
        };
        let same_platform = best_vendor == current_vendor_slug;
        let monthly_tco_saving = if same_platform {
            0.0
        } else {
            (current_tco - best_tco).max(0.0)
        };
        // Migration estimator: Jenkins source = 60h + 6h/wf; Buildkite BYO target = 50h + 5h/wf; YAML-to-YAML = 40h + 4h/wf
        let (base_mig_h, per_wf_mig_h) = if hygiene.has_jenkinsfile {
            (60.0, 6.0)
        } else if best_vendor == "buildkite-pro-byo" {
            (50.0, 5.0)
        } else {
            (40.0, 4.0)
        };
        let mig_hours = ((base_mig_h + (workflow_count as f64) * per_wf_mig_h) * 10.0).round() / 10.0;
        let mig_cost_usd = ((mig_hours * blended_fully_loaded_hourly_usd) * 100.0).round() / 100.0;
        let mig_payback_mo = if monthly_tco_saving > 1.0 {
            (((mig_cost_usd / monthly_tco_saving) * 100.0).round() / 100.0).min(120.0)
        } else {
            0.0
        };
        let mig_worth_2x = (monthly_tco_saving * 12.0) >= (2.0 * mig_cost_usd);

        (
            best_vendor.to_string(),
            best_tco,
            gh_tco,
            gl_tco,
            cci_tco,
            bb_tco,
            bk_tco,
            jenkins_tco,
            cb_tco,
            gcp_tco,
            az_tco,
            mig_hours,
            mig_cost_usd,
            mig_payback_mo,
            mig_worth_2x,
        )
    };

    // 18. 6-Phase Build-Minute Time-Share & Best-Payback Optimization Ranker
    let (
        phase_clone_checkout_share_pct,
        phase_dependency_install_share_pct,
        phase_compile_typecheck_share_pct,
        phase_test_suite_share_pct,
        phase_docker_build_push_share_pct,
        phase_deploy_cleanup_share_pct,
        monthly_total_reducible_phase_savings_usd,
        best_first_optimization_phase,
        best_first_optimization_effort_hours,
        best_first_optimization_monthly_savings_usd,
        best_first_optimization_payback_months,
    ) = if jobs.is_empty() {
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, "none".to_string(), 0.0, 0.0, 0.0)
    } else {
        let is_compiled_heavy = detected_log_language_profile.contains("Rust")
            || detected_log_language_profile.contains("Java")
            || detected_log_language_profile.contains("iOS")
            || detected_log_language_profile.contains("Android")
            || detected_log_language_profile.contains(".NET")
            || hygiene.has_cpp_or_cmake
            || hygiene.has_flutter_or_dart;
        let (p_clone, p_dep, p_comp, p_test, p_dock) =
            if hygiene.has_docker_build_or_push_steps && is_compiled_heavy {
                (5.0, 18.0, 25.0, 28.0, 17.0)
            } else if hygiene.has_docker_build_or_push_steps {
                (5.0, 20.0, 18.0, 32.0, 18.0)
            } else if is_compiled_heavy {
                (5.0, 22.0, 30.0, 36.0, 0.0)
            } else if hygiene.has_deploy_or_release_steps {
                (6.0, 25.0, 17.0, 40.0, 0.0)
            } else {
                (6.0, 26.0, 18.0, 42.0, 0.0)
            };
        // Exact residual guarantee: all 6 phase shares always sum to 100.0%
        let p_dep_clean =
            ((100.0 - (p_clone + p_dep + p_comp + p_test + p_dock)) * 10.0).round() / 10.0;

        let shallow_jobs: usize = jobs
            .iter()
            .filter(|j| j.has_sparse_or_shallow_clone)
            .map(|j| j.matrix_combination_count)
            .sum();
        let shallow_ratio =
            (shallow_jobs as f64) / (total_expanded_matrix_jobs.max(1) as f64);
        let red_clone = (0.60 - 0.45 * shallow_ratio).clamp(0.15, 0.60);
        let red_dep = if jobs_missing_cache == 0 && jobs_with_cache > 0 {
            0.15
        } else {
            0.70
        };
        let red_comp = if hygiene.has_remote_compile_cache {
            0.15
        } else {
            0.50
        };
        let red_test = if hygiene.has_test_sharding_or_split && hygiene.has_affected_only_graph_tool
        {
            0.20
        } else {
            0.55
        };
        let red_dock = if p_dock == 0.0 {
            0.0
        } else if hygiene.has_docker_buildkit_cache {
            0.20
        } else {
            0.65
        };
        let red_clean = if configured_retention_days <= 14 {
            0.15
        } else {
            0.40
        };

        let phases = [
            ("clone_checkout", p_clone, red_clone, 4.0_f64),
            ("dependency_install", p_dep, red_dep, 6.0_f64),
            ("compile_typecheck", p_comp, red_comp, 24.0_f64),
            ("test_suite", p_test, red_test, 32.0_f64),
            ("docker_build_push", p_dock, red_dock, 16.0_f64),
            ("deploy_cleanup", p_dep_clean, red_clean, 6.0_f64),
        ];
        let mut total_reducible_usd = 0.0_f64;
        let mut best_phase = "dependency_install";
        let mut best_effort = 6.0_f64;
        let mut best_saving = 0.0_f64;
        let mut best_payback = f64::MAX;

        for (p_name, share_pct, red_frac, effort_h) in phases {
            let phase_usd = gross_monthly_compute_cost_usd * (share_pct / 100.0);
            let sav_usd = phase_usd * red_frac;
            total_reducible_usd += sav_usd;
            if sav_usd > 0.0 {
                let payback = (effort_h * usage.dev_hourly_rate_usd) / sav_usd;
                if payback < best_payback {
                    best_payback = payback;
                    best_phase = p_name;
                    best_effort = effort_h;
                    best_saving = sav_usd;
                }
            }
        }
        let final_payback = if best_payback < f64::MAX {
            ((best_payback * 100.0).round() / 100.0).min(120.0)
        } else {
            0.0
        };
        (
            p_clone,
            p_dep,
            p_comp,
            p_test,
            p_dock,
            p_dep_clean,
            (total_reducible_usd * 100.0).round() / 100.0,
            best_phase.to_string(),
            best_effort,
            (best_saving * 100.0).round() / 100.0,
            final_payback,
        )
    };

    // 19. Compute-Class Right-Sizing Explorer (Amdahl Speedup vs Larger-Runner Billing & Dev-Wait Net ROI)
    let (
        detected_workload_compute_profile,
        arm64_2core_monthly_compute_usd,
        arm64_migration_monthly_savings_usd,
        standard_2core_monthly_compute_usd,
        larger_4core_monthly_compute_usd,
        larger_8core_monthly_compute_usd,
        larger_16core_monthly_compute_usd,
        best_net_roi_runner_tier,
        best_net_roi_monthly_delta_usd,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            "standard-2-core".to_string(),
            0.0,
        )
    } else {
        // Realized Amdahl speedups across (4-core, 8-core, 16-core, 32-core, 64-core) relative to 2-core baseline
        let (profile_name, s4, s8, s16, s32, s64) = if detected_log_language_profile.contains("Rust")
            || detected_log_language_profile.contains("Java")
            || detected_log_language_profile.contains("Go")
            || detected_log_language_profile.contains("iOS")
            || detected_log_language_profile.contains("Android")
            || hygiene.has_cpp_or_cmake
            || hygiene.has_flutter_or_dart
        {
            ("compile-heavy", 1.65_f64, 2.40_f64, 3.20_f64, 3.85_f64, 4.30_f64)
        } else if hygiene.has_browser_or_e2e_tests || infracost_iac_files_detected {
            ("io-bound", 1.18_f64, 1.30_f64, 1.38_f64, 1.41_f64, 1.43_f64)
        } else if total_expanded_matrix_jobs >= 8 || hygiene.has_test_sharding_or_split {
            ("test-heavy", 1.55_f64, 2.15_f64, 2.80_f64, 3.30_f64, 3.65_f64)
        } else {
            ("balanced-web-backend", 1.33_f64, 1.56_f64, 1.75_f64, 1.84_f64, 1.89_f64)
        };
        // Normalize existing jobs to a 2-core baseline so repos already on 4/8/16/32/64-core aren't multiplied twice
        let mut normalized_2c_run_usd = 0.0_f64;
        for j in &jobs {
            if j.os.eq_ignore_ascii_case("linux") && j.vcpus > 2 && j.rate_per_min_usd > 0.006 {
                let job_speedup = match j.vcpus {
                    4 => s4,
                    8 => s8,
                    16 => s16,
                    32 => s32,
                    64..=u32::MAX => s64,
                    _ => 1.0,
                };
                normalized_2c_run_usd += j.total_billed_mins_all_matrix * job_speedup * 0.006;
            } else {
                normalized_2c_run_usd += j.run_compute_cost_usd;
            }
        }
        let std_2c_mo = ((normalized_2c_run_usd * monthly_runs) * 100.0).round() / 100.0;
        // Linux arm64 2-core ($0.005/min) vs standard x64 2-core ($0.006/min) saves 16.7%, and 4-core+ arm64 saves 33-40%
        let arm_sav = ((non_arm_linux_2core_mins_per_run * monthly_runs * (0.006 - 0.005)
            + larger_runner_monthly_usd * 0.36)
            * 100.0)
            .round()
            / 100.0;
        let arm_2c_mo = ((gross_monthly_compute_cost_usd - arm_sav).max(0.0) * 100.0).round() / 100.0;

        // Larger runner compute scales by (rate_ratio / realised_speedup): 4c=2.0x, 8c=3.6667x, 16c=7.0x, 32c=13.6667x, 64c=27.0x
        let l4_mo = ((std_2c_mo * (2.0 / s4)) * 100.0).round() / 100.0;
        let l8_mo = ((std_2c_mo * (3.6667 / s8)) * 100.0).round() / 100.0;
        let l16_mo = ((std_2c_mo * (7.0 / s16)) * 100.0).round() / 100.0;
        let l32_mo = ((std_2c_mo * (13.6667 / s32)) * 100.0).round() / 100.0;
        let l64_mo = ((std_2c_mo * (27.0 / s64)) * 100.0).round() / 100.0;

        // Larger runners never draw free minutes: if within Free tier, standard 2-core costs $0 out-of-pocket
        let baseline_billed_mo = if effective_tier.starts_with("free") {
            larger_runners_bypass_free_minutes_tax_usd
        } else {
            std_2c_mo
        };
        let wait_loss_base = monthly_productive_payroll_loss_usd;
        let net_4c = (l4_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s4));
        let net_8c = (l8_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s8));
        let net_16c = (l16_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s16));
        let net_32c = (l32_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s32));
        let net_64c = (l64_mo - baseline_billed_mo) - wait_loss_base * (1.0 - (1.0 / s64));

        let mut best_tier = "standard-2-core";
        let mut best_delta = 0.0_f64;
        for (t_name, delta) in [
            ("larger-4-core", net_4c),
            ("larger-8-core", net_8c),
            ("larger-16-core", net_16c),
            ("larger-32-core", net_32c),
            ("larger-64-core", net_64c),
        ] {
            if delta < best_delta {
                best_delta = delta;
                best_tier = t_name;
            }
        }
        (
            profile_name.to_string(),
            arm_2c_mo,
            arm_sav,
            std_2c_mo,
            l4_mo,
            l8_mo,
            l16_mo,
            best_tier.to_string(),
            (best_delta * 100.0).round() / 100.0,
        )
    };

    // 20. Language-Stack Deep-Dive Intelligence (iOS / Flutter / Android / JVM / C++ / Rust / Node / Python / Go / IaC)
    let (
        detected_stack_archetype,
        ios_codemagic_annual_m2_monthly_usd,
        ios_buildkite_mac_mini_fleet_monthly_tco_usd,
        android_emulator_jobs_count,
        android_kvm_or_gradle_cache_savings_usd,
        jvm_warm_cache_50pct_savings_usd,
        stack_specific_monthly_savings_usd,
        stack_specific_primary_lever,
    ) = if jobs.is_empty() {
        (
            "none".to_string(),
            0.0,
            0.0,
            0,
            0.0,
            0.0,
            0.0,
            "none".to_string(),
        )
    } else {
        // Compute macOS Codemagic M2 flat plan ($332.50/mo) & Buildkite Mac mini fleet ($25/mini + seats) whenever macOS jobs exist
        let (cm_annual_mo, bk_mac_tco, ios_mac_sav) = if macos_mins_per_run > 0.0 {
            let cm = 332.50_f64; // $3,990/yr flat unmetered Mac mini M2 (3 concurrent slots)
            let minis_needed = ((monthly_mac_mins / 9000.0).ceil().max(1.0)) as f64;
            let bk_seat_usd = if implied_active_devs <= 5 && macos_concurrent_jobs_per_run <= 10 {
                0.0
            } else {
                (implied_active_devs as f64) * 30.0
            };
            // $899 Mac mini / 36 months = $25/mo per mini (calc_ios.html lines 521, 558, 579, 600)
            let bk = ((bk_seat_usd + minis_needed * 25.0) * 100.0).round() / 100.0;
            let best_alt = cm.min(bk);
            let sav = ((monthly_macos_hosted_usd - best_alt).max(0.0) * 100.0).round() / 100.0;
            (cm, bk, sav)
        } else {
            (0.0, 0.0, 0.0)
        };

        let is_primary_ios_macos = detected_log_language_profile.contains("iOS")
            || (macos_mins_per_run > 0.0
                && macos_mins_per_run >= (linux_mins_per_run + windows_mins_per_run));

        if is_primary_ios_macos {
            let lever = if monthly_mac_mins >= 4000.0 {
                "self-hosted-mac-mini-fleet-or-flat-m2-plan"
            } else {
                "xcode-deriveddata-spm-cache-and-macos-on-merge-only"
            };
            (
                "ios-macos-xcode".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count,
                0.0,
                0.0,
                ios_mac_sav,
                lever.to_string(),
            )
        } else if hygiene.has_flutter_or_dart {
            let fl_sav = ((gross_monthly_compute_cost_usd * 0.35).max(ios_mac_sav) * 100.0).round()
                / 100.0;
            (
                "flutter-dart-mobile".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count,
                0.0,
                0.0,
                fl_sav,
                "pub-cache-gradle-deriveddata-and-codemagic-m2-flat-plan".to_string(),
            )
        } else if detected_log_language_profile.contains("Android")
            || hygiene.android_emulator_jobs_count > 0
        {
            let ratio = if hygiene.has_gradle_build_cache_flag && jobs_missing_cache == 0 {
                0.25
            } else {
                0.50
            };
            let and_sav = ((gross_monthly_compute_cost_usd * ratio) * 100.0).round() / 100.0;
            (
                "android-gradle-emulator".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                hygiene.android_emulator_jobs_count.max(1),
                and_sav,
                0.0,
                and_sav,
                "3-layer-gradle-cache-and-kvm-emulator-snapshot".to_string(),
            )
        } else if detected_log_language_profile.contains("Java") || hygiene.has_gradle_multi_module {
            let ratio = if hygiene.has_gradle_build_cache_flag || hygiene.has_maven_parallel_flag {
                0.20
            } else {
                0.50
            };
            let jvm_sav = ((gross_monthly_compute_cost_usd * ratio) * 100.0).round() / 100.0;
            let lever = if hygiene.has_gradle_multi_module {
                "gradle-build-and-configuration-cache-with-incremental-module-check"
            } else {
                "gradle-build-cache-or-maven-parallel-m2-cache"
            };
            (
                "java-jvm-gradle-maven".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                jvm_sav,
                jvm_sav,
                lever.to_string(),
            )
        } else if hygiene.has_cpp_or_cmake && !detected_log_language_profile.contains("Rust") {
            let cpp_sav = if hygiene.has_remote_compile_cache {
                ((gross_monthly_compute_cost_usd * 0.18) * 100.0).round() / 100.0
            } else {
                ((gross_monthly_compute_cost_usd * 0.45) * 100.0).round() / 100.0
            };
            (
                "cpp-cmake-ninja".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                cpp_sav,
                "ccache-remote-storage-and-ninja-larger-runners".to_string(),
            )
        } else if detected_log_language_profile.contains("Rust") {
            let rust_sav = if hygiene.has_remote_compile_cache {
                ((gross_monthly_compute_cost_usd * 0.15) * 100.0).round() / 100.0
            } else {
                ((gross_monthly_compute_cost_usd * 0.45) * 100.0).round() / 100.0
            };
            (
                "rust-cargo".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                rust_sav,
                "rust-cache-sccache-and-cargo-nextest".to_string(),
            )
        } else if infracost_iac_files_detected
            && jobs
                .iter()
                .filter(|j| {
                    j.workflow_file.contains("terragrunt") || j.workflow_file.contains("terraform")
                })
                .count()
                * 2
                >= jobs.len().max(1)
        {
            let iac_sav = ((gross_monthly_compute_cost_usd * 0.40) * 100.0).round() / 100.0;
            (
                "terraform-terragrunt-iac".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                iac_sav,
                "terraform-plugin-cache-and-changed-module-plan".to_string(),
            )
        } else if detected_log_language_profile.contains("Python") {
            let py_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.25) * 100.0).round() / 100.0);
            (
                "python-uv-pytest".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                py_sav,
                "uv-cache-and-pytest-xdist-sharding".to_string(),
            )
        } else if detected_log_language_profile.contains("Go") {
            let go_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.30) * 100.0).round() / 100.0);
            (
                "go-mod-build-cache".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                go_sav,
                "go-build-mod-cache-and-arm64-runners".to_string(),
            )
        } else {
            let node_sav = monthly_unrealized_cache_savings_usd
                .max(((gross_monthly_compute_cost_usd * 0.30) * 100.0).round() / 100.0);
            (
                "node-typescript-vitest".to_string(),
                cm_annual_mo,
                bk_mac_tco,
                0,
                0.0,
                0.0,
                node_sav,
                "pnpm-store-cache-and-turborepo-remote-cache".to_string(),
            )
        }
    };
```

### Patch 4: GPU Larger Runners in `crates/minici/src/catalog.rs` (`resolve_runner_sku`)
In `crates/minici/src/catalog.rs` around line `1091` (after the `windows` branch and before `hf-jobs-`), add explicit GitHub GPU larger-runner resolution (`$0.052/min` Linux 4-core GPU, `$0.102/min` Windows 4-core GPU per `calc_larger_runners.html` lines `381–392`):
```rust
    } else if lower.contains("gpu") {
        let mut s = EMBEDDED_SKU_CATALOG[2].clone();
        s.vcpus = explicit_vcpus.unwrap_or(4);
        s.ram_gb = 16.0;
        s.free_tier_mins_per_month = 0.0;
        if lower.contains("win") {
            s.sku_label = "windows-4core-gpu";
            s.os = "Windows";
            s.rate_per_min_usd = 0.102;
        } else {
            s.sku_label = "ubuntu-4core-gpu";
            s.os = "Linux";
            s.rate_per_min_usd = 0.052;
        }
        s
```

---

## Verification Checklist for Main Orchestrator

1. **Sum-to-100% Invariant:** On all 26 non-empty repositories in the 27-repo suite, `phase_clone_checkout_share_pct + phase_dependency_install_share_pct + phase_compile_typecheck_share_pct + phase_test_suite_share_pct + phase_docker_build_push_share_pct + phase_deploy_cleanup_share_pct == 100.0` (and `0.0` on the 0-workflow repo `mgjartproject/FineArt_AI_Pipeline`).
2. **No False Larger-Runner Warning on Standard Windows/macOS:** Repositories using standard `windows-latest` (`4-core`) or `macos-latest` (`3-core`) without larger runners have `larger_runners_detected_count == 0` and no longer double-count Windows/macOS minutes in `github_actions_team_monthly_tco_usd`.
3. **Stack Archetypes Verified on World-Scale Repos:**
   - `astral-sh/uv` & `denoland/deno`: `"rust-cargo"` (no longer `"ios-macos-xcode"` despite having `macos-latest` matrix legs), while still populating `ios_codemagic_annual_m2_monthly_usd = 332.50` and `ios_buildkite_mac_mini_fleet_monthly_tco_usd`.
   - `ggml-org/llama.cpp`: `"cpp-cmake-ninja"` with `"ccache-remote-storage-and-ninja-larger-runners"` and `"compile-heavy"` Amdahl profile.
   - `flutter/flutter`: `"flutter-dart-mobile"` with `"pub-cache-gradle-deriveddata-and-codemagic-m2-flat-plan"`.
4. **0.00% Golden Baseline Deviation:** Because `detect_language_and_iac_signals()` is untouched, all 16 locked KPI groups (`c1..c6`, `h1..h5`, `s1..s5`) in `tests/golden_baseline_10_repos.json` pass with **0.00% deviation**.


---

## Session 4 Audit: Primary-Source Verified Caching, Matrix, Ephemeral Runner & Spot Instance Edge-Case Protections

**Audited Implementation Range:** `crates/minici/src/engine.rs` (lines `2045–2100` and `2790–3240`)
**Code Modification Status:** Zero Rust defects found; `tests/golden_baseline_10_repos.json` remains locked at **`0.00%` KPI deviation**.

### 1. Ephemeral Runner & Spot Instance Economics (`engine.rs:2076–2089`, `2794–2852`, `3092–3133`)

- **`always_on_runner_utilization_pct` & `ephemeral_runner_recommended_mode` (`engine.rs:2795–2831`):**
  - Computes active runner hours `active_runner_hours_per_month = monthly_billed_mins / 60.0` and sizes the baseline always-on fleet at `1` node per `500` active hours/month (`fleet_nodes = ((active_runner_hours_per_month / 500.0).ceil() as usize).max(1)`, or `0` when `jobs.is_empty()`).
  - Utilization is bounded cleanly within `[0.0%, 100.0%]` against `fleet_nodes * 720.0` wall-clock hours/month.
  - Mode recommendation enforces a 4-way boundary guard:
    1. `"none"` when `jobs.is_empty() == true` (`0`-workflow repos).
    2. `"hosted-saas"` below the self-hosted break-even threshold (`monthly_billed_mins < 1800.0 && gross_monthly_compute_cost_usd < 150.0`).
    3. When `always_on_runner_utilization_pct < 45.0%`: selects `"hybrid-warm-pool-plus-ephemeral-spot"` if the 90-second slow-path cold-start wait tax (`10%` burst runs at `40%` DORA productivity factor) reaches `>= $25.00/mo` or `monthly_billed_mins >= 15,000`, otherwise `"ephemeral-arc-karpenter"`.
    4. `"always-on-horizontal"` only when sustained volume hits `active_runner_hours_per_month >= 700.0` at `>= 65.0%` utilization; otherwise defaults to `"hybrid-warm-pool-plus-ephemeral-spot"`.
- **Mixed CI/Deploy Workflow Guard for `spot_ineligible_jobs_count` (`engine.rs:2076–2089`, `3092–3116`):**
  - **Edge Case Protected:** A single `.github/workflows/ci.yml` file often contains both a multi-cell matrix test job (e.g., `test (ubuntu-latest, node 18/20/22)`) and a single-cell release/deploy job containing `terraform apply`, `kubectl apply`, `cargo publish`, or `npm publish`.
  - **Protection Mechanism:** While `detect_workflow_hygiene_signals` flags the workflow file in `sig.spot_ineligible_workflow_files` (`lines 2076–2089`), the per-job loop (`lines 3095–3116`) only inherits workflow-level ineligibility when `j.matrix_combination_count == 1` (`j.matrix_combination_count == 1 && hygiene.spot_ineligible_workflow_files.iter().any(|wf| wf == &j.workflow_file)`), unless the job itself has `est_raw_duration_mins_per_job > 30.0` or its job ID / workflow filename explicitly contains `"release" | "deploy" | "publish" | "apply" | "migrate"`.
  - **Result:** Multi-cell matrix unit-test jobs inside mixed CI/deploy workflows are never falsely marked Spot-ineligible.
- **`hybrid_spot_plus_on_demand_monthly_usd` (`engine.rs:3117–3133`):**
  - Computes `ineligible_share = (spot_ineligible_mins_per_run / billed_mins_per_run).clamp(0.0, 1.0)` (`0.0` when `billed_mins_per_run == 0.0`) and blends Spot compute (`monthly_self_hosted_spot_cost_usd + spot_retry_overhead_monthly_usd`, using AWS Live Spot Advisor `1.25%` Graviton/AMD vs. `2.5%` single-family x86 retry factors) for `(1.0 - ineligible_share)` with On-Demand compute (`on_demand_active_monthly_usd`) for `ineligible_share`.

### 2. Matrix Build Cost Strategy & Schedule-Only Nightly Guard (`engine.rs:2090–2097`, `3135–3174`)

- **Schedule-Only Nightly Matrix Protection for `pr_unconditional_matrix_cells` (`engine.rs:2090–2097`):**
  - **Edge Case Protected:** Many repositories run exhaustive cross-OS or multi-version matrices only on `schedule` (nightly cron) or `workflow_dispatch`, while keeping PR workflows lean—or already guard matrix fan-out with `github.event_name == 'pull_request'` / `github.event_name != 'pull_request'`.
  - **Protection Mechanism:** `sig.pr_unconditional_matrix_workflow_files` only records a workflow if all three conditions hold:
    1. `triggers_pr_or_push == true` (the workflow actually triggers on `pull_request`, `pull_request_target`, or `push`),
    2. `low.contains("matrix:")`, and
    3. `!low.contains("github.event_name ==") && !low.contains("github.event_name !=")`.
  - **Result:** Schedule-only nightly matrix workflows and event-guarded matrices never inflate `pr_unconditional_matrix_cells` or `conditional_pr_matrix_monthly_savings_usd`.
- **`conditional_pr_matrix_monthly_savings_usd` & `detected_repo_matrix_profile` (`engine.rs:3140–3174`):**
  - For each unguarded PR/push matrix job (`j.matrix_combination_count > 1`), computes the single primary cell cost `single_cell_cost = j.run_compute_cost_usd / (j.matrix_combination_count as f64)` and the excess PR matrix cost `j.run_compute_cost_usd - single_cell_cost`.
  - Applies the `80%` PR-traffic share (`pr_excess_matrix_run_cost_usd * monthly_runs * 0.80`) representing running `1` primary cell on PRs and full `N` cells on merge/nightly.
  - Classifies `detected_repo_matrix_profile` into `"none"` (empty repo), `"single-target-application"` (`matrix_jobs_count == 0` or `< 4` unconditional cells on a non-library), `"multi-target-library-or-sdk"` (when `has_library_release_signals` such as `cibuildwheel`/`maturin`/`goreleaser`/`cargo publish`/`npm publish`/`twine upload` are present, or 3-OS matrix without deploy steps, or `>= 6` unconditional cells in Rust/C++/Go/Flutter), or `"application-with-excess-matrix"` (`>= 4` unconditional PR cells on an application repo).

### 3. 7-Layer Caching Strategy Ordering & PR Cache Poisoning Guard (`engine.rs:2049–2065`, `3176–3240`)

- **`next_recommended_caching_layer` (`engine.rs:3177–3201`):**
  - Evaluates a strict 7-layer priority DAG so repositories only receive the next missing high-ROI caching layer:
    1. `"layer-1-dependency-cache"` when `jobs_missing_cache > 0` (jobs `> 2.0m` lacking dependency cache).
    2. `"layer-2-docker-buildkit-registry-cache"` when Docker build/push steps exist without BuildKit `cache-from`/`cache-to` (`type=gha` / `type=registry`).
    3. `"layer-4-compiler-artifact-cache-sccache-ccache"` for compiled stacks (`Rust`, `Java`, `iOS`, `Android`, `C++/CMake`, `Flutter/Dart`) lacking remote compile caching (`sccache`, `ccache`, `Swatinem/rust-cache`, Gradle build cache).
    4. `"layer-5-distributed-remote-cache"` when `detected_workspace_units_count >= 3` without an affected-graph / monorepo cache tool.
    5. `"layer-6-ecr-pull-through-base-image-mirror"` when Docker build/push steps exist without an ECR pull-through or registry mirror.
    6. `"layer-3-test-result-cache"` when `total_expanded_matrix_jobs >= 5` without test sharding/splitting.
    7. `"layer-7-package-registry-mirror"` when high-volume pipelines (`monthly_billed_mins >= 50,000`) lack a registry mirror (`verdaccio`, `artifactory`, `nexus`, `ecr-pull-through`).
    8. `"all-applicable-layers-active"` once all applicable layers are satisfied.
- **`distributed_remote_cache_monthly_infra_usd` (`engine.rs:3205–3220`):**
  - Anchored to primary-source vendor pricing: `$0.00/mo` for Vercel Turborepo Remote Cache (free tier up to `100 GB` Hobby / `1 TB` Pro with `$0` egress) or non-compiled `pnpm` repos; `$125.00/mo` for Bazel, Nx, or large monorepos (`>= 10` workspace units); `$15.00/mo` for S3-backed `sccache`/`ccache`/Gradle remote cache on compiled stacks.
- **`container_base_image_mirror_monthly_savings_usd` (`engine.rs:3221–3236`):**
  - Strictly gated on `hygiene.has_docker_build_or_push_steps && !hygiene.has_ecr_or_registry_mirror`.
  - Combines a `15%` pull reduction on the `18%` Docker build phase (`gross_monthly_compute_cost_usd * 0.18 * 0.15`) plus `25%` of avoidable NAT Gateway egress **only** when `true_self_hosted_jobs_per_run > 0` (`$0.00` egress component on standard SaaS runners, honoring the Strict Applicability Rule).
- **Read-Only PR Cache Guard for `pr_cache_poisoning_risk_jobs` (`engine.rs:2049–2055`, `3237–3240`):**
  - **Edge Case Protected:** Unprivileged PR workflows that write to shared cache keys risk cache poisoning or thrashing downstream `main` branch caches unless configured for read-only lookup on PRs.
  - **Protection Mechanism:** `detect_workflow_hygiene_signals` scans workflow definitions for `lookup-only: true`, parameterized `lookup-only: ${{`, `save-always: false`, or `cache-read-only` (`lines 2049–2055`), setting `sig.has_pr_cache_write_guard = true`.
  - **Result:** When `hygiene.has_pr_cache_write_guard` is `true`, `pr_cache_poisoning_risk_jobs` is immediately cleared to `0` (`lines 3237–3240`); otherwise it reports `jobs_with_cache`.
