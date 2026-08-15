# Phase 8 Artifact — Relatório Final PIBIC 2025/2026
### Portuguese · 15 Pages · LaPeSD/UFSC · PIBIC CNPq

Date: 2026-08-15
Project: Sustainable High Performance Computing on AWS
Venue: PIBIC 2025/2026 — CNPq / UFSC
Repository: hpcac-toolkit · Branch: tcc

---

## 1. Purpose

This artifact records the completed Phase 8 work: writing the final PIBIC
2025/2026 research report ("Relatório Final") for submission to Propesq/UFSC.
The document adapts the UCC 2026 paper (Phase 7) to the PIBIC report format —
Portuguese language, more explanatory tone, no Related Work section, and an
added §6 evaluating the scientific training benefits of the PIBIC cycle.

All experimental content (282 executions, 6 figures, 5 tables) comes directly
from Phase 7; no new experiments were conducted. The main work was translation,
adaptation, and restructuring for the PIBIC audience and format.

---

## 2. Document Structure

| # | Section | Subsections | Pages |
|---|---|---|---|
| — | Abstract + Palavras-chave | — | ~0.3 |
| 1 | Introdução | §1.1 Motivação; §1.2 Objetivos (Geral + Específicos) | ~1.5 |
| 2 | Fundamentação Teórica | §2.1 HPC na Nuvem e Instâncias Spot; §2.2 Checkpoint/Restart Transparente: DMTCP e MANA; §2.3 HPC@Cloud e Slurm; §2.4 NAS Parallel Benchmarks | ~2.0 |
| 3 | Proposta | §3.1 Visão Geral da Arquitetura; §3.2 Inicialização do Cluster; §3.3 Detecção de Falhas e Checkpoint; §3.4 Estratégias de Recuperação | ~2.5 |
| 4 | Avaliação Experimental | §4.1 Configuração Experimental; §4.2 Overhead do MANA; §4.3 Latência de Escrita do Checkpoint; §4.4 Comparação de Estratégias; §4.5 Análise Econômica | ~6.0 |
| 5 | Conclusão | — (3 paragraphs + future work) | ~0.8 |
| 6 | Avaliação PIBIC: Benefícios e Formação Científica | — (3 paragraphs) | ~0.5 |
| — | Referências | — | ~1.0 |
| — | **Total** | | **15 pages** |

**Figures (6 total, all reused from Phase 7):**

| Fig | Description | Width | Section |
|---|---|---|---|
| 1 | Fault-tolerant execution architecture | 0.72\linewidth | §3.1 |
| 2 | Replace vs. Degraded recovery paths | 0.50\linewidth | §3.4 |
| 3 | MANA overhead per benchmark and cluster size | 0.88\linewidth | §4.2 |
| 4 | Phase P1 duration vs. per-process memory footprint | 0.78\linewidth | §4.3 |
| 5 | Strategy crossover heatmap (ΔT = Replace − Degraded) | 0.62\linewidth | §4.4 |
| 6 | Cost per run across benchmarks and configurations | \linewidth | §4.5 |

**Tables (5 total):**

| Table | Description | Section |
|---|---|---|
| tab:hardware | Cluster configuration and spot pricing (us-west-2, May 2026) | §4.1 |
| tab:npb | NPB kernels: benchmark, class, communication pattern | §2.4 |
| tab:phases | Execution and recovery phase decomposition (P0–P3) | §3.3 |
| tab:cg-breakdown | CG-C phase breakdown by strategy and cluster size (seconds) | §4.4 |
| tab:cost | Cost savings (%) vs. noFT on-demand baseline | §4.5 |

---

## 3. Content Decisions — Differences from UCC 2026 Paper

| UCC Paper Content | Report Decision | Rationale |
|---|---|---|
| §3 Related Work | Removed entirely | Not convention for PIBIC reports |
| Double-column IEEE layout | Single-column article layout | PIBIC report format |
| English | Portuguese | PIBIC requirement |
| Paper tone ("we propose", "this paper") | Adapted ("este trabalho propõe") | Report convention |
| Phase table in §4.4 (Recovery Strategies) | Moved to §3.3 (Detecção de Falhas) | Better narrative flow: introduces phases before strategies |
| Strategy figure in §4.4 | Moved to §3.4 (Estratégias de Recuperação) | Closes the strategies section visually |
| Synthetic benchmark details | Summary sentence in §4.2 | Sufficient for PIBIC audience |
| No §6 | Added §6 Avaliação PIBIC | PIBIC requirement: scientific training self-assessment |

---

## 4. Key Writing Decisions

### Style (enforced throughout)
- No em dashes (—) anywhere; rephrase or use commas/parentheses
- No "neste artigo" — use "neste trabalho" or "neste relatório"
- Recovery strategies typeset as `\textsc{Replace}` and `\textsc{Degraded}`
- English technical terms consistently italicized (`\textit{spot}`, `\textit{job}`, etc.)
- "precoces" replaced with "no início da execução" (abstract and §5)

### §3 Structure — Table and Figure Placement
- Phase decomposition table (P0–P3) placed in §3.3, integrated with prose:
  text covers P1/P2a narrative; table provides structured naming; avoids redundancy
- Strategy-flow figure placed at end of §3.4, after describing both strategies;
  closing sentence: "evidenciando como a divergência em P2b e a convergência
  em P2c estruturam o fluxo de recuperação"

### §4.1 — t3.large as Burstable
- t3.large described as "instância burstable de propósito geral" with
  `\cite{soda2025erad,soda2025wcc}`, linking to the prior burstable
  characterization work. This is the only place in the report where "burstable"
  appears in the body, anchoring the justification for the architecture choice.

### §4.4 — Strategy Comparison Clarity
- P2b times (11 s vs. 121 s) stated explicitly before any derived figure
- "110 segundos de vantagem" derived visibly from table values (121 − 11)
- Degraded kept as explicit grammatical subject when describing the N=2 scenario
- Multi-fault caveat scoped as structural analysis only ("não avaliado
  experimentalmente neste trabalho"), preserving its validity without overstating

### §6 — Avaliação PIBIC
Three-paragraph structure:
1. Technical skills developed (Rust, AWS EC2/EFS/SSM, Slurm, MANA/DMTCP,
   experimental evaluation, scientific writing in English)
2. Publication trajectory: ERAD-RS 2025 (burstable study) → WCC/SBAC-PAD 2025
   (extension) → UCC 2026 (spot FT, current submission). Notes that burstable
   characterization informed the t3.large head-node choice, even though the
   burstable study is not the focus of the UCC paper.
3. HPC@Cloud contribution: FT module (watcher + reactive checkpoint pipeline +
   two recovery strategies), enabling automatic recovery on spot instances
   without application modification.

---

## 5. LaTeX Compilation

```bash
cd TCC/artifacts/phase8/relatorio-pibic-2025-2026
latexmk -pdf -interaction=nonstopmode main.tex
```

Output: `main.pdf` — 15 pages, no errors.
Warning: `legacy month field 'mar' in entry 'aws2018spot'` — pre-existing in
references.bib; cosmetic only, does not affect output.

---

## 6. Files Produced

### Core LaTeX
| File | Description |
|---|---|
| `relatorio-pibic-2025-2026/main.tex` | Master document — all 6 sections + bibliography |
| `relatorio-pibic-2025-2026/acronyms.tex` | GLS acronym definitions |
| `relatorio-pibic-2025-2026/references.bib` | ~30 BibTeX entries (biblatex/biber); includes soda2025erad and soda2025wcc |

### Figures (reused from Phase 7)
| File | Description |
|---|---|
| `imgs/arch-overview-UCCversion.pdf` | Architecture diagram |
| `imgs/strategy-flow-UCCversion.pdf` | Replace vs. Degraded flow diagram |
| `imgs/fig_overhead.pdf` | MANA overhead grouped bar chart |
| `imgs/fig_ckpt_latency.pdf` | Checkpoint write latency |
| `imgs/fig_crossover.pdf` | Strategy crossover heatmap |
| `imgs/fig_cost.pdf` | Cost per run bar chart |

### Header Assets
| File | Description |
|---|---|
| `imgs/ufsc.jpg` | UFSC logo (fancyhdr header) |
| `imgs/ine.pdf` | INE logo (fancyhdr header) |

### Phase 8 Planning Files
| File | Description |
|---|---|
| `PHASE8_PLAN.md` | Writing plan, content mapping (UCC → report), acceptance gates |
| `PHASE8_ARTIFACT.md` | This file |

---

## 7. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | LaTeX compiles without errors after each section | ✅ Pass |
| G2 | No occurrence of "neste artigo" or "this paper" | ✅ Pass |
| G3 | All 6 figures present and referenced | ✅ Pass |
| G4 | §6 Avaliação PIBIC written and complete | ✅ Pass |
| G5 | General revision and final compilation | ✅ Pass (15 pages, no errors) |

---

## 8. File References

- UCC 2026 paper (Phase 7): `TCC/artifacts/phase7/UCC2026-MANA/`
- Phase 7 artifact: `TCC/artifacts/phase7/PHASE7_ARTIFACT.md`
- Phase 8 plan: `TCC/artifacts/phase8/PHASE8_PLAN.md`
- Report source: `TCC/artifacts/phase8/relatorio-pibic-2025-2026/`
- Phase 9 plan (SIC seminar): `TCC/artifacts/phase9/PHASE9_PLAN.md`
