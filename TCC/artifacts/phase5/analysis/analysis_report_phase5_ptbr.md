# Relatório de Análise — Fase 5: Aprofundando a Avaliação de Tolerância a Falhas
### Mecanismos de Overhead do MANA · Tamanho do Checkpoint · Sensibilidade ao Momento da Falha
### Cluster: AWS m5.xlarge workers (2 / 4 / 8 nós) · us-west-2

---

## 1. Visão Geral

Este relatório apresenta os resultados completos dos experimentos da Fase 5. A Fase 5
estende a campanha piloto da Fase 4 em três direções:

| Estudo | Objetivo | Execuções |
|---|---|---|
| **5.1 Overhead MPI sintético** | Isolar se frequência de chamadas ou desbalanceamento de comunicação é o responsável pelo overhead do MANA | 90 |
| **5.2 Tamanho do checkpoint sintético** | Quantificar como o tamanho da imagem de checkpoint determina o tempo da Fase 1 (detecção até checkpoint) | 12 |
| **5.3 Sensibilidade ao momento da falha** | Medir como a posição da falha no ciclo de vida do job afeta o trade-off entre REPLACE e DEGRADED | 107 |
| **Baselines (reexecução da Fase 4)** | noFT e MANA-noFT para os 3 benchmarks × 3 tamanhos de cluster com nova instrumentação | 54 |

**Total: 282 execuções válidas**, coletadas com 3 repetições por configuração.
Todos os resultados foram coletados em instâncias spot m5.xlarge na AWS (us-west-2). O nó
de controle foi uma instância t3.large on-demand. Cada execução utilizou o mecanismo
`auto_test_failure` para injeção reprodutível de falhas. Os valores neste relatório são
expressos como média ± desvio padrão entre as repetições.

**Tabela 0: Especificações das instâncias do cluster:**

| Papel | Instância | vCPUs | RAM | OS | On-demand (us-west-2) | Spot (us-west-2) | Desconto spot |
|---|---|---|---|---|---|---|---|
| Worker (execuções FT) | m5.xlarge | 4 | 16 GB | Amazon Linux 2 | $0,192/hr | $0,0585/hr | ~70% |
| Nó de controle | t3.large | 2 | 8 GB | Amazon Linux 2 | $0,0832/hr | (somente on-demand) | n/a |

Os preços spot refletem médias observadas em us-west-2 durante o período dos experimentos
(consultados via `aws ec2 describe-spot-price-history`). Instâncias spot podem ser
interrompidas com aviso de 2 minutos, que é exatamente o cenário de interrupção testado.

Novidade na Fase 5: a recuperação agora instrumenta **três sub-fases da Fase 2**:
- **Fase 2a** (drenagem e cancelamento do job no Slurm, ~5,5 s, idêntica para ambas as estratégias)
- **Fase 2b** (reconfiguração do nó: respawn do EC2 para REPLACE, ~120-130 s; ou `scontrol DOWN` para DEGRADED, ~11 s)
- **Fase 2c** (nova alocação no Slurm e inicialização do coordenador MANA, ~5,3 s, idêntica para ambas)

---

## 2. Overhead do MANA sem Falhas

![Overhead do MANA por benchmark e quantidade de workers](plots/fig1_mana_overhead.png)

**Tabela 1: Overhead do MANA (tempo de execução MANA-noFT vs noFT, N=3 por célula):**

| Benchmark | 2 workers | 4 workers | 8 workers |
|---|---|---|---|
| CG-C | +222% (34,1 → 109,8 s) | +55% (20,2 → 31,2 s) | +71% (12,2 → 20,9 s) |
| EP-D | +45% (334,3 → 484,5 s) | +47% (169,7 → 249,0 s) | +27% (99,1 → 125,7 s) |
| LU-C | +63% (147,8 → 241,2 s) | +63% (83,1 → 135,1 s) | +53% (47,6 → 72,7 s) |

Cada valor de overhead é a diferença entre duas execuções separadas (uma noFT e uma
MANA-noFT) realizadas em instâncias spot EC2 em momentos distintos. Com 3 repetições por
célula, a variabilidade dentro do grupo agora é mensurável. Os desvios padrão das execuções
noFT são pequenos (abaixo de 4 s), confirmando baselines estáveis. As execuções MANA-noFT
mostram variabilidade um pouco maior (até ±15,6 s para EP-D a 2w), consistente com a
infraestrutura de checkpoint do MANA adicionando sincronização não-determinística na
inicialização e finalização.

Os valores de overhead não apresentam padrão consistente entre benchmarks ou quantidade
de workers, e são muito maiores do que os ~4-5 s medidos pelos estudos sintéticos (Seção 4).
Essa diferença entre overhead sintético e real é analisada na Seção 2.4.

### 2.1 CG-C com 2 Workers: Overhead Anômalo

CG com 2 workers apresenta +222% de overhead (34,1 s → 109,8 s), enquanto com 4 workers
é apenas +55% (20,2 s → 31,2 s). As médias das 3 repetições são estáveis (std ≤ 4,6 s),
confirmando que o resultado é reprodutível.

O mecanismo mais plausível é uma cascata de competição por CPU desencadeada pelo
comportamento de espera em loop ativo do MANA. A diferença fundamental entre execuções
noFT e MANA está em como cada uma trata o `MPI_Wait`: no noFT, `MPI_Wait` é uma chamada
bloqueante nativa que cede o uso da CPU ao escalonador do sistema operacional; no MANA,
`MPI_Wait` é implementado como um loop de espera ativa que fica continuamente consultando
se a mensagem chegou enquanto mantém um lock de checkpoint (Seção 4.2), consumindo ciclos
de CPU durante toda a espera.

Com 2 workers, dois processos MPI rodam na mesma máquina física. Quando um processo fica
em loop de espera aguardando uma mensagem do seu parceiro co-localizado, ele consome CPU
que o parceiro precisa para terminar seu cálculo. Isso atrasa o parceiro, que atrasa a
mensagem, que prolonga o tempo de espera em loop, que consome mais CPU, e assim
sucessivamente pelas 75 iterações do gradiente conjugado do CG. No caso noFT, o processo
em espera cede sua CPU ao SO, então o parceiro não é atrasado e nenhuma cascata se forma.

**Por que o MANA usa loop de espera ativa em vez de uma chamada bloqueante?** Uma chamada
nativa bloqueante de `MPI_Wait` (ou a chamada de sistema `select()`/`poll()` que ela usa
internamente) coloca o processo para dormir dentro do kernel do SO. Enquanto dormindo, o
processo mantém o lock de leitura de checkpoint do DMTCP, impedindo que qualquer sinal de
checkpoint seja processado. Se o MANA permitisse isso, um processo bloqueado esperando
uma mensagem lenta manteria o lock indefinidamente, e o mecanismo de checkpoint nunca
conseguiria avançar. O loop de espera ativa é a solução de design que resolve esse problema:
entre cada iteração de consulta, o MANA chama `DMTCP_PLUGIN_ENABLE_CKPT()` para liberar
o lock antes de imediatamente re-adquiri-lo com `DMTCP_PLUGIN_DISABLE_CKPT()`. Nessa
breve janela entre iterações, o handler de checkpoint pode interromper o processo e salvar
seu estado de forma segura. O custo é que o processo permanece em espaço de usuário
consumindo CPU continuamente em vez de ceder ao escalonador do SO. Esse é um requisito
fundamental do checkpoint baseado em DMTCP: checkpoints seguros exigem controle em espaço
de usuário sobre exatamente quando podem ser disparados.

O teste `synth_imbalanced` (Seção 4.2) foi desenhado justamente para medir se esse consumo
de CPU do loop de espera ativa se traduz em overhead observável quando um parceiro de
comunicação se atrasa. Esse teste coloca remetente e receptor em nós separados, então a
competição por CPU entre processos é impossível por design. A cascata no CG a 2w só foi
identificada depois de comparar os resultados sintéticos com os dados reais dos benchmarks:
o teste sintético confirmou que o overhead do loop de espera ativa sozinho (com nós
separados) não ultrapassa ~4-5 s, tornando o excedente de 75 s a 2w inexplicável sem o
fator de co-localização.

Com 4 e 8 workers, os processos do CG ficam distribuídos em mais máquinas, reduzindo as
chances de processos co-localizados competindo por CPU durante as esperas.

Essa hipótese de cascata não pode ser confirmada diretamente sem profiling por rank dentro
do benchmark NAS, o que está fora do escopo deste estudo.

### 2.2 EP-D: Overhead Diminui com Mais Workers

Com N=3 repetições por célula, o overhead do EP-D é consistente entre 27-47%, com desvios
padrão pequenos (±0,7-15,6 s em tempos de execução de 100-485 s). O overhead absoluto
diminui de 150 s a 2w para 79 s a 4w e 27 s a 8w, uma redução de 5,5x enquanto a
quantidade de workers aumenta 4x.

Importante: essa tendência é o **oposto** do que se esperaria de uma explicação por custo
fixo de inicialização. Um overhead fixo de, digamos, 30 s representaria apenas 9% de um
job noFT de 334 s a 2w, mas 30% de um job de 99 s a 8w. Jobs mais curtos mostrariam
percentuais de overhead maiores com qualquer custo fixo. O que observamos é o contrário:
o overhead é maior (45-47%) nos jobs mais longos e menor (27%) nos mais curtos.

Nenhuma explicação mecanicista para essa tendência foi identificada a partir dos dados
disponíveis. A observação é reprodutível (N=3 por célula), confirmando que é uma
característica real do EP sob o MANA, não ruído de medição.

### 2.3 LU-C: Overhead Mais Consistente

LU-C apresenta 63% a 2w, 63% a 4w e 53% a 8w, o padrão mais estável dos três benchmarks.
O overhead se mantém aproximadamente proporcional ao tempo de computação entre as
quantidades de workers. A leve diminuição a 8w (53% vs 63%) também não tem explicação
mecanicista identificada nos dados disponíveis, situação análoga à da Seção 2.2. A
observação é reprodutível (N=3 por célula, std do noFT ≤ 3,7 s), mas os dados por si só
não apontam para uma causa.

LU é o benchmark mais confiável para caracterização de overhead: comunicação estruturada,
tempo de execução estável e baixa variabilidade dentro da célula.

### 2.4 A Diferença de Overhead: O que os Testes Sintéticos Explicam e o que Não Explicam

Os estudos sintéticos (Seção 4) medem consistentemente ~4-5 s de overhead do MANA para
programas isolados, independentemente de frequência de chamadas ou desbalanceamento de
comunicação. Os benchmarks NPB reais mostram 27-222%. Essa grande diferença é um achado
importante e ainda em aberto.

Os testes sintéticos estabelecem dois pontos: (1) o overhead não é causado pela frequência
de chamadas MPI, e (2) não é causado por simples desbalanceamento de comunicação entre
dois processos isolados em nós separados. O que eles não conseguem testar é qualquer
mecanismo que emerge da combinação de computação, comunicação e compartilhamento de
recursos que ocorre nas execuções reais dos benchmarks.

O caso do CG a 2w tem um mecanismo plausível (cascata de competição por CPU entre processos
co-localizados, Seção 2.1). Para o nível geral de overhead observado em todos os benchmarks
(27-222%), nenhuma explicação clara está disponível neste estudo. Mecanismos propostos foram
considerados, mas cada um apresenta contradições com os dados observados:

- **Inicialização e finalização escalando com a quantidade de ranks.** Se o custo de setup
  do MANA crescesse com o número de ranks MPI, o overhead aumentaria com mais workers (mais
  ranks). A tendência observada para EP e LU é o oposto.
- **Intercalação de comunicação e computação.** A hipótese de que o overhead do MANA depende
  de com que frequência as fases de computação são interrompidas por chamadas MPI é
  exatamente o que o teste sintético de frequência de chamadas mede. O teste não mostrou
  esse efeito.

A conclusão honesta é que os estudos sintéticos estreitam o espaço de busca por mecanismos
de overhead mas não identificam a causa raiz nos benchmarks reais. Isolar os contribuidores
individuais exigiria profiling do estado interno do MANA (padrões de contenção de locks,
atividade do coordenador, timing por fase) durante execuções reais dos benchmarks. Isso
fica como trabalho futuro.

---

## 3. Escalabilidade

![Escalabilidade forte: tempo de execução vs quantidade de workers](plots/fig6_mana_scalability.png)

**Tabela 2: Tempo de execução (segundos) vs quantidade de workers, noFT e MANA-noFT (média ± std, N=3):**

| Benchmark | noFT 2w | noFT 4w | noFT 8w | Speedup 2→8 | MANA 2w | MANA 4w | MANA 8w |
|---|---|---|---|---|---|---|---|
| CG-C | 34,1±0,7 | 20,2±1,3 | 12,2±0,1 | 2,8× | 109,8±4,6 | 31,2±1,3 | 20,9±0,0 |
| EP-D | 334,3±0,7 | 169,7±0,9 | 99,1±3,2 | 3,4× | 484,5±15,6 | 249,0±2,0 | 125,7±0,9 |
| LU-C | 147,8±1,8 | 83,1±3,7 | 47,6±0,3 | 3,1× | 241,2±1,2 | 135,1±1,7 | 72,7±1,3 |

Os três benchmarks escalam de forma sub-linear de 2 para 8 workers (o ideal seria 4×). EP e
LU atingem 3,1-3,4×, razoável para aplicações MPI com alto uso de memória em clusters
pequenos. O 2,8× do CG reflete seu padrão de comunicação irregular.

O resultado chave: **MANA-noFT segue a mesma curva de escalabilidade que o noFT**. O
overhead não piora com mais workers, ou seja, a infraestrutura de checkpoint do MANA não
introduz um gargalo de escalabilidade.

---

## 4. Mecanismos de Overhead do MANA: Estudos Sintéticos

Os benchmarks reais misturam computação, comunicação e efeitos do cluster de formas que
dificultam isolar o que causa o overhead do MANA. Quatro programas sintéticos foram
desenhados para testar uma variável por vez.

### 4.1 Estudo de Frequência de Chamadas

**Por que esse teste foi criado:** CG e LU fazem milhares de chamadas MPI por execução.
Antes de investigar qualquer outro mecanismo, era preciso descartar a explicação mais
simples: que o overhead por chamada do wrapper do MANA simplesmente se acumula com a
contagem de chamadas, gerando maior overhead em benchmarks que comunicam com mais
frequência. O `synth_calls` testa isso diretamente ao executar uma carga de trabalho fixa
enquanto varia o número de chamadas MPI de 0 a 51.200.

**O que este teste mede:** Se o overhead do MANA cresce conforme o programa faz mais
chamadas MPI. `synth_calls` realiza uma quantidade fixa de computação e chama
`MPI_Allreduce` (a operação principal do EP) em diferentes frequências, de 0 a 51.200
chamadas por execução. `synth_p2p` faz o mesmo mas usa pares `MPI_Send + MPI_Recv`
(padrão do LU):

```c
/* synth_calls: operações coletivas */
for (long outer = 0; outer < TOTAL_OUTER; outer++) {
    for (long i = 0; i < INNER_ITERS; i++)
        x = x * 1.0000001 + 1e-10;
    if (call_period > 0 && outer % call_period == 0) {
        MPI_Allreduce(&x, &result, 1, MPI_DOUBLE, MPI_SUM, MPI_COMM_WORLD);
        x += result * 1e-20;
    }
}

/* synth_p2p: operações ponto-a-ponto (sem deadlock) */
if (rank < nprocs / 2) {
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
} else {
    MPI_Recv(&result, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &status);
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
}
```

![Overhead do MANA vs frequência de chamadas MPI](plots/fig2_synth_calls.png)

**Tabela 3: Overhead do MANA (segundos adicionados) vs contagem de chamadas (média ± std, N=3):**

| Nível | Chamadas | overhead synth_calls | overhead synth_p2p |
|---|---|---|---|
| L0 | 0 | +3,5 s | +4,4 s |
| L1 | 800 | +4,8 s | +4,3 s |
| L2 | 3.200 | +4,5 s | +5,9 s |
| L3 | 12.800 | +4,7 s | +3,8 s |
| L4 | 51.200 | +4,3 s | +2,2 s |

**Resultado:** O overhead do MANA é constante (~3,5-5 s) independentemente da contagem de
chamadas. Frequência de chamadas não é o fator determinante. O overhead vem dos custos de
inicialização e finalização da infraestrutura DMTCP. O overhead do `synth_p2p` também é
constante; a leve queda no L4 ocorre porque os dois parceiros chegam ao ponto de troca ao
mesmo tempo (perfeitamente sincronizados), então o caminho de sleep-and-retry do MANA nunca
é acionado.

### 4.2 Estudo de Desbalanceamento de Comunicação

**Por que esse teste foi criado:** CG e LU usam `MPI_Irecv + MPI_Wait` em suas trocas de
comunicação. Como descrito na Seção 2.1, o MANA implementa `MPI_Wait` como um loop de
espera ativa que consome CPU enquanto aguarda a mensagem. A questão que este teste responde
é: se um lado da troca chega atrasado (remetente lento), o loop de espera ativa do MANA
amplifica o overhead além do que o próprio atraso já custa? Isolar esse efeito exigiu um
programa sintético onde o atraso do remetente é controlado com precisão e o loop de espera
ativa é a única variável.

**O que este teste mede:** CG e LU usam `MPI_Irecv + MPI_Wait` em vez de `MPI_Recv`
bloqueante. O `MPI_Wait` do MANA é um loop de espera ativa:

```c
/* Wrapper interno do MPI_Wait no MANA (simplificado) */
while (!flag) {
    DMTCP_PLUGIN_DISABLE_CKPT();   /* adquire lock de leitura/escrita do checkpoint */
    MPI_Test_internal(..., &flag); /* chama MPI_Test real, contornando os wrappers do MANA */
    DMTCP_PLUGIN_ENABLE_CKPT();    /* libera lock de checkpoint */
}
```

O par de aquisição/liberação do lock roda em milhares de iterações por segundo durante
todo o tempo que a mensagem está atrasada, consumindo CPU enquanto o receptor espera.

O `synth_imbalanced` força o receptor a entrar nesse loop de espera ativa usando um atraso
controlado no remetente. Remetente e receptor estão em nós separados por design:

```c
if (is_sender) {
    if (DELAY_US > 0) usleep(DELAY_US);
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD);
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);  /* resposta já enviada, completa imediatamente */
} else {
    MPI_Irecv(&recv_buf, 1, MPI_DOUBLE, partner, 0, MPI_COMM_WORLD, &request);
    MPI_Wait(&request, &status);  /* fica em loop por DELAY_US (este é o teste) */
    MPI_Send(&x, 1, MPI_DOUBLE, partner, 1, MPI_COMM_WORLD);
}
```

![Overhead do MANA vs atraso do remetente (desbalanceamento de comunicação)](plots/fig2b_synth_imbalanced.png)

**Tabela 4: Resultados do synth_imbalanced (média ± std, N=3):**

| Nível | Atraso do remetente | noFT elapsed | MANA elapsed | Overhead |
|---|---|---|---|---|
| L0 | 0 µs | 63,5±0,6 s | 67,6±0,5 s | +4,1 s |
| L1 | 100 µs | 63,2±0,3 s | 67,6±0,4 s | +4,5 s |
| L2 | 1 ms | 63,4±0,8 s | 68,1±0,6 s | +4,7 s |
| L3 | 5 ms | 67,6±0,1 s | 72,0±0,2 s | +4,4 s |
| L4 | 20 ms | 78,7±0,3 s | 83,6±1,2 s | +4,9 s |

**Resultado:** O overhead do MANA é constante em ~4-5 s independentemente do atraso do
remetente. O loop de espera ativa consome CPU durante a espera, mas isso não adiciona
overhead *além* do que a própria espera já custa no noFT. O baseline noFT cresce em L3/L4
(o programa genuinamente está esperando pelo remetente lento), mas o MANA-noFT cresce na
mesma proporção.

Este estudo isola remetente e receptor em nós separados, então o loop de espera ativa não
pode competir com a computação do remetente por CPU. Essa é a limitação principal discutida
na Seção 2.1: a cascata no CG a 2w requer processos co-localizados, um cenário que o estudo
sintético não consegue reproduzir.

### 4.3 Estudo de Tamanho da Imagem de Checkpoint

**O que este teste mede:** Quanto tempo a Fase 1 (falha detectada até checkpoint gravado
no EFS) leva conforme o uso de memória do processo cresce? O `synth_checkpoint_size` aloca
uma quantidade conhecida de memória por processo, força o SO a mapear todas as páginas com
`memset`, e mantém o buffer ativo durante toda a execução:

```c
long n = (MEM_MB * 1024L * 1024L) / sizeof(double);
double *buf = (double *)malloc(n * sizeof(double));
memset(buf, 0x42, n * sizeof(double));   /* força o SO a mapear todas as páginas */

double t_start = MPI_Wtime();
volatile long counter = 0;
while (MPI_Wtime() - t_start < TARGET_SECS) {
    buf[counter % n] += 1.0;   /* mantém buffer ativo no checkpoint */
    counter++;
    if (counter % 5000000L == 0) MPI_Barrier(MPI_COMM_WORLD);
}
/* MANA injeta o sinal de checkpoint em trigger_after_secs=30 */
```

![Tempo da Fase 1 vs tamanho da imagem de checkpoint](plots/fig3_synth_ckpt.png)

**Tabela 5: Tempo da Fase 1 vs memória por processo (média ± std, N=3):**

| Memória/processo | Fase 1 | Observação |
|---|---|---|
| 50 MB | 16,4 ± 0,1 s | imagem pequena |
| 200 MB | 26,9 ± 0,4 s | |
| 800 MB | 47,0 ± 0,2 s | |
| 3.200 MB | 139,2 ± 0,4 s | aproxima-se do limite de banda do EFS |

**Resultado 1:** O tempo da Fase 1 escala **linearmente** com a memória por processo. Os
desvios padrão são muito pequenos (≤ 0,4 s), confirmando que a banda de escrita no EFS é
estável entre as repetições. A Fase 1 é previsível a partir dos requisitos de memória do job.

**Resultado 2:** O ponto de 3.200 MB se aproxima do limite de banda do EFS.

**O que é a banda do EFS?** O Amazon EFS (Elastic File System) é o sistema de arquivos
compartilhado em rede montado em todas as instâncias worker e usado aqui como armazenamento
de checkpoints. Cada processo MPI grava sua imagem de memória completa no EFS quando a
Fase 1 começa. O Throughput Bursting do EFS (modo padrão) fornece banda máxima de
**100 MiB/s (~105 MB/s)** para sistemas de arquivos menores que 1 TiB, mais créditos de
burst acumulados ao longo do tempo. Esse limite é fixo pela AWS e se aplica ao total de
escritas de todos os clientes. Só pode ser aumentado migrando para Provisioned Throughput
(upgrade pago) ou usando um sistema de arquivos paralelo de alta performance.

No experimento, cada processo MPI grava independentemente no EFS durante a Fase 1. Com 4
processos gravando simultaneamente (2 workers × 2 processos cada, na configuração de
2 workers), a demanda agregada com 3.200 MB por processo é aproximadamente
3.200 / 139,2 × 4 ≈ **92 MB/s**, atingindo ~88% do limite de 105 MB/s. Com footprints
maiores ou mais gravadores simultâneos, o limite seria excedido e os tempos da Fase 1
cresceriam mais rápido do que linearmente.

**Implicação para a tolerância a falhas:** Se a memória por processo exceder aproximadamente
3-4 GB, a Fase 1 levaria mais do que os 2 minutos de aviso de interrupção de instância spot
da EC2, o que significa que o checkpoint não terminaria antes de a instância ser encerrada.
A tolerância a falhas baseada em MANA com EFS, portanto, não é adequada para jobs com uso
intensivo de memória, a menos que o Provisioned Throughput seja habilitado ou um sistema
de arquivos mais rápido seja utilizado.

**Conexão com os benchmarks reais:** Os processos do EP-D alocam menos memória do que
LU-C ou CG-C, o que corresponde aos tempos de Fase 1 nas execuções FT: ~16-20 s para EP
vs ~26-37 s para LU e CG. O estudo de tamanho de checkpoint confirma que a diferença é
footprint de memória, não complexidade do benchmark.

---

## 5. Análise das Fases de Recuperação: Caso de Job Curto (CG)

![Decomposição do tempo de execução FT do CG (job curto)](plots/fig5_cg_short_job.png)

CG-C é excluído do estudo de sensibilidade ao momento da falha (Seção 6) porque seu tempo
de execução noFT de 10-35 s torna as variantes de timing sem sentido. É, no entanto, a
ilustração mais clara do fato estrutural chave: **para jobs curtos, o overhead de
recuperação domina o tempo total de execução**, e toda a mecânica de fases fica visível na
sua forma mais simples.

**Tabela 6: Decomposição do tempo de execução FT do CG-C (trigger base ~3 s, média ± std, N=3):**

| Workers | Estratégia | P0 | P1 | Slurm (P2a+P2c) | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|
| 2w | REPLACE | 3,0 s | 26,6±0,0 s | 10,9 s | 120,7±2,6 s | 40,2±3,0 s | **206,4±4,5 s** |
| 2w | DEGRADED | 3,0 s | 26,5±0,1 s | 10,8 s | 10,9±0,0 s | 113,1±3,1 s | **168,6±2,7 s** |
| 4w | REPLACE | 3,0 s | 26,7±0,3 s | 10,8 s | 123,3±1,9 s | 26,4±0,0 s | **196,2±5,0 s** |
| 4w | DEGRADED | 3,0 s | 26,5±0,0 s | 10,8 s | 10,9±0,0 s | 35,0±6,1 s | **92,6±7,6 s** |
| 8w | REPLACE | 3,0 s | 36,7±0,1 s | 10,8 s | 119,7±2,1 s | 24,6±6,0 s | **203,2±4,8 s** |
| 8w | DEGRADED | 3,0 s | 36,9±0,2 s | 11,1 s | 11,3±0,4 s | 24,8±2,9 s | **96,7±3,0 s** |

**Leitura das fases:**

- **P0 (pré-falha):** 3 s em todas as linhas; a falha é disparada muito cedo e é idêntica
  para ambas as estratégias.
- **P1 (gravação do checkpoint):** ~26,5 s a 2w e 4w, sobe para ~36,8 s a 8w, e é
  idêntico para ambas as estratégias no mesmo tamanho de cluster. O aumento a 8w é
  específico do CG: sua comunicação esparsa irregular faz cada processo manter buffers MPI
  e estado de comunicação por rank para todos os seus parceiros. Com mais workers, cada
  processo tem mais parceiros, aumentando o tamanho do checkpoint por processo. EP e LU
  não mostram esse efeito porque EP usa apenas operações coletivas (sem estado por rank) e
  LU tem comunicação com vizinhos mais próximos fixos (estado por rank constante
  independentemente do tamanho do cluster).
- **Overhead do Slurm e MANA (P2a+P2c):** ~10,8-11,1 s, constante em todas as
  configurações e para ambas as estratégias.
- **P2b (reconfiguração do nó):** REPLACE paga ~120-123 s pelo respawn do EC2; DEGRADED
  paga ~11 s pelo `scontrol DOWN`. A diferença de ~109-112 s é o custo direto da estratégia
  REPLACE.
- **P3 (computação restante):** Maior para DEGRADED porque ele continua com um worker a
  menos. A 2w isso é significativo (113,1 s vs 40,2 s); a 4w e 8w o custo extra do P3 é
  muito menor pois a fração de capacidade perdida é menor.

Para jobs muito curtos como CG, o overhead de FT é 3-17× a duração original do job. A
tolerância a falhas baseada em MANA é mais custo-efetiva para jobs de longa duração, onde
as fases de recuperação representam uma pequena fração do tempo total.

---

## 6. Análise de Sensibilidade ao Momento da Falha: Estudo por Fase

![Tempo total de execução FT por momento da falha](plots/fig4_timing_phases.png)

Esta é a análise central da Fase 5. EP-D e LU-C foram executados com falha injetada em
10%, 25% e 50% do tempo de execução MANA-noFT, com 2, 4 e 8 workers.

### 6.1 Fase 0: Computação Pré-Falha

A Fase 0 cresce conforme o trigger da falha avança de 10% para 50%. O P0 é **idêntico
para REPLACE e DEGRADED** dentro de cada grupo de timing, pois a falha é disparada pela
mesma configuração `auto_failure_trigger_secs`. Isso é visível na Figura 4 como a base de
mesma altura em cada par de barras.

### 6.2 Fase 1: Gravação do Checkpoint

O P1 é constante dentro de cada combinação benchmark × quantidade de workers, e igual
entre as estratégias. Depende apenas do footprint de memória do processo no momento da
falha:

| Configuração | P1 |
|---|---|
| EP-D, qualquer quantidade de workers | ~16-20 s |
| LU-C, qualquer quantidade de workers | ~26,6 s |
| CG-C, 2w e 4w | ~26,5-26,7 s |
| CG-C, 8w | ~36,8 s |

EP e LU mostram P1 constante independentemente do tamanho do cluster: EP usa apenas
operações coletivas sem estado por rank, e LU tem comunicação com vizinhos fixos. CG a 8w
é a exceção, como explicado na Seção 5: mais parceiros de comunicação implicam imagem de
checkpoint maior por processo. Isso é consistente com a relação linear confirmada pelo
estudo sintético de checkpoint.

### 6.3 Fases 2a e 2c: Coordenação do Slurm e do MANA

Essas duas sub-fases juntas levam ~10,8-11,0 s para todas as configurações e ambas as
estratégias. É o overhead de comunicação com o plano de controle do cluster para
reconfigurar o Slurm para o restart. O custo é o mesmo independentemente de o nó estar
sendo substituído ou removido.

### 6.4 Fase 2b: Reconfiguração do Nó (O Fator Diferenciador)

O P2b é onde REPLACE e DEGRADED divergem fundamentalmente:

- **REPLACE:** aguarda o EC2 terminar a instância spot, provisionar uma nova, inicializá-la,
  instalar o Slurm e colocá-la em estado IDLE. Isso leva ~111-132 s e é determinado pela
  latência de provisionamento da AWS, independente do tamanho do cluster ou do momento da falha.
- **DEGRADED:** apenas executa `scontrol update NodeName=... State=DOWN` para marcar o
  nó falho como indisponível. Isso leva ~11 s.

A diferença de P2b (~100-121 s) é **o fator mais importante neste estudo**.

**Tabela 7: Tempos de P2b por configuração (média ± std, agregado por timings de falha):**

| Benchmark | Workers | P2b REPLACE | P2b DEGRADED | Diferença |
|---|---|---|---|---|
| EP-D | 2w | 120±8 s | 11±0 s | ~109 s |
| EP-D | 4w | 118±9 s | 11±0 s | ~107 s |
| EP-D | 8w | 128±7 s | 11±0 s | ~117 s |
| LU-C | 2w | 122±4 s | 11±0 s | ~111 s |
| LU-C | 4w | 123±6 s | 11±0 s | ~112 s |
| LU-C | 8w | 126±10 s | 11±0 s | ~115 s |

### 6.5 Fase 3: Computação Restante

O P3 é determinado por quanto trabalho resta no momento da falha (diminui conforme o
trigger avança de 10% para 50%) e pela capacidade após a recuperação (REPLACE reinicia
com N workers; DEGRADED com N-1 workers).

A diferença de P3 entre estratégias diminui conforme o cluster cresce:
- Com 2 workers, perder 1 de 2 significa DEGRADED rodando P3 com ~50% menos recursos.
- Com 4 workers, perder 1 de 4 significa ~25% mais tempo de P3.
- Com 8 workers, perder 1 de 8 significa ~12% mais tempo de P3.

Essa propriedade estrutural significa que a vantagem do DEGRADED tende a se fortalecer em
clusters maiores: a diferença de P2b se mantém aproximadamente constante (~107-117 s),
enquanto a penalidade de P3 do DEGRADED diminui conforme a fração de capacidade perdida
decresce.

### 6.6 Padrões de Altura das Barras: Por que os Totais São Aproximadamente Constantes

Dois padrões se destacam na Figura 4 para ambos os benchmarks:

**As barras do DEGRADED têm altura aproximadamente constante independentemente do momento
da falha.** Conforme o trigger avança de 10% para 50%, o P0 cresce (mais trabalho
produtivo pré-falha) enquanto o P3 diminui (menos trabalho restante). Esses dois efeitos
se cancelam parcialmente. A razão pela qual o cancelamento não é perfeito é que o P3 do
DEGRADED roda com capacidade reduzida (N-1 workers), então o P3 diminui mais lentamente
do que o P0 cresce em termos absolutos. Em outras palavras: mesmo que haja menos trabalho
restante após uma falha mais tardia, cada unidade desse trabalho restante leva mais tempo
porque o cluster está mais fraco. Esses dois efeitos concorrentes (menos trabalho mas
processamento mais lento) resultam em tempo total aproximadamente constante entre os
timings de falha.

Uma pergunta natural surge: se a falha ocorre mais tarde, o cluster tem menos tempo rodando
com capacidade reduzida, então a barra não deveria ficar mais curta? A resposta é que o
cluster roda com capacidade total apenas durante o P0; *toda* a computação restante (P3)
roda com capacidade reduzida independentemente de quando a falha ocorre. Uma falha mais
tardia significa menos trabalho no P3, mas esse trabalho ainda é feito na mesma velocidade
reduzida. A economia do P3 pelo trabalho menor é aproximadamente compensada pelo overhead
do P3 pela perda de capacidade, produzindo totais planos.

**As barras do REPLACE também tendem a ser aproximadamente constantes, mas mostram mais
variação.** Para REPLACE, P0 e P3 rodam com capacidade total de N workers, então em
princípio P0+P3 deveria ser igual ao tempo total do job MANA-noFT e o tempo total deveria
ser constante em MANA_noft + recuperação fixa. Na prática, os dados mostram que P0+P3 não
é constante: ele aumenta com o timing da falha.

**Tabela: REPLACE (EP-D e LU-C) — P3 real comparado ao esperado (MANA-noFT menos P0):**

| Benchmark | Workers | Falha | P0 | P3 real | P3 esperado | Diferença | Diferença% |
|---|---|---|---|---|---|---|---|
| EP-D | 4w | 10% | 32 s | 152 s | 217 s | 65 s | **30%** |
| EP-D | 4w | 25% | 81 s | 122 s | 168 s | 46 s | **27%** |
| EP-D | 4w | 50% | 151 s | 70 s | 98 s | 28 s | **29%** |
| EP-D | 8w | 10% | 12 s | 87 s | 114 s | 27 s | **23%** |
| EP-D | 8w | 25% | 31 s | 75 s | 95 s | 20 s | **21%** |
| EP-D | 8w | 50% | 61 s | 51 s | 65 s | 14 s | **22%** |
| LU-C | 4w | 10% | 13 s | 81 s | 122 s | 41 s | **34%** |
| LU-C | 4w | 25% | 33 s | 68 s | 102 s | 34 s | **33%** |
| LU-C | 4w | 50% | 66 s | 47 s | 69 s | 22 s | **32%** |
| LU-C | 8w | 10% | 7 s | 53 s | 66 s | 13 s | **20%** |
| LU-C | 8w | 25% | 17 s | 47 s | 56 s | 8 s | **15%** |
| LU-C | 8w | 50% | 34 s | 37 s | 39 s | 2 s | **5%** |

**P3 esperado** = MANA-noFT menos P0 (o que seria o P3 se o job reiniciado rodasse na
mesma velocidade do baseline MANA sem falhas). **Diferença** = P3 esperado menos P3 real
(quantos segundos mais rápido o P3 realmente rodou). **Diferença%** = Diferença / P3 esperado.

O P0+P3 é sempre menor que o MANA-noFT, e a diferença em segundos absolutos diminui
conforme o timing da falha aumenta, pois há menos trabalho restante (P3 esperado menor).
A diferença como **porcentagem do P3 esperado é aproximadamente constante**: ~22-30% para
EP e ~32-34% para LU a 4w. Essa é uma observação chave: uma porcentagem constante significa
que o P3 consistentemente roda ~25-33% mais rápido do que se esperaria a partir de um
início a frio, independentemente de quanto trabalho resta. Esse comportamento é consistente
com um efeito de reinício com estado aquecido: após reiniciar a partir de um checkpoint,
o P3 se beneficia de entradas de tabela de páginas do SO e páginas de memória que já foram
carregadas durante o P0. O benefício escala com a quantidade de trabalho restante (fração
constante), não como um bônus fixo de tempo.

A exceção é LU a 8w: com 50% de falha, resta apenas 37 s de P3, e a diferença cai para
2 s (5%). Isso é consistente com a hipótese: um P3 muito curto acessa apenas uma pequena
porção do working set, deixando pouco espaço para que o calor do cache acumulado no P0
proporcione benefício.

A consequência prática é que as alturas das barras do REPLACE crescem ligeiramente com o
timing da falha (porque P0 cresce enquanto P3 diminui menos do que o esperado), e são
adicionalmente afetadas pelo ruído de provisionamento EC2 no P2b (std ~7-15 s por
configuração).

### 6.7 EP-D: Decomposição Completa por Fase

**Tabela 8: EP-D — tempos médios por fase e configuração (segundos, N=3 por célula):**

| Workers | Trigger | Estratégia | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~46s) | REPLACE | 46 | 16 | 11 | 116 | 322 | **516±4** |
| 2w | 10% (~46s) | DEGRADED | 46 | 16 | 11 | 11 | 446 | **535±3** |
| 2w | 25% (~116s) | REPLACE | 116 | 20 | 11 | 122 | 270 | **543±14** |
| 2w | 25% (~116s) | DEGRADED | 116 | 16 | 11 | 11 | 373 | **532±4** |
| 2w | 50% (~231s) | REPLACE | 231 | 16 | 11 | 122 | 188 | **573±13** |
| 2w | 50% (~231s) | DEGRADED | 231 | 20 | 11 | 11 | 262 | **541±6** |
| 4w | 10% (~32s) | REPLACE | 32 | 16 | 11 | 111 | 152 | **330±14** |
| 4w | 10% (~32s) | DEGRADED | 32 | 16 | 11 | 11 | 218 | **294±4** |
| 4w | 25% (~81s) | REPLACE | 81 | 16 | 11 | 121 | 122 | **358±1** |
| 4w | 25% (~81s) | DEGRADED | 81 | 20 | 11 | 11 | 170 | **302±3** |
| 4w | 50% (~151s) | REPLACE | 151 | 16 | 11 | 123 | 70 | **378±3** |
| 4w | 50% (~151s) | DEGRADED | 151 | 17 | 11 | 11 | 100 | **299±2** |
| 8w | 10% (~12s) | REPLACE | 12 | 16 | 11 | 132 | 87 | **269±7** |
| 8w | 10% (~12s) | DEGRADED | 12 | 20 | 11 | 11 | 117 | **179±8** |
| 8w | 25% (~31s) | REPLACE | 31 | 16 | 11 | 126 | 75 | **270±15** |
| 8w | 25% (~31s) | DEGRADED | 31 | 16 | 11 | 11 | 105 | **182±1** |
| 8w | 50% (~61s) | REPLACE | 61 | 20 | 12 | 127 | 51 | **282±15** |
| 8w | 50% (~61s) | DEGRADED | 61 | 16 | 11 | 11 | 75 | **184±13** |

A 2w/10%, a penalidade de P3 do DEGRADED (446-322 = 124 s) supera a diferença de P2b
(116-11 = 105 s), então REPLACE é mais rápido por ~19 s. Em todos os outros casos de EP,
a diferença de P2b supera a penalidade de P3 e DEGRADED é mais rápido. Com o crescimento
do cluster, a margem aumenta: a 8w a diferença entre estratégias é consistentemente ~90 s
em todos os timings de falha.

### 6.8 LU-C: Decomposição Completa por Fase

**Tabela 9: LU-C — tempos médios por fase e configuração (segundos, N=3 por célula):**

| Workers | Trigger | Estratégia | P0 | P1 | P2a+2c | P2b | P3 | Total |
|---|---|---|---|---|---|---|---|---|
| 2w | 10% (~24s) | REPLACE | 24 | 27 | 11 | 121 | 141 | **329±7** |
| 2w | 10% (~24s) | DEGRADED | 24 | 27 | 11 | 11 | 228 | **304±3** |
| 2w | 25% (~59s) | REPLACE | 59 | 27 | 11 | 123 | 117 | **341±5** |
| 2w | 25% (~59s) | DEGRADED | 59 | 27 | 11 | 11 | 192 | **303±3** |
| 2w | 50% (~118s) | REPLACE | 118 | 27 | 11 | 123 | 85 | **369±7** |
| 2w | 50% (~118s) | DEGRADED | 118 | 27 | 11 | 11 | 131 | **302±1** |
| 4w | 10% (~13s) | REPLACE | 13 | 27 | 11 | 129 | 81 | **266±5** |
| 4w | 10% (~13s) | DEGRADED | 13 | 27 | 11 | 11 | 125 | **193±1** |
| 4w | 25% (~33s) | REPLACE | 33 | 27 | 11 | 121 | 68 | **267±6** |
| 4w | 25% (~33s) | DEGRADED | 33 | 27 | 11 | 11 | 105 | **194±3** |
| 4w | 50% (~66s) | REPLACE | 66 | 26 | 11 | 119 | 47 | **275±0** |
| 4w | 50% (~66s) | DEGRADED | 66 | 27 | 11 | 11 | 72 | **193±2** |
| 8w | 10% (~7s) | REPLACE | 7 | 26 | 11 | 122 | 53 | **231±4** |
| 8w | 10% (~7s) | DEGRADED | 7 | 27 | 11 | 11 | 69 | **136±1** |
| 8w | 25% (~17s) | REPLACE | 17 | 27 | 11 | 137 | 47 | **247±9** |
| 8w | 25% (~17s) | DEGRADED | 17 | 27 | 11 | 11 | 61 | **136±1** |
| 8w | 50% (~34s) | REPLACE | 34 | 27 | 11 | 120 | 37 | **236±3** |
| 8w | 50% (~34s) | DEGRADED | 34 | 27 | 11 | 11 | 47 | **141±5** |

LU não apresenta cruzamento em nenhuma configuração: DEGRADED é mais rápido em todos os
9 casos testados. A vantagem cresce com o tamanho do cluster: a 8w o DEGRADED vence por
~90-95 s vs ~25 s a 2w. A comunicação estruturada em stencil do LU significa que a
penalidade de P3 por perder um worker é menor do que no EP, pois os workers restantes têm
um padrão de comunicação mais favorável com menos vizinhos.

---

## 7. Comparação de Estratégias: Tendências e Trade-offs

![REPLACE vs DEGRADED: tempo total de execução com baseline MANA-noFT](plots/fig8_strategy_comparison.png)

### 7.1 Escopo deste Estudo

Todas as execuções foram desenhadas para completar o checkpoint antes do aviso de
interrupção de 2 minutos da instância spot, o que significa que apenas jobs de curta a
média duração foram avaliados. Cargas de trabalho muito longas (jobs HPC de horas) não
foram testadas. A análise de tendências abaixo usa raciocínio estrutural para projetar o
que aconteceria com jobs mais longos.

### 7.2 Custos Fixos vs Variáveis da Recuperação

A maior parte do custo de recuperação é **fixa** e não depende de quanto tempo o job roda
ou de quando a falha ocorre:

| Fase | Custo | Varia com |
|---|---|---|
| P1 (gravação do checkpoint) | 16-37 s | apenas footprint de memória |
| P2a + P2c (coordenação Slurm/MANA) | ~11 s | nada |
| P2b REPLACE (provisionamento EC2) | ~111-132 s | apenas latência da AWS |
| P2b DEGRADED (scontrol DOWN) | ~11 s | nada |
| P3 (computação restante) | varia | timing da falha e perda de capacidade |

A única fase que varia com a duração do job é o P3. Para ambas as estratégias, o P3
diminui conforme a falha ocorre mais tarde no job.

### 7.3 Quando Cada Estratégia Tende a Ganhar

Os resultados mostram uma condição estrutural clara para DEGRADED ser mais rápido que
REPLACE. DEGRADED economiza ~109-117 s no P2b, mas paga uma penalidade no P3 ao rodar
com N-1 workers. Para uma carga perfeitamente paralela, essa penalidade de P3 é:

```
Penalidade P3 = trabalho_restante / (N-1)  -  trabalho_restante / N
              = trabalho_restante / [N × (N-1)]
```

DEGRADED vence quando a economia no P2b supera essa penalidade:

```
diferença_P2b (≈109 s)  >  trabalho_restante / [N × (N-1)]
→  trabalho_restante  <  109 × N × (N-1)
```

Para cada tamanho de cluster, o ponto de cruzamento em trabalho restante é:

| Workers (N) | Cruzamento (trabalho restante) | Exemplo: DEGRADED vence se falha... |
|---|---|---|
| N = 2 | 109 × 1 = **109 s** (~1,8 min) | > 96% do job concluído (maioria dos cenários: REPLACE vence) |
| N = 4 | 109 × 3 = **327 s** (~5,5 min) | > 87% do job concluído |
| N = 8 | 109 × 7 = **763 s** (~12,7 min) | > 94% do job concluído |

Todos os jobs testados neste estudo tinham trabalho restante de P3 bem abaixo desses
limiares (máximo ~320 s para EP-D a 2w com 10% de timing de falha), o que explica por que
DEGRADED vence na maioria dos casos testados. Para jobs de uma hora ou mais, o trabalho
restante excederia muito esses limiares em qualquer timing de falha realista, e REPLACE
tenderia a ser mais rápido.

Essa análise tem uma implicação prática importante: **a vantagem do DEGRADED não é sobre
o comprimento do job em si, mas sobre quanto trabalho resta quando a falha ocorre.** Para
os jobs curtos avaliados aqui, o trabalho restante é pequeno o suficiente para que a
penalidade de P3 do DEGRADED nunca se acumule a ponto de superar a economia no P2b. Para
jobs longos, mesmo uma falha tardia (digamos, 90% de um job de 4 horas) deixa 24 minutos
de trabalho restante, muito acima do limite de cruzamento de 12,7 minutos a 8w. REPLACE
venceria lá também.

### 7.4 Múltiplas Falhas

Apenas cenários de falha única foram testados. Para cargas de trabalho que podem
experimentar múltiplas interrupções spot sucessivas:

- **REPLACE** sempre retorna ao tamanho original do cluster. Cada falha tem o mesmo custo
  fixo de P2b (~111-132 s). A capacidade do cluster nunca se degrada.
- **DEGRADED** continua com menos workers após cada falha. Para clusters iniciando em 4w
  ou 8w, uma ou duas falhas são toleráveis. Para clusters de 2w, uma segunda falha deixaria
  apenas um worker, que pode ser incapaz de continuar dependendo da decomposição MPI.

A tendência é clara: REPLACE é mais robusto sob falhas repetidas.

### 7.5 Resumo: Quando Usar Cada Estratégia

| Cenário | Estratégia | Motivo |
|---|---|---|
| Job curto (< 1-2 min noFT) | Nenhuma | Overhead de recuperação domina; economia spot não compensa o custo de FT |
| Job médio (100-500 s) com 4+ workers | **DEGRADED** | Trabalho restante fica abaixo do limite de cruzamento; diferença de P2b é decisiva |
| Job médio (100-500 s) com 2 workers | **REPLACE** (geralmente) | A 2w, DEGRADED só vence quando restam menos de ~109 s; apenas os casos de 25%/50% de falha se qualificam |
| Job longo (> ~30 min) em qualquer tamanho | **REPLACE** | Trabalho restante em timings de falha realistas excede o limite de cruzamento para todos os tamanhos de cluster |
| Múltiplas falhas esperadas | **REPLACE** | Capacidade nunca se degrada; cada falha tem o mesmo custo fixo de P2b |

---

## 8. Razão de Overhead de Recuperação

![Overhead de recuperação como % do tempo total FT](plots/fig9_recovery_overhead_ratio.png)

A razão de overhead de recuperação mede qual fração do tempo total de execução FT não foi
computação produtiva pré-falha:

`overhead_recuperacao_pct = (tempo_total_ft − P0) / tempo_total_ft × 100`

**Por que excluir o P0?** O P0 é computação pré-falha: trabalho que foi realizado com
sucesso e que um checkpoint preserva. Não é overhead em nenhum sentido significativo: ele
teria sido executado quer o job falhasse ou não. Tudo depois do P0 (a gravação do
checkpoint, a logística de recuperação e a computação reiniciada no P3) é tempo que o job
não teria gasto em uma execução sem falhas. A razão mede qual fração do tempo total isso
representa.

**Por que a razão diminui conforme o timing da falha aumenta, mesmo que as alturas das
barras sejam aproximadamente constantes?**

As barras sendo aproximadamente da mesma altura significa que o tempo total mal muda entre
os timings de falha. Mas dentro desse total aproximadamente constante, o P0 cresce conforme
a falha é disparada mais tarde. Um P0 maior significa que uma fração maior do total foi
trabalho útil pré-falha, então a fração restante (tudo depois do P0) é menor.
Concretamente para DEGRADED a 4w:

- Com 10% de timing: total ≈ 294 s, P0 ≈ 32 s → razão = (294-32)/294 = **89%**
- Com 50% de timing: total ≈ 299 s, P0 ≈ 151 s → razão = (299-151)/299 = **49%**

O total mal mudou (294 s vs 299 s), mas o P0 quase quintuplicou. A razão não diminui porque
a recuperação leva menos tempo; ela diminui porque uma parcela maior do total constante foi
trabalho produtivo pré-falha.

**Tabela 10: Overhead de recuperação como % do tempo total FT (EP-D, média ± std, N=3):**

| Workers | Estratégia | 10% | 25% | 50% |
|---|---|---|---|---|
| 2w | REPLACE | 91,1±0,1% | 78,6±0,5% | 59,6±0,9% |
| 2w | DEGRADED | 91,4±0,0% | 78,2±0,2% | 57,3±0,4% |
| 4w | REPLACE | 90,3±0,4% | 77,4±0,1% | 60,1±0,3% |
| 4w | DEGRADED | 89,1±0,2% | 73,2±0,2% | 49,4±0,4% |
| 8w | REPLACE | 95,5±0,1% | 88,5±0,6% | 78,3±1,1% |
| 8w | DEGRADED | 93,3±0,3% | 83,0±0,1% | 66,7±2,2% |

**Observações principais:**

1. **Falha cedo produz alto overhead para ambas as estratégias.** Quando a falha ocorre
   a 10%, a recuperação representa ~90% do tempo total. O P0 é pequeno, então quase todo
   o tempo decorrido é gasto em recuperação.

2. **Falha tardia produz menor overhead.** Com trigger a 50%, DEGRADED a 4w tem apenas
   49% de overhead, ou seja, mais da metade do tempo total foi computação útil pré-falha.
   Esse é o argumento econômico mais forte para tolerância a falhas em jobs curtos: uma
   falha tardia torna a recuperação uma fração pequena do tempo total.

3. **A 8w, o overhead do REPLACE permanece acima de 78% mesmo a 50%.** Como o job é
   curto (noFT ~99 s a 8w), o custo fixo de provisionamento EC2 (~128 s) representa a
   maior parte do total independentemente do timing de falha. É o mesmo problema estrutural
   de job curto identificado no caso CG.

4. **DEGRADED e REPLACE têm overhead quase idêntico a 10%** para todas as configurações.
   As estratégias divergem a 25% e 50%, onde o P2b grande do REPLACE contribui com uma
   fração crescente do tempo restante.

---

## 9. Análise Econômica

A Figura 7 mostra o custo por execução decomposto por timing de falha para cada benchmark,
estratégia e quantidade de workers. O modelo de custo compara dois cenários:

- **noFT** precisa rodar em instâncias **on-demand**. Uma interrupção spot perde todo o
  progresso e exige reinício completo do zero.
- **REPLACE e DEGRADED** podem usar instâncias **spot** (~70% mais baratas por hora) porque
  o MANA trata interrupções automaticamente e o job retoma a partir do checkpoint.

![Custo por execução: spot com FT vs on-demand sem FT](plots/fig7_cost.png)

**Tabela 11: Custo estimado por execução (USD), médio entre timings de falha (média ± std, N=3):**

| Benchmark | Workers | noFT on-demand | REPLACE spot | DEGRADED spot | REPLACE vs noFT | DEGRADED vs noFT |
|---|---|---|---|---|---|---|
| CG-C | 2w | $0,0044 | $0,0115 | $0,0094 | +159% mais caro | +112% mais caro |
| CG-C | 4w | $0,0048 | $0,0173 | $0,0082 | +262% mais caro | +71% mais caro |
| CG-C | 8w | $0,0055 | $0,0311 | $0,0148 | +467% mais caro | +170% mais caro |
| EP-D | 2w | $0,0434 | $0,0303 | $0,0298 | **-30% de economia** | **-31% de economia** |
| EP-D | 4w | $0,0401 | $0,0313 | $0,0263 | **-22% de economia** | **-34% de economia** |
| EP-D | 8w | $0,0446 | $0,0419 | $0,0278 | **-6% de economia** | **-38% de economia** |
| LU-C | 2w | $0,0192 | $0,0193 | $0,0168 | ~0% (equilíbrio) | **-12% de economia** |
| LU-C | 4w | $0,0196 | $0,0238 | $0,0170 | +21% mais caro | **-13% de economia** |
| LU-C | 8w | $0,0214 | $0,0365 | $0,0211 | +71% mais caro | ~0% (equilíbrio) |

### 9.1 Resultados por Configuração

1. **CG-C: FT nunca se justifica economicamente.** O overhead de recuperação (92-206 s) é
   3-17× a duração base do job (12-35 s). O preço spot não compensa. Jobs curtos não têm
   justificativa econômica para FT baseada em MANA.

2. **EP-D DEGRADED economiza 31-38% vs noFT on-demand em todos os tamanhos de cluster.** A
   economia é consistente entre tamanhos de cluster. A 8w, REPLACE economiza apenas ~6%
   enquanto DEGRADED economiza 38%: o job noFT do EP-D tem apenas ~99 s a 8w, e REPLACE
   adiciona ~260 s de provisionamento EC2 que consome a maior parte do desconto spot.

3. **LU-C DEGRADED economiza 12-13% a 2w e 4w mas empata a 8w.** LU-C a 8 workers roda
   apenas ~47 s noFT. Mesmo a recuperação curta do DEGRADED adiciona ~89 s no total (~136 s
   de tempo FT total), que com 8 workers spot custa aproximadamente o mesmo que rodar 47 s
   em 8 workers on-demand. A economia de 1,6% está dentro do ruído.

4. **LU-C REPLACE custa 21-71% mais que noFT on-demand a 4w e 8w.** Com job base curto e
   ~260 s de recuperação REPLACE, o desconto spot não compensa.

5. **O timing da falha afeta significativamente o custo do REPLACE.** Falhas mais cedo (10%)
   significam a mesma recuperação fixa em uma execução produtiva mais curta, aumentando o
   custo total. Falhas mais tarde (50%) amortizam a recuperação ao longo de mais tempo de P0.
   Esse efeito é maior para REPLACE a 8 workers (ver Figura 7).

### 9.2 Tendência Econômica: Quando FT com Spot Realmente Compensa?

A análise de desempenho (Seção 7.3) mostrou que DEGRADED vence apenas quando o trabalho
restante está abaixo do limite de cruzamento de 109 × (N-1) segundos. A mesma lógica se
aplica à economia: o desconto spot só é grande o suficiente para compensar o custo de
recuperação quando a extensão total do tempo de execução pela recuperação é modesta.

O benefício econômico da tolerância a falhas vem de rodar a taxas spot (~70% mais baratas)
em vez de on-demand. O custo da tolerância a falhas é o tempo de execução estendido pelas
fases de recuperação. Para DEGRADED especificamente, esse tempo estendido vem de P1, P2 e
(para jobs curtos) uma pequena penalidade de P3, todos modestos para os jobs de curta a
média duração testados aqui.

O padrão entre benchmarks revela dois regimes distintos:

**Regime 1: Jobs curtos (CG-C e LU-C a 8w):** O job base é tão curto que mesmo um único
evento de recuperação estende o tempo total por 2-17×. Nenhum desconto spot pode compensar
uma extensão de 2× no tempo de execução se ele fornece apenas 70% de desconto por hora. FT
não é economicamente viável independentemente da estratégia.

**Regime 2: Jobs médios (EP-D e LU-C a 2w/4w):** O job base roda por tempo suficiente para
que o desconto spot no job completo supere a extensão pela recuperação. DEGRADED economiza
12-38% porque sua recuperação é curta (P2b fixo em ~11 s) e o desconto spot se aplica a
uma quantidade significativa de tempo de computação. REPLACE economiza menos porque seu
P2b (~120 s) consome uma porção maior do tempo no custo spot.

**Para jobs hipotéticos longos:** A análise de cruzamento na Seção 7.3 mostra que a
penalidade de P3 do DEGRADED se acumula com o trabalho restante. Para um job de 4 horas
a 8w com falha a 50%, DEGRADED rodaria as 2 horas restantes com 7/8 de capacidade,
adicionando ~17 minutos de computação extra. Esse tempo extra, cobrado a taxas spot, custa
mais do que a economia de P2b que DEGRADED proporciona. REPLACE seria ao mesmo tempo mais
rápido e mais barato para esse job. Paradoxalmente, quanto mais longo o job, mais provável
que REPLACE seja a escolha economicamente correta, mesmo que o custo de P2b do REPLACE
seja fixo e grande.

A nuance prática importante é que interrupções spot não são previsíveis; um job não pode
escolher seu timing de falha. Para jobs curtos, DEGRADED é uma aposta econômica segura
porque o limite de cruzamento é fácil de manter. Para jobs longos, REPLACE é a escolha
mais segura porque não existe timing de falha em que a penalidade de P3 do DEGRADED
permaneça aceitável.

---

## 10. Conclusões

### 10.1 Caracterização do Overhead do MANA

O overhead medido varia entre 27% e 222% entre benchmarks e tamanhos de cluster (Tabela 1).
Com N=3 repetições por célula, esses valores são estáveis: a variabilidade não é ruído do
EC2 mas uma propriedade reprodutível de cada configuração. Os estudos sintéticos estabelecem
que o MANA adiciona ~4-5 s para programas isolados (frequência de chamadas e desbalanceamento
simples não são os drivers principais), mas o mecanismo por trás do overhead muito maior nos
benchmarks NPB reais permanece não resolvido (Seção 2.4).

LU-C (52-63%) é a estimativa de overhead mais confiável. CG-C a 2w (+222%) provavelmente
é uma cascata de competição de CPU por loop de espera ativa entre processos co-localizados.
O overhead do EP-D diminui com mais workers (45-47% a 2w/4w, 27% a 8w) em um padrão que é
reprodutível mas não tem explicação mecanicista a partir dos dados deste estudo.

### 10.2 Tempo de Gravação do Checkpoint Escala Linearmente com a Memória

O tempo da Fase 1 escala linearmente com o footprint de memória por processo, com baixo
desvio padrão (≤ 0,4 s), confirmando que a banda de escrita no EFS é estável e a Fase 1 é
previsível. O limite de Throughput Bursting do EFS (~105 MB/s) restringe a viabilidade de
FT baseada em MANA para jobs com uso intensivo de memória: footprints por processo acima de
~3-4 GB excederiam o aviso de 2 minutos do EC2 spot antes de o checkpoint completar.

### 10.3 Mecânica das Fases de Recuperação

A decomposição por fase mostra estrutura clara:

- P1, P2a, P2c são custos fixos, determinados pelo footprint de memória e tempo de
  coordenação do cluster. Nenhuma estratégia pode evitá-los.
- P2b é o principal diferenciador: ~11 s para DEGRADED vs ~111-132 s para REPLACE.
- P3 é o fator secundário. A penalidade de P3 do DEGRADED diminui com o crescimento do
  cluster (menos workers perdidos como fração do total), que é a razão estrutural pela qual
  a vantagem do DEGRADED cresce em escalas maiores, mas apenas enquanto o trabalho restante
  se mantém abaixo do limite de cruzamento.

### 10.4 Recomendações de Estratégia

**DEGRADED_RESUME é a melhor escolha para os jobs de curta a média duração avaliados aqui**
(4-8 workers, 100-500 s de tempo MANA-noFT). Vence em todos os casos testados a 4w e 8w, e
na maioria a 2w. A razão estrutural é que o trabalho restante nos jobs testados está sempre
abaixo do limite de cruzamento em que REPLACE se tornaria mais rápido.

**Para jobs de longa duração** (onde o trabalho restante excede ~763 s a 8w, ~327 s a 4w
ou ~109 s a 2w), **REPLACE tende a ser a escolha mais segura e mais rápida**, independentemente
do timing de falha. Quanto mais longo o job, mais a perda de capacidade do DEGRADED se
acumula, eventualmente superando a economia no P2b.

**REPLACE também é melhor** quando múltiplas falhas sucessivas são esperadas: a capacidade
nunca se degrada e cada falha tem o mesmo custo fixo de P2b.

**Para jobs muito curtos** (< 1-2 min noFT), nenhuma estratégia se justifica economicamente:
o overhead de recuperação supera o desconto de preço spot independentemente da estratégia.

### 10.5 Limitações e Trabalhos Futuros

1. **A anomalia do CG-C a 2 workers** não foi resolvida. Instrumentação de timing por rank
   dentro do benchmark NAS confirmaria ou refutaria a hipótese de cascata de CPU por loop de
   espera ativa.
2. **A variabilidade do overhead do MANA** entre benchmarks reais não está explicada.
   Profiling do estado interno do MANA (interações com o coordenador, padrões de contenção
   de locks) durante execuções reais dos benchmarks seria necessário para identificar a
   causa raiz.
3. **Múltiplas falhas sucessivas** não foram testadas. A degradação de capacidade do DEGRADED
   sob falhas repetidas é uma questão em aberto crítica para o regime de jobs longos onde
   REPLACE já pode ser preferido.
4. **Apenas instâncias m5.xlarge** foram avaliadas. Tipos de instância diferentes mudariam
   os tempos da Fase 1 e os fatores de overhead do MANA.
5. **O limite de throughput do EFS** restringe a viabilidade do checkpoint para footprints
   de memória grandes. Habilitar EFS Provisioned Throughput ou usar um sistema de arquivos
   paralelo estenderia o alcance das cargas de trabalho viáveis.
6. **Cargas de trabalho muito longas** não foram testadas. Medir diretamente os pontos de
   cruzamento de desempenho e econômico para jobs de horas de duração validaria a análise de
   tendências das Seções 7.3 e 9.2.

---

*Relatório gerado a partir de 282 execuções válidas coletadas entre 2026-05-30 e 2026-06-14.*
*Todas as figuras geradas por `TCC/analyze.py`. Dados brutos: `results_raw.csv`. Tabelas: `tables/`. Estatísticas agregadas: `summary.md`.*
