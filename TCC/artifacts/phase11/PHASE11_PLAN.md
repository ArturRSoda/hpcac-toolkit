# Phase 11 Plan — Consolidação do ADAPTIVE na Monografia

Date: 2026-10-06
Projeto: Sustainable High Performance Computing on AWS
Bolsista: Artur Luiz Rizzato Toru Soda
Orientador: Prof. Dr. Márcio Bastos Castro
Instituição: LaPeSD, INE/UFSC

---

## Objetivo

Incorporar os achados da Fase 10 (estratégia `ADAPTIVE`, validação de
N-independência, testes de integração, e a exploração de calibração
automática do coeficiente na Etapa 13) à versão já escrita da monografia
(`TCC/artifacts/phase6/lapesd-thesis/body.tex` e `prolog.tex`), preservando o
estilo de escrita original, o fluxo lógico entre seções, a legibilidade, e
sem introduzir nenhuma inconsistência com o texto já existente.

Esta fase é de escrita, não de experimentação — nenhum dado novo é coletado.
O trabalho é inteiramente sobre como editar um documento de 2873 linhas já
defendido (banca anterior) sem degradá-lo.

---

## Decisão de estrutura: onde o conteúdo novo entra

Análise feita sobre o sumário completo de `body.tex` (grep de
`\chapter`/`\section`/`\subsection`) e leitura das seções candidatas antes de
decidir.

**Decisão: não cria capítulo novo.** O conteúdo do ADAPTIVE se divide
naturalmente em dois tipos — mecanismo (arquitetura/implementação) e
validação experimental — e a monografia já tem um capítulo dedicado a cada
tipo. Um capítulo novo duplicaria essa separação e quebraria a organização
que o leitor (banca) já viu antes.

1. **Capítulo 5 (`Integração de Tolerância a Falhas com MANA no HPC@Cloud`),
   Seção `Estratégias de Recuperação` (`sec:estrategias`, linha 1432).**
   Hoje essa seção descreve `REPLACE` e `DEGRADED` como "os dois caminhos de
   recuperação configurados pelo usuário" (linha 1436), um parágrafo cada,
   sem subseções. O `ADAPTIVE` é um terceiro caminho, de mesma natureza
   (mecanismo de decisão do watcher) — entra aqui como conteúdo novo (ver
   Etapa 3), não como seção nova. A frase de abertura da seção precisa
   mudar de "um dos dois caminhos" para refletir três.

2. **Capítulo 6 (`Avaliação Experimental`), nova seção entre `Análise de
   Custo` (`sec:custo`, termina ~2646) e `Discussão` (`sec:discussao`, 2648).**
   Não entra como subseção dentro de `Comparação de Estratégias`
   (`sec:crossover`, 2293) porque essa seção termina com uma referência
   direta para `Análise de Custo` (linha 2526: "A Seção~\ref{sec:custo}
   quantifica essa implicação...") que seria quebrada por uma inserção no
   meio. Entrando depois de `Análise de Custo`, a nova seção fecha o
   capítulo logicamente: primeiro o leitor vê que `REPLACE`/`DEGRADED` têm
   um ponto de troca (crossover) em tempo e em custo, só então vê como o
   `ADAPTIVE` automatiza essa escolha. Como bônus, `Discussão` (2648) e
   `Ameaças à Validade` (2699), que já sintetizam o capítulo todo, passam a
   sintetizar o `ADAPTIVE` junto, sem precisar de uma segunda rodada de
   discussão/ameaças dedicada só a ele.

   Subseções propostas (espelhando a estrutura do
   `analysis_report_phase10_ptbr.md`, adaptadas ao tom da monografia):
   - Derivação da heurística empírica (coeficiente a partir dos 18 pontos
     NPB, Tabela correspondente)
   - Validação de independência de N (`synth_duration`, 84 execuções)
   - Implementação (breve — a mecânica detalhada já está no Capítulo 5;
     aqui só a ligação com os dados)
   - Testes de integração (escopo reduzido da Etapa 12)
   - Limitação identificada: o coeficiente não é universal entre cargas de
     trabalho (demonstração `synth_duration` f=50%/N=2)
   - Exploração de calibração automática do coeficiente (Etapa 13, resumida
     — ver Etapa 6 abaixo sobre como tratar isso sem virar uma segunda
     monografia dentro da seção)

3. **Capítulo 1 (`Introdução`).** `Objetivos Específicos` (linha 87) e
   `Contribuições` (linha 112) listam hoje duas estratégias (`REPLACE`,
   `DEGRADED`); decidir, na Etapa 6, se o `ADAPTIVE` justifica um item novo
   em cada lista ou uma extensão do item existente. `Organização do
   Trabalho` (142) precisa de uma frase indicando a nova seção do Capítulo 6.

4. **Capítulo 7 (`Conclusão`).** `Considerações Finais` (2740) ganha uma
   síntese curta do `ADAPTIVE` na mesma voz do restante da seção.
   `Trabalhos Futuros` (2817) ganha a calibração automática (Etapa 13) como
   extensão futura — e precisa de uma revisão pontual (ver Etapa 5, item 4).

---

## Pontos de atualização textual já identificados (correções, não conteúdo novo)

Levantados por grep direto no `body.tex` atual durante o planejamento desta
fase. Tratados como checklist de correções independente do conteúdo novo —
aplicar tudo na Etapa 5, de uma vez, para não editar o documento em
pedaços.

1. **Tabela 16 (`tab:warm-restart-gap`):** faltam as 6 linhas de N=2 (EP-D e
   LU-C, f=10/25/50%). Já calculadas a partir de `results_raw.csv` em
   `PHASE10_PLAN.md` ("Etapa 11.1 cancelada: Tabela 16 completada com dados
   já existentes") — não precisa recalcular, só inserir.

2. **"entre 5 e 34% para o LU-C" → "entre 5 e 37%".** Com N=2 incluído,
   LU-C f=25% chega a 37,0% em 2 das 3 repetições, excedendo o limite hoje
   declarado. Ocorre em **três** lugares (confirmado por grep nesta fase de
   planejamento, não só o local já catalogado na Fase 10):
   - linha ~2254 (Seção 6.4.2, `Sensibilidade ao Momento da Falha`)
   - linha ~2680 (Seção `Discussão`, "o P3 observado ficou entre 5 e 34%
     abaixo do valor esperado")
   - linha ~2862 (Seção `Trabalhos Futuros`, "entre 5 e 34% mais rápida")
   Re-grep obrigatório na Etapa 5 antes de fechar o item, para garantir que
   nenhuma quarta ocorrência foi perdida.

3. **Explicação mecanicista "fração de capacidade" (modelo 1/(N−1)).**
   **[RESOLVIDO em 2026-10-12, ver achados 1c/13/14-18 no índice da Etapa 1
   abaixo — mantido aqui só como histórico da análise original.]**
   Confirmado em **quatro** lugares, não três — o checklist original da
   Fase 10 só catalogava um, o levantamento de planejamento achou mais
   dois, e a leitura real da Etapa 1 achou um quarto que **todos os
   greps anteriores perderam** (mesmo padrão do bug de case-sensitivity
   do achado 1b — grep nunca é garantia de cobertura completa, só reduz o
   risco):
   - Seção `Tempo Total de Execução` (6.5.1) — a ocorrência que escapou de
     todas as buscas anteriores, encontrada só na leitura direta do PDF
   - Seção `Análise de Crossover` (6.5.2) — já catalogado na Fase 10
   - Seção `Discussão` — já catalogado no planejamento
   - Seção `Decomposição Temporal`, sobre o CG-C — já catalogado no
     planejamento (ver achado 13, tratamento diferenciado)
   As três primeiras fazem uma afirmação **geral** sobre o mecanismo (Δ
   cresce com N porque perder um nó pesa menos em clusters maiores) para
   EP-D/LU-C — é essa generalização que a Fase 10 refina, porque o
   coeficiente real fica quase N-independente (0,345/0,316/0,292), e o
   crescimento de Δ vem majoritariamente de `remaining_secs` cair em
   clusters maiores (efeito de duração), não do coeficiente de penalidade
   mudar. Verificação numérica feita nesta sessão (EP-D, f=10%): de N=2
   para N=8, `remaining_secs` encolhe ~3,85×, `penalidade` encolhe ~4,17×
   (quase o mesmo fator), `coef` mal se move (~1,08×) — confirma que é
   efeito de duração, não encolhimento do coeficiente. As três ocorrências
   foram marcadas com `\todo` para virar menção breve + referência à nova
   seção do ADAPTIVE (Etapa 4), não uma reescrita completa triplicada. O
   quarto caso (CG-C, achado 13) é estruturalmente diferente — ver esse
   achado para o tratamento específico, que **não deve ser confundido**
   com o efeito de duração das outras três (verificação quantitativa:
   para o CG-C, `remaining_secs` encolhe só ~1,63× mas `penalidade`
   encolhe ~364× e `coef` colapsa ~223× — o efeito de duração explica
   quase nada aqui, é um colapso genuíno do coeficiente).

4. **Proposta de "contadores de hardware" como trabalho futuro — duas
   ocorrências, não uma:**
   - linha ~2257 (Seção `Sensibilidade ao Momento da Falha`: "profiling de
     hardware com contadores de eventos de processador, representando uma
     direção para trabalhos futuros")
   - linha ~2865 (Seção `Trabalhos Futuros`: "contadores de eventos de
     hardware poderiam revelar o mecanismo subjacente")
   Ambas propõem `perf stat`/contadores de hardware para explicar a
   aceleração de P3 na `REPLACE` — exatamente o que a Etapa 13 tentou e
   descartou com evidência direta (AWS Nitro não expõe PMU em instâncias
   não-`.metal`). Se não forem revisadas, o texto fica **contradizendo**
   um achado da própria monografia, nos dois lugares. Substituir por algo
   como: tentativa feita via `perf stat`, inviável por limitação da
   virtualização Nitro; a calibração direta por densidade de processos por
   núcleo (Etapa 13.2–13.4) é uma alternativa parcial, com discrepância
   ainda não resolvida entre o previsto e o medido em `synth_duration` —
   ver nova seção do Capítulo 6. Decidir se as duas ocorrências devem dizer
   a mesma coisa (uma delas pode virar só uma referência cruzada para a
   outra, para não repetir o mesmo parágrafo duas vezes no documento).

5. **Contagem agregada "282 execuções".** Ocorre em 7 lugares em `body.tex`
   (linhas 133, 155, 262, 1596, 1679, 2347¹, 2777) e mais 2 no `prolog.tex`
   (resumo curto linha ~48, abstract em inglês linha ~261) — não contar a
   linha 2347, que é um valor de tabela não relacionado (confirmar na
   Etapa 5). Todas as ocorrências qualificam "282" como execuções **sobre
   os benchmarks NPB** — o número está correto e não muda, porque os
   experimentos da Fase 10 usam `synth_duration` (sintético), não NPB.
   **Ação:** não alterar "282" em si; garantir que, em cada lugar onde "282"
   aparece como "o total de execuções do trabalho" (em vez de "o total de
   execuções sobre NPB"), o texto deixe claro que há execuções adicionais
   fora desse número — citando o total da Fase 10 (92 novas em AWS + 108
   reaproveitadas, ou o número final que a Etapa 4 abaixo consolidar) no
   lugar apropriado (provavelmente na introdução da nova seção do Capítulo
   6, e possivelmente em `Contribuições`).

6. **Resumo e Abstract curtos (`prolog.tex`, não o Resumo Estendido).**
   Mencionam as duas estratégias e o "282 execuções" — avaliar, na Etapa 7,
   se merecem uma frase sobre o `ADAPTIVE` ou se o escopo de um resumo curto
   não comporta. Não confundir com o Resumo Estendido (Etapa 9, abaixo,
   adiada).

7. **Figuras: quais mudam, quais são novas, quais ficam iguais.**
   Levantamento feito nesta fase de planejamento, abrindo os arquivos de
   imagem (não só as legendas) para decidir caso a caso:
   - `imgs/arch/watcher-flow.png` (`fig:watcher-flow`, Capítulo 5) —
     **[FEITO em 2026-10-18 pelo usuário]**. Caixa verde agora lê
     "REPLACE_RESUME || DEGRADED_RESUME (static config, or ADAPTIVE
     heuristic)" — mesma forma/posição/cor, sem diamante novo. Verificado
     direto no PDF compilado (`make pdf`), renderiza corretamente (a
     aparência de fundo preto num visualizador de PNG cru era só
     transparência, não afeta a composição na página branca do LaTeX).
     `\todo` correspondente removido de `body.tex`.
   - `imgs/arch/strategy-flow.png` (`fig:estrategias-flow`, Capítulo 5) —
     **[FEITO em 2026-10-18 pelo usuário]**. Em vez da segunda linha no
     rótulo do container (minha proposta original), o usuário adicionou um
     diamante de decisão "static config, or ADAPTIVE heuristic" logo após
     "Checkpoint Made (P1) and Failed Node Drained (P2a)", com exatamente
     dois ramos de saída (REPLACE/DEGRADED, iguais a antes) — abordagem
     diferente da que eu tinha sugerido, mas resultado melhor: usa a mesma
     convenção visual de decisão já estabelecida pelo diamante do
     `watcher-flow.png`, e como mantém só dois ramos de saída, não sugere
     um terceiro caminho de infraestrutura. Verificado direto no PDF
     compilado. `\todo` correspondente removido de `body.tex`.
   - `imgs/plots/fig8_strategy_comparison.png` e `fig10_crossover.png`
     (Capítulo 6) — **ficam iguais**. Reportam dados empíricos brutos de
     tempo de parede por estratégia, sem relação com a *seleção* de
     estratégia; a validação do ADAPTIVE é aditiva, não uma correção
     desses dados.
   - `TCC/artifacts/phase10/imgs/fig_synth_duration_boundary.png` e
     `fig_coefficient_n_independence.png` — verificado nesta fase que estão
     **essencialmente no mesmo estilo** das demais figuras do documento
     (inglês, matplotlib simples, paleta semelhante — checado diretamente
     contra `fig1_mana_overhead.png` e `fig6_mana_scalability.png`, não só
     por suposição). A única inconsistência real: todas as figuras de dados
     do TCC não têm título embutido na imagem, só eixos e legenda, com a
     descrição vivendo no `\caption{}` do LaTeX; as duas figuras da Fase 10
     têm um título de duas linhas embutido no PNG, em tom de depuração
     ("diamond = ranges overlap (genuinely ambiguous point)"). Correção
     pequena: remover o título embutido na imagem e mover essa descrição
     para o `\caption{}`, igual às demais.
   - `TCC/artifacts/phase10/imgs/fig_adaptive_boundary.png` — figura extra
     já gerada na Fase 10 (não usada nos relatórios finais), mostra os 18
     pontos NPB de treino com o limiar de 389s. Boa candidata para a
     subseção de "derivação da heurística" no Capítulo 6; mesma correção de
     título embutido antes de usar.
   - `body.tex` linha ~1368, trecho de configuração YAML em Capítulo 5
     (`strategy: "REPLACE_RESUME"  # ou DEGRADED_RESUME`) — não é uma
     figura, mas é um trecho de código ilustrativo que lista as estratégias
     disponíveis; precisa do comentário atualizado para incluir `ADAPTIVE`.

8. **Tabelas: o que é novo.** Além da Tabela 16 (item 1 acima), a nova
   seção do Capítulo 6 precisa de uma tabela de matriz de experimentos
   própria para o `synth_duration` (mesmo formato da Tabela~\ref{tab:matriz},
   Fator/Valores: benchmark=`synth_duration`, N=2/4/8, f%, total de
   execuções da Fase 10) — mantida **separada** da Tabela~\ref{tab:matriz}
   existente, não mesclada nela, consistente com a decisão do item 5 acima
   de manter "282" escopado ao NPB.

---

## Etapas

### Etapa 1 — Leitura completa do `body.tex` em partes

Ler o arquivo inteiro (2873 linhas) em blocos por capítulo, nesta ordem:
Introdução (1–159) → Fundamentação Teórica (162–836) → Trabalhos
Relacionados (838–1128) → Análise de Viabilidade de Burstable (1130–1245) →
Integração com MANA (1246–1583) → Avaliação Experimental (1584–2729) →
Conclusão (2730–2873). Objetivo duplo: (a) recuperar o contexto fino do
texto já escrito antes de editar nada; (b) procurar ativamente por
inconsistências não relacionadas ao `ADAPTIVE` (números que não batem entre
seções, referências cruzadas quebradas, terminologia inconsistente) —
registrar tudo encontrado, mesmo que fora do escopo da Fase 10, numa lista
separada para decidir com o usuário o que vale corrigir nesta fase.

Esta etapa é executada no início da Fase 11, não nesta etapa de
planejamento.

#### Cópia de trabalho compilável e marcadores inline (decisão de 2026-10-08)

A leitura por partes não usa só uma lista no plano — usamos uma **cópia de
trabalho completa e compilável** da monografia, em
`TCC/artifacts/phase11/lapesd-thesis/` (mesma estrutura de
`phase6/lapesd-thesis/`: `main.tex`, `body.tex`, `prolog.tex`, `epilog.tex`,
`acronyms.tex`, `glossary.tex`, `imgs/`, classe UFSC, `.bst`, `Makefile`
etc., sem os artefatos de build gerados nem os arquivos de análise da Fase 5
que não são referenciados por nenhum `.tex`). Testado com `make pdf` —
compila igual ao original, 102 páginas.

**O original em `phase6/lapesd-thesis/` não é mais tocado a partir de
agora — vira referência congelada da defesa anterior.** A cópia em
`phase11/lapesd-thesis/` é a que recebe todas as edições daqui pra frente,
inclusive as da Fase 11.

Achados de inconsistência são marcados **diretamente no texto, de forma
visível no PDF compilado** (não como comentário LaTeX invisível — pedido
explícito do usuário, 2026-10-08), usando o pacote `todonotes`
(`\usepackage[textwidth=3.2cm]{todonotes}` em `main.tex`). Cada achado vira
uma caixa amarela/laranja inline, logo acima da linha afetada:
```latex
\todo[inline,backgroundcolor=yellow!30,bordercolor=orange,linecolor=orange]{\textbf{[FASE11]}\ <achado curto, pode referenciar o PHASE11_PLAN.md para detalhe>}
```
**Não envolver `\todo` num `\newcommand` com `#1`** — tentamos isso
primeiro (`\newcommand{\fase11}[1]{\todo[...]{...#1...}}`) e quebrou a
compilação (`! LaTeX Error: Missing \begin{document}`, `Illegal parameter
number`) por conflito de parse entre o `#1` do wrapper e o parsing interno
de argumentos do `\todo`. Chamar `\todo[...]{...}` diretamente em cada
marcador evita o problema. Usar aspas LaTeX `` ` ` `` / `''` dentro do
texto do `\todo`, nunca aspas retas `"..."` — aspas retas dentro do
argumento do `\todo` perderam a aspa de fechamento na renderização (bug
observado e corrigido nesta sessão). Dentro de blocos `minted`, o marcador
vai *fora* do bloco, nunca dentro — um `\todo` dentro de um listing
`minted` apareceria literalmente na listagem renderizada.

Greppável com `grep -rn "\[FASE11\]" body.tex prolog.tex`. Nada é corrigido
de verdade enquanto a leitura não terminar (Etapa 5 aplica tudo de uma
vez); os marcadores só *apontam* o que precisa mudar. **O pacote
`todonotes` e todos os `\todo[...]{...}` são temporários** — remover ambos
antes de considerar a Fase 11 encerrada (ver Gate dedicado abaixo).

Este plano mantém um índice condensado por parte lida — detalhe completo
fica no marcador em si, dentro do arquivo.

**Parte 1 — Introdução (linhas 1–159 do `body.tex` original):**

| # | Marcador em | Achado (resumo) | Status |
|---|---|---|---|
| 1 | `body.tex` (4 ocorrências) | "Duas estratégias" precisa virar três com o ADAPTIVE — Objetivos Específicos, Contribuições, abertura do Capítulo 5, abertura de `sec:estrategias`. ~7 ocorrências equivalentes no Capítulo 6 (comparação empírica REPLACE/DEGRADED) foram conferidas e **não** precisam mudar. | marcado, correção na Etapa 5 |
| 1b | `body.tex` (1), `prolog.tex` (2) | **Mesmo achado, 3 ocorrências a mais** encontradas depois — o grep original do item 1 era *case-sensitive* e não pegava "Duas estratégias" com D maiúsculo no início de frase. Localizações: Introdução (`body.tex`, parágrafo da arquitetura híbrida, "Duas estratégias de recuperação são implementadas e avaliadas neste trabalho"), Resumo curto **não-estendido** (`prolog.tex`, mesma frase em PT-BR), Abstract curto em inglês (`prolog.tex`, "Two recovery strategies are implemented"). Nenhuma das três está dentro do Resumo Estendido — são conteúdo permanente, não sujeitas à decisão pendente do item 2. **Lição**: re-grep de termos como esse deve ser sempre `grep -i`, não case-sensitive — vale conferir se outros achados desta leitura têm o mesmo ponto cego antes de fechar a Etapa 1. | marcado, correção na Etapa 5 |
| 1c | `body.tex` (Capítulo 5, "Detecção de Interrupções e Checkpoint Reativo") | **Mais uma ocorrência, com nuance própria** — "o fluxo diverge de acordo com a estratégia de recuperação configurada, REPLACE ou DEGRADED" descreve o ponto exato do pipeline do \textit{watcher} onde `choose_effective_strategy()` roda de verdade para o ADAPTIVE (mesmo ponto já identificado na análise de `watcher-flow.png`, achado de figuras). Não é só trocar "duas" por "três": a frase precisa dizer que, com ADAPTIVE configurado, é justamente aqui que a escolha entre REPLACE/DEGRADED é resolvida antes do fluxo divergir — texto e figura devem ficar consistentes entre si (Etapa 3 cobre os dois). | marcado, correção na Etapa 3 (texto) + Etapa 3 (figura, já no checklist de `watcher-flow.png`) |
| 2 | `prolog.tex` (2 ocorrências) | Mesmo padrão "duas estratégias", mas dentro do Resumo Estendido (remoção adiada, Etapa 9). **Decisão pendente:** corrigir mesmo assim ou deixar para quando a seção for removida. | marcado, decisão pendente |
| 3 | `body.tex` (Introdução + Capítulo 5) | **Erro factual confirmado contra o código**, não terminologia: a Introdução dizia que instâncias spot substituídas "recebem novos endereços IP" — falso para esta arquitetura. Verificado em `src/integrations/providers/aws/interface.rs:148`, `elastic_network_interface.rs:18-49`, `resource_manager.rs:1117-1126`: cada worker tem IP privado fixo por slot via ENI persistente reanexada (não recriada) a cada respawn; só o head tem IP público (Elastic IP real). O trecho do Capítulo 5 (linha ~1472 do original) já estava certo e batia com o código. | **corrigido em 2026-10-10** — reescrito trocando a causa (endereço mudou) pela causa real (instância EC2 nova, sem estado de conexão TCP/MPI preservado); a mesma edição também reescreveu a transição para o parágrafo de instâncias burstable, que mudava de assunto sem introduzir o conceito de "processo coordenador" antes de precisar dele (achado adicional do usuário, não do grep) |
| 4 | `body.tex` (Contribuições, bullet 3) | **Não é erro** — "282 execuções sobre os NAS Parallel Benchmarks CG-C, EP-D e LU-C" já está corretamente escopado ao estudo NPB, não precisa mudar. O que fica em aberto é uma decisão de conteúdo (Etapa 6): Contribuições ganha um bullet novo para o ADAPTIVE (derivação da heurística + validação com `synth_duration`) ou estende um dos três já existentes (provavelmente o bullet 2, "duas estratégias", já marcado no item 1). | marcado, decisão de conteúdo na Etapa 6 |
| 5 | `body.tex` (Organização do Trabalho) | **Precisa de dois ajustes**, confirmados pelo usuário: (a) a lista "estudos de overhead, tamanho de checkpoint, sensibilidade ao momento da falha e comparação de estratégias" precisa citar a nova seção do ADAPTIVE (Etapa 4); (b) aqui "282 execuções" descreve o Capítulo 6 **inteiro**, não só o estudo NPB — diferente do bullet de Contribuições (item 4 acima, que já está escopado corretamente) — então esta ocorrência específica precisa deixar claro que há execuções adicionais da Fase 10 fora desse número. Mesmo achado do item 5 do checklist principal (seção "Pontos de atualização textual"), agora localizado com precisão. | marcado, correção na Etapa 5 |
| 6 | `body.tex` (Fundamentação Teórica, "Amazon Web Services", justificativa de escolha da AWS) | **Erro real, não só escopo** — "que viabilizou as 282 execuções experimentais" subestima o que a Chamada CNPq/AWS Nº 64/2022 realmente financiou: a mesma frase já cita, como primeiro motivo da escolha da AWS, os estudos de instâncias burstable (ERAD-RS 2025, WCC/SBAC-PAD 2025), que não fazem parte das 282; some-se a isso as execuções novas da Fase 10 (`synth_duration`, calibração). Diferente do item 5, aqui não dava para só "deixar claro que há mais" — o número específico estava simplesmente errado no contexto. | **corrigido em 2026-10-10** — número removido, frase generalizada para "viabilizou a execução dos experimentos computacionais deste trabalho", sem comprometer-se com uma contagem total que ficaria desatualizada a cada nova fase |
| 7 | `body.tex` (Fundamentação Teórica: NIST, split-process, "Técnicas de Checkpoint/Restart") | Três correções pontuais sem relação com o ADAPTIVE, encontradas pelo usuário lendo o PDF renderizado (não vieram de grep): (a) tabela do NIST usava "+" como atalho matemático informal para "inclui também" nas linhas PaaS/SaaS; (b) parágrafo do split-process do MANA repetia o mesmo erro do item 3 ("nova metade inferior assume o novo endereço IP"); (c) frase sobre o mecanismo de salvamento dizia "periódico e reativo" quando só o reativo está implementado (periódico é trabalho futuro, Seção Trabalhos Futuros); (d) parêntese "equivalente a 60% da capacidade total dos dois vCPUs" continha um erro aritmético (30%/vCPU × 2 vCPUs = 60% de **um** vCPU, não 60% do total de dois); (e) menção a "sistemas de arquivos paralelos como o Lustre" nunca mais referenciada no resto do documento. | **todas corrigidas em 2026-10-10** — tabela NIST: "+"→"Inclui ainda"; split-process: "endereço IP diferente"→"configuração de rede diferente"; mecanismo de salvamento: "periódico e reativo"→apenas "mecanismo de salvamento" (deixa o Capítulo 5 definir que é reativo); vCPU: parêntese removido inteiramente; Lustre: cláusula inteira removida |
| 8 | `body.tex` (Trabalhos Relacionados: "Tolerância a Falhas em Aplicações MPI", parágrafo de posicionamento + tabela comparativa + discussão SOMPI) | **Mesmo erro do item 3, em quatro lugares a mais**, todos repetindo ou dependendo da ideia "substituição de instância spot implica IP diferente": (a) parágrafo de posicionamento ("assumem que a topologia de rede permanece estável... endereços IP existentes no momento do salvamento... endereçamento distinto"), citado inclusive como justificativa direta da contribuição do próprio trabalho ("Este trabalho investiga justamente essa lacuna"); (b) discussão do SOMPI ("o BLCR não realiza reinício em nós com endereços IP distintos, o que ocorre rotineiramente..."), que cita o item (a) por referência cruzada; (c) parágrafo "dois vazios complementares" ("presumem que os endereços IP se mantêm... implica um novo IP, essa premissa é violada a cada interrupção... eliminando a dependência de endereço IP"); (d) tabela comparativa (`tab:rel-comparacao`), linhas BLCR/DMTCP/SOMPI com "IP-dependente" na coluna de lacunas. Como o item (a) é usado para justificar a própria contribuição do TCC, deixar os outros três com a linguagem antiga criava uma inconsistência onde o texto afirma simultaneamente "o problema é troca de IP" (aqui) e "o problema é estado de conexão não preservado, independente do IP" (Introdução e Capítulo 5, já corrigidos). | **corrigido em 2026-10-10** — todas as quatro reescritas para a mesma causa já estabelecida (instância EC2 nova = estado de conexão de rede não preservado), linguagem consistente com os itens 3 e 7(b); tabela comparativa simplificada (DMTCP não precisava do prefixo "IP-dependente", a frase "falha ao reiniciar com nova instância spot" já era suficiente e correta sozinha) |
| 9 | `body.tex` (Trabalhos Relacionados: discussão do SOMPI, réplicas) | Duas afirmações fortes demais, encontradas pelo usuário lendo o PDF: "a replicação pura elimina o risco de interrupção (uma cópia sempre conclui)" — replicação reduz o risco, não elimina (revogação correlacionada entre réplicas é possível); correção inicial assumiu 2 réplicas ("as duas cópias"), mas o SOMPI otimiza uma proporção variável de réplicas, não um número fixo — corrigido novamente pelo usuário. | **corrigido em 2026-10-10** — "elimina... (uma cópia sempre conclui)" → "reduz... (improvável que todas as cópias sejam revogadas simultaneamente)" |
| 10 | `body.tex` (Fundamentação Teórica "Instâncias Burstable" 2.1.4 + Capítulo 4 "Configuração Experimental" 4.1) | Lacuna conceitual, não erro: a Seção 2.1.4 explicava o modelo de créditos (acúmulo/consumo/*throttling*) mas nunca mencionava que a AWS oferece dois modos de operação configuráveis (*Standard*, que aplica o *throttling*; *Unlimited*, que sustenta desempenho acima do *baseline* a custo extra por crédito excedente). O Capítulo 4 então introduzia "T3 Standard, T3 Unlimited e M5" sem nunca ter definido o que "Standard"/"Unlimited" significa — inclusive a coluna "Crédito excedente" da Tabela 4 só faz sentido no modo Unlimited, e isso também nunca era dito. Usuário notou a lacuna ao perceber que presumia (incorretamente) que a 2.1.4 já cobria essa distinção. | **corrigido em 2026-10-10** — 2.1.4 ganhou duas frases definindo os dois modos, com `\cite{aws2026burstable}`; 4.1 (Configuração Experimental, não 4.2 como tentado inicialmente) ganhou uma cláusula explícita amarrando "T3 avaliada nos modos Standard e Unlimited descritos na Seção 2.1.4" logo na frase que introduz a Tabela 4, antes de qualquer uso de "T3 Standard"/"T3 Unlimited" no texto |
| 11 | `body.tex` (abertura do Capítulo 6, antes da Seção 6.1) | Dois ajustes, mesma família dos achados 5 e 1c: (1) "Os experimentos estão organizados em três grupos: (i)...(ii)...(iii)..." precisa virar quatro grupos — a nova seção do ADAPTIVE (Etapa 4) entra entre Análise de Custo (grupo iii) e Discussão, precisa de um item (iv) próprio já na abertura do capítulo; (2) "No total, foram coletadas 282 execuções válidas" descreve o capítulo inteiro (igual ao achado 5), não só os grupos (i)-(iii) — precisa deixar claro que as execuções da Fase 10 (`synth_duration`, ~92) também fazem parte deste capítulo, fora desse número. | marcado, correção na Etapa 5 (depende do total final da Etapa 4) |
| 12 | `body.tex` (Capítulo 6, "Análise das Fases de Recuperação", decomposição de P2b via CG-C) | **Não é correção, é oportunidade de conteúdo** identificada pelo usuário: o gap de P2b medido aqui para o CG-C (REPLACE 120–123s, DEGRADED ~11s, diferença ~110s) é a mesma grandeza física da constante fixa de 109s usada na fórmula da heurística ADAPTIVE (`remaining_secs × coef > 109`, derivada do EP-D/LU-C na Fase 10 — ver `PHASE10_PLAN.md`). Como o CG-C não fez parte do treino da heurística (excluído da análise de sensibilidade ao timing por ter execução curta demais), essa coincidência serve como um reforço empírico independente de que 109s é um fato de latência de provisionamento da AWS, não específico de benchmark — vale citar explicitamente na derivação da heurística (Etapa 4). | marcado, conteúdo a incorporar na Etapa 4 |
| 13 | `body.tex` (Capítulo 6, mesma subseção, decomposição de P3 via CG-C — achado 3 da checklist principal, agora resolvido) | **Verificação pedida pelo usuário contra a Fase 10: confirmada, com uma conexão adicional não prevista.** A explicação "fração de capacidade" para o P3 do CG-C (ver achado 3) usa o mesmo modelo ingênuo 1/(N−1) que a Fase 10 mostrou não valer para EP-D/LU-C. Recalculando o coeficiente do CG-C do mesmo jeito que a Fase 10 (`penalidade/P3_replace`): N=2 → 1,81; N=4 → 0,33; N=8 → 0,008 — contra o naive 1,000/0,333/0,143. **Direção bate** (cai com N, ao contrário do achatamento visto em EP-D/LU-C), **magnitude não** (cai mais rápido que o naive, não mais devagar). Usuário propôs conexão forte: a Seção "Comunicação Desbalanceada" (`synth_imbalanced`, já existente) estabelece que o laço ativo do MANA só amplifica *overhead* quando processos competem pelo mesmo núcleo (co-localização), e que o `synth_imbalanced` não pôde testar isso por desenho (separação explícita entre nós). O DEGRADED, ao reduzir *workers*, é literalmente essa condição de co-localização ocorrendo de verdade — os dados de P3 do CG-C podem ser evidência sugestiva (não confirmada) do mesmo mecanismo, nunca conectada ao trabalho futuro já registrado sobre isso. Tratar como hipótese, não fato, mesmo padrão de cautela epistêmica que o relatório da Fase 10 usa para suas próprias inferências não medidas (ver Seções 5.3/3.5 do `analysis_report_phase10_ptbr.md`). | marcado, conteúdo a incorporar na Etapa 4; considerar também reforçar o item de trabalho futuro já existente sobre co-localização + atraso controlado, citando os dados de DEGRADED do CG-C como indício adicional |
| 14 | `body.tex` (abertura da Seção 6.5, "Comparação de Estratégias: REPLACE vs. DEGRADED") | **Não é correção — oportunidade de conexão proposta pelo usuário.** O corpo da Seção 6.5 (os três parágrafos sobre tempo total, crossover, razão de overhead) não precisa mudar — continua descrevendo a comparação empírica REPLACE/DEGRADED, que fica correta independente do ADAPTIVE existir (mesma categoria dos achados já classificados como "fica igual"). Mas o parágrafo de abertura pode ganhar uma frase final conectando essa comparação a seu reaproveitamento como dado de treino da heurística ADAPTIVE (Etapa 4), no mesmo padrão de sinalização que a própria Seção 6.5 já usa no fim da 6.5.3 ("A Seção~\ref{sec:custo} quantifica essa implicação..."). Não dá pra escrever a referência real sem o `\label` da nova seção do ADAPTIVE ainda não existir. | marcado, aplicar a frase real só depois da Etapa 4 criar a seção e seu `\label` |
| 15 | `body.tex` (Seção 6.5.1, "Tempo Total de Execução", logo após a discussão da coluna Δ) | **Mesmo achado 3 da checklist principal — 4ª ocorrência não catalogada em nenhum levantamento anterior** (nem no planejamento original, nem no re-grep da Etapa 1). Encontrada só porque o usuário leu o parágrafo diretamente no PDF. Mesmo padrão do achado 1b: grep nunca garante cobertura completa. | marcado, correção na Etapa 5 (menção breve + referência à nova seção do ADAPTIVE, Etapa 4) |
| 16 | `body.tex` (Seção 6.5.2, "Análise de Crossover") | A ocorrência já catalogada no planejamento original (achado 3) — agora localizada com precisão e marcada no texto. | marcado, correção na Etapa 5 (mesmo tratamento do achado 15) |
| 17 | `body.tex` (Seção "Discussão") | A ocorrência já catalogada no planejamento original (achado 3) — agora localizada com precisão e marcada no texto. Por ser a seção de síntese do capítulo, pode ser o lugar mais natural para a referência à nova seção do ADAPTIVE ficar mais explícita, já que o leitor já viu as duas menções breves anteriores (achados 15 e 16) antes de chegar aqui. | marcado, correção na Etapa 5 (mesmo tratamento dos achados 15/16) |
| 18 | `body.tex` (marcador do achado 13, atualizado) | **Esclarecimento quantitativo pedido pelo usuário, não um achado novo por si.** O usuário pediu para verificar se o "efeito de duração" (achados 15–17, que explica por que Δ cresce com N para EP-D/LU-C apesar do coeficiente ficar achatado) também se aplica ao CG-C (achado 13). Verificação numérica: para EP-D, `remaining_secs` encolhe ~3,85× de N=2 a N=8 e isso sozinho explica quase todo o encolhimento de `penalidade` (~4,17×), com `coef` quase imóvel (~1,08×) — efeito de duração puro. Para o CG-C, `remaining_secs` (proxy: P3_replace) encolhe só ~1,63×, mas `penalidade` encolhe ~364× e `coef` colapsa ~223× — o efeito de duração explica quase nada; é um colapso genuíno do coeficiente, consistente com a hipótese do laço ativo/co-localização do achado 13, não com o efeito de duração. **São dois mecanismos diferentes para perfis de carga diferentes, não duas instâncias do mesmo fenômeno** — a Etapa 4 precisa deixar isso explícito se citar os dois achados juntos. | marcador do achado 13 atualizado em `body.tex` com a distinção quantificada |
| 19 | `body.tex` (Seção 6.6, "Análise de Custo", logo após a convergência EP-D) | **Oportunidade de conteúdo proposta pelo usuário**, revisada contra a Seção 6.5 (verificada limpa, sem repetição do modelo ingênuo) e contra a Discussão (confirmado que não contradiz "tempo e custo apontam consistentemente na mesma direção em todos os cenários avaliados" — essa frase é escopada ao intervalo testado, 12–500s). A Seção 6.5.2 já tem um parágrafo de extrapolação para jobs mais longos que projeta a REPLACE como preferível *em tempo*; a Seção 6.6 só vai até a convergência de custo no EP-D (30% vs. 31%) e nunca dá o passo seguinte, que a própria seção já tem a base lógica para dar: se o P2b da REPLACE pesa cada vez menos com jobs maiores (já estabelecido na mesma seção), e cargas HPC reais tipicamente rodam por horas ou dias (já dito na Introdução), a extrapolação natural é que a REPLACE ultrapassaria a DEGRADED em economia de custo para jobs de produção — espelhando a extrapolação de tempo que já existe em 6.5.2. | marcado, conteúdo a escrever na Etapa 5 como extrapolação explícita, não resultado medido |
| 20 | `body.tex` (Capítulo 6, "Ameaças à Validade", parágrafo de validade externa) | **Oportunidade de conexão, achada na varredura completa do restante do capítulo** (pedida pelo usuário depois de ele confirmar ter lido tudo exceto o Capítulo 7). O parágrafo já afirma abstratamente que os três benchmarks testados "não esgotam o espaço de aplicações MPI" e que outros padrões "podem exibir overhead e sensibilidade às estratégias diferentes". A Limitação 2 do ADAPTIVE (Fase 10: "o coeficiente não é universal entre cargas de trabalho", demonstrada com `synth_duration` f=50%/N=2, custo real de ~5,8% por usar coeficiente calibrado para outro perfil) é evidência concreta e medida para essa alegação hoje genérica. | marcado, conteúdo a incorporar na Etapa 4/5 (depende do `\label` da nova seção do ADAPTIVE ainda não existir) |

**Varredura completa do restante do capítulo (2026-10-12):** além do achado 20, revisadas e confirmadas limpas (sem repetição do modelo ingênuo, sem erro de IP, sem inconsistência nova) as seções: Gerenciamento de Recursos (SLURM), Programação Paralela com MPI, HPC@Cloud, NAS Parallel Benchmarks, as três seções restantes de Trabalhos Relacionados ("HPC em Instâncias Spot e Custo", "Abordagens Proativas e Reativas", "Síntese Comparativa" — já refletem a correção IP/conexão de rede aplicada antes), Capítulo 4.3 ("Oportunidade de Uso de Instâncias Burstable"), todo o restante do Capítulo 5 (Visão Geral da Arquitetura, Bootstrap, Coordenação entre Processos e Métricas), e o restante do Capítulo 6 (Overhead do MANA sem Falhas, Escalabilidade — números conferidos e batem com a Tabela 8 —, os três estudos sintéticos completos, e a parte de validade interna de Ameaças à Validade). Também verificado e decidido **não alterar** o resumo "4 a 5s" de overhead dos programas sintéticos — maioria dos pontos medidos (11 de 15, entre as três subseções de estudos sintéticos) cai de fato nessa faixa; os poucos fora dela (2,2s e 3,5s por baixo, 5,9s por cima) não justificam trocar por uma caracterização menos natural.

### Etapa 2 — Revisão do checklist de correções contra o texto pós-leitura

**[CONCLUÍDA em 2026-10-12]**

Depois da Etapa 1, re-rodar os greps que fundamentam a seção "Pontos de
atualização textual" acima (`282`, `5 e 34`, `fração.*capacidade`,
`contadores de.*hardware`) sobre o `body.tex` já relido, para confirmar que
nada mudou e que nenhuma ocorrência foi perdida no levantamento inicial.
Incorporar qualquer inconsistência nova encontrada na Etapa 1 que seja
relevante ao escopo do `ADAPTIVE` (fora de escopo fica anotado, não
corrigido nesta fase, salvo decisão em contrário do usuário).

**Resultado da execução:**

- **`282`**: todas as ocorrências conferidas (`body.tex` e `prolog.tex`).
  Todas já marcadas (achados 4, 5, 11) ou corretamente escopadas sem precisar
  de marcador: Resumo curto e Abstract curto (`prolog.tex:51,274`, dizem
  explicitamente "sobre os benchmarks NPB"), Resumo Estendido
  (`prolog.tex:182`, escopado pela própria matriz descrita na frase, mesma
  categoria do achado 2, deferida), `tab:matriz` (`body.tex:1782`, escopada
  pelo texto que a precede). Uma ocorrência (`body.tex:2500`) é valor de
  tabela não relacionado, não um "total de execuções". Uma ocorrência
  (`body.tex:2978`) está no Capítulo 7 (Considerações Finais), já
  corretamente escopada ("sobre os benchmarks NPB") — fora de escopo por
  enquanto (Conclusão deferida), anotada aqui só para registro.
- **`5 e 34`**: **achou 2 ocorrências reais que nunca tinham virado marcador
  de verdade** — estavam catalogadas no checklist de planejamento original
  (itens 2 e 4 de "Pontos de atualização textual") mas nunca foram
  transferidas pra um `\todo` no texto durante a Etapa 1. Corrigido agora:
  Seção 6.4.2 (`body.tex`, ~linha 2398, junto com a correção de contadores
  de hardware) e Discussão (~linha 2881). A 3ª ocorrência (Trabalhos
  Futuros, Capítulo 7) confirmada fora de escopo por enquanto.
- **`fração.*capacidade`**: confirmadas as 4 ocorrências reais do modelo
  ingênuo 1/(N−1) (CG-C, 6.5.1, Crossover, Discussão), todas já marcadas.
  Descoberto que esse padrão de grep tem um ponto cego: não captura a
  fraseologia "1/2 da capacidade"/"1/8 da capacidade" (sem a palavra
  "fração"), usada em 2 das 4 ocorrências (6.5.1 e Crossover) — essas já
  tinham sido encontradas e marcadas por leitura direta, não por este grep,
  então não houve lacuna real, mas o padrão de busca em si não seria
  suficiente sozinho. Sweep mais amplo por "da capacidade" confirmou não
  haver uma 5ª ocorrência escondida.
- **`contadores de.*hardware`**: mesma situação do "5 e 34" — a ocorrência
  da Seção 6.4.2 nunca tinha virado marcador (corrigida junto com o item
  acima). A ocorrência de Trabalhos Futuros (Capítulo 7) confirmada fora de
  escopo por enquanto.

**Lição geral desta etapa**: o checklist de planejamento (seção "Pontos de
atualização textual") e os marcadores `\todo` reais no texto podem divergir
silenciosamente — catalogar um achado durante o planejamento não garante
que ele vire um marcador visível depois. Vale repetir esse tipo de
re-verificação antes de fechar cada etapa futura que dependa do checklist.

### Etapa 3 — Redigir a subseção ADAPTIVE no Capítulo 5

**[CONCLUÍDA em 2026-10-18]**

Adicionar ao final da Seção `Estratégias de Recuperação` (depois do
parágrafo da `DEGRADED`, antes da Figura `fig:estrategias-flow`, ou depois
dela — decidir na redação conforme o fluxo) o mecanismo do `ADAPTIVE`: a
fórmula `remaining_secs × coef > custo_fixo`, o papel de
`estimated_duration_secs`, o caso especial `remaining_workers == 0 →
REPLACE`, e a função `choose_effective_strategy()` como referência de
implementação. Ajustar a frase de abertura da seção ("um dos dois
caminhos" → formulação que admita três) e o comentário do trecho YAML da
linha ~1368 (item 7 do checklist). Aplicar aqui as edições de
`strategy-flow.png` e `watcher-flow.png` (item 7 do checklist), já que
ambas ilustram diretamente o mecanismo descrito nesta subseção. Sem números
de validação aqui — isso é Capítulo 6.

**Execução, dividida em partes para verificação incremental (pedido do usuário):**

- **Parte 1** (texto pequeno, já marcado): frase de abertura reescrita
  ("o watcher aplica a estratégia de recuperação configurada... Replace ou
  Degraded... ou Adaptive, que escolhe automaticamente entre as duas");
  comentário YAML atualizado (`# ou DEGRADED_RESUME, ADAPTIVE`).
- **Figuras**: `watcher-flow.png` e `strategy-flow.png` editadas pelo
  usuário — ver achado correspondente no checklist de figuras (ambas
  marcadas `[FEITO]` ali). `strategy-flow.png` ganhou um diamante de
  decisão (abordagem diferente da minha proposta original de só adicionar
  uma segunda linha ao rótulo do container, mas resultado melhor —
  reaproveita a convenção visual de decisão já usada em `watcher-flow.png`
  e mantém exatamente dois ramos de saída, não sugerindo um terceiro
  caminho de infraestrutura).
- **Parte 2** (conteúdo novo, 3 parágrafos no fim da seção, depois do
  parágrafo de convergência P2c): parágrafo 1 introduz o `ADAPTIVE` e cita
  `choose_effective_strategy()` com o timing exato de chamada (depois do
  nó drenado e `job` cancelado no Slurm, antes do despacho) — confirmado
  contra o código via agente de verificação antes de escrever (nomes
  exatos: `PENALTY_COEFF`, `P2B_SAVING_SECS`, `remaining_workers`,
  `estimated_duration_secs`); parágrafo 2 apresenta a fórmula em notação
  `cases` do `amsmath` (já carregado pela classe), **mantida simbólica**
  (`coef`, `limiar`, não os valores 0,28/109) — decisão deliberada,
  consistente com o padrão já usado para Replace/Degraded (sem números de
  validação em evidência no Capítulo 5), evita um "número mágico" sem
  justificativa aparecendo antes da derivação no Capítulo 6; parágrafo 3
  cobre o caso `remaining_workers == 0`. Verificado visualmente no PDF
  compilado — fórmula renderiza corretamente, fluxo de leitura natural até
  a Seção 5.5.
- **Revisão pós-Parte 2, pedida pelo usuário (2026-10-18):** três ajustes de
  estilo/conteúdo no texto recém-escrito: (1) removida a citação ao nome da
  função `choose_effective_strategy()` — confirmado por grep que não há
  nenhuma outra citação de nome de função interna em todo o documento (só
  comandos externos, como `dmtcp_command -c`), então nomear a função era
  um desvio de estilo sem precedente; reescrito para descrever o
  comportamento (quando/onde a decisão ocorre) sem citar o nome; (2)
  `estimated_duration_secs` adicionado como campo de exemplo na
  Listagem 2 (`lst:ft-yaml`), que antes não mostrava esse campo apesar do
  texto já referenciá-lo; (3) frase sobre o comportamento padrão sem
  `estimated_duration_secs` simplificada, removida a justificativa ("a
  recuperação mais barata, em vez de arriscar uma estimativa sem base"),
  mantido só o fato (aplica Degraded por padrão). Também resolvido, na
  mesma passada: achado 1c (Seção 5.3, frase "o fluxo diverge de acordo
  com a estratégia de recuperação configurada, REPLACE ou DEGRADED"),
  reescrita para indicar que esse é o ponto exato onde o ADAPTIVE resolve
  a escolha, sem citar o nome da função, consistente com a correção (1)
  acima.
- Markers removidos (figuras: 2; texto: 3) ao longo da execução — contagem
  de `[FASE11]` em `body.tex` caiu de 26 para 19.

### Etapa 4 — Redigir a nova seção no Capítulo 6

Seção nova entre `Análise de Custo` e `Discussão`, com as subseções listadas
na "Decisão de estrutura" acima. Reaproveitar as figuras já geradas na
Fase 10 (`fig_synth_duration_boundary.png`,
`fig_coefficient_n_independence.png`, e possivelmente
`fig_adaptive_boundary.png` na subseção de derivação), aplicando a correção
de título embutido descrita no item 7 do checklist acima, e a nova tabela de
matriz de experimentos do `synth_duration` (item 8). Adaptar o texto dos
relatórios de análise (`analysis_report_phase10.md`/`_ptbr.md`) para o tom
da monografia (mais formal, mais denso em dados, menos didático que o
relatório — a banca já leu o Capítulo 5 e não precisa de reexplicação do
mecanismo). A subseção de calibração automática (Etapa 13) deve ser curta —
um parágrafo ou dois resumindo a tentativa, a alternativa, o resultado
parcial e a decisão de documentar como trabalho futuro, não uma reprodução
de toda a investigação do `PHASE10_PLAN.md`.

### Etapa 5 — Aplicar o checklist de correções pontuais

Aplicar, de uma vez, os 4 itens técnicos do checklist acima (Tabela 16,
"34%→37%" nos três lugares, revisão cuidadosa dos três trechos "fração de
capacidade" — dois reescritos, um avaliado caso a caso —, e revisão das
duas menções a contadores de hardware). Re-grep imediatamente depois para
confirmar que não restou nenhuma ocorrência desatualizada.

### Etapa 6 — Atualizar Capítulo 1 e Capítulo 7

Capítulo 1: decidir e aplicar a extensão de `Objetivos Específicos` e
`Contribuições` (um item novo vs. extensão de item existente — decisão a
justificar no momento da escrita, não antecipada aqui), e a frase de
`Organização do Trabalho` sobre a nova seção do Capítulo 6. Capítulo 7:
estender `Considerações Finais` com uma síntese curta do `ADAPTIVE`, e
`Trabalhos Futuros` com a calibração automática do coeficiente como
extensão natural (ligando à Etapa 13).

### Etapa 7 — Consolidar contagens agregadas de execuções

Aplicar o item 5 do checklist: confirmar escopo de cada menção de "282",
adicionar o total da Fase 10 onde o texto hoje implica (mesmo que
implicitamente) que 282 é o total geral de execuções do trabalho. Revisar
`Resumo`/`Abstract` curtos (item 6 do checklist) e decidir se recebem uma
frase sobre o `ADAPTIVE`.

### Etapa 8 — Compilação e revisão final

`latexmk` (ou o `Makefile` do projeto) para compilar o PDF; verificar que
não há warnings novos de referência quebrada (`\ref`/`\label`) introduzidos
pelas edições; revisão humana do PDF gerado, não só do `.tex`, prestando
atenção a: numeração de tabelas/figuras/seções (toda inserção no meio do
Capítulo 6 desloca numeração subsequente), quebras de página ruins ao redor
de tabelas/figuras novas, e fluidez de leitura ponta a ponta nos trechos
editados.

### Etapa 9 — Remoção do Resumo Estendido [CONCLUÍDA em 2026-10-18]

O orientador já havia autorizado a remoção da seção "Resumo Estendido"
(`prolog.tex`, ambiente `\begin{resumo}[Resumo Estendido]...\end{resumo}`,
linhas 70–258, com subseções `Introdução`, `Objetivos`, `Metodologia`,
`Resultados e Discussão`, `Considerações Finais`). Executada mediante pedido
explícito do usuário durante a varredura dos marcadores `[FASE11]` em ordem
de documento, ao chegar no primeiro marcador dentro dessa seção (o item 2 do
checklist principal, "duas estratégias" dentro do Resumo Estendido, decisão
antes pendente). Removida via `sed` (linhas 70–260, preservando o
espaçamento entre o Resumo curto e o Abstract); compilação verificada limpa
depois (112 páginas, antes 116 — sem \texttt{undefined references}). Os dois
marcadores `[FASE11]` que estavam dentro da seção removida (achado 2 do
checklist) saíram junto, sem precisar de correção individual.

---

## Gates de Aceitação

| Gate | Descrição |
|------|-----------|
| G1 | Leitura completa do `body.tex` (Etapa 1) concluída; lista de inconsistências (relacionadas e não relacionadas ao ADAPTIVE) revisada com o usuário antes de qualquer edição |
| G2 | Decisão de estrutura (Capítulo 5 + nova seção no Capítulo 6, sem capítulo novo) confirmada com o usuário antes da redação do conteúdo novo |
| G3 | Subseção ADAPTIVE no Capítulo 5 redigida, consistente em terminologia e tom com `REPLACE`/`DEGRADED` já descritos na mesma seção; `strategy-flow.png`, `watcher-flow.png` e o comentário YAML da linha ~1368 atualizados para refletir o ADAPTIVE (item 7 do checklist) |
| G4 | Nova seção no Capítulo 6 redigida, com as figuras da Fase 10 (título embutido removido, movido para `\caption{}`) e a nova tabela de matriz de experimentos do `synth_duration`, sem duplicar explicações já dadas no Capítulo 5 |
| G5 | Checklist de correções pontuais 100% aplicado (Tabela 16, "34%→37%" nos 3 locais, trechos "fração de capacidade" revisados nos 3 locais com o tratamento diferenciado decidido na Etapa 2, menções a contadores de hardware revisadas nos 2 locais) e re-verificado via grep após a edição |
| G6 | Contagens agregadas de execuções consistentes em todas as menções (`body.tex` e `prolog.tex`), sem implicar que 282 é o total geral quando não é |
| G7 | Capítulo 1 (Objetivos Específicos, Contribuições, Organização) e Capítulo 7 (Considerações Finais, Trabalhos Futuros) atualizados para refletir o ADAPTIVE |
| G8 | Compilação limpa via `latexmk`/`Makefile`, sem warnings novos de referência quebrada; PDF revisado visualmente pelo usuário |
| G9 | Resumo Estendido removido somente mediante pedido explícito do usuário (2026-10-18), não executado de passagem junto com outras edições — ✓ concluído |
| G10 | Todos os marcadores `\todo[...]{...}` removidos de `body.tex`/`prolog.tex` e o `\usepackage{todonotes}` removido de `main.tex` (`grep -rn "\[FASE11\]\|todonotes" *.tex` sem resultado), com `make pdf` compilando limpo depois da remoção |

---

## Arquivos de Referência

| Arquivo | Uso |
|---------|-----|
| `TCC/artifacts/phase6/lapesd-thesis/body.tex` | Documento a editar — Capítulos 1, 5, 6 e 7 |
| `TCC/artifacts/phase6/lapesd-thesis/prolog.tex` | Resumo/Abstract curtos e Resumo Estendido (este último, ver Etapa 9) |
| `TCC/artifacts/phase6/lapesd-thesis/main.tex` | Estrutura de includes do documento |
| `TCC/artifacts/phase10/PHASE10_PLAN.md` | Checklist original de mudanças em `body.tex` (Etapas 1–13), dados brutos de N=2 para a Tabela 16 |
| `TCC/artifacts/phase10/PHASE10_ARTIFACT.md` | Síntese fechada da Fase 10 — base para as contagens de execuções e lista de limitações |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10.md` | Relatório consolidado (EN) — base de texto para a nova seção do Capítulo 6 |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10_ptbr.md` | Relatório consolidado (PT-BR) — base de texto e tom para a nova seção do Capítulo 6 |
| `TCC/artifacts/phase10/analysis/fig_synth_duration_boundary.png` | Figura a reaproveitar no Capítulo 6 |
| `TCC/artifacts/phase10/analysis/fig_coefficient_n_independence.png` | Figura a reaproveitar no Capítulo 6 |
| `src/commands/cluster/watcher.rs` | Referência de implementação (`choose_effective_strategy()`) para a subseção do Capítulo 5 |
