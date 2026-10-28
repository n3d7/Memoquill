# Learning mechanisms and evidence-informed product decisions

Status: Foundation research and design input; no application implementation or product-effectiveness study.
Additional question-format, generative-practice, instructional-sequencing and AI evidence is in [learning-activities.md](learning-activities.md); [the supplied Consensus report audit](consensus-review-audit.md) records source/status corrections. The original S1–S22 register remains the mechanism baseline.
Evidence checked: 2026-10-27.
Scope: an offline, subject-agnostic learning application; English, Python, mathematics, quantum physics, and cooking are contrasting examples rather than domain types.

## Interpretation and method

The strongest starting point is to support spaced opportunities to retrieve and use knowledge, with informative feedback and appropriate instruction.
The evidence does **not** justify making every subject a flashcard deck, treating every mistake as desirable, or equating a review score with understanding or practical competence.
Retention, explanation, problem solving, transfer, and practical performance need different tasks and evidence.

This is a targeted evidence review, not a preregistered systematic review or an exhaustive literature search.
Consensus was used by the coordinating research task for discovery; bibliographic details and the material findings below were checked independently against publisher pages, primary article text/abstracts, or author/institutional copies.
Public search and retrieval used the existing self-hosted Firecrawl tools and web retrieval; no learner data were supplied.
Recent 2024–2026 studies are included where they sharpen a decision, alongside foundational syntheses.
Publication year is not a quality score.
The document reports published estimates, never an expected percentage improvement for Memoquill.

Evidence labels used below:

- **Supported:** converging evidence supports the mechanism for the specified outcome; effectiveness in this product remains untested.
- **Context-dependent:** credible evidence exists, with material differences by learner, task, comparator, setting, or outcome.
- **Hypothesis:** a plausible application of research that needs direct evaluation.
- **UX/engineering:** a product choice justified by usability, accessibility, reliability, or offline operation; no independent learning-effect claim.

Meta-analytic averages summarize different interventions and control conditions.
They do not rank features on a common scale, identify an optimal schedule for every individual, or establish that a bundle of features has additive effects.
Laboratory experiments can isolate mechanisms; classroom studies improve ecological relevance but introduce attendance, homework, curriculum, feedback, and implementation differences.
Neither automatically represents an adult studying independently with imported material.
Many studies use short delays, narrow tasks, and samples from relatively advantaged educational settings; these limitations matter for an unrestricted learning app.

## Evidence register

Expected benefits in this register are design expectations to test, not demonstrated Memoquill outcomes.
Validation methods are proposed future checks; none was conducted in this Foundation task.
`g` and `d` are standardized mean differences; they are not percentage-point gains.

| Finding / status | Evidence quality, design, sample, verified effect | Applicability and limitations | Product decision | Expected benefit | Validation method |
| --- | --- | --- | --- | --- | --- |
| Spacing repeated encounters supports retention — **Supported** | Cepeda et al. [S1]: quantitative synthesis, 317 experiments / 839 assessments of verbal recall. Spacing and final-test delay interact. | Strong memory evidence, with much laboratory verbal learning; it does not establish a universal expanding interval sequence. | Make scheduled return to previously learned material a core capability; allow an intended retention horizon. | More durable access to learned knowledge than a single concentrated exposure. | Delayed unaided recall at prespecified intervals, with practice time and exposure recorded. |
| Classroom spacing is promising, but effects are heterogeneous — **Context-dependent** | Mawson & Kang [S2], 2025: 22 reports, 31 effects, N > 3,000; d = 0.54, 95% CI [0.31, 0.77]. | Curriculum-relevant classroom review; small corpus, heterogeneous practice, some spacing/interleaving confounding, limited moderator power. | Treat spacing as a defensible default, retain provenance and schedule settings, and avoid guaranteed benefit copy. | Feasible repeated engagement across sessions. | Compare delayed outcomes and workload against a credible time-matched study alternative. |
| Retrieval practice can improve educational learning — **Supported** | Yang et al. [S3]: classroom testing meta-analysis, 222 studies, N = 48,478; overall g = 0.499. Agarwal et al. [S4]: 50 classroom experiments, N = 5,374. | Benefits vary with controls and implementation; only 6% of experiments in S4 were from non-WEIRD countries. A quiz is not an entire curriculum. | Offer recall before answer reveal, plus a route back to explanation or source material. | Strengthen retrieval and expose gaps hidden by rereading. | Delayed tests with alternate items; distinguish recognition from generated answers and record assistance. |
| Feedback is a family of interventions — **Context-dependent** | Wisniewski et al. [S5]: 435 studies, 994 effects, N > 61,000; substantial heterogeneity and differences by information content. | Correctness signals, explanations, rewards, and task/process feedback are not interchangeable. Timing is task-dependent. | Show the expected answer or rubric, relevant explanation, and source; permit correction and another attempt. | Identify what failed and support an actionable next step. | Error correction and later performance on related tasks; assess whether feedback is understood and used. |
| Spaced retrieval helps retention; expanding gaps are not uniformly superior — **Supported / context-dependent** | Latimier et al. [S6]: 29 studies; spaced vs massed retrieval g = 0.74; expanding vs uniform g = 0.034, not significant. | Different schedules and exposure counts; this does not prove all schedules equivalent or prescribe any particular algorithm. | Keep the scheduler replaceable; support a transparent simple baseline before claiming advanced adaptation. | Repeated retrieval without unnecessary schedule complexity. | Retention per unit practice time; evaluate schedule calibration separately from learning outcomes. |
| Mathematics is a caution against domain-wide extrapolation — **Context-dependent** | Murray et al. [S7], 2025: spacing, 27 studies / 53 effects, g = 0.28; retrieval vs restudy, seven studies / 32 effects, g = 0.18 with CI crossing zero. | Smaller math effects; retrieval evidence is limited and inconclusive, not proof of no benefit. | Schedule actual mathematical problem attempts, not only formula recall; retain worked examples and strategy instruction. | Practice relevant operations and choose methods in context. | Delayed problems with changed values and representations; score reasoning and solution quality. |
| Applied STEM results vary across courses — **Context-dependent** | Bego et al. [S8], 2024: nine introductory courses with spaced/massed quiz practice; significant positive spacing results in two courses. Aggregate significance disappeared when calculus was removed. | Course and implementation differences; outside study and feedback use can affect the result. | Do not advertise uniform STEM effectiveness; report outcomes by task and subject rather than only an overall average. | Detect where a method is useful or burdensome. | Course/task-level outcomes, uncertainty, participation, practice exposure, and feedback engagement. |
| Interleaving can help discrimination, but blocking can win — **Context-dependent** | Brunmair & Richter [S9]: 59 studies / 238 effects; overall g = 0.42; math g = 0.34; word tasks favored blocking, g = -0.39; text/taste results ambiguous. | Most relevant to learning categories and selecting between confusable alternatives; not evidence for random topic switching. | Let authors/learners define contrast sets; start with focused instruction when needed, then mix related problem/category types. | Better discrimination and strategy selection. | New exemplars or unlabeled mixed problems; compare with blocked practice while matching exposure. |
| Worked examples can support mathematical learning — **Supported / context-dependent** | Barbieri et al. [S10], 2023: 55 experimental/quasi-experimental studies / 181 effects, g = 0.48. Correct examples outperformed some alternative example types. | Effects depend on comparison and design; adding self-explanation prompts was not an automatic improvement in this synthesis. | Support complete solutions, completion tasks, hints, and independent attempts; allow support to fade. | Reduce unproductive search while a learner builds a usable procedure. | Independent solution after support removal, delayed performance, and assistance needed. |
| Cognitive load and prior knowledge constrain useful difficulty — **Context-dependent** | Sweller et al. [S11]: theoretical and empirical review of cognitive load research; worked-example, split-attention, and expertise-reversal findings. | A theory-informed design account, not a diagnostic test or a universal numeric working-memory limit. | Keep task-relevant explanation near the task; offer stepwise support and a way to skip familiar guidance. | Reduce avoidable coordination demands without removing useful challenge. | Task completion, errors, support use, and subjective effort, alongside delayed learning. |
| Meaningful multimedia segmentation can help — **Context-dependent** | Rey et al. [S12]: meta-analysis of 56 investigations / 88 comparisons; small-to-medium retention/transfer benefits, lower load, and longer learning time. | Pacing and extra time can contribute; arbitrary sentence-sized fragments are not the tested intervention. | Preserve coherent steps; provide pause, replay, navigation, and accessible text alternatives. | Make transient explanations inspectable at an appropriate pace. | Comprehension/transfer plus study time and navigation usability, not completion rate alone. |
| Metacognitive judgments can be misled by fluency — **Supported** | Bjork et al. [S13]: review of monitoring/control, perceived fluency, learning beliefs, and self-regulated study. | Confidence and familiarity are imperfect measures; low confidence alone is not a reason to prescribe extra practice. | Collect optional confidence before reveal; compare it with later performance; keep self-report separate from scored evidence. | Help learners notice overconfidence and uncertainty. | Confidence–accuracy calibration and whether subsequent study choices become more effective. |
| Self-regulation benefits require more than a dashboard — **Context-dependent** | Theobald [S14]: 49 studies / 55 samples, N = 5,786; extended university SRL training, academic-performance g = 0.37. | Multi-component training with instruction/support; effects cannot be assigned to a goal widget or progress chart. | Offer a small plan–attempt–review loop with editable goals and specific strategy guidance. | Make deliberate study choices easier. | Goal use, strategy adoption, workload, and delayed outcomes; no inference from screen visits alone. |
| Transfer from retrieval is possible but conditional — **Context-dependent** | Pan & Rickard [S15]: 122 experiments, N = 10,382; transfer d = 0.40, CI [0.31, 0.50], moderated by response congruency, elaboration, and initial success. | Transfer varies greatly by task; the pooled result is not proof of general reasoning skill or far transfer. | Include changed-context, explanation, strategy-selection, and application tasks as separate objectives. | Use remembered knowledge when its relevance is less obvious. | Novel tasks with explicit distance from practice and separate scoring of recall/application. |
| Complex-concept results depend on practice and comparator — **Context-dependent** | Corral & Carpenter [S16], 2025: three experiments, N = 309, 158, 255. One retrieval round did not improve application; repeated retrieval with feedback and a one-week delay gave stronger evidence. | Research-method concepts, mostly multiple-choice final outcomes; quiz-study is a stronger comparator than rereading. One-week transfer is not years-long competence. | Avoid a single-question “mastered” flag; use multiple exemplars and later application checks. | More opportunities to correct confusions and apply a concept. | Delayed new scenarios compared with time-matched question-and-answer study; track attrition and assistance. |
| Formative assessment can inform the next learning action — **Context-dependent** | Yao et al. [S17], 2024: 118 K–12 studies / 258 effects, overall g = 0.25; larger effects in quasi-experimental designs. | Broad definitions and classroom interventions; an app score alone is not formative unless someone acts on it. | Connect an observed gap to feedback, instruction, practice, or a goal revision. | Turn assessment information into useful action. | Document resulting actions and later task performance; distinguish diagnostic accuracy from intervention effects. |
| Personalized review is promising, not a universal optimizer — **Context-dependent / hypothesis for Memoquill** | Lindsey et al. [S18]: semester-long middle-school Spanish study, time-matched personalized, fixed-spaced, and massed review comparisons. | A specific population, curriculum, and memory model; it does not validate another scheduler, subject, or target recall probability. | Use local item/learner histories and explain recommendations; retain manual control and a fixed baseline. | Allocate limited review time more appropriately. | Held-out delayed recall calibration, workload, robustness to missed sessions, then outcome comparison. |
| Deliberate practice is useful but does not fully explain expertise — **Context-dependent** | Macnamara et al. [S19]: cross-domain meta-analysis; associations differed by domain and left substantial variance unexplained. | Largely association/retrospective practice measurement; definitions are disputed and correlation is not a causal dose response. | Let learners specify a subskill, attempt a meaningful task, receive feedback, and retry; do not sell an hours-to-mastery formula. | More purposeful practice than undirected repetition. | Domain-relevant task/rubric evidence; practical evaluation when the target is physical performance. |
| Simulation evidence does not make screen performance practical competence — **Context-dependent** | Chernikova et al. [S20]: 145 studies, N = 10,532; mainly medical education, mixed designs/control types, high heterogeneity. | Evidence supports structured complex-skill practice in studied settings, not a general simulation or cooking effect. | Support scenario tasks and evidence of off-screen performance; do not require a simulation engine in the core. | Bridge conceptual knowledge and situated decisions. | Authentic performance assessment outside the practiced representation; analyze transfer explicitly. |
| Recent classroom scheduling evidence can differ by task complexity — **Context-dependent** | DeFouw et al. [S21], 2026: randomized study, 13 classrooms, equal daily practice dose, four-week follow-up. Distributed sessions favored simple computation; complex computation showed a different pattern. | One classwide intervention and restricted tasks; not a general law that complex skills require massing. | Permit focused blocks and spaced revisits; do not force every task into the same session length or gap. | Match practice format to task demands. | Delayed simple/complex outcomes separately, with dosage and teacher/learner acceptability. |

## Mechanisms translated into a learning loop

The following loop is a design inference from the evidence, not a validated instructional package.
It can run entirely offline using authored content, preserved sources, learner input, and transparent rules.
AI can suggest drafts or alternatives later; none of the mechanisms requires AI generation or automated grading.

1. **Define the intended performance.** State what the learner should recall, explain, choose, solve, produce, or do, and in what conditions.
2. **Establish prerequisites and initial instruction.** Provide a source, explanation, demonstration, or worked example when the learner cannot yet make a meaningful attempt.
3. **Attempt before reveal when appropriate.** Give enough time and a response format aligned with the objective; permit “I do not know” and explicit hints.
4. **Use feedback.** Compare against a trustworthy answer, explanation, rubric, or observed result; identify an actionable correction.
5. **Revisit after a gap.** Return to important material with an opportunity to retrieve or perform; adjust workload without presenting uncertainty as a precise diagnosis.
6. **Vary and apply.** Once basic performance is possible, contrast confusable cases and introduce changed contexts or constraints.
7. **Check later and separately.** Evaluate unaided retention, application, and practical performance with appropriate tasks; revise the plan using the resulting evidence.

Useful difficulty must still leave a route to learning.
Repeated failure without informative correction can be an instruction, prerequisite, or task-design problem.
Supported retrieval and novice guidance can coexist; it is unnecessary to choose one universal mode for an entire learner.

### Spacing, retention, and adaptation

Spacing is the distribution of repeated encounters with the same target over time.
Interleaving changes the order of different targets/categories; it can also introduce spacing, but the two mechanisms are not identical.
A scheduler therefore needs to distinguish “when revisit this target?” from “which contrasting target comes next?”

Product recommendations are hypotheses unless an evidence label above says otherwise:

- Record the actual attempt time, task version, objective, result, assistance, and next recommendation locally.
- Associate review evidence with the task and target objective; a successful definition should not silently upgrade an independent practical skill.
- Begin with a documented scheduling baseline and explicit parameters. A learned model is optional and must outperform a relevant baseline on the intended outcome before superiority is claimed.
- Explain due dates as recommendations. Expose rescheduling, postponement, suspension, and manual review without a punitive reset of history.
- Distinguish a missed session from an observed failure. Use the real elapsed interval; avoid inventing attempts or treating non-use as loss of ability.
- Keep acquisition, maintenance, and recovery after a long break distinguishable. A learner may need instruction again rather than a stream of failed reviews.
- A selected “target recall probability” is a workload policy, not a proven optimum or a direct measurement of understanding.
- Spacing evidence does not imply that every interval must expand. The schedule should consider intended use, current evidence, and available time; numeric defaults require later validation.

These choices support user control and recoverability as UX/engineering goals.
Their learning effects are untested.
An algorithm that predicts the response to an item accurately may still choose inefficient practice or neglect important objectives.
Prediction, scheduling policy, engagement, and learning outcomes must be evaluated separately.

### Instruction, examples, and multimedia

Worked examples should expose how a result is obtained and when the method applies.
Completion tasks and hints can provide intermediate support before an independent attempt.
The evidence for examples is strongest within the tested tasks and learners; it does not authorize automatic help removal after a fixed number of correct answers.

Do not implement “cognitive load” as a single ability score.
Subjective difficulty, response time, and errors are useful observations but have multiple causes.
Task design should make necessary relationships inspectable and avoid gratuitous coordination; meaningful complexity still has to be learned.

Multimedia is useful when it represents something the objective requires: a pronunciation, changing graph, code trace, movement, or procedure.
More media are not intrinsically better.
Segmentation should follow coherent instructional steps, with learner control and a visible overview; this differs from requiring every unit to fit an arbitrary duration.
Captions, transcripts, adjustable pacing, readable equations/code, and text alternatives are accessibility/engineering requirements here, not a claim that one modality maximizes learning for everyone.

### Metacognition and formative assessment

Confidence, effort, enjoyment, time spent, and successful performance are different observations.
Optional confidence-before-feedback can be compared with later results, but should not replace evidence of the target performance.
Brief guidance explaining why retrieval or delayed checks may feel harder is a plausible support, not a tested Memoquill intervention.

Assessment becomes formative when it informs a subsequent action.
A useful response to a gap could be “review this worked example,” “compare these two cases,” “repeat the procedure with feedback,” or “revise this objective.”
It need not always be “review sooner.”
Rich self-regulation programs involve more than prompts; tiny interface elements should not inherit the effects of the whole program.

### Transfer and practical skill

Recall supports later use, but recognizing relevance, selecting a method, executing it, and monitoring the result add demands.
Changed numbers or reworded questions usually test a narrower generalization than a new problem setting.
Transfer claims should name exactly what changed: representation, context, cue, response format, constraints, or domain.

Deliberate practice suggests goal-directed attempts and informative feedback, not a universal repetition quota.
For practical objectives, self-reported attempts, photos, recordings, rubrics, or observer feedback can be useful evidence with explicit provenance and uncertainty.
None should be silently equated with objective certification.
Physical practice, tools, materials, social interaction, and sensory feedback may lie outside the app.
Memoquill should help plan and document those activities without claiming that studying the plan is the activity itself.

## Subject contrasts and objective/task alignment

This matrix is a product-design inference; it is not evidence that the particular task examples improve learning.
Every subject can contain several kinds of objective, and the same generic task capabilities should serve them.

| Example subject | Recall / conceptual objective | Problem solving / practical objective | Appropriate evidence and boundary |
| --- | --- | --- | --- |
| English as a second language | Recall a form–meaning relationship; explain a tense contrast; recognize a sound distinction. | Produce an utterance in a new situation; understand unfamiliar speech; sustain a conversation. | Separate receptive vocabulary, productive recall, listening, pronunciation, and interaction. A translated word pair does not establish speaking fluency. |
| Python | Recall an API constraint; explain mutation, scope, or an error; predict a short trace. | Debug unfamiliar code; design and implement a solution; explain tradeoffs. | Separate answer correctness, reasoning, code behavior, and help used. Imported or generated code remains untrusted; this research does not authorize automatic execution. |
| Mathematics | Recall a definition; explain why an operation is valid; identify conditions of a theorem. | Select a method without a topic label; solve a novel problem; construct or critique a proof. | Use mixed problems, altered representations, reasoning rubrics, and delayed attempts. Formula recall and symbolic manipulation are distinct. |
| Quantum physics | Explain assumptions and representations; connect operators, states, and measurement concepts. | Set up and solve a new model/problem, interpret results, and identify approximations. | Check mathematical prerequisites and conceptual reasoning separately. Do not claim transfer from memorized equations or conceptual multiple-choice alone. |
| Cooking | Recall a process, ingredient role, or sequence; explain a transformation. | Execute a procedure, coordinate timing, and judge texture or other observable results. | Plan and record real practice; use process/result rubrics and feedback. Recipe recall, video recognition, taste discrimination, and safe independent performance are different outcomes. |

### Second-language-specific qualification

Kim & Webb [S22] synthesized 48 experiments with N = 3,411 and found an overall spacing advantage.
Longer gaps were more advantageous on delayed than immediate tests, while equal and expanding schedules did not differ significantly overall.
Learning target, practice type, activity, feedback timing, and retention delay moderated results.
This supports spaced language practice as an option, not one schedule for vocabulary, grammar, pronunciation, comprehension, and conversation.

Treat a direction of retrieval as part of a task: recognizing a written word and producing it from a meaning cue are different demands.
Include source language, target language, script, pronunciation material, and accepted variants in content metadata where appropriate; none should become a privileged core subject type.
Production and interaction objectives need production and interaction evidence.
Automated pronunciation scores or AI conversation ratings would require separate validation and optionality; their effectiveness is not established by the spacing literature.

## Evaluation contract for later authorized work

No learning-outcome experiment, learner recruitment, or application implementation was performed here.
The following separates future kinds of validation so engineering success cannot be mistaken for educational effectiveness.

| Question | Suitable future evidence | Insufficient substitute |
| --- | --- | --- |
| Does the app behave correctly offline? | Functional, accessibility, import/export, scheduling, recovery, and data-integrity checks. | A paper about spacing or a working online demo. |
| Can people use the intended learning loop? | Usability tasks with representative goals, comprehension of feedback/due dates, and observed friction. | Session count, streak length, or positive reactions alone. |
| Is a response estimate calibrated? | Prediction on later held-out attempts, calibration/error analysis, and missing-data handling. | In-sample fit, high average accuracy, or an opaque “mastery” number. |
| Does a schedule improve retention efficiently? | Prespecified delayed unaided outcomes, a credible time/exposure-matched comparator, practice burden, and uncertainty. | Immediate review accuracy or a scheduler's own estimate. |
| Does learning transfer? | Novel items/problems with a documented change from training, matched prior knowledge, and independent scoring where feasible. | Repeating practiced questions or showing better recall alone. |
| Does practical performance improve? | A relevant authentic task or independently justified rubric/assessment beyond the practiced screen activity. | Recipe cards reviewed, code syntax quizzes passed, or videos watched. |

Before any study, define the intended population, objective, comparison, retention interval, primary outcome, assistance policy, missingness/attrition handling, and analysis plan.
Report benefits, null results, harms/burden, and uncertainty by meaningful task groups.
Retest delays should match the claim; a one-week result cannot establish annual retention.
Time on task and exposure are outcomes and possible confounds, not interchangeable controls.
Do not claim educational superiority from observational correlations between app use and achievement.

## Decisions and unresolved questions

**Proceed as design inputs:** offline retrieval with answer review; scheduled revisits; task/assistance provenance; source-linked explanations; multiple objective/task types; worked examples and independent attempts; optional contrast sets; separate retention and application evidence.

**Keep configurable and evaluate:** default interval policy, session workload, initial success criterion, fading guidance, interleaving conditions, feedback timing, confidence prompts, and adaptation.
The evidence does not select a particular memory model or establish that a complex scheduler is needed in the first version.

**Open:** which task formats learners can author reliably; how to validate rich answers without mandatory AI; how to represent partial success without false precision; how much feedback people actually use; which transfer/practical assessments are feasible; and which benefits persist under long gaps, missed sessions, and diverse accessibility needs.
These are product/research questions, not reasons to delay a transparent offline baseline indefinitely.

## Verified source register

Primary publisher pages, primary article records, and author/institutional copies are linked below.
Abstract-only verification cannot support unreported subgroup, risk-of-bias, or statistical claims; full-text access was uneven.
Self-hosted retrieval returned cookie/access failures for some PubMed/Wiley/DOI pages; web retrieval, publisher text, or author copies supplied the stated evidence instead.
No dataset, figure, assessment bank, or article text is licensed for redistribution merely because its paper is cited.

- **S1.** Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). *Distributed practice in verbal recall tasks: A review and quantitative synthesis*. Psychological Bulletin, 132(3), 354–380. [DOI / primary record](https://doi.org/10.1037/0033-2909.132.3.354). Abstract/bibliographic findings checked; verbal-recall scope retained.
- **S2.** Mawson, R. D., & Kang, S. H. K. (2025). *The distributed practice effect on classroom learning: A meta-analytic review of applied research*. Behavioral Sciences, 15(6), 771. [Article](https://doi.org/10.3390/bs15060771). Full-text methods, abstract, discussion, and limitations inspected.
- **S3.** Yang, C., Luo, L., Vadillo, M. A., Yu, R., & Shanks, D. R. (2021). *Testing (quizzing) boosts classroom learning: A systematic and meta-analytic review*. Psychological Bulletin, 147(4), 399–435. [Primary record](https://pubmed.ncbi.nlm.nih.gov/33683913/) · [DOI](https://doi.org/10.1037/bul0000309). Abstract and metadata checked.
- **S4.** Agarwal, P. K., Nunes, L. D., & Blunt, J. R. (2021). *Retrieval practice consistently benefits student learning: A systematic review of applied research in schools and classrooms*. Educational Psychology Review, 33, 1409–1453. [DOI](https://doi.org/10.1007/s10648-021-09595-9) · [Author copy](https://pdf.poojaagarwal.com/Agarwal_etal_2021_EDPR.pdf). Abstract and publication metadata checked; the correct publication year is 2021.
- **S5.** Wisniewski, B., Zierer, K., & Hattie, J. (2020). *The power of feedback revisited: A meta-analysis of educational feedback research*. Frontiers in Psychology, 10, 3087. [Full text](https://doi.org/10.3389/fpsyg.2019.03087). Full-text abstract/results inspected; the DOI contains 2019 but publication is 2020. No single pooled effect is used here because the abstract and unadjusted-results summaries differ.
- **S6.** Latimier, A., Peyre, H., & Ramus, F. (2021; online 2020). *A meta-analytic review of the benefit of spacing out retrieval practice episodes on retention*. Educational Psychology Review, 33, 959–987. [Publisher](https://doi.org/10.1007/s10648-020-09572-8). Publisher abstract and issue/online dates checked.
- **S7.** Murray, E., Horner, A. J., & Göbel, S. M. (2025). *A meta-analytic review of the effectiveness of spacing and retrieval practice for mathematics learning*. Educational Psychology Review, 37, 75. [Publisher](https://doi.org/10.1007/s10648-025-10035-1) · [Institutional record](https://eprints.whiterose.ac.uk/id/eprint/229807/). Publisher abstract and institutional metadata checked.
- **S8.** Bego, C. R., Lyle, K. B., Ralston, P. A. S., et al. (2024). *Single-paper meta-analyses of the effects of spaced retrieval practice in nine introductory STEM courses: Is the glass half full or half empty?* International Journal of STEM Education, 11, 9. [Article](https://doi.org/10.1186/s40594-024-00468-5). Publisher abstract checked; this studies spaced retrieval, not a nine-course interleaving intervention.
- **S9.** Brunmair, M., & Richter, T. (2019). *Similarity matters: A meta-analysis of interleaved learning and its moderators*. Psychological Bulletin, 145(11), 1029–1052. [DOI](https://doi.org/10.1037/bul0000209) · [Author manuscript](https://www.psychologie.uni-wuerzburg.de/fileadmin/06020400/2019/Brunmair_Richter_in_press__2019_META-ANALYSIS_OF_INTERLEAVED_LEARNING.pdf). Author manuscript abstract and task definitions inspected.
- **S10.** Barbieri, C. A., Miller-Cotto, D., Clerjuste, S. N., & Chawla, K. (2023). *A meta-analysis of the worked examples effect on mathematics performance*. Educational Psychology Review, 35, 11. [Publisher](https://doi.org/10.1007/s10648-023-09745-1) · [Author copy](https://www.danamillercotto.com/uploads/4/7/7/2/47725475/barbieri_et_al__2023__we_meta-analysis.pdf). Abstract and bibliographic record checked; moderator associations are not randomized component effects.
- **S11.** Sweller, J., van Merriënboer, J. J. G., & Paas, F. (2019). *Cognitive architecture and instructional design: 20 years later*. Educational Psychology Review, 31, 261–292. [Full text](https://doi.org/10.1007/s10648-019-09465-5). Worked examples, expertise reversal, and theory limitations inspected.
- **S12.** Rey, G. D., Beege, M., Nebel, S., Wirzberger, M., Schmitt, T. H., & Schneider, S. (2019). *A meta-analysis of the segmenting effect*. Educational Psychology Review, 31, 389–419. [Publisher](https://doi.org/10.1007/s10648-018-9456-4). Publisher abstract and metadata checked.
- **S13.** Bjork, R. A., Dunlosky, J., & Kornell, N. (2013; online 2012). *Self-regulated learning: Beliefs, techniques, and illusions*. Annual Review of Psychology, 64, 417–444. [DOI](https://doi.org/10.1146/annurev-psych-113011-143823) · [Author copy](https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/07/RBjork_Dunlosky_Kornell_2013.pdf). Abstract and publication metadata checked; not a feature-level intervention estimate.
- **S14.** Theobald, M. (2021). *Self-regulated learning training programs enhance university students' academic performance, self-regulated learning strategies, and motivation: A meta-analysis*. Contemporary Educational Psychology, 66, 101976. [DOI](https://doi.org/10.1016/j.cedpsych.2021.101976) · [Institutional copy](https://www.uni-trier.de/fileadmin/fb1/prof/PSY/SUK/Theobald_2021-1.pdf). Sample and academic-performance results inspected.
- **S15.** Pan, S. C., & Rickard, T. C. (2018). *Transfer of test-enhanced learning: Meta-analytic review and synthesis*. Psychological Bulletin, 144(7), 710–756. [DOI](https://doi.org/10.1037/bul0000151) · [Article copy](https://pdf.retrievalpractice.org/transfer/Pan_Rickard_2018.pdf). Abstract, sample, pooled estimate, and moderators checked.
- **S16.** Corral, D., & Carpenter, S. K. (2025). *Effects of retrieval practice on retention and application of complex educational concepts*. Learning and Instruction, 100, 102219. [Publisher full text](https://www.sciencedirect.com/science/article/pii/S0959475225001434) · [DOI](https://doi.org/10.1016/j.learninstruc.2025.102219). Abstract, experiment discussions, comparator limitations, and transfer interpretation inspected.
- **S17.** Yao, Y., Amos, M., Snider, K., & Brown, T. (2024). *The impact of formative assessment on K–12 learning: A meta-analysis*. Educational Research and Evaluation, 29(7–8), 452–475. [Publisher](https://doi.org/10.1080/13803611.2024.2363831). Publisher abstract and bibliographic metadata checked.
- **S18.** Lindsey, R. V., Shroyer, J. D., Pashler, H., & Mozer, M. C. (2014). *Improving students' long-term knowledge retention through personalized review*. Psychological Science, 25(3), 639–647. [Publisher](https://doi.org/10.1177/0956797613504302) · [Author copy](https://laplab.ucsd.edu/articles/LindseyShroyerPashlerMozer2014.pdf). Abstract and classroom-method description checked; no product uplift is inferred.
- **S19.** Macnamara, B. N., Hambrick, D. Z., & Oswald, F. L. (2014). *Deliberate practice and performance in music, games, sports, education, and professions: A meta-analysis*. Psychological Science, 25(8), 1608–1618. [Publisher](https://doi.org/10.1177/0956797614535810). Abstract and association-based interpretation checked; expertise definitions and measurement remain contested.
- **S20.** Chernikova, O., Heitzmann, N., Stadler, M., Holzberger, D., Seidel, T., & Fischer, F. (2020). *Simulation-based learning in higher education: A meta-analysis*. Review of Educational Research, 90(4), 499–541. [Full text](https://doi.org/10.3102/0034654320933544). Sample composition, mixed controls/designs, and high heterogeneity inspected.
- **S21.** DeFouw, E. R., Klingbeil, D. A., Codding, R. S., Latham, A. D., Pigg, B., & McCroskey, G. (2026; online 2025). *The impact of distributed practice schedules within a classwide computation intervention: A randomized control trial*. Journal of School Psychology, 114, 101522. [Primary record](https://pubmed.ncbi.nlm.nih.gov/41478700/) · [DOI](https://doi.org/10.1016/j.jsp.2025.101522). Abstract checked; publication-year/DOI-year difference retained.
- **S22.** Kim, S. K., & Webb, S. (2022). *The effects of spaced practice on second language learning: A meta-analysis*. Language Learning, 72(1), 269–319. [Publisher](https://doi.org/10.1111/lang.12479). Publisher abstract and scope/moderator descriptions checked.
