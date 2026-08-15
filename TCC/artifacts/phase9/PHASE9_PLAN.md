# Phase 9 Plan — SIC UFSC 36º Seminário de Iniciação Científica

Date: 2026-08-15
Projeto: Sustainable High Performance Computing on AWS
Bolsista: Artur Luiz Rizzato Toru Soda
Orientador: Prof. Dr. Márcio Bastos Castro
Instituição: LaPeSD, INE/UFSC

---

## Status

| Etapa | Atividade | Status |
|---|---|---|
| 0 | Aguardar edital do 36º SIC e confirmar datas | ☐ |
| 1 | Escrever o resumo (máx. 3.000 caracteres) | ☐ |
| 2 | Planejar roteiro do vídeo | ☐ |
| 3 | Gravar e editar o vídeo (MP4, 2–5 min) | ☐ |
| 4 | Submeter vídeo no Repositório Institucional | ☐ |
| 5 | Fazer inscrição no formulário do SIC | ☐ |
| 6 | [Condicional] Preparar apresentação oral (PDF/PPTX) | ☐ |

---

## 1. Contexto

O 35º SIC (ciclo 2024/2025) ocorreu em outubro de 2025. O **36º SIC (ciclo
2025/2026)** deve seguir o mesmo calendário, com edital publicado por volta de
agosto/setembro de 2026. O edital do 35º SIC é a referência de estrutura e
critérios; as datas exatas devem ser confirmadas no edital do 36º.

Fonte de conteúdo: o Relatório Final PIBIC 2025/2026 já está completo
(`phase8/relatorio-pibic-2025-2026/main.tex`). Todo o conteúdo do resumo e do
vídeo é derivado dele.

---

## 2. Artefatos a Produzir

| Artefato | Formato | Requisito |
|---|---|---|
| Resumo | Texto puro | Máx. 3.000 caracteres; mesmo texto no formulário e no Repositório |
| Vídeo | MP4 | 2–5 min; linguagem acessível ao público geral; legendas recomendadas |

---

## 3. Detalhes de Cada Etapa

### Etapa 0 — Aguardar edital do 36º SIC
- O edital de 2025 foi publicado em 20/08/2025.
- Esperar publicação do 36º SIC (~ago/2026) para confirmar prazos exatos.
- **Prazo crítico a verificar:** data-limite de submissão do vídeo no
  Repositório (no 35º foi 08/09/2025, ~1 semana antes do prazo de inscrição).

### Etapa 1 — Resumo
- Máximo de 3.000 caracteres (com espaços).
- Deve cobrir: contexto/motivação, objetivos, metodologia resumida,
  principais resultados, conclusão.
- Palavras-chave: máximo de 5 (ex.: Computação em Nuvem, Instâncias Spot,
  Tolerância a Falhas, Checkpoint/Restart, HPC).
- Linguagem: mais acessível que o abstract do relatório, mas ainda precisa.
- Pode ser gerado diretamente do abstract do relatório com adaptações.

### Etapa 2 — Roteiro do Vídeo
Estrutura sugerida para 3–4 minutos:

| Segmento | Conteúdo | Tempo sugerido |
|---|---|---|
| Abertura | Problema: spot instances, risco de perda de progresso em jobs HPC | ~30 s |
| Solução | MANA + HPC@Cloud: checkpoint transparente, sem modificar aplicações | ~45 s |
| Estratégias | Replace vs Degraded: trade-off velocidade × capacidade | ~45 s |
| Resultados | Overhead, latência de checkpoint, comparação de estratégias, análise econômica | ~60 s |
| Conexão UFSC | Eixo Tecnologia e Inovação: HPC acessível em nuvem de baixo custo | ~20 s |
| Conclusão | Viabilidade demonstrada; trabalhos futuros | ~20 s |

- Linguagem: evitar jargões; explicar "checkpoint" como "salvar o estado da
  aplicação"; "spot" como "instâncias de computação mais baratas, mas que
  podem ser desligadas a qualquer momento".
- Recomendado: legendas em português (acessibilidade auditiva).

### Etapa 3 — Gravação e Edição
- Formato de saída: MP4.
- Pode ser screencast com narração sobre slides, vídeo falado, ou combinação.
- Duração: entre 2 e 5 minutos (o regulamento não aceita fora desse intervalo).

### Etapa 4 — Submissão no Repositório Institucional
Campos a preencher:
- **Autor:** Artur Luiz Rizzato Toru Soda
- **Orientador:** Márcio Bastos Castro
- **Título:** pode ser diferente do projeto, mas deve referenciar o título
  original no resumo
- **Local:** Florianópolis
- **Tipo:** Vídeo
- **Idioma:** Português (Brasil)
- **Palavras-chave:** máx. 5
- **Resumo:** mesmo texto da Etapa 1
- **Extensão do Item:** Resumo + Vídeo
- **Descrever:** Seminário de IC / UFSC / CTC / INE
- **Arquivo:** vídeo MP4
- **Programa/Área/Departamento:** PIBIC > Ciências Exatas, da Terra e
  Engenharias > Departamento de Informática e Estatística

### Etapa 5 — Inscrição no Formulário
- URL: https://pibic.sistemas.ufsc.br/sic/login
- Inserir: resumo (mesmo da Etapa 1) + link do vídeo no Repositório.
- Guardar recibo eletrônico de protocolo enviado por e-mail.

### Etapa 6 — Apresentação Oral (condicional)
- Só se o trabalho for selecionado (divulgação ~out/2026).
- Formato: PDF ou PPTX.
- Local: presencial em Florianópolis (SEPEX).
- Duração: a definir no edital.

---

## 4. Critérios de Avaliação do Vídeo

O vídeo é avaliado por pesquisadores da UFSC com os seguintes critérios:

| Critério | O que significa na prática |
|---|---|
| Conexão com eixo UFSC | Deixar explícito "Tecnologia e Inovação" |
| Objetividade | Cobrir os pontos principais sem enrolação |
| Comunicabilidade | Leigo deve entender o problema e a solução |
| Conteúdo | Objetivos, resultados e conclusão presentes |
| Atratividade | Apresentação que desperta interesse |
| Adequação ao tempo | Usar bem os 3–4 minutos disponíveis |
| Relevância científica | Contribuição clara para HPC em nuvem |

---

## 5. Calendário Estimado (baseado no 35º SIC, deslocado ~1 ano)

| Etapa | Prazo estimado |
|---|---|
| Publicação do edital do 36º SIC | ~20/08/2026 |
| Prazo para submissão do vídeo no Repositório | ~08/09/2026 |
| Prazo para inscrição no formulário | ~15/09/2026 |
| Divulgação dos selecionados para oral | ~08/10/2026 |
| Apresentações orais (SEPEX) | ~20–24/10/2026 |
| Divulgação dos premiados | ~30/10/2026 |

---

## 6. Gates de Aceitação

| Gate | Descrição |
|---|---|
| G1 | Resumo dentro de 3.000 caracteres |
| G2 | Vídeo em MP4, entre 2 e 5 minutos |
| G3 | Vídeo submetido no Repositório antes do prazo da Etapa 4 |
| G4 | Inscrição no formulário confirmada com recibo eletrônico |
