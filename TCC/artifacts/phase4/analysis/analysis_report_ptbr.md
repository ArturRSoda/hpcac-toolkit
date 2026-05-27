# Tolerância a Falhas em Clusters Spot — Resultados dos Experimentos
### Benchmarks: NAS Parallel Benchmarks · LU Classe C · EP Classe D · CG Classe C
### Cluster: AWS m5.xlarge workers (2 e 4 nós) · us-west-2

---

## 1. Visão Geral

Este relatório analisa os resultados de 24 execuções em três benchmarks (LU, EP, CG), dois tamanhos de cluster (2 e 4 workers) e quatro estratégias de tolerância a falhas:

| Estratégia | Descrição |
|---|---|
| **noFT (nativo)** | MPI puro, sem MANA, sem tolerância a falhas |
| **MANA noFT** | Camada de interposição MANA ativa, nenhuma falha injetada |
| **REPLACE** | MANA FT: nó com falha é substituído por uma nova instância EC2 |
| **DEGRADED** | MANA FT: o trabalho continua com um processo a menos após a falha |

Cada execução com tolerância a falhas (FT) teve uma falha injetada em um tempo fixo, garantindo que o momento da falha fosse reproduzível entre as estratégias.
Foram realizadas somente 1 repetição em cada cenário.

---

## 2. Tempo Total de Execução

![Tempo de execução por benchmark e estratégia](plots/fig1_wall_time.png)

**Valores brutos do tempo de execução (segundos):**

| Benchmark | Workers | noFT | MANA noFT | REPLACE | DEGRADED |
|---|---|---|---|---|---|
| CG Classe C | 2w | 34.3 | 107.2 | 283.4 | 170.4 |
| CG Classe C | 4w | 20.9 | 32.8 | 256.8 | 93.7 |
| EP Classe D | 2w | 330.5 | 492.6 | 583.7 | 537.4 |
| EP Classe D | 4w | 167.0 | 249.0 | 409.1 | 294.6 |
| LU Classe C | 2w | 146.9 | 242.1 | 397.3 | 302.3 |
| LU Classe C | 4w | 80.7 | 134.6 | 360.1 | 190.2 |

Os três benchmarks acabaram abrangendo tempo muito diferentes: O CG Classe C termina em menos de 35 segundos nativamente, o LU em cerca de 2,5 minutos, e o EP Classe D em mais de 5 minutos.
Mas acredito que essa variação pode ser útil porque revela como o tempo de recuperação se comporta em relação à duração do trabalho.

---

## 3. Custo Adicional de Desempenho (Overhead) do MANA

![Fatores de overhead relativo ao noFT](plots/fig2_mana_overhead.png)

**Overhead do MANA (MANA noFT ÷ noFT):**

| Benchmark | 2 workers | 4 workers |
|---|---|---|
| CG Classe C | **3.13×** | 1.57× |
| EP Classe D | 1.49× | 1.49× |
| LU Classe C | 1.65× | 1.67× |

Os resultados expõem um padrão claro baseado na intensidade de comunicação:

*   **EP Classe D:** Apresenta o overhead mais baixo e consistente (~1.49×), independentemente do tamanho do cluster. Como os processos EP têm comunicação entre processos quase nula, o custo de interceptação do MANA é mínimo e fixo por processo.
*   **LU Classe C:** Mostra um overhead estável de ~1.66×, um pouco maior que o EP, mas consistente com 2 e 4 workers. Os padrões de comunicação no LU são estruturados e previsíveis.
*   **CG Classe C:** Apresenta 1.57× com 4 workers, consistente com os outros benchmarks. Porém, o resultado com 2 workers (3.13×) é uma suspeita de anomalia isolada. Vamos ver se com mais repetições esse número irá se manter.

---

## 4. Analisando as Fases de Recuperação de Falhas

![Fases de recuperação empilhadas para execuções FT](plots/fig3_recovery_phases.png)

A recuperação é dividida em três fases:

| Fase | Definição | Fator principal |
|---|---|---|
| **Fase 1** | Detecção da falha → checkpoint salvo | Tamanho do checkpoint / intervalo de poll do watcher |
| **Fase 2** | Checkpoint salvo → despachado do restart | Provisionamento de instância AWS (REPLACE) ou setup de reinício (DEGRADED) |
| **Fase 3** | inicio do restart → conclusão | Computação restante + mudança na quantidade de processos |

**Tempos medidos por fase (segundos):**

| Benchmark | Workers | Estratégia | Fase 1 | Fase 2 | Fase 3 | Recuperação Total |
|---|---|---|---|---|---|---|
| CG C | 2w | REPLACE | 26.6 | 190.6 | 53.7 | 271.0 |
| CG C | 2w | DEGRADED | 26.6 | 23.1 | 110.3 | 160.0 |
| CG C | 4w | REPLACE | 26.5 | 185.4 | 38.1 | 250.0 |
| CG C | 4w | DEGRADED | 26.6 | 21.7 | 36.9 | 85.2 |
| EP D | 2w | REPLACE | 16.3 | 196.4 | 314.7 | 527.4 |
| EP D | 2w | DEGRADED | 16.3 | 21.7 | 444.4 | 482.4 |
| EP D | 4w | REPLACE | 16.4 | 185.6 | 151.5 | 353.5 |
| EP D | 4w | DEGRADED | 16.3 | 21.6 | 205.0 | 243.0 |
| LU C | 2w | REPLACE | 26.5 | 185.0 | 130.5 | 342.0 |
| LU C | 2w | DEGRADED | 26.6 | 21.7 | 198.9 | 247.2 |
| LU C | 4w | REPLACE | 26.5 | 220.9 | 57.5 | 305.0 |
| LU C | 4w | DEGRADED | 26.5 | 21.6 | 88.8 | 136.9 |

**Principais observações:**

A **Fase 1** depende do tamanho do checkpoint do benchmark, não da estratégia:
*   LU e CG: ~26.5 s (maior uso de memória, mais dados para gravar no checkpoint).
*   EP: ~16.3 s (embaraçosamente paralelo, cada processo guarda um estado menor e independente).

A **Fase 2** é determinada pela infraestrutura:
*   DEGRADED: **~21–23 s** em todos os benchmarks e tamanhos de cluster, tempo de ler o checkpoint e reiniciar com um processo a menos.
*   REPLACE: **~185–221 s**, dominado pelo tempo de provisionamento da instância AWS EC2 (boot, inicialização da AMI, conexão com o Slurm).

A **Fase 3** é onde as estratégias divergem pela carga de trabalho:
*   Para **trabalhos curtos (CG)**: A Fase 3 do DEGRADED é maior, deixando ~98% do trabalho restante para uma quantidade reduzida de processos. Apesar disso, a recuperação total do DEGRADED (85–160 s) ainda é bem menor que a do REPLACE (250–271 s) porque a Fase 2 domina o custo do REPLACE.
*   Para **trabalhos longos (EP)**: A penalidade da Fase 3 do DEGRADED é proporcionalmente grande (444 s para 2w) porque o EP é embaraçosamente paralelo, perder 1 de 4 processos significa que os 3 processos restantes precisam cobrir 33% a mais de trabalho cada. A Fase 3 do REPLACE (315 s) é mais curta porque restaura a contagem total de processos antes de continuar.

---

## 5. Comparação Direta: REPLACE vs DEGRADED

![Overhead REPLACE vs DEGRADED relativo ao MANA noFT](plots/fig6_replace_vs_degraded.png)

**Custo extra (overhead) FT relativo à base MANA noFT:**

| Benchmark | Workers | Overhead REPLACE | Overhead DEGRADED |
|---|---|---|---|
| CG Classe C | 2w | 2.64× | 1.59× |
| CG Classe C | 4w | **7.83×** | 2.86× |
| EP Classe D | 2w | 1.18× | **1.09×** |
| EP Classe D | 4w | 1.64× | 1.18× |
| LU Classe C | 2w | 1.64× | 1.25× |
| LU Classe C | 4w | 2.68× | 1.41× |

O DEGRADED supera consistentemente o REPLACE em todos os benchmarks e tamanhos de cluster. A vantagem do DEGRADED é mais evidente para trabalhos curtos como o CG, onde a Fase 2 do REPLACE (~185–220 s) pesa muito na duração total do trabalho.
Dessa forma, para o EP Classe D com 2 workers, ambas as estratégias se aproximam (1.18× vs 1.09×) porque a longa Fase 3 do DEGRADED compensa parcialmente a economia de tempo da Fase 2.

Isso indica que para workloads mais longo, o REPLACE acaba sendo o mais indicado, enquanto para workloads curtos o DEGRADED passa a ser mais interessante.

A teoriase se confirma quando observamos o resultado do CG 4w REPLACE (7.83×), onde representa um caso extremo: um trabalho de 20.9 s sofrendo um atraso de recuperação de ~251 s, com os custos de infraestrutura dominando completamente.
Assim, a estratégias REPLACE para trabalhos paralelos de curta duração em infraestrutura de nuvem, se mostra não ser uma escolha inteligente.

---

## 6. Escalabilidade

![Tempo de execução vs número de workers por estratégia](plots/fig4_scalability.png)

Escalar de 2 para 4 workers reduz o tempo de execução para todas as estratégias. E aparenta ser linear.

---

## 7. Análise Financeira e de Custos

![Custo por execução: noFT on-demand vs estratégias FT em spot](plots/fig5_cost.png)

**Modelo de precificação:**
*   noFT (nativo): todos os nós sob demanda (on-demand).
*   REPLACE / DEGRADED: workers em spot (desconto de ~70% para m5.xlarge: \$0.0585/h vs \$0.1920/h on-demand); nó principal (head node) on-demand.

**Custo por execução (USD), cluster de 4 workers:**

| Benchmark | noFT On-Demand | DEGRADED Spot | Economia |
|---|---|---|---|
| CG Classe C | $0.0049 | $0.0083 | –67% (mais caro) |
| LU Classe C | $0.0191 | $0.0168 | **+12%** |
| EP Classe D | $0.0395 | $0.0260 | **+34%** |

O custo financeiro para tolerância a falhas com instâncias spot aparenta depender da duração do trabalho:

*   **Trabalhos curtos (CG Classe C, ~35 s):** O desconto do spot não compensa o tempo de recuperação adicionado pelo DEGRADED (+73 s). Rodar o noFT no modelo on-demand é mais rápido e mais barato. FT não tem bom custo-benefício para trabalhos dessa duração.
*   **Trabalhos médios (LU Classe C, ~150 s):** DEGRADED no spot é ligeiramente mais barato (12% de economia) ao mesmo tempo em que fornece recuperação de falhas. O ponto de equilíbrio (break-even) é visível aqui.
*   **Trabalhos mais longos (EP Classe D, ~300 s):** O desconto do spot domina, tornando o DEGRADED 34% mais barato que o noFT on-demand, tolerando também falhas nos nós. Este é o cenário onde a estratégia entrega valor claro.

Nos casos observados, a estratégia REPLACE não é competitiva em custo em nenhum benchmark: seu tempo da Fase 2 (~185–220 s) inflaciona o tempo total além do ponto em que a economia com spot possa compensar.
Mas talvez para workloads mais longos ainda, passa a compensar.

---

## 8. Conclusões

Os resultados experimentais dos benchmarks LU, EP e CG demonstram que reiniciar de um checkpoint baseado em MANA é um mecanismo viável de tolerância a falhas para cargas de trabalho MPI em clusters spot na nuvem, com alguns pontos importantes ligadas à duração do trabalho e ao padrão de comunicação.

**Principais descobertas:**

1. **Impacto inerente da ferramenta:** A execução através do MANA, mesmo sem a ocorrência de falhas, já introduz um overhead de ~1.5x no tempo de execução da aplicação.

2. **Tempo de salvamento (Fase 1):** O tempo gasto para a preparação e gravação do checkpoint depende exclusivamente da quantidade de dados em memória que precisam ser gravados no arquivo de restauração, e não da estratégia adotada.

3. **REPLACE vs. DEGRADED:** A eficiência da estratégia de recuperação depende da duração da tarefa. A estratégia DEGRADED se mostra muito superior para workloads curtos e médios (pois evita o tempo de provisionamento de infraestrutura). Já a estratégia REPLACE só começa a ser indicada para workloads longos, onde o tempo gasto subindo uma nova máquina se paga ao restaurar o poder total de processamento.

4. **Viabilidade Econômica:** A utilização de tolerância a falhas compensa financeiramente graças aos altos descontos das instâncias Spot. A única exceção são os workloads muito curtos, onde o tempo de atraso da recuperação da falha acaba anulando a economia gerada pelo uso do modelo Spot.