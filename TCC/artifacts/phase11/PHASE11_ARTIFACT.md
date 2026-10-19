# Phase 11 Artifact — Integrating ADAPTIVE into the TCC Monograph

### New Chapter 6 Section · Chapter 5/1/7 Updates · Pointwise Correction Checklist · Resumo Estendido Removal

Date: 2026-10-18
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

---

## 1. Purpose

This artifact records the completed Phase 11 work: integrating the Phase 10 findings
(the `ADAPTIVE` recovery strategy — heuristic derivation, N-independence validation,
implementation, integration testing, and the coefficient-calibration exploration) into
the already-defended TCC monograph (`lapesd-thesis`), originally finalized in Phase 6.

All edits were made on a **new working copy** (`TCC/artifacts/phase11/lapesd-thesis/`),
created explicitly so the Phase 6 original (`TCC/artifacts/phase6/lapesd-thesis/`)
stays untouched as a frozen reference. Every change was cross-checked against the
actual numbers in `PHASE10_PLAN.md`/`PHASE10_ARTIFACT.md` and the implementation
source (`watcher.rs`), rather than re-derived from memory, and reviewed with the user
section by section before being applied.

---

## 2. Document Structure Changes

| Chapter/Section | Change |
|---|---|
| Prolog — Resumo (PT) | Rewritten: Adaptive introduced as a third option that automates the Replace/Degraded choice, with its own validation result stated |
| Prolog — Abstract (EN) | Mirrored fix |
| Prolog — Resumo Estendido | **Removed entirely** (advisor-authorized in a prior phase, executed in Phase 11 on explicit user request) |
| Cap. 1 — Objetivos Específicos | New item 7: deriving and validating the `ADAPTIVE` heuristic (now 7 objectives, not 6) |
| Cap. 1 — Contribuições | New bullet: heuristic derivation, out-of-sample validation, live integration testing, and the demonstrated workload-generalization limit |
| Cap. 1 — Organização do Trabalho | Updated to mention the new Capítulo 6 section and to correctly scope the "282 execuções" claim to the NPB study, with the 90 additional `ADAPTIVE`-related executions noted separately |
| Cap. 5 — Estratégias de Recuperação | Opening sentence and YAML listing updated to introduce `ADAPTIVE` as a third, automated option (not a third recovery mechanism); three new paragraphs describing the mechanism symbolically (no real coefficient/threshold values — those live in Capítulo 6) |
| Cap. 6 — new Seção "Validação da Estratégia Adaptive" | Entirely new, ~15 pages, inserted between Análise de Custo and Discussão (see §3 below) |
| Cap. 6 — Discussão | Restructured from 3 dense paragraphs into 8 focused ones; corrected the naive "fraction of capacity" mechanistic explanation (same fix applied in 3 other places, see §4); added a 2-paragraph Adaptive synthesis |
| Cap. 6 — Ameaças à Validade | Added a cross-reference from the abstract "other workloads may behave differently" claim to the concrete, measured Adaptive miscalibration evidence |
| Cap. 7 — Considerações Finais | Adaptive added to the Capítulo 5 summary clause; new paragraph synthesizing its validation and limitation; "seis" → "sete objetivos" fixed, with objective 7 covered in the achievement list |
| Cap. 7 — Trabalhos Futuros | New paragraph on continuing the automatic-calibration exploration, grounded in `PHASE10_PLAN.md`'s own documented next steps |

---

## 3. New Chapter 6 Section: "Validação da Estratégia Adaptive"

Mirrors the structure of `analysis_report_phase10_ptbr.md`, adapted to the monograph's
denser, less didactic tone (the committee already read Capítulo 5's mechanism
description, so no re-explanation).

| Subsection | Content |
|---|---|
| 6.7.1 Derivação da Heurística Empírica | 4 sub-parts: O Modelo Ingênuo (13/18 on the naive 1/(N−1) model); Extração do Coeficiente Empírico (full 18-row coefficient table, `remaining_secs` traced back to the measured MANA-noFT baseline); Escolha do Coeficiente e Fórmula Final (the two binding EP-D N=2 configurations that bound the valid coefficient range); Por que o Coeficiente Não Depende de N (redistribution math + hyperthreading idle-capacity absorption, written at the detail level requested — concrete examples, no unexplained jargon) |
| 6.7.2 Validação de Independência de N | `synth_duration` design (fixed-iteration-count loop, trimmed code listing), the 14-config/84-run matrix, both regenerated figures, the 3-row coefficient table, and an explicit derivation of why the crossover lands at ≈210s from the same 109s/coefficient formula |
| 6.7.3 Limitação Identificada | The coefficient-mismatch demonstration (`synth_duration` N=2 f=50%), reframed as a genuine open question investigated rather than a foregone conclusion, with the Replace/Degraded/Adaptive outcome table |
| 6.7.4 Implementação e Testes de Integração | Merged into one short subsection; the two unambiguous cases plus the 3-repetition boundary case, in a compact table |
| 6.7.5 Exploração de Calibração Automática | The `perf stat` infeasibility finding, the `coef=(1-r)/r` derivation, and both density-calibration result tables (compute-bound, memory-bound kernels) — kept proportionate to how inconclusive the finding is |

**Figures reused from Phase 10** (`fig_synth_duration_boundary.png`,
`fig_coefficient_n_independence.png`): regenerated via `analyze_synth_duration.py`
with the embedded titles removed (moved to LaTeX `\caption{}`, matching every other
figure in the document), renumbered `fig11`/`fig12`. The coefficient-comparison figure
(`fig12`) was ultimately **not used** in the final text — the table alone already
carries the same 3 numbers, and keeping the figure would have duplicated it.

**New tables**: `tab:adaptive-coef` (18 rows), `tab:synth-duration-coef` (3 rows),
`tab:adaptive-descompasso` (3 rows), `tab:adaptive-integracao` (3 rows),
`tab:calib-cpu-bound` (4 rows), `tab:calib-memory-bound` (4 rows).

**New bibliography entries** (`main.bib`): `aws2026m5`, `intel2017xeon8175m` — cited to
source the m5.xlarge cache-size claim used to justify why `synth_duration`'s 50MB
buffer is memory-bandwidth-bound.

---

## 4. Pointwise Correction Checklist (20 items, all resolved)

Full detail in `PHASE11_PLAN.md`; summary by category:

- **Terminology ("duas estratégias" → "duas estratégias... uma terceira opção, a
  Adaptive")**: applied consistently across Resumo, Abstract, Introdução, Objetivos
  Específicos, Capítulo 5's opening and roadmap sentences, and Capítulo 7 — deliberately
  *not* phrased as "três estratégias," since Adaptive automates a choice between two
  recovery mechanisms rather than adding a third one.
- **Naive "fraction of capacity" mechanistic error** (the pre-Phase-10 explanation that
  losing a node matters less in larger clusters because it's "a smaller capacity
  fraction," equivalent to the discredited 1/(N−1) model): corrected in 4 locations —
  Capítulo 6's CG-C breakdown, Seção 6.5.1, Seção 6.5.2, and Discussão — replaced with
  the verified mechanism (the job finishes faster with more workers, so less work is
  left when the fault hits; the coefficient itself stays flat).
- **CG-C-specific findings incorporated into the new section's reasoning**: the ≈110s
  CG-C P2b gap cited as independent confirmation that 109s is an AWS infrastructure
  fact, not benchmark-specific; the CG-C P3 coefficient collapse (1.81→0.33→0.008,
  far steeper than EP-D/LU-C's near-flat coefficient) connected, as an explicit
  hypothesis and not a confirmed fact, to the existing `synth_imbalanced`
  active-loop/co-location finding.
- **"5 e 34%" → "5 e 37%"**: the real source error — `tab:warm-restart-gap` was
  missing its 6 N=2 rows entirely (data already existed in `PHASE10_PLAN.md`, just
  never inserted). Added, and the percentage range corrected in all 3 locations that
  repeated it (Seção 6.4.2, Discussão, Trabalhos Futuros), plus a dependent fix to a
  "contadores de hardware" future-work suggestion that Phase 10's own Etapa 13 already
  showed is infeasible on non-`.metal` AWS instances.
- **Execution-count scoping**: "282 execuções" is specific to the NPB fault-injection
  matrix; corrected 2 locations where it was ambiguously used to describe the whole
  chapter, now paired with "complementadas por... mais 90 execuções" wherever the
  broader scope is intended.
- **Content opportunities** (not corrections): a new cost-extrapolation paragraph in
  Análise de Custo, mirroring the time-based extrapolation already in Seção 6.5.2; a
  concrete-evidence cross-reference added to Ameaças à Validade's external-validity
  paragraph.

---

## 5. Working Method

Followed the user's explicit process for this phase: read the monograph section by
section before editing (not all at once), catalog findings as visible `\todo[inline]`
boxes in the compiled PDF rather than invisible LaTeX comments, draft every
non-trivial piece of prose in chat for review before applying it, and recompile after
every change to catch broken references immediately. Two real slips were caught this
way before they reached the final document: an `Edit` call that duplicated an
existing paragraph instead of extending it, and a Trabalhos Futuros sentence that
referenced a "decoy" test which had been trimmed out of Seção 6.7.5 earlier in the
session (a dangling reference to content no longer in the document).

---

## 6. LaTeX Compilation

### Requirements
Same as Phase 6, plus `-shell-escape` (required by `minted` for the new C code listing
in Seção 6.7.2).

### Compile
```bash
cd TCC/artifacts/phase11/lapesd-thesis
pdflatex -shell-escape -interaction=nonstopmode main.tex
bibtex main
pdflatex -shell-escape -interaction=nonstopmode main.tex
pdflatex -shell-escape -interaction=nonstopmode main.tex
```

Final state: **110 pages**, compiles cleanly, zero undefined references, zero
`[FASE11]` markers remaining in `body.tex`/`prolog.tex`.

---

## 7. Files Produced / Changed

### New working copy (full compilable thesis)
| Path | Description |
|---|---|
| `lapesd-thesis/` | Complete copy of the Phase 6 thesis; this is now the only version being edited |
| `lapesd-thesis/imgs/plots/fig11_synth_duration_boundary.png` | Regenerated, embedded title removed |
| `lapesd-thesis/imgs/plots/fig12_coefficient_n_independence.png` | Regenerated, embedded title removed (ultimately unused in final text) |
| `lapesd-thesis/imgs/arch/watcher-flow.png` | Edited by the user to show the `ADAPTIVE` decision point |
| `lapesd-thesis/imgs/arch/strategy-flow.png` | Edited by the user to add the third branch |

### Modified source files
| File | Change |
|---|---|
| `lapesd-thesis/main.tex` | `todonotes` package added for the review markers (removal deferred, see §9) |
| `lapesd-thesis/prolog.tex` | Resumo/Abstract rewritten; Resumo Estendido removed |
| `lapesd-thesis/body.tex` | All changes described in §2–4 |
| `lapesd-thesis/main.bib` | 2 new entries (`aws2026m5`, `intel2017xeon8175m`) |

### Regeneration script
| File | Change |
|---|---|
| `TCC/artifacts/phase10/analysis/analyze_synth_duration.py` | Titles removed from both figures (moved to LaTeX captions); y-axis label on the coefficient figure simplified |

### Planning
| File | Description |
|---|---|
| `PHASE11_PLAN.md` | Full decision log: structure decision, 20-item correction checklist, figures checklist, per-Etapa changelogs |
| `PHASE11_ARTIFACT.md` | This document |

---

## 8. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | Full read-through of `body.tex`, inconsistencies cataloged with the user before any edit | ✅ PASS |
| G2 | Structure decision (Cap. 5 mechanism + new Cap. 6 section, no new chapter) confirmed with the user | ✅ PASS |
| G3 | Adaptive subsection in Capítulo 5 written, consistent with Replace/Degraded's existing tone; both flow figures updated | ✅ PASS |
| G4 | New Capítulo 6 section written, figures de-duplicated against Capítulo 5, no repeated explanations | ✅ PASS |
| G5 | All 20 pointwise-correction checklist items applied and re-verified | ✅ PASS |
| G6 | Execution counts consistent everywhere (no implied-but-wrong "282 is the grand total") | ✅ PASS |
| G7 | Capítulo 1 and Capítulo 7 updated to reflect `ADAPTIVE` | ✅ PASS |
| G8 | Clean compile, no broken references; **incremental page-by-page visual review done throughout, final end-to-end read-through still pending** | 🟡 PARTIAL |
| G9 | Resumo Estendido removed only on the user's explicit request, not in passing | ✅ PASS |
| G10 | All `\todo`/`todonotes` markers removed from `body.tex`/`prolog.tex`; **`todonotes` package itself still in `main.tex`, removal deferred to next session** | 🟡 PARTIAL |

---

## 9. Remaining Work (deferred to next session, by explicit user request)

1. Remove the `todonotes` package (and its explanatory comment) from `main.tex` — now
   safe, since every `[FASE11]` marker is gone.
2. One final full end-to-end read-through of the compiled PDF (page breaks, float
   placement, general flow) — distinct from the section-by-section review already done
   throughout this phase.

Both are small; nothing else from the original Phase 11 plan is outstanding.

---

## 10. File References

- Phase 11 plan (full decision log): `TCC/artifacts/phase11/PHASE11_PLAN.md`
- Phase 10 artifact (source of all `ADAPTIVE` findings integrated here):
  `TCC/artifacts/phase10/PHASE10_ARTIFACT.md`
- Phase 10 analysis reports: `TCC/artifacts/phase10/analysis/analysis_report_phase10.md`,
  `analysis_report_phase10_ptbr.md`
- Phase 6 artifact (original monograph, now frozen): `TCC/artifacts/phase6/PHASE6_ARTIFACT.md`
- Working copy (active document): `TCC/artifacts/phase11/lapesd-thesis/`
- Frozen original (do not edit): `TCC/artifacts/phase6/lapesd-thesis/`
- Project master plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
