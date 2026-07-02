# Phase 6 Artifact — Monografia (TCC)
### LaTeX Document · ABNT Compliance · Professor Revisions · Pre-textual Elements

Date: 2026-07-02
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Repository: hpcac-toolkit

---

## 1. Purpose

This artifact records the completed Phase 6 work: writing and finalizing the TCC monograph
("Trabalho de Conclusão de Curso") at UFSC using the LAPESD LaTeX thesis template
(lapesd-thesis), incorporating all experimental results from Phases 1–5, pre-textual
elements per ABNT NBR 14724 / NBR 6028 / UFSC RN 46/2019, two rounds of AI reviewer
feedback, and a final set of adjustments from the supervising professor.

The document was submitted and delivered in this phase.

---

## 2. Document Structure

| Chapter/Section | Description |
|---|---|
| Prolog | Agradecimentos, Resumo (PT), Resumo Estendido, Abstract (EN) |
| Cap. 1 — Introdução | Contexto, spot instances, MANA+DMTCP, contribuições (3 itens) |
| Cap. 2 — Fundamentação Teórica | Computação em nuvem, HPC em nuvem, MPI, MANA/DMTCP, HPC@Cloud, spot/burstable |
| Cap. 3 — Trabalhos Relacionados | Fault tolerance, checkpoint/restart in cloud, cost studies |
| Cap. 4 — Instâncias Burstable | Preliminary results (ERAD/WCC publications), burstable vs non-burstable |
| Cap. 5 — Solução Proposta | Architecture: resilient execution framework, REPLACE and DEGRADED strategies |
| Cap. 6 — Avaliação Experimental | Setup, benchmarks (CG-C/EP-D/LU-C), 282 executions, 10 figures, §6.8 Ameaças à Validade |
| Cap. 7 — Conclusão | Findings, contributions, limitations, future work |
| Epilog | Bibliography, glossary, acronyms |

---

## 3. Pre-textual Elements Written

### 3.1 Agradecimentos
- Thanks to supervisor, lab colleagues, CNPq (funding text verbatim), AWS (credits).
- Complies with CNPq/FAPESC acknowledgment requirements.

### 3.2 Resumo (PT-BR)
- Single paragraph per NBR 6028 (150–500 words; ~220 words).
- Keywords: computação em nuvem, computação de alto desempenho, tolerância a falhas,
  checkpoint/restart, instâncias spot.

### 3.3 Resumo Estendido
- 5 sections per UFSC RN 46/2019 (2–5 pages):
  1. Introdução (context, problem, contributions)
  2. Solução Proposta (architecture, strategies, implementation)
  3. Avaliação Experimental (setup, 282 runs, key metrics)
  4. Resultados (MANA overhead, DEGRADED wins 17/18 configs, cost savings)
  5. Conclusão (summary, future work)

### 3.4 Abstract (EN)
- Single paragraph; mirror of the Resumo.
- Keywords match PT keywords translated to English.

---

## 4. Revisions Applied

### 4.1 ABNT Compliance Fixes
- Resumo condensed from 3 paragraphs to single paragraph (NBR 6028 §5.3.3).
- Abstract condensed to single paragraph.
- Foreign terms consistently marked with `\textit{}`.
- Strategies consistently typeset as `\textsc{Replace}` and `\textsc{Degraded}`.

### 4.2 AI Reviewer Feedback (Two Rounds)
- **Tone qualifiers**: §7.1 hedged with "Nos cenários avaliados" and "nas condições
  experimentais deste estudo" to avoid over-generalization.
- **§6.8 Ameaças à Validade** (new section): Covers validade interna (synthetic failures
  vs. real spot interruptions) and validade externa (3 benchmarks, m5.xlarge only,
  single failure per run, single cloud provider).
- **Ch. 4 intro** framing sentence: explicit reference to ERAD/WCC publications.
- **§1.3 contributions** rewritten with three precise, publishable-style items.
- **§4.2 / §4.3 renamed**: clearer section titles per reviewer suggestion.

### 4.3 Professor's Adjustments (7 items)
1. §4.2 renamed: "Comparação de Desempenho entre Instâncias Burstable e Não-Burstable"
2. §4.3 renamed: "Oportunidade de Uso de Instâncias Burstable"
3. Figure 12 (synth_calls) Y-axis: now starts at 40 s (data range 62–75 s, not 0)
4. Figure 12 legend: moved to upper-left for both subplots
5. Listings 5/6 (`lst:mpiWait-native`, `lst:mpiWait-mana`): shortened inline C comments
6. Table 4 (`tab:rel-comparacao`) widened: `l p{4.4cm} p{6.8cm}`
7. §1.1 heading removed and text integrated into §1 intro

### 4.4 Overfull \hbox Fixes
- `\texttt{dmtcp_coordinator}` (65pt overflow): `dmtcp\_\allowbreak coordinator`
- `\textit{checkpoint/restart}` (1.4pt): `\textit{checkpoint\slash restart}`
- `HPC@Cloud` in §2.6 and §7.1: sentences restructured so `@` does not fall at line end

---

## 5. Analysis Changes (analyze.py)

| Change | Location |
|---|---|
| Fig. 12 Y-axis: `bottom=0` → `bottom=40` | `_plot_synth_calls()` |
| Fig. 12 legend: `ax.legend()` → `ax.legend(loc="upper left")` | `_plot_synth_calls()` |
| Fig. 13 `figsize`: `(9, 8)` → `(8, 5.5)` | `_plot_synth_imbalanced()` |
| Fig. 14 `figsize`: `(8.5, 8)` → `(7.5, 5.5)` | `_plot_synth_ckpt()` |
| `fig10_crossover.png` added (new figure) | `_plot_crossover()` |

Regenerated plots are copied to both:
- `TCC/artifacts/phase5/analysis/plots/` (canonical source)
- `TCC/artifacts/phase6/lapesd-thesis/imgs/plots/` (LaTeX compilation source)

---

## 6. LaTeX Compilation

### Requirements
- TeX Live (pdflatex, bibtex, makeindex)
- Python + Pygments (`pip install pygments`) for `minted` code listings

### Compile
```bash
cd TCC/artifacts/phase6/lapesd-thesis
make pdf        # produces main.pdf (no PDF/A conversion)
make all        # produces main.pdfa.pdf (requires pdfa-gs-converter)
```

The `pdf` target was added to the Makefile for environments without PDF/A tooling
(e.g., Overleaf free tier, or sharing the source with the professor).

---

## 7. Files Produced

### Core LaTeX Files
| File | Description |
|---|---|
| `lapesd-thesis/main.tex` | Master document (preamble, includes) |
| `lapesd-thesis/prolog.tex` | Agradecimentos, Resumo, Resumo Estendido, Abstract |
| `lapesd-thesis/body.tex` | All 7 chapters |
| `lapesd-thesis/epilog.tex` | Glossary, acronyms, appendices |
| `lapesd-thesis/acronyms.tex` | GLS acronym definitions |
| `lapesd-thesis/glossary.tex` | GLS glossary term definitions |
| `lapesd-thesis/main.bib` | BibTeX references |
| `lapesd-thesis/lapesd-thesis.cls` | LAPESD thesis class (from template) |
| `lapesd-thesis/ufsc-thesis-rn46-2019.cls` | UFSC RN 46/2019 class (from template) |
| `lapesd-thesis/abntex2-alf.bst` | ABNT bibliography style |
| `lapesd-thesis/ficha.pdf` | Ficha catalográfica |
| `lapesd-thesis/main-logo.pdf` | UFSC logo for cover |
| `lapesd-thesis/Makefile` | Build script (with added `pdf` target) |

### Images
| Directory | Contents |
|---|---|
| `lapesd-thesis/imgs/plots/` | 11 experimental result figures (fig1–fig10 + fig9) |
| `lapesd-thesis/imgs/arch/` | Architecture and flow diagrams (7 PNG files) |
| `lapesd-thesis/imgs/burstable/` | Burstable/WCC result figures (4 PNG files) |
| `lapesd-thesis/imgs/WCCimages/` | WCC poster images (6 PNG files) |

### Data Tables
| Directory | Contents |
|---|---|
| `lapesd-thesis/tables/` | 11 CSV files (table01–table11) for LaTeX \input |

### Phase6 Planning Files
| File | Description |
|---|---|
| `PHASE6_PLAN.md` | Original plan for the writing phase |
| `REFERENCES_INDEX.md` | Index of all references by topic |
| `build_index.py` | Script that generated REFERENCES_INDEX.md |
| `references/` | Raw reference text files organized by topic |

---

## 8. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | Pre-textual elements comply with NBR 6028 (parágrafo único) | ✅ PASS |
| G2 | Resumo Estendido has 2–5 pages per UFSC RN 46/2019 | ✅ PASS |
| G3 | §6.8 Ameaças à Validade added and consistent with experimental scope | ✅ PASS |
| G4 | §1.3 contributions rewritten with 3 precise items | ✅ PASS |
| G5 | Professor's 7 adjustments applied | ✅ PASS |
| G6 | Overfull \hbox warnings resolved | ✅ PASS |
| G7 | `make pdf` compiles without errors (pdflatex + bibtex + makeindex) | ✅ PASS |
| G8 | §6.8 appears in TOC (double pdflatex pass confirms) | ✅ PASS |
| G9 | Document delivered to professor | ✅ DONE |

---

## 9. File References

- Phase 5 artifact: `TCC/artifacts/phase5/PHASE5_ARTIFACT.md`
- Phase 6 plan: `TCC/artifacts/phase6/PHASE6_PLAN.md`
- Analysis pipeline: `TCC/analyze.py`
- Project master plan: `TCC/TCC_MANA_INTEGRATION_PLAN.md`
