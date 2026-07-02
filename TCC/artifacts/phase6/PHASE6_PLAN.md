# Phase 6 Plan — TCC Document Writing

Date: 2026-06-14
Project: Integrating MANA Fault Tolerance into HPC@Cloud for AWS Spot Clusters
Goal: Write the complete TCC1 document in LaTeX using the lapesd-thesis template,
      from a working build environment to a submittable PDF.

---

## Status

| Step | Description | Status |
|---|---|---|
| Step 0 | Fix LaTeX build (Makefile, dependencies) | ✅ Done |
| Step 1 | Copy and organize assets (plots, CSVs, reports) | ✅ Done |
| Step 2 | Fix and complete BibTeX entries | ✅ Done |
| Step 3 | Hunt for missing papers and add to main.bib | ✅ Done |
| Step 4 | Update acronyms.tex | ✅ Done |
| Step 5 | Write Ch. 2 — Fundamentação Teórica | ✅ Done (§2.1–§2.5; SSM+EFS adicionados a §2.1.2) |
| Step 6 | Write Ch. 3 — Trabalhos Relacionados | ✅ Done (§3.1, §3.2, §3.3, §3.4, intro) |
| Step 7 | Write Ch. 4 — Análise de Instâncias Burstable | ✅ Done |
| Step 8 | Write Ch. 5 — Implementação | ✅ Done (§5.1–§5.5 escritos; §5.6 removido; audit completo 2026-06-22) |
| Step 9 | Write Ch. 6 — Avaliação Experimental | ✅ Concluído (§6.1–§6.7 escritos) |
| Step 10 | Write Ch. 7 — Conclusão | ⬜ TODO |
| Step 11 | Complete prolog.tex (agradecimentos, resumo estendido) | ⬜ TODO |
| Step 12 | Create/insert architecture and flow diagrams | ⬜ TODO |
| Step 13 | LaTeX tables from CSVs | ⬜ TODO |
| Step 14 | Final review, spell-check, compile clean | ⬜ TODO |

**Already done (before Phase 6 started):**
- [x] body.tex: Chapter 1 — Introdução (complete, with citations; **revisado em §5.5**)
- [x] prolog.tex: Resumo (PT) — complete
- [x] prolog.tex: Abstract (EN) — complete
- [x] prolog.tex: Resumo Estendido structure (skeleton with writing guidance)
- [x] main.tex: author, title, advisor, department, course, degree
- [x] main.bib: core entries (MANA, DMTCP, HPC@Cloud, NPB, cloud basics, FT)

**Done in current session:**
- [x] body.tex: Chapter 4 — Análise de Viabilidade de Instâncias Burstable (complete prose, §4.1–§4.4)
- [x] body.tex: §2.1.2 Amazon Web Services — new subsection added (writing guidance)
- [x] body.tex: §6.1.1 Ambiente + §6.1.2 Matriz de Experimentos — escritos (ver §6.5 Step 9 §6.1 para decisões)
- [x] body.tex: §6.2 Overhead do MANA sem Falhas — §6.2.1 e §6.2.2 escritos (ver §6.5 Step 9 §6.2 para decisões)
- [x] body.tex: §6.3 Estudos Sintéticos de Overhead — §6.3.1, §6.3.2 e §6.3.3 escritos (ver §6.5 Step 9 §6.3 para decisões)
- [x] analyze.py: suptitles removidos de todas as figuras; rcParams calibrados (font.size=14); ax.set_title() mantidos; symlink imgs/plots/ → data/plots/ criado
- [x] body.tex §6.4.1: forward reference corrigida ("Como se verificará na Seção~\ref{...}") + P2b restrito a "três configurações de cluster do CG-C"
- [x] body.tex §6.4.2: parágrafo DEGRADED com framing contraintuitivo; parágrafo REPLACE simplificado para anomalia honesta + tabela Gap%; §6.4.3 eliminado
- [x] body.tex §6.5.3: seção Razão de Overhead de Recuperação escrita (fig9 + tab:overhead-ratio + 3 parágrafos)
- [x] analyze.py: legendas movidas para rodapé (lower center); ax.text() internos → ax.set_title() em todas as figuras; títulos padronizados via rcParams (sem fontsize/fontweight explícito); fig2b e fig3 ganharam títulos; fig7 título interno → set_title()
- [x] body.tex: §6.1.1 Ambiente — updated to mention CNPq/AWS credits
- [x] prolog.tex: Agradecimentos — updated with CNPq/AWS credit acknowledgement template
- [x] Step 0 — Fix LaTeX Build (ver §1 abaixo para detalhes e ressalvas)
- [x] Step 1 — Copy and Organize Assets (ver §2 abaixo para detalhes e ressalvas)
- [x] Step 2 + Step 3 — BibTeX: reescrita completa + revisão de citações por seção (ver §3 abaixo)
- [x] Revisão Ch. 1 — Introdução: 3 correções aplicadas + seção Contribuições adicionada (ver §5.5)
- [x] Ch. 2 §2.1 — Computação em Nuvem: 4 subseções escritas + Tab. 2.1 (ver §6.1 §2.1)
- [x] Ch. 2 §2.2 — Computação de Alto Desempenho: 4 subseções escritas + Fig. 2.1 criada (ver §6.1 §2.2)
- [x] Ch. 2 §2.3 — Tolerância a Falhas em Sistemas Distribuídos: intro + 3 subseções
  (Técnicas de C/R, DMTCP, MANA) escritas; MANA subdividido em 3 `\subsubsection`;
  2 figuras autorais + 1 algoritmo (`algorithmicx`/`algpseudocode`) + 2 tabelas criados;
  revisão final de coerência feita (ver §6.1 §2.3 para decisões detalhadas)
- [x] Ch. 2 §2.4 — HPC@Cloud: 2 parágrafos; sem subseções; capacidades de alto nível +
  gap de tolerância a falhas que motiva o TCC; bridge para Cap. 5 (ver §6.1 §2.4)
- [x] Ch. 2 §2.5 — NAS Parallel Benchmarks: 1 parágrafo + tabela 3 colunas +
  1 parágrafo de justificativa de classes; EP-D, LU-C, CG-C; `tabular` p{9.5cm}
  (ver §6.1 §2.5 para decisões detalhadas)
- [x] Ch. 3 — parágrafo introdutório do capítulo escrito (3 frases, anuncia §3.4)
- [x] Ch. 3 §3.1 — Tolerância a Falhas em Aplicações MPI: escrita completa com
  parágrafo introdutório da seção + 4 parágrafos (ULFM, SCR, BLCR, Posner) +
  parágrafo de posicionamento; revisão crítica externa aplicada (ver §6.2 §3.1)
- [x] Ch. 3 §3.2 — HPC em Instâncias Spot e Custo: escrita completa com parágrafo
  introdutório + Gong + FarSpot + posicionamento; 2 ajustes de acessibilidade
  aplicados (ver §6.2 §3.2)
- [x] Ch. 3 §3.3 — Abordagens Proativas e Reativas: escrita completa com framing
  proativo/reativo + Ghavamipour (ANN + RA-HEFT) + limitações de escopo (MPI vs
  workflow) + posicionamento com trabalho futuro em duas etapas; revisão de
  consistência do Cap. 3 inteiro aplicada (ver §6.2 §3.3)
- [x] Ch. 3 §3.4 — Síntese Comparativa: reescrita completa com framing de lacuna
  na literatura + tabela 3 colunas (Foco + Lacuna) sem TCC + 2 parágrafos de
  fechamento ("dois vazios" + TCC na interseção); revisão completa do Cap. 3 com
  ajustes cirúrgicos em §3.1–§3.3 para criar ganchos narrativos (ver §6.2 §3.4)
- [x] Ch. 4 — Reescrito e finalizado: 3 seções com títulos que carregam a conclusão;
  tabela 4.1 adicionada; Fig. 4.1 (2x2 montage EP/LU) e Fig. 4.2 (2x1 microkernel)
  inseridas a `\textwidth` usando PDFs originais de `imgs/WCCimages/`; custo integrado
  junto ao desempenho em todo §4.2; justificativa dual do coordenador em §4.3
  (perfil de uso + disponibilidade on-demand); sem "--" nem "---" em todo o capítulo
  (ver §6.3 para todas as decisões de design)

---

## 1. Step 0 — Fix LaTeX Build ✅ Done

**Resultado:** compilação limpa com `latexmk -pdf -shell-escape main.tex`.
PDF final: 40 páginas, 216 KB, zero erros BibTeX.

### 1.1 Três correções aplicadas

#### Fix 1 — Dummy `imgs/alphachannel.pdf`

O `Makefile` tenta converter `imgs/alphachannel.svg` via `inkscape` (não instalado).
O arquivo faz parte do template demo e não é usado no documento.
Solução: criar arquivo vazio como placeholder.

```bash
touch TCC/artifacts/phase6/lapesd-thesis/imgs/alphachannel.pdf
```

**Ressalva:** se no futuro forem criados diagramas em SVG (Step 12), instalar inkscape
ou converter manualmente antes de compilar:
```bash
sudo apt-get install inkscape
```

#### Fix 2 — Reescrita completa de `main.bib`

O BibTeX reportava "I was expecting `{' or `('" sem linha exata útil.
Problemas encontrados e corrigidos:

| Problema | Causa | Correção |
|---|---|---|
| `% TODO:` dentro de entradas `@type{...}` | BibTeX NÃO trata `%` como comentário dentro de entradas | Todos os `% TODO:` movidos para fora das entradas como `%% TODO:` |
| `leitner2016burstable` com campo `booktabs` | Typo: `booktabs` em vez de `booktitle` | Corrigido; entrada verificada como Leitner & Scheuner UCC 2015 |
| `hpdc14` com `author = {others}` | BibTeX inválido | Trocado para `author = {{Authors TBD}}` |
| `garg2024mana` com campo `url` | Causava warning no abntex2 | Convertido para `note = {arXiv:2408.02218}` |
| UTF-8 em campos de texto | Pode causar erros dependendo do driver | Todos substituídos por escapes LaTeX (`{\'e}`, `{\~a}`, `{\c{c}}` etc.) |

#### Fix 3 — `%% HPC@Cloud` → `%% HPCatCloud` (bug crítico do BibTeX)

**Causa raiz do erro persistente:** o BibTeX escaneia o arquivo inteiro em busca de `@`,
*incluindo linhas de comentário* que começam com `%`. A linha:
```
%% HPC@Cloud
```
fazia o BibTeX interpretar `@Cloud` como declaração de tipo de entrada e disparar o erro
"I was expecting `{' or `('" imediatamente a seguir.

**Correção:**
```
%% HPCatCloud
```

**Ressalva importante para o futuro:** nunca usar `@` em linhas de comentário `%` no
`main.bib`, mesmo fora de entradas. O BibTeX ignora `%` como mecanismo de comentário
apenas para o texto entre entradas; o scanner de `@` opera de forma independente sobre
o arquivo inteiro.

### 1.2 Comando de compilação confirmado

```bash
cd TCC/artifacts/phase6/lapesd-thesis
latexmk -pdf -shell-escape main.tex
```

Para limpar auxiliares:
```bash
latexmk -C
```

Equivalente manual (quando `latexmk` não disponível):
```bash
pdflatex -shell-escape main.tex && bibtex main && pdflatex -shell-escape main.tex && pdflatex -shell-escape main.tex
```

### 1.3 Warnings esperados (não são erros)

- **`LaTeX Warning: Citation 'X' on page Y undefined`** para referências dentro de linhas
  `% ESCREVER:` — são comentários LaTeX e não geram citações reais; desaparecerão quando
  a prosa for escrita.
- **`Package pdftex.def Error: File 'alphachannel.pdf' not found`** — só aparece se o
  arquivo dummy foi deletado; recriar com `touch imgs/alphachannel.pdf`.

### 1.4 Entradas BibTeX com `%% TODO` pendentes

As seguintes entradas ainda têm dados incompletos (marcados no arquivo):

| Chave | Dado faltante | Onde verificar |
|---|---|---|
| `leitner2016burstable` | Confirmar autores e título exatos | `references/burstable-and-spot-instances/` |
| `aws2023efa` | Lista completa de autores | DOI 10.1007/s10586-023-04060-4 |
| `zhang2022farspot` | Confirmar autores | `references/cost-study/FarSpot-TPDS22.txt` |
| `bot2020` | Dados completos | `references/cost-study/BoT.txt` |
| `hpdc14` | Autores reais | `references/fault-tolerance/hpdc14.txt` |
| `wang2007partial` | Título e ano exatos | `references/fault-tolerance/wang2007.txt` |
| `gong2015cost` | Autores e venue | `references/fault-tolerance/gong2015.txt` |
| `ghavamipour2020` | Dados completos | `references/fault-tolerance/IST2020-3299-Ghavamipour.txt` |
| `amoon2018` | Título e dados | `references/fault-tolerance/amoon2018.txt` |

Essas pendências são tratadas em Step 2.

---

## 2. Step 1 — Copy and Organize Assets ✅ Done

**Resultado:** todos os assets do Phase 5 copiados para `lapesd-thesis/`. Estrutura
de diretórios criada. Plots do Phase 4 descartados (consolidados nos gráficos do Phase 5).

### 2.1 Estrutura criada em `lapesd-thesis/`

```
imgs/
  plots/       ← 10 PNGs do Phase 5 (figuras definitivas do Cap. 6)
  arch/        ← vazio; diagramas de arquitetura a criar no Step 12
data/
  tables/      ← 11 CSVs do Phase 5 (dados brutos para tabelas LaTeX no Step 13)
analysis_report_phase5.md       ← relatório EN (referência principal ao escrever §6)
analysis_report_phase5_ptbr.md  ← relatório PT-BR (idem)
report_tables.md                ← tabelas em markdown (referência rápida)
phase5_summary.md               ← resumo executivo do Phase 5
```

### 2.2 Figuras disponíveis em `imgs/plots/`

| Arquivo | Capítulo | Descrição |
|---|---|---|
| `fig1_mana_overhead.png` | §6.2 | Overhead MANA vs nativo (CG, EP, LU; 2/4/8 workers) |
| `fig2_synth_calls.png` | §6.2.2 | Microkernel MPI: frequência de chamadas |
| `fig2b_synth_imbalanced.png` | §6.2.2 | Microkernel com desbalanceamento (CG outlier) |
| `fig3_synth_ckpt.png` | §6.2.3 | Tamanho checkpoint vs tempo Phase 1 |
| `fig4_timing_phases.png` | §6.3 | Decomposição das fases P0–P3 por estratégia |
| `fig5_cg_short_job.png` | §6.3 | CG jobs curtos: regime desfavorável à FT |
| `fig6_mana_scalability.png` | §6.4 | Escalabilidade: overhead MANA vs N workers |
| `fig7_cost.png` | §6.5 | Custo financeiro por estratégia |
| `fig8_strategy_comparison.png` | §6.5 | REPLACE vs DEGRADED: comparação direta |
| `fig9_recovery_overhead_ratio.png` | §6.5 | Razão overhead/desconto spot por cenário |

### 2.3 Phase 4 Plots — DESCARTADOS (decisão)

Os gráficos de Phase 4 (burstable pilot) **não foram copiados**. O Cap. 4 será escrito
em prosa descritiva baseada nos resultados do ERAD/WCC, sem reutilizar os plots de Phase 4
porque o Phase 5 já consolida todas as comparações relevantes.

**Ressalva:** se durante a escrita do Cap. 4 (Step 7 — já feito em prosa) for decidido
incluir uma figura comparativa de instâncias burstable, os arquivos originais ainda estão em:
- `TCC/artifacts/phase4/analysis/plots/`

### 2.4 CSVs disponíveis em `data/tables/`

| Arquivo | Seção | Uso |
|---|---|---|
| `table01_mana_overhead.csv` | §6.2 | Tabela overhead MANA |
| `table02_strong_scaling.csv` | §6.4 | Escalabilidade wall time |
| `table03_synth_calls.csv` | §6.2.2 | Microkernel frequência de chamadas |
| `table04_synth_imbalanced.csv` | §6.2.2 | Microkernel desbalanceado |
| `table05_synth_ckpt.csv` | §6.2.3 | Checkpoint size vs Phase 1 |
| `table06_cg_ft_breakdown.csv` | §6.3 | CG breakdown por fase FT |
| `table07_ep_ft_wall_time.csv` | §6.3 | EP wall time com FT |
| `table08_lu_ft_wall_time.csv` | §6.3 | LU wall time com FT |
| `table09_strategy_winner.csv` | §6.3 / §6.5 | Vencedor REPLACE vs DEGRADED por cenário |
| `table10_recovery_overhead.csv` | §6.5 | Overhead de recuperação por estratégia |
| `table11_cost.csv` | §6.5 | Custo financeiro por run/estratégia |

Esses CSVs serão convertidos para tabelas LaTeX no Step 13.

**Ressalva:** os relatórios em `lapesd-thesis/*.md` são referência de escrita apenas;
não são compilados no PDF. Não adicioná-los ao `\input{}` do LaTeX.

### 2.5 Copy CSV Tables (for LaTeX table generation in Step 13)

```bash
cp -r TCC/artifacts/phase5/analysis/tables/  lapesd-thesis/data/
```

Key tables that will become LaTeX tables in Ch. 6:

| File | Chapter section | Purpose |
|---|---|---|
| `table01_mana_overhead.csv` | §6.2 | MANA overhead per benchmark |
| `table02_strong_scaling.csv` | §6.4 | Wall time vs workers |
| `table03_synth_calls.csv` | §6.2.2 | Synthetic MPI call frequency results |
| `table04_synth_imbalanced.csv` | §6.2.2 | Synthetic imbalanced results |
| `table05_synth_ckpt.csv` | §6.2.3 | Checkpoint size vs Phase 1 time |
| `table09_strategy_winner.csv` | §6.3 / §6.5 | REPLACE vs DEGRADED winner per scenario |
| `table11_cost.csv` | §6.5 | Cost per run per strategy |

---

## 3. Step 2 + Step 3 — Fix BibTeX Entries e Completar Referências ✅ Done

**Resultado:** `main.bib` completamente reescrito. Compilação limpa, zero erros BibTeX.
Citações em `body.tex` atualizadas para as novas chaves.

### 3.1 Ações realizadas

**Problema raiz de todos os erros anteriores:**
BibTeX interpreta qualquer `@` no arquivo como início de entrada, mesmo em linhas `%`.
Por isso o cabeçalho usa apenas `%%` e nunca usa `@` em comentários.

**Chaves renomeadas** (de DOI-style / auto-geradas → author-year legíveis):

| Chave antiga (user) | Chave nova | Entrada |
|---|---|---|
| `10.1007/s10586-023-04060-4` | `dancheva2023efa` | HPC on AWS (EFA) |
| `awsefs_whatisefs` | `aws2026efs` | AWS EFS docs |
| `aws_ec2_burstable_2026` | `aws2026burstable` | AWS burstable docs |
| `aws_spot_termination_notices` | `aws2026spot_termination` | Spot interruption notice |
| `aws_ec2_spot_pricing_2018` | `aws2018spot` | AWS spot pricing blog |
| `10.1145/3084448` | `wang2017burstable` | Wang et al. 2017, burstable |
| `10.1145/3447545.3451183` | `guidi2021cloud` | HPC/cloud gap closing |
| `10.1145/3150224` | `netto2018hpccloud` | HPC cloud survey (ACM) |
| `10.1145/3241737` | `buyya2018manifesto` | Cloud manifesto |
| `8355463` | `aljamal2018hpc` | HPC cloud providers comparison |
| `unknown` → `@misc` | `richter2016cloud` | Cloud for HPC (arXiv) |
| `article` (Saini) | `saini2021enablehpc` | Enable HPC in Cloud |
| `10.1098/rsta.2019.0061` | `shalf2020moores` | Moore's Law future |
| `teylo2020scheduling...` | `teylo2020bot` | Bag-of-Tasks BoT |
| `9648022` | `zhou2022farspot` | FarSpot |
| `inbook` | `ferretti2020cloudvsonprem` | Cloud vs on-premise HPC cost |
| `article` (Amoon) | `amoon2019checkpoint` | FT in cloud (checkpoint) |
| `PaulHHargrove_2006` | `hargrove2006blcr` | BLCR |
| `10.1177/1094342013488238` | `bland2013ulfm` | ULFM IJHPCA version |
| `7832806` | `gong2015cost` | Gong 2015, MPI cost EC2 |
| `10.1145/2600212.2600226` | `marathe2014hpdc` | Marathe 2014, HPDC redundancy |
| `9345896` | `ghavamipour2020reliability` | Reliability spot ANN |
| `5645453` | `moody2010scr` | SCR multi-level |
| `4228035` | `wang2007partial` | Wang 2007, partial restart |
| `https://doi.org/10.1002/cpe.7976` | `munhoz2024hpccloud` | HPC@Cloud CPE 2024 |
| `xu2024enabling...` | `xu2024mana` | MANA arXiv 2024 |
| `10.1145/3624062.3624255` | `xu2023mana` | MANA SC'23 workshop |
| `garg2019manampimpi...` | `garg2019mana` | MANA HPDC '19 (→ publicado) |
| `article` (Ansel) → `@inproceedings` | `ansel2009dmtcp` | DMTCP IPDPS 2009 |
| `inproceedings` (Filho) | `filho2022mpi` | MPI cloud benchmark ERAD |
| `9297048` | `hursey2020containers` | MPI containers |
| `MPISpec` | `mpi2009spec` | MPI Forum spec 2009 |
| `inproceedings` (Jette) | `jette2003slurm` | Slurm JSSPP 2003 |
| `10.1177/109434209100500306` | `bailey1991nas` | NAS Parallel Benchmarks |
| `ERAD-RS` | `soda2025erad` | Soda 2025, ERAD-RS |
| `11264716` | `soda2025wcc` | Soda 2025, WCC/SBAC-PADW |

**Entradas adicionadas** (estavam faltando):
- `arya2016split` — Arya et al. 2016, split-process design (CLUSTER '16)
- `munhoz2023thesis` — dissertação mestrado Munhoz 2023 (UFSC)

**Correções de conteúdo:**
- `garg2019mana`: era arXiv @misc → corrigido para @inproceedings HPDC '19 com DOI `10.1145/3307681.3325962`
- `ansel2009dmtcp`: era @article com ano 2007 → corrigido para @inproceedings IPDPS 2009
- `jette2003slurm`: era @inproceedings com type errado → corrigido para JSSPP 2003 com volume/série
- `marathe2014hpdc`: comentário em body.tex dizia "Oprescu & Kielmann" (ERRADO) → corrigido para "Marathe et al." (autores corretos)
- `wang2017burstable` (Wang et al. 2017 ACM SIGMETRICS): substitui a antiga `leitner2016burstable` que apontava para o paper errado

**Campos UTF-8 corrigidos:** todos os autores PT-BR com acentos usam escapes LaTeX (`{\'e}`, `{\~a}`, `{\c{c}}`).

**Regra de ouro:** NUNCA usar `@` em linhas de comentário `%` no `.bib`. O scanner BibTeX opera sobre o arquivo inteiro independentemente de `%`.

### 3.2 Citações atualizadas em `body.tex`

| Chave antiga | Chave nova |
|---|---|
| `aws2023efa` | `dancheva2023efa` |
| `aws2017spot` | `aws2018spot` |
| `leitner2016burstable` | `wang2017burstable` |
| `aws2023burstable` | `aws2026burstable` |
| `garg2023nersc` | `xu2023mana` |
| `garg2024mana` | `xu2024mana` |
| `zhang2022farspot` | `zhou2022farspot` |
| `bot2020` | `teylo2020bot` |
| `hpdc14` | `marathe2014hpdc` |
| `ghavamipour2020` | `ghavamipour2020reliability` |
| `amoon2018` | `amoon2019checkpoint` |
| `bailey1994nas` | `bailey1991nas` |

### 3.3 Revisão de citações por seção e melhorias adicionadas

| Seção | Melhoria aplicada |
|---|---|
| §2.1.3 Instâncias Spot | Adicionado `aws2026spot_termination` para o aviso de 2 min |
| §2.2 HPC intro | Adicionado `netto2018hpccloud` como survey de HPC em nuvem |
| §2.2.1 Clusters HPC | Adicionado `jette2003slurm` na menção ao Slurm |
| §2.2.2 MPI | Adicionado `mpi2009spec` em vez de referência genérica |
| §2.2.3 HPC na Nuvem | Adicionados `netto2018hpccloud,guidi2021cloud` junto a `dancheva2023efa` |
| §3.2 Spot e Custo | Adicionado `ferretti2020cloudvsonprem`; corrigido autor de `marathe2014hpdc` |

### 3.4 Entradas em main.bib não citadas ainda (disponíveis para uso)

Estas entradas estão no `main.bib` e podem ser citadas ao escrever os capítulos:

| Chave | Onde pode ser útil |
|---|---|
| `buyya2018manifesto` | §2.1 visão geral de cloud computing |
| `aljamal2018hpc` | §2.1.2 AWS ou §2.2.3 comparação de provedores HPC |
| `richter2016cloud` | §2.2.3 suitability of clouds for HPC |
| `saini2021enablehpc` | §2.2.3 estado da arte HPC em cloud |
| `shalf2020moores` | Introdução: motivação por fim de escala física |
| `guidi2021cloud` | §2.2.3 gap cloud vs HPC fechando |
| `netto2018hpccloud` | §2.2 e §2.2.3 survey HPC cloud |
| `ferretti2020cloudvsonprem` | §3.2 custo cloud vs on-premise |
| `teylo2020bot` | §3.2 BoT em spot/burstable |
| `munhoz2022hpc` | §2.4 HPC@Cloud versão anterior (SSCAD 2022) |
| `filho2022mpi` | §2.2.2 ou §2.4 benchmark MPI em cloud |
| `hursey2020containers` | §2.4 containers em HPC@Cloud |
| `aws2026efs` | §5.2 EFS como storage compartilhado |
| `buyya2011cloud` | §2.1 definição cloud computing (livro) |
| `xu2024mana` | §2.3.3 MANA desenvolvimento recente |

### 3.5 Papers separados pelo usuário que NÃO foram incluídos no main.bib

**Decisão:** 4 papers foram descartados por serem tangenciais demais ao escopo do TCC.
Eles existem como `.txt` em `references/` mas não têm entrada em `main.bib` e não serão citados.

| Arquivo em references/ | Motivo do descarte |
|---|---|
| `cloud-computing/Efficiency-or-Innovation...txt` (Li et al. 2021) | Foco em eficiência de negócios em cloud, não técnico |
| `cloud-computing/The_Evolution_of_the_Cloud.txt` (MIT thesis 2015) | Tese MIT muito genérica sobre evolução da cloud |
| `fault-tolerance/document.txt` (Hariyale 2012) | BLCR com load balancing — BLCR já coberto por `hargrove2006blcr` |
| `fault-tolerance/s2t4.txt` | Overlap com `bland2013ulfm`; não acrescenta escopo |

**Ressalva:** `csit64803.txt` (HPC em cloud, Cilardo 2015) também nunca foi adicionado.
Se ao escrever §2.2.3 ou §3.x surgir a necessidade de mais referências sobre HPC em cloud,
verificar este arquivo antes de buscar novos papers.

---

## 4. Verificação de Conformidade PDF — ABNT/UFSC ✅ Done

**Contexto:** após implementar a Lista de Siglas (Step 4 — ver §5 abaixo), foi realizada
uma verificação visual completa do PDF gerado comparando com as exigências do padrão TCC
da BU/UFSC (ABNT NBR 14724:2011 + RN 46/2019/CPG + RN 95/CPG). Três não-conformidades
foram encontradas e corrigidas; o restante está conforme.

**PDF resultante:** 38 páginas, 222 KB, zero erros LaTeX.

**Comando de build atual (com shell-escape para logo UFSC):**
```bash
latexmk -pdf -shell-escape main.tex
# ou equivalente direto:
pdflatex -shell-escape main.tex && makeglossaries main && pdflatex -shell-escape main.tex
```
O arquivo `.latexmkrc` já configura `-shell-escape` automaticamente para o latexmk.

---

### 4.1 Resultado por elemento pré-textual

| Elemento | PDF (antes da correção) | PDF (após correção) | Conformidade |
|---|---|---|---|
| Capa (p. 1) | Correto (sem logo, correto para TCC) | Inalterado | ✅ |
| Verso capa (p. 2) | Blank (reservado para ficha catalográfica) | Inalterado | ✅ |
| Folha de Rosto (p. 3) | ❌ aparecia na p. 5 | **Movida para p. 3** | ✅ |
| Folha de Aprovação (p. 5) | ❌ aparecia na p. 3 | **Movida para p. 5** | ✅ |
| Agradecimentos | Correto (p. 7, ímpar) | Inalterado | ✅ (TODO) |
| Resumo PT | Correto | Inalterado | ✅ |
| Resumo Estendido | Correto (esqueleto) | Inalterado | ✅ (TODO) |
| Abstract EN | Correto | Inalterado | ✅ |
| Lista de Figuras | ❌ lista vazia (proibida pelo ABNT) | **Comentada** | ✅ |
| Lista de Tabelas | ❌ lista vazia (proibida pelo ABNT) | **Comentada** | ✅ |
| Lista de Siglas | ❌ números de página errados ("15") | **Corrigida** | ✅ |
| Sumário | Correto | Inalterado | ✅ |
| Numeração pré-textual | Contadas mas não exibidas | Inalterado | ✅ |
| Numeração textual | Canto superior direito, a partir do Cap. 1 | Inalterado | ✅ |

### 4.2 Fix 1 — Ordem Folha de Rosto / Folha de Aprovação

**Sintoma:** PDF gerava Folha de Aprovação na p. 3 e Folha de Rosto na p. 5.
ABNT exige a ordem inversa (Rosto antes de Aprovação).

**Causa raiz — bug no mecanismo `\@ifstar` do abntex2:**

O abntex2 define `\imprimirfolhaderosto` com argumento opcional + `\@ifstar` no corpo:
```latex
\newcommand{\imprimirfolhaderosto}[1][\folhaderostoname]{%
   \@ifstar
     \imprimirfolhaderstststar
     \imprimirfolhaderostonostar
```

E `\imprimirfolhederstststar` toma UM argumento obrigatório. Por isso:
1. `\imprimirfolhaderosto*` consome o argumento opcional (default), depois `\@ifstar`
   detecta o `*` e chama `\imprimirfolhederstststar` tomando o **próximo token do input**
   como seu argumento — que é `\imprimirfolhadecertificacao`.
2. `\imprimirfolhadecertificacao` vira argumento de `\imprimirfolhederstststar`, e dentro
   de `folhaderosto*` esse argumento é passado como grupo para `\begin{folhaderosto*}{...}`,
   que o executa **antes** de `\folhaderostocontent`.
3. Resultado: `\imprimirfolhadecertificacao` executa dentro do ambiente `folhaderosto*`
   (produzindo Aprovação na p. 3), depois `\folhaderostocontent` chega na p. 5.

O design **intencional** do template é que `\protect\incluirfichacatalografica{ficha.pdf}`
fique nessa posição — a ficha catalográfica seria colocada na p. 2 (verso da folha de rosto)
antes do `\folhaderostocontent` na p. 3. Sem a ficha, o próximo token é
`\imprimirfolhadecertificacao`, causando o bug.

**Correção em `prolog.tex`:**
```latex
\imprimirfolhaderosto*
% \protect\incluirfichacatalografica{ficha.pdf}  % descomentar quando tiver a ficha
{}% placeholder: evita que \imprimirfolhadecertificacao seja engolido pelo \@ifstar
\imprimirfolhadecertificacao
```

O `{}` vazio é consumido como argumento seguro de `\imprimirfolhederstststar`, não produz
conteúdo visível, e deixa `\imprimirfolhadecertificacao` livre para executar normalmente.

**Importante — ao receber a ficha catalográfica da BU:**
1. Remover a linha `{}%placeholder`
2. Descomentar `\protect\incluirfichacatalografica{ficha.pdf}`
3. Colocar o PDF da ficha na pasta `lapesd-thesis/` com o nome `ficha.pdf`
4. Resultado: p. 2 = ficha catalográfica (verso), p. 3 = folha de rosto, p. 5 = aprovação

### 4.3 Fix 2 — Números de página na Lista de Siglas

**Sintoma:** cada entrada da lista aparecia com "15" à direita (número da página onde
`\glsaddall` foi chamado, já que nenhuma sigla é citada com `\gls{}` no texto ainda).

**Causa:** a opção `glossariespages` de `lapesd-thesis.cls` é `true` por default.

**Correção em `main.tex`:**
```latex
\documentclass[noglossariespages]{lapesd-thesis}
```

**Nota:** a BU/UFSC não exige (e geralmente não quer) números de página na lista de siglas.
A opção `noglossariespages` remove os números de todas as entradas.

### 4.4 Fix 3 — Lista de Figuras e Lista de Tabelas vazias

**Sintoma:** `\listoffigures*` e `\listoftables*` geravam páginas com apenas o título
e sem entradas (pois nenhum capítulo com figuras/tabelas foi escrito ainda).

**Por que é um problema:** ABNT NBR 14724:2011 proíbe listas de ilustrações/tabelas
quando não há figuras/tabelas no documento. A lista vazia também gera duas páginas em
branco desnecessárias.

**Correção em `prolog.tex`:**
```latex
%%% TODO: descomentar quando houver figuras no documento (ABNT proíbe lista vazia)
% \listoffigures*
%%% TODO: descomentar quando houver tabelas no documento (ABNT proíbe lista vazia)
% \listoftables*
```

**Quando descomentar:** ao inserir a primeira figura/tabela em qualquer capítulo.
O LaTeX popula as listas automaticamente a partir das entradas `\caption{}` nas
environments `figure` e `table`.

### 4.5 Nota sobre o logo da UFSC

A capa do TCC **não tem logo** por definição da BU. O logo UFSC aparece apenas em
teses e dissertações (`ufscthesistcc` toggle desativa o logo automaticamente).
O template gera o arquivo `main-logo.pdf` ao compilar com `-shell-escape`, mas
esse arquivo é usado apenas em modo dissertação/tese.

### 4.6 Atualizações no `.latexmkrc`

O arquivo `.latexmkrc` foi atualizado para incluir `-shell-escape` no comando
`pdflatex`, necessário para a extração do logo UFSC (usado em dissertações/teses
futuras, inofensivo para o TCC):

```perl
$pdflatex = 'pdflatex -shell-escape %O %S';
```

---

## 5. Step 4 — Update acronyms.tex ✅ Done

**Resultado:** `acronyms.tex` substituído. Total: 35 entradas, ordenadas alfabeticamente.
Compilação limpa confirmada: 40 páginas, 220 KB, zero erros.

### 5.1 Entradas presentes antes (19 entradas do template)

AMI, API, AWS, CFD, CPU, DMTCP, EC2, EFA, EFS, EIP, HPC, IaaS, MANA, MPI, MTTF, NPB,
SLA, SSM, VM.

### 5.2 Entradas adicionadas (16 novas)

| Sigla | Expansão |
|---|---|
| ANN | Artificial Neural Network |
| BLCR | Berkeley Lab Checkpoint/Restart |
| BU | Biblioteca Universitária |
| CG | Conjugate Gradient |
| EP | Embarrassingly Parallel |
| FT | Fault Tolerant |
| LU | Lower-Upper Symmetric Gauss-Seidel |
| MTTR | Mean Time To Recovery |
| NIST | National Institute of Standards and Technology |
| PaaS | Platform as a Service |
| PIBIC | Programa Institucional de Bolsas de Iniciação Científica |
| SaaS | Software as a Service |
| SCR | Scalable Checkpoint/Restart |
| TCC | Trabalho de Conclusão de Curso |
| ULFM | User-Level Failure Mitigation |

### 5.3 Modificação: expansão do MANA

Entrada anterior: `MPI-Agnostic, Network-Agnostic transparent checkpointing`
Corrigido para: `MPI-Agnostic, Network-Agnostic`

Razão: "transparent checkpointing" é descrição da ferramenta, não parte do acrônimo.
O acrônimo M-A-N-A expande apenas para "MPI-Agnostic, Network-Agnostic".

### 5.4 Regra para novos acrônimos durante a escrita

**Ao usar uma nova sigla no texto, adicionar em `acronyms.tex` na mesma hora,
mantendo a ordem alfabética.**

Script de verificação para encontrar siglas esquecidas (rodar de dentro de `lapesd-thesis/`):

```bash
comm -23 \
  <(grep -ohE '\b[A-Z]{2,}\b' body.tex | sort -u) \
  <(grep -oP '(?<=\\xnewacronym\{)[^}]+' acronyms.tex | sort -u)
```

O resultado lista candidatos a siglas no texto que ainda não têm entrada.
Filtre visualmente: `TODO`, `ESCREVER`, `YAML`, `REPLACE`, `DEGRADED` não são siglas.

### 5.5 Nota sobre a Lista de Siglas no PDF

O template `lapesd-thesis` imprime TODAS as entradas de `acronyms.tex` na lista,
independente de `\gls{}` ser usado ou não. O documento atual usa siglas diretamente
no texto (ex.: "HPC" sem `\gls{HPC}`). Isso é correto — a lista é um índice manual.

Se no futuro quiser que as siglas sejam expandidas automaticamente na primeira ocorrência
(ex.: "MPI (Message Passing Interface)"), seria necessário trocar os usos por `\gls{MPI}`.
Não é necessário para o padrão UFSC/ABNT; a lista de siglas no prolog é suficiente.

---

## 5.5 Revisão do Capítulo 1 — Introdução ✅ Done

**Data:** 2026-06-15. Revisão completa de estrutura, escrita, referências e extensão.

### Diagnóstico

| Aspecto | Avaliação |
|---|---|
| Estrutura (normas UFSC) | ✅ boa — mas faltava seção **Contribuições** (padrão em TCC-CC UFSC) |
| Qualidade de escrita | ✅ linguagem clara, fluente, adequada ao nível de TCC |
| Referências | ⚠️ dois gaps (ver abaixo) |
| Extensão | ⚠️ ~4 páginas — limite inferior; Contribuições eleva para ~5 p. |
| Fluxo narrativo | ✅ progressão lógica: cloud HPC → spot → C/R → MANA → topologia híbrida |

### Correções aplicadas em `body.tex`

**Fix 1 — Citação faltante (aviso de 2 min):**
A primeira menção ao aviso de dois minutos (§1.1) não tinha citação.
```latex
% antes:
prévio, quando o provedor precisar recuperar a capacidade.
% depois:
prévio~\cite{aws2026spot_termination}, quando o provedor precisar recuperar a capacidade.
```

**Fix 2 — "70% de economia" sem suporte:**
A afirmação "economias de até 70%" era inconsistente com "90% de desconto" citado logo acima
e não tinha referência. Substituído pela afirmação citada corretamente:
```latex
% antes:
viabilizando economias de até 70\% em relação ao preço sob demanda
% depois:
adquiridas ao preço \textit{spot} com desconto de até 90\%~\cite{aws2018spot}
em relação ao valor sob demanda
```

**Fix 3 — Seção Contribuições adicionada:**
Inserida entre §1.2.2 (Objetivos Específicos) e §1.3 (Organização do Trabalho).
Lista três contribuições em framing macro (tendências/framework, sem números pontuais):
1. Arquitetura de execução resiliente para MPI em spot (sem modificar aplicações)
2. Duas estratégias de recuperação com análise das condições de preferência
3. Avaliação de overhead e custo-benefício delimitando regimes econômicos viáveis

### O que NÃO foi alterado

- Parágrafos de abertura (antes de §1.1): estilístico, não é erro; pode ser expandido na
  revisão final se o orientador recomendar
- §1.3 Organização do Trabalho: está completo e correto com todas as referências de capítulo

---

## 6. Steps 5–11 — Writing Order and Section Guide

Write chapters in the order below. Each section includes: what to write, which
source documents to consult, and which figures/tables are used.

### 6.1 Step 5 — Chapter 2: Fundamentação Teórica

**Estimated length:** 15–20 pages.
**Source documents:** analysis reports in `lapesd-thesis/` (copied in Step 1),
papers in `TCC/artifacts/phase6/references/`.

#### §2.1 Computação em Nuvem ✅ Escrito e revisado (2026-06-15)

**Notas de implementação:**
- Abertura da seção (antes de §2.1.1): define cloud via NIST + mercado IaaS (Gartner 2021).
- `\listoftables*` descomentado em `prolog.tex` ao adicionar a primeira tabela (Tab. 2.1).
- Regiões AWS (us-east-1/us-west-2) removidas de §2.1.2 a pedido do autor; detalhe vai para §6.1.1.

**§2.1.1 Modelos de Serviço e Implantação** ✅
- 2 parágrafos: (1) IaaS/PaaS/SaaS com foco em IaaS + âncora para Tab. 2.1; (2) modelos de implantação.
- **Tabela 2.1** — comparação dos 3 modelos de serviço; fonte: `\citeonline{mell2011nist}`.
- Refs: `mell2011nist`.

**§2.1.2 Amazon Web Services** ✅
- 2 parágrafos: (1) liderança AWS, EC2, 3 modelos de precificação com refs para §2.1.3 e §2.1.4;
  (2) justificativa para uso da AWS (continuidade, literatura, créditos CNPq/AWS 64/2022).
- Refs: `gartner2022iaas`, `aws2018spot`, `soda2025erad`, `soda2025wcc`, `dancheva2023efa`, `gong2015cost`, `zhou2022farspot`.

**§2.1.3 Instâncias Spot** ✅
- 1 parágrafo: histórico de leilão → modelo simplificado 2017, aviso de 2 min, motivação para C/R periódico.
- Refs: `aws2018spot`, `aws2026spot_termination`.
- Ponte explícita para Capítulo~\ref{ch:implementacao}.

**§2.1.4 Instâncias Burstable** ✅
- 1 parágrafo: créditos/baseline/throttling; `wang2017burstable` para mecanismo token bucket;
  imprevisibilidade para HPC; uso restrito ao papel de coordenador MANA neste trabalho.
- Refs: `aws2026burstable`, `wang2017burstable`.
- Ponte para Capítulo~\ref{ch:burstable}.

#### §2.2 Computação de Alto Desempenho ✅

Estrutura final: **4 subseções**. Revisões aplicadas após revisão do usuário (iterações registradas abaixo).

**§2.2 intro** ✅
- Histórico: supercomputadores → clusters commodity → nuvem. Guia para 4 subseções.
- Refs: `netto2018hpccloud`.

**§2.2.1 Arquitetura de Clusters HPC** (`sec:hpc-clusters`) ✅
- Arquitetura típica: head, workers, NFS (explicado: "protocolo que exporta um diretório..."),
  InfiniBand (explicado: "tecnologia de alta velocidade desenvolvida para HPC"). Ponte neutra para
  `sec:hpc-nuvem` sem afirmar que nuvem usa só Ethernet.
- Referencia Fig. 2.1 (`fig:cluster-arch`) — **diagrama criado pelo usuário em `imgs/arch/cluster-arch.png`**.
- Refs: nenhuma (conteúdo técnico estabelecido).

**§2.2.2 Gerenciamento de Recursos: SLURM** (`sec:slurm`) ✅ (nova seção adicionada)
- P1: o que o SLURM faz (partições, sbatch/srun, srun como lançador MPI direto).
- P2: motivação técnica para uso neste trabalho — incompatibilidade mpirun/Hydra com MANA restart
  (SIGKILL após restauração; Hydra persiste estado PMI no checkpoint que fica inconsistente).
  `srun` funcionou em todos os cenários avaliados.
- Refs: `jette2003slurm`.

**§2.2.3 Programação Paralela com MPI** (`sec:mpi`) ✅
- P1: visão ampla — padrão desde 1993, portabilidade (OpenMPI/MPICH/Intel MPI), dominância em HPC.
  Desvantagem: programação explícita vs OpenMP.
- P2: modelo (memória privada, rank, comunicador), operações ponto-a-ponto e coletivas (sem entrar
  em detalhes de MPI_Wait/MANA — isso pertence à análise de testes sintéticos). MPICH 3.3.2.
- P3: **limitação MPI em ambientes não confiáveis** — falha de processo encerra todo o job
  (`bland2013ulfm`); ULFM como extensão que requer modificação de código; ponte para
  `sec:checkpoint-restart` (abordagem deste trabalho).
- Refs: `mpi2009spec`, `bland2013ulfm`.

**§2.2.4 HPC na Nuvem: Desafios e Oportunidades** (`sec:hpc-nuvem`) ✅
- P1: 3 desafios — (1) rede: Ethernet como opção mais comum, mas existem aceleradas (InfiniBand
  na Azure, EFA na AWS); (2) noisy neighbor; (3) confiabilidade geral de recursos (hardware,
  manutenção, spot como exemplo) — sem focar exclusivamente em spot.
- P2: oportunidades (elasticidade, pay-per-use). Convergência: `guidi2021cloud` (2021).
  EFA: "interface de baixa latência sem TCP convencional"; `dancheva2023efa` (CFD vs Cray XC40,
  2304 processos). Encerra positivamente.
- Refs: `netto2018hpccloud`, `guidi2021cloud`, `dancheva2023efa`.

#### §2.3 Tolerância a Falhas em Sistemas Distribuídos ✅

**§2.3 intro** (`sec:fault-tolerance`) ✅
- Cadeia causal fault→error→failure (`avizienis2004taxonomy`), vetores de falha em
  clusters HPC e em nuvem, MPI não contempla tolerância a falhas nativamente
  (`camargo2017ftmpi`), MTBF decrescente com a escala (`moody2010scr`).
- Papers novos trazidos pelo usuário nesta sessão para fundamentar este parágrafo:
  Avizienis et al. 2004 (taxonomia clássica de dependability) e Camargo & Duarte 2017
  (minicurso WSCAD em português sobre FT em MPI).

**§2.3.1 Técnicas de Checkpoint/Restart** (`sec:checkpoint-restart`) ✅
- Dois eixos ortogonais: (1) nível de abstração — aplicação vs. sistema, com
  Tabela 2.2 comparativa (`tab:ckpt-comparacao`); (2) esquema de coordenação —
  coordenado vs. não-coordenado, explicando o efeito dominó (`elnozahy2002survey`).
- Custo de I/O dominante (citado via `garg2019mana`, não mais via SCR) + BLCR como
  referência histórica e sua descontinuação em 2013.
- **Decisão**: cortado o aprofundamento no SCR (hierarquia de 3 níveis) que estava
  no plano original — não conectava com o resto do TCC (MANA/DMTCP não usam
  multi-level checkpointing) e parecia "jogado". SCR continua só como exemplo na
  tabela comparativa, sem elaboração.
- Refs: `posner2020syslevel` (novo — define nível de aplicação vs. sistema),
  `elnozahy2002survey` (novo — taxonomia de rollback-recovery, efeito dominó),
  `garg2019mana`, `hargrove2006blcr`.

**§2.3.2 DMTCP: Checkpoint Transparente para Computações em Cluster** (`sec:dmtcp`) ✅
- Abre justificando por que está sendo explicado antes do MANA: "o MANA é uma
  extensão do DMTCP, reaproveitando sua infraestrutura".
- `LD_PRELOAD` explicado em termos simples; protocolo de checkpoint como **lista
  numerada de 4 etapas** (suspensão, drenagem, gravação, reabastecimento/retomada)
  em vez de prosa corrida — decisão tomada após feedback de que o parágrafo único
  estava "pesado" de ler.
- **Decisão importante sobre a motivação real do MANA**: a primeira versão deste
  parágrafo justificava o MANA citando portabilidade de rede (InfiniBand, RDMA,
  plugins por interconnect) — mas isso não faz sentido para este trabalho, que usa
  TCP/IP convencional na AWS (não InfiniBand). A motivação real e específica deste
  TCC é outra: o DMTCP sempre restaura a **mesma topologia de processos** (mesmo nº
  de ranks) que existia no checkpoint, porque ele não reinicializa o MPI — só
  restaura memória salva. Isso é incompatível com a estratégia \textsc{Degraded}
  (reinício com um processo a menos), que é o que este trabalho efetivamente
  explora. Essa é a ponte real para a Seção 2.3.3.
- Refs: `ansel2009dmtcp`, `garg2019mana`.

**§2.3.3 MANA: MPI-Agnostic Network-Agnostic Checkpoint/Restart** (`sec:mana`) ✅
- **Decisão estrutural**: subdividida em 3 `\subsubsection` (Arquitetura Split-Process,
  Consistência do Checkpoint, Maturidade e Adoção em Produção) após feedback de que a
  seção única estava ficando extensa e cansativa de ler.
- Intro (antes das subsubseções): a propriedade central do MANA para este trabalho é
  a **reconstrução** (não restauração) da sessão MPI a cada reinício — isso é o que
  habilita topologia de processos diferente e independência de implementação
  MPI/rede. Texto enxugado para não repetir o que já foi dito no fim de §2.3.2.

  - **Arquitetura Split-Process** (`sec:mana-splitprocess`):
    - Upper/lower half explicados com cuidado extra: ficou claro que é **um único
      processo do SO** (não dois processos separados) — por isso uma chamada da
      metade superior para a inferior é uma chamada de função comum, sem custo de
      IPC. Esse ponto gerou confusão em rascunhos anteriores e foi reescrito após
      feedback direto.
    - Conteúdo de cada metade (superior: app + wrappers, salva; inferior:
      biblioteca MPI real + drivers de rede, descartada) em parágrafo próprio.
    - **Figura 2.2** (`fig:mana-splitprocess`, `imgs/arch/mana-splitprocess.png`) —
      diagrama autoral (criado pelo usuário em ferramenta externa, não TikZ).
      Mostra upper half (app + wrappers, com a tabela de tradução de IDs virtuais
      embutida visualmente dentro da caixa de wrappers) e lower half (biblioteca
      MPI real + drivers de rede).
    - Propriedade de topologia/IP isolada em parágrafo curto e focado (decisão:
      dar mais destaque visual/estrutural a essa propriedade, que é central ao
      TCC e estava se perdendo em meio à explicação mecânica).
    - Parágrafo de tabela de IDs virtuais com transição explícita ("resolve X,
      cria um desafio distinto: Y") para não parecer repetição do parágrafo
      anterior — e referência de volta à Figura 2.2, já que a tabela aparece
      visualmente nela.
  - **Consistência do Checkpoint** (`sec:mana-consistencia`):
    - Ponto a ponto: exemplo concreto com `MPI_Send`/`MPI_Recv` (processo A envia,
      checkpoint cai antes de B receber → mensagem perdida) + explicação do
      algoritmo de Chandy e Lamport (citação nova: `chandy1985snapshots`).
    - Coletivas: explica o **porquê** antes do como — estado parcial de uma
      coletiva vive na lower half (descartada), por isso a drenagem ponto-a-ponto
      não se generaliza; invariante "nenhum checkpoint durante uma coletiva" +
      exceção da barreira trivial; protocolo de duas fases (intenção → pronto →
      executar).
    - **Figura 2.3** (`fig:mana-collective-protocol`,
      `imgs/arch/mana-checkpoint-sequence.png`) — diagrama de sequência autoral
      (Coordenador + Rank A + Rank B), mostrando Rank A terminando uma coletiva em
      andamento e Rank B esperando antes de entrar numa nova.
    - **Algoritmo 1** (`alg:mana-checkpoint-decision`) — pseudocódigo curto (6
      linhas, `algorithmicx`/`algpseudocode`, já configurado no `.cls`) que unifica
      a decisão de checkpoint seguro por rank (drenar → checar coletiva → responder
      pronto → aguardar comando → salvar). Adicionado a pedido do usuário como
      resumo visual complementar à prosa e à Figura 2.3. Necessário adicionar
      `\listofalgorithms*` ao `prolog.tex` (feito).
    - Overhead: framing cuidadoso — não afirma que o overhead observado neste
      trabalho "contradiz" a literatura (seria impreciso); apenas aponta que há um
      componente fixo, aparentemente independente do volume de chamadas MPI, cuja
      origem é investigada nos testes sintéticos do Capítulo 6.
  - **Maturidade e Adoção em Produção** (`sec:mana-producao`):
    - Perlmutter/NERSC 2023 (único pacote de C/R transparente em produção em
      supercomputador de grande escala com MPI de uso intensivo) + versão usada
      neste trabalho (MPICH 3.3.2).
    - **Decisão**: removida a tabela de comandos operacionais (`mana_coordinator`,
      `mana_launch`, `mana_status`) que estava aqui — além de operacional demais
      para a fundamentação teórica (pertence ao Cap. 5 Implementação), a descrição
      de `mana_launch` como "substituto do `srun`" estava **factualmente errada**:
      o `srun` continua sendo o lançador; `mana_launch` é o binário que cada
      *rank* executa, recebendo a aplicação real como argumento (confirmado pelo
      usuário com o script real de orquestração usado no cluster). Removida também
      a frase sobre "sistema de arquivos compartilhado" do MANA, que sugeria
      incorretamente que isso é uma propriedade do MANA — na realidade é uma
      decisão arquitetural deste trabalho (uso do EFS), motivada pelo modelo de
      falha considerar a perda da instância inteira, não só da aplicação. Ambos os
      detalhes (comandos reais de orquestração + por que EFS) devem ser
      retomados no Cap. 5.
- Refs: `garg2019mana`, `arya2016split`, `xu2023mana`, `xu2024mana`,
  `chandy1985snapshots` (nova).

**Diagramas autorais criados pelo usuário (não TikZ) nesta sessão:**
- `imgs/arch/mana-splitprocess.png`
- `imgs/arch/mana-checkpoint-sequence.png`
- (Tentativas de TikZ para ambos foram feitas primeiro e descartadas — diagramas
  complexos com múltiplos atores/caixas de ativação são mais fáceis de ajustar em
  ferramenta visual do que depurando coordenadas TikZ manualmente.)

**Novas entradas em `main.bib` nesta sessão:**
- `avizienis2004taxonomy`, `camargo2017ftmpi`, `posner2020syslevel` — renomeadas de
  chaves genéricas (`1335465`, `inbook`, `inproceedings`) que o usuário colou prontas
  no arquivo.
- `elnozahy2002survey` — renomeada de `10.1145/568522.568525` (DOI como chave).
- `chandy1985snapshots` — nova, adicionada para citar o algoritmo clássico de
  snapshots distribuídos (Chandy & Lamport, 1985), referenciado nos próprios papers
  do MANA/DMTCP.

#### §2.4 HPC@Cloud ✅

**Decisões de estrutura:**
- Subseções §2.4.1 e §2.4.2 **removidas**. A seção ficou como `\section` plana sem subseções.
- §2.4.2 ("Mecanismos de Tolerância a Falhas Pré-existentes") eliminado porque os mecanismos
  descritos eram trabalho do próprio autor (anterior ao TCC), não uma funcionalidade do HPC@Cloud.
  Incluí-la causaria atribuição errada de contribuição.
- Nível de detalhe: apenas capacidades de alto nível. As referências
  (`munhoz2024hpccloud`, `munhoz2023thesis`) documentam uma versão mais antiga em Python;
  a versão atual em Rust não tem publicação. Não citar detalhes de implementação.

**Estrutura final (2 parágrafos):**
- P1: o que é o HPC@Cloud (código aberto, independente de provedor, YAML declarativo,
  gerencia ciclo de vida completo do cluster).
- P2: acesso a instâncias spot via API → redução de custo; mas a ferramenta não oferecia,
  originalmente, suporte a tolerância a falhas transparente para spot (gap que motiva o TCC);
  bridge para Capítulo~\ref{ch:implementacao} na última frase.

**Refs:** `munhoz2024hpccloud`, `munhoz2023thesis`.

#### §2.5 NAS Parallel Benchmarks ✅

**Decisões de estrutura:**
- NPB 3.4.4; três benchmarks: EP (Classe D), LU (Classe C), CG (Classe C).
- EP-D e não EP-C: Classe C resultava em tempo muito curto em m5.xlarge (medição imprecisa).
- Ordem no texto e na tabela: EP → LU → CG (mínima → regular → irregular; escalonamento de carga).
- Nível de detalhe: superficial (NPB é bem conhecido). O texto justifica a escolha dos três
  em função dos perfis de comunicação MPI complementares; a tabela sintetiza os padrões.
- Tabela: `tabular` com colunas `l l p{9.5cm}` — 3 colunas (Benchmark, Classe, Padrão de
  comunicação). `tabularx` foi adicionado ao `main.tex` mas NÃO é usado aqui porque o
  ajuste automático de coluna X quebrava o texto em muitas linhas curtas.
- `\usepackage{tabularx}` permanece em `main.tex` para uso eventual em outras tabelas.

**Estrutura final:**
- P1: introdução ao NPB, classes de tamanho, versão 3.4.4, justificativa dos 3 benchmarks
  em termos de espectro de comunicação MPI.
- Tabela~\ref{tab:npb-benchmarks}: 3 linhas (EP/LU/CG) × 3 colunas.
- P2: justificativa das classes (EP-D vs EP-C; CG-C e LU-C por tempo adequado).

**Refs:** `bailey1991nas`.

---

### 6.2 Step 6 — Chapter 3: Trabalhos Relacionados

**Estimated length:** 4–5 páginas (dimensionado para TCC, não dissertação).
**Structure:** 4 sections. Each subsection ends with 1–2 positioning sentences;
§3.4 has a synthesis table + brief concluding paragraph.

#### Decisões de estrutura e seleção de referências (2026-06-17)

**Papers mantidos (7 no total):**

| Seção | Chave BibTeX | Papel no argumento |
|---|---|---|
| §3.1 | `bland2013ulfm` | ULFM: requer redesign da aplicação; contraste direto com transparência do MANA |
| §3.1 | `hargrove2006blcr` | BLCR: predecessor sistêmico do MANA, acoplado à rede; contexto histórico |
| §3.1 | `moody2010scr` | SCR: C/R multi-nível mas requer instrumentação da aplicação; contraste direto |
| §3.1 | `posner2020syslevel` | Compara system-level vs application-level C/R; argumento teórico para escolha do MANA |
| §3.2 | `gong2015cost` | Trabalho anterior mais próximo: MPI + FT + EC2 com checkpoint e replicação |
| §3.2 | `zhou2022farspot` | FarSpot: otimização de instância/preço sem C/R transparente; contraste limpo |
| §3.3 | `ghavamipour2020reliability` | ANN para predição de revogação; abordagem proativa vs reativa deste TCC |

**Papers removidos e motivo:**

| Chave BibTeX | Motivo da remoção |
|---|---|
| `s2t4` | Companion ULFM (EuroMPI'12, mesmos autores de `bland2013ulfm`); redundante |
| `wang2007partial` | Analogia com DEGRADED_RESUME é forçada; wang2007 é live migration cooperativa |
| `marathe2014hpdc` | Modelo de bidding eliminado pela AWS em 2017; sem contraste direto com contribuição |
| `teylo2020bot` | BoT vs MPI é trivialmente diferente; não acrescenta posicionamento real |
| `amoon2019checkpoint` | Survey genérico sem contraste específico com o TCC |
| `ferretti2020cloudvsonprem` | Cloud vs on-premise; completamente tangencial à contribuição |

#### §3.1 Tolerância a Falhas em Aplicações MPI

Papers: `bland2013ulfm`, `hargrove2006blcr`, `moody2010scr`, `posner2020syslevel`.

- ULFM (`bland2013ulfm`): extensões ao padrão MPI para detecção de falhas — requer que
  a aplicação trate ranks falhos explicitamente; contraste com transparência do MANA.
- BLCR (`hargrove2006blcr`): C/R sistêmico para Linux; predecessor do DMTCP/MANA;
  acoplado à rede (falha se IP muda ao reiniciar); não mais mantido.
- SCR (`moody2010scr`): checkpoint multi-nível (local → burst buffer → PFS) para
  amortizar I/O; requer instrumentação da aplicação; contraste com transparência do MANA.
- `posner2020syslevel`: compara diretamente C/R a nível de sistema vs a nível de
  aplicação; fornece o argumento teórico central para a escolha do MANA.
- Posicionamento: este TCC é o único que combina transparência sistêmica (sem mudança
  de código) + independência de rede + orquestração integrada em nuvem.

#### §3.2 HPC em Instâncias Spot e Custo

Papers: `gong2015cost`, `zhou2022farspot`.

- `gong2015cost`: otimização de custo para MPI com deadline em EC2, usando checkpoint
  e replicação combinados; trabalho mais próximo do TCC, mas sem C/R network-agnostic
  nem integração com orquestrador.
- `zhou2022farspot` (FarSpot): otimização que seleciona tipo de instância e prevê
  preços; foca em seleção de recurso, não em C/R transparente.
- Posicionamento: este TCC adiciona recuperação MPI transparente com duas estratégias
  configuráveis (Replace e Degraded) avaliadas empiricamente em spot real.

#### §3.2 — Decisões de escrita (2026-06-17)

**Estrutura adotada:** parágrafo introdutório temático (custo como objetivo central)
→ Gong (modelo antigo, BLCR, checkpoint + replicação) → FarSpot (novo modelo spot,
ML, migração) → posicionamento (2 frases: TCC cobre a dimensão de recuperação após
interrupção inevitável).

**Ordem dos papers:** Gong primeiro (2015, modelo de lances, BLCR — antecede o novo
modelo spot) → FarSpot (2022, novo modelo, ML) — ordem cronológica que também é
narrativa (problema evoluiu com a mudança do mercado spot em 2017).

**Argumento principal do posicionamento:** os dois papers focam em evitar ou minimizar
o custo da interrupção via seleção de recursos; este TCC foca em recuperar de forma
transparente quando a interrupção já ocorreu. Ângulo complementar, não concorrente.

**Sobre "sem C/R network-agnostic" e "sem integração com orquestrador":** esses
argumentos do plano original NÃO foram incluídos em §3.2. Decisão: são propriedades
arquiteturais, não argumentos de custo, e não pertencem a uma seção cujo título é
"HPC em Instâncias Spot e Custo". O lugar correto é a §3.4 Síntese Comparativa, onde
a tabela cobre todas as dimensões simultaneamente.

**Sem tabela ou diagrama:** dimensão de custo é capturada textualmente; tabela
comparativa reservada para §3.4 onde cobre todos os trabalhos das três seções juntos.

**Ajustes de acessibilidade aplicados após revisão:**
- "restrição de \textit{deadline}" → reestruturada para "prazo máximo estipulado
  pelo usuário (\textit{deadline})" — leitor não precisa conhecer o termo técnico
  de otimização para entender o argumento.
- ML concepts do FarSpot (Random Forest, LightGBM, SVM) → removidos e substituídos
  por "\textit{ensemble learning}, técnica que combina múltiplos modelos de aprendizado
  de máquina para obter previsões mais precisas do que qualquer modelo isolado" —
  preserva o nível técnico sem exigir background em ML.
- FTGLB (§3.1): **não ajustado** — decisão do autor; contexto de sistema-level vs
  application-level C/R é suficiente para entender o argumento sem explicar o FTGLB.

#### §3.3 Abordagens Proativas e Reativas à Interrupção de Instâncias

Paper: `ghavamipour2020reliability`.

- `ghavamipour2020reliability`: usa ANN para prever revogações de instâncias spot;
  abordagem proativa (antecipar e evitar interrupção).
- Posicionamento: este TCC adota abordagem reativa (detectar sinal e recuperar via
  checkpoint); predição proativa é ortogonal e apontada como trabalho futuro.
- Nota: seção intencionalmente enxuta — um paper basta para estabelecer o contraste
  proativo/reativo; não adicionar papers só para engrossar.

#### §3.1 — Decisões de escrita (2026-06-17)

**Estrutura adotada:** parágrafo classificatório (duas famílias: nível de aplicação
vs nível de sistema) → ULFM → SCR → BLCR → Posner → posicionamento (2 frases).

**Ordem dos papers:** aplicacionais primeiro (ULFM, SCR — mostram o custo de
instrumentação), sistêmico depois (BLCR — mostra a limitação de rede), comparação
por último (Posner — fundamenta a escolha do MANA). Ordem lógica narrativa, não cronológica.

**Sem tabela em §3.1:** tabela comparativa foi mantida apenas em §3.4, onde cobre
todos os trabalhos das três seções juntos. Tabela parcial aqui fragmentaria a
comparação e tornaria §3.2 e §3.3 assimétricas (2 e 1 paper, respectivamente).

**Ajustes linguísticos após revisão crítica externa:**
- "transparente" → "amplamente transparente à aplicação, sem exigir modificações
  no código-fonte" (transparência nunca é absoluta)
- ULFM: "substancialmente reestruturados" → "alterações não triviais... barreira
  prática de adoção" (mais defensável; o paper não afirma "reestruturação")
- SCR hierarquia: "RAM, SSD, PFS" → "dispositivos locais ao nó... com o sistema de
  arquivos paralelo" (mais geral e correto; SCR não tem hierarquia fixa)
- SCR "1.000 vezes" → "ordens de grandeza" (número é sistema-específico do LLNL)
- SCR 85%: **MANTIDO** com "em sua avaliação" — está diretamente na Tabela II
  do Moody 2010 (LOCAL 31% + PARTNER/XOR 54% = 85%); verificado no paper
- SCR cloud storage: corrigido de "não oferecem armazenamento local" (factualmente
  errado — EC2 tem instance store) para "armazenamento efêmero é perdido no momento
  da revogação" (correto e é o ponto relevante para spot instances)
- BLCR "sem garantia de portabilidade" → "reduzindo a portabilidade" (mais suave)
- Posner: adicionado "nos cenários avaliados" (benchmark sintético Java, não MPI)
- Posner: "estado definido pelo usuário" → "estado necessário à sua estratégia de
  recuperação" (mais preciso para FTGLB)
- Posicionamento: "nenhuma delas oferece" → "entre as referências analisadas, não
  foi identificada uma solução que reúna" (defensável sem exigir prova de universalidade)

#### §3.3 — Decisões de escrita (2026-06-17)

**Estrutura adotada:** parágrafo classificatório (proativo vs reativo) → contexto
de workflow scheduling + definição de workflow + descrição do HEFT → contribuição do
paper (preditor ANN + RA-HEFT como modificação do HEFT) → limitações de escopo
(revogação inevitável + incompatibilidade MPI/workflow) → posicionamento com trabalho
futuro em duas etapas.

**Framing proativo/reativo:** aberto com a definição dos dois paradigmas (sem citar
o paper ainda), para que o leitor chegue à apresentação de Ghavamipour já com o mapa
conceitual. Decisão análoga ao framing aplicação/sistema em §3.1.

**HEFT explicado antes do RA-HEFT:** a versão inicial não explicava o HEFT, tornando
ambíguo o que o RA-HEFT modifica. Reestruturado para: (1) HEFT = EFT mínimo, não
considera confiabilidade; (2) RA-HEFT = mesma lógica + filtro de confiabilidade via
ANN antes de cada alocação.

**Workflow definido inline:** "conjunto de tarefas com dependências entre si, formando
um grafo em que cada tarefa pode ser executada de forma independente assim que suas
predecessoras concluam" — evita jargão (DAG) sem sacrificar precisão.

**Pegasus:** definido como "\textit{framework} amplamente utilizado para execução de
fluxos de trabalho científicos" — suficiente para contextualizar a avaliação sem
desviar o foco.

**Explicação MPI vs workflow granularity:** o argumento central ("não existe
granularidade de tarefa individual que possa ser reexecutada de forma isolada")
exigiu explicitação do mecanismo: se um nó é revogado, os processos restantes ficam
bloqueados aguardando mensagens que nunca chegarão — tornando necessário reiniciar
a execução inteira. Versão anterior dizia que não se encaixavam sem explicar por quê.

**Trabalho futuro em duas etapas:**
1. Checkpoints periódicos automáticos (extensão direta, independe de predição)
2. Predição proativa de revogações para antecipar disparo de checkpoints
Ordem: do mais imediato/simples ao mais sofisticado/dependente de passo anterior.
Versão anterior ia direto para predição, omitindo o passo intermediário mais natural.

**Sem tabela ou diagrama:** a seção é intencionalmente enxuta (um paper). O framing
proativo/reativo é conceitual e capturável em texto. Qualquer tabela aqui seria
redundante com §3.4.

**Revisão geral do Cap. 3 (2026-06-17):** revisão completa pós-escrita de §3.3.
Quatro correções aplicadas (todas em §3.3, artefatos de edição no IDE):
- Duplicação "Um \textit{workflow} Um workflow" → corrigido para "Um \textit{workflow} é..."
- Falta de quebra de parágrafo entre HEFT e preditor → linha em branco adicionada
- "no cenário experimental" redundante após "no conjunto de treinamento" → removido
- "reduziu significativamente os erros" → revertido para "eliminou todos os erros"
  (factualmente correto: Table 2 do paper mostra 0 erros em todos os cenários RA-HEFT)

#### §3.4 — Síntese Comparativa (reescrita 2026-06-17)

**Abordagem adotada — "lacuna na literatura" (gap framing):**
O plano original propunha tabela ✓/✗ por dimensões (Transparência, OC, ME, CR, Custo).
Após múltiplas revisões e feedback externo, essa abordagem foi descartada. Motivos:
1. Dimensões como OC (orquestrador de nuvem) e ME (estratégias múltiplas) não foram
   discutidas nos trabalhos relacionados — comparar nelas seria injusto.
2. Linhas com muitos ✗ geram conotação negativa sem revelar contribuição própria.
3. O framing proativo/reativo de §3.3 não aparecia na tabela original.
4. "Avaliação em spot real" ficou imprecisa: falhas foram injetadas (não orgânicas da AWS);
   FarSpot usa traces históricos reais; os termos precisavam de distinção mais cuidadosa.

**Solução final — tabela de 3 colunas + narrativa de dois vazios:**
```
| Trabalho | Foco principal | Lacuna para MPI em instâncias spot |
```
- Coluna "Lacuna" avalia cada trabalho PELO CRITÉRIO DO TCC (recuperação transparente
  de MPI acoplado em spot), não por objetivos fora do escopo de cada trabalho.
- TCC não aparece na tabela; sua posição é descrita nos dois parágrafos subsequentes.
- Widths: `{l p{3.6cm} p{5.8cm}}` com `\small` para caber na página.

**Narrativa dos dois parágrafos pós-tabela:**
1. "Dois vazios complementares": ferramentas sistêmicas (BLCR, DMTCP) garantem
   transparência mas são IP-dependentes; trabalhos de custo/escalonamento (SOMPI,
   FarSpot, RA-HEFT) tratam MPI como caixa-preta.
2. TCC na interseção: MANA reconstrói sessão MPI (resolve IP), HPC@Cloud automatiza
   ciclo de detecção + checkpoint + reinício. Bridge para Cap. 5.

**Ganchos narrativos adicionados nas seções anteriores (revisão cirúrgica):**
- §3.1 posicionamento (editado pelo usuário no IDE): reframeado de positivo ("TCC faz X")
  para lacuna ("BLCR e DMTCP vinculam reinício ao IP; violado a cada interrupção spot").
- §3.2 SOMPI: BLCR introduzido ao descrever o mecanismo (não só na limitação); modelo de
  custo explicado com intuição dos extremos (retrabalho domina vs computação duplicada domina).
- §3.2 FarSpot: cadeia causal explicitada (predição → migração proativa → evita custo alto
  e risco de revogação); "traces" = logs históricos de preços EC2, não execuções reais;
  "estado da arte" = outros algoritmos de migração/predição spot.
- §3.2 posicionamento: "Replace" e "Degraded" removidos (só nos objetivos, Cap. 5 explica);
  adicionado framing "SOMPI delega ao BLCR; FarSpot trata MPI como caixa-preta".
- §3.3 RA-HEFT: separado em dois parágrafos — (1) RA-HEFT estuda workflows, não MPI;
  (2) MPI tightly-coupled segue modelo fundamentalmente diferente de workflow; análise
  de workflows explicitamente fora do escopo do TCC.
- §3.3 posicionamento: mantido trabalho futuro (checkpoints periódicos + predição);
  acrescentado: "predição não dispensa mecanismo reativo para MPI acoplado".

**Regra de estilo estabelecida nesta sessão:**
Não usar travessão tipográfico (—) em nenhum parágrafo do TCC. Substituir por
vírgulas, parênteses ou ponto-e-vírgula conforme o contexto.

---

### 6.3 Step 7 — Chapter 4: Análise de Instâncias Burstable ✅ Concluído (2026-06-17)

**Estimated length:** ~3 páginas (original: 5+ páginas — reduzido intencionalmente).
**Source:** `references/scientific-initiation/` (ERAD-RS 2025 e SBAC-PADW 2025).

#### Decisões de reestruturação (2026-06-17)

**Problema diagnosticado:** versão anterior com 4 seções era cansativa porque:
1. §4.2 (Desempenho) e §4.3 (Custo) repetiam a mesma história: se o LU é 3x mais
   lento na T3, o custo total obviamente também é pior. Separar em duas seções não
   adicionava informação nova.
2. §4.1 (Metodologia) descrevia as 3 fases da avaliação com nível de detalhe de
   artigo científico, não de capítulo de contexto num TCC.
3. A conclusão mais importante (perfil do coordenador → topologia híbrida) ficava
   enterrada na §4.4 depois de quatro páginas de resultados redundantes.

**Solução adotada: 3 seções, ~3 páginas**

| Seção nova | Conteúdo | Substituiu |
|---|---|---|
| §4.1 Configuração Experimental | Instâncias (Tabela 4.1), HPC@Cloud, CloudWatch, benchmarks em 2 frases | §4.1 antigo (3-fase verboso) |
| §4.2 Por que Instâncias Burstable Penalizam Workers HPC | EP competitivo; LU penalizado em desempenho E custo juntos; escalabilidade amplifica; microkernel isola causa de rede | §4.2 + §4.3 antigos (redundantes) |
| §4.3 Por que o Coordenador é um Caso à Parte | Perfil do coordenador (CPU intermitente, rede baixa) = ponto forte do burstable; t3.large on-demand; ponte para topologia de Cap. 5 | §4.4 antigo (renomeado e antecipado) |

**Decisão de nomenclatura das seções:** os títulos carregam explicitamente a conclusão,
não apenas o tema ("Por que X" em vez de "Resultados" ou "Discussão"). O argumento fica
visível no sumário e o leitor não precisa ler o corpo inteiro para saber o veredito.

**Tabela adicionada:** Tab. 4.1 — specs das instâncias (t3.2xlarge vs m5.2xlarge:
vCPUs, rede, custo/hora, custo crédito excedente). `booktabs` + `\fonte{soda2025wcc}`.
Campo "Custo de crédito excedente" usa `N/A` para m5.2xlarge (não usa "--" nem "---",
que são proibidos em todo o documento).

#### Decisões sobre figuras (2026-06-17)

**Figuras adicionadas (duas):**

- **Fig. 4.1** (`burstable/fig1_rearranged.png`) — montagem 2x2: EP e LU, cada um
  com gráfico de tempo e gráfico de custo.
- **Fig. 4.2** (`burstable/fig4_rearranged.png`) — montagem 2x1: microkernel MPI
  (tempo e custo lado a lado).

Ambas inseridas com `width=\textwidth` para legibilidade máxima.

**Fonte das imagens: PDFs originais da publicação WCC (`imgs/WCCimages/`)**

Não usar crops do PDF do artigo compilado (tentado primeiro e descartado).
O artigo compilado agrupa múltiplos plots numa figura com eixos que variam por subplot:
eixo-Y do EP em "400.000" vs LU em "10000.000". Ao extrair e montar em grade 2x2,
as barras ficam desalinhadas porque a largura do label Y difere entre subplots.
A solução foi usar os PDFs individuais submetidos à publicação, disponíveis em
`imgs/WCCimages/`. Cada arquivo é um plot isolado com formatação uniforme, permitindo
montagem com `ImageMagick montage` sem artefatos de alinhamento.

**Construção do fig1_rearranged.png (grade 2x2, 3831x3622 px):**

```bash
# Converter PDFs para PNG a 300 DPI
for f in EP_performace_exp1 EP_cost_exp1 LU_performace_exp1 LU_cost_exp1; do
  pdftoppm -r 300 -png "$f.pdf" "${f%.*}" && mv "${f%.*}-1.png" "${f%.*}.png"
done

# Trim bordas brancas e adicionar margem uniforme
for f in EP_performace_exp1 EP_cost_exp1 LU_performace_exp1 LU_cost_exp1; do
  convert "${f}.png" -trim -bordercolor white -border 30 "${f}_trimmed.png"
done

# Montar grade 2x2
montage EP_performace_exp1_trimmed.png EP_cost_exp1_trimmed.png \
        LU_performace_exp1_trimmed.png LU_cost_exp1_trimmed.png \
        -tile 2x2 -geometry +6+6 -background white fig1_rearranged.png
```

Disposição: linha superior = EP (tempo | custo); linha inferior = LU (tempo | custo).
A comparação EP/LU emerge verticalmente (mesmo tipo de métrica) e horizontalmente
(tempo vs custo para o mesmo benchmark), seguindo leitura em Z natural.

**Construção do fig4_rearranged.png (grade 2x1, 3830x1800 px):**

```bash
for f in "SMB_-_Performace" "SMB_-_Cost"; do
  pdftoppm -r 300 -png "${f}.pdf" "${f%.*}" && mv "${f%.*}-1.png" "${f%.*}.png"
  convert "${f}.png" -trim -bordercolor white -border 30 "${f}_trimmed.png"
done
montage "SMB_-_Performace_trimmed.png" "SMB_-_Cost_trimmed.png" \
        -tile 2x1 -geometry +6+0 -background white fig4_rearranged.png
```

O par mostra que o microkernel reduz as diferenças de ~50%/40% para ~20%/10%,
isolando o efeito da largura de banda do efeito do modelo de créditos.

**Arquivos antigos mantidos como backup (não referenciados no LaTeX):**
- `imgs/burstable/fig1_final.png` — crop do artigo compilado (alinhamento ruim)
- `imgs/burstable/fig4_final.png` — idem

#### Decisões de escrita do corpo (2026-06-17)

**Integração de custo e desempenho:** custo não é discutido em seção separada. O
argumento central em §4.2 é que custo e desempenho contam a mesma história para
cargas longas: T3 mais lenta → mais horas cobradas → custo total maior apesar da
tarifa horária menor (inversão de custo). Essa inversão é apresentada explicitamente
como achado principal do LU, não apenas deduzida de números.

**Parágrafo de abertura do capítulo:** menciona tanto desempenho quanto custo como
critérios da IC ("sem penalizar o desempenho nem elevar o custo total por trabalho"),
enquadrando o capítulo como avaliação multidimensional desde o início.

**Microkernel como evidência de causa:** o SMB isola o efeito do modelo de créditos
da penalidade de largura de banda. Após o microkernel, as diferenças caem de ~50%/40%
para ~20%/10%. Apresentado como evidência, não como experimento separado.

**Justificativa do coordenador burstable (§4.3):** dois elos distintos:
1. Perfil do coordenador (CPU intermitente, rede baixa) coincide com cenário favorável
   ao burstable: créditos acumulam na ociosidade e ficam disponíveis para picos de checkpoint.
2. Ser on-demand (não spot) garante disponibilidade durante preempções dos workers,
   que é exatamente quando a presença do coordenador é mais necessária.
O segundo elo (disponibilidade, não só custo/CPU) deve permanecer explícito em §4.3.

**Regras de estilo global reforçadas nesta sessão:**
- Proibido: travessão (—), en-dash tipográfico (--), "---" em qualquer contexto.
- Substituições: vírgulas, parênteses, dois-pontos, "ou seja", reformulação.
- Construções de lista artificial ("O primeiro é... O segundo é...") substituídas
  por parágrafos com transições naturais.
- O texto foi reescrito duas vezes nesta sessão. A primeira versão ainda tinha lista
  artificial e travessões. A segunda eliminou esses padrões e reorganizou §4.2 em dois
  parágrafos com transição orgânica (EP → LU → microkernel).

---

### 6.4 Step 8 — Chapter 5: Implementação 🔄 Em andamento (§5.1 ✅, 2026-06-18)

**Estimated length:** 12–15 pages.
**Sources:** `PHASE3_ARTIFACT.md`, `PHASE2_ARTIFACT.md`, `TCC_MANA_INTEGRATION_PLAN.md`,
`src/commands/cluster/watcher.rs`, `src/commands/cluster/run_task.rs`.

**Objetivo do capítulo:** mostrar ao leitor o que foi construído, como os componentes se
encaixam e por que as principais decisões de design foram tomadas assim. Não é um
passo-a-passo de implementação, mas uma leitura técnica que deixa o leitor com um modelo
mental claro da arquitetura e capaz de entender os resultados do Cap. 6.

**Cuidado com repetitividade:**
- §2.2.2 já explica Slurm, §2.3.3 já explica MANA internals (split-process, coordinator,
  consistência), §4.3 já justifica t3.large on-demand no head.
- §2.1.2 agora explica SSM e EFS (adicionados nesta sessão) — Cap. 5 referencia sem redefinir.
- Nomes de estratégias: usar **Replace** e **Degraded** (sem o sufixo `_Resume`).

**Estrutura atual (5 seções — §5.2 Topologia fundida em §5.1; §5.3 e §5.4 originais fundidas):**

| Seção | Título | Status |
|---|---|---|
| §5.1 | Visão Geral da Arquitetura | ✅ Escrito |
| §5.2 | Bootstrap Automatizado | ✅ Escrito |
| §5.3 | Detecção de Interrupções e Checkpoint Reativo | ✅ Escrito |
| §5.4 | Estratégias de Recuperação | ✅ Escrito |
| §5.5 | Coordenação entre Processos e Coleta de Métricas | ✅ Escrito |

#### Decisão de reestruturação do Cap. 5 (2026-06-21)

**Problema:** a estrutura anterior com 6 seções separava Detecção (§5.3) e Checkpoint (§5.4)
como se fossem etapas independentes. No código real (`watcher.rs`), não existe ponto de decisão
entre as duas: assim que `enqueue_detected_interruptions` identifica um worker interrompido e
`handle_interruption` é chamado, o pipeline de checkpoint começa imediatamente. Separar em seções
distintas fragmentaria um fluxo que é contínuo e linear no sistema.

**Sobre mover Coordenação para antes da Detecção (proposta alternativa descartada):**
O §5.1 já introduz o modelo dual-processo (`run_task` + `watcher`) e menciona o canal de eventos.
Isso é suficiente para que o leitor tenha o mapa mental antes de ler §5.3. Colocar Coordenação
primeiro exigiria descrever a sequência de eventos SQLite (`interruption_detected`, `checkpoint_completed`,
`node_drained`, ...) antes de o leitor saber o que esses eventos representam — criando confusão no
sentido inverso. Coordenação permanece ao final como seção que "fecha o ciclo": o leitor já viu
todos os eventos sendo produzidos nas seções anteriores.

**Solução: 3 seções restantes (de 4 para 3)**

| Seção antiga | Seção nova | Razão |
|---|---|---|
| §5.3 Detecção + §5.4 Checkpoint | §5.3 Detecção e Checkpoint Reativo | São uma pipeline contínua: detecção dispara checkpoint imediatamente |
| §5.5 Estratégias de Recuperação | §5.4 Estratégias de Recuperação | Renumeração; conteúdo inalterado |
| §5.6 Coordenação e Observabilidade | §5.5 Coordenação e Observabilidade | Renumeração; permanece ao final |

**Coerência visual entre os três diagramas:**

Os três diagramas restantes contam uma história visual sequencial com handoff explícito entre figuras:

- **Fig. 5.3** (Detecção + Checkpoint): watcher poll → detecção → clear/dmtcp/poll-filesystem →
  drain+scancel. **Termina na bifurcação**: ponto de decisão "Replace ou Degraded?" com duas setas
  saindo sem entrar nos caminhos.
- **Fig. 5.4** (Estratégias): **começa exatamente na bifurcação** (`node_drained`), mostra os dois
  caminhos em colunas paralelas com as fases P2a/P2b/P2c rotuladas. O ponto de entrada é o mesmo
  ponto de saída da Fig. 5.3.
- **Fig. 5.5** (Coordenação): visão ortogonal ao fluxo, não uma continuação. Mostra o canal SQLite
  com todos os eventos produzidos ao longo das Figs. 5.3 e 5.4, e como o `run_task` os consome.

Esta continuidade visual reduz a carga cognitiva: o leitor sempre sabe onde está na sequência e
por que a Fig. 5.4 começa onde a Fig. 5.3 termina.

---

#### §5.1 Visão Geral da Arquitetura ✅ Escrito (2026-06-18)

**Figura 5.1 — `imgs/arch/arch-overview.png` ✅ Criada pelo autor**
Mostra: HPC@Cloud CLI (local) com `run_task` e `watcher` → SSM → cluster HPC (head
burstable on-demand: slurmctld + MANA coordinator; workers spot: slurmd + MPI ranks) →
EFS (`/shared`). Consulta à EC2 Spot API (watcher → AWS API Gateway) também visível.
Caption curto: apenas título da imagem. Conteúdo descritivo integrado ao corpo do texto.

**Estrutura do texto (4 parágrafos):**
1. Âncora na figura; diferença vs Fig. 2.1: orquestrador externo (HPC@Cloud CLI fora
   do cluster), SSM e EFS referenciados à §2.1.2 sem redefinição.
2. Dois processos concorrentes do HPC@Cloud: `run_task` (driver do job) e `watcher`
   (monitor de falhas). Não se comunicam diretamente; coordenação via canal de eventos
   (antecipa §5.6 sem detalhar).
3. Slurm e MANA nos nós: daemon names (slurmctld/slurmd) introduzidos com contexto
   funcional pela referência à figura ("conforme mostrado na figura"); referencias a
   §2.2.2 e §2.3.3 sem repetição.
4. Topologia e rationale de instâncias (antes era §5.2 separado): head burstable
   on-demand (ref §4.3 + disponibilidade durante preempções); workers spot (desconto
   até 90%); bridge explícita para tolerância a falhas nas seções seguintes.

**Decisões de design registradas:**
- **Framing "três componentes" descartado:** HPC@Cloud é o sistema; Slurm e MANA são
  ferramentas que ele orquestra, não "componentes" equivalentes. Descrever como sistema
  + ferramentas evita o erro de colocar o watcher falando "com" o Slurm antes de qualquer
  introdução ao watcher.
- **§5.2 Topologia fundida em §5.1:** a figura já mostra head/workers/instâncias; o YAML
  schema foi removido (é interface pré-existente do HPC@Cloud, não arquitetura construída
  no TCC). Topologia virou P4 de §5.1 (rationale de instâncias + bridge).
- **SSM e EFS em §2.1.2:** adicionados como parágrafo novo entre EC2/pricing e "por que
  AWS". Cap. 5 referencia `\ref{sec:aws}` sem redefinir inline.
- **slurmctld/slurmd introduzidos via figura:** nomes técnicos aparecem primeiro no texto
  de §5.1 ancorados em "conforme mostrado na figura", onde os boxes já estão rotulados.
  Não precisam de subseção própria porque o contexto funcional é dado pela figura.
- **Caption curto (apenas título):** o texto descritivo que estava no caption foi
  integrado aos parágrafos do corpo. Caption = "Arquitetura da integração de tolerância
  a falhas no HPC@Cloud."
- **Fig. 5.1 é vista de orquestração, não de topologia:** diferente da Fig. 2.1
  (topologia interna do cluster), a Fig. 5.1 mostra o HPC@Cloud CLI como ator externo,
  SSM como canal, e os dois fluxos de execução. Perspectiva complementar, não repetida.

---

#### §5.2 Bootstrap Automatizado ✅ Escrito (2026-06-18)

**Estrutura final do texto (5 parágrafos + 1 listing + 1 figura):**
1. Declaração declarativa via YAML + `cluster spawn` como ponto de entrada
2. Listagem~\ref{lst:cluster-yaml}: YAML simplificado (2 workers) mostrando role, instance_type,
   allocation_mode, image_id, init_commands (comentados)
3. AMI: imagem pré-configurada com Slurm + MANA + MPICH; bootstrap = configurar, não instalar
4. Infraestrutura: VPC, IPs estáticos (alocados antes das instâncias), provisão de head + workers
   juntos, montagem do EFS
5. Init scripts do head (ordem deliberada dos SCRIPTS, não do provisionamento): slurm.conf +
   munge key + publicação no EFS + sentinel
6. Init scripts dos workers (paralelo): aguarda sentinel → copia configs → slurmd → Running
7. Nota de fechamento: MANA não faz parte do bootstrap; coordinator é per-task

**Figura 5.2 — `imgs/arch/bootstrap-flow.png` ✅ Criada pelo autor**
Diagrama de sequência UML simplificado com três atores: HPC@Cloud, Head node, Worker nodes.
5 steps: (1) Create network infrastructure, (2) Provisioning EC2 instances (head + workers),
(3) Mount EFS on all nodes, (4) Execute HEAD init scripts, (5) Execute Workers init scripts.
Head e workers são instanciados juntos no step 2 (setas 2.1 e 2.2 separadas).
Steps 4 e 5 têm "finished successfully" de retorno; steps de mount não têm retorno explícito
(aceitável: ordering é comunicado pela numeração).

**Decisões de design registradas:**

- **Sem subsections:** `sec:slurm-config` e `sec:mana-config` do skeleton removidos;
  nenhuma delas era referenciada em outro lugar. A seção é prosa contínua.

- **YAML listing incluído:** mostra a interface declarativa do sistema; init_commands
  aparecem como comentários descritivos (não os scripts reais). Caption = título curto.
  Decisão: incluir YAML porque mostra que o usuário define topologia declarativamente,
  o que é relevante para o TCC (ao contrário do schema YAML que foi removido de §5.1).

- **AMI explicada brevemente:** contexto necessário para o leitor entender que o bootstrap
  é de configuração, não instalação. Slurm 24.05.4, MANA e MPICH 3.3.2 já pré-instalados.

- **Sem menção a SSH:** o bootstrap usa SSH internamente, mas o texto não cita o mecanismo.
  Decisão: SSH é detalhe de implementação que não agrega ao entendimento do leitor do TCC.

- **Distinção clara: provisionamento vs scripts de init:**
  O erro original confundia "o head é inicializado primeiro" (que soa como provisionamento)
  com a ordem dos scripts. Texto corrigido: "os comandos do head são executados primeiro".
  Diagrama reforça isso com step 2 provisionando head + workers JUNTOS e step 4/5 mostrando
  a ordem dos scripts separadamente.

- **EFS como mecanismo de coordenação** (não apenas armazenamento): o EFS é o canal pelo
  qual o head publica configs e o sentinel, e os workers consomem. Isso foi enfatizado
  no texto como o mecanismo que permite a inicialização assíncrona entre head e workers.

- **MANA per-task, não bootstrap:** coordinator iniciado no `setup_commands` da task YAML
  via `srun`. Arquivo `.mana-slurm-$SLURM_JOB_ID.rc` distribuído per-task via EFS.
  Mencionado ao final da seção com referência forward a `\ref{sec:estrategias}`.

- **Sentinel como mecanismo de sincronização:** `/shared/slurm/head_ready` (arquivo simples)
  é o sinal que os workers aguardam (timeout 300s). Não há comunicação direta entre nós;
  toda coordenação passa pelo EFS.

- **HPCAC_* vars:** mencionadas no texto como "variáveis de ambiente que descrevem a
  topologia em vigor" sem listar os nomes. Detalhes de implementação omitidos por deliberação.

- **"O que não entra":** detalhes dos comandos shell dos init_commands (muito verbosos),
  porta 7779 do coordinator, deploy de binários MANA, IAM roles/security groups.

---

#### §5.3 Detecção de Interrupções e Checkpoint Reativo (~2.5 p) [MERGED] ✅ Escrito (2026-06-21)

**Fusão de §5.3+§5.4 originais:** no código (`watcher.rs`), a chamada `handle_interruption`
dispara imediatamente o pipeline de checkpoint sem nenhum ponto de decisão intermediário.
Separar seria fragmentar um fluxo linear; ver Decisão de reestruturação acima.

---

##### Decisões de escrita do §5.3 (2026-06-21)

**Estrutura final do texto (5 parágrafos + 1 listing + 1 figura):**

| Bloco | Conteúdo |
|---|---|
| P1 | Bridge de §5.2: cluster inicializado → submissão via task YAML → define job E fault_tolerance no mesmo arquivo → apresenta a Listagem |
| Listing `lst:ft-yaml` | Bloco `fault_tolerance` completo com `auto_test_failure`; vem imediatamente após P1 (não depois dos parágrafos) |
| P2 | Dual process model: `run_task` lança watcher como tarefa assíncrona; watcher monitora todos os workers em ciclos periódicos; explica o `auto_test_failure` em contexto |
| P3 | Detecção: apenas o mecanismo via AWS Spot API (ver decisão de fallback abaixo) + serialização ao final |
| P4 | Pipeline de checkpoint reativo (referencia `fig:watcher-flow`): limpar → `dmtcp_command -c` (não-bloqueante) → poll 5s/300s → drain+cancel → bifurcação → ref §5.4 |
| Fig. 5.3 | Placeholder substituído pela imagem real `arch/watcher-flow.png` |
| P5 | Tarefa independente (restart monitor) + watcher retoma detecção + ref §5.5 (usuário confirmou que esse parágrafo estava bom; mantido com pequeno ajuste) |

**Decisão: fallback por ausência REMOVIDO do texto e do diagrama (2026-06-21)**

O mecanismo de fallback (3 ciclos consecutivos sem resposta) foi removido de ambos o texto e o diagrama.

Motivação técnica: se um worker desaparece sem emitir o aviso da API Spot, os ranks MPI desse
worker já não estão mais acessíveis quando o watcher detecta a ausência. Ao tentar executar
`dmtcp_command -c`, o coordinator MANA não consegue coordenar todos os ranks (o rank do worker
perdido não responde), fazendo o pipeline de checkpoint falhar ou travar até o timeout de 300s.
Resultado: nenhum checkpoint válido é produzido, e o ciclo de recovery é abortado sem possibilidade
de reiniciar com o estado atual.

**Implicação para o texto:** apenas o mecanismo primário (AWS Spot API → `marked-for-termination`)
é apresentado como o canal de detecção. O fallback existe no código, mas não representa um caminho
de recuperação funcional dentro do escopo deste trabalho, portanto não foi incluído.

**Decisão: `auto_test_failure` explicado junto ao YAML, não em parágrafo separado**

A versão anterior introduzia o YAML em um parágrafo ("para reprodutibilidade...") e depois mostrava
a listing. Isso criava sensação de repetição. Solução adotada: P1 introduz o arquivo YAML como um
todo (define job + fault_tolerance), a listing aparece imediatamente, e P2 explica o `auto_test_failure`
dentro do contexto do dual-process model.

**Decisão: referência `sec:checkpoint-reativo` corrigida**

A introdução do capítulo 5 ainda referenciava o label antigo `sec:checkpoint-reativo` (label da versão
pré-fusão). Corrigido para `sec:watcher`. Warning de referência indefinida eliminado.

**Estado da compilação após §5.3:**
- `make` em `lapesd-thesis/` compila sem erros, 72 páginas
- 10 warnings restantes são todos pré-existentes:
  - 2 de fonte (`T1/lmr/bx/sc` — cosmético)
  - 8 de dest de glossário (`glo:EFS`, `glo:AWS`, etc.) — comportamento esperado de `noglossariespages`

---

##### Modelo de concorrência (descoberta 2026-06-21)

O watcher envolve três partes independentes — importante entender antes de escrever:

| Componente | Como roda | O que faz |
|---|---|---|
| Tarefa do watcher | `tokio::spawn` em `run_task.rs` | Loop de detecção + pipeline de recovery (sequencial, bloqueante dentro da tarefa) |
| Tarefa do monitor de restart | `spawn_restart_monitor` → `tokio::spawn` | Criada no final de `handle_interruption`; monitora conclusão do restart; escreve eventos terminais no SQLite |
| `run_task` | Tarefa principal | Consulta SQLite para saber quando recovery terminou; retoma o job MPI |

**Ponto-chave:** detecção + checkpoint + drain + dispatch do restart são **sequenciais** dentro da tarefa do watcher (o loop não avança enquanto `handle_interruption` está em andamento). Somente o monitoramento da conclusão do restart é externalizado para uma tarefa independente via `tokio::spawn`, o que libera o watcher para continuar monitorando outros workers enquanto a aplicação é reexecutada.

**Implicação para o texto:** não é correto dizer que "o watcher tem uma thread de detecção e outra de recovery". O correto é: o watcher executa detecção e recovery em sequência; o que é paralelizado é o monitoramento do restart após o despacho.

**Implicação para o diagrama (Fig. 5.3):** deve mostrar o monitor de restart como uma ramificação que sai ao final do fluxo principal (após "dispatcha restart"), indicando que roda em paralelo enquanto o watcher retoma a detecção.

---

##### Figura 5.3 — Fluxo unificado do watcher

Fluxograma com dois blocos principais (detecção e recovery) e um paralelo final:

```
┌──────────────────────────────────────────────────────────────────┐
│  DETECÇÃO (a cada poll_interval_secs):                           │
│    Para cada worker:                                             │
│      ├─ Já em recovery ou na fila? → pula                       │
│      └─ EC2 Spot API: status de interrupção?                     │
│           (marked-for-termination, instance-terminated-*, etc.)  │
│           ↓ sim                                                  │
│         Enfileira worker                                         │
│                                                                  │
│  Nota: fallback por ausência (3 polls) existe no código mas      │
│  NÃO é apresentado no texto/diagrama — se o worker já sumiu      │
│  sem aviso, o dmtcp_command -c falhará (rank inalcançável).      │
└──────────────────────────────────────────────────────────────────┘
           ↓ pop da fila
┌──────────────────────────────────────────────────────────────────┐
│  RECOVERY (sequencial, bloqueante):                              │
│    interruption_detected + recovery_started → SQLite             │
│                                                                  │
│  [CHECKPOINT]                                                    │
│    → Limpa /shared/checkpoints                                   │
│    → Envia dmtcp_command -c (não-bloqueante) via SSM             │
│    → Poll filesystem a cada 5s (timeout 300s):                   │
│        N ckpt_*.dmtcp + N header.mana + 0 *.dmtcp.temp?         │
│           não → aguarda 5s                                       │
│           sim ↓                                                  │
│    checkpoint_completed → SQLite                                 │
│                                                                  │
│  [DRAIN]                                                         │
│    → Drena nó no Slurm (scontrol DRAIN)                          │
│    → Cancela job corrente (scancel)                              │
│    node_drained → SQLite                                         │
│                                                                  │
│    ↓ BIFURCAÇÃO: Replace | Degraded → Fig. 5.4                  │
│    Dispatcha restart; cria tarefa monitor (paralela)            │
│    Libera in_flight → watcher retoma ciclo de detecção           │
└──────────────────────────────────────────────────────────────────┘
           ↓ (tarefa independente, em paralelo)
    Monitor: poll conclusão do restart (até 3600s)
    → restart_completed → recovery_completed | degraded_resume → SQLite
```

---

##### Conteúdo do texto (parágrafos planejados)

**P1 — Bridge + YAML (estrutura adotada no texto final):**
- Bridge de §5.2: cluster inicializado → task YAML submetido via `cluster run-task`
- Mesmo arquivo define job E fault_tolerance (estratégia, diretório, `auto_test_failure`)
- Listing `lst:ft-yaml` vem imediatamente após; depois: dual-process model e watcher
- "Cada worker identificado pelo IP" foi OMITIDO do texto (detalhismo desnecessário que criava
  confusão sobre o escopo do monitoramento)

**P3 — Mecanismo de detecção (implementado assim no texto final):**
- Único mecanismo apresentado: consulta periódica à EC2 Spot API — condições como `marked-for-termination` indicam interrupção iminente (aviso de até 2 min)
- **Fallback por ausência (3 polls) OMITIDO do texto** — existe no código mas não leva a recovery bem-sucedida (ver decisão acima)
- **Serialização como limitação:** o sistema processa uma recovery de cada vez; interrupções simultâneas são enfileiradas. Mencionado brevemente no texto; direção de trabalho futuro.

**P3 + Listing — Injeção sintética de falhas:**
- O YAML de tasks aceita um bloco `auto_test_failure` com três campos:
  - `trigger_after_secs`: tempo após o início dos `run_commands` para acionar a falha
  - `target_worker_index`: índice do worker-alvo (0-indexed, por IP)
  - `warning_time_secs`: simula a janela de aviso spot (padrão: 120s)
- O mecanismo termina a instância real via EC2 API, garantindo que todos os experimentos sejam acionados no mesmo ponto da execução
- **Mostrar listing YAML** — análogo ao listing do §5.2, adequado aqui. Exemplo simplificado:

```yaml
fault_tolerance:
  strategy: "REPLACE_RESUME"    # ou DEGRADED_RESUME
  process_count: 4              # número de ranks MPI
  checkpoint_dir: "/shared/checkpoints"
  poll_interval_secs: 10        # frequência de monitoramento
  auto_test_failure:            # omitir em produção
    trigger_after_secs: 120     # aciona 2 min após início do job
    target_worker_index: 0      # primeiro worker (ip-10-0-0-11)
    warning_time_secs: 120      # janela de aviso antes da terminação
```

**P4 — Pipeline de checkpoint reativo:**
- Quando interrupção confirmada: limpa o diretório de checkpoint (`/shared/checkpoints`) para remover arquivos de runs anteriores
- Envia o comando de checkpoint ao coordinator MANA (`dmtcp_command -c`) via SSM no head — comando não-bloqueante: retorna imediatamente, mas os ranks escrevem seus arquivos de checkpoint em paralelo e de forma assíncrona
- Por isso, o sistema aguarda verificando o EFS a cada 5 segundos (timeout de 300s): confirma a presença de um arquivo de estado (`ckpt_*.dmtcp`) e um arquivo de cabeçalho (`header.mana`) por rank, e a ausência de arquivos temporários (`*.dmtcp.temp`) — que indicariam gravações ainda em andamento
- Com o checkpoint confirmado: drena o nó no Slurm e cancela o job corrente, persistindo o evento `node_drained`
- Bifurcação para a estratégia configurada → §5.4 (Fig. 5.4)

**Sobre drain + scancel:** não detalhar a ordem nem justificá-la no texto. É um detalhe de implementação sem impacto pedagógico relevante para o leitor do TCC.

**Nomes de comandos e arquivos que PODEM aparecer no texto:**
- `dmtcp_command -c` (o comando de checkpoint)
- `/shared/checkpoints` (diretório no EFS)
- `ckpt_*.dmtcp` e `header.mana` (arquivos gerados por rank)
- `mana_coordinator` (mencionado de passagem; detalhado em §5.4)
- Nomes de eventos SQLite (`interruption_detected`, `checkpoint_completed`, `node_drained`) — aparecem no diagrama; texto pode referenciá-los

**O que NÃO mencionar:**
- `scontrol DRAIN` / `scancel` (detalhes de shell sem valor pedagógico)
- `in_flight`, `queued`, `pending` (nomes de variáveis internas)
- `enqueue_simulated_warning_events` (nome de função interna)
- Porta 7779 do coordinator
- `--exit-on-last` flag (detalhe operacional)

---

#### §5.4 Estratégias de Recuperação (~2.5 p) [era §5.5] ✅ Escrito (2026-06-22)

**Figura 5.4 — Bifurcação Replace vs Degraded — ✅ Criada (autor)**
Arquivo: `imgs/arch/strategy-flow.png`. Label: `fig:estrategias-flow`.
Começa onde a Fig. 5.3 termina: `node_drained`. Mostra os dois caminhos em colunas
paralelas com fases P2b/P2c rotuladas. Deixa claro visualmente onde o custo
dominante de cada estratégia reside (P2b).

**Conexão visual entre Fig. 5.3 e Fig. 5.4 (verificada 2026-06-22):**
- Fig. 5.3 termina em: "REPLACE_RESUME || DEGRADED_RESUME" → "Dispatch Restart"
- Fig. 5.4 começa em: "Checkpoint Made (P1) and Failed Node Drained (P2a)" — marca claramente o handoff
- A ligação é conceitual correta: Fig. 5.4 detalha o que acontece entre a decisão da estratégia e o despacho do restart, zoom no P2b que Fig. 5.3 mostra como caixa única

---

##### Decisões de escrita do §5.4 (2026-06-22)

**Estrutura final do texto (6 parágrafos + 1 tabela + 1 figura):**

| Bloco | Conteúdo |
|---|---|
| P1 | Bridge de §5.3: mesmo estado de partida (checkpoint + nó drenado); duas estratégias com diferença fundamental no tratamento do nó perdido |
| P2 | Introduz fases P0–P3 como framework para o Cap. 6; referencia tabela e §5.5 |
| Tabela `tab:fases-recuperacao` | P0–P3 com Intervalo e Descrição concisa; sem mencionar tempos |
| P3 | P0/P1/P2a são idênticas (já descritas); bifurcação começa em P2b (referencia Fig. 5.4) |
| P4 | Replace P2b: espera terminação (IP estático → `InvalidNetworkInterface.InUse`); nudges; bootstrap §5.2; reintegra ao cluster; forward para Cap. 6 |
| P5 | Degraded P2b: marca DOWN; P2b significativamente mais curto; tradeoff N ranks em N-1 nós; `--oversubscribe`/`--overcommit`; competição CPU em P3; forward para Cap. 6 |
| Fig. 5.4 | `strategy-flow.png` a 0.68\textwidth |
| P6 | P2c convergente: coordinator encerra naturalmente com o job → novo coordinator iniciado → `mana_restart` restaura ranks de `/shared/checkpoints`; forward para Cap. 6 |

**Decisão: tempos (150-200s e ~10s) OMITIDOS do texto e da tabela (2026-06-22)**

Os tempos de P2b para Replace e Degraded foram removidos do texto e das descrições da tabela.
Motivo: a análise quantitativa pertence ao Capítulo de Avaliação Experimental, não à descrição
da implementação. O texto apenas qualifica: Replace tem "custo dominante em P2b"; Degraded tem
P2b "significativamente mais curto". A quantificação é referenciada com forward ao Cap. 6.

**Decisão: coordinator encerra naturalmente com o job (2026-06-22)**

O coordinator MANA encerra automaticamente quando o job é cancelado (comportamento padrão do
processo — ele não precisa ser "morto" manualmente). O script de restart inicia um novo
coordinator antes do `mana_restart`. Uma verificação de segurança existe (para evitar conflito
de porta caso o processo não tenha terminado), mas não é mencionada no texto.

Texto adotado: "O coordinator MANA do job interrompido encerra naturalmente junto com o
cancelamento do job, de modo que um novo coordinator é iniciado antes do restart ser
despachado."

**Decisão: sem subsections (2026-06-22)**

O skeleton original tinha `\subsection{\textsc{Replace\_Resume}}` e
`\subsection{\textsc{Degraded\_Resume}}`. Removidas — Replace e Degraded são parágrafos de
prosa, consistente com o estilo de §5.2 e §5.3 (sem subseções).

**Tabela `tab:fases-recuperacao` — descrições finais:**

| Fase | Intervalo | Descrição |
|---|---|---|
| P0 | Início do job até detecção | Execução normal antes da interrupção |
| P1 | Detecção até checkpoint concluído | Escrita do checkpoint ao detectar a interrupção |
| P2a | Checkpoint concluído até nó drenado | Drenagem do nó e cancelamento do job no Slurm |
| P2b | Nó drenado até nó disponível | Recuperação de infraestrutura; ponto de divergência entre as estratégias |
| P2c | Nó disponível até restart despachado | Inicialização do coordinator MANA e despacho do restart |
| P3 | Restart despachado até conclusão | Execução retomada; N workers (Replace) ou N-1 workers (Degraded) |

**Tabela de fases** (referenciada ao longo do Cap. 6):

| Fase | Intervalo | Descrição |
|---|---|---|
| P0 | Início até detecção | Execução normal antes da falha |
| P1 | Detecção → checkpoint_completed | Escrita do checkpoint |
| P2a | checkpoint_completed → node_drained | Drain + scancel (igual nas duas estratégias) |
| P2b | node_drained → node_became_idle | Recuperação da infraestrutura (diferenciador) |
| P2c | node_became_idle → restart_dispatched | Reconfigura Slurm + despacha restart |
| P3 | restart_dispatched → terminal event | Execução restante pós-recovery |

**Replace:**
- Espera instância terminar completamente (não apenas sair de "running"). Motivo: o IP
  privado é estático (determinístico por índice). Respawn antes da rede ser liberada
  gera erro `InvalidNetworkInterface.InUse`. Watcher envia nudges a cada 15s para
  destravar instâncias presas em `shutting-down`.
- Respawn via HPC@Cloud (mesmo fluxo de spawn, mesmo IP).
- `scontrol RESUME` → aguarda IDLE → restart com N processos em N nós.
- Custo dominante: P2b ~150-200s (EC2 provisioning + boot + MANA deploy).

**Degraded:**
- `scontrol DOWN` no nó perdido (sem respawn, sem espera).
- Restart com N processos em N-1 nós usando `--oversubscribe --overcommit`.
- Por que essas flags são necessárias: sem elas, o Slurm recusa alocações em que
  #processos > #slots disponíveis (4 vCPUs × (N-1) nós < N processos). `--oversubscribe`
  permite múltiplos tasks por slot; `--overcommit` remove o teto de `ntasks_per_node`.
  Juntos, permitem distribuir N ranks sobre N-1 nós sem modificar o código MPI.
- Implicação de desempenho: P3 do Degraded tem mais processos por nó (competição de CPU).
- Custo dominante: P2b ~10s (apenas `scontrol DOWN` via SSM).

**Nota sobre o restart em ambas as estratégias:**
O restart requer um novo `mana_coordinator`. O script sempre mata o coordinator anterior
e inicia um novo antes do `srun mana_restart` (consequência da arquitetura split-process
do MANA, §2.3.3: um coordinator com estado de sessão anterior conflitaria com o novo job).

---

#### §5.5 Coordenação entre Processos e Coleta de Métricas ✅ Escrito (2026-06-22)

**Título final:** "Coordenação entre Processos e Coleta de Métricas" (título original do plano era "Observabilidade"; ajustado para refletir melhor o conteúdo real).

**Figura 5.5 — OMITIDA (decisão 2026-06-22):**
O plano previa um diagrama de estados. Decidido não criar: o algoritmo formal (Algoritmo 5.1)
cumpre a função com mais precisão e no mesmo espaço. Não há Fig. 5.5 no texto final.

---

##### Decisões de escrita do §5.5 (2026-06-22)

**Estrutura final (4 parágrafos + 1 algoritmo):**

| Bloco | Conteúdo |
|---|---|
| P1 | Bridge de Fig. 5.1: duas tarefas Tokio concorrentes no mesmo processo CLI; o que a seção cobre |
| P2 | Canal de coordenação: SQLite **local ao processo** (não EFS); pool compartilhado; watcher escreve eventos, run_task consulta |
| P3 | Comportamento do run_task durante recuperação: falha esperada vs erro real; laço de espera; `recovery_failed` não encerra o laço (watcher retenta); timeout aborta |
| Algoritmo 5.1 | `algorithm` + `algpseudocode`; nível médio — mostra estrutura IF-ELSE e While sem expor contadores internos |
| P4 | `[RUN_METRICS]`: metadados + durações de fase calculadas dos timestamps de `interruption_events`; emitido apenas em execuções com sucesso; usado pelo script de análise |

**Decisão: SQLite é LOCAL ao processo CLI (2026-06-22)**

Versão inicial errada dizia "SQLite no EFS". Corrigido após leitura do código:
`run_task` e `watcher` são `tokio::spawn` tasks no mesmo processo, compartilhando o
mesmo `SqlitePool`. A base é um arquivo local na máquina do usuário, não no EFS.
O EFS é usado apenas para checkpoints MANA e configuração do Slurm.

**Decisão: sem figura — usar algoritmo formal (2026-06-22)**

O plano previa Fig. 5.5 (diagrama de canal SQLite). Substituído por `Algoritmo 5.1`
usando o ambiente `algorithm` + `algpseudocode` (já carregado pelo lapesd-thesis, ver
Algoritmo 2.1 em §2.3.3). O algoritmo comunica a lógica do run_task com maior precisão
e sem adicionar uma figura de baixo valor pedagógico.

**Decisão: linguagem formal, sem perguntas ao leitor (2026-06-22)**

Primeira versão do §5.5 usava perguntas retóricas ("o que faz o run_task?"). Reescrito
em estilo declarativo, adequado para TCC.

**Decisão: nível do algoritmo — meio-termo (2026-06-22)**

Três iterações até o nível certo:
1. Primeira versão: muito alto nível (`aguardar restart_dispatched... aguardar restart_completed`)
2. Segunda versão: baixo demais (`$n_s$`, `$n_t$`, `\Repeat...\Until{$n_s \leq n_t$ e último terminal ≠ recovery_failed}`)
3. Versão final: `\ElsIf{há ciclo de recuperação ativo}` + `\While{sem evento terminal bem-sucedido}` + `\If{tempo limite atingido}` — mostra a estrutura de decisão sem expor contadores

**Comportamento real do run_task ao detectar falha (verificado no código 2026-06-22):**

- `wait_for_watcher_recovery_if_needed` checa `recovery_started_count vs terminal_count`
- `recovery_failed` É um evento terminal, mas NÃO encerra o laço — run_task continua esperando nova tentativa
- Eventos terminais bem-sucedidos: `recovery_completed` (Replace) e `degraded_resume` (Degraded)
- Se timeout (`recovery_timeout_secs`, padrão 86400s): função retorna `Err`, execução abortada
- run_task NÃO retenta o comando — faz `run_idx += 1` (avança) tratando a falha como recuperada

**`[RUN_METRICS]` — campos emitidos:**

```
task_tag, strategy, workers, worker_instance_type, head_instance_type
ft_wall_time_s, setup_time_s, total_time_s
phase1_s, phase2_s, phase2a_s, phase2b_s, phase2c_s, phase3_s
status=SUCCESS
```

Emitido apenas se task concluída com sucesso (garante integridade ao parse).
Script `analyze.py` parseia blocos `[RUN_METRICS]...[/RUN_METRICS]` do arquivo de resultado.

---

#### Figuras do capítulo

| Figura | Seção | Conteúdo | Status |
|---|---|---|---|
| Fig. 5.1 | §5.1 | Arquitetura da integração (arch-overview.png) | ✅ Criada (autor) |
| Fig. 5.2 | §5.2 | Sequência de bootstrap (bootstrap-flow.png) — diagrama de sequência UML | ✅ Criada (autor) |
| Fig. 5.3 | §5.3 | Fluxo unificado: watcher poll → detecção → checkpoint → drain → bifurcação | ✅ Criada (autor) |
| Fig. 5.4 | §5.4 | Bifurcação Replace vs Degraded com fases P2b/P2c; começa onde Fig. 5.3 termina | ✅ Criada (autor) |
| Fig. 5.5 | §5.5 | Canal SQLite: eventos produzidos ao longo de Fig. 5.3/5.4 e como run_task os consome | ⬜ Step 12 |

**Continuidade visual:** Fig. 5.3 termina na bifurcação `[Replace|Degraded]`; Fig. 5.4
começa em `node_drained` mostrando os dois caminhos; Fig. 5.5 é visão ortogonal
(canal de comunicação, não continuação de fluxo).

#### O que NÃO entra no capítulo

- Parâmetros internos do `slurm.conf` (TaskPlugin, SlurmctldAddr, etc.)
- Schema YAML de cluster (interface pré-existente do HPC@Cloud, não arquitetura do TCC)
- O script completo de restart (190+ linhas)
- Schema SQL da tabela `interruption_events`
- Contagens exatas de retry/timeout (exceto onde justificam uma decisão de design)
- Por que `pmi2` (detalhe operacional sem impacto conceitual)

---

### 6.5 Step 9 — Chapter 6: Avaliação Experimental

**Estimated length:** 18–22 pages. The most figure/table-heavy chapter.
**Source:** `analysis_report_phase5.md` (fonte primária — correto e verificado contra os CSVs),
`report_tables.md`, all figures in `imgs/plots/`.

**ATENÇÃO — erros corrigidos no plano (2026-06-24):**
Os valores abaixo estão corretos e verificados contra os CSVs. O plano anterior continha:
- EP overhead incorreto (14–39%): correto é 26.9–46.7%.
- CG 2w overhead incorreto (251%): correto é 222%.
- Checkpoint size descrito como "Non-linear": correto é aproximadamente linear (conforme analysis_report).
- Limitação "Single repetition per point": INCORRETA — Phase 5 tem N=3 por célula.

**ESTILO DE ANÁLISE (obrigatório em todas as seções):**
NÃO escrever "no cenário X a estratégia Y venceu". O foco é em **tendências estruturais e
comportamentos do sistema**:
- "O sistema tende a...", "o comportamento observado revela que...", "à medida que N cresce..."
- Identificar a RAZÃO MECÂNICA por trás de cada padrão (ex: P2b é custo fixo de EC2,
  independente de N; P3 de DEGRADED é custo que escala com trabalho restante).
- Resultados pontuais (tabelas, números exatos) entram na tabela/legenda — a prosa extrai
  o comportamento geral.
- Seguir o estilo do `analysis_report_phase5.md` §7.2–7.5 e §9.2 como referência de tom.

---

#### §6.1 Configuração dos Experimentos ✅ ESCRITO (2026-06-24)

**Análise:** seção objetiva, sem tendências a discutir. Apresentar ambiente e matriz de forma
clara para que o leitor possa contextualizar todos os resultados seguintes.

**✅ Decisões de escrita (§6.1.1):**
- AMI **não mencionada** no texto (ID técnico de baixo valor para o leitor do TCC).
- Coluna "Rede" removida da tabela de hardware; apenas vCPU, RAM, preços on-demand e spot.
- MANA versão: `v1.2.0` (não o hash de commit).
- Fonte de todas as tabelas: `\fonte{Elaborado pelo autor.}` (padrão para todo o Cap. 6).
- EFS mencionado apenas como armazenamento compartilhado em `\texttt{/shared/checkpoints}`;
  sem citação de capacidade ou burst — simplicidade intencional.
- Financiamento CNPq/AWS colocado como **último parágrafo de §6.1.2** (não em §6.1.1).

**✅ Decisões de escrita (§6.1.2):**
- As 4 configurações (noFT, MANA-noFT, REPLACE, DEGRADED) são **explicadas brevemente
  em prosa** antes da tabela (não apenas listadas na tabela) para que o leitor entenda
  o que cada uma representa antes de ver os dados.
- Justificativa de classes (C, D) **referenciada para §2.5** (Fundamentação Teórica),
  que é onde foi justificada originalmente. Confirmado como estrutura adequada para TCC.
- Estudos sintéticos mencionados no último parágrafo de §6.1.2 como complemento;
  detalhados em §6.3 (sec:sinteticos, que é uma `\section` separada no body.tex).

**§6.1.1 Ambiente e Infraestrutura** (referência — dados já no documento)

| Componente | Especificação |
|---|---|
| Nó coordenador | t3.large, on-demand, us-west-2 |
| Workers | m5.xlarge (4 vCPU, 16 GB RAM), spot |
| SO | Amazon Linux 2023 |
| MPI | MPICH 3.3.2 |
| MANA | v1.2.0 |
| Slurm | 24.05.4 |
| NPB | 3.4.4 |
| Armazenamento | Amazon EFS, montado em /shared/checkpoints |

**§6.1.2 Matriz de Experimentos** (referência)
- 3 benchmarks (CG-C, EP-D, LU-C) × 3 tamanhos (2/4/8 workers) × estratégias (noFT,
  MANA-noFT, REPLACE, DEGRADED) × variações de timing + estudos sintéticos.
- Total: **282 execuções válidas**, 3 repetições por célula; valores reportados como média ± dp.
- Mencionar `auto_test_failure` como mecanismo de injeção controlada de falha (referência §5.3).
- Nota: os estudos sintéticos (synth_calls, synth_imbalanced, synth_checkpoint_size) testam
  variáveis isoladas para identificar mecanismos de overhead — apresentados em §6.3.

---

#### §6.2 Overhead do MANA sem Falhas ✅ ESCRITO (2026-06-24)

**Objetivo da seção:** caracterizar o custo que o MANA impõe a uma execução normal —
sem nenhuma falha — antes de discutir recuperação. O leitor precisa saber o "piso" de
overhead antes de avaliar se a tolerância a falhas vale a pena.

**✅ Decisões de escrita (§6.2 — estrutura no body.tex):**
- `\section{Overhead do MANA sem Falhas}` tem 2 subseções:
  - `\subsection{Impacto por Benchmark}` (sec:overhead-benchmark) — fig1 + tab:overhead
  - `\subsection{Escalabilidade}` (sec:escalabilidade) — fig6
- A seção `\section{Estudos Sintéticos de Overhead}` é uma **seção separada** no body.tex
  (sec:sinteticos), não uma subseção de §6.2. Os stubs do body.tex mantêm essa estrutura.
- Figuras regeneradas com `figsize` maior: fig1 `(14,8)` em vez de `(14,5)`;
  fig6 `(15,6.5)` em vez de `(15,4.5)`. Razão: plots originais tinham apenas 5–6 cm
  de altura no PDF, pequeno demais. Com os novos tamanhos: ~8.6 cm e ~6.5 cm.
- `fig.suptitle()` removido de ambas as figuras — LaTeX já fornece caption; título
  interno era redundante.

**✅ Decisões de escrita (§6.2.1 — Impacto por Benchmark):**
- **Abertura com perfil de tempo de execução**: o parágrafo de análise abre com o contexto
  dos perfis distintos (EP maior 99–334 s, LU médio 48–148 s, CG menor 12–34 s). Integrado
  ao início do parágrafo de LU+EP — não como parágrafo separado. Razão: contextualiza
  imediatamente por que a sensibilidade ao overhead difere entre benchmarks.
- **LU-C e EP-D analisados juntos**: comparados em um único parágrafo. Tendência
  compartilhada (ambos reduzem % em 8 workers) identificada antes de explicar o contraste
  entre eles (LU 10 pp, EP 20 pp).
- **Componente fixo de custo**: explicado brevemente (inicialização/finalização de processo)
  antes de dizer que o padrão observado é o oposto. Motivação: sem esse contexto, a queda
  do % com mais workers parece trivial, não contraintuitiva.
- **CG-C hipótese revisada**: a hipótese de co-localização de EC2 foi **descartada**
  porque não explicaria por que o efeito é específico ao CG (LU e EP têm 2 workers também
  e não mostram o fenômeno). Hipótese atual: **desbalanceamento de carga por particionamento
  esparso**. CG realiza produtos esparsos de matriz × vetor com dados irregulares; com 2
  processos, o particionamento grosso cria desbalanceamento persistente; o laço ativo do
  MANA amplifica o atraso ao longo de 75 iterações. EP (embaraçosamente paralelo) e LU
  (grade regular, troca estruturada) não geram esse desbalanceamento.
- **CG sensibilidade por tempo curto**: integrado ao parágrafo CG como primeiro ponto.
  O overhead absoluto em 2w (75,7 s) supera o tempo noFT (34,1 s) — isso é evidência de
  amplificação real, não apenas efeito de denominador pequeno.
- **label{sec:gap-overhead}** mantido como label inline antes do parágrafo final de §6.2.1
  (para cross-references do body.tex funcionarem).
- **Parágrafo final (gap)**: apresenta a distância 4–5 s (sintéticos) vs 27–222% (reais)
  como achado central aberto. CG e EP/LU contrastados explicitamente: LU/EP reduzem %;
  CG exibe tendência inversa 4→8w e anomalia em 2w.

**✅ Decisões de escrita (§6.2.2 — Escalabilidade):**
- Comprimido para **um único parágrafo** + figura. O segundo parágrafo original (dados
  absolutos de overhead por EP/LU) foi removido por ser repetição de §6.2.1.
- Conteúdo único mantido: MANA-noFT segue mesma curva de escalabilidade que noFT;
  aceleração EP 3,4× e LU 3,1× com valores similares sob MANA; sem gargalo de escalabilidade.
- "N" substituído por "número de workers" em toda a seção.

**§6.2.1 Caracterização por Benchmark e Escalabilidade** (referência de dados)
Figuras: `fig1_mana_overhead.png` + `fig6_mana_scalability.png`
Tabelas: `table01_mana_overhead.csv` (tab:overhead no documento)

**Dados corretos (verificados contra CSV):**
| Benchmark | 2 workers | 4 workers | 8 workers |
|---|---|---|---|
| CG-C | **+222%** (34.1 → 109.8 s) | +55% (20.2 → 31.2 s) | +71% (12.2 → 20.9 s) |
| EP-D | +45% (334.3 → 484.5 s) | +47% (169.7 → 249.0 s) | **+27%** (99.1 → 125.7 s) |
| LU-C | +63% (147.8 → 241.2 s) | +63% (83.1 → 135.1 s) | +53% (47.6 → 72.7 s) |

**Análise — o que a prosa deve explorar (NÃO apenas listar os números):**
- O overhead não apresenta padrão consistente entre benchmarks: LU mostra estabilidade
  (~53–63%) enquanto EP apresenta tendência de redução com mais workers (45→47→27%) e
  CG tem comportamento anômalo em 2w. Explorar isso como ausência de mecanismo único.
- fig6 mostra que MANA-noFT segue a **mesma curva de escalabilidade** que noFT:
  o overhead não piora com mais workers, o que significa que a infraestrutura de checkpoint
  do MANA não introduz gargalo de escalabilidade.
- CG em 2w (+222%) deve ser destacado como achado anômalo — contraste marcante com
  CG em 4w (+55%); hipótese: desbalanceamento esparso + laço ativo (NÃO co-localização).
- **NÃO concluir ainda** qual mecanismo causa o overhead dos benchmarks reais — isso
  vai para §6.3 (Estudos Sintéticos).

**§6.2.2 Investigação dos Mecanismos: Estudos Sintéticos**
Figuras: `fig2_synth_calls.png` + `fig2b_synth_imbalanced.png`
Tabelas: `table03_synth_calls.csv` + `table04_synth_imbalanced.csv`

Dados de referência: overhead sintético estável em ~3.5–5 s em todos os níveis de ambos
os estudos (L0-L4; 0–51.200 chamadas; delay 0–20 ms).

**Análise — o que a prosa deve explorar:**
- O foco é eliminar hipóteses sobre o que CAUSA o overhead dos benchmarks reais:
  frequência de chamadas MPI não é o driver; desbalanceamento de comunicação entre
  nós separados tampouco. Os sintéticos convergem para um overhead de infraestrutura
  fixo de ~4–5 s.
- Apresentar o mecanismo do spin-loop do MANA/DMTCP (MPI_Wait como busy-wait entre
  DISABLE_CKPT/ENABLE_CKPT) como contexto técnico para entender o que os sintéticos
  medem — e o que eles NÃO medem.
- A hipótese de cascade por co-localização (CG 2w): o sintético de desbalanceamento
  não pode reproduzir esse efeito porque o sender está em nó separado. A lacuna entre
  os sintéticos (~4–5 s) e os benchmarks reais (27–222%) é um achado em aberto —
  não apresentar como conclusão, mas como hipótese bem fundamentada.
- **Tom da análise:** "os estudos sintéticos delimitam o espaço de busca mas não
  identificam a causa raiz no caso dos benchmarks reais" (ver análise §2.4 do report).

**§6.2.3 Pegada de Memória e Tempo de Checkpoint (Fase P1)**
Figura: `fig3_synth_ckpt.png`
Tabela: `table05_synth_ckpt.csv`

Dados: 50 MB → 16.4 s; 200 MB → 26.9 s; 800 MB → 47.0 s; 3.200 MB → 139.2 s
(todos com std ≤ 0.4 s).

**Análise — o que a prosa deve explorar:**
- O tempo de P1 cresce **aproximadamente linearmente** com a memória por processo
  (o analysis_report usa "scales linearly" — manter essa terminologia). A baixa
  variabilidade (std ≤ 0.4 s) confirma que a largura de banda do EFS é estável.
- O ponto de 3.200 MB (~139 s) se aproxima do teto de burst do EFS (~105 MB/s):
  apresentar o cálculo (4 processos × 3.200 MB / 139 s ≈ 92 MB/s ≈ 88% do teto).
- **Implicação prática e ponte para §6.3:** jobs com pegada de memória por processo
  acima de ~3–4 GB corriam o risco de não concluir o checkpoint dentro da janela de
  2 minutos do aviso spot. Isso delimita o espaço de aplicações para as quais a
  abordagem é viável. Além disso, explica por que EP-D tem P1 mais curta (~16–20 s)
  que LU-C (~26.6 s) nos experimentos com falha — diferença de footprint de memória.

---

#### §6.3 Estudos Sintéticos de Overhead ✅ ESCRITO (2026-06-24)

**Objetivo da seção:** explicar mecanisticamente por que os benchmarks reais mostram
27–222% de overhead quando os sintéticos medem apenas 4–5 s. Cada subseção testa uma
hipótese com uma variável isolada. A conclusão é que a discrepância não tem causa
identificável com esses dados — o que o estudo consegue é **estreitar o espaço de
hipóteses**.

**✅ Decisões de escrita (parágrafo introdutório de §6.3):**
- Palavra "gap" substituída por "discrepância" em toda a seção (e na referência de §6.2.1).
  Razão: "gap" é informal e de sabor técnico-anglófono; "discrepância" é mais preciso e
  adequado ao registro acadêmico formal.
- Motivação da necessidade de isolamento (benchmarks misturam computação, comunicação e
  efeitos de cluster) apresentada antes de enumerar as hipóteses.
- A abertura menciona os programas sintéticos (synth_calls, synth_p2p, synth_imbalanced,
  synth_checkpoint_size) brevemente, referenciando as subseções subsequentes.

**✅ Decisões de escrita (§6.3.1 — Frequência de Chamadas MPI):**
- Hipótese testada explicitada antes dos dados: overhead acumula com número de chamadas MPI?
- Overhead plano (~3,5–5 s), independente de 0 a 51.200 chamadas: apresentado como resultado
  central, com destaque para o que ele exclui (custo por chamada).
- synth_p2p com leve queda em L4 explicado como sincronização sender/receiver que inibe o
  laço ativo — não como redução de overhead em geral.
- Conclusão explícita: frequência de chamadas **descartada** como causa da discrepância.
- Figura `fig2_synth_calls.png`: xticklabels com `rotation=20, ha='right'` para evitar
  sobreposição dos labels "L{N}\n({K} calls)" após aumento de fonte.

**✅ Decisões de escrita (§6.3.2 — Comunicação Desbalanceada e Laço Ativo):**
- Seção estruturada em dois blocos:
  1. Explicação do mecanismo `MPI_Wait` no MANA: listagens comparando implementação
     nativa (`PMPI_Wait` — bloqueia no kernel) vs MANA (`MPI_Test_internal` em laço ativo
     entre `DMTCP_PLUGIN_DISABLE_CKPT` / `DMTCP_PLUGIN_ENABLE_CKPT`).
  2. Resultado do experimento sintético e conclusão.
- **Conexão com CG:** parágrafo introdutório menciona que a seção foi concebida inicialmente
  como análogo ao CG (padrão ponto-a-ponto), mas o cenário não se encaixa perfeitamente
  (discussão ao final da seção). Postura honesta: CG tem co-localização como hipótese aberta.
- **`MPI_Test_internal` vs `PMPI_Test`:** o listing usa `MPI_Test_internal` (como no
  analysis_report) porque é a função interna do MANA que chama o MPI real sem passar
  pelos wrappers do próprio MANA. `PMPI_Wait` é correto no listing nativo (interface de
  profiling MPI que bypassa wrappers do usuário). Essa distinção foi validada contra o
  analysis_report.
- **Proibição de "handler do DMTCP":** esse termo nunca foi introduzido ao leitor;
  reescrito como "O \textit{wrapper} do MANA" para o leitor que chegou via §5.3.
- **Sem travessão:** 4 iterações de revisão até atingir texto formal sem "—", "---", nem
  construções de lista artificial. Substituições: vírgulas, parênteses, dois-pontos,
  reformulação de orações.
- **Parágrafo WHY (por que o laço ativo):** estrutura em duas partes:
  1. Limitação do `PMPI_Wait` nativo: suspende no kernel → não pode fazer checkpoint.
  2. Solução do MANA: laço com `MPI_Test_internal` (não-bloqueante, retorna imediato) +
     liberação do lock entre iterações → permite interromper para checkpoint. Contrapartida:
     consome CPU continuamente em espaço de usuário.
- Resultado: overhead constante ~4–5 s mesmo com 20 ms de delay no sender. Conclusão:
  o laço ativo **sozinho** (sem co-localização) não amplifica o overhead. Teste sintético
  exclui imbalance em nós separados, não imbalance com co-localização.

**✅ Decisões de escrita (§6.3.3 — Pegada de Memória e Tempo de Checkpoint):**
- Phase 1 escala linearmente; dp ≤ 0,4 s confirma EFS estável.
- A 3.200 MB/processo: demanda agregada ≈ 92 MB/s (88% do teto de burst do EFS).
  Implicação: footprint > ~3–4 GB excede a janela de 2 min do aviso spot.
- Conexão com benchmarks reais: EP-D footprint menor → P1 ~16–20 s vs LU-C/CG-C ~26–37 s.
- Nota de escopo explícita: Phase 1 aparece apenas quando há falha; este resultado é sobre
  previsibilidade do tempo de checkpoint, não sobre o overhead normal do MANA (§6.2).

**✅ Decisões sobre figuras (sesão inteira §6.3):**
- `fig.suptitle()` removido de TODAS as figuras; `ax.set_title()` MANTIDO para painéis
  individuais (fig1, fig2, fig6). Razão: `suptitle` duplicava o caption do LaTeX (redundante);
  `ax.set_title()` identifica painéis individuais em figuras multipanel (necessário).
- **Distinção crítica estabelecida:** `fig.suptitle()` = título global da figura (removido);
  `ax.set_title()` = título do painel individual (mantido). Confundir os dois resulta em
  perda de informação nos multipanel ou duplicação desnecessária.
- `rcParams` calibrados em `main()` antes de qualquer chamada de plot:
  `font.size=14, axes.titlesize=15, axes.labelsize=14, xtick/ytick.labelsize=12, legend.fontsize=12`.
  Valores 22 pt foram rejeitados ("muito exagerado"); 14 pt foi aceito.
- **Symlink criado:** `imgs/plots/ → ../data/plots/` para que LaTeX leia automaticamente
  os arquivos gerados por `analyze.py` (que salva em `data/plots/`) sem cópia manual.
  Comando: `cd imgs && rm -rf plots && ln -s ../data/plots plots`.
  Bug anterior: LaTeX compilava com imagens antigas porque lia `imgs/plots/` enquanto
  `analyze.py` salvava em `data/plots/` — os arquivos nunca eram atualizados.
- Anotações de canto (`ax.text(0.03, 0.97, ...)`) substituídas por `ax.set_title()` em
  fig1, fig2, fig6 após feedback que o canto tampava conteúdo e deslocava a legenda.

**§6.3.1 — Dados de referência (synth_calls + synth_p2p, overhead adicionado, N=3):**
| Nível | Chamadas | synth_calls overhead | synth_p2p overhead |
|---|---|---|---|
| L0 | 0 | +3,5 s | +4,4 s |
| L1 | 800 | +4,8 s | +4,3 s |
| L2 | 3.200 | +4,5 s | +5,9 s |
| L3 | 12.800 | +4,7 s | +3,8 s |
| L4 | 51.200 | +4,3 s | +2,2 s |

**§6.3.2 — Dados de referência (synth_imbalanced, overhead adicionado, N=3):**
| Nível | Delay no sender | noFT (s) | MANA-noFT (s) | Overhead |
|---|---|---|---|---|
| L0 | 0 µs | 63,5 s | 67,6 s | +4,1 s |
| L1 | 100 µs | 63,2 s | 67,6 s | +4,5 s |
| L2 | 1 ms | 63,4 s | 68,1 s | +4,7 s |
| L3 | 5 ms | 67,6 s | 72,0 s | +4,4 s |
| L4 | 20 ms | 78,7 s | 83,6 s | +4,9 s |

**§6.3.3 — Dados de referência (synth_checkpoint_size, N=3, Phase 1 = detect → checkpoint no EFS):**
| Memória/processo | Phase 1 | Notas |
|---|---|---|
| 50 MB | 16,4 ± 0,1 s | imagem pequena |
| 200 MB | 26,9 ± 0,4 s | |
| 800 MB | 47,0 ± 0,2 s | |
| 3.200 MB | 139,2 ± 0,4 s | próximo do teto do EFS Bursting (~105 MB/s) |

---

#### §6.4 Análise das Fases de Recuperação ✅ ESCRITO

**Estrutura final (2 subseções — §6.4.3 foi eliminado):**

```
§6.4 Análise das Fases de Recuperação
  §6.4.1 Decomposição Temporal        ← fig5 (CG-C) + tab:cg-ft-breakdown
  §6.4.2 Sensibilidade ao Momento da Falha  ← fig4 + tab:warm-restart-gap
```

§6.4.3 "Hipótese de Reinício Aquecido" foi planejada e depois eliminada — nenhuma
hipótese mecanística sobreviveu ao escrutínio (ver decisão abaixo). O comportamento
anômalo do REPLACE é reportado em §6.4.2 com tabela e indicação de trabalho futuro.

---

**✅ Decisões de escrita (§6.4.1 — Decomposição Temporal):**

- **Forward reference para dados não vistos:** a frase "Nos demais benchmarks, esse
  comportamento não se repete" foi removida porque o leitor ainda não viu EP/LU.
  Substituída por: "Como se verificará na Seção~\ref{sec:timing-sensitivity}, esse
  crescimento é específico ao CG..." — padrão para qualquer dado que aparece depois.

- **Restrição de escopo nas afirmações:** "P2b não varia com o job nem com o número
  de workers" generalizava além dos dados disponíveis na subseção. Corrigido para
  "nas três configurações de cluster do CG-C" — restringir sempre ao subconjunto
  de dados já apresentado no momento da afirmação.

- CG escolhido como caso ilustrativo: job curto (noFT ~12–34 s) torna as proporções
  imediatamente visíveis. Omitir o termo "caso ilustrativo" no texto (formal demais).

- **P1 em CG 8w cresce para ~37 s** (vs ~26–27 s em 2w/4w): CG usa estado MPI por
  rank (buffers de comunicação esparsa), então mais workers = checkpoint maior por
  processo. EP e LU não mostram esse efeito. Consistente com §6.3.3.

Dados de referência — CG-C (médias, N=3):
| Workers | Estratégia | P0 | P1 | P2a+P2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|
| 2w | REPLACE | 3,0 s | 26,6 s | 10,9 s | 120,7 s | 40,2 s | 206,4 s |
| 2w | DEGRADED | 3,0 s | 26,5 s | 10,8 s | 10,9 s | 113,1 s | 168,6 s |
| 4w | REPLACE | 3,0 s | 26,7 s | 10,8 s | 123,3 s | 26,4 s | 196,2 s |
| 4w | DEGRADED | 3,0 s | 26,5 s | 10,8 s | 10,9 s | 35,0 s | 92,6 s |
| 8w | REPLACE | 3,0 s | 36,7 s | 10,8 s | 119,7 s | 24,6 s | 203,2 s |
| 8w | DEGRADED | 3,0 s | 36,9 s | 11,1 s | 11,3 s | 24,8 s | 96,7 s |

---

**✅ Decisões de escrita (§6.4.2 — Sensibilidade ao Momento da Falha):**

**Parágrafo DEGRADED — framing contraintuitivo:**
- O texto introduz explicitamente que o resultado "pode parecer contraintuitivo": como
  a DEGRADED reinicia com N-1 workers, falhas precoces implicam mais trabalho com
  capacidade reduzida — esperaria-se barras maiores nesses casos.
- Resolução: P0 crescendo e P3 diminuindo (proporcionalmente) se compensam; total
  permanece aproximadamente estável.
- **Regra de linguagem:** NÃO escrever "Isso é contraintuitivo." (afirmação forte
  e informal). Usar "pode parecer contraintuitivo" (hedged, adequado para TCC).
- NÃO dizer "compensação imperfeita": se P3 encolhe mais devagar que P0 cresce, o
  total deveria crescer — contradiz a observação. A compensação é suficiente para
  manter o total aproximadamente estável; omitir o qualificador "imperfeita".

**Parágrafo REPLACE — anomalia honesta (hipótese eliminada):**
- P3 do REPLACE é sistematicamente mais curto que o esperado (MANA-noFT − P0).
- Gap% entre 20–34% para EP-D e 5–34% para LU-C, **consistente entre repetições**.
- **Hipóteses rejeitadas** durante a escrita:
  - Variabilidade de EC2: descartada — comportamento presente em todas as repetições
    do mesmo cenário (3 reps concordam). Poderia ser variabilidade *entre lotes* de
    experimento (MANA-noFT medido em batch diferente dos runs FT), mas não identificável.
  - Overhead assimétrico do MANA no startup: descartado — é efeito sobre P0, não P3.
  - Warm restart (TLB/páginas de memória): descartado — nó substituto tem TLB frio,
    o que tornaria P3 mais lento (não mais rápido); nós originais com TLB quente
    executariam no mesmo tempo, não mais rápido.
- **Decisão final:** reportar a anomalia com a tabela `tab:warm-restart-gap` e declarar
  que o mecanismo não é identificável com os dados disponíveis. Uma investigação mais
  aprofundada pode demandar instrumentação do coordinator MANA e profiling de hardware.
  Indicar como direção para trabalhos futuros (modalidade condicional, não certeza).
- **NÃO forçar hipótese** que não sobrevive ao escrutínio — honestidade acadêmica
  é preferível a uma explicação mecanística inventada.

**Tabela `tab:warm-restart-gap` (incluída em §6.4.2):**
Colunas: Benchmark, Workers, Timing, P3 esp.(s), P3 obs.(s), Gap%.
12 linhas: EP-D (4w/8w) × 3 timings + LU-C (4w/8w) × 3 timings.

| Benchmark | Workers | Timing | P3 esp. (s) | P3 obs. (s) | Gap% |
|---|---|---|---|---|---|
| EP-D | 4 | 10% | 217 | 152 | 30% |
| EP-D | 4 | 25% | 168 | 122 | 27% |
| EP-D | 4 | 50% | 98 | 70 | 29% |
| EP-D | 8 | 10% | 114 | 87 | 23% |
| EP-D | 8 | 25% | 95 | 75 | 21% |
| EP-D | 8 | 50% | 65 | 51 | 22% |
| LU-C | 4 | 10% | 122 | 81 | 34% |
| LU-C | 4 | 25% | 102 | 68 | 33% |
| LU-C | 4 | 50% | 69 | 47 | 32% |
| LU-C | 8 | 10% | 66 | 53 | 20% |
| LU-C | 8 | 25% | 56 | 47 | 15% |
| LU-C | 8 | 50% | 39 | 37 | 5% |

Dados de referência — razão de overhead, EP-D (N=3) — em §6.4.2 (movido de §6.5.3):
| Workers | Estratégia | 10% | 25% | 50% |
|---|---|---|---|---|
| 2w | REPLACE | 91,1% | 78,6% | 59,6% |
| 2w | DEGRADED | 91,4% | 78,2% | 57,3% |
| 4w | REPLACE | 90,3% | 77,4% | 60,1% |
| 4w | DEGRADED | 89,1% | 73,2% | 49,4% |
| 8w | REPLACE | 95,5% | 88,5% | 78,3% |
| 8w | DEGRADED | 93,3% | 83,0% | 66,7% |

---

#### §6.5 Comparação de Estratégias: Replace vs. Degraded ✅ ESCRITO (2026-06-26)

**Estrutura final (3 subseções):**

```
§6.5 Comparação de Estratégias
  §6.5.1 Tempo Total de Execução          ← fig8 + tab:strategy-wall-time  ✅ ESCRITO
  §6.5.2 Análise de Crossover             ← fig10 (heatmap) + texto        ✅ ESCRITO
  §6.5.3 Razão de Overhead de Recuperação ← fig9 + tab:overhead-ratio      ✅ ESCRITO
```

**Decisão estrutural:** §6.5.3 é métrica comparativa entre estratégias (overhead ratio
mostra DEGRADED consistentemente menor que REPLACE) — pertence a §6.5, não a §6.4.
Isso foi alterado do plano original que previa §6.4.4 para essa métrica.

---

**✅ §6.5.1 Tempo Total de Execução — ESCRITO**

Figura: `fig8_strategy_comparison.png`
Tabela: `tab:strategy-wall-time` (18 linhas, EP-D e LU-C × 3 workers × 3 timings)

Dados de referência (Δ = T_Replace − T_Degraded, sinal positivo = DEGRADED mais rápida):
| Benchmark | 2w 10% | 2w 25% | 2w 50% | 4w médio | 8w médio |
|---|---|---|---|---|---|
| EP-D | −19 s (REPLACE) | +11 s | +32 s | +36–79 s | +88–98 s |
| LU-C | +25 s | +38 s | +67 s | +73–82 s | +95–111 s |

**Estrutura do texto (4 parágrafos + fig8 + tab):**
1. P2b e P3 como únicos diferenciadores; P1, P2a, P2c idênticos entre estratégias.
2. Resultado da coluna Δ: 17/18 favoráveis ao DEGRADED; vantagem cresce com N.
   Mecanismo: penalidade P3 = 1/N da capacidade, que diminui para N grande;
   economia P2b permanece ~109 s.
3. Único caso REPLACE (EP-D 2w 10%): framing estrutural (não anomalia) — tendência
   do Δ decresce quando cluster é menor e falha é mais precoce; forward para §6.5.2.
4. Ponte para §6.5.2: "a tendência da coluna Δ revela uma estrutura geral..."

**Decisão de linguagem:**
- "P2b gap" → "economia de P2b" ou "diferença de P2b" (não usar "gap" no corpo do texto)
- Δ < 0 é EP-D 2w 10%; frasear como "único caso em que REPLACE termina antes" sem
  chamar de "anomalia" — o crossover é estrutural, não acidental.

---

**✅ §6.5.2 Análise de Crossover — ESCRITO**

**Figura: `fig10_crossover.png`** — adicionada nesta sessão; nova figura não prevista no plano original.

**Design do heatmap (decisão final após 3 iterações de design):**
- 2 painéis lado a lado: EP-D (esquerda) e LU-C (direita)
- X = workers (2w, 4w, 8w); Y = timing (10%, 25%, 50%, de cima para baixo = falha precoce → tardia)
- Cor = Δ = T_Replace − T_Degraded; escala divergente `RdYlGn` com `TwoSlopeNorm(vcenter=0)`
- Sem colorbar numérica; legenda de patch abaixo dos painéis: verde = DEGRADED mais rápida / vermelho = REPLACE mais rápida
- Borda vermelha escura (`#7a1a1a`, linewidth=2.5) na célula onde Δ < 0 (EP-D 2w 10%)
- Anotações numéricas em cada célula: `"+NN s"` com cor branca quando |val| > 55 s
- `figsize=(8.5, 3.2)` para caber sem cortar a legenda; `bbox_to_anchor=(0.45, -0.12)`

**Iterações de design descartadas:**
1. Scatter Δ vs P3_Rep com linhas verticais de threshold por N: confuso — grupos N misturados
2. Decision space N × P3_Rep com curva de fronteira: visualmente não deixava clara a tendência
3. Heatmap (adotado): mostra diretamente todos os dados e o gradiente de tendência

**Estrutura do texto (4 parágrafos):**
1. Mecanismo (P2b vs P3); dois fatores: N e timing; fórmula qualitativa.
2. Apresentação do heatmap.
3. Análise dos dois padrões: (a) N como fator dominante — gradiente esquerda→direita;
   (b) timing dentro de cada coluna — EP-D cruza o ponto de inversão em 2w/10%.
   EP-D vs LU-C explicados pela diferença de duração de execução (P3 maior = mais perto do crossover).
4. Projeção para múltiplas falhas e jobs longos → bridge para §6.5.3.

**Decisão de linguagem:**
- "Determina os limiares" → "analisa essa tendência estruturalmente" (§6.5.1 → §6.5.2 bridge)
  Motivo: §6.5.2 não deriva fórmulas explícitas de threshold; mostra a tendência visualmente.

---

**✅ §6.5.3 Razão de Overhead de Recuperação — ESCRITO e REVISADO (2026-06-26)**

Métrica: $(T_{\text{total}} - P0) / T_{\text{total}}$ — fração do tempo total que NÃO
foi computação pré-falha preservada pelo checkpoint.

**Estrutura do texto (4 parágrafos + fig9 + tab:overhead-ratio):**
1. Bridge explícito de §6.5.2: seção anterior mostrou Δ absoluto; esta seção normaliza
   pelo total — perspectiva complementar (eficiência relativa vs tempo absoluto).
2. Convergência das estratégias a 10% de timing (89–96% para ambas): mecanismo é o
   mesmo do heatmap — P3 grande domina, P2b ~109 s representa fração pequena.
   Conexão explícita: "as células de 10% no mapa de calor apresentam os menores Δ de cada linha."
3. Divergência a 25%/50%: P3 encolhe, ~109 s de P2b representa fração crescente;
   diferença entre estratégias se abre progressivamente. Espelha gradiente do heatmap.
4. Caso patológico REPLACE 8w: razão permanece > 78% mesmo em 50% porque P2b (~128 s)
   supera o noFT do job (~99 s) — estruturalmente alto. Bridge para §6.6 (custo).

**Erros corrigidos nesta revisão:**
- "MANA-noFT de ~99 s" → "noFT de ~99 s" (99 s é o tempo nativo sem MANA; MANA-noFT é ~126 s)
- "tempo total do job sem falhas" → "tempo total do job sem instrumentação" (mais preciso)

**"Gap%" substituído por $\Delta_{\%}$ (decisão 2026-06-26):**
- `Gap\%` era informal e anglófono; substituído por `$\Delta_{\%}$` na header da tabela
  e na fórmula da caption, e por "redução relativa" na prosa.

**Decisão de manter a seção:**
Avaliado se §6.5.3 agrega o suficiente para justificar sua existência. Conclusão:
a seção mantém dois insights genuinamente novos que §6.5.1/6.5.2 não cobrem:
(1) convergência das estratégias a 10% de timing (ambas ~90%) como confirmação normalizada
do padrão do heatmap; (2) caso patológico estrutural do REPLACE 8w (P2b > noFT).
Não é redundante — é ângulo complementar com dados novos. Mantida.

---

---

#### Decisões de escrita globais do Cap. 6 (consolidado desta sessão)

**Regras de linguagem (acrescentadas/confirmadas):**

| Proibido | Usar em vez disso |
|---|---|
| travessão — e en-dash — em prosa | vírgula, parênteses, dois-pontos |
| "Isso é contraintuitivo." | "À primeira vista, esse resultado pode parecer contraintuitivo" |
| "gap" (em qualquer contexto de prosa) | "discrepância", "diferença residual", "distância" |
| "Gap%" em tabelas e captions | `$\Delta_{\%}$` na notação matemática; "redução relativa" na prosa |
| Dados futuros sem forward ref | "Como se verificará na Seção~\ref{...}" |
| Generalizar além do subconjunto visto | Restringir: "nas três configurações de cluster do CG-C" |
| Hipótese sem evidência suficiente | Reportar anomalia + future work; NÃO inventar mecanismo |
| Afirmações de mecanismo incerto como certeza | Modal condicional: "pode demandar", "pode ser" |
| Over-promise em bridges de seção | Descrever o que a seção realmente faz (ex.: "analisa essa tendência estruturalmente", não "determina os limiares") |
| Misturar noFT e MANA-noFT nos valores | noFT = tempo nativo sem MANA; MANA-noFT = tempo com MANA mas sem falhas; nunca trocar |

**Padrão de fechamento de seção:**
Todo parágrafo final de subseção deve conectar o achado à seção seguinte (bridge).
Exemplo de §6.5.3: "a vantagem reflete P2b mais curto → §6.6 quantifica em custo".

---

**✅ Revisão de consistência do Cap. 6 inteiro (2026-06-26):**

Análise do capítulo como um todo. Três correções aplicadas:

1. **Referência adiantada em §6.2.1** (linha ~1795): o parágrafo final citava números
   dos estudos sintéticos ("4 a 5 s") antes de §6.3 apresentá-los. Corrigido para
   futuro: "o que os estudos sintéticos da Seção~\ref{sec:sinteticos} mostrarão".

2. **Over-promise em §6.5.1** (ponte para §6.5.2): "determina os limiares" → "analisa
   essa tendência estruturalmente" (§6.5.2 não deriva fórmulas explícitas de threshold).

3. **Erro factual em §6.5.3**: "MANA-noFT de ~99 s" corrigido para "noFT de ~99 s"
   (EP-D 8w: noFT = 99 s, MANA-noFT = 126 s; ambos menores que P2b ~128 s, então o
   argumento estrutural permanece válido; só o rótulo estava errado).

**Avaliação de leiturabilidade:** o capítulo é denso mas não repetitivo — cada seção
avança o argumento (não repete conclusões anteriores). O risco de fadiga vem do volume
de figuras+tabelas, que o §6.7 Discussão alivia ao sintetizar. Fluxo:
Config → overhead sem falhas → por que existe (sintéticos) → fases de recuperação →
qual estratégia vence (3 ângulos) → custo → discussão. Progressão lógica confirmada.

**Padrão para anomalias sem hipótese confirmada:**
1. Apresentar a observação com tabela de suporte (tab:warm-restart-gap).
2. Declarar que o mecanismo não é identificável com os dados disponíveis.
3. Sugerir (modal condicional) o que uma investigação futura poderia envolver.
4. Não fechar o parágrafo com hipótese não testada como se fosse explicação.

---

#### Decisões do analyze.py — plots (esta sessão, 2026-06-25)

**Legendas movidas para o rodapé (lower center):**
- Problema: `loc="upper right"` com `bbox_to_anchor=(0.99, 0.99)` cobria barras nos
  subgráficos do canto superior direito em fig4, fig5, fig8, fig9.
- Solução: `loc="lower center", bbox_to_anchor=(0.5, 0.01), ncol=5` (fig4/fig5),
  `ncol=3` (fig8), `ncol=2` (fig9).
- `tight_layout(rect=[...])` bottom reduzido para aproximar legenda dos gráficos:
  fig4 → `rect=[0, 0.05, 1.0, 1.0]`; fig5 → `rect=[0, 0.07, 1.0, 1.0]`;
  fig8 → `rect=[0, 0.04, 1.0, 1.0]`; fig9 → `rect=[0, 0.04, 1.0, 1.0]`.
- Labels de fase simplificadas de multiline para single-line:
  "Slurm + MANA overhead\n(drain/cancel...)" → "P2a+2c — Slurm + MANA".

**Títulos acima dos subgráficos (set_title em vez de ax.text interno):**
- `ax.text(0.03, 0.97, ...)` com `bbox` dentro dos eixos → `ax.set_title(...)` acima.
- Aplicado em: fig4, fig5 (única panel, título "CG-C"), fig7, fig8, fig9.
- fig2b ("synth_imbalanced") e fig3 ("synth_ckpt") não tinham título — adicionados.

**Fonte uniforme em todos os títulos:**
- Problema: alguns `set_title()` tinham `fontsize=12`, outros `fontsize=14`, outros
  sem argumento (usavam rcParams padrão de 15).
- Solução: remover todos os parâmetros `fontsize` e `fontweight` explícitos de cada
  `set_title()`. Todos herdam `axes.titlesize: 15` do rcParams configurado em `main()`.

**Comando para regenerar plots:**
```bash
python3 TCC/analyze.py --results-dir results --output-dir TCC/artifacts/phase6/lapesd-thesis/data
```
(Executar de `/home/artur/hpcac-toolkit`. Os plots chegam em `data/plots/` e ficam
acessíveis via symlink `imgs/plots/ → ../data/plots/`.)

---

#### §6.6 Análise de Custo ✅ ESCRITO (2026-06-26)

**Estrutura final:**
- **P1 (bridge + pergunta):** §6.5 mostrou vantagem temporal; §6.6 pergunta se isso se
  traduz em vantagem financeira. Tom de "faz sentido pagar spot + overhead para sair
  mais barato que on-demand sem FT?"
- **P2 (modelo):** equação LaTeX `\label{eq:cost}` exibida separada do texto.
  Fórmula: `custo = (T/3600) × (p_workers × N + p_head)`. O que MUDA entre noFT e FT
  é apenas `p_workers` ($0,192/h on-demand vs $0,0585/h spot). Desconto ~70%.
- **Fig7 + Tab11:** fig7 em grade 2×3 (linha = estratégia, coluna = benchmark).
  Tabela: 7 colunas com economy separada para REPLACE e DEGRADED.
- **P3 (CG-C):** resultado negativo era esperado — recovery > compute time (§6.2 já
  mostrou isso). Serve de ponto de partida para a questão "a partir de quando muda?"
- **P4 (tendência por duração):** ambas as estratégias melhoram com duração do job.
  DEGRADED evidencia melhor porque P2b ≈ 11 s é quase desprezível.
- **P5 (P2b penalty da REPLACE):** intervalo de 120 s sem cálculo, com instâncias
  faturadas. LU-C é o ponto onde DEGRADED já está no positivo e REPLACE ainda não.
  P2b é constante → EP-D 2w: economies convergem (30% vs 31%).
- **P6 (tamanho do cluster):** ambas têm custos similares em 2w e 4w, mas 8w aumenta
  mais. Exceção: DEGRADED EP-D 4w→8w custo mal sobe (+6%) mas economy melhora (34→38%)
  — explicação "plausível" (não assertiva) via Eq. cost: workers on-demand encarecem
  mais rápido que spot ao adicionar N.

**Decisões de linguagem:**
- Fórmula em ambiente `\begin{equation}` separado do texto corrido.
- Sem referência explícita de qual seção mostra a constância de P1 para EP-D/LU-C
  (nenhuma seção específica mostra isso diretamente).
- Tom "aparenta ser" / "explicação plausível" para o efeito de N no DEGRADED EP-D
  (não confirmado por dados controlados).
- Última frase da seção removida (era solta): bridge para §6.7 feita implicitamente.
- Tabela: legenda menciona economia positiva = savings, negativa = acréscimo.
- CG coluna Eco.(%): valores negativos grandes (−112% a −465%) aparecem como `$-$NNN`.

**Modelo de custo real (verificado com CSVs):**
`custo = (T/3600) × (p_workers × N + p_head)` onde:
- `p_workers = $0,0585/h` (spot, FT) ou `$0,1920/h` (on-demand, noFT)
- `p_head = $0,0832/h` (t3.large, sempre on-demand)
- `T` = ft_wall_time_s (tempo total da execução FT)

**Decisão figura fig7:**
- Títulos dos subgráficos incluem a estratégia: "Replace — CG-C", "Degraded — EP-D".
- Legendas padronizadas em `loc="lower center"` para todos os 6 painéis.
- Output correto: `python3 TCC/analyze.py --results-dir /home/artur/hpcac-toolkit/results
  --output-dir TCC/artifacts/phase6/lapesd-thesis/imgs`
  (análise.py salva em `out_dir/plots/`; body.tex busca em `imgs/plots/`).

---

#### Revisão do Capítulo 6 — 2026-06-26

Revisão completa de consistência, fluxo lógico e linguagem. Issues encontradas e ações:

| Issue | Linha(s) | Ação |
|---|---|---|
| Redundância: "específico ao CG" dito 2× + ref. errada a `sec:timing-sensitivity` | 2172–2178 | Removida a ref. a timing-sensitivity (nenhuma seção mostra P1 de EP-D/LU-C vs N diretamente); eliminada repetição de "específico ao CG" no segundo enunciado |
| Transição abrupta §6.3.1 → §6.3.2 (CG exclusão não anunciada) | 2206–2208 | Bridge adicionado ao fim de §6.3.1: "A subseção a seguir amplia a análise para benchmarks de maior duração..."; abertura de §6.3.2 simplificada ("excluído dessa análise" em vez de "excluído dos experimentos com variação") |
| Labels LaTeX com "gap" (sec:gap-overhead) | interno | NÃO corrigido — labels são internos ao LaTeX, invisíveis ao leitor; alterar quebraria todas as `\ref{}` |
| Legenda tab:cg-breakdown — P0/P2a+P2c não aparecem como colunas | 2146 | MANTIDA como estava (breve, lembra que são constantes; adicionar nova coluna seria over-engineering) |

**Resultado:** compilação limpa após todas as correções. Nenhuma ref. quebrada detectada.

---

#### §6.7 Discussão ✅ ESCRITO (2026-06-26)

**Estrutura final — 3 parágrafos:**

**P1 (intro suave + mecanismo estrutural):**
- Abre com "Ao longo deste capítulo, a decomposição em fases P0–P3 revelou..." para
  situar o leitor antes de sintetizar.
- Define brevemente P2b (reconfiguração do nó) e P3 (computação pós-reinício) como
  relembrada rápida — leitor já percorreu o capítulo inteiro.
- Explica a oposição estrutural: Degraded ganha ~109 s em P2b mas paga penalidade
  em P3 (N-1 workers); Replace paga ~120 s em P2b mas mantém capacidade integral em P3.
- Dois fatores determinam o vencedor: N e volume de trabalho no momento da falha.
- Em clusters maiores: perda de 1 nó = fração decrescente da capacidade → Degraded domina.
- Em clusters pequenos com falha precoce: P3 penalty pode superar economia de P2b →
  Replace vence (único caso: EP-D 2w, 10%).
- Encerra com referência ao mapa de calor da sec:crossover.

**P2 (limiar de viabilidade + achado complementar):**
- "Esse mecanismo de compensação pressupõe que a duração do job seja suficiente..."
- CG-C curto: overhead = 3–17× tempo útil; desconto spot não compensa nem em tempo
  nem em custo.
- EP-D/LU-C moderados: Degraded economiza 12–38%; tempo e custo apontam na mesma
  direção (análises convergentes, não necessitam de avaliação separada).
- Achado complementar: P3 da Replace sistematicamente 20–34% mais curto que o esperado
  (warm restart em cluster íntegro) — não completamente explicado pelos dados disponíveis.

**P3 (limitações do escopo):**
- Falha única apenas: Degraded degrada progressivamente (N→N-1→N-2) com falhas
  múltiplas; Replace recompõe e mantém capacidade → Replace mais robusta nesse regime.
- Intervalo 12–500 s: regime de jobs longos discutido analiticamente, não validado
  empiricamente.
- Específico a m5.xlarge + Amazon EFS: P1 e P2b deslocariam com outra infraestrutura.

**Decisões de linguagem:**
- NÃO dizer que P0 é fixo entre momentos de falha (P0 É o tempo pré-falha, varia
  por definição).
- NÃO dizer que P2b é o único diferenciador — P3 capacity (N vs N-1) também difere.
- NÃO dizer que o momento da falha tem pouca influência — §6.3.2 inteiro mostra que
  tem influência (crossover EP-D 2w existe exatamente por isso).
- Tom da seção: síntese estrutural, não repetição de resultados pontuais; leitura
  leve para leitor que já percorreu o capítulo.
- Comprimento: 3 parágrafos (não 4–5) — seção de fechamento, não de análise nova.

---
### 6.6 Step 10 — Chapter 7: Conclusão

**Estimated length:** 3–4 pages.

#### §7.1 Considerações Finais

- Restate the problem (spot interruptions destroy MPI progress) and the solution
  (MANA + HPC@Cloud + hybrid topology).
- Confirm each objective was met (list §1.2.2 objectives and check against results).
- Summarize three key contributions:
  1. Transparent fault tolerance for unmodified MPI apps on AWS spot.
  2. Two recovery strategies with characterization of conditions for each.
  3. Economic analysis showing when spot + FT is cheaper than on-demand.

#### §7.2 Trabalhos Futuros

From TCC_MANA_INTEGRATION_PLAN.md §9 Remaining Work:
1. ≥3 repetitions per point for statistical confidence (standard deviation on all metrics).
2. LU Class D and EP Class E for longer jobs — observe if REPLACE crossover is
   empirically reached.
3. Confirm CG Class C MANA overhead at 2 workers via per-rank profiling.
4. Multi-failure scenarios — DEGRADED capacity degrades with each successive failure.
5. Predictive checkpointing using spot interruption ML prediction.
6. Integration with AWS Spot Fleet / Auto Scaling Groups.

---

### 6.7 Step 11 — Complete prolog.tex

Do this step LAST — after all chapters are written — so the resumo estendido can
summarize accurately.

#### Agradecimentos

Thank: Prof. Márcio Bastos Castro (advisor), LaPeSD lab colleagues, family.
Mention: PIBIC/CNPq support (if applicable), AWS credits.
Length: 1 paragraph.

#### Resumo Estendido (2–5 pages, mandatory by BU/RN 95)

Fill in the five mandatory sections in `prolog.tex`:

| Section | What to write | Source |
|---|---|---|
| Introdução | Context (spot + MPI + C/R), problem, hypothesis (MANA + HPC@Cloud) | Ch. 1 summary |
| Objetivos | The 6 specific objectives from §1.2.2 | Ch. 1 |
| Metodologia | Hybrid topology, NPB benchmarks, 94-run matrix, auto_test_failure | Ch. 5 + §6.1 |
| Resultados e Discussão | Two economic regimes, crossover formula, MANA overhead range | §6.5–6.6 |
| Considerações Finais | Objectives met, 3 contributions, future work | Ch. 7 |

---

## 7. Step 12 — Architecture and Flow Diagrams

The following diagrams need to be created and placed in `imgs/arch/`.
Recommended tool: draw.io (export as PDF or PNG), or TikZ directly in LaTeX.

| Figure | Content | Used in |
|---|---|---|
| Fig 5.1 — System Architecture | Head node (t3.large, Slurm controller, MANA coordinator) + spot workers + EFS | §5.1 |
| Fig 5.2 — Watcher Flow | Flowchart: job starts → watcher monitors → interruption detected → checkpoint → drain → strategy dispatch | §5.4–5.5 |
| Fig 5.3 — Recovery Strategies | Two-branch diagram: REPLACE (respawn + deploy + reconfigure) vs DEGRADED (DOWN + restart N-1) | §5.6 |
| Fig 5.4 — Bootstrap Sequence | Timeline: HPC@Cloud spawn → SSM init → Slurm config → MANA deploy → job ready | §5.3 |

**TikZ alternative:** If no drawing tool is available, these diagrams can be
written directly in LaTeX using the `tikz` package. The lapesd-thesis.cls does
not include tikz by default — add `\usepackage{tikz}` to `main.tex`.

---

## 8. Step 13 — LaTeX Tables from CSVs

Convert the key CSV files from `lapesd-thesis/data/` into LaTeX `tabular` environments.

**Table mapping:**

| LaTeX table | CSV source | Chapter |
|---|---|---|
| Tab 6.1 — MANA overhead | `table01_mana_overhead.csv` | §6.2.1 |
| Tab 6.2 — Synth calls (coletivas + p2p) | `table03_synth_calls.csv` | §6.3.1 |
| Tab 6.3 — Synth imbalanced | `table04_synth_imbalanced.csv` | §6.3.2 |
| Tab 6.4 — Checkpoint size | `table05_synth_ckpt.csv` | §6.3.3 |
| Tab 6.5 — CG phase breakdown | `table06_cg_ft_breakdown.csv` | §6.4.1 |
| Tab 6.6 — Recovery overhead ratio (EP-D) | `table10_recovery_overhead.csv` | §6.4.2 |
| Tab 6.7 — Strategy winner | `table09_strategy_winner.csv` | §6.5.1 |
| Tab 6.8 — Warm restart Gap% | derivada de tables 07/08 | §6.5.3 |
| Tab 6.9 — Cost per run | `table11_cost.csv` | §6.6 |

**LaTeX table format:** use `booktabs` (`\toprule`, `\midrule`, `\bottomrule`)
which is already loaded by lapesd-thesis.cls. Numeric columns should be right-aligned.
Use `\fonte{o autor.}` after each table.

---

## 9. Validation Gates (Phase 6 Acceptance)

- [ ] G1: `latexmk -pdf -shell-escape main.tex` produces a clean PDF with no errors.
- [ ] G2: All citations in body.tex are defined in main.bib and have no `% TODO` flags.
- [ ] G3: No `\cite{???}` or undefined reference warnings in the LaTeX log.
- [ ] G4: All figures referenced with `\autoref` exist in `imgs/`.
- [ ] G5: All chapters 2–7 have no `% TODO` lines remaining.
- [ ] G6: prolog.tex has no `% TODO` lines (agradecimentos and resumo estendido complete).
- [ ] G7: acronyms.tex covers all acronyms used in the document.
- [ ] G8: Table of contents, list of figures, and list of tables render correctly.
- [ ] G9: Document compiles to ≥40 pages (rough target for a complete TCC1).
- [ ] G10: Advisor has reviewed and approved for submission.

---

## 10. Deliverables

| Deliverable | Location | Status |
|---|---|---|
| Fixed LaTeX build | `lapesd-thesis/` — clean `latexmk` run | ⬜ |
| All phase 5 plots copied | `lapesd-thesis/imgs/plots/` | ⬜ |
| CSV tables copied | `lapesd-thesis/data/` | ⬜ |
| main.bib — all entries verified and complete | `lapesd-thesis/main.bib` | ⬜ |
| acronyms.tex — all TCC acronyms | `lapesd-thesis/acronyms.tex` | ⬜ |
| body.tex — Ch. 1 | `lapesd-thesis/body.tex` | ✅ Done |
| body.tex — Ch. 2 | `lapesd-thesis/body.tex` | ⬜ |
| body.tex — Ch. 3 | `lapesd-thesis/body.tex` | ⬜ |
| body.tex — Ch. 4 | `lapesd-thesis/body.tex` | ✅ Done |
| body.tex — Ch. 5 | `lapesd-thesis/body.tex` | ⬜ |
| body.tex — Ch. 6 | `lapesd-thesis/body.tex` | ⬜ |
| body.tex — Ch. 7 | `lapesd-thesis/body.tex` | ⬜ |
| prolog.tex — Agradecimentos | `lapesd-thesis/prolog.tex` | ⬜ |
| prolog.tex — Resumo (PT) | `lapesd-thesis/prolog.tex` | ✅ Done |
| prolog.tex — Resumo Estendido | `lapesd-thesis/prolog.tex` | ⬜ |
| prolog.tex — Abstract (EN) | `lapesd-thesis/prolog.tex` | ✅ Done |
| Architecture diagrams (4 figures) | `lapesd-thesis/imgs/arch/` | ⬜ |
| LaTeX tables (6 tables) | inline in `body.tex` | ⬜ |
| PHASE6_ARTIFACT.md | `TCC/artifacts/phase6/` | ⬜ |
| Final compiled PDF | `lapesd-thesis/main.pdf` | ⬜ |
