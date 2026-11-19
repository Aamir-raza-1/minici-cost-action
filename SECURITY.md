# 🔒 Security Policy & Read-Only Least-Privilege Architecture (`minici`)

## 1. Zero Proprietary Source-Code Access (`Contents: None`)

`minici` is architected from the ground up for security-conscious engineering organizations and CISOs:

1. **Zero Source-Code Read Access Required in API Mode:** When syncing live GitHub Actions telemetry (`minici sync` or `minici breakdown --live`), `minici` requires **`Contents: None`**. It never clones, downloads, or uploads your application source code.
2. **Zero External Runtime Crates:** `crates/minici/Cargo.toml` has **`0` external crate dependencies** (`100%` Rust `std` library only), eliminating supply-chain `build.rs` and transitive crate risks.
3. **Zero Write Permissions Ever Requested for Analysis:** `minici` actively inspects OAuth/PAT headers via `minici permissions check` and **prints an explicit Security Advisory warning** if a developer accidentally passes an over-privileged `repo` / `workflow` / `write:*` token, guiding them to downgrade to a Read-Only Fine-Grained PAT.

---

## 2. Five Progressive Read-Only Telemetry Tiers (`108` FinOps Attributes)

Run `minici permissions check --repo owner/repo` at any time to verify your exact token posture and unlocked FinOps telemetry coverage:

| Tier | Mode | Required Token Permissions | Source Code Access | Attributes Unlocked |
| :--- | :--- | :--- | :---: | :---: |
| **Tier 0** | Local Offline Static YAML (`minici breakdown --path .`) | **None (`0` network calls)** | Workflow YAML only | **`33 / 108` (`30.6%`)** |
| **Tier 1** | Public Repo Live API (`minici sync --repo owner/repo`) | **None (`Zero-Token` Public API + `ETag 304`)** | **`None`** | **`68 / 108` (`63.0%`)** |
| **Tier 2** | Private Repo Fine-Grained Read PAT | `Actions: Read`, `Metadata: Read` | **`None`** | **`89 / 108` (`82.4%`)** |
| **Tier 3** | Offline Billing CSV (`--billing-csv focus.csv`) | **None (`100%` Air-Gapped CSV)** | **`None`** | **`99 / 108` (`91.7%`)** |
| **Tier 4** | Fine-Grained Read PAT + Billing CSV | `Actions: Read`, `Metadata: Read`, `Org Admin: Read` | **`None`** | **`108 / 108` (`100.0%`)** |

> **Note on Sticky PR Comments (`minici comment github --post`):** Only when you opt into posting sticky PR comments inside GitHub Actions does the workflow job use GitHub's ephemeral `GITHUB_TOKEN` with `pull-requests: write` scoped strictly to that pull request.

---

## 3. Reporting a Vulnerability

Please report security issues privately to **Aamir Raza** at **`raza.aamir01@gmail.com`**. All reports receive an acknowledgment within 24 hours.
