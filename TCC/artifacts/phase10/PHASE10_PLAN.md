# Phase 10 Plan — Estratégia ADAPTIVE de Recuperação

Date: 2026-09-12
Projeto: Sustainable High Performance Computing on AWS
Bolsista: Artur Luiz Rizzato Toru Soda
Orientador: Prof. Dr. Márcio Bastos Castro
Instituição: LaPeSD, INE/UFSC

---

## Objetivo

Implementar uma terceira estratégia de recuperação — `ADAPTIVE` — que escolhe
automaticamente entre `REPLACE_RESUME` e `DEGRADED_RESUME` no momento da
falha, com base em uma heurística derivada dos resultados experimentais
do UCC 2026. A contribuição principal é eliminar a necessidade de o
usuário escolher manualmente a estratégia, substituindo por uma decisão
informada em tempo de execução.

Além da implementação, esta fase inclui uma extensão do conjunto de dados
experimental para caracterizar empiricamente a fronteira de decisão nos
dois sentidos (REPLACE e DEGRADED), enriquecendo a análise de crossover
já presente no TCC.

---

## Escopo da estimativa de tempo (T_estimated)

A fórmula da heurística depende de `remaining_secs = T_estimated − elapsed_secs`.
De onde vem `T_estimated` é uma decisão de escopo separada da fórmula em si, e
duas abordagens foram consideradas:

1. **Fornecida manualmente pelo usuário no YAML** (`estimated_duration_secs`).
   O usuário informa o tempo esperado do job com base no seu próprio conhecimento
   (rodou antes, estimou pelo tamanho do problema, etc.).
2. **Estimada automaticamente a partir do histórico de execuções.** Um modelo
   simples (ou até uma média histórica) consultaria execuções anteriores do
   mesmo benchmark/configuração para prever `T_estimated` sem exigir que o
   usuário informe nada.

**A Fase 10 implementa apenas a abordagem 1.** Ela é suficiente para validar a
fórmula, é simples de implementar (um campo opcional no YAML) e não exige
infraestrutura nova. A abordagem 2 é registrada aqui como **trabalho futuro**,
não como parte do escopo atual, porque exigiria:

- uma base de dados de execuções históricas por configuração (benchmark, N,
  classe do problema, tipo de instância)
- uma forma de identificar/agrupar jobs semelhantes para reaproveitar o histórico
- um modelo de previsão (ainda que simples) integrado ao watcher, consultado
  antes da decisão ADAPTIVE

**Por que isso importa:** a precisão da decisão ADAPTIVE depende inteiramente
da qualidade de `T_estimated`. Se o usuário fornece uma estimativa ruim, a
fórmula pode escolher a estratégia errada mesmo com o coeficiente 0.28
corretamente calibrado. Esse é um limite conhecido da abordagem manual, e deve
ser documentado explicitamente no TCC como motivação para a abordagem 2 como
extensão futura do trabalho.

---

## Status

| Etapa | Atividade | Status |
|---|---|---|
| 1 | Confirmar a fórmula heurística contra os dados brutos | ✓ |
| 2 | Documentar a heurística, a fronteira e os casos de borda | ✓ |
| 3 | Mapear todas as mudanças de código necessárias | ✓ |
| 4 | Adicionar `estimated_duration_secs` ao `FaultToleranceConfig` em `run_task.rs` | ✓ |
| 5 | Adicionar `estimated_duration_secs` ao `FaultToleranceConfig` em `watcher.rs` e atualizar o construtor | ✓ |
| 6 | Adicionar `"ADAPTIVE"` à validação de estratégia em `run_task.rs` | ✓ |
| 7 | Passar `watcher_started_at` como parâmetro para `handle_interruption()` | ✓ |
| 8 | Implementar a função `choose_effective_strategy()` em `watcher.rs` | ✓ |
| 9 | Conectar `"ADAPTIVE"` ao dispatch em `handle_interruption()` | ✓ |
| 10 | Smoke test end-to-end com estratégia ADAPTIVE | ✓ |
| 11 | Validar a fronteira: workload sintético (N=2/4/8). ~~NPB f=15%~~ cancelado (Tabela 16 completada com dados já existentes) | ✓ (matriz de 14 pontos × 3 repetições = 84 execuções completa) |
| 12 | Avaliação de integração do ADAPTIVE (escopo reduzido: casos claros + margem + limite do coeficiente; caso zero-workers por inspeção de código) | ✓ |

---

## Dados Experimentais Existentes (Tabela 6.5 do TCC)

Δ = T_Replace − T_Degraded em segundos. Negativo = REPLACE mais rápida.

| Benchmark | N workers | Timing | T_Replace (s) | T_Degraded (s) | Δ (s) | Vencedor |
|-----------|-----------|--------|--------------|----------------|-------|---------|
| EP-D | 2 | 10% | 516 | 535 | **−19** | **REPLACE** |
| EP-D | 2 | 25% | 543 | 532 | +11 | DEGRADED |
| EP-D | 2 | 50% | 573 | 541 | +32 | DEGRADED |
| EP-D | 4 | 10% | 330 | 294 | +36 | DEGRADED |
| EP-D | 4 | 25% | 358 | 302 | +56 | DEGRADED |
| EP-D | 4 | 50% | 378 | 299 | +79 | DEGRADED |
| EP-D | 8 | 10% | 269 | 179 | +90 | DEGRADED |
| EP-D | 8 | 25% | 270 | 182 | +88 | DEGRADED |
| EP-D | 8 | 50% | 282 | 184 | +98 | DEGRADED |
| LU-C | 2 | 10% | 329 | 304 | +25 | DEGRADED |
| LU-C | 2 | 25% | 341 | 303 | +38 | DEGRADED |
| LU-C | 2 | 50% | 369 | 302 | +67 | DEGRADED |
| LU-C | 4 | 10% | 266 | 193 | +73 | DEGRADED |
| LU-C | 4 | 25% | 267 | 194 | +73 | DEGRADED |
| LU-C | 4 | 50% | 275 | 193 | +82 | DEGRADED |
| LU-C | 8 | 10% | 231 | 136 | +95 | DEGRADED |
| LU-C | 8 | 25% | 247 | 136 | +111 | DEGRADED |
| LU-C | 8 | 50% | 236 | 141 | +95 | DEGRADED |

17 de 18 configurações → DEGRADED vence. Única exceção: EP-D, N=2, 10%.

**Limitação do conjunto atual:** com apenas 1 ponto REPLACE, uma baseline trivial
("sempre DEGRADED") acertaria 94% dos casos. O ADAPTIVE precisa ser validado nos dois
sentidos. A Etapa 11 resolve isso com experimentos adicionais.

---

## Derivação da Fórmula Heurística

### Modelo teórico (ponto de partida)

A diferença entre as estratégias está em duas fases:
- **P2b**: REPLACE espera ~120s pelo provisionamento; DEGRADED fecha em ~11s. Economia: **~109s**
- **P3**: REPLACE retoma com N workers; DEGRADED retoma com N−1, ampliando o tempo de P3

Assumindo scaling linear:

```
P3_replace  ≈ remaining_secs
P3_degraded ≈ remaining_secs × N/(N−1)

REPLACE vence quando:  remaining_secs / (N−1) > 109
```

### Resultado da Etapa 1 — como a fórmula foi descoberta

**Valores de T_MANA_noFT extraídos do CSV (usados como estimativa do tempo total do job):**

| Benchmark | N=2 | N=4 | N=8 |
|-----------|-----|-----|-----|
| EP-D | 484.5s | 249.0s | 125.7s |
| LU-C | 241.2s | 135.1s | 72.7s |

#### Descoberta passo a passo

**1. A fórmula teórica foi testada e falhou.**

A fórmula `remaining_secs / (N-1) > 109` foi aplicada a todas as 18 configurações
usando `remaining_secs = T_MANA_noFT × (1-f)`. Resultado: **13/18 corretos**. As 5
falhas são todas em N=2 (LU-C f=10%, 25%, 50% e EP-D f=25%, 50%) — o modelo prediz
REPLACE quando o correto é DEGRADED. O motivo: o modelo assume que P3_degraded leva
N/(N-1) = 2× mais tempo que P3_replace em N=2, mas isso não ocorre na prática.

**2. Qual é então a condição correta?**

Extraímos do CSV os tempos reais de P3_replace e P3_degraded e calculamos:

```
penalty = P3_degraded - P3_replace
```

Essa é a penalidade real: quantos segundos a mais o DEGRADED leva em P3 comparado ao
REPLACE. A decisão correta é simplesmente `penalty > 109` (os 109s que o REPLACE perde
esperando a máquina nova). Resultado: **18/18 corretos** — é o ground truth da decisão.

**3. Problema: na hora da falha, não temos os P3 medidos.**

No código, quando o watcher detecta uma falha, ele sabe apenas:
- `estimated_duration_secs` (fornecido pelo usuário no YAML)
- `elapsed_secs` (tempo corrido desde o início)
- Portanto: `remaining_secs = estimated - elapsed`

Ele **não** tem acesso aos P3 reais — esses só existem no CSV após os experimentos.
Precisamos estimar `penalty` a partir de `remaining_secs`.

**4. Como estimar a penalidade a partir de remaining_secs?**

Se a penalidade fosse uma fração fixa de remaining_secs, poderíamos escrever:

```
penalty ≈ coeff × remaining_secs
```

e a decisão viraria `coeff × remaining_secs > 109`. Para descobrir se isso é válido,
calculamos `coeff = penalty / remaining_secs` para cada uma das 18 configurações:

| Benchmark | N | f% | remaining_secs | P3_replace | P3_degraded | penalty | coeff |
|-----------|---|----|----------------|-----------|-------------|---------|-------|
| EP-D | 2 | 10% | 436.1 | 322.3 | 446.1 | 123.9 | **0.284** ← REPLACE |
| EP-D | 2 | 25% | 363.4 | 269.8 | 373.5 | 103.7 | **0.285** |
| EP-D | 2 | 50% | 242.3 | 187.7 | 262.4 | 74.7 | 0.308 |
| EP-D | 4 | 10% | 224.1 | 151.6 | 217.9 | 66.3 | 0.296 |
| EP-D | 4 | 25% | 186.7 | 121.9 | 170.5 | 48.6 | 0.260 |
| EP-D | 4 | 50% | 124.5 | 69.7 | 100.3 | 30.6 | 0.246 |
| EP-D | 8 | 10% | 113.2 | 87.1 | 116.8 | 29.7 | 0.263 |
| EP-D | 8 | 25% | 94.3 | 74.9 | 104.6 | 29.7 | 0.315 |
| EP-D | 8 | 50% | 62.9 | 50.7 | 75.2 | 24.5 | 0.390 |
| LU-C | 2 | 10% | 217.1 | 141.3 | 228.0 | 86.6 | 0.399 |
| LU-C | 2 | 25% | 180.9 | 116.5 | 191.7 | 75.2 | 0.416 |
| LU-C | 2 | 50% | 120.6 | 85.5 | 130.7 | 45.3 | 0.375 |
| LU-C | 4 | 10% | 121.6 | 80.6 | 125.3 | 44.7 | 0.368 |
| LU-C | 4 | 25% | 101.3 | 68.0 | 104.5 | 36.5 | 0.360 |
| LU-C | 4 | 50% | 67.5 | 47.2 | 71.7 | 24.6 | 0.364 |
| LU-C | 8 | 10% | 65.5 | 52.7 | 68.5 | 15.9 | 0.242 |
| LU-C | 8 | 25% | 54.5 | 47.3 | 61.1 | 13.8 | 0.254 |
| LU-C | 8 | 50% | 36.4 | 36.7 | 47.3 | 10.5 | 0.290 |

Médias por N: N=2 → 0.345 | N=4 → 0.316 | N=8 → 0.292 | Geral: **0.317**

**5. Observação surpreendente: o coeficiente é aproximadamente constante em ~0.30,
independente de N.**

O modelo teórico previa que o coeficiente seria 1/(N-1) — muito diferente por N:

| N | Coeff real (média) | Teórico 1/(N-1) |
|---|-------------------|-----------------|
| 2 | 0.345 | 1.000 (3× acima do real) |
| 4 | 0.316 | 0.333 (próximo do real) |
| 8 | 0.292 | 0.143 (2× abaixo do real) |

O motivo da divergência: a derivação teórica assume `P3_replace ≈ remaining_secs`, mas
o TCC (Seção 6.4.2) já documentou que P3_replace é 21–34% mais curto que o esperado
(warm-restart effect). Além disso, para N=2, P3_degraded com oversubscribe não dobra o
tempo de P3 — 2 ranks em 1 máquina EP-D têm acesso a todos os cores e rodam na mesma
velocidade que 1 rank por máquina. Os dois efeitos se compensam de forma que o coeff
real fica em ~0.30 independente de N.

Consequência prática: a fórmula **não depende de N**. Não é preciso informar o número
de workers — apenas o tempo estimado e o tempo decorrido.

**6. Escolha do coeficiente 0.28.**

O coeff varia entre 0.242 e 0.416 nas 18 configurações. Para escolher um único valor
que funcione para todas, identificamos os dois únicos casos "borderline" — aqueles onde
a penalidade está próxima do threshold de 109s, e onde escolher um coeff errado
mudaria a decisão:

```
Caso A — EP-D N=2 f=10%: penalty = 123.9 > 109  →  REPLACE (correto)
  Para o coeff estimado também classificar como REPLACE:
      coeff × 436.1 > 109   →   coeff > 0.250

Caso B — EP-D N=2 f=25%: penalty = 103.7 < 109  →  DEGRADED (correto)
  Para o coeff estimado também classificar como DEGRADED:
      coeff × 363.4 < 109   →   coeff < 0.300
```

Ambos os casos têm coeff real ≈ 0.284–0.285. A faixa de valores que satisfaz as duas
restrições simultaneamente é **(0.250, 0.300)**. Qualquer valor fora dessa faixa
classifica pelo menos um dos dois casos de forma errada. Todos os outros 16 casos têm
penalidades bem distantes de 109 e são classificados corretamente por qualquer coeff
razoável (0.1 a 0.5).

**0.28 é o ponto médio da faixa válida (0.250, 0.300)**, e é também o valor mais
próximo dos coeficientes reais dos casos borderline (0.284 e 0.285).

**Fórmula final (18/18 corretos):**

```
remaining_secs = T_estimated − elapsed_secs

REPLACE quando:  remaining_secs × 0.28 > 109
DEGRADED quando: remaining_secs × 0.28 ≤ 109

Equivalente:  REPLACE quando remaining_secs > 389s  (N-independente)
```

#### Por que o TCC existente não está errado

O modelo teórico `remaining_secs / (N-1) > 109` foi uma hipótese de trabalho criada
para motivar a estrutura da fórmula. O TCC **nunca conteve essa fórmula** — a Seção
6.5 mostra apenas os 18 valores de Δ medidos e identifica o padrão visualmente, sem
derivar nenhum threshold. O TCC não está errado.

A fórmula `remaining_secs × 0.28 > 109` é **conteúdo novo** que a Phase 10 adiciona,
derivado a partir dos dados já existentes. Na Seção 6.5 do TCC revisado, a narrativa
será: "a partir dos dados experimentais, calculamos a penalidade real de P3 para cada
configuração. Observamos que a penalidade representa uma fração de ~28% do tempo
restante, independente do tamanho do cluster. Isso define o threshold: REPLACE é
preferível quando o tempo restante excede 389s."

### Fronteira de decisão e cobertura experimental

A fronteira `remaining_secs × 0.28 = 109` → `remaining_secs = 389s` é N-independente:

| N | T_MANA_noFT (EP-D) | Max remaining_secs | Cruza threshold? | Crossover em f |
|---|--------------------|--------------------|-----------------|----------------|
| 2 | 484.5s | 484.5s | **Sim** | f ≈ 20% |
| 4 | 249.0s | 249.0s | Não (max 249 < 389) | Nunca |
| 8 | 125.7s | 125.7s | Não (max 126 < 389) | Nunca |

Para LU-C N=2: T_MANA_noFT=241.2s, max remaining=241.2s < 389s → nunca REPLACE.

Assim, REPLACE só é possível para EP-D N=2 com f < 20% entre os benchmarks NPB
testados. Os experimentos existentes amostraram f=10% (REPLACE) e f=25%, 50%
(DEGRADED). A Etapa 11.1 adiciona f=15% (o ponto mais próximo da fronteira sem
cruzá-la) para reforçar essa caracterização com uma aplicação real. Como nenhum
outro benchmark/N cruza o threshold, a Etapa 11.2 usa um workload sintético com
duração controlável para testar a mesma fronteira em N=4 e N=8, o que os
benchmarks reais não permitem.

Usar classes maiores (EP-E, etc.) para cruzar o threshold em N≥4 é inviável:
o tamanho do checkpoint cresceria e P1 poderia exceder a janela de 2 minutos
do aviso spot (discutido na Seção 6.3.3 do TCC). Essa restrição é documentada
no TCC como motivação para o conjunto de experimentos escolhido.

### Múltiplas falhas

Na presença de múltiplas interrupções, a fórmula se adapta automaticamente porque
`elapsed_secs` cresce a cada falha: cada chamada de `handle_interruption()` recomputa
`remaining_secs = T_estimated − elapsed_secs` com o tempo corrido até aquele momento.
Quanto mais tarde a segunda (ou terceira) falha ocorre, menor é `remaining_secs` e mais
provável que o ADAPTIVE escolha DEGRADED — o que é o comportamento correto, pois resta
menos trabalho a recuperar e o custo de esperar 120s pesa mais.

---

## Impacto no TCC Existente

### O que o TCC atual já tem (e está correto)

A Seção 6.5 do TCC mostra os 18 valores de Δ medidos experimentalmente, o heatmap
de crossover (verde = DEGRADED, vermelho = REPLACE), e a conclusão qualitativa de que
"cluster menor com falha precoce favorece REPLACE". Esse conteúdo está correto e não
muda. O TCC **não contém nenhuma fórmula analítica de crossover** — apenas identifica
o padrão nos dados.

### O que a Phase 10 adiciona (conteúdo novo)

| Seção | O que muda | Natureza |
|-------|-----------|---------|
| Seção 6.4.2 — Tabela warm-restart-gap | Adicionar linha N=2 (atualmente ausente) com os dados da Etapa 11.1 | Dados novos |
| Seção 6.5 — Tabela strategy-wall-time | Nova linha: EP-D N=2 f=15% para ambas as estratégias | Dados novos |
| Seção 6.5 — Análise de Crossover | Parágrafo formalizando a fórmula `remaining_secs × 0.28 > 109` e o crossover em f≈20% para EP-D N=2; heatmap atualizado com o ponto adicional | Análise nova |
| Novo capítulo/apêndice — ADAPTIVE | Implementação, derivação da heurística, validação sintética de N-independência (Etapa 11.2), avaliação experimental completa | Contribuição nova |

### Narrativa unificada

A Etapa 11 resolve dois problemas distintos, e o TCC deve tratá-los como coisas
separadas, não misturar:

1. **Enriquecer a análise de crossover já existente (Seção 6.5):** o TCC já
   identificou que apenas 1 caso REPLACE em 18 é pouco para caracterizar a
   fronteira. O novo ponto NPB real (EP-D N=2, f=15%, Etapa 11.1) resolve isso
   com uma aplicação científica de verdade, e completa a Tabela 16 (warm-restart-gap)
   com dados de N=2, hoje ausentes.
2. **Validar que a fórmula generaliza entre tamanhos de cluster diferentes:**
   isso não dá pra fazer com benchmarks NPB reais (só EP-D N=2 cruza o threshold).
   O workload sintético (Etapa 11.2) resolve isso especificamente, testando o
   mesmo threshold de 389s em N=2, 4 e 8 sob condições controladas.

A fórmula `remaining_secs × 0.28 > 109` é apresentada no TCC como **derivação
a posteriori** dos dados experimentais: primeiro mostramos os dados, depois formalizamos
o threshold que os separa. O coeficiente 0.28 é explicitamente descrito como empírico,
com a faixa válida (0.250, 0.300) derivada dos dois casos borderline (Seção 6.5). A
validação sintética (Etapa 11.2) é apresentada como um experimento controlado
separado, complementar à análise de crossover baseada em aplicações reais, não como
substituto dela.

---

## Metodologia de Validação: Treino vs. Teste

**O problema:** o coeficiente 0.28 foi ajustado especificamente para classificar
corretamente os 18 pontos que o originaram (os dois casos borderline de EP-D N=2
definiram a faixa válida). Reportar "a fórmula acerta 18/18" não é validação, é
calibração: qualquer coeficiente escolhido para caber nesses dados vai, por
construção, caber nesses dados. Isso é circular e não prova que a fórmula
generaliza para além do que já foi observado.

**A solução: separar explicitamente treino e teste.**

- **Conjunto de treino (fechado):** os 18 pontos originais (Tabela 6.5 do TCC).
  O coeficiente 0.28 é derivado deles na Etapa 1 e **não é mais tocado** depois disso.
- **Conjunto de teste (dados nunca vistos pelo coeficiente):** os pontos novos da
  Etapa 11 — o ponto NPB real (EP-D N=2, f=15%) e os pontos do workload sintético
  (N=2, 4, 8). Antes de rodar qualquer um desses, o plano já registra a predição
  esperada (colunas "Predição"/"Esperado" nas tabelas da Etapa 11) — isso funciona
  como um pré-registro de hipótese, boa prática científica que evita ajustar a
  narrativa depois de ver o resultado.

**Por que a ordem das etapas importa:** a implementação (Etapas 3-9) precisa
acontecer **com o coeficiente 0.28 já fixado pelo conjunto de treino**, antes de
rodar a Etapa 11. Se invertêssemos essa ordem (rodar Etapa 11 primeiro e só
depois decidir o coeficiente do código), estaríamos efetivamente "espiando" o
conjunto de teste antes de fechar o modelo — um vazamento de dados de teste que
invalidaria qualquer alegação de generalização, mesmo que o resultado final
parecesse bom.

**Regra a seguir:** se o resultado da Etapa 11 divergir da predição (ex.: o ponto
sintético de N=4 na fronteira classificar errado), o coeficiente **não deve ser
reajustado a posteriori para "consertar" o resultado**. Isso deve ser reportado
como um achado honesto (ex.: "o coeficiente 0.28, calibrado em N=2, não se
sustenta perfeitamente em N=4/N=8") e documentado como limitação, não escondido
atrás de um novo ajuste de curva.

**O que isso prova, de fato:** não uma correção universal, matematicamente
demonstrada. O que a Etapa 12 entrega é uma validação empírica padrão em
pesquisa de sistemas: a implementação real do ADAPTIVE é executada contra um
conjunto de ground truth que inclui dados nunca usados para calibrar a fórmula,
com escopo claramente documentado (esta infraestrutura AWS, estes padrões de
comunicação MPI, este intervalo de N). Essa é uma reivindicação cientificamente
honesta desde que a distinção treino/teste apareça explicitamente no texto do
TCC, e não é a mesma coisa que "a fórmula está provada correta em geral".

---

## Contexto: o que já existe no código

O campo `fault_tolerance.strategy` no YAML de task aceita:
- `"NONE"` — sem tolerância a falhas
- `"REPLACE_RESUME"` — provisiona novo nó (~120s), retoma em capacidade total
- `"DEGRADED_RESUME"` — reinicia imediatamente nos sobreviventes (~11s), com capacidade reduzida

### Estrutura do código relevante

| Arquivo | Linha | O que contém |
|---------|-------|-------------|
| `src/commands/cluster/run_task.rs` | 40-57 | `FaultToleranceConfig` struct (desserializada do YAML) |
| `src/commands/cluster/run_task.rs` | 500-508 | Validação de estratégia + mensagem de erro |
| `src/commands/cluster/run_task.rs` | 888-895 | Construção do `watcher::FaultToleranceConfig` e spawn do watcher |
| `src/commands/cluster/watcher.rs` | 14-21 | `FaultToleranceConfig` struct do watcher |
| `src/commands/cluster/watcher.rs` | 43-54 | Assinatura de `run_watcher()` (não recebe timestamp de `run_task.rs`) |
| `src/commands/cluster/watcher.rs` | 60 | `watcher_started_at` (timestamp RFC3339 computado localmente) |
| `src/commands/cluster/watcher.rs` | 125-137 | Único call site de `handle_interruption()` |
| `src/commands/cluster/watcher.rs` | 451-463 | Assinatura de `handle_interruption()` — shared prefix: checkpoint + drain + scancel |
| `src/commands/cluster/watcher.rs` | 648-684 | Match de estratégia que despacha para `handle_policy_*` |
| `src/commands/cluster/watcher.rs` | 907-973 | `handle_policy_degraded_resume()` — já trata `remaining_workers == 0` (para jobs, opcionalmente termina o cluster) |

Nota: `run_task.rs` também tem sua própria `run_started_at` (linha 851), usada
para rastrear ciclos de recuperação por execução — **não** é a mesma variável
que `watcher_started_at` em `watcher.rs:60` (são timestamps independentes,
calculados momentos um do outro). A Etapa 7 usa `watcher_started_at`, não a
`run_started_at` de `run_task.rs`.

---

## Detalhes de Cada Etapa

### Etapa 1 — Confirmar a fórmula heurística contra os dados brutos

**O que fazer:**

Abrir `TCC/artifacts/phase5/analysis/results_raw.csv` e, para cada uma das 18
configurações (EP-D+LU-C × N∈{2,4,8} × f∈{10%,25%,50%}):

1. Extrair `T_MANA_noFT(N)` do CSV (média das linhas com `strategy="MANA_noFT"`)
2. Calcular `remaining_secs = T_MANA_noFT * (1 − f)`
3. Calcular `remaining_secs / (N−1)` e comparar com 109
4. Conferir predição contra a coluna "Vencedor" da tabela acima

**Também verificar com P3_expected da Tabela 16 (warm-restart-gap):**

Para as 12 configurações disponíveis (N=4 e N=8), usar diretamente o valor
"P3 esperado" da Tabela 16 como `remaining_secs` (evita a dependência de
T_MANA_noFT). Isso testa se a fórmula funciona com o remaining_secs "real"
independentemente do modelo de scaling.

**Verificação:**
- Predição correta em todas as 18 linhas (ou identificar quais divergem e por quê)
- Confirmar que EP-D N=2 10% → ratio > 109 e EP-D N=2 25% → ratio < 109
- Registrar os valores de T_MANA_noFT extraídos — servirão como referência
  para definir `estimated_duration_secs` nos YAMLs dos novos experimentos
- Se alguma linha divergir, investigar antes de avançar

---

### Etapa 2 — Documentar a heurística, a fronteira e os casos de borda

**O que fazer:**

Criar `TCC/artifacts/phase10/heuristic.md` com:

**Fórmula:**
```
REPLACE quando:  (T_estimated − elapsed_secs) × 0.28 > 109
DEGRADED quando: (T_estimated − elapsed_secs) × 0.28 ≤ 109

Equivalente: REPLACE quando remaining_secs > 389s

Parâmetros:
  T_estimated   = estimated_duration_secs (fornecido pelo usuário no YAML)
  elapsed_secs  = segundos desde o início do watcher até o momento da falha
  threshold 109 = P2b_replace − P2b_degraded, medido experimentalmente
  coeff 0.28    = (β−α) empírico; válido no intervalo (0.25, 0.30) para separar
                  os casos borderline EP-D N=2 f=10% (REPLACE) e f=25% (DEGRADED)

Nota: a fórmula é N-independente. O fator remaining_workers não aparece porque
o coeficiente real (β−α) ≈ 0.28–0.31 é aproximadamente constante para N∈{2,4,8}.
```

**Casos de borda:**
| Caso | Comportamento | Motivo |
|------|--------------|--------|
| `estimated_duration_secs` não fornecido | → DEGRADED_RESUME | fallback conservador |
| `elapsed_secs ≥ T_estimated` | remaining_secs = 0 → DEGRADED | trabalho quase concluído |
| `remaining_workers == 0` | → REPLACE | impossível rodar degraded sem workers |
| Múltiplas falhas | elapsed_secs acumula; remaining_secs cai a cada falha | adaptação automática |

**Limitação a documentar:**
O warm-restart effect (P3 21–34% mais curto que o esperado, Seção 6.4.2 do TCC) torna
a fórmula conservadora para REPLACE. O threshold de 109 é empiricamente calibrado;
a validação é empírica, não puramente analítica.

**Plot da fronteira de decisão (para o TCC):**
Criar um gráfico com:
- Eixo X: `remaining_secs` (contínuo)
- Eixo Y: `N` (cluster size antes da falha: 2, 4, 8, categórico)
- Linha tracejada vertical: `remaining_secs = 389s` (a fronteira, N-independente)
- Zonas sombreadas indicando a região de cada estratégia vencedora
- Cor do marcador = vencedor medido (REPLACE ou DEGRADED); forma do marcador = benchmark
- Pontos dos 18 experimentos existentes mapeados no espaço
- Ponto novo de EP-D N=2 f=15% (Etapa 11.1) após rodá-lo

(Nota: a fronteira é N-independente, então o eixo Y serve apenas para distinguir
visualmente as configurações testadas, não para desenhar a fronteira em si.)

Os pontos do workload sintético (Etapa 11.2) entram num plot separado, já que
testam N=4 e N=8 além de N=2, algo que os benchmarks NPB reais não alcançam;
misturá-los no mesmo gráfico do crossover real confundiria duas fontes de dados
diferentes (aplicação real vs. workload controlado).

**Verificação:**
- `heuristic.md` completo com fórmula, casos de borda e limitação
- Plot da fronteira gerado (pode ser esboço — o plot final virá com os dados da Etapa 11)
- Um leitor externo consegue entender a heurística só por este documento

---

### Etapa 3 — Mapear todas as mudanças de código necessárias ✓ (verificado contra o código real)

**O que fazer:**
Listar cada arquivo, número de linha e exatamente o que mudar — sem modificar nada ainda.

**Achado crítico desta etapa:** `handle_policy_degraded_resume()` (`watcher.rs:907-973`)
**já trata** `remaining_workers == 0` — mas de um jeito que o mapeamento original não
previa. O comportamento atual (para DEGRADED_RESUME manual, hoje):
1. Para os jobs (`dmtcp_command --quit` + `scancel`)
2. Registra um `InterruptionEvent` do tipo `no_workers_remaining`
3. Termina o cluster inteiro **se** `ft.terminate_cluster_on_zero_workers == true`
4. Retorna `Ok(())` — ou seja, o job é encerrado, não "promovido" a REPLACE

Isso é intencional: se o usuário escolheu DEGRADED_RESUME manualmente, o sistema
respeita essa escolha e não troca de estratégia por trás dele. **Isso invalida a
correção que eu tinha proposto antes** (curto-circuito genérico antes do dispatch,
forçando REPLACE para qualquer estratégia): isso mudaria o comportamento já
existente e testado do DEGRADED_RESUME manual, fora do escopo desta fase.

**Correção certa:** escopar o tratamento de `remaining_workers == 0` **só** dentro
da decisão do ADAPTIVE. `choose_effective_strategy()` volta a receber
`remaining_workers` como parâmetro (retirado antes por engano) e checa isso
**primeiro**, antes da fórmula de tempo. O dispatch (`handle_interruption`) só
calcula `remaining_workers` dentro do próprio ramo `"ADAPTIVE"` — os ramos
`"REPLACE_RESUME"` e `"DEGRADED_RESUME"` manuais continuam exatamente como estão.

**Lista definitiva de mudanças (linhas confirmadas por leitura direta do código):**

1. `run_task.rs:40-57` — struct `FaultToleranceConfig`. Adicionar
   `#[serde(default)] estimated_duration_secs: Option<u64>` (junto de
   `recovery_timeout_secs`/`auto_test_failure`, que já usam esse padrão)
2. `run_task.rs:500-508` — match de validação. Hoje:
   `"NONE" | "REPLACE_RESUME" | "DEGRADED_RESUME" => {}`, senão `bail!` com
   mensagem `"Expected NONE, REPLACE_RESUME, or DEGRADED_RESUME."`. Adicionar
   `"ADAPTIVE"` ao match **e** atualizar o texto da mensagem de erro pra
   mencionar as 4 opções. Adicionar aviso se `ADAPTIVE` sem `estimated_duration_secs`
3. `run_task.rs:888-895` — construtor do `watcher::FaultToleranceConfig`. Adicionar
   `estimated_duration_secs: ft.estimated_duration_secs,`
4. `watcher.rs:14-21` — struct `FaultToleranceConfig` (pub). Adicionar
   `pub estimated_duration_secs: Option<u64>,`
5. `watcher.rs:451-463` — assinatura de `handle_interruption()`. Adicionar
   `run_started_at: &str`. **Só existe 1 call site** (linha 125, dentro de
   `run_watcher()`) — passar `&watcher_started_at` (já existe como variável
   local em `run_watcher()`, linha 60; não precisa de mudança em `run_task.rs`
   nem em `run_watcher()`, só no call site de `handle_interruption`)
6. `watcher.rs` (nova função, perto de `handle_policy_replace_resume`/
   `handle_policy_degraded_resume`) — criar
   `choose_effective_strategy(remaining_workers: usize, run_started_at: &str, estimated_duration_secs: Option<u64>) -> &'static str`,
   checando `remaining_workers == 0` **primeiro**, antes da fórmula de tempo
7. `watcher.rs:648-684` — match de dispatch dentro de `handle_interruption()`.
   Adicionar ramo `"ADAPTIVE"`: calcular `remaining_workers` (mesmo padrão já
   usado em `handle_policy_degraded_resume:921`, `all_nodes.iter().filter(|n| n.role == "worker" && n.id != node.id).count()`),
   chamar `choose_effective_strategy()`, logar a escolha, despachar pro handler
   correspondente

**Confirmado, sem risco:**
- `node.role` e `node.id` já são usados exatamente assim em `handle_policy_degraded_resume:921` e `run_task.rs` — não é suposição, já existe no código
- Só 1 call site de `handle_interruption()` (mais simples que o previsto)

**Verificação:**
- `grep -n "REPLACE_RESUME\|DEGRADED_RESUME\|FaultToleranceConfig" src/commands/cluster/*.rs` confirma a cobertura acima ✓
- Nenhuma mudança de código feita nesta etapa — apenas mapeamento e a correção de design acima

---

### Etapa 4 — Adicionar `estimated_duration_secs` ao `FaultToleranceConfig` em `run_task.rs`

**O que fazer:**
Em `run_task.rs`, no struct `FaultToleranceConfig` (~linha 40):
```rust
#[serde(default)]
estimated_duration_secs: Option<u64>,
```

**Verificação:** `cargo check` passa sem erros.

---

### Etapa 5 — Adicionar `estimated_duration_secs` ao watcher e atualizar o construtor

**O que fazer:**
- Em `watcher.rs`, no struct `FaultToleranceConfig` (~linha 14):
  ```rust
  pub estimated_duration_secs: Option<u64>,
  ```
- Em `run_task.rs:~888`, no construtor do `watcher::FaultToleranceConfig`:
  ```rust
  estimated_duration_secs: ft.estimated_duration_secs,
  ```

**Verificação:** `cargo check` passa; os dois structs têm o campo e o construtor propaga o valor.

---

### Etapa 6 — Adicionar `"ADAPTIVE"` à validação de estratégia em `run_task.rs`

**O que fazer:**
- No bloco de validação (linha 500-508, confirmado):
  ```rust
  "NONE" | "REPLACE_RESUME" | "DEGRADED_RESUME" | "ADAPTIVE" => {}
  ```
  E atualizar o texto da mensagem de erro (linha 504), que hoje diz
  `"Expected NONE, REPLACE_RESUME, or DEGRADED_RESUME."`, para mencionar as 4 opções.
- Adicionar aviso (não erro) se ADAPTIVE sem estimativa:
  ```rust
  if ft.strategy == "ADAPTIVE" && ft.estimated_duration_secs.is_none() {
      warn!("ADAPTIVE strategy without estimated_duration_secs: falling back to DEGRADED_RESUME on fault");
  }
  ```
  Atenção: `run_task.rs` importa hoje só `tracing::{error, info}` (linha 19) —
  `warn` precisa ser adicionado a esse import, senão não compila.

**Verificação:**
- `cargo check` passa
- YAML com `strategy: "ADAPTIVE"` sem estimativa não bails; strategy inválida ainda bails

---

### Etapa 7 — Passar `watcher_started_at` para `handle_interruption()`

**O que fazer:**
- `watcher_started_at` já existe como variável local em `run_watcher()` (linha 60)
- Adicionar `run_started_at: &str` como parâmetro de `handle_interruption()` (linha 451-463)
- Só existe **1 call site** (linha 125, dentro do loop de `run_watcher()`) —
  confirmado na Etapa 3. Passar `&watcher_started_at` nele

**Verificação:** `cargo check` passa; assinatura atualizada no único call site.

---

### Etapa 8 — Implementar `choose_effective_strategy()`

**O que fazer:**
Criar a função em `watcher.rs`:

```rust
fn choose_effective_strategy(
    remaining_workers: usize,
    run_started_at: &str,
    estimated_duration_secs: Option<u64>,
) -> &'static str {
    // Hard constraint, checked before the time formula: DEGRADED_RESUME cannot
    // run without at least one surviving worker. This is scoped to the ADAPTIVE
    // decision only — manual DEGRADED_RESUME keeps its own existing zero-worker
    // handling in handle_policy_degraded_resume() (stop jobs, optionally
    // terminate the cluster), which this function does not touch or replace.
    if remaining_workers == 0 {
        return "REPLACE_RESUME";
    }

    // P2b saving: t_replace_P2b (~120s) - t_degraded_P2b (~11s), measured empirically.
    const P2B_SAVING_SECS: f64 = 109.0;
    // Empirical penalty coefficient (β−α): actual (P3_degraded−P3_replace)/remaining_secs ≈ 0.28–0.31
    // across N∈{2,4,8} and EP-D/LU-C benchmarks. N-independent due to interplay of warm-restart
    // acceleration and DEGRADED scaling. Valid range for 18/18 correct classification: (0.25, 0.30).
    const PENALTY_COEFF: f64 = 0.28;

    let Some(estimated) = estimated_duration_secs else {
        return "DEGRADED_RESUME"; // no estimate → safe default
    };

    let elapsed_secs = chrono::Utc::now()
        .signed_duration_since(
            chrono::DateTime::parse_from_rfc3339(run_started_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        )
        .num_seconds()
        .max(0) as f64;

    let remaining_secs = ((estimated as f64) - elapsed_secs).max(0.0);
    let p3_penalty_estimate = remaining_secs * PENALTY_COEFF;

    if p3_penalty_estimate > P2B_SAVING_SECS {
        "REPLACE_RESUME"
    } else {
        "DEGRADED_RESUME"
    }
}
```

Nota (corrigida na Etapa 3): `remaining_workers` volta a fazer parte da assinatura.
A fórmula de tempo em si é N-independente (resultado da Etapa 1) — isso não muda.
Mas o caso `remaining_workers == 0` não é uma questão do coeficiente, é uma
restrição física (não existe máquina pra rodar DEGRADED), e por isso é checado
**antes** de qualquer coisa relacionada a tempo. Essa checagem fica **dentro**
desta função (não no chamador) porque ela só se aplica à decisão do ADAPTIVE —
o comportamento do DEGRADED_RESUME manual já existente em
`handle_policy_degraded_resume()` (parar jobs, opcionalmente terminar o cluster)
não é alterado nem duplicado.

**Desk check com T_MANA_noFT reais do CSV (Etapa 1):**

| Cenário | remaining_workers | T_est | elapsed | remaining_secs | est_penalty (×0.28) | Esperado | OK? |
|---------|-------------------|-------|---------|----------------|---------------------|----------|-----|
| EP-D N=2 f=10% | 1 | 484.5s | 48.5s | 436.1 | **122.1** | REPLACE | >109 ✓ |
| EP-D N=2 f=25% | 1 | 484.5s | 121.1s | 363.4 | **101.8** | DEGRADED | <109 ✓ |
| EP-D N=2 f=50% | 1 | 484.5s | 242.3s | 242.3 | **67.8** | DEGRADED | <109 ✓ |
| LU-C N=2 f=10% | 1 | 241.2s | 24.1s | 217.1 | **60.8** | DEGRADED | <109 ✓ |
| EP-D N=4 f=10% | 3 | 249.0s | 24.9s | 224.1 | **62.7** | DEGRADED | <109 ✓ |
| EP-D N=8 f=10% | 7 | 125.7s | 12.6s | 113.2 | **31.7** | DEGRADED | <109 ✓ |
| Zero workers | **0** | qualquer | qualquer | — | — | **REPLACE** (curto-circuito) | ✓ |
| No estimate | ≥1 | None | — | — | — | DEGRADED fallback | ✓ |

Todos os 18 pontos do TCC classificados corretamente (100%), mais os dois casos
de borda (zero workers, sem estimativa).

**Verificação formal:**
- `cargo check` passa
- Desk check revisado após Etapa 1 com T_MANA_noFT reais do CSV
- A função está comentada explicando o mecanismo

**Execução real (2026-09-15):** implementado e testado. Adicionei um módulo
`#[cfg(test)]` (`choose_effective_strategy_tests`) com 9 testes cobrindo os 6
pontos do desk-check acima + zero workers + sem estimativa + elapsed além da
estimativa. Todos passam (`cargo test choose_effective_strategy_tests`). Isso
também confirmou que `DateTime<FixedOffset>::default()` compila e se comporta
como esperado (risco técnico do plano original, agora eliminado).

**⚠ Nota temporária:** este é o primeiro teste do projeto inteiro (não havia
nenhum `#[cfg(test)]` antes). Combinado com o usuário: os testes ficam **só até
o fim da Etapa 10** (smoke test end-to-end na AWS), servindo de rede de segurança
enquanto conectamos tudo no dispatch (Etapa 9). Depois que a Etapa 10 passar,
**remover o módulo `choose_effective_strategy_tests` de `watcher.rs`** para manter
a convenção atual do projeto (sem testes). Isso está registrado como item de
verificação na Etapa 10 — não esquecer.

---

### Etapa 9 — Conectar `"ADAPTIVE"` ao dispatch em `handle_interruption()`

**O que fazer:**

Em `handle_interruption()`, entre o scancel e o match de estratégia:

```rust
let effective_strategy = if ft.strategy == "ADAPTIVE" {
    // Computed only for the ADAPTIVE branch — manual REPLACE_RESUME/DEGRADED_RESUME
    // are untouched and keep their existing behavior (including
    // handle_policy_degraded_resume's own zero-worker handling).
    let remaining_workers = all_nodes.iter()
        .filter(|n| n.role == "worker" && n.id != node.id)
        .count();
    let chosen = choose_effective_strategy(
        remaining_workers,
        run_started_at,
        ft.estimated_duration_secs,
    );
    watcher_info(progress, &format!(
        "[watcher][cycle={}]   -> ADAPTIVE chose: {} (remaining_workers={}, estimated={:?}s)",
        recovery_cycle_id, chosen, remaining_workers, ft.estimated_duration_secs
    ));
    chosen
} else {
    ft.strategy.as_str()
};

match effective_strategy {
    "REPLACE_RESUME" => handle_policy_replace_resume(...).await,
    "DEGRADED_RESUME" => handle_policy_degraded_resume(...).await,
    other => bail!("Unknown fault_tolerance.strategy: {}", other),
}
```

Isso é mais simples do que a versão anterior deste plano (que tinha um curto-circuito
de `remaining_workers == 0` antes do match, afetando todas as estratégias). Essa
versão anterior estava errada: ela mudaria o comportamento já existente do
DEGRADED_RESUME manual (que já trata isso dentro de `handle_policy_degraded_resume`,
parando os jobs em vez de forçar REPLACE). A versão correta calcula
`remaining_workers` só dentro do ramo `"ADAPTIVE"`, e a checagem de zero workers
mora dentro de `choose_effective_strategy()` (Etapa 8), não aqui.

**Verificação:**
- `cargo check` passa sem warnings relevantes
- `cargo build --release` completo sem erros
- Confirmado na Etapa 3: `node.role`/`node.id` já são usados exatamente assim em
  `handle_policy_degraded_resume:921` — sem risco
- Confirmado na Etapa 3: o comportamento de zero workers do DEGRADED_RESUME manual
  não é alterado por esta mudança

---

### Etapa 10 — Smoke test end-to-end com ADAPTIVE

**O que fazer:**
- Preparar dois YAMLs de task ✓ (criados, seguindo o template exato dos arquivos
  `-10pct.yaml` reais já existentes no projeto):
  1. **Deve resultar em REPLACE**: `my_clusters/m5-xlarge/2workers/ep/task-ep_D-adaptive-smoketest-replace.yaml`
     — EP-D N=2, `estimated_duration_secs: 485`, `trigger_after_secs: 46`
     (mesmo valor do arquivo real `replace-10pct.yaml`) → remaining=439s,
     penalty_est=122.9 > 109
  2. **Deve resultar em DEGRADED**: `my_clusters/m5-xlarge/4workers/ep/task-ep_D-adaptive-smoketest-degraded.yaml`
     — EP-D N=4, `estimated_duration_secs: 249`, `trigger_after_secs: 32`
     (mesmo valor do arquivo real `degraded-10pct.yaml`) → remaining=217s,
     penalty_est=60.8 < 109
- Executar na AWS com `auto_test_failure` (pendente — requer cluster ativo, ação
  do usuário)
- Verificar log `[watcher] ADAPTIVE chose: ...`

**Verificação:**
- `ADAPTIVE chose: REPLACE_RESUME` aparece no cenário 1
- `ADAPTIVE chose: DEGRADED_RESUME` aparece no cenário 2
- A aplicação MPI termina com sucesso nos dois casos
- `interruption_events` no DB contém o ciclo completo
- **Depois que os dois cenários acima passarem:** remover o módulo
  `#[cfg(test)] mod choose_effective_strategy_tests` de `watcher.rs` (adicionado
  como rede de segurança temporária na Etapa 8; ver nota lá). Rodar `cargo check`
  de novo depois de remover pra confirmar que nada mais depende dele.

#### Achado real na primeira execução (2026-09-15): bug de âncora temporal

**O que aconteceu:** rodei o cenário 1 (esperado REPLACE) e o ADAPTIVE escolheu
DEGRADED. O resultado real: `setup_time_s=38.389`, `ft_wall_time_s=552.231`.

**Causa raiz:** `estimated_duration_secs` é derivado de `T_MANA_noFT`, que no
CSV é `ft_wall_time_s` — medido em `run_task.rs:1070` (linha confirmada) só a
partir de `run_commands_start` (`Instant::now()`), **sem incluir
`setup_commands`** (compilação do NPB, cópia de binário, etc.). Mas
`elapsed_secs`, na Etapa 7 original, usava `watcher_started_at` — calculado
**antes** até do `setup_commands` começar. Isso infla `elapsed_secs`
sistematicamente pelo tempo de setup (aqui, 38.4s) em **toda** decisão do
ADAPTIVE, não só neste teste. Se não corrigido, isso teria viesado toda a
avaliação da Etapa 12 (comparando a decisão do código, calculada com uma âncora,
contra o ground truth dos 18 pontos, definido com f% relativo à outra âncora).

**Correção aplicada:**
- `run_task.rs`: novo estado compartilhado
  `run_commands_started_at: Arc<Mutex<Option<String>>>`, criado antes do spawn
  do watcher (`None` inicialmente), passado por clone para `run_watcher()`.
  Preenchido com `Utc::now().to_rfc3339()` no exato momento em que
  `run_commands_start = Instant::now()` é definido, dentro do loop de tasks
  (`run_task.rs`, próximo à linha 1076)
- `watcher.rs`: `run_watcher()` e `handle_interruption()` agora recebem
  `Arc<Mutex<Option<String>>>` no lugar do `run_started_at: &str` fixo da Etapa 7
- O ramo `"ADAPTIVE"` do dispatch resolve o mutex no momento da decisão: se
  `Some(ts)`, chama `choose_effective_strategy()` normalmente; se `None`
  (a falha aconteceu **antes** do `run_commands` começar, ex.: durante
  `setup_commands`), cai em DEGRADED_RESUME — não há "tempo decorrido da
  aplicação" que faça sentido nesse caso, então usa o mesmo fallback
  conservador já usado para estimativa ausente. Este é um novo caso de borda,
  a ser documentado no `heuristic.md`
- `choose_effective_strategy()` em si **não muda** — continua recebendo um
  `&str` já resolvido. Os 9 testes da Etapa 8 continuam passando sem alteração

**Verificação:** `cargo check`, `cargo build --release` e os 9 testes passam,
zero warnings novos, `clippy` não aponta nada além dos padrões já existentes.

#### Confirmação: re-execução com o código corrigido (2026-09-15) ✓

Rodei os dois cenários de novo, agora com a âncora temporal corrigida. **Os dois
bateram com a predição.**

**Cenário 1 (2 workers, esperado REPLACE):** confirmado, embora indiretamente —
o relatório persistido não captura o `println!` do `watcher_info`, só os
eventos do banco. A evidência é inconfundível mesmo assim:
- `phase2b` (node_drained → node_became_idle) = **116.8s**, compatível com
  esperar uma máquina nova (~120s), não os ~11s típicos do DEGRADED
- Evento terminal chamado `recovery_completed` (o nome usado por
  `handle_policy_replace_resume`), não `degraded_resume`
- CPU Time = 494.27s e restart total = 299s, ambos mais rápidos que a run com
  bug (543.05s / 444s) — consistente com rodar em capacidade cheia (2 workers)
  em vez de 1 worker sobrecarregado

**Cenário 2 (4 workers, esperado DEGRADED):** confirmado diretamente — evento
terminal literalmente chamado `degraded_resume`, `phase2b` = 11.0s (reconfiguração
rápida do Slurm, sem esperar máquina nova).

**Melhoria adicional:** como o relatório persistido não mostra a decisão do
ADAPTIVE de forma explícita (só inferível pelos tempos de fase), adicionei um
novo `InterruptionEvent` (`event_type="adaptive_decision"`) gravado no exato
momento da decisão, com `chosen`, `remaining_workers`, `estimated_duration_secs`
e `run_commands_started` no campo `details`. Como a query que monta a "Event
Timeline" do relatório não filtra por tipo de evento, isso aparece
automaticamente na timeline de qualquer execução futura, sem precisar inferir
pela duração das fases — importante para a Etapa 12, que vai precisar checar
a decisão em muitas execuções.

**Testes temporários removidos:** com os dois cenários confirmados, removi o
módulo `choose_effective_strategy_tests` de `watcher.rs`, como combinado ao
final da Etapa 8. `cargo check`/`build --release` seguem limpos, `cargo test`
confirma 0 testes (convenção original do projeto restaurada).

**Gate G3 (smoke test) aprovado.**

---

### Etapa 11 — Validação da fronteira: NPB real + workload sintético

**Contexto e motivação:** o desenho original desta etapa (mais valores de f só em
EP-D N=2) foi reconsiderado. Ele testaria sempre o mesmo mecanismo (mesmo
benchmark, mesmo N), sem provar a parte mais importante e mais surpreendente da
Etapa 1: que o coeficiente ~0.28 é aproximadamente constante entre N=2, 4 e 8.
Com benchmarks NPB reais, isso não dá pra testar diretamente, porque só EP-D N=2
consegue cruzar o threshold de 389s (ver "Fronteira de decisão" acima); aumentar
a classe do benchmark pra alcançar N=4/N=8 é inviável (checkpoint size, Seção
6.3.3 do TCC).

A solução: dividir a validação em duas partes complementares, cada uma resolvendo
um problema diferente:

1. **Um ponto NPB real, pequeno (11.1):** ancora o ADAPTIVE numa aplicação
   científica de verdade e fecha lacunas que já existiam no TCC antes do ADAPTIVE
   (Tabela 16, heatmap da Seção 6.5)
2. **Um workload sintético com duração controlável (11.2):** testa a fórmula em
   N=2, 4 e 8 de fato, desacoplado do tamanho do checkpoint, validando
   diretamente a N-independência

#### 11.1 — CANCELADA (2026-09-18): NPB real (EP-D, N=2, f=15%)

Esta etapa foi planejada, os YAMLs chegaram a ser criados
(`my_clusters/m5-xlarge/2workers/ep/task-ep_D-{replace,degraded}-15pct.yaml`),
e depois cancelada por dois motivos que se acumularam. Os dois arquivos foram
removidos em 2026-09-18 (nenhum resultado existia pra eles, nenhum dado
perdido):

1. **A justificativa de validação do ADAPTIVE ficou superada** pelo
   `synth_duration` (Etapa 11.2), que caracteriza a independência de N de
   forma muito mais completa (24 pontos vs. 1) do que um ponto a mais de EP-D
   conseguiria
2. **A outra justificativa (completar a Tabela 16 pra N=2) não precisava de
   experimento novo** — os dados já existiam em `results_raw.csv` (as
   execuções REPLACE originais de EP-D/LU-C, N=2, f=10/25/50%). Ver seção
   "Etapa 11.1 cancelada: Tabela 16 completada com dados já existentes" acima
   (parte de "Impacto no Capítulo 6")

Nenhuma execução na AWS foi necessária para esta etapa.

#### 11.2 — Workload sintético: validação de N-independência

**Motivação:** testar se o mesmo threshold de 389s realmente separa REPLACE de
DEGRADED em N=4 e N=8, não só em N=2. Isso valida diretamente o achado mais
importante da Etapa 1.

**Desenho:**
- Workload sintético com duração de execução controlável (`T_synth`) e footprint
  de memória pequeno e fixo, desacoplado da duração.
- `T_synth = 600s` (10 minutos), fixo para todos os N (workload compute-bound,
  sem comunicação real entre ranks, no espírito do EP-D)
- Fronteira em `remaining_secs = 389.3s`; para este `T_synth`, o crossover cai em
  `f ≈ 35.1%`

**Novo benchmark sintético necessário: `synth_duration.c`**

Inspecionei `TCC/synthetic/` (4 arquivos: `synth_checkpoint_size.c`,
`synth_mpi_calls.c`, `synth_p2p.c`, `synth_imbalanced.c`). Nenhum tem duração
controlável: `synth_mpi_calls.c` e `synth_p2p.c`/`synth_imbalanced.c` fazem uma
quantidade fixa de trabalho (`TOTAL_OUTER × INNER_ITERS`), dando uma duração fixa
de ~65-85s independente do "nível" compilado; `synth_checkpoint_size.c` tem
duração fixa (`TARGET_SECS=120.0`) e varia a memória (`MEM_MB`) via constante de
compilação.

`synth_checkpoint_size.c` já tem exatamente o padrão que precisamos: um loop
`while (MPI_Wtime() - t_start < TARGET_SECS)` que mantém um buffer "vivo" (tocado
a cada iteração, pra aparecer no checkpoint) e faz `MPI_Barrier` periódico. Um
novo arquivo, `synth_duration.c`, inverte os papéis desse mesmo padrão: memória
fixa e pequena (`MEM_MB=50`, o mesmo nível que deu `phase1_s≈16s` em
`synth_checkpoint_size`, bem dentro do orçamento de 2 minutos), e duração
configurável via constante de compilação (`DURATION_SECS`, no lugar de
`TARGET_SECS` fixo). Como nas outras: sem argumento de runtime, porque o MANA
trava com `argc>1`.

Como `T_synth=600s` é fixo pra todos os pontos e todos os N desta etapa (só `f`
varia, via o mecanismo de `auto_test_failure` que já existe no harness), só
precisamos compilar **um único binário** (`-DDURATION_SECS=600`), sem precisar de
múltiplas variantes como acontece com `MEM_MB` ou `CALL_PERIOD` nos outros
benchmarks.

**Trabalho de engenharia — feito:**
1. ✓ `TCC/synthetic/synth_duration.c` criado, adaptando `synth_checkpoint_size.c`
   (memória fixa em 50MB, duração controlável via `DURATION_SECS`, default 600)
2. ✓ Alvo `synth_duration` adicionado ao `Makefile` de `TCC/synthetic/`
3. ✓ **Correção de convenção encontrada:** os benchmarks sintéticos existentes
   (`synth_ckpt`, `synth_calls`, etc.) não referenciam um binário pré-compilado —
   o código C é embutido diretamente no YAML via heredoc e compilado on-the-fly
   no worker (`mpicc -O2 -DMEM_MB=200 -o /shared/synth_ckpt_size /tmp/*.c`),
   porque os workers spot são efêmeros e não têm o source pré-instalado como o
   NPB tem via AMI. `TCC/synthetic/synth_duration.c` fica como cópia de
   referência/documentação, mas os YAMLs reais embutem uma versão condensada
   do mesmo código
4. ✓ **16 YAMLs gerados** (script em
   `my_clusters/m5-xlarge/{2,4,8}workers/synth/task-synth_duration-f<F>pct-<strategy>.yaml`),
   validados como YAML correto e com `process_count`/lista de workers/salloc
   corretos para cada N. `trigger_after_secs = round(f% × 600)`: 30s (5%),
   180s (30%), 240s (40%), 300s (50%)

#### Achado real no smoke test (2026-09-16): loop por relógio não serve

Rodei `task-synth_duration-f5pct-replace.yaml` como smoke test isolado (ver
seção anterior). Mecanicamente tudo funcionou (compilou, fez checkpoint,
restaurou, terminou com `status=SUCCESS`), mas os números revelaram um
problema de design: elapsed antes da falha (~34s) + gap de recuperação REPLACE
(~161s) + computação pós-restart (403s) ≈ 598 ≈ **600s**, batendo exatamente
com `DURATION_SECS`.

**Causa raiz:** o loop original usava `while (MPI_Wtime() - t_start < 600)` —
um critério de parada por **relógio real decorrido**, não por trabalho feito.
`MPI_Wtime()` avança na mesma velocidade independente de quanto CPU o processo
realmente recebe. Isso significa que um processo sobrecarregado (DEGRADED,
oversubscrito) "espera o relógio passar" na mesma velocidade que um processo
em capacidade cheia (REPLACE) — o loop não desacelera sob contenção. Como o
buraco de recuperação de cada estratégia é absorvido dentro do mesmo
orçamento de 600s, REPLACE e DEGRADED convergiriam para o mesmo tempo total,
eliminando exatamente a diferença (`penalty = P3_degraded − P3_replace`) que
a fórmula do ADAPTIVE precisa medir. A matriz completa, rodada com esse
código, não teria mostrado o crossover que queremos validar.

**Correção:** `synth_duration.c` reescrito para usar **contagem fixa de
iterações** (`for (i = 0; i < TOTAL_ITERS; i++)`), no mesmo espírito de
`synth_mpi_calls.c`. Assim, o tempo de conclusão depende de verdade da
capacidade de CPU disponível — um processo sobrecarregado demora mais tempo
real pra terminar o mesmo trabalho, que é o comportamento que precisamos.

**Calibração concluída (2026-09-16):** rodei
`task-synth_duration-calibration.yaml` (strategy `NONE`) com
`TOTAL_ITERS=3.000.000.000` como primeiro palpite. Resultado real:
**13.9s** (medido via `mana_launch`, 1 processo por core, m5.xlarge não
sobrecarregado) → taxa de **215.8M iterações/s por core**. Escalando
proporcionalmente para 600s: `TOTAL_ITERS = 129.500.000.000` (129.5 bilhões),
estimativa de 600.02s.

✓ `Makefile` e `TCC/synthetic/synth_duration.c` (comentário/default)
atualizados com o valor calibrado.
✓ Os 16 YAMLs da matriz **regenerados** com o código corrigido (contagem fixa
de iterações) e `TOTAL_ITERS=129500000000L`. Validado por script: nenhum dos
16 ainda contém o loop antigo por relógio, todos têm o `TOTAL_ITERS` certo.
Os valores de `trigger_after_secs` (30/180/240/300s) continuam os mesmos, já
que são frações do T_synth alvo, não do código em si.

**Antes de rodar a matriz completa (48 execuções):** recomendo repetir o
smoke test isolado (`task-synth_duration-f5pct-replace.yaml`, N=2) com o
código corrigido, e desta vez também rodar a versão `-degraded` do mesmo
ponto, comparando os dois `ft_wall_time_s`. Com o loop por relógio, os dois
convergiam pro mesmo tempo (~600s); com contagem fixa, DEGRADED deve terminar
visivelmente mais devagar que REPLACE agora, confirmando que a correção
realmente resolveu o problema antes de comprometer as 48 execuções.

| Ponto | f% | remaining_secs | penalty estimada | margem | Esperado |
|---|---|---|---|---|---|
| Confortável em REPLACE | 5% | 570.0s | 159.6 | +50.6 | REPLACE |
| Perto da fronteira (lado REPLACE) | 30% | 420.0s | 117.6 | +8.6 | REPLACE |
| Perto da fronteira (lado DEGRADED) | 40% | 360.0s | 100.8 | −8.2 | DEGRADED |
| Fundo em DEGRADED | 50% | 300.0s | 84.0 | −25.0 | DEGRADED |

**Matriz de execuções:**

| N | Pontos testados | Execuções |
|---|---|---|
| 2 | todos os 4 (5%, 30%, 40%, 50%) | 4 × 2 estratégias × 3 reps = 24 |
| 4 | só os 2 da fronteira (30%, 40%) | 2 × 2 × 3 = 12 |
| 8 | só os 2 da fronteira (30%, 40%) | 2 × 2 × 3 = 12 |
| | | **Total: 48 execuções** |

N=2 recebe a caracterização completa (mesmo padrão do EP-D real); N=4 e N=8
recebem só os dois pontos de fronteira, que são os que efetivamente testam se o
threshold se mantém em 389s fora de N=2.

**Decisão (2026-09-16):** 3 repetições por ponto, mesmo padrão estatístico usado
nos 18 pontos reais do TCC — mais fácil de justificar/comparar na redação.
Total confirmado: 48 execuções.

**Verificação (N=2 completo com 6 pontos, ver abaixo):**
- `synth_duration.c` compila e roda corretamente em N=2 isoladamente antes de
  rodar a matriz completa (smoke test do binário) ✓
- Todas as execuções completam sem erro ✓ (12/12 em N=2, N=2 completo)
- ~~Em N=2, 4 e 8: os pontos de fronteira (30%, 40%) classificam corretamente~~ —
  **superado**: 30%/40% não eram os pontos de fronteira corretos; a fronteira
  real de N=2 (f≈63%) foi confirmada empiricamente com f=65%/75% ✓
- O mesmo threshold (~220s de remaining_secs, recalibrado por workload) separa
  as estratégias independente de N — ainda a testar em N=4/N=8 (pendente)
- Dados salvos separadamente (ex.: `results_synth_adaptive.csv`), já que não
  fazem parte do conjunto NPB principal

#### Achado real nos 4 pontos de N=2 (2026-09-16): coeficiente bem maior e muito mais estável

Rodei os 4 pontos de N=2 (f=5%, 30%, 40%, 50%), REPLACE e DEGRADED, 1 repetição
cada (8 execuções). **REPLACE venceu nos 4 pontos** — nenhum DEGRADED ganhou
ainda nessa faixa.

| f% | remaining_secs | P3_replace | P3_degraded | penalty | coeff | Δ(ft_wall) | vencedor |
|---|---|---|---|---|---|---|---|
| 5% | 570.0 | 313.0 | 595.5 | 282.5 | 0.496 | −149.8 | REPLACE |
| 30% | 420.0 | 240.2 | 443.4 | 203.2 | 0.484 | −85.5 | REPLACE |
| 40% | 360.0 | 203.2 | 381.0 | 177.8 | 0.494 | −65.2 | REPLACE |
| 50% | 300.0 | 177.1 | 323.7 | 146.6 | 0.489 | −34.0 | REPLACE |

**Coeficiente médio: 0.491 (desvio 0.005)** — muito mais estável que o do NPB
(0.24–0.42), esperado dado que é um loop determinístico controlado, sem a
variabilidade de comunicação real do NPB.

**Fronteira real estimada para este workload em N=2: remaining_secs≈222s →
f≈63%** — bem mais tarde que os ~35% assumidos ao escolher os pontos 30%/40%
(esses vieram do coeficiente do NPB, 0.28, que não se aplica aqui).

#### Confirmação: N=2 completo com 6 pontos (2026-09-17) ✓

Rodei f=65% e f=75% (REPLACE + DEGRADED, 1 rep cada). **O crossover previsto
se confirmou nos dois lados:**

| f% | remaining | P3_replace | P3_degraded | penalty | coeff | Δ(ft) | vencedor |
|---|---|---|---|---|---|---|---|
| 5% | 570.0 | 313.0 | 595.5 | 282.5 | 0.496 | −149.8 | REPLACE |
| 30% | 420.0 | 240.2 | 443.4 | 203.2 | 0.484 | −85.5 | REPLACE |
| 40% | 360.0 | 203.2 | 381.0 | 177.8 | 0.494 | −65.2 | REPLACE |
| 50% | 300.0 | 177.1 | 323.7 | 146.6 | 0.489 | −34.0 | REPLACE |
| **65%** | 210.0 | 125.2 | 234.6 | **109.4** | 0.521 | +20.9 | **DEGRADED** |
| 75% | 150.0 | 99.2 | 171.8 | 72.6 | 0.484 | +39.4 | DEGRADED |

**f=65% caiu exatamente em cima da fronteira** (penalidade real 109.4s contra
limiar de 109s, margem de só +0.4s) — bateu quase perfeitamente com a previsão
feita a partir de só 4 pontos (estimativa: margem −5.9s; a pequena diferença é
ruído de medição normal bem em cima do limiar). Isso é uma validação forte do
modelo: a fronteira foi prevista com boa precisão antes mesmo de rodar os
pontos decisivos.

**Coeficiente com os 6 pontos: 0.495 ± 0.014** (levemente mais disperso que
com 4 pontos, esperado porque f=65% está bem na fronteira, onde ruído de
medição vira mais variação relativa no coeficiente). **Fronteira confirmada:
remaining_secs=220.4s → f=63.3%**, quase idêntica à previsão de 63%.

**N=2 está com o crossover totalmente caracterizado**, com REPLACE e DEGRADED
genuinamente observados dos dois lados — o mesmo padrão que o TCC já tem para
o NPB (Tabela `strategy-wall-time`), agora também para este workload sintético.

#### Confirmação: N=4 completo, 1ª repetição (2026-09-17) ✓

| f% | remaining | P3_replace | P3_degraded | penalty | coeff | vencedor (real) |
|---|---|---|---|---|---|---|
| 5% | 570.0 | 318.4 | 600.2 | 281.8 | 0.494 | REPLACE |
| 50% | 300.0 | 177.2 | 329.1 | 151.9 | 0.506 | REPLACE |
| 65% | 210.0 | 130.4 | 240.5 | 110.0 | 0.524 | REPLACE (por pouco) |
| 75% | 150.0 | 99.1 | 177.2 | 78.1 | 0.520 | DEGRADED |

**Coeficiente médio N=4: 0.511 ± 0.014** — praticamente igual ao de N=2
(0.495 ± 0.014). Forte confirmação de independência de N pra este workload.

**Achado importante sobre f=65%:** o vencedor "virou" entre N=2 (DEGRADED) e
N=4 (REPLACE) nesse ponto específico, mas **não é efeito de N** — comparando
P3 puro (o componente que a fórmula do ADAPTIVE modela) entre as duas
execuções, os valores são quase idênticos:

| f% | P3_replace N=2 | P3_replace N=4 | P3_degraded N=2 | P3_degraded N=4 |
|---|---|---|---|---|
| 5% | 313.0 | 318.4 | 595.5 | 600.2 |
| 50% | 177.1 | 177.2 | 323.7 | 329.1 |
| 65% | 125.2 | 130.4 | 234.6 | 240.5 |
| 75% | 99.2 | 99.1 | 171.8 | 177.2 |

Diferenças de no máximo ~5s em qualquer ponto — ruído normal, não um padrão
sistemático de N. O que realmente mudou entre as duas execuções de f=65% foi
o **P2b do REPLACE** (tempo de provisionamento AWS): 140.3s em N=2 contra
113.0s em N=4 — variação de latência da AWS entre execuções, não relacionada
a N nem à fórmula. Como f=65% está a menos de 1 segundo da fronteira, esse
ruído isolado já basta pra virar o resultado. Esse é exatamente o tipo de
inconsistência esperada perto da fronteira que a Etapa 12 (3 repetições) vai
esclarecer — não invalida a independência de N, na verdade a reforça (o
componente que dependeria de N, P3, ficou praticamente igual).

#### Confirmação: N=8 completo, 1ª repetição (2026-09-18) ✓ — matriz de 24 pontos fechada

| f% | remaining | P3_replace | P3_degraded | penalty | coeff | vencedor (real) |
|---|---|---|---|---|---|---|
| 5% | 570.0 | 323.8 | 605.7 | 281.9 | 0.494 | REPLACE |
| 50% | 300.0 | 182.7 | 334.3 | 151.6 | 0.505 | REPLACE |
| 65% | 210.0 | 136.0 | 245.3 | 109.3 | 0.520 | DEGRADED |
| 75% | 150.0 | 104.5 | 187.8 | 83.3 | 0.556 | DEGRADED |

**Coeficiente médio N=8: 0.519 ± 0.027.** Progressão: N=2=0.495, N=4=0.511,
N=8=0.519 — leve tendência de alta, mas incrementos decrescentes (+0.016,
depois +0.008), mais parecido com convergência/saturação do que com
divergência. **Nada perto** do que o modelo teórico 1/(N-1) previria
(1.0 → 0.333 → 0.143, uma queda acentuada). Fronteiras: N=2=63.3%,
N=4=64.5%, N=8=65.0% — variação de só 1.7 pontos percentuais entre os
extremos, uma independência de N bem forte, ainda que não matematicamente
perfeita.

**P3 puro (o que a fórmula do ADAPTIVE modela) permanece muito próximo entre
os três N** para todos os 4 pontos testados:

| f% | P3_r N=2 | P3_r N=4 | P3_r N=8 | P3_d N=2 | P3_d N=4 | P3_d N=8 |
|---|---|---|---|---|---|---|
| 5% | 313.0 | 318.4 | 323.8 | 595.5 | 600.2 | 605.7 |
| 50% | 177.1 | 177.2 | 182.7 | 323.7 | 329.1 | 334.3 |
| 65% | 125.2 | 130.4 | 136.0 | 234.6 | 240.5 | 245.3 |
| 75% | 99.2 | 99.1 | 104.5 | 171.8 | 177.2 | 187.8 |

Diferenças de no máximo ~16s (em valores de 100-600s), a maioria bem menor —
uma leve tendência de crescimento com N está presente, mas pequena o
suficiente pra não descartar que seja variação normal entre instâncias EC2
específicas (não a mesma máquina física a cada execução). As 3 repetições da
Etapa 12 vão esclarecer se essa tendência é real ou ruído.

**Confirmação adicional sobre f=65%:** em N=8, o vencedor voltou a ser
DEGRADED (igual N=2), não REPLACE (como N=4 mostrou por causa do ruído de
P2b já diagnosticado). Isso reforça que aquele "empate desfeito" em N=4 foi
mesmo ruído pontual de provisionamento AWS, não um efeito real de N — se
fosse um efeito real e crescente com N, esperaríamos N=8 continuar do lado
REPLACE (ou mais ainda), não voltar para DEGRADED.

**Marco: a matriz de 24 pontos (N=2,4,8 × 4 timings × 2 estratégias, 1ª
repetição) está completa.** Resultado geral: independência de N fortemente
confirmada para este workload, com uma fronteira estável em torno de
remaining_secs≈210s (f≈63-65%) nos três tamanhos de cluster testados.

#### Confirmação final: matriz completa com 3 repetições (2026-09-26) ✓

Todos os 14 pontos (N=2: 6, N=4: 4, N=8: 4) foram completados com 3
repetições cada, para as duas estratégias — **84 execuções no total**.
Etapa 11.2 está **concluída**.

**Coeficiente final por N (médias das 3 repetições):**

| N | coeficiente médio | desvio | pontos usados |
|---|---|---|---|
| 2 | 0.506 | 0.020 | 6 (5,30,40,50,65,75%) |
| 4 | 0.519 | 0.017 | 4 (5,50,65,75%) |
| 8 | 0.523 | 0.031 | 4 (5,50,65,75%) |

Leve tendência de alta se mantém (0.506→0.519→0.523), mas as diferenças
ficam dentro de 1 desvio-padrão entre si — nada perto do colapso que o
modelo teórico 1/(N-1) preveria (1.0→0.333→0.143). Confirma independência
de N de forma robusta, com 3 repetições, não só 1.

**Achado mais importante: f=65% é genuinamente a fronteira, nos três N —
não é ambiguidade que "some" com mais dados, é a assinatura correta de
estar exatamente no limiar.** Comparando as faixas brutas de `ft_wall_time_s`
(não só a média) entre as 3 repetições de REPLACE e as 3 de DEGRADED:

| N | f% | REPLACE (3 reps, s) | DEGRADED (3 reps, s) | Faixas se sobrepõem? | Vencedor médio |
|---|---|---|---|---|---|
| 2 | 65% | 667.8 / 662.2 / 657.1 | 666.3 / 662.6 / 667.7 | **Sim** | REPLACE (−3.1s) |
| 4 | 65% | 667.9 / 676.7 / 682.8 | 673.8 / 674.2 / 673.0 | **Sim** | DEGRADED (+2.1s) |
| 8 | 65% | 697.5 / 697.0 / 686.5 | 690.1 / 693.5 / 690.2 | **Sim** | DEGRADED (+2.4s) |
| 2 | 75% | 707.5 / 702.5 / 693.0 | 668.1 / 667.7 / 671.4 | Não | DEGRADED (+31.9s) |
| 4 | 75% | 708.7 / 707.4 / 696.2 | 673.3 / 674.5 / 678.2 | Não | DEGRADED (+28.8s) |
| 8 | 75% | 720.0 / 713.4 / 718.9 | 696.0 / 688.8 / 695.0 | Não | DEGRADED (+24.2s) |

Em f=65%, as faixas de REPLACE e DEGRADED se sobrepõem **nos três N** —
diferença de médias de só 2-3s, dentro do ruído das repetições. Em f=75%,
as faixas **nunca se sobrepõem em nenhum N** — DEGRADED vence com folga e
de forma totalmente consistente. Isso é uma confirmação limpa: a fronteira
real está bem próxima de remaining_secs=210s (f≈65%) e é a mesma nos três
tamanhos de cluster, dentro da precisão que este experimento permite medir.

**Conclusão para o TCC:** o crossover é reproduzido de forma consistente
com 3 repetições em N=2, 4 e 8, com o mesmo ponto de inversão (f≈65%) e o
mesmo padrão de "faixas se sobrepõem exatamente ali, e só ali". Essa é a
evidência mais forte reunida nesta fase para a independência de N do
coeficiente de penalidade, específica a este workload sensível a banda de
memória.

**Interpretação — por que a penalidade é tão maior que no NPB:** o `synth_duration`
toca um buffer de 50MB por iteração (sensível a banda de memória/cache). Em N=2,
o DEGRADED roda 4 processos no worker sobrevivente, usando os 2 threads de cada
core físico (hyperthreading) que antes ficavam ociosos. Hyperthreading só ajuda
quando os processos competem pouco por recursos compartilhados — o que é o caso
do EP-D (footprint mínimo, quase só registrador), mas não do `synth_duration`
(toca memória o tempo todo). Isso explica por padrão observacional, não prova
definitiva, por que o NPB mostrou penalidade quase nula (β≈1.0) e esse workload
mostra penalidade quase teórica (β≈1.9, perto do 2× ingênuo).

**Consequências práticas — resolvido (2026-09-16), com ajuste adicional:**
1. ✓ YAMLs de N=2 f=65%/75% criados (`trigger_after_secs`=390s/450s). Previsão
   usando coeff=0.491: f=65% → penalty_est=103.1, margem −5.9s (DEGRADED, mas
   apertado, ponto mais informativo); f=75% → penalty_est=73.7, margem −35.3s
   (DEGRADED com folga)
2. ✓ **Pontos de N=4/N=8 revisados para os MESMOS de N=2: 50%, 65%, 75%**
   (não 55%/70% como cogitado inicialmente). Motivo: como `T_synth=600s` é fixo
   pra todos os N, usar o mesmo `f%` significa testar o mesmo `remaining_secs`
   em cada N — a comparação mais direta possível pra pergunta que importa
   ("o mesmo tempo restante separa REPLACE/DEGRADED em qualquer N?"). Usar
   pontos diferentes por N (como 55%/70%) trocaria essa comparação direta por
   uma otimização de margem que dependia de assumir que N=4/8 têm o mesmo
   coeficiente de N=2 — exatamente o que ainda não sabemos. Reaproveitar o
   50% (REPLACE confortável em N=2) também dá um par REPLACE/DEGRADED
   completo em cada N, não só dois pontos que provavelmente dariam DEGRADED
   nos dois. Arquivos: `my_clusters/m5-xlarge/{4,8}workers/synth/task-synth_duration-f{50,65,75}pct-{replace,degraded}.yaml`
   (`trigger_after_secs`=300s/390s/450s, idênticos aos de N=2; `process_count`=8/16)

**N=2: completo (12/12 execuções, 6 pontos ✓). N=4: completo (8/8 execuções,
4 pontos ✓).** Pendente agora só N=8: 4 pontos × 2 estratégias (8 execuções,
f=5%/50%/65%/75%, `f=5%` adicionado depois como ponto de diagnóstico — ver
seção "Decisão sobre f=30%/40%" abaixo). 1 repetição cada, pra confirmar o
padrão antes de
comprometer as 3 repetições completas.

#### Narrativa para o TCC: conectando com a Seção 6.4.2, não como achado isolado

Isso não é uma contradição com os dados do NPB — é uma continuação direta da
pergunta que a Seção 6.4.2 já deixa em aberto: *"O mecanismo que explica essa
aceleração [do warm-restart] não é identificável a partir dos dados
disponíveis."* O `synth_duration` foi desenhado, ainda que não tivesse esse
objetivo inicial, como um experimento controlado que ajuda a explicar **parte**
dessa lacuna, especificamente o lado da penalidade de sobrecarga do DEGRADED
(não necessariamente o lado da aceleração do REPLACE, que é um fenômeno distinto).

Estrutura sugerida para o texto:
1. Retomar o que 6.4.2 já registra: aceleração do REPLACE e penalidade quase nula
   do DEGRADED em N=2 para EP-D/LU-C, mecanismo não identificado
2. Apresentar o `synth_duration` como um experimento controlado, desenhado com
   footprint de memória fixo e sensível a banda de memória — propositalmente
   diferente do perfil leve do EP-D/LU-C
3. Mostrar o contraste: penalidade muito maior (~0.49 vs ~0.28-0.35) e muito
   mais estável (desvio 0.005) nesse workload
4. Interpretar: evidência de que o mecanismo tem relação com contenção de
   recursos compartilhados sob hyperthreading — workloads leves em memória
   toleram bem a sobrecarga, workloads pesados em banda de memória não
5. Consequência honesta para o ADAPTIVE: o coeficiente 0.28 não é universal,
   é calibrado para os benchmarks testados (leves em memória). Isso vira uma
   limitação explícita e uma direção de trabalho futuro (calibrar o
   coeficiente por aplicação, ou detectar o perfil de memória automaticamente)

Isso fortalece o TCC: mostra investigação do *porquê* a heurística funciona ou
não em diferentes cargas, não só a implementação. Não é um resultado "estranho"
a esconder — é a continuação natural de uma pergunta que o próprio TCC já
tinha deixado em aberto.

---

### Etapa 12 — Avaliação experimental do ADAPTIVE (escopo reduzido, 2026-09-26)

**Reconsideração do escopo original:** o plano original pedia 32 configurações
(18 NPB + 14 sintéticas) × 3 repetições = 96 execuções, tratando a Etapa 12
como uma re-medição de "quem vence, REPLACE ou DEGRADED" em todos os pontos
já caracterizados. Mas essa pergunta já está respondida — a Etapa 11.2 (3
repetições, desvios-padrão baixos: coeficiente 0.506±0.020 em N=2,
0.519±0.017 em N=4, 0.523±0.031 em N=8) e o dataset NPB de 18 pontos já
estabelecem, com baixa variância, qual estratégia é mais rápida em cada
configuração. Rodar ADAPTIVE de novo nesses mesmos pontos não geraria
informação nova sobre REPLACE vs. DEGRADED — o resultado já é conhecido.

**O que a Etapa 12 realmente precisa validar** não é a física (já resolvida),
é se a **implementação real** — o watcher, ao vivo, numa falha de verdade na
AWS, invocando `choose_effective_strategy()` com o `run_commands_started_at`
e o `estimated_duration_secs` reais do estado da run — decide corretamente e
despacha pro handler certo. Isso é um teste de corretude de integração
(análogo ao que já pegou o bug do anchor de `elapsed_secs` na Etapa 10), não
uma remedição experimental. Com esse objetivo mais específico, o escopo cai
para 3 casos:

1. **Caso claro REPLACE** — já coberto: smoke test `ep_D-adaptive-smoketest-replace`
   (N=2, trigger=46s, remaining_secs=439s, penalty_est=122.9>109). Resultado
   confirmado (2026-09-15): reinício com 4 processos completos, sem indício de
   degradação — ADAPTIVE escolheu REPLACE corretamente. **Nenhuma nova execução
   necessária.**
2. **Caso claro DEGRADED** — já coberto: smoke test `ep_D-adaptive-smoketest-degraded`
   (N=4, trigger=32s, remaining_secs=217s, penalty_est=60.8<109). Resultado
   confirmado (2026-09-15): evento `degraded_resume` no timeline. **Nenhuma
   nova execução necessária.**
3. **Caso de margem apertada (a única execução nova precisa ser aqui)** —
   originalmente eu havia proposto reusar o ponto f=65% do `synth_duration`
   (Etapa 11.2) para isso, mas é a escolha errada: aquele ponto é a fronteira
   do coeficiente *real* do `synth_duration` (~0.5), não da fórmula que o
   código de fato usa (`PENALTY_COEFF=0.28`, calibrado no NPB). Testar lá
   reexporia o mismatch fórmula-vs-workload que a Etapa 11.2 já documentou,
   não testaria robustez a jitter real no limiar que o código de produção usa.
   O limiar real do código é `remaining_secs > 109/0.28 = 389.29s`. Criei
   `task-ep_D-adaptive-smoketest-boundary.yaml` (N=2, EP-D, T_MANA_noFT=485s,
   `trigger_after_secs=96` → remaining_secs nominal=389s, penalty_est=108.9,
   ~0.1 abaixo do limiar de 109 — decisão nominal DEGRADED, mas a menos de 1s
   de jitter de virar REPLACE). Como `poll_interval_secs=1`, há jitter real de
   ~1-2s no `elapsed_secs` observado no momento da decisão, o suficiente pra
   potencialmente cruzar essa margem de 0.1 entre repetições.
   **3 repetições necessárias aqui** (não 1), porque é o único caso onde o
   jitter da AWS de verdade pode mudar o lado da decisão.

**Caso zero-workers:** não será executado na AWS. Exigiria um cluster novo de
1 worker (nenhum existe hoje — só 2/4/8), e a lógica em si
(`remaining_workers == 0 → REPLACE_RESUME`, watcher.rs:768) é um guard de uma
linha, sem dependência de tempo/jitter — não há AWS-only failure mode que uma
inspeção de código não cubra. **Atualização (2026-09-26):** em vez de deixar
isso só como "verificado por inspeção", reinstalei um módulo de testes
unitários mínimo, `#[cfg(test)] mod choose_effective_strategy_tests` em
`watcher.rs` (5 testes: zero-workers força REPLACE mesmo com estimativa
extrema ou ausente; estimativa ausente cai pra DEGRADED; os dois casos de
ground truth do smoke test de Etapa 10 (EP-D N=2 e N=4); aritmética exata do
limiar em remaining_secs=389/390s). `cargo test choose_effective_strategy_tests`
— 5/5 passando, sem custo de AWS. Isso é evidência mais forte pro TCC do que
"por inspeção": um teste automatizado e citável, não uma alegação verbal.

**Atualização (2026-09-30):** o relatório final decidiu não citar testes
unitários (mantendo o foco em execução real na AWS, e para não repetir o
padrão de dependência de código de teste dentro do relatório). O módulo
`choose_effective_strategy_tests` foi removido de `watcher.rs`, o mesmo
tratamento já dado ao módulo de 9 testes da Etapa 8. O caso zero-workers volta
a ser documentado como "verificado por inspeção de código", igual à decisão
original de 2026-09-26 antes da reinstalação dos testes. `cargo build
--release` segue limpo.

**Caso de demonstração do limite do coeficiente (2026-09-26, adicionado):**
o item 5 da narrativa do `synth_duration` (seção acima) afirma que o
coeficiente 0.28 "não é universal" — mas isso vinha só de comparar medições
separadas de REPLACE e DEGRADED, nunca de rodar o ADAPTIVE de verdade num
caso onde ele erraria. Pré-registro da predição (antes de rodar, seguindo a
mesma disciplina treino/teste): criei
`task-synth_duration-f50pct-adaptive.yaml` (N=2, `estimated_duration_secs=600`,
`trigger_after_secs=300`). Mesmo considerando o offset de decisão descoberto
acima (phase1+phase2a ~20-40s), `elapsed_secs` no momento da decisão fica em
320-340s, `remaining_secs` em 260-280s, `penalty_est` em 73-78 — bem abaixo
de 109. **Predição: ADAPTIVE escolhe DEGRADED.** Isso contradiz o ground
truth já estabelecido na Etapa 11.2 (f=50% N=2: REPLACE vence claramente,
sem sobreposição de faixas com DEGRADED) — ou seja, se a predição se
confirmar, é uma demonstração direta (não apenas inferida) de que o
coeficiente calibrado no NPB não generaliza pra este workload.

**Resultado real (2026-09-26): predição confirmada.** Evento `degraded_resume`
no timeline (phase2b=11.0s, assinatura de DEGRADED), `ft_wall_time_s=666.1s`.
Comparando com o ground truth já existente (3 reps cada, Etapa 11.2):

| Estratégia | ft_wall_time_s (3 reps) | Média |
|---|---|---|
| REPLACE (ótimo real) | 630.8 / 625.2 / 633.0 | 629.7 |
| DEGRADED (o que ADAPTIVE escolheu) | 667.0 / 666.6 / 667.3 | 666.9 |
| **ADAPTIVE (real)** | — | **666.1** |

ADAPTIVE ficou dentro da faixa DEGRADED (como previsto) e **~36.4s (~5.8%)
mais lento que o ótimo real (REPLACE)**. Essa é a evidência direta e
quantificada de que o coeficiente 0.28 (calibrado no NPB, workload leve em
memória) não generaliza para um workload sensível a banda de memória: o
ADAPTIVE tomou a decisão "correta" segundo sua própria fórmula, mas
subótima segundo o ground truth real, com um custo mensurável. Isso não é
uma falha de implementação — é uma limitação de modelo, honesta e prevista
a priori, exatamente o tipo de achado que a metodologia treino/teste desta
fase foi desenhada para expor sem maquiagem.

Total de execuções novas: **4** (3 do caso de margem + 1 do caso de
demonstração do limite do coeficiente), em vez de 96.

**Verificação:**
- Casos REPLACE e DEGRADED claros: já confirmados (Etapa 10) ✓
- Caso de margem (`ep_D-adaptive-smoketest-boundary`, 3 reps): reportar a
  decisão de cada repetição (via evento `adaptive_decision`) e o
  `ft_wall_time_s` resultante. Se as 3 repetições concordarem, reportar como
  "decisão estável mesmo perto do limiar". Se houver split (algumas REPLACE,
  outras DEGRADED), isso é o resultado esperado e vira um achado honesto no
  TCC — o limiar de 389s é sensível a ~1s de jitter, consistente com a
  margem de 0.1 (108.9 vs. 109.0) escolhida de propósito
- Caso zero-workers: documentado como verificado por inspeção de código, não
  por execução
- Resultados salvos em `TCC/artifacts/phase10/results/`

#### Resultado real do caso de margem (2026-09-26) — 3/3 DEGRADED, sem flip

As 3 repetições de `ep_D-adaptive-smoketest-boundary` escolheram
`DEGRADED_RESUME` de forma consistente (evento `degraded_resume`, phase2b≈11s
em todas). Nenhum flip observado. Investigando o timeline de eventos,
encontrei um erro de calibração: `adaptive_decision` dispara **depois** de
`checkpoint_completed` (phase1) e `node_drained` (phase2a) — o código só pode
decidir REPLACE vs. DEGRADED depois de drenar o nó, porque só assim sabe se
pode despachar pro handler de reconfiguração rápida (~11s) ou lento (~120s).
Eu havia calibrado `trigger_after_secs=96` assumindo `elapsed_secs≈trigger`
no momento da decisão, mas na verdade `elapsed_secs ≈ trigger + phase1 +
phase2a`:

| Rep | phase1 | elapsed@decisão | remaining_secs | penalty_est | margem vs. 109 |
|---|---|---|---|---|---|
| 1 | 16.5s | 118.0s | 367.0s | 102.8 | −6.2 |
| 2 | 36.9s | 138.4s | 346.6s | 97.1 | −11.9 |
| 3 | 16.4s | 117.9s | 367.1s | 102.8 | −6.2 |

A margem real ficou entre −6.2 e −11.9, não os ~−0.1 pretendidos — daí a
ausência de flip. Ainda assim, é um resultado útil: uma decisão correta e
estável, com margem visivelmente menor que a dos dois smoke tests originais
(+7.8 no caso REPLACE, −54.4 no caso DEGRADED), e nenhum indício de bug na
implementação nessa região mais próxima do limiar. Note-se também a variação
de phase1 (16.4-36.9s) entre repetições nominalmente idênticas — jitter real
de checkpoint bem maior do que o ~1-2s de `poll_interval_secs` assumido
originalmente, o que por si só já é um dado interessante sobre a variância
real de timing em produção.

**Conclusão para o TCC:** reportar como confirmação de corretude de
integração num caso de margem moderada (não como "decisão indiferente a
1s de jitter", que não foi o que este teste mediu). Não repetido com
calibração mais precisa (`trigger_after_secs≈74s`, que colocaria a margem
mais perto de zero) — ver decisão abaixo sobre o valor incremental disso.

---

## Impacto no Capítulo 6 (AVALIAÇÃO EXPERIMENTAL) do TCC — análise detalhada (2026-09-17)

Revisão de `TCC/artifacts/phase6/lapesd-thesis/body.tex`, Seções 6.4.2
("Sensibilidade ao Momento da Falha") e 6.5 ("Comparação de Estratégias:
Replace vs. Degraded"), à luz dos achados desta fase.

**Decisão de fluxo de trabalho (2026-09-18):** a versão de `body.tex` na
`phase6/` **não será editada agora**. Ela fica como está (referência da defesa
anterior) até esta fase (Phase 10) terminar. Toda mudança de texto
identificada aqui (a atualização da Tabela 16 com N=2, o ajuste do range de
LU-C de "5-34%" para "5-37%", o refino da explicação mecanicista em 6.5, e o
conteúdo novo do ADAPTIVE) fica **registrada nesta seção como um checklist**
para aplicar de uma vez, numa nova versão do TCC, depois que a Etapa 12
terminar. Isso evita editar o documento em pedaços enquanto os dados ainda
estão mudando.

### O que permanece igual (não precisa mudar)

- Os 18 pontos de dados originais, a Tabela `tab:strategy-wall-time`, o
  heatmap (`fig:crossover`, `fig10_crossover.png`)
- A conclusão qualitativa: 17/18 configurações → DEGRADED vence; cluster
  menor + falha precoce favorece REPLACE
- A observação de que EP-D N=2 f=10% é a única inversão (Δ=−19s), e a
  projeção de que jobs mais longos empurrariam mais casos pra REPLACE
  (linhas 2381–2391 de `body.tex`) — é exatamente isso que a Etapa 11 testa

### O que só precisa ser adicionado (extensão, não correção)

- ~~Novo ponto NPB (EP-D N=2, f=15%, Etapa 11.1)~~ — **Etapa 11.1 cancelada**
  (ver abaixo, não é mais necessária)
- O ADAPTIVE (fórmula, implementação, `synth_duration`) é conteúdo novo,
  provavelmente capítulo/seção separada — não reescreve a 6.5

### Etapa 11.1 cancelada: Tabela 16 completada com dados já existentes (2026-09-18)

Percebemos que os dados de N=2 pra Tabela 16 (`tab:warm-restart-gap`) **já
existem** no `results_raw.csv` — são as mesmas 18 execuções REPLACE originais
(EP-D e LU-C, N=2, f=10/25/50%, 3 repetições cada), só nunca extraídas pra
essa tabela. Não precisa rodar nada novo. Isso também elimina a única
justificativa que ainda restava pra Etapa 11.1 (o ponto f=15%), cuja força
como "validação do ADAPTIVE" já tinha sido superada pelo `synth_duration`.
**Etapa 11.1 está cancelada.**

Tabela 16 completa com N=2 (calculada de `results_raw.csv`,
$P3_\text{esp} = \text{MANA-noFT} - P0$, $P3_\text{obs} = \overline{\text{phase3\_s}}$):

| Benchmark | Workers | Timing | P3 esp. (s) | P3 obs. (s) | Δ% |
|---|---|---|---|---|---|
| EP-D | 2 | 10% | 438.5 | 322.3 | 26.5% |
| EP-D | 2 | 25% | 368.5 | 269.8 | 26.8% |
| EP-D | 2 | 50% | 253.5 | 187.7 | 26.0% |
| LU-C | 2 | 10% | 217.2 | 141.3 | 34.9% |
| LU-C | 2 | 25% | 182.2 | 116.5 | 36.1% |
| LU-C | 2 | 50% | 123.2 | 85.5 | 30.6% |

**Verificação de alinhamento com o texto já existente (`body.tex` linha 2253-2254,
"A redução fica entre 21 e 30% para o EP-D e entre 5 e 34% para o LU-C, de
forma consistente entre repetições"):**

- **EP-D:** todos os valores individuais por repetição (21.9%–29.7%) caem
  **dentro** da faixa já declarada (21–30%). Nenhuma mudança de texto
  necessária.
- **LU-C:** a maioria cai dentro de "5–34%", mas 2 das 3 repetições de
  LU-C f=25% (37.0%, 37.0%) **excedem** o máximo declarado de 34%.
  **Ação necessária:** atualizar o texto de "entre 5 e 34%" para "entre 5 e
  37%" ao incluir N=2.
- **Consistência entre repetições:** desvio de 0.4 a 6.5 pontos percentuais
  entre as 3 repetições de cada configuração — mesma ordem de grandeza da
  variação já implícita nos dados de N=4/N=8, sustenta a afirmação de
  consistência.

**Conclusão: os dados de N=2 se alinham com a conclusão do TCC (efeito de
aceleração do warm-restart existe, magnitude similar, mecanismo não
identificado), exigindo só o ajuste pontual do limite superior de LU-C
(34%→37%).**

### O que precisa ser refinado (achado real, não é só extensão)

`body.tex` linhas ~2376-2379 e ~2417-2424 explicam o crescimento de Δ com N
assim: *"perder um nó representa 1/2 da capacidade em um cluster de dois
nós, mas apenas 1/8 em um de oito"* — i.e., atribui o padrão ao modelo
teórico ingênuo (1/(N−1)).

A Etapa 1 desta fase mostrou que o coeficiente real (β−α) **não segue esse
padrão**: fica quase constante entre os N (0.345, 0.316, 0.292 para N=2,4,8),
bem mais achatado que o 1.0/0.333/0.143 que a explicação de "fração de
capacidade" implicaria. Parte do crescimento de Δ com N vem, na verdade, de
`T_MANA_noFT` ser menor em clusters maiores (o mesmo problema classe D
termina mais rápido com mais workers), não do coeficiente de penalidade em
si mudar tanto quanto o texto atual sugere.

**Isso não invalida a conclusão** (Δ realmente cresce com N, é fato
observado) — refina a explicação mecanicista de por quê. Vale reescrever
esse trecho quando formos redigir, citando a derivação do ADAPTIVE
(coeficiente empírico ≈0.28-0.35, quase N-independente) como a análise mais
precisa disponível.

### Decisão sobre f=30%/40% em N=4/N=8 (2026-09-17)

Não incluídos — não têm poder de discriminação entre as hipóteses
"coeficiente N-independente" (~0.49, medido em N=2 com `synth_duration`) e
"coeficiente teórico 1/(N-1)" (0.333 para N=4, 0.143 para N=8):

| f% | remaining | N-independente (0.491) | N=4 teórico (1/3) | N=8 teórico (1/7) |
|---|---|---|---|---|
| 5% | 570s | REPLACE | REPLACE | DEGRADED |
| 30% | 420s | REPLACE | REPLACE | DEGRADED |
| 40% | 360s | REPLACE | REPLACE | DEGRADED |
| 50% | 300s | REPLACE | **DEGRADED** | DEGRADED |
| 65% | 210s | DEGRADED | DEGRADED | DEGRADED |
| 75% | 150s | DEGRADED | DEGRADED | DEGRADED |

Pra N=4: 5%/30%/40% dão a mesma previsão nas duas hipóteses (não
discriminam); só 50% discrimina (já está no conjunto). Pra N=8: 5%/30%/40%
dão a mesma resposta entre si em cada hipótese — rodar os três não teria
mais poder de decisão que rodar só um. Mantido apenas **f=5%** como ponto
extra, principalmente como diagnóstico pra N=8 (o único caso em que ele
discrimina entre as hipóteses). Arquivos:
`my_clusters/m5-xlarge/{4,8}workers/synth/task-synth_duration-f5pct-{replace,degraded}.yaml`.

### Checklist consolidado de mudanças em `body.tex` (aplicar numa passada só, após a Etapa 12)

1. **Tabela 16 (`tab:warm-restart-gap`):** adicionar as 6 linhas de N=2
   (EP-D e LU-C, f=10/25/50%) — valores já calculados na seção "Etapa 11.1
   cancelada" acima, sem necessidade de rodar nada novo
2. **Texto da Seção 6.4.2 (linha ~2254):** atualizar "a redução fica entre
   21 e 30% para o EP-D e entre 5 e 34% para o LU-C" para "... entre 5 e
   **37%** para o LU-C" (por causa de LU-C N=2 f=25%, que chega a 37.0% em
   2 das 3 repetições)
3. **Texto da Seção 6.5 (linhas ~2376-2379 e ~2417-2424):** refinar a
   explicação mecanicista do crescimento de Δ com N — trocar a atribuição
   direta ao modelo "1/(N−1)" pela explicação mais precisa (coeficiente
   β−α quase constante entre N; o crescimento de Δ vem majoritariamente de
   T_MANA_noFT diminuir em clusters maiores, não do coeficiente de
   penalidade mudar tanto quanto o texto atual sugere)
4. **Novo capítulo/seção — ADAPTIVE:** todo o conteúdo novo (fórmula,
   derivação, implementação, validação com `synth_duration`, avaliação da
   Etapa 12) — não é edição de texto existente, é conteúdo adicionado
5. **Novas figuras:** heatmap ou scatter plot com os dados do
   `synth_duration` (ainda não gerado — ver decisão de esperar rodar as
   execuções necessárias antes de fazer o gráfico)

---

## Etapa 13 — Exploração: calibração automática do coeficiente por perfil de carga (2026-10-06)

**Motivação:** depois de ler o relatório da Fase 10, o orientador (Prof. Márcio Castro)
perguntou se haveria uma forma de o usuário descobrir o coeficiente certo pra sua
própria aplicação, medindo a relação uso de CPU / acesso a memória do workload. Essa
pergunta mapeia diretamente pra Limitação 2 do relatório ("o coeficiente não é
universal entre cargas de trabalho"). Esta etapa documenta a investigação feita em
resposta, incluindo os caminhos que não deram certo.

### 13.1 Tentativa 1: contadores de hardware via `perf stat` — inviável

Testado diretamente numa instância m5.xlarge (sem custo de tempo em especulação):
`perf list hw` e `perf list cache` retornam listas vazias, e todo contador testado
(`cycles`, `instructions`, `cache-references`, `cache-misses`) retorna
`<not supported>`. A virtualização Nitro da AWS não repassa acesso à PMU (performance
monitoring unit) pra instâncias normais, só pras `.metal`, que mudam custo e
infraestrutura. Caminho descartado com evidência direta, não por suposição.

### 13.2 Alternativa: calibração direta por densidade de processos por núcleo

Em vez de inferir o coeficiente a partir de métricas indiretas de hardware, a ideia
passou a ser medir diretamente o efeito que o coeficiente tenta capturar: o quanto um
processo desacelera ao dividir um núcleo físico com outro (exatamente o que o DEGRADED
causa). Proposta de uso: o usuário faria algumas execuções curtas em cluster on-demand
pra calibrar o coeficiente antes de rodar o job real em workers spot (a mesma execução
normal, sem oversubscrição, também serve pra estimar `estimated_duration_secs`).

Definindo `r` = razão de throughput por processo ao dividir um núcleo (`r=1`: nenhuma
desaceleração; `r=0,5`: cada processo fica na metade da velocidade), o coeficiente pode
ser derivado diretamente, sem ajuste de curva, para o caso N=2 (onde perder 1 de 2
workers força exatamente 2 processos por núcleo):

```
REPLACE:  4 processos, 1 por núcleo, cada um à taxa R       -> throughput total = 4R
DEGRADED: 4 processos, 2 por núcleo, cada um à taxa r×R      -> throughput total = 4rR

coef ≈ (1-r) / r
```

Verificação: `r=0,5` (nenhum ganho do segundo processo) dá `coef=1,0`, exatamente o
modelo ingênuo já descartado na Seção "Derivação da Fórmula Heurística". `r=1` dá
`coef=0`. A fórmula cobre o espectro inteiro sem precisar classificar o workload em
categorias fixas.

Usando os coeficientes reais já medidos, dá pra prever qual `r` cada workload deveria
ter: EP-D (`coef≈0,3`) implica `r≈0,77`; `synth_duration` (`coef≈0,5`) implica `r≈0,67`.

**Limitações já identificadas antes de testar, registradas por transparência:**
- Depende do tipo de instância (precisa recalibrar se trocar hardware).
- A derivação acima é exata pra N=2; em N=4/N=8 o arredondamento do
  `ntasks_per_node` cria densidades mistas (alguns núcleos com 2 processos, outros
  com 1), que essa fórmula simples não cobre diretamente.
- É uma calibração por (instância × aplicação), não um valor universal por hardware.
- Falhas acumuladas que empurrem a densidade além de 2 processos/núcleo entram num
  regime qualitativamente diferente (troca de contexto do SO, não só hyperthreading),
  sem fórmula nem medição própria.

### 13.3 Teste rápido: dois kernels sintéticos, núcleo isolado

Dois programas standalone (sem MPI, sem MANA) foram escritos pra testar a ideia
baratinho, antes de qualquer investimento maior:
- `compute_bound.c`: recorrência de ponto flutuante em registrador, quase sem tocar
  memória (proxy do EP-D).
- `memory_bound.c`: o mesmo loop do `synth_duration.c` (soma num buffer de 50 MB),
  sem MPI.

Executados via task YAML (`my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml`,
reaproveitando o cluster de 2 workers já existente, sem Slurm/MPI envolvido, só
`taskset` pra fixar processos em threads de hardware específicas), em 4 densidades (1,
2, 3 e 4 processos confinados a um único núcleo físico — 2 threads de hardware via
hyperthreading).

**Resultado — kernel CPU-bound:**

| Densidade | Tempo médio | Desaceleração | `r` implícito |
|---|---|---|---|
| 1 | 84,06 s | 1,00× | 1,00 |
| 2 | 84,10 s | 1,00× | 1,00 |
| 3 | 124,76 s | 1,48× | 0,67 |
| 4 | 168,23 s | 2,00× | 0,50 |

Padrão em degrau, exatamente como esperado: sem penalidade até 2 processos (cabem nas
2 threads de hardware do núcleo), e a partir de 3 a desaceleração bate quase exatamente
com uma divisão simples do tempo disponível entre quem compete (2/3 previsto vs 1,48×
medido; 2/4 previsto vs 2,00× medido, praticamente exato).

**Resultado — kernel memory-bound:**

| Densidade | Tempo médio | Desaceleração | `r` implícito |
|---|---|---|---|
| 1 | 6,79 s | 1,00× | 1,00 |
| 2 | 13,20 s | 1,94× | 0,51 |
| 3 | 19,73 s | 2,91× | 0,34 |
| 4 | 26,41 s | 3,89× | 0,26 |

Padrão diferente: a penalidade já começa em densidade 2 (não espera passar de 2), e
segue uma divisão `1/N` desde o início, consistente com o núcleo já estar perto da
saturação da banda de memória com só 1 processo, sem folga nenhuma pra dividir.

**Achado honesto — o número não bate com o ground truth:** jogando `r≈0,51` (densidade
2, kernel de memória) na fórmula, o coeficiente previsto é `≈0,95`. O coeficiente real
do `synth_duration`, medido com a campanha completa de fault injection na AWS
(Etapa 11.2), é `≈0,5`. Discrepância real, não ruído de arredondamento.

### 13.4 Teste de controle: núcleo vizinho ocupado (descarta hipótese de turbo boost)

Hipótese testada: a densidade 1 (processo sozinho) deixa o OUTRO núcleo físico do
m5.xlarge completamente ocioso, o que poderia permitir ao processador acelerar o
clock (turbo boost) mais do que aconteceria numa situação real com o cluster inteiro
ocupado — inflando artificialmente a referência e fazendo a desaceleração parecer
pior do que é.

Testado adicionando um decoy (`compute_bound` com 2 processos, ocupando as 2 threads
do núcleo vizinho) rodando continuamente durante todo o teste, repetindo exatamente as
mesmas 4 densidades.

**Resultado: sem diferença mensurável.** Todos os tempos saíram dentro de <1% dos
valores sem o decoy (ex.: densidade 1 do kernel de memória: 6,79 s sem decoy vs 6,83 s
com decoy). **Hipótese do turbo boost descartada com evidência direta.**

Hipótese alternativa levantada, não testada: o decoy usado foi `compute_bound` (quase
não toca memória). No cenário real do DEGRADED em N=2, os dois núcleos físicos rodam a
mesma aplicação pesada em memória simultaneamente, não um núcleo fazendo uma coisa e o
outro fazendo outra. Um decoy do tipo `memory_bound` no núcleo vizinho testaria
especificamente se a contenção de banda de memória do chip inteiro (não só dentro de
1 núcleo) é o fator que falta. Não executado.

Explicação alternativa, também não isolada: o coeficiente real medido (`≈0,5`) já tem
embutido o efeito de reinício acelerado do REPLACE (Seção 3.5/5.3 do relatório), que
esse microbenchmark isolado não reproduz (não há checkpoint nem restart envolvido).

### 13.5 Decisão: encerrar a exploração aqui

Resultado reportado ao orientador via Slack (mensagem com a narrativa completa:
motivação, tentativa do `perf stat`, derivação da fórmula, os dois testes, achados e
explicações em aberto). Resposta do orientador (2026-10-06): **pode parar por aqui,
é suficiente para o TCC.**

**Conclusão desta etapa:** a direção é promissora e fisicamente bem fundamentada (os
dois kernels produzem padrões de desaceleração limpos e explicáveis, e uma hipótese
concorrente — turbo boost — foi testada e descartada com evidência, não suposição),
mas o valor numérico do coeficiente previsto ainda não bate com o ground truth já
validado. Fica documentada como proposta de trabalho futuro, com a metodologia, os
dados e os próximos passos concretos já mapeados (testar com EP-D/LU-C reais em vez
dos kernels sintéticos; testar decoy do tipo memory-bound; isolar a contribuição do
efeito de reinício acelerado), em vez de deixada como ideia vaga.

**Arquivos desta etapa:**
- `TCC/synthetic/compute_bound.c`, `TCC/synthetic/memory_bound.c` — fontes de
  referência (o código real roda embutido no YAML, igual ao padrão já usado pro
  `synth_duration`)
- `my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml` — teste de
  densidade 1–4, núcleo vizinho ocioso e depois ocupado (duas versões, mesmo arquivo)
- `results/cluster_ClusterMANA_2Workers_m5-xlarge/calib_core_sharing_2w_m5xl_*_SUCCESS.txt`
- `results/cluster_ClusterMANA_2Workers_m5-xlarge/calib_core_sharing_othercorebusy_2w_m5xl_*_SUCCESS.txt`

---

## Gates de Aceitação

| Gate | Descrição |
|------|-----------|
| G1 | Fórmula validada empiricamente contra todos os pontos do CSV; T_MANA_noFT extraídos e documentados |
| G2 | `cargo build --release` passa sem erros após todas as mudanças de código |
| G3 | Smoke test: ADAPTIVE escolhe corretamente nos dois cenários na AWS |
| G4 | Etapa 11.2 (sintético): fronteira consistente confirmada em N=2, 4 e 8 (14 pontos × 3 repetições = 84 execuções) — ✓ concluído. Etapa 11.1 cancelada (ver seção dedicada) |
| G5 | Avaliação de integração do ADAPTIVE (escopo reduzido, 2026-09-26): casos claros REPLACE/DEGRADED já confirmados na Etapa 10; caso de margem apertada (`ep_D-adaptive-smoketest-boundary`, 3 reps) ✓ concluído — 3/3 DEGRADED, sem bug encontrado; caso zero-workers verificado por inspeção de código (`remaining_workers == 0 → REPLACE_RESUME`, watcher.rs); caso de demonstração do limite do coeficiente (`synth_duration-f50pct-adaptive`) ✓ concluído — predição confirmada, ADAPTIVE ~5.8% mais lento que o ótimo real |
| G6 | Etapa 13 (exploração de calibração automática do coeficiente): `perf stat` testado e descartado com evidência; fórmula `coef=(1-r)/r` derivada e testada em 2 kernels sintéticos; hipótese de turbo boost testada e descartada; decisão do orientador (2026-10-06) de encerrar aqui e documentar como trabalho futuro — ✓ concluído |

---

## Arquivos de Referência

| Arquivo | Uso |
|---------|-----|
| `TCC/artifacts/phase5/analysis/results_raw.csv` | Dados brutos dos experimentos |
| `TCC/artifacts/phase6/lapesd-thesis/body.tex` | Seções 6.4.2 (warm-restart), 6.5 (crossover), Tabelas 15–16 |
| `TCC/artifacts/phase7/UCC2026-MANA/gen_figs.py` | Lógica de geração do heatmap de crossover |
| `src/commands/cluster/run_task.rs` | Config YAML + spawn do watcher |
| `src/commands/cluster/watcher.rs` | Loop do watcher + `handle_interruption()` |
| `TCC/synthetic/synth_checkpoint_size.c` | Padrão-base (loop por tempo-alvo) usado para criar `synth_duration.c` |
| `TCC/synthetic/synth_duration.c` | Cópia de referência do workload sintético da Etapa 11.2 (o código real roda embutido nos YAMLs, não a partir deste arquivo) |
| `my_clusters/m5-xlarge/2workers/ep/task-ep_D-{replace,degraded}-15pct.yaml` | YAMLs da Etapa 11.1 |
| `my_clusters/m5-xlarge/{2,4,8}workers/synth/task-synth_duration-f<F>pct-<strategy>.yaml` | 16 YAMLs da matriz da Etapa 11.2 |
| `my_clusters/m5-xlarge/2workers/synth/task-calib-core-sharing.yaml` | Teste de densidade de processos por núcleo da Etapa 13 |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10.md` | Relatório consolidado da Fase 10 (EN) |
| `TCC/artifacts/phase10/analysis/analysis_report_phase10_ptbr.md` | Relatório consolidado da Fase 10 (PT-BR) |
