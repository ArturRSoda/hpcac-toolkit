# Roteiro de Apresentação — SIC UFSC 36º

**Título:** Soluções para a Convergência entre Computação de Alto Desempenho e Computação em Nuvem
**Duração estimada:** ~4min 45s (limite: 5min)

---

## [Slide 1 — Título] ~10s

"Olá! Meu nome é Artur Soda, sou aluno de Ciências da Computação na UFSC, e vou apresentar meu trabalho de iniciação científica sobre tolerância a falhas em computação de alto desempenho na nuvem."

---

## [Slide 4 — HPC na Nuvem] ~25s

"Aplicações científicas exigem enorme poder computacional. Em vez de comprar servidores físicos, é possível alugar esse poder pela internet de empresas como a Amazon — a AWS. Quando os computadores da AWS ficam ociosos, ela os disponibiliza por preços muito menores, com descontos de até 90%. Essa modalidade se chama spot. Uma opção muito atrativa para pesquisa científica, mas que tem uma desvantagem importante."

---

## [Slide 5 — O Problema das Instâncias Spot] ~35s

"Pesquisadores veem no spot uma oportunidade: aproveitar o desconto para rodar aplicações científicas de alto desempenho — programadas com um padrão chamado MPI — que dividem a computação entre vários computadores trabalhando em paralelo. Mas há um problema: essas máquinas podem ser interrompidas a qualquer momento. Quando a AWS precisa delas de volta, ela as retoma com apenas 2 minutos de aviso. Se qualquer um desses computadores for interrompido, toda a execução para — e horas de processamento acumulado se perdem por completo."

---

## [Slide 8 — O que Este Trabalho Propõe] ~35s

"A solução clássica para esse problema é o checkpoint/restart — salvar periodicamente o progresso e retomar de onde parou em caso de falha. Mas as abordagens existentes têm duas limitações: exigem modificações no próprio programa, o que inviabiliza para aplicacaoes legadas, onde eh nao eh possivel ou muito dificil de alterar o codigo, ou entao dependem de endereços fixos de rede — condição que máquinas spot não garantem. Este trabalho propõe uma arquitetura que supera ambas: o checkpoint é transparente — nenhuma modificação no programa — funciona mesmo quando uma máquina é substituída por outra com endereço diferente, e todo o processo de detecção e recuperação é completamente automatizado."

---

## [Slide 9 — Arquitetura da Solução] ~25s

"Nossa solução funciona assim. O HPC@Cloud — sistema do nosso laboratório — gerencia o cluster na AWS. O MANA eh responsavel por realizar o checkpoint transparente: captura o estado completo da aplicação e o salva num armazenamento persistente na nuvem, o EFS, sem que o programa precise ser modificado. E o elemento central é o Watcher: que monitora os avisos de encerramento e, ao detectar um, aciona o MANA imediatamente e inicia a recuperação automática."

---

## [Slide 11 — Duas Estratégias de Recuperação] ~30s

"A estratégia de recuperação é definida pelo usuário antes de iniciar a execução. São duas opções. O Replace solicita uma nova máquina à AWS e, quando ela está pronta, retoma a execução com o cluster completo — isso leva cerca de 120 segundos. O Degraded não espera: reinicia imediatamente nos computadores que ainda estão ativos, em apenas 11 segundos, mas com menos poder de processamento. Replace garante capacidade total, mas exige mais espera; Degraded recupera rápido, porém com menos poder computacional."

---

## [Slide 13 — Configuração Experimental] ~20s

"Para avaliar a solução, realizamos 282 execuções na AWS. Variamos o tamanho do cluster — de 2 a 8 máquinas spot — e simulamos interrupções em pontos distintos: aos 10%, 25% e 50% do tempo total, para ver como as estratégias se comportam dependendo de quando a falha ocorre. Testamos três programas científicos de referência: o EP-D, com pouca troca de dados entre computadores; o CG-C, com troca intensa; e o LU-C, com troca estruturada."

---

## [Slide 14 — Comparação das Estratégias] ~30s

"Analisamos quando cada estratégia é mais vantajosa. O gráfico cruza o tamanho do cluster com o momento da falha — onde verde indica que Degraded foi mais rápida, e vermelho que Replace foi mais vantajosa. O padrão entao é claro: clusters menores com falha precoce favorecem o Replace, pois ele retoma em capacidade total e ainda tem muito trabalho pela frente; E clusters maiores com falha tardia favorecem o Degraded, cuja recuperação imediata de cerca de 11 segundos compensa a redução de capacidade."

---

## [Slide 15 — Análise Econômica] ~35s

"Por fim, avaliamos se a solução compensa economicamente. O tempo de recuperação — 11 segundos no Degraded ou 120 no Replace — é fixo, independente de quanto tempo a aplicação já rodou. O desconto do spot, por outro lado, acumula com a duração. Para o EP-D, com execuções longas, ambas as estratégias compensaram. Para o LU-C, de duração média, apenas o Degraded foi viável — os 120 segundos do Replace representam overhead demais. Para o CG-C, curto, nenhuma compensou. A lógica é direta: quanto mais longa a execução, mais o custo fixo de recuperação se torna irrelevante frente ao desconto acumulado."

---

## [Slide 17 — Conclusão] ~25s

"Para concluir: o problema era claro — máquinas spot oferecem altos descontos, mas podem ser interrompidas a qualquer momento, causando a perda de horas de processamento em aplicações MPI. Nossa solução entao resolve isso: salva o progresso automaticamente e retoma a execução — via Replace ou Degraded — sem precisar modificar os programas. Alem de ser Economicamente viável para execuções longas, que são o perfil mais comum em computação de alto desempenho. Uma contribuição ao eixo Tecnologia e Inovação da UFSC, tornando computação de alto desempenho mais acessível em infraestrutura de baixo custo."

---

## [Slide 18 — Obrigado] ~5s

"Obrigado pela atenção — foi um prazer apresentar este trabalho!"

---

## Notas de Gravação

- Slide 2 (Sumário): **pular**, avançar direto para Slide 4
- Slide 6 (Checkpoint/Restart): **pular**, conteúdo integrado ao Slide 8
- Ritmo natural, com breve pausa entre slides
- Legendas em português recomendadas (requisito de acessibilidade do edital)
