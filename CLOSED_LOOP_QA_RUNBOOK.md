# Closed-Loop FinOps & CI/CD Pricing QA Runbook (`CLOSED_LOOP_QA_RUNBOOK.md`)

Use this runbook to execute or extend the **10-Session Closed-Loop QA & Invariant Verification Suite** from any terminal or separate Antigravity session.

---

## 1. One-Command Closed-Loop Verification

Run the complete deterministic test suite, 10-session invariant verifier, 10-repository + 3-cloud benchmark, and 5-source calculator cross-check:

```bash
cd /Users/araza/personal/cicd-pricing-engine
python3 -m pytest -v
python3 closed_loop_qa.py
python3 benchmark_10_repos_and_clouds.py
python3 qa_verifier.py samples/01_simple_cloudbuild_and_gha.yml
python3 qa_verifier.py samples/02_complex_monorepo_7_providers.yml
```

To also run the **Adversarial Local Ollama (`gpt-oss:20b` / `qwen2.5-coder:7b`) Receipt Auditor**:

```bash
python3 closed_loop_qa.py --with-ollama
```

---

## 2. Prompt to Paste into Another Antigravity Session for Continuous Closed-Loop QA

If you open a new Antigravity session in `/Users/araza/personal/cicd-pricing-engine` and want it to act as an adversarial closed-loop QA auditor, paste this prompt:

```text
Act as an Adversarial FinOps QA Auditor for /Users/araza/personal/cicd-pricing-engine.
1. Read LESSONS_LEARNED.md, engine/pricing.py, engine/billing_ingestor.py, and closed_loop_qa.py.
2. Run `python3 -m pytest -v` and `python3 closed_loop_qa.py`.
3. Generate new adversarial CI/CD workloads (wild custom runner labels, mixed OS allowance pools, sharded workflows with skipped gatekeeper runs, negative/zero boundary inputs, and multi-cloud storage/cache/egress/DLC taxes).
4. Verify that every output satisfies the hard invariants in `closed_loop_qa.assert_receipt_invariants`:
   - `round(gross_consumed_usd - discount_allowance_usd, 2) == round(net_billable_usd, 2)`
   - `monthly_active_runs + monthly_skipped_runs == monthly_triggered_runs`
   - `sum(pool_used) <= included_allowance` across both `ordered` and `blended` modes
5. If any edge case fails, fix it in `engine/pricing.py` or `engine/billing_ingestor.py`, add a regression test to `tests/test_pricing_7_providers.py`, and document the rule in `LESSONS_LEARNED.md`.
```
