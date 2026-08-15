# Phase 8 Plan — Relatório Final PIBIC 2025/2026

Date: 2026-08-13
Projeto: Sustainable High Performance Computing on AWS
Bolsista: Artur Luiz Rizzato Toru Soda
Orientador: Prof. Dr. Márcio Bastos Castro
Instituição: LaPeSD, INE/UFSC

---

## Status

| Etapa | Atividade | Status |
|---|---|---|
| 0 | Planejamento e esqueleto do documento | ✅ |
| 1 | Abstract | ✅ |
| 2 | §1 Introdução (Motivação + Objetivos) | ✅ |
| 3 | §2 Fundamentação Teórica | ✅ |
| 4 | §3 Proposta | ✅ |
| 5 | §4 Avaliação Experimental | ✅ |
| 6 | §5 Conclusão | ✅ |
| 7 | §6 Avaliação PIBIC | ✅ |
| 8 | Revisão geral e compilação final | ✅ |

---

## 1. Objetivo

Escrever o relatório final do ciclo PIBIC 2025/2026, adaptando o conteúdo do artigo
submetido ao UCC 2026 para o formato de relatório PIBIC adotado pelo LaPeSD/UFSC.

A fonte principal é o artigo do UCC (inglês), não a monografia TCC. O artigo já está
na extensão e no nível de condensação adequados; o principal trabalho é tradução e
adaptação de tom, não reescrita de conteúdo. Não há experimentos novos.

---

## 2. Fontes

| Fonte | Uso |
|---|---|
| `TCC/artifacts/phase7/UCC2026-MANA/main.tex` | **Fonte principal** — todo o conteúdo técnico |
| `TCC/artifacts/phase6/lapesd-thesis/body.tex` | Consulta pontual (figuras, tabelas, detalhes) |
| `Downloads/Relatório_Final_PIBIC_2024_2025___Artur_Soda/main.tex` | Template LaTeX |

---

## 3. Template

- Classe: `article`, 11pt, a4paper
- Header: logos UFSC + INE (`fancyhdr`)
- Bib: `biblatex` com backend `biber`, estilo numérico
- Título: `Relatório Final - PIBIC 2025/2026`
- Projeto: `Sustainable High Performance Computing on AWS`

---

## 4. Mapeamento de Conteúdo (UCC → Relatório)

| Seção do relatório | Origem no UCC | Ação |
|---|---|---|
| Abstract | §Abstract | Traduzir; adaptar para pt-BR; paragrafo único |
| §1 Introdução | §1 Introduction | Adaptar; retirar "this paper" e linguagem de artigo |
| §1.1 Motivação | §1 parágrafos 1–3 | Adaptar; focar em spot + FT + MANA |
| §1.2 Objetivos | §1 contributions (itemize) | Reformular como Objetivo Geral + Objetivos Específicos |
| §2.1 HPC na Nuvem e Instâncias Spot | §2.1 Cloud Spot Instances and Fault Tolerance | Adaptar; expandir contexto de HPC em nuvem |
| §2.2 Checkpoint/Restart Transparente | §2.1 (DMTCP + MANA paragraphs) | Adaptar; manter split-process como ponto central |
| §2.3 HPC@Cloud e Slurm | §2.2 HPC@Cloud and Job Scheduling | Adaptar |
| §2.4 NAS Parallel Benchmarks | §4.2 Software (NPB block) | Condensar para tabela + 1 parágrafo |
| §3.1 Visão Geral da Arquitetura | §4.1 Architecture Overview | Adaptar |
| §3.2 Inicialização do Cluster | §4.2 System Initialization | Adaptar |
| §3.3 Detecção de Falhas e Checkpoint | §4.3 Failure Detection and Checkpointing | Adaptar |
| §3.4 Estratégias de Recuperação | §4.4 Recovery Strategies | Adaptar; manter tabela de fases P0–P3 |
| §4.1 Configuração Experimental | §4.1 Platform + §4.2 Software | Adaptar; tabela de instâncias + stack de software |
| §4.2 Overhead do MANA | §4.3 MANA Overhead | Adaptar; manter fig_overhead |
| §4.3 Latência de Checkpoint | §4.4 Checkpoint Write Latency | Adaptar; manter fig_ckpt_latency |
| §4.4 Comparação de Estratégias | §4.5 Recovery Strategy Comparison | Adaptar; manter tabela CG + fig_crossover |
| §4.5 Análise Econômica | §4.6 Economic Analysis | Adaptar; manter tabela de savings + fig_cost |
| §5 Conclusão | §5 Conclusion | Adaptar; condensar para 2–3 parágrafos |
| §6 Avaliação PIBIC | — | Escrever do zero (ver §7) |

---

## 5. Figuras

Reutilizar os PDFs gerados na fase 7. Nenhuma figura nova.

| # | Figura | Arquivo | Seção |
|---|---|---|---|
| 1 | Arquitetura do sistema | `phase7/UCC2026-MANA/imgs/arch-overview-UCCversion.pdf` | §3.1 |
| 2 | Fluxo das estratégias | `phase7/UCC2026-MANA/imgs/strategy-flow-UCCversion.pdf` | §3.4 |
| 3 | Overhead do MANA | `phase7/UCC2026-MANA/imgs/fig_overhead.pdf` | §4.2 |
| 4 | Latência de checkpoint | `phase7/UCC2026-MANA/imgs/fig_ckpt_latency.pdf` | §4.3 |
| 5 | Heatmap crossover | `phase7/UCC2026-MANA/imgs/fig_crossover.pdf` | §4.4 |
| 6 | Custo por execução | `phase7/UCC2026-MANA/imgs/fig_cost.pdf` | §4.5 |

---

## 6. O Que Cortar

- Seção de Trabalhos Relacionados (§3 do UCC) — não é convenção de relatório PIBIC
- Linguagem de paper: "this paper", "we propose", "our contribution" → "este trabalho", "propõe-se"
- Synthetic benchmark details (synth_calls, synth_p2p, synth_imbalanced) — manter apenas o resultado geral de overhead
- Detalhes de implementação Rust/código — não relevantes para relatório PIBIC

---

## 7. Decisões de Escrita

- Idioma: português
- Tom: mais explicativo que o paper; menos formal que a monografia
- Estratégias: sempre `\textsc{Replace}` e `\textsc{Degraded}`
- Sem travessão (—) em nenhuma parte do texto
- Sem "neste artigo" → usar "neste trabalho" ou "neste relatório"
- Siglas introduzidas por extenso na primeira ocorrência
- UCC submetido ao IEEE/ACM UCC 2026; resultado de avaliação pendente

---

## 8. Seção Avaliação PIBIC

Escrever do zero com os seguintes pontos:

- Habilidades técnicas desenvolvidas: Rust, AWS (EC2/SSM/EFS), Slurm, MANA/DMTCP,
  análise experimental, escrita científica em inglês
- Publicação anterior: artigo WCC/SBAC-PAD 2025 (instâncias burstable)
- Publicação atual: artigo "Transparent Resilience for Legacy HPC Applications on
  Cloud Spot Instances with MANA", submetido ao IEEE/ACM UCC 2026, avaliação pendente
- Contribuição ao projeto HPC@Cloud: integração completa do watcher de tolerância a falhas

---

## 9. Gates de Aceitação

| Gate | Descrição |
|---|---|
| G1 | LaTeX compila sem erros após cada seção escrita |
| G2 | Nenhuma ocorrência de "neste artigo" ou "this paper" |
| G3 | Todas as 6 figuras presentes e referenciadas |
| G4 | Seção Avaliação PIBIC escrita e completa |
| G5 | Revisado pelo professor antes da entrega |
