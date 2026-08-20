# Resumo SIC 36º — PIBIC 2025/2026

**Título:** Resiliência Transparente para Aplicações HPC Legadas em Instâncias Spot da AWS com MANA
**Palavras-chave:** Computação em Nuvem. Instâncias Spot. Tolerância a Falhas. Checkpoint/Restart. Computação de Alto Desempenho.

---

## Resumo

Instâncias spot da AWS oferecem descontos de até 90% sobre o preço convencional, porém podem ser encerradas a qualquer momento com apenas dois minutos de aviso prévio. Esse risco é especialmente severo para aplicações MPI, que distribuem a computação entre múltiplos instâncias de forma fortemente acoplada: a falha de qualquer um deles encerra toda a execução e horas de processamento acumulado se perdem. Para lidar com esse problema, este trabalho integra o MANA, um sistema de checkpoint transparente para aplicações MPI, ao orquestrador HPC@Cloud do LaPeSD/UFSC. O MANA salva o estado completo de todos os processos em armazenamento persistente e, em caso de interrupção, retoma a execução do ponto salvo sem exigir qualquer modificação no código da aplicação. Ao detectar o aviso de encerramento, o sistema dispara o salvamento e retoma a execução por uma de duas estratégias: a Replace, que provisiona uma nova máquina para restaurar a capacidade original do cluster, e a Degraded, que reinicia imediatamente nos instâncias sobreviventes com paralelismo reduzido, priorizando velocidade de recuperação. Em 282 execuções de três aplicações científicas de referência em clusters de 2, 4 e 8 instâncias na AWS, verificou-se que o tamanho do cluster e o momento da falha determinam qual estratégia é preferível, e que o desconto das instâncias spot supera o custo de recuperação em aplicações de duração moderada a longa. O trabalho amplia o acesso a computação de alto desempenho em infraestrutura de nuvem de baixo custo, contribuindo para o eixo Tecnologia e Inovação da UFSC.

---

## Contagem

Caracteres com espaços: 1.574 / 3.000 (limite)
