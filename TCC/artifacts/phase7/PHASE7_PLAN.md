# Phase 7 Plan — UCC 2026 Paper
### "Transparent Resilience for Legacy HPC Applications on Cloud Spot Instances with MANA"

Date: 2026-07-02
Venue: 19th IEEE/ACM UCC 2026 — Florianópolis, Brazil, December 1–4, 2026
Format: IEEE conference double-column, max 10 pages (incl. figures, tables, references)
Overleaf: https://www.overleaf.com/3459393211rzvkwmfjgrjd
Authors: Artur Soda (UFSC), Vanderlei Munhoz (UFSC/Inria), Márcio Castro (UFSC)

---

## 1. Deadlines

| Milestone | Date |
|---|---|
| **Paper submission** | **August 19, 2026** |
| Acceptance notification | September 30, 2026 |
| Camera-ready | October 15, 2026 |
| Conference | December 1–4, 2026 |

7 weeks from today (2026-07-02) to submission.

---

## 2. Goal

Condense and sharpen the TCC content into a 10-page IEEE conference paper.
No new experiments are needed — all data comes from Phases 1–5 (282 executions).
The challenge is *selection and compression*, not generation.

---

## 3. Proposed Paper Structure

| # | Section | Est. columns | Notes |
|---|---|---|---|
| — | Abstract | 0.3 col | ✅ Done |
| 1 | Introduction | 1.0 col | ✅ Done |
| 2 | Background | 0.7 col | ✅ Done (spot+reactive, DMTCP, MANA topology, HPC@Cloud+Slurm) |
| 3 | Related Work | 0.5 col | ✅ Done (source-mod vs transparent axis; 3 paragraphs + gap) |
| 4 | Proposed Approach | 1.5 col | ✅ Done |
| 5 | Experimental Evaluation | 5.5 col | ✅ Done (6 subsections: Platform, Software, Overhead, Ckpt Latency, Recovery, Economic) |
| 6 | Conclusion | 0.6 col | ✅ Done (4 paragraphs) |
| — | References | ~1.0 col | ✅ Done |
| — | **Total** | **10 pages** | At page limit; no room for GenAI ack without trimming |

---

## 4. Writing Order (per professor's guidance)

1. **§1 Introduction** — ✅ Done
2. **§4 Proposed Approach** — ✅ Done
3. **§5 Experimental Evaluation** — ✅ Done
4. **§2 Background** — ✅ Done
5. **§3 Related Work** — ✅ Done
6. **§6 Conclusion** — ✅ Done
7. **Abstract** — ✅ Done

---

## 5. Content Mapping (TCC → Paper)

| Paper section | TCC source | Action |
|---|---|---|
| Abstract | Resumo | Compress to ~150 words in English |
| §1 Introduction | TCC §1 (intro + §1.3 contributions) | ✅ Done |
| §2 Background | TCC §2 (cloud, spot, MANA/DMTCP, HPC@Cloud, MPI) | ~1 paragraph each concept |
| §3 Related Work | TCC §3 (FT, C/R on cloud, cost studies) | 5–7 references, 3–4 sentences each |
| §4 Proposed Approach | TCC §5 (architecture, watcher, REPLACE, DEGRADED) | ✅ Done |
| §5 Evaluation setup | TCC §6.1–6.2 (cluster, benchmarks, metrics) | Compact table + short paragraph |
| §5 Results | TCC §6.3–6.7 (overhead, strategies, cost) | Keep key numbers; cut synthetic study detail |
| §6 Conclusion | TCC §7 | Compress; one sentence on threats to validity |
| **Cut entirely** | TCC §4 (burstable instances) | Separate publication (WCC); not cited in paper |

---

## 6. Figure Selection

6 figures in the final paper. All images are PDFs (converted from PNG for better print quality):

| # | Figure | File | Placement | Notes |
|---|---|---|---|---|
| 1 | Architecture overview | `imgs/arch-overview-UCCversion.pdf` | §4.1, single col `[t]` | ✅ Done; labels fixed ("On-Demand", "EC2 Spot Status API") |
| 2 | Strategy flow | `imgs/strategy-flow-UCCversion.pdf` | §4.4, single col `[t]` | ✅ Done |
| 3 | MANA overhead | `imgs/fig_overhead.pdf` | §5 Software, double col `figure*[t]` | ✅ Done; 0.8\textwidth |
| 4 | Checkpoint write latency | `imgs/fig_ckpt_latency.pdf` | §5.3, single col `[t]` | ✅ Done; new figure (gen_figs.py) |
| 5 | Strategy crossover heatmap | `imgs/fig_crossover.pdf` | §5.4, single col `[t]` | ✅ Done |
| 6 | Cost per run | `imgs/fig_cost.pdf` | §5.5 Economic, double col `figure*[t]` | ✅ Done |
| Cut | Watcher-flow diagram | — | — | Implementation detail; naming inconsistent |

---

## 7. What to Cut vs Keep

**Cut:**
- TCC §4 (burstable instances) — its own published story; cite ERAD/WCC papers in Related Work
- Synthetic studies detail (synth_calls, synth_p2p, synth_imbalanced) — keep only the *finding*: overhead is flat ~3 s fixed cost, independent of MPI call frequency
- Phase 2a/2c breakdown — paper focuses on P2b as the differentiating phase
- §6.8 Ameaças à Validade — compress to 1–2 sentences in Conclusion
- Strong scaling figure (fig6) — not the main story for this paper

**Keep:**
- 282-run experiment scope — signals rigor
- 3 benchmark × 3 cluster sizes × 2 strategies matrix
- The single crossover case (EP-D 2w at 10% failure timing) — shows intellectual honesty
- Cost savings 31–78% for medium/long jobs

---

## 8. Technical Setup Tasks

### 8.1 main.tex
- [x] Add `\thanks{}` with CNPq + AWS funding acknowledgment — done
- [x] Add packages: `booktabs`, `todonotes` — done
- [x] Section structure (6 sections: Intro, Background, Related Work, Proposed Approach, Evaluation, Conclusion)
- [x] `\todo` placeholders replaced with real content for §1 and §4
- [x] Fixed `\EC2` macro error — digits can't be in LaTeX command names; replaced with plain `EC2`
- [x] Fixed BibTeX error — `%% HPC@Cloud` comment had `@` which BibTeX interpreted as entry start; changed to `%% HPC at Cloud`
- [ ] Fix author email — Artur's UFSC institutional email still a placeholder
- [x] All §\todo placeholders replaced with real content (§2, §3, §5, §6 written)
- [ ] Write Abstract (~150 words)
- [ ] Add GenAI acknowledgment (required by UCC 2026)
- [ ] Fix arch-overview.png labels: "Burstable on-Demand" → "On-Demand", "AWS API Gateway" → "EC2 Spot Status API"

### 8.2 acronyms.tex
- [x] Kept Vanderlei's original 44 entries intact
- [x] Appended: AWS, EC2, EFS, NPB, FT, VM
- Note: `\EC2` macro is defined but unusable (digit in name); use plain `EC2` in text

### 8.3 references.bib
- [x] Populated with entries from TCC main.bib, all with DOIs
- [x] Removed Portuguese-language entries: `soda2025erad` (ERAD-RS), `camargo2017ftmpi` (WSCAD), `munhoz2022hpc` (SSCAD)
- [x] Fixed `@` in section comment causing BibTeX parse error
- Remaining: ~22 entries covering MANA/DMTCP, HPC@Cloud, NPB, spot instances, C/R systems, MPI FT, cost studies, WCC prior work

### 8.4 Figures
- [x] Created `UCC2026-MANA/imgs/` directory
- [x] Copied and upscaled 3x: `arch-overview.png`, `strategy-flow.png`
- [ ] Copy and resize evaluation figures when §5 is written

---

## 9. Suggested 7-Week Timeline

| Week | Dates | Goal |
|---|---|---|
| 1 | Jul 2–8 | Set up LaTeX skeleton (fix emails, acronyms, bib); write §1 Introduction draft |
| 2 | Jul 9–15 | Write §3 Proposed Approach; share with Vanderlei for feedback |
| 3 | Jul 16–22 | Write §4 Evaluation (setup + results); select and adapt figures |
| 4 | Jul 23–29 | Write §2 Background & Related Work; first full draft |
| 5 | Jul 30–Aug 5 | Internal review (Artur + Vanderlei + Márcio); revise |
| 6 | Aug 6–12 | Second round revisions; check page count; finalize figures |
| 7 | Aug 13–19 | Final polish, abstract, GenAI acknowledgment, submit |

---

## 10. Notes

- **GenAI acknowledgment:** UCC 2026 requires acknowledging GenAI tool use in the paper.
- **Language:** English throughout.
- **Overleaf:** primary collaborative editing environment (shared with Vanderlei and Márcio).
  Local copy in `TCC/artifacts/phase7/UCC2026-MANA/` kept in sync.
- **No new experiments** required.
- **At least one author must register and present** for IEEE Xplore inclusion.

---

## 11. Acceptance Gates

| Gate | Description | Status |
|---|---|---|
| G1 | LaTeX skeleton compiles cleanly (emails, acronyms, bib fixed) | ✅ Done |
| G2 | §1 Introduction drafted and shared with Vanderlei/Márcio | ✅ Done |
| G3 | §4 Proposed Approach drafted | ✅ Done |
| G4 | §5 Evaluation complete (6 subsections, 4 tables, 3 figures) | ✅ Done |
| G5 | Full paper at 10 pages (page limit) | ✅ Done |
| G6 | All references have DOIs | ✅ Done |
| G7 | GenAI acknowledgment included | Deferred — paper at page limit; to decide with advisor |
| G8 | Paper submitted by August 19, 2026 | Pending |

---

## 12. Writing Decisions (Introduction)

Decisions made during the writing of §1, to be kept consistent throughout the paper:

### Style
- No em dashes anywhere (avoid "AI-sounding" prose); use commas, parentheses, or rephrase
- Recovery strategies typeset as `\textsc{Replace}` and `\textsc{Degraded}` (small caps)
- MANA and DMTCP defined inline on first use with full name + acronym; not in GLS
- Checkpoint/restart written as "C/R" after first definition; not a GLS entry

### Structure chosen for Introduction
Seven-paragraph structure:
1. Motivation (HPC on cloud, spot instances)
2. Problem (C/R needed, but code modification limits adoption)
3. Existing solution (MANA/DMTCP transparent C/R)
4. **Gap** (transparent C/R alone is not enough; no existing orchestrator integrates end-to-end recovery)
5. Proposed solution (HPC@Cloud + MANA integration, REPLACE and DEGRADED)
6. Contributions (3 items)
7. Paper organization

The gap paragraph (4) was the most important addition after the first draft.
It shifts the framing from "we integrated two tools" to "we solve an open orchestration problem."

### Specific phrasings chosen
- "This paper presents" (not "proposes, implements, and evaluates")
- "typically do not integrate" (not "do not" — avoids universal claims a reviewer can challenge)
- "coordinates transparent checkpoint capture" (not "captures a checkpoint" — correctly
  attributes checkpoint mechanism to MANA, orchestration to our system)
- "To address the provisioning latency" (not "To handle")
- Contribution 1 is one sentence — no implementation detail list

### What was deliberately left out of the Introduction
- TCC Chapter 4 (burstable instances) — separate published work; not cited in this paper
- 282-execution count mentioned in Contribution 3 only, not forced earlier
- Synthetic studies (synth_calls, synth_p2p) — findings summarized in one sentence in Evaluation

---

## 14. Writing Decisions (Experimental Evaluation)

Decisions made during the writing of §5, to be kept consistent throughout the paper.

### Narrative structure chosen

Each subsection follows: **establish observation → explain mechanism → derive implication → transition**.
Not: show figure → explain figure → next figure.
This is the shift that makes §5 read like a systems paper rather than a thesis chapter.

### §5.1 Setup

- 282 executions = 180 NPB (3 runs per configuration × 3 benchmarks × 3 cluster sizes × 2 strategies + 3 timing variants for EP-D and LU-C) + 102 synthetic runs.
- Synthetic runs noted in Setup to avoid surprising the reader when referenced in §5.2; no detail beyond the count.
- Single-fault limitation stated explicitly ("each FT run simulates a single spot interruption; multi-fault behavior is not directly measured") so scope is clear before the analysis.
- CG-C excluded from timing variation: completes too quickly for a meaningful interrupt window; this is stated in the Setup rather than buried in §5.3.

### §5.2 MANA Overhead

**Structure:** Observation (EP-D/LU-C trend) → Exception (CG-C) → Rejected hypothesis → Leading hypothesis → Limitation statement → Forward bridge.

- CG-C anomaly framed as hypothesis only: synthetic microbenchmarks *rule out* call frequency; the polling-imbalance hypothesis is *proposed*, not confirmed; the paper explicitly says "not directly confirmed by internal instrumentation and is left as future work." Never claim more than the data shows.
- Bridge sentence at the end: "The following sections assess which configurations remain economically viable under these overhead levels." Prevents the section from ending on "future work."
- EP-D/LU-C forward pointer kept in first paragraph ("as the Economic Analysis below demonstrates") rather than moved to end, to maintain logical flow within the paragraph.

### §5.3 Recovery Strategy Comparison

**Core framing decision:** Factor-driven, not winner-driven. The section answers "what governs the choice?" not "which strategy wins?" The two factors are cluster size and failure timing.

- Table~\ref{tab:cg-breakdown} isolates cluster-size effect via CG-C (no timing variation = controlled experiment).
- Fig.~\ref{fig:crossover} shows both factors acting together via 18 EP-D + LU-C configurations (2 benchmarks × 3 cluster sizes × 3 timings).
- "17 of 18" framing was deliberately avoided: instead describes the one exception directly (EP-D, 2w, 10%) and explains why.
- Paragraph after table split into two: (A) figure introduction + single exception explanation; (B) pattern reading + multi-fault structural note.
- "Reading the columns left to right" sentence split into three: overall claim, N=2 behavior, N=8 behavior. One idea per sentence.
- Multi-fault paragraph: structural argument based on mechanism, not measured data. "These single-fault experiments do not capture multi-fault behavior." is the explicit scope marker. No fabricated numbers.

### §5.4 Economic Analysis

- Both strategies shown throughout (not just DEGRADED). fig_savings replaced with tab:cost, a 4-column table (Benchmark, N, Replace savings %, Degraded savings %) that makes both strategies directly comparable.
- Single merged paragraph: job-duration baseline → benchmark trend (Replace amortization) → EP-D cluster-size divergence → design insight closing.
- Long baseline sentence (originally 70 words, two semicolons) split into four sentences: one claim per sentence.
- Closing sentence is a **design insight**, not a table summary: "Overall, job duration determines whether spot execution is economically viable, while cluster size determines how far Replace's provisioning overhead erodes its savings relative to Degraded."
- Avoided: "DEGRADED is the more cost-efficient choice" and any other winner declarations. Show the governing tendencies; let the reader draw the comparison.

### Reviewer feedback rounds (for calibration)

Two rounds of external AI reviewer feedback were applied:

**Round 1 changes applied:** inverted §4.3 checkpoint timeout sentence (design decision first, empirical data second); "favorable" → "preferable" in §4.4 (two occurrences); bridge sentence added to §5.2; §5.3 dense paragraph split; "Reading the columns" mega-sentence split into three; §5.4 three-clause baseline sentence split; EP-D divergence sentence split; "Both tendencies are visible" closing replaced with design insight; "can still deliver" in §5.2.

**Round 2 verdict:** ~8.5–9/10, conference-quality evaluation. No further changes to §5 recommended.

**Suggestions explicitly NOT adopted:**
- "We evaluate three NPB kernels..." (kept "Three kernels exercise..." — tighter and more precise)
- Rephrase "The smaller the cluster and the earlier the failure..." (left as is — compound conditional mirrors the two-factor structure)
- Split "As N grows, losing one node..." into multiple sentences (left as is — still readable, splitting would feel choppy)

### What to write next (priority order)

Per reviewer feedback: further polishing §5 is diminishing returns. The missing sections matter more.

1. **§6 Conclusion** — second thing reviewers read; must close the loop on §1 contributions
2. **§2 Background** — spot instances/fault model, DMTCP/MANA mechanism, HPC@Cloud toolkit
3. **§3 Related Work** — cost-aware FT on spot, MPI FT approaches, system-level vs application-level C/R
4. **Abstract** — write last (~150 words)
5. **Figure quality** — captions, fonts, spacing, annotations
6. **Ensure contributions in §1 are explicitly answered by §5 results**

---

## 13. Writing Decisions (Proposed Approach)

Decisions made during the writing of §4, to be kept consistent throughout the paper:

### Mindset
The goal is to convince the reviewer that (1) the architecture is technically sound, (2) it solves the problem from the Introduction, (3) every design decision is justified, and (4) the evaluation will validate those choices. This is a systems contribution, not an algorithmic one — no pseudocode.

### Structure
Four subsections, each answering one question:
1. **Architecture Overview** — what exists and why (figure appears immediately after opening sentence)
2. **System Initialization** — how the cluster is prepared for fault-tolerant execution
3. **Failure Detection and Checkpointing** — how interruption is detected and state is saved (mechanism)
4. **Recovery Strategies** — REPLACE vs. DEGRADED policy, phase decomposition, tradeoff discussion

### Figures in this section
- `imgs/arch-overview.png` — architecture diagram; placed right after "Fig. 1 shows..."
- `imgs/strategy-flow.png` — REPLACE vs DEGRADED recovery paths
- Table~1 — phase decomposition (P0–P3 with P2a/P2b/P2c sub-phases)
- Watcher-flow diagram was **rejected**: contains implementation-level details (`dmtcp_command -c`) and uses inconsistent naming (REPLACE_RESUME/DEGRADED_RESUME vs. \textsc{Replace}/\textsc{Degraded})

### Key framing decisions
- "MANA is an existing tool; the orchestration pipeline around it is the contribution" — explicit separation of existing vs. novel components; keep this framing in all sections
- Architecture-level verbs throughout: coordinates, provisions, restores, monitors, resumes (not creates, calls, executes)
- "burstable" omitted from Proposed Approach — TCC Chapter 4 is a separate publication not covered in this paper; head node described only as "on-demand"; instance types appear in Experimental Setup table only
- No pseudocode — contribution is architectural; the strategy-flow figure already shows control flow

### Specific phrasings chosen
- "Our architecture is responsible for the remaining recovery workflow" (not "everything else")
- "P2b is reduced to a single Slurm operation" for DEGRADED (not "P2b is skipped" — marking the node inactive IS part of P2b)
- "a monitoring component, referred to as the *watcher*" — formal introduction before using the term
- Static IP justification: "reusing the same address on a replacement node means that the existing Slurm configuration and MANA restart require no reconfiguration after recovery"
- 300-second timeout justification: "Checkpoint writes completed in under 120 s in all our experiments, well within the two-minute termination window; a 300-second timeout serves as a configurable safety net for larger workloads"

### Explicit assumptions stated in text
1. AWS delivers the two-minute spot termination notice before reclaiming the instance
2. A single worker is interrupted at a time (parallel recovery is future work)
3. Head node and EFS remain available throughout execution

### Strategy choice factors (closing paragraph of §4.4)
Two factors govern the choice between REPLACE and DEGRADED:
1. **Cluster size** (dominant): losing 1/N workers causes smaller P3 penalty as N grows; DEGRADED increasingly favored with larger clusters
2. **Failure timing** (secondary): early failures leave more remaining work at reduced capacity, favoring REPLACE in small clusters
3. **Multiple failures**: REPLACE has structural advantage — restores capacity after each event; DEGRADED accumulates capacity deficit with each failure

The detailed quantitative analysis of these factors belongs in §5 Evaluation, not here.

---

## 15. Writing Decisions — §6 Conclusion

### Structure (3 paragraphs)
1. **What was built**: recap architecture — MANA + HPC@Cloud integration, two recovery strategies, automated detection-checkpoint-resume cycle. Opens with the main claim ("Legacy MPI applications can run on spot instances with transparent fault tolerance and without source-code modifications") as a direct statement, not a "this paper showed" opener.
2. **What was learned**: governed by two interacting factors (cluster size + failure timing). Includes economic summary — spot discount offsets MANA overhead for medium- and long-running jobs, not for short ones.
3. **Scope + future work**: single-interruption assumption, cluster size range (2–8 workers), one instance type and region. Natural next steps: larger clusters, repeated interruptions, other cloud providers.

### Reviewer feedback decisions (round 1 feedback, 9.5/10)
- **Applied**: Moved "no source-code modifications" into opening sentence (stronger claim upfront). Changed "governs" → "governed by two interacting factors" (richer framing). Added "progressively shifting the balance toward DEGRADED" for quantitative feel without invented numbers. Added "near-instant restart" qualifier to DEGRADED capacity trade-off sentence. Changed "findings suggest" → "These findings suggest" (specificity).
- **Not applied**: Reviewer suggested ending with a forward-looking sentence about broader cloud HPC adoption — rejected because it generalizes beyond what the paper measures and risks sounding like marketing.

### Phrasing choices
- "two interacting factors" — signals factorial interaction without claiming orthogonality
- "cloud billing is proportional to instance-hours consumed" (§5.4 cross-link) — grounds economic model in mechanism, not just empirical observation
- Assumptions paragraph closes the section honestly; avoids language like "limitations" which could weaken the contribution framing

---

## 16. Writing Decisions — §3 Related Work

### Structure (3 paragraphs + closing statement)
Para 1 — **Application-level fault tolerance**: ULFM (MPI standard extension), SCR (checkpoint library). Common thread: explicit programmer control; shared limitation: source modification required. Closes by connecting to Wang et al., which sidesteps instrumentation but breaks on IP change.

Para 2 — **System-level C/R**: BLCR (kernel module), DMTCP (user-space). Common thread: code transparency; shared limitation: IP-bound restart. Posner empirically confirms the overhead tradeoff. MANA resolves IP limitation.

Para 3 — **Cost + cloud strategies**: Gong et al. (checkpoint frequency + EC2 instance selection, uses BLCR → inherits IP dependency), Marathe (redundant execution across AZs — no checkpointing, 2× cost), FarSpot (proactive migration, avoids interruption entirely). Internal comparison: Gong/FarSpot address cost but not transparent post-interruption recovery; Marathe addresses resilience but at doubled resource cost.

**Closing**: "To our knowledge, existing approaches do not combine transparent, network-agnostic MPI checkpointing with an automated cloud recovery orchestrator, which is the gap this paper addresses."

### Key decision: internal comparison before gap statement
Initial draft compared each work directly to ours. Final structure first pits the works against each other (showing their trade-offs), then draws the collective gap. This is more intellectually honest and harder to falsify.

### Key decision: gap statement phrasing
"To our knowledge, existing approaches do not combine..." chosen over "None of these works integrates..." (too absolute, one counterexample falsifies it) and "This paper is the first to..." (priority claim, unverifiable). Current phrasing is descriptive of the observed state of cited literature, which is safe for peer review.

### Reviewer feedback decisions
- **Applied**: Removed "transparent" from BLCR sentence (BLCR does require no source change but the key point is IP-binding, not transparency). Reframed final two lines of Para 3 to internal comparison format rather than per-paper vs. ours. Added Posner citation context sentence.
- **Not applied**: Reviewer suggested adding a sentence noting MANA's NERSC production deployment in Related Work — rejected because it belongs in Background (where it already appears) and would duplicate content.

### Decisions NOT taken
- Static IP / MANA network-agnosticism tension: a reviewer suggestion to note "Slurm needs static IPs, not MANA" was considered and rejected. The explanation, while technically correct, immediately raises "why use Slurm instead of a hostfile?" — a question outside the paper's scope that could derail review. Current text says static IPs simplify cluster reconstruction without attributing it to either system.

---

## 17. Writing Decisions — §2 Background

### Structure (4 paragraphs)
1. **Spot instances**: discount magnitude (70–90%), two-minute termination notice mechanism, why the window is too short for reactive-only without prior checkpoints, IP change problem.
2. **DMTCP**: user-space interception, coordinated drain protocol, IP-bound restart limitation (sets up MANA).
3. **MANA**: split-process architecture (upper half saved, lower half rebuilt), virtual-to-physical translation tables, checkpoint consistency enforcement (quiescence at collectives), NERSC production deployment.
4. **HPC@Cloud**: cluster lifecycle management, declarative config, Slurm scheduler, spot workers, no original FT support, natural integration point.

### Source material
All content derived from TCC (`body.tex`): §Instâncias Spot (line 264), §DMTCP (line 540), §MANA (line 603), §HPC@Cloud (line 774). No new technical claims introduced.

### Reviewer feedback decisions (9.2–9.4/10)
- **Applied**: Added "An additional challenge is that a replacement instance typically receives a new IP address..." as explicit transition between spot and DMTCP paragraphs — makes the problem chain legible without spoiling MANA's solution. Changed "achieves the same transparency in user space" → "achieves the same transparency in user space and extends it to distributed processes without kernel support" (sharpens DMTCP's contribution). Added "demonstrating viability on a large-scale, commercially supported MPI stack" to NERSC sentence (more substantive than just citing the year). Added "making it a natural place to integrate automated recovery" to HPC@Cloud closing sentence (links Background to §4 without redundancy).
- **Not applied**: Reviewer suggested adding "MANA was specifically designed for HPC at scale" — not added because it's implied by NERSC deployment and would be an unsupported general claim. Reviewer suggested moving NERSC sentence earlier — kept at end of MANA paragraph because deployment evidence is a conclusion, not a setup fact.

### Key phrasing
- "a replacement instance typically receives a new IP address" — "typically" hedges correctly; some cloud configs preserve IPs but it is not the default
- "without knowledge of the library replacement" — captures the transparency guarantee precisely without over-claiming
- "making it a natural place to integrate automated recovery" — forward pointer to §4 that feels organic, not mechanical

---

## 18. Remaining Submission Tasks

| Task | Status | Notes |
|---|---|---|
| Write Abstract | ✅ Done | Single IEEE paragraph, ~170 words |
| Fix Artur's email in author block | ✅ Done | artur.soda@ufsc.br |
| Fix arch-overview.png labels | ✅ Done | "On-Demand", "EC2 Spot Status API" |
| Proofread full paper | ✅ Done | No em dashes; minor grammar fix (line 714) still pending |
| Add GenAI acknowledgment | Deferred | Paper at 10-page limit; adding it requires trimming elsewhere; to decide with advisor |
| Upload to submission system | **Pending** | Deadline: 2026-08-19 |

---

## 19. Writing Decisions — §5 Platform and Software Subsections

The Experimental Evaluation section was restructured from a single "Setup" subsection into
two dedicated subsections — **Platform** and **Software** — to give the experimental context
enough depth for a conference audience unfamiliar with the specific instance types and benchmarks.

### §5.1 Platform

- Two-item bullet list: t3.large (head, on-demand) and m5.xlarge (workers, spot or on-demand).
- Each item describes the role of the instance in the experiment, not just its specs.
- Pricing table (`tab:hardware`) placed here with 4-decimal-place spot/on-demand values
  (US-West-2, May 2026): m5.xlarge spot \$0.0585/h, on-demand \$0.1920/h.
- Repeated EFS sentence ("All shared state... persists across instance replacements") anchors
  the platform description to the recovery mechanism.

### §5.2 Software

- Stack listed inline: Amazon Linux 2023, MPICH 3.3.2, MANA v1.2.0, Slurm 24.05.4, NPB 3.4.4.
- `fig_overhead` (double-column `figure*[t]`) placed here so it appears at the top of the page
  that introduces the overhead analysis, keeping it visually near §5.3 MANA Overhead.
- NPB descriptions expanded into a three-item bullet list (EP, CG, LU) with kernel names,
  communication profiles, and class selections — gives a reviewer enough context to evaluate
  whether the benchmarks are representative.
- Four synthetic programs described in one paragraph with explicit role of each:
  two for MPI call frequency (collective + point-to-point), one for imbalance (sender delay),
  one for checkpoint size characterization (forward reference to §5.4).
- Commented-out `\todo` blocks left in source as development notes; they do not appear in the PDF.
- Interruption injection methodology explained: 10%/25%/50% of MANA-noFT time, with CG-C fixed at 3 s.
- Total run count stated explicitly: 180 NPB + 102 synthetic = 282 executions.

---

## 20. Writing Decisions — §5 Checkpoint Write Latency (new subsection)

Added based on Vanderlei's feedback after initial draft review.

### Rationale for the new subsection

The paper's abstract promises "practical viability thresholds." The existing content only covered
the economic threshold (job duration). The checkpoint latency study adds the second threshold:
per-process memory footprint. Without §5.4, the abstract claim was partially unmet.

### Content

- Introduces synth_checkpoint_size as the instrument.
- Figure (`fig_ckpt_latency`): linear x-axis (0–3 500 MB), orange dashed regression line
  (38.3 ms/MB), blue data points with error bars, red dotted 2-minute window at 120 s,
  point annotations for each footprint level. Generated by `gen_figs.py`.
- Key finding: write time scales linearly; slope 38.3 ms/MB; intercept ~16.7 s.
- At 3.2 GB/process: P1 = 139.2 s, exceeding the window (aggregate demand saturates
  EFS burst ceiling at ~105 MB/s).
- NPB footprints well below boundary; all checkpoint writes complete within the window.
- Closing paragraph (Option B framing): defines the ~3–4 GB/process applicability limit
  and defers higher-throughput storage / periodic checkpointing to future work.

### Framing decision

The subsection is positioned as an "applicability characterization," not an overhead study.
This is why synth_checkpoint_size gets its own results section while the other three
synthetic programs are summarized only in §5.3 Overhead. The other three answer "how much
does MANA add?" (same question as §5.3); synth_checkpoint_size answers "when does the
2-minute window become a hard limit?" — a different question.

---

## 21. Writing Decisions — §5 Recovery Strategy Comparison (update)

One sentence was added to the CG-C phase table analysis:

> "The increase in phase P1 at N=8 (from ~27 s to 37 s) is specific to CG-C, since each rank
> maintains per-partner communication buffers, so more workers enlarge the checkpoint image
> per process, whereas EP-D (collectives only) and LU-C (fixed-cardinality neighbors)
> keep it constant."

This explains an otherwise puzzling data point visible in `tab:cg-breakdown`.
The explanation grounds the observation in mechanism without requiring new experiments.

---

## 22. Writing Decisions — §5 Economic Analysis (update)

The Economic Analysis was extended with a new double-column figure (`fig_cost`) showing
cost per run across all benchmark × cluster-size × failure-timing configurations.
The table (`tab:cost`) is kept alongside it to give exact percentage savings.

- `fig_cost` uses `figure*[t]` (double-column) — same treatment as `fig_overhead`.
- The text was updated to reference `fig_cost` directly and uses it as the primary
  visual, with `tab:cost` as the precision supplement.
- "neither strategy achieve cost savings" — grammar error (should be "achieves") noted
  but not yet fixed; to correct before final camera-ready.

---

## 23. Writing Decisions — §6 Conclusion (update)

Conclusion expanded from 3 to 4 paragraphs.

**Paragraph 1** (unchanged): what was built — architecture recap, main claim.
**Paragraph 2** (unchanged): two recovery strategies and their design tradeoffs.
**Paragraph 3** (revised): experimental findings + economic + checkpoint latency thresholds.
  Added explicit mention of the checkpoint latency study and the storage scalability limits
  to fulfill the "practical viability thresholds" promise from the abstract.
**Paragraph 4** (new): future work — multiple interruptions, periodic checkpointing,
  larger clusters/providers, adaptive recovery policies.

The future work paragraph was added to give the paper a proper closing that signals
the work is a foundation rather than a final answer, which reviewers expect.
