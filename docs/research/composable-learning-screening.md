# Bounded follow-up: research admission and lesson sources

Date: 2026-11-01. Applied scientific-critical-thinking appraisal and literature-review search/screening guidance; arxiv skill guidance and the native arXiv MCP supplied metadata/sections. This is a targeted pass, not a preregistered systematic review, formal GRADE assessment or product-effectiveness experiment.

## Search and verification

- arXiv: two searches, 2023 onward, four results each, for generative/declarative interfaces and tutor/activity authoring. Inspected selected full abstracts; downloaded 2405.14713v1 and read its evaluation/results/limitations sections. AI2T's PDF-only conversion was unavailable in the existing MCP; no server/dependency was installed.
- Consensus: two focused searches on learner control/cognitive load and individual game elements, ten results each; used fetch for selected candidates. One broad query was rejected and replaced with one shorter focused query. Search/fetch content was treated as provisional until verified.
- Verified the selected cognitive-load review at its Frontiers publisher page. ScienceDirect retrieval failed through selfhost; after checking the cloud balance, one schema-correct hosted scrape succeeded. Its first request failed schema validation because the live connector requires an objective; no repeated paid extraction was run. Cloud/source success is not a claim of actual final billing.
- Kept public raw results locally under `.qualification/m2/adaptive-ui-research`. Do not commit whole papers. Existing architecture/learning-science sources were reused, not fetched again to inflate coverage.

## Admission decisions

| Work / reading depth | Decision | Applicability and limit |
| --- | --- | --- |
| [Educator-driven tutor authoring, 2405.14713v1](https://arxiv.org/abs/2405.14713v1), methods/results/limitations | Keep as limited authoring precedent | Whole-interface generation, component generation and direct refinement fit manual/agent parity. Preliminary comparison used four team members and a historical expert comparator, with a keystroke model. Outcomes were authoring time/keystrokes, not learner retention/transfer; do not reuse its percentages as Memoquill learning-speed promises. Peer-review status was not independently verified. |
| [Qian et al., 2026 cognitive-load review](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2026.1921504/full), publisher abstract and indexed metadata | Keep as a measurement caveat | Reported outcomes/contexts and load measures are heterogeneous. Reduced effort can support a learner or substitute for the target thinking. Do not infer durable learning from a pooled/subjective load score, faster generation or task completion. This review does not establish one universally beneficial AI interaction. |
| [Mekler et al., 2017 individual game elements](https://www.sciencedirect.com/science/article/pii/S0747563215301229), publisher highlights/abstract | Keep as a metric-design warning | An image-annotation experiment found increased tag quantity without the same change in intrinsic motivation/competence need satisfaction. It is not a lesson-retention study and does not prove that badges/points always help or harm. Game/task volume must not be labeled learning evidence. |
| [Efficient Personalization of Generative User Interfaces, 2604.09876v2](https://arxiv.org/abs/2604.09876v2), abstract only | Future option; not a core requirement | Small preference studies suggest UI preferences differ. No learning-effect claim was verified. Keep explicit learner overrides and evaluation extensibility; do not add a learned personalization model/service now. |
| [AI2T, 2411.17924](https://arxiv.org/abs/2411.17924), metadata/abstract only | Future research pointer | Interactively teaching a tutor/model-tracing agent may later inform authoring assistance. Full text was not obtained through the configured PDF path; no reliability/efficiency claim is admitted. No adaptive tutor or learned evaluator is added to M2. |

VR/3D authoring, serious-game architectures, OS-agent benchmarks and design-to-code pipelines were not promoted into the core plan: their tasks/runtime assumptions are outside the requested offline lesson constructor. The abstract-only working-definition paper (2505.15049) did not change a concrete decision and was not retained as a future work item. Other unverified indexed reviews remain search leads in the ignored snapshot, not extra architecture requirements. Gamification-bundle meta-analyses from the first pass do not validate every isolated mechanism.

## Narrow architecture consequences

1. Keep the manual editor and whole/component-level agent proposals on the same bounded semantic service path. Preview/apply/undo and learner override remain explicit; papers that generate code are not authority to execute generated code in Memoquill.
2. Pin content/activity/composition versions during real study attempts. Mid-session regeneration is staged for an explicit safe boundary; it must not silently change a question, answer policy, reading position or focus target. A live edit requires defined conflict/transition behavior.
3. Keep **authoring assistance** separate from **assistance received by the learner during a study attempt**. AI-authored material is provenance; it does not automatically make an otherwise unaided learner response assisted. Conversely, AI solving the learner's task does not establish independent competence.
4. Give metrics a declared meaning/unit/denominator/provenance. Task volume, completion, authoring time, UI preference, self-report, reward progress and target learning outcomes remain distinct. Recommended mechanisms need appropriate learning/time checks; custom safe experiments remain possible without scientific labels.

These are correctness/control/measurement requirements supported by the user goal and relevant evidence, not an empirical claim that this architecture maximizes learning speed. No automatic learner-modeling pipeline, universal cognitive-load score or new runtime dependency is admitted.

## Lesson-scoped research files and books

The user's additional requirement belongs to M3/KER-100 and M6/KER-106. A lesson has scoped source bindings to immutable managed originals and extracted/normalized source revisions. Preserve bibliographic metadata, actual used ranges, extraction coverage/warnings and format-appropriate locators. Source facts, teaching-method evidence and personal learning outcomes are distinct.

Target a broad qualified format matrix including PDF/EPUB books, DOCX, text/Markdown/HTML and supported other document/image formats. Report view/extract/partial/original-only capability honestly; do not promise universal parsing or silently treat an unread book as processed. EPUB is now an explicit M3 qualification target, superseding its old blanket deferral. OCR/complex formats remain conditional. Existing developer Docling/MarkItDown/arXiv tools do not prove product support or justify adding their dependencies automatically.

Manual source use remains useful offline. Later agents read only authorized lesson/source ranges, select relevant material for the learner's objective/prerequisites, preserve quotations/provenance and propose content/activity/composition changes through the same validation/review or matching delegated-policy path. Distinguish extraction, source interpretation and educational synthesis. Flag partial/unread coverage and uncertain claims.

Importing a file does not authorize uploading it to an arbitrary remote provider. Preserve source-read scopes, destination choice, safe passive parsing, bounded workers/archives/resources and export/recovery semantics. Parsing is offline by default: no automatic linked-resource fetch, external XML entities, macros/scripts or paths outside staged input. Document text is untrusted data, never a new tool instruction or grant. Do not automatically redistribute private originals in public lesson packs.
