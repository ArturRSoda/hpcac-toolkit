# Phase 7 Artifact — UCC 2026 Conference Paper
### IEEE Double-Column · 10 Pages · Blind Review · Submission Deadline Aug 19, 2026

Date: 2026-08-11
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Venue: 19th IEEE/ACM International Conference on Utility and Cloud Computing (UCC 2026)
Repository: hpcac-toolkit · Branch: tcc

---

## 1. Purpose

This artifact records the completed Phase 7 work: writing and finalizing the UCC 2026
conference paper "Transparent Resilience for Legacy HPC Applications on Cloud Spot
Instances with MANA." The paper condenses and sharpens the TCC monograph (Phase 6)
into the IEEE conference format (max 10 pages), presenting the same experimental results
(282 executions) with a tighter narrative and without the burstable-instances chapter
(TCC §4), which belongs to a separate WCC/ERAD publication.

The paper is ready for submission as of this artifact date.

---

## 2. Document Structure

| # | Section | Subsections | Pages |
|---|---|---|---|
| — | Abstract | — | ~0.3 |
| 1 | Introduction | — | ~1.0 |
| 2 | Background | §2.1 Cloud Spot Instances and Fault Tolerance; §2.2 HPC@Cloud and Job Scheduling | ~0.8 |
| 3 | Related Work | — (3 paragraphs + gap statement) | ~0.5 |
| 4 | Proposed Approach | §4.1 Architecture Overview; §4.2 System Initialization; §4.3 Failure Detection; §4.4 Recovery Strategies | ~1.5 |
| 5 | Experimental Evaluation | §5.1 Platform; §5.2 Software; §5.3 MANA Overhead; §5.4 Checkpoint Write Latency; §5.5 Recovery Strategy Comparison; §5.6 Economic Analysis | ~5.5 |
| 6 | Conclusion | — (4 paragraphs + future work) | ~0.6 |
| — | References | — | ~0.8 |
| — | **Total** | | **10 pages** |

**Figures (6 total):**

| Fig | Description | Type | Section |
|---|---|---|---|
| 1 | Fault-tolerant execution architecture | single-col | §4.1 |
| 2 | Recovery paths for Replace and Degraded | single-col | §4.4 |
| 3 | MANA overhead per benchmark and cluster size | double-col | §5.2 |
| 4 | Phase P1 duration vs. per-process memory footprint | single-col | §5.4 |
| 5 | Strategy crossover heatmap (ΔT = Replace − Degraded) | single-col | §5.5 |
| 6 | Cost per run across benchmarks and configurations | double-col | §5.6 |

**Tables (4 total):**

| Table | Description | Section |
|---|---|---|
| 1 | Cluster configuration and spot pricing | §5.1 |
| 2 | Execution and recovery phase decomposition (P0–P3) | §4.4 |
| 3 | CG-C phase decomposition by strategy and cluster size | §5.5 |
| 4 | Cost savings (%) vs. noFT on-demand baseline | §5.6 |

---

## 3. Content Decisions — What Was Cut from the TCC

| TCC Content | Paper Decision | Rationale |
|---|---|---|
| §4 Burstable instances | Removed entirely | Separate WCC/ERAD publication; not cited |
| Synthetic studies detail (synth_calls, synth_p2p) | Summarized in one §5.3 sentence | Finding is "overhead is flat ~3–5 s fixed cost"; full data not needed |
| synth_imbalanced results | Mentioned in §5.3 hypothesis paragraph | Supports the CG-C polling-imbalance hypothesis |
| §6.8 Ameaças à Validade | Folded into §4.3 assumptions + §6 future work | Conference format; no standalone validity section |
| Strong scaling figure | Cut | Not the primary story |
| Chapter/section-level summaries | Cut | Conference papers do not have running summaries |

---

## 4. Key Writing Decisions

### Style (enforced throughout)
- No em dashes (—) anywhere; rephrase or use commas/parentheses
- Recovery strategies typeset as `\textsc{Replace}` and `\textsc{Degraded}`
- One idea per sentence in analysis paragraphs
- Narrative structure: observation → mechanism → implication → transition
- No "leverage", "robust", "seamless", or other AI-sounding filler words

### §2 Background
- Spot paragraph includes periodic vs. reactive distinction; reactive framed as
  "avoiding overhead of unnecessary checkpoints while still guaranteeing a save before loss"
- MANA paragraph includes topology flexibility sentence: split-process architecture
  enables restart on N-1 workers (DEGRADED mode possible; DMTCP cannot do this)
- Slurm given its own two-sentence paragraph after HPC@Cloud paragraph

### §3 Related Work
- Taxonomy axis: "source modification vs. transparent" (not "application-level vs. system-level";
  ULFM is not a checkpoint system)
- IP trap avoided: Wang et al., BLCR/DMTCP, and Gong et al. framed as
  "boots with a fresh OS state / severs network connections" — never attributed to Slurm or MANA
- Gap statement uses "to our knowledge" (not "first to" or "none of these")

### §4 Proposed Approach
- EFS introduced as "shared network file system" at first prose mention
- Static IP justified as "existing Slurm configuration and MANA restart require no reconfiguration"
- 300-second timeout scoped to NPB runs, with forward reference to §5.4
- Assumptions stated explicitly in §4.3: 2-min notice, single failure, head/EFS available

### §5 Experimental Evaluation
- §5.4 Checkpoint Write Latency added based on Vanderlei's feedback; framed as
  "applicability characterization" (different question from overhead study) — explains why
  synth_checkpoint_size gets its own section while the other three synthetic programs do not
- CG-C P1 increase at N=8 explained mechanistically (per-partner communication buffers)
- Economic Analysis extended with fig_cost (double-col figure) alongside tab:cost

### §6 Conclusion
- 4 paragraphs: (1) what was built, (2) strategy tradeoffs, (3) experimental findings
  including checkpoint latency boundary, (4) future work
- Third paragraph explicitly mentions both viability thresholds (job duration + memory footprint)
  to fulfill the abstract's "practical viability thresholds" promise
- Future work paragraph is concrete: multiple interruptions, periodic checkpointing,
  larger clusters/providers, adaptive recovery policies

---

## 5. New Figure — fig_ckpt_latency

Generated by `gen_figs.py` (`fig_ckpt_latency()` function, added this phase).

| Property | Value |
|---|---|
| Data source | synth_checkpoint_size (4 footprint levels × 3 runs each) |
| X-axis | Memory per MPI process (MB), linear, 0–3 500 MB |
| Y-axis | Phase P1 duration (s), linear, 0–150 s |
| Data points | 50 MB → 16.4 s; 200 MB → 26.9 s; 800 MB → 47.0 s; 3 200 MB → 139.2 s |
| Regression | Linear fit via numpy.polyfit: slope 38.3 ms/MB, intercept 16.7 s |
| Reference line | Red dotted horizontal at 120 s (2-minute termination window) |
| Annotations | Point labels at each data point |
| Figure size | 3.4 × 1.8 in (COLWIDTH × height) |

Key finding: write time scales linearly at 38.3 ms/MB. At 3.2 GB/process, P1 = 139.2 s
(exceeds window). All NPB footprints well below this boundary.

---

## 6. LaTeX Compilation

```bash
cd TCC/artifacts/phase7/UCC2026-MANA
pdflatex -interaction=nonstopmode main.tex
bibtex main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```

Output: `main.pdf` — 10 pages, no errors.
Warnings: Underfull hbox/vbox (cosmetic; loose lines in abstract and Related Work). Not blocking.

---

## 7. Files Produced

### Core LaTeX
| File | Description |
|---|---|
| `UCC2026-MANA/main.tex` | Master document — all 6 sections, bibliography call |
| `UCC2026-MANA/acronyms.tex` | GLS acronym definitions (Vanderlei's original + AWS, EC2, EFS, NPB, FT, VM) |
| `UCC2026-MANA/references.bib` | ~22 BibTeX entries; all DOIs; Portuguese refs removed |
| `UCC2026-MANA/IEEEtran.cls` | IEEE conference class |
| `UCC2026-MANA/IEEEtran.bst` | IEEE bibliography style |

### Figures
| File | Description |
|---|---|
| `imgs/arch-overview-UCCversion.pdf` | Architecture diagram (labels fixed from TCC version) |
| `imgs/strategy-flow-UCCversion.pdf` | Replace vs. Degraded flow diagram |
| `imgs/fig_overhead.pdf` | MANA overhead grouped bar chart |
| `imgs/fig_ckpt_latency.pdf` | Checkpoint write latency (new; generated by gen_figs.py) |
| `imgs/fig_crossover.pdf` | Strategy crossover heatmap |
| `imgs/fig_cost.pdf` | Cost per run bar chart |

### Analysis Scripts
| File | Description |
|---|---|
| `UCC2026-MANA/gen_figs.py` | Generates all evaluation figures from raw data; added `fig_ckpt_latency()` this phase |

### Phase 7 Planning Files
| File | Description |
|---|---|
| `PHASE7_PLAN.md` | Writing plan, decisions log, acceptance gates |
| `PHASE7_ARTIFACT.md` | This file |

---

## 8. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | LaTeX compiles cleanly (no errors) | ✅ Pass |
| G2 | All sections written and internally consistent | ✅ Pass |
| G3 | All 6 figures and 4 tables present and referenced | ✅ Pass |
| G4 | Blind review fields correct (Anonymous, footnote omitted) | ✅ Pass |
| G5 | No em dashes anywhere in the text | ✅ Pass |
| G6 | Abstract fulfills its promises (both viability thresholds covered in §5 and §6) | ✅ Pass |
| G7 | All references have DOIs; Portuguese refs removed | ✅ Pass |
| G8 | Page count at or under 10 pages | ✅ Pass (10 pages exactly) |
| G9 | GenAI acknowledgment included | Deferred — at page limit; decision pending with advisor |
| G10 | Paper submitted by August 19, 2026 | Pending |

---

## 9. Known Issues (non-blocking)

| Issue | Location | Impact |
|---|---|---|
| Grammar: "neither strategy achieve" | main.tex line 714 | Minor; fix before camera-ready |
| GenAI acknowledgment missing | — | Required by CFP if AI used; adding it needs 1–2 lines trimmed elsewhere |
| fig_ckpt_latency placement | PDF page layout | Figure floats slightly far from §5.4 section header (IEEE figure* interaction); acceptable for submission |

---

## 10. File References

- TCC monograph (Phase 6): `TCC/artifacts/phase6/lapesd-thesis/`
- Phase 6 artifact: `TCC/artifacts/phase6/PHASE6_ARTIFACT.md`
- Phase 7 plan: `TCC/artifacts/phase7/PHASE7_PLAN.md`
- Paper source: `TCC/artifacts/phase7/UCC2026-MANA/`
- Analysis pipeline: `TCC/artifacts/phase7/UCC2026-MANA/gen_figs.py`
