# 💰 MiniCI (`Skipwise`) — Infracost for CI/CD Pipelines (`354 µs` Pure Rust)

**Catch Expensive Pull Request CI Matrix Spikes Before You Merge.**
Sub-millisecond Pure Rust engine (`573 KB` zero-dependency binary, `1.20 MB` peak RAM, `0.00s` CPU) that expands your CI workflow matrices on every Pull Request, local `pre-push` hook, and IDE session—showing exact **Free-Tier Quota Burn %**, **Developer Wait-Time Dollars**, and **Per-PR Compute Cost Deltas** across all **7 Official CI Providers** (`GitHub Actions`, `GitLab CI`, `CircleCI`, `Bitbucket`, `AWS CodeBuild`, `GCP Cloud Build`, `Azure DevOps`) + **Blacksmith** & **AWS Spot via Infracost** (`Exit 0` Always).

* 🌐 **Live Interactive Website & 7-Provider Calculator:** https://minici-cost-action.raza-aamir01.workers.dev
* 🎬 **Playable MP4 Product Demo (`1.8 MB`):** [https://minici-cost-action.raza-aamir01.workers.dev/minici-cost-demo.mp4](https://minici-cost-action.raza-aamir01.workers.dev/minici-cost-demo.mp4) (`web/minici-cost-demo.mp4`)

---

## 🏗️ Shift-Left Architecture

![MiniCI Customer Architecture Diagram](web/customer-diagram.svg)

---

## ⚡ Quickstart (30-Second GitHub Action)

Drop `.github/workflows/minici-cost.yml` into any repository (or generate it automatically via `minici init-hooks --path .`):

```yaml
name: PR CI Cost Receipt
on: [pull_request]
jobs:
  ci-cost-diff:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: Aamir-raza-1/minici-cost-action@v1
        with:
          tier: auto
          prs-per-week: 25
          pushes-per-pr: 4
          dev-hourly-rate: 65
```

---

## 🛠️ Local CLI, Zero-Clone Public GitHub URL Audit & Multi-Format Reports

```bash
# 1. Build & install the 573 KB zero-dependency Rust binary locally
cargo build --release --manifest-path crates/minici/Cargo.toml
cp crates/minici/target/release/minici ~/.local/bin/minici

# 2. Audit any local repository in < 1 ms (Cross-Platform ASCII Box Table)
minici breakdown --path .

# 3. Audit ANY public GitHub repository WITHOUT cloning (Zero-Clone API + 1.46 ms Local Cache)
minici breakdown --repo https://github.com/vercel/next.js
minici breakdown --repo ollama/ollama

# 4. Generate standalone Executive HTML, JSON, CSV, or GitHub PR Markdown reports
minici --report report.html
minici --report report.json --quiet

# 5. Compare PR branch CI cost delta against main before pushing
minici diff --path . --compare-to main

# 6. Sync all 7 CI Providers + Infracost v2.19.0 Cloud Instance SKUs (0-Recompile)
minici self-update
```

---

## 👤 Author & License
Built by **Aamir Raza** (`raza.aamir01@gmail.com` • GitHub: `@Aamir-raza-1`) • Licensed under Apache-2.0.
