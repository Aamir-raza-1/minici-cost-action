# 📘 MANUAL: HOW TO RUN `minici` ON ANY LOCAL REPO, ANY PUBLIC GITHUB URL, OR ANY NEW MACHINE (`MANUAL_HOW_TO_RUN.md`)

> [!IMPORTANT]
> **THREE QUESTIONS ANSWERED IMMEDIATELY:**
> 1. **Why was the CLI output so long before?** Because `minici` used to print the full 40-line GitHub PR Markdown table in your terminal by default! Now, running `minici breakdown --path .` or `minici diff --path . --compare-to main` prints a **clean, 8-line Executive Box Summary** in your terminal (and only prints the big Markdown table if you explicitly pass `--format markdown` for a GitHub PR comment).
> 2. **Can I pass a public GitHub URL directly without cloning it first?** **YES!** You can pass `--repo https://github.com/owner/repo` or `--repo owner/repo` directly to `minici breakdown` or `minici diff`. `minici` automatically fetches the workflow metadata in a temporary directory, prints your 8-line cost receipt, and cleans up after itself!
> 3. **Why did `minici diff --path . --compare-to main` show `+28 jobs` earlier?** Because earlier, `main` wasn't a folder on disk so `minici` treated `main` as an empty directory! Now, `minici` automatically detects when `--compare-to main` (or `HEAD~1`, `origin/main`) is a **Git branch/commit ref** and compares your working tree against that exact Git branch.

---

## 1. Everyday Commands You Run in Your Terminal (`Short, Clean 8-Line Output`)

### A. Check Any Local Repository (Where You Are Standing Right Now)
```bash
minici breakdown --path .
```
**What you see (clean 8-line summary):**
- **Line 1:** Monthly CI Bill (`$/mo` & `$/run`) + Effective Tier (`free` vs `paid`)
- **Line 2:** Free-Tier Quota Burn % (`mins used / 2,000 free mins`)
- **Line 3:** Developer Wait-Time Cost (`$/mo` & `$/run` at `$65/hr`)
- **Line 4:** Top Cost Driver (the single most expensive workflow + matrix job in the repo)
- **Line 5:** Break-Even Status (whether you crossed `5,000 mins/mo` to switch to Blacksmith or Self-Hosted Spot)

### B. Check Any Public GitHub Repository Directly by URL or `owner/repo` (No Manual Clone Needed!)
```bash
# Pass any full GitHub URL:
minici breakdown --repo https://github.com/psf/requests

# Or pass owner/repo shorthand directly:
minici breakdown --repo fastapi/fastapi
```

### C. Compare Your Current Branch Against `main` or `HEAD~1` (`Pull Request Cost Diff`)
```bash
minici diff --path . --compare-to main
# Or compare against the previous commit:
minici diff --path . --compare-to HEAD~1
```

### D. When You Want the Full Per-Job Markdown Table or JSON (For GitHub PR Comments)
```bash
minici breakdown --path . --format markdown
minici diff --path . --compare-to main --format markdown
minici breakdown --path . --format json
```

---

## 2. How to Install & Run on Any Brand-New Machine, Server, or Laptop (`Zero Local Scripts`)

If you log into a **brand-new MacBook, Linux VM, Docker container, or CI runner**, here is how you install and run `minici` from scratch in **under 30 seconds**:

### Option 1: Install Globally with `cargo` (Works on Any Machine with Rust)
```bash
cargo install --git https://github.com/Aamir-raza-1/minici-cost-action.git
minici --version
minici breakdown --repo https://github.com/psf/requests
```

### Option 2: Clone & Build in 2 Seconds (Zero External Dependencies!)
Because `crates/minici` uses **zero external crates (`std` only)**, it compiles in `~2 seconds` even on an air-gapped or fresh server without downloading anything from `crates.io`:
```bash
git clone https://github.com/Aamir-raza-1/minici-cost-action.git
cd minici-cost-action
cargo build --release --manifest-path crates/minici/Cargo.toml
sudo cp crates/minici/target/release/minici /usr/local/bin/minici

# Now run from anywhere on that new machine:
minici breakdown --path .
```

### Option 3: Run in Any GitHub Repository with Zero Install (`GitHub Action`)
Add 1 step to `.github/workflows/ci.yml`:
```yaml
- uses: Aamir-raza-1/minici-cost-action@v1
  with:
    tier: auto
    prs-per-week: 25
    dev-hourly-rate: 65
```
