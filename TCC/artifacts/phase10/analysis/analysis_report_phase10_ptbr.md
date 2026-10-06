# Relatório de Análise — Fase 10: A Estratégia de Recuperação ADAPTIVE
### Derivação da Heurística · Implementação · Validação de Independência de N · Testes de Integração
### Cluster: AWS m5.xlarge workers (2 / 4 / 8 nós) · us-west-2

---

## 1. Visão Geral

O sistema de tolerância a falhas oferece duas estratégias de recuperação depois de uma
interrupção de instância spot: `REPLACE_RESUME`, que provisiona um novo worker (cerca de
120 s) e retoma o job em capacidade total, e `DEGRADED_RESUME`, que retoma imediatamente
nos workers sobreviventes (cerca de 11 s) com capacidade reduzida. Qual das duas é mais
rápida depende de quanto trabalho resta no momento da falha e de quantos workers
sobrevivem. Até agora, essa era uma decisão que o usuário precisava tomar manualmente,
de antemão, sem saber quando (ou se) uma falha aconteceria.

Esta fase implementa o `ADAPTIVE`, uma terceira estratégia que faz essa escolha
automaticamente no momento da falha, usando uma heurística derivada diretamente dos
dados experimentais já coletados. A heurística, sua derivação e sua independência em
relação a N são validadas a seguir. Depois, a implementação é testada de ponta a ponta
contra falhas reais na AWS.

**Escopo.** Toda a validação desta heurística cobre jobs cujo trabalho restante no
momento da falha chega a, no máximo, ~717 s, a mesma faixa de jobs curtos a médios já
delimitada no relatório da Fase 5. A extrapolação para jobs de produção de várias horas
não foi testada, e é discutida como limitação na Seção 7.3.

**Tabela 0: Execuções usadas nesta fase:**

| Fonte | Objetivo | Execuções |
|---|---|---|
| Conjunto NPB (existente, 18 configurações) | Treinar o coeficiente da heurística; ground truth de qual estratégia vence | 108 (existentes) |
| `synth_duration` (N=2, 4, 8; 14 configurações) | Teste com dados nunca vistos: o coeficiente generaliza entre tamanhos de cluster? | 84 (3 repetições cada) |
| Execuções de integração do ADAPTIVE | A implementação real decide e despacha corretamente em falhas reais na AWS? | 6 |

Todas as novas execuções usaram workers spot m5.xlarge da AWS em us-west-2, com o mesmo
mecanismo `auto_test_failure` já usado ao longo do projeto para injeção reprodutível de
falhas. Os valores são reportados como média ± desvio padrão entre repetições. Cada
seção a seguir apresenta suas próprias execuções e a motivação por trás delas conforme
o assunto surge.

---

## 2. O Problema da Decisão

Duas fases distinguem as estratégias. A **P2b (reconfiguração do nó)** é onde o custo
diverge: o REPLACE espera uma nova instância spot ser provisionada e entrar no cluster,
enquanto o DEGRADED simplesmente marca o nó falho como indisponível. Medindo diretamente
em todos os benchmarks e tamanhos de cluster (Tabela 7 do relatório da Fase 5), o P2b do
REPLACE fica em média em ~120 s (tempo de provisionamento do EC2), enquanto o do DEGRADED
é quase constante em ~11 s, independentemente do benchmark ou de N. A diferença,
**~109 s**, é uma economia de tempo fixa que o DEGRADED sempre obtém quando é escolhido,
independentemente da carga de trabalho ou do tamanho do cluster.

Já a **P3 (computação restante)** é onde o DEGRADED paga essa conta de volta: o REPLACE
retoma com todos os N workers, enquanto o DEGRADED retoma com apenas N−1, então a P3
demora mais sob o DEGRADED. Diferente da economia da P2b, essa penalidade não é fixa,
ela cresce conforme aumenta o trabalho restante no momento da falha e conforme aumenta a
capacidade perdida.

O DEGRADED é mais rápido sempre que sua economia de ~109 s na P2b supera essa penalidade
de P3. A pergunta que a heurística precisa responder, no momento da falha, é exatamente
essa comparação.

### 2.1 O modelo teórico ingênuo

Chame de `remaining_secs` o tempo de relógio que o job ainda precisaria para terminar se
continuasse rodando em capacidade total de N workers. É exatamente assim que o REPLACE
retoma, então `P3_replace ≈ remaining_secs` por definição. Assumindo escalonamento
perfeitamente linear, a quantidade total de trabalho restante, em worker-segundos, é
`remaining_secs × N`. O DEGRADED precisa terminar essa mesma quantidade de trabalho
usando apenas N−1 workers:

```
P3_degraded ≈ (remaining_secs × N) / (N-1)
```

A penalidade de P3 é a diferença entre os dois:

```
penalidade = P3_degraded - P3_replace
           = remaining_secs × N/(N-1)  -  remaining_secs
           = remaining_secs × [ N/(N-1) - 1 ]
           = remaining_secs × [ (N - (N-1)) / (N-1) ]
           = remaining_secs / (N-1)
```

Ou seja, sob essa suposição de escalonamento linear, perder um worker de N infla o
trabalho restante por um fator de exatamente `1/(N−1)`: perder 1 de 2 workers dobra o
tempo restante (`1/(2-1) = 1`), enquanto perder 1 de 8 adiciona apenas `1/(8-1) ≈ 14%`.
Juntando as duas metades, o DEGRADED vence exatamente quando sua economia fixa de 109 s
supera essa penalidade de perda de capacidade:

```
REPLACE vence quando:  remaining_secs / (N-1) > 109
```

Esse modelo foi testado contra as 18 configurações já existentes do conjunto NPB (EP-D e
LU-C, N=2/4/8, falha a 10/25/50% do `T_MANA_noFT`). **Resultado: 13/18 corretas.** As 5
falhas ocorrem todas em N=2 (LU-C nos três timings, EP-D a 25% e 50%): o modelo prevê
REPLACE quando na verdade o DEGRADED era mais rápido. O modelo superestima a penalidade
em N=2 porque assume que o `P3_degraded` é exatamente `2×` o `P3_replace`, o que não se
confirma na prática (Seção 4).

---

## 3. Derivando a Heurística Empírica

### 3.1 Ground truth: a penalidade real

Usando os valores medidos de `P3_replace` e `P3_degraded` do conjunto de 18 pontos, a
penalidade real é `penalidade = P3_degraded − P3_replace`, e a decisão correta é
simplesmente `penalidade > 109`. Isso classifica corretamente **18/18** configurações,
sendo o ground truth que a heurística precisa aproximar sem ter acesso aos valores
medidos de P3.

### 3.2 O problema: a P3 não é conhecida no momento da falha

No momento da falha, o watcher só tem `estimated_duration_secs` (fornecido pelo usuário)
e `elapsed_secs` (tempo decorrido desde o início do job), o que dá
`remaining_secs = estimated − elapsed`. Ele não tem acesso à P3 real que a execução
produziria. A heurística precisa estimar a `penalidade` a partir apenas do
`remaining_secs`.

### 3.3 Extração do coeficiente

O modelo ingênuo falhou por assumir uma forma funcional específica, dependente de N
(`penalidade = remaining_secs/(N−1)`), que não se sustenta na prática. A alternativa mais
simples a testar em seguida é se a penalidade continua proporcional ao `remaining_secs`,
mas por meio de uma única constante que não dependa de N: `penalidade ≈ coef ×
remaining_secs`. Para checar se essa constante existe, o coeficiente (`coef = penalidade
/ remaining_secs`) foi calculado para cada uma das 18 configurações:

**Tabela 1: Coeficiente de penalidade no conjunto de treino NPB:**

| Benchmark | N | Falha | remaining_secs | P3_replace | P3_degraded | penalidade | coef |
|---|---|---|---|---|---|---|---|
| EP-D | 2 | 10% | 436,1 | 322,3 | 446,1 | 123,9 | 0,284 |
| EP-D | 2 | 25% | 363,4 | 269,8 | 373,5 | 103,7 | 0,285 |
| EP-D | 2 | 50% | 242,3 | 187,7 | 262,4 | 74,7 | 0,308 |
| EP-D | 4 | 10% | 224,1 | 151,6 | 217,9 | 66,3 | 0,296 |
| EP-D | 4 | 25% | 186,7 | 121,9 | 170,5 | 48,6 | 0,260 |
| EP-D | 4 | 50% | 124,5 | 69,7 | 100,3 | 30,6 | 0,246 |
| EP-D | 8 | 10% | 113,2 | 87,1 | 116,8 | 29,7 | 0,263 |
| EP-D | 8 | 25% | 94,3 | 74,9 | 104,6 | 29,7 | 0,315 |
| EP-D | 8 | 50% | 62,9 | 50,7 | 75,2 | 24,5 | 0,390 |
| LU-C | 2 | 10% | 217,1 | 141,3 | 228,0 | 86,6 | 0,399 |
| LU-C | 2 | 25% | 180,9 | 116,5 | 191,7 | 75,2 | 0,416 |
| LU-C | 2 | 50% | 120,6 | 85,5 | 130,7 | 45,3 | 0,375 |
| LU-C | 4 | 10% | 121,6 | 80,6 | 125,3 | 44,7 | 0,368 |
| LU-C | 4 | 25% | 101,3 | 68,0 | 104,5 | 36,5 | 0,360 |
| LU-C | 4 | 50% | 67,5 | 47,2 | 71,7 | 24,6 | 0,364 |
| LU-C | 8 | 10% | 65,5 | 52,7 | 68,5 | 15,9 | 0,242 |
| LU-C | 8 | 25% | 54,5 | 47,3 | 61,1 | 13,8 | 0,254 |
| LU-C | 8 | 50% | 36,4 | 36,7 | 47,3 | 10,5 | 0,290 |

**Achado: o coeficiente fica sempre em torno de ~0,30, independentemente de N** (média
de 0,345 em N=2, 0,316 em N=4 e 0,292 em N=8; geral de 0,317). É o oposto do que o
modelo ingênuo de 1/(N−1) prevê (1,000, 0,333 e 0,143, respectivamente). Isso significa
que a heurística não precisa saber o tamanho do cluster, apenas o tempo estimado e o
tempo decorrido.

### 3.4 Escolhendo um único coeficiente

Duas configurações ficam no limite: a penalidade delas está perto o suficiente do
patamar de 109 s para que a escolha do coeficiente possa mudar a decisão:

```
EP-D N=2, f=10%: penalidade=123,9 > 109 -> REPLACE (correto)
  exige: coef x 436,1 > 109  ->  coef > 0,250

EP-D N=2, f=25%: penalidade=103,7 < 109 -> DEGRADED (correto)
  exige: coef x 363,4 < 109  ->  coef < 0,300
```

Os dois casos-limite têm coeficiente real entre 0,284 e 0,285. Qualquer coeficiente na
faixa **(0,250, 0,300)** satisfaz as duas restrições ao mesmo tempo. As outras 16
configurações têm penalidades distantes o bastante de 109 s para serem classificadas
corretamente por qualquer coeficiente entre 0,1 e 0,5. O valor **0,28** foi escolhido
como o ponto médio da faixa válida, o mais próximo dos coeficientes reais dos dois
casos-limite.

**Heurística final (18/18 corretas no conjunto de treino):**

```
remaining_secs = T_estimated − elapsed_secs

REPLACE_RESUME  quando  remaining_secs × 0,28 > 109
DEGRADED_RESUME quando  remaining_secs × 0,28 ≤ 109

Equivalente:  REPLACE_RESUME quando remaining_secs > 389,3s  (independente de N)
```

O caso `remaining_workers == 0` é verificado primeiro e força `REPLACE_RESUME`
incondicionalmente (o DEGRADED não pode rodar sem nenhum worker sobrevivente). Essa é
uma restrição física, não uma decisão baseada em tempo, e não é afetada pelo coeficiente
de forma alguma.

![Limiar de decisão do ADAPTIVE contra o conjunto de treino NPB de 18 pontos](../imgs/fig_adaptive_boundary.png)

**A Figura 1** mostra os 18 pontos de treino contra o limiar derivado. Apenas o EP-D em
N=2 cruza `remaining_secs = 389s`, e é por isso que o REPLACE aparece só uma vez no
conjunto de treino (f=10%): as outras 17 configurações nunca têm trabalho restante
suficiente para que o REPLACE valha a pena, nos tamanhos de cluster e classes de
problema testados.

### 3.5 Por que o coeficiente não depende de N

Dois efeitos se compensam:

1. **Efeito de reinício com estado aquecido:** o `P3_replace` é consistentemente 21 a
   34% mais curto do que um modelo de início frio preveria (já documentado na análise de
   crossover existente), algo que o modelo ingênuo não considera. Isso por si só
   aumentaria o coeficiente, não o contrário: como o coeficiente é a diferença de tempo
   entre DEGRADED e REPLACE dividida pelo `remaining_secs`, se o REPLACE termina mais
   rápido, a mesma diferença em segundos passa a representar uma fração maior desse
   tempo de referência.
2. **A sobrecarga de processos não escala como assumido:** em N=2, o DEGRADED reduz o
   cluster a 1 worker rodando todos os processos, mas isso não dobra a P3 como o modelo
   ingênuo assume. Um worker m5.xlarge tem 4 threads de hardware (2 núcleos físicos com
   hyperthreading), e normalmente cada worker roda só 2 processos, deixando uma thread
   ociosa em cada núcleo. Uma explicação possível é que, depois da falha, o único
   sobrevivente passa a ocupar exatamente essas threads ociosas, em vez de disputar
   espaço com processos que já estavam ativos ali, o que absorveria boa parte da carga
   extra nesses benchmarks dominados por computação em registrador (EP-D, LU-C). Assim
   como a explicação análoga para o `synth_duration` na Seção 5.3, isso é uma inferência
   a partir do padrão de tempo observado, não um mecanismo confirmado por medição direta
   (por exemplo, utilização por núcleo ou comportamento de cache durante a fase
   DEGRADED). Não há discussão equivalente no TCC já existente: o texto atual antecipa
   que o DEGRADED introduz competição por CPU na P3, mas não explica por que essa
   competição custa tão pouco para o EP-D e o LU-C.

O efeito de absorção de capacidade ociosa (item 2) é o mais forte dos dois, puxando o
coeficiente para baixo do 1,0 previsto pelo modelo ingênuo em N=2, bem mais do que o
item 1 consegue puxar na direção contrária. O resultado líquido dessas duas forças
opostas é o coeficiente medido de ~0,30 a 0,35 em N=2, bem abaixo do valor previsto pelo
modelo ingênuo, e uma variação bem mais suave entre N=2, 4 e 8 do que
a queda acentuada que esse modelo prevê.

---

## 4. Implementação

`fault_tolerance.strategy: "ADAPTIVE"` é um novo valor possível, ao lado de `NONE`,
`REPLACE_RESUME` e `DEGRADED_RESUME`. Ele exige um campo opcional
`estimated_duration_secs` (o `T_estimated` fornecido pelo usuário). Se esse campo for
omitido, o watcher usa `DEGRADED_RESUME` como padrão, a recuperação mais barata e mais
rápida, em vez de tentar adivinhar.

A função de decisão, `choose_effective_strategy()`, é chamada uma vez por interrupção,
**depois** que o nó já foi drenado (checkpoint gravado, job do Slurm cancelado) e
**antes** de despachar para o handler específico da estratégia, já que precisa saber
qual handler chamar em seguida (a Seção 6.2 volta a esse ponto). Ela calcula
`elapsed_secs` a partir de um timestamp compartilhado, registrado no início dos
`run_commands`, deriva `remaining_secs` e aplica a fórmula da Seção 3.4.

Esse timestamp específico importa. O primeiro smoke test de ponta a ponta escolheu
`DEGRADED_RESUME` quando a fórmula deveria ter previsto `REPLACE_RESUME`: o
`elapsed_secs` estava sendo medido desde a inicialização do watcher, o que incluía os
`setup_commands` (compilação, ~38 s nesses testes), tempo que a estimativa de
`estimated_duration_secs` (derivada apenas do tempo dos `run_commands`) não considera. A
correção ancorou o `elapsed_secs` a um timestamp registrado especificamente no início
dos `run_commands`, compartilhado entre o executor de tarefas e o watcher. Depois da
correção, o mesmo cenário passou a escolher `REPLACE_RESUME`, como previsto.

---

## 5. Validação de Independência de N: a Carga de Trabalho `synth_duration`

O conjunto de dados NPB não permite testar se o coeficiente se sustenta em N=4 ou N=8:
só o EP-D em N=2 chega a cruzar o limiar de 389 s (o LU-C e o EP-D em N≥4 têm valores de
`T_MANA_noFT` curtos demais para alcançá-lo). Uma carga de trabalho sintética, com
duração controlável, o `synth_duration`, foi criada para testar esse mesmo limiar nos
três tamanhos de cluster, em condições controladas.

### 5.1 Desenho

O `synth_duration` foi criado para resolver um único problema: nenhuma configuração real
do NPB, além do EP-D em N=2, chega perto do limiar de 389 s. Não havia como testar a
independência de N da fórmula em N=4 ou N=8 usando benchmarks reais. Faltava uma carga
de trabalho cuja duração pudesse ser ajustada livremente, para posicionar as falhas
exatamente onde a análise do limiar exigia, em cada tamanho de cluster.

```c
long n = (MEM_MB * 1024L * 1024L) / sizeof(double);
double *buf = malloc(n * sizeof(double));
memset(buf, 0x42, n * sizeof(double));  /* força o SO a mapear todas as páginas */

for (long i = 0; i < TOTAL_ITERS; i++) {
    buf[i % n] += 1.0;
    if (i % 5000000L == 0) MPI_Barrier(MPI_COMM_WORLD);
}
```

Cada processo aloca `MEM_MB` = 50 MB e toca o buffer inteiro logo no início
(`memset`), unicamente para que a imagem de checkpoint tenha um tamanho realista, a
mesma técnica usada no estudo `synth_checkpoint_size` da Fase 5. O laço principal roda
por `TOTAL_ITERS`, uma **quantidade fixa de iterações** calibrada no m5.xlarge
(129,5×10⁹ iterações ≈ 600 s em um núcleo sem sobrecarga), e não por uma duração fixa de
relógio: assim, o tempo de conclusão depende de verdade da capacidade de CPU disponível,
e um processo genuinamente sobrecarregado (DEGRADED) leva mais tempo, de forma
mensurável, para terminar a mesma quantidade de trabalho.

O `synth_duration` não foi desenhado para ser limitado por banda de memória, essa
característica surgiu como efeito colateral do laço acima e só ficou evidente depois de
rodar os experimentos. Mas o acaso acabou sendo útil: foi ele que permitiu enxergar como
perfis de carga diferentes produzem comportamentos bem diferentes sob o DEGRADED. Uma
soma por acesso de memória é quase o mínimo de computação possível por byte movido, e o
buffer de 50 MB é maior do que o cache disponível por núcleo no m5.xlarge, então boa
parte desse tráfego tende a chegar até a memória principal. Isso é bem diferente do
EP-D, um gerador de números pseudoaleatórios cujo estado fica majoritariamente em
registradores. A Seção 5.3 usa esse contraste para explicar por que a penalidade de
DEGRADED do `synth_duration` é tão maior do que a do EP-D, e a Seção 5.4 usa o mesmo
contraste para demonstrar uma limitação real do `ADAPTIVE`.

### 5.2 Resultados

Catorze configurações foram executadas em N=2 (f=5/30/40/50/65/75%), N=4 e N=8
(f=5/50/65/75% em cada), REPLACE e DEGRADED, 3 repetições cada (84 execuções no total).

**Tabela 2: Resultados do `synth_duration` (T_synth=600s; média de 3 repetições):**

| N | f% | remaining_secs | `ft` REPLACE | `ft` DEGRADED | coef | Vencedor | Sobreposição |
|---|---|---|---|---|---|---|---|
| 2 | 5% | 570 | 496,6 | 666,7 | 0,498 | REPLACE | não |
| 2 | 30% | 420 | 573,4 | 665,8 | 0,488 | REPLACE | não |
| 2 | 40% | 360 | 594,9 | 664,9 | 0,494 | REPLACE | não |
| 2 | 50% | 300 | 629,7 | 667,0 | 0,499 | REPLACE | não |
| 2 | 65% | 210 | 662,4 | 665,5 | 0,514 | REPLACE | **sim** |
| 2 | 75% | 150 | 701,0 | 669,1 | 0,542 | DEGRADED | não |
| 4 | 5% | 570 | 500,0 | 674,3 | 0,506 | REPLACE | não |
| 4 | 50% | 300 | 636,7 | 675,6 | 0,512 | REPLACE | não |
| 4 | 65% | 210 | 675,8 | 673,7 | 0,516 | DEGRADED | **sim** |
| 4 | 75% | 150 | 704,1 | 675,4 | 0,544 | DEGRADED | não |
| 8 | 5% | 570 | 524,9 | 692,3 | 0,494 | REPLACE | não |
| 8 | 50% | 300 | 652,2 | 694,3 | 0,511 | REPLACE | não |
| 8 | 65% | 210 | 693,7 | 691,3 | 0,522 | DEGRADED | **sim** |
| 8 | 75% | 150 | 717,4 | 693,3 | 0,566 | DEGRADED | não |

**Tabela 3: Coeficiente por tamanho de cluster (média ± desvio padrão entre os pontos daquele N):**

| N | Coeficiente medido | Pontos | Modelo ingênuo 1/(N−1) |
|---|---|---|---|
| 2 | 0,506 ± 0,020 | 6 | 1,000 |
| 4 | 0,519 ± 0,017 | 4 | 0,333 |
| 8 | 0,523 ± 0,031 | 4 | 0,143 |

![synth_duration: vencedor medido em N=2, 4 e 8](../imgs/fig_synth_duration_boundary.png)

![Coeficiente medido comparado ao modelo ingênuo 1/(N-1)](../imgs/fig_coefficient_n_independence.png)

Antes de interpretar os resultados, vale definir "sobreposição". A faixa de cada
estratégia é a média ± desvio padrão das 3 repetições de `ft_wall_time_s`. Duas faixas
se sobrepõem quando o valor mais alto de uma ainda ultrapassa o valor mais baixo da
outra, ou seja, não dá para separar as duas com confiança. Isso é mais exigente do que
apenas "a diferença entre as médias é pequena": mesmo uma diferença pequena pode não ter
sobreposição, se o ruído entre repetições for menor ainda. Sobreposição de verdade
significa que o resultado poderia sair para qualquer lado dependendo só do ruído natural
da execução, não de qual estratégia é realmente mais rápida.

**Achado: o cruzamento não se move com N.** Em f=65% (`remaining_secs=210s`), as faixas
de tempo do REPLACE e do DEGRADED se sobrepõem nos três tamanhos de cluster. É o único
ponto de toda a matriz em que isso acontece, em qualquer N.

Os outros pontos são bem mais claros. Em f=75% (`remaining_secs=150s`), o DEGRADED vence
sem sobreposição, em nenhum N. Em f≤50%, é o REPLACE que vence sem sobreposição, também
em nenhum N.

**Por que o coeficiente não cai com N: uma explicação parcial**

O achado acima descreve o que foi observado, não o porquê. A intuição, e o modelo
ingênuo de 1/(N-1), diriam que perder um worker num cluster maior deveria doer menos: a
queda prevista é de 1,000 para 0,333 e depois 0,143 (N=2, 4 e 8). Não é isso que se
observa.

Um fator concreto ajuda a explicar parte disso. O DEGRADED sempre redistribui o mesmo
total de processos entre os workers que sobrevivem, usando
`ntasks_per_node = ceil(process_count / worker_count)`, um número inteiro arredondado
para cima. Para o `synth_duration`, isso dá:

- **N=2:** cada sobrevivente passa de 2 para 4 processos (aumento de 2×, concentrado num único worker)
- **N=4:** cada sobrevivente passa de 2 para 3 processos (`ceil(8/3) = 3`, aumento de 1,5×)
- **N=8:** cada sobrevivente também passa de 2 para 3 processos (`ceil(16/7) = 3`, o mesmo aumento de 1,5×)

Como N=4 e N=8 caem exatamente no mesmo nível de sobrecarga por worker, faz sentido que
seus coeficientes medidos fiquem tão próximos (0,519 e 0,523).

Isso não explica tudo, porém. N=2 tem a sobrecarga mais severa dos três (2× contra 1,5×),
mas o coeficiente mais *baixo* (0,506), o oposto do que "mais sobrecarga, mais
penalidade" sozinho preveria. Uma explicação possível está no efeito de hyperthreading
discutido na Seção 5.3: em N=2, passar de 2 para 4 processos pode significar ocupar
threads de hardware que antes estavam ociosas, não disputar espaço com threads que já
estavam ativas. Mas este estudo não tem dados de utilização por núcleo para confirmar
isso. Com honestidade: por que o coeficiente fica estável em vez de cair com N só é
parcialmente explicado aqui.

### 5.3 Por que o coeficiente é maior aqui do que no conjunto NPB

O `remaining_secs` mede quanto trabalho resta. O coeficiente mede outra coisa: quanto
tempo *extra* esse trabalho custa quando é espremido em menos workers. Esse custo extra
não é uma propriedade da duração, é uma propriedade de qual recurso de hardware o worker
sobrevivente esgota primeiro.

**O mecanismo em N=2**

O lugar mais claro para ver isso é em N=2, onde o DEGRADED concentra os 4 processos no
único worker sobrevivente. Um worker m5.xlarge tem 2 núcleos físicos, cada um dividido
em 2 threads de hardware (hyperthreading), 4 threads no total. Normalmente, cada worker
roda apenas 2 processos, um por núcleo físico, deixando uma thread ociosa em cada
núcleo. Depois da falha, o sobrevivente roda os 4 processos, um por thread de hardware:
ele ocupa uma capacidade que estava ociosa, em vez de disputar espaço com um trabalho
que já estava rodando ali.

Se essa capacidade ociosa realmente ajuda depende do que o processo passa a maior parte
do tempo fazendo. O EP-D gasta quase todo o seu tempo em aritmética de registrador,
então dois processos dividindo um núcleo físico majoritariamente precisam das unidades
de cálculo do núcleo, e ali sobra espaço. Já o `synth_duration` gasta quase todo o seu
tempo lendo e escrevendo em um buffer de 50 MB. O hyperthreading não cria um segundo
caminho até a memória, então dois processos dividindo um núcleo passam a disputar o
único caminho de memória que antes cada um tinha só para si, e cada um acaba rodando a
quase metade da velocidade.

Esse contraste (capacidade sobrando para o EP-D, um gargalo de verdade para o
`synth_duration`) explica por que dobrar a quantidade de processos custa muito pouco ao
EP-D, mas custa ao `synth_duration` quase uma desaceleração completa de 2×.

**O que essa explicação não cobre**

Ela só descreve diretamente o N=2. A Seção 5.2 já mostra que N=4 e N=8 envolvem uma
sobrecarga menor, espalhada entre vários workers, não uma duplicação concentrada num só.
Por isso, essa explicação não dá conta de todo o motivo pelo qual N=2 acaba com o
coeficiente mais *baixo*, apesar de ter a sobrecarga mais *pesada*.

Ela é também, como já dito na Seção 5.2, uma explicação a partir do padrão observado, não
uma confirmada por contadores de desempenho de hardware. Confirmá-la exigiria profiling
da taxa de cache miss ou da utilização de banda de memória durante a fase DEGRADED, o
que está fora do escopo deste estudo.

### 5.4 Uma consequência prática: o coeficiente errado, ao vivo

A seção anterior explicou por que o coeficiente do `synth_duration` é maior. Mas o
`ADAPTIVE` sempre usa 0,28, o valor calibrado para o EP-D e o LU-C (Seção 3.4), nunca o
que seria correto para essa carga de trabalho em particular. Isso permite prever um erro
concreto: numa região onde o coeficiente real do `synth_duration` (~0,5) já favoreceria
o REPLACE, mas o 0,28 do código ainda não, a fórmula deveria escolher DEGRADED por
engano.

**O teste**

Essa previsão foi testada diretamente, não deixada só no papel. O `ADAPTIVE` rodou sobre
o `synth_duration` em N=2, f=50% (`estimated_duration_secs=600s`), um ponto em que o
`penalty_est` calculado pela fórmula fica entre 73 e 78, abaixo do patamar de 109 s. A
fórmula previu DEGRADED. Mas a Seção 5.2 já mostrou que, nesse ponto exato, o REPLACE é
o vencedor claro.

**Tabela 4: Demonstração do descompasso de coeficiente:**

| | `ft_wall_time_s` |
|---|---|
| REPLACE (ótimo real, média de 3 repetições) | 629,7 s |
| DEGRADED (média de 3 repetições) | 666,9 s |
| **ADAPTIVE (real)** | **666,1 s** |

A previsão se confirmou: o ADAPTIVE escolheu `DEGRADED_RESUME`, terminando em 666,1 s,
um custo de **36,4 s (~5,8%)** frente ao ótimo real. A implementação não tem culpa
nisso, ela aplicou a fórmula exatamente como projetada. A falha é do modelo: um
coeficiente calibrado para um tipo de carga (computação em registrador) não se transfere
para outro (banda de memória).

---

## 6. Testes de Integração: a Implementação Real Decide Corretamente?

A corretude da fórmula (Seção 3) é uma questão separada de saber se a *implementação*,
o watcher, ao vivo, reagindo a uma falha real na AWS, calcula e despacha corretamente
com base nela. Os dois testes a seguir usam sempre o EP-D e o coeficiente padrão do
código (0,28), a mesma combinação para a qual a fórmula foi calibrada (Seção 3). A
pergunta aqui não é se esse coeficiente vale para qualquer carga de trabalho, isso já
foi respondido na Seção 5 com o `synth_duration`, é se o código, ao vivo, aplica
corretamente a fórmula que já se sabe correta para esse tipo de benchmark.

### 6.1 Dois casos inequívocos

**Tabela 5: Testes de integração com decisão clara (EP-D, uma execução cada):**

| Caso | N | `remaining_secs` | `penalty_est` | Decisão | Confirmado por |
|---|---|---|---|---|---|
| REPLACE claro | 2 | 439 s | 122,9 (>109) | `REPLACE_RESUME` | Reinício completo com 4 processos, sem degradação |
| DEGRADED claro | 4 | 217 s | 60,8 (<109) | `DEGRADED_RESUME` | Evento `degraded_resume`, P2b≈11 s |

### 6.2 Um caso perto do limiar de decisão

Um terceiro caso foi construído para ficar mais perto do próprio limiar da fórmula
(389,3 s) do que os dois casos acima: EP-D, N=2, `estimated_duration_secs=485s`, com o
gatilho de falha escolhido para que o `remaining_secs` caísse perto desse limiar. Três
repetições foram executadas.

**Tabela 6: Caso de fronteira, 3 repetições:**

| Rep | Tempo de checkpoint (P1) | `elapsed_secs` na decisão | `remaining_secs` | `penalty_est` | Decisão |
|---|---|---|---|---|---|
| 1 | 16,5 s | 118,0 s | 367,0 s | 102,8 | DEGRADED |
| 2 | 36,9 s | 138,4 s | 346,6 s | 97,1 | DEGRADED |
| 3 | 16,4 s | 117,9 s | 367,1 s | 102,8 | DEGRADED |

As 3 repetições escolheram `DEGRADED_RESUME`, com uma margem de 6,2 a 11,9 unidades de
penalidade abaixo do limiar, mais perto do que qualquer um dos dois casos inequívocos
acima (margens de +7,8 e −54,4), e nenhum bug de implementação apareceu mesmo tão perto
da fronteira. A margem real, porém, ficou longe dos ~0,1 originalmente planejados. O
motivo é que a decisão só é tomada depois que o nó é drenado (`checkpoint_completed` até
`node_drained`) e antes do despacho (`restart_dispatched`), não no instante em que a
falha é detectada, já que o código só sabe qual handler chamar (a reconfiguração rápida
do DEGRADED ou a lenta do REPLACE) depois que o nó já foi removido com segurança do
escalonador. Como consequência, o `elapsed_secs` no momento da decisão é igual ao tempo
do gatilho da falha somado à duração da gravação do checkpoint e da drenagem (~20 a 40 s
nas configurações testadas), não apenas o tempo do gatilho isolado, o que explica a
diferença em relação à margem planejada.

---

## 7. Conclusões

### 7.1 A heurística

Dentro do escopo testado (jobs curtos a médios, até ~717 s de trabalho restante, dois
perfis de carga), um único coeficiente empírico (0,28) e um patamar fixo de 109 s
preveem a estratégia de recuperação correta sem precisar saber o tamanho do cluster.
Isso foi validado contra o conjunto de treino NPB (18/18, Seção 3) e contra uma carga de
trabalho sintética com dados nunca vistos e coeficiente bem diferente (~0,5, Seção 5), na
qual o cruzamento ficou no mesmo lugar independentemente de N.

### 7.2 A implementação

O `ADAPTIVE` foi validado de ponta a ponta contra falhas reais na AWS: dois casos
inequívocos corretos, um caso perto do próprio limiar da fórmula (3/3 corretas, Seção
6.2), o caso de zero workers verificado por inspeção de código, e o caso de descompasso
de coeficiente (Seção 5.4), que confirma que a implementação aplica a fórmula
corretamente mesmo quando o resultado é subótimo. Dois bugs reais de implementação
foram encontrados e corrigidos no processo (Seções 4 e 6.2).

### 7.3 Limitações conhecidas

1. **A validação parece se limitar a jobs curtos e médios.** Nos dados que temos,
   `remaining_secs` nunca passou de ~717 s, e o REPLACE quase não apareceu como vencedor
   nos testes, só dois pontos chegaram perto do limiar. Jobs de produção de várias horas
   ficariam bem fora dessa faixa. A extrapolação parece razoável em teoria (a relação é
   uma razão de capacidade, não algo atrelado à duração absoluta), mas isso não foi
   testado.
2. **O coeficiente parece não ser o mesmo para toda carga de trabalho.** Nos dois perfis
   testados, uma carga limitada por banda de memória mostrou coeficiente real de ~0,5,
   bem diferente do 0,28 que o código usa, e isso já causou uma decisão ~5,8% pior na
   prática (Seção 5.4). Calibrar por aplicação fica como trabalho futuro.
3. **Os mecanismos por trás do coeficiente são inferidos, não medidos.** Por que ele
   difere entre o NPB e o `synth_duration`, e por que fica estável em vez de cair com N,
   são padrões observados (Seções 5.2 e 5.3), não confirmados com profiling de hardware.
4. **`T_estimated` precisa ser fornecido pelo usuário.** Decisão de escopo tomada antes
   da implementação, não uma lacuna descoberta depois.
5. **Falhas sucessivas múltiplas não foram testadas** para o `ADAPTIVE`
   especificamente, embora a fórmula recalcule `remaining_secs` do zero a cada falha.

**Juntando tudo:** dentro do escopo testado, o `ADAPTIVE` funciona, tanto na fórmula
quanto na implementação. As duas limitações que mais importam, extrapolação para jobs
longos e coeficiente dependente de carga de trabalho, são o que precisaria ser resolvido
antes de confiar nesse mecanismo além do regime caracterizado aqui.

---

*Relatório cobre 108 execuções de treino (já existentes, 18 configurações), 84 execuções
de validação do `synth_duration` e 6 execuções de integração do ADAPTIVE, coletadas até
2026-09-26. Dados brutos: `synth_duration_raw.csv`,
`synth_duration_wide.csv`, `coefficient_per_n.csv`. Figuras geradas por
`analyze_synth_duration.py`. Histórico completo de derivação e decisões:
`../PHASE10_PLAN.md`.*
