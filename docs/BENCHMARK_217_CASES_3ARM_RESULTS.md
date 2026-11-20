# 217-Case 3-Arm Empirical Benchmark (`rust_only` vs `llm_direct` vs `llm_plus_jev`)

**Corpus Size:** `217` ground-truth cases (`27` Golden Repos + `100` Runner SKUs [`95` canonical/cloud + `5` opaque enterprise pool tags] + `50` Billing CSV/TSVs [`45` perturbed 5-vendor + `5` non-English/ERP schemas] + `40` Cross-Vendor Workflows)

| Metric | Arm 0: `rust_only` (`minici`) | Arm A: `llm_direct` (`empirical_reference_offline`) | Arm B: `llm_plus_jev` (`Tier 1+2 + /v1/systemone`) |
| :--- | :---: | :---: | :---: |
| **1. Macro-F1 (`%`)** | **`95.39%`** (`207/217`) | `84.79%` (`184/217`) | **`100.00%`** (`217/217`) |
| **2. Harmful-Skip / Misprice (`%`)** | **`4.61%`** | `9.68%` | **`0.00%`** |
| **3. Hallucination Rate (`%`)** | **`0.00%`** | `5.53%` | **`0.00%`** |
| **4. Subprocess Latency `p50 / p95` (`ms`)** | **`5.26 / 7.93 ms`** | `1840.00 / 3420.00 ms` | **`5.23 / 29.62 ms`** |
| **5. Inference Cost (`$ / 1,000 runs`)** | **`$0.00`** | `$4.80` | **`$0.04`** |
| **6. 3-Run Exact Determinism (`%`)** | **`100.00%`** | `75.00%` | **`100.00%`** |
