# Source register and research method

Snapshot: **2026-10-27**. This is targeted research for product/engineering decisions, not a systematic review, original experiment, legal opinion, or proof of Memoquill effectiveness. Public information only was used for external retrieval; no learner content/credentials were sent. Source documents preserve direct supporting links, exact access limitations and version distinctions.

## Evidence locations

| Area | Canonical register and verification scope |
| --- | --- |
| Learning science | [learning-science.md](learning-science.md), S1–S22: publisher/primary author copies or abstracts, samples/design/effects when verified, important contradictions and 2024–2026 work. Abstract-only limitations are explicit; discovery metadata was corrected against primary records. |
| Activity research extension | [learning-activities.md](learning-activities.md), A1–A14, and [Consensus audit](consensus-review-audit.md): selected primary evidence, versioned arXiv reads, source access limits, response-format/sequencing/assistance decisions and input provenance. The user supplied a separate Deep report; no additional Consensus API call was made. |
| Educational UX | [educational-ux.md](educational-ux.md), E/H/S labels: primary experiments, standards and heuristics distinguished. Display-polarity evidence is abstract-only, not a universal theme recommendation. |
| Existing applications | [existing-applications.md](existing-applications.md): exact retrieved modules/tags/commits/licenses for Anki, Mnemosyne, fsrs-rs and Joplin. Selective inspection, not a code/security audit; upstream applications/tests not run. |
| Stack alternatives | [technical-options.md](technical-options.md): official framework/runtime/editor/storage/SDK docs and actual notices. Tauri 2 stable vs 3 alpha and rmcp manifest vs shipped pin explicitly distinguished. No performance/packaging spike run. |
| Ownership/licensing | [third-party-reuse.md](../architecture/third-party-reuse.md): incorporated guidance vs candidate code/data, actual license obligations and missing implementation-time inventory. |

## Consensus query ledger

The user allowed two research queries. Exactly two searches returned papers (10 results each). An earlier >500-character request was rejected by argument validation and returned no papers; the final successful response reported `searches_used=30`, `searches_limit=30`, and zero remaining. No further Consensus searches/fetches/threads were invoked. Treat the final receipt as authoritative rather than assuming a rejected call could not affect accounting.

1. “What do meta-analyses and controlled studies show about spacing and retrieval practice with corrective feedback for delayed retention and transfer, versus immediate test performance? Include classroom versus laboratory evidence, null results/moderators, second-language and STEM learning, and relevant 2024–2026 reviews. Give verified paper metadata/DOIs and sample/effect details; separate established findings from context-dependent claims.”
2. “What systematic reviews/meta-analyses show when worked examples/scaffolding, interleaving, metacognitive self-regulation and deliberate practice improve complex problem solving or procedural skill transfer? Contrast novice/expert effects, delayed performance versus quiz scores, adaptive learning evidence, and 2024–2026 findings. Report limitations and null findings, especially why factual retrieval cannot certify practical competence.”

Discovery surfaced Kim/Webb L2 spacing, Bego STEM courses, Murray mathematics, Latimier spaced retrieval, Agarwal classroom retrieval, Carpenter/Pan/Butler, Barbieri worked examples, Chernikova simulation, Chen/Paas/Sweller, and other context-specific papers. The learning register selected decision-relevant independently checked evidence, not every ranked hit. Consensus indexing/citation counts and generated summaries were not treated as study quality or independent corroboration. Publication-year discrepancies (e.g. online-first vs journal issue) were resolved in the paper register.

## Primary architecture/standards references

- [MCP current specification](https://modelcontextprotocol.io/specification/2026-07-28), [versioning](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning), [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio), [tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools), [authorization](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization), and [security guidance](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices). `/latest` resolved to 2026-07-28 during research. HTTP OAuth requirements are not substituted for the chosen local application-grant model. Modern/legacy compatibility is an explicit release gate.
- [SQLite foreign keys](https://sqlite.org/foreignkeys.html), [FTS5](https://sqlite.org/fts5.html), and [online backup](https://sqlite.org/backup.html): constraints/derived indexes/snapshot mechanisms inform engineering design; they do not automatically make DB+asset backups atomic.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) and [WCAG2ICT](https://www.w3.org/TR/wcag2ict-22/): current Recommendation and informative desktop interpretation, respectively. Memoquill has no conformance claim yet.
- [WordNet publisher license](https://wordnet.princeton.edu/license-and-commercial-use): evaluated version-specific lexical-data candidate. No dataset was downloaded/incorporated; no Oxford redistribution rights were assumed.

## Retrieval and integration limitations

Self-hosted Firecrawl searches sometimes returned successful empty payloads. Scraping supplied primary URLs worked; web retrieval filled missing searches without claiming absent evidence. Some full papers were not openly retrieved; their material use is limited and marked in the relevant document. No paywall bypass, new service installation, or infrastructure reconfiguration was attempted.

Linear's exposed `get_agent_context` method returned `Tool get_agent_context not found`; ordinary project/team/issue tools remained usable. The sole discovered team was Kerigan (KER); no Memoquill project or matching issue existed before creation. Remote operations are recorded only after successful returned results in [linear-backlog.md](../planning/linear-backlog.md). Other products were not modified.

The local repository initially contained tracked README, MIT LICENSE and `.gitignore`, with no application implementation. Codebase Memory project `home-void-Project-Memoquill` reported generation `2026-10-27T08:32:13Z`; source instructions/license were read directly where coverage was unavailable. Newly written documentation is verified as files, not assumed covered by that earlier graph generation. Scryntic guidance was read/copied/adapted only into Memoquill; its Git status, tracked diff and guidance hashes were compared with the initial snapshots.

Further research should be triggered by a decision-changing gap, a changed dependency/specification, an implementation failure, or new usage evidence. Do not repeat this literature search as a ritual before each milestone.

A public web request for the Memoquill GitHub repository returned a cache-miss retrieval error. Local README/license/Git state were inspected directly; no claim is made that the local checkout equals current remote contents. No fetch, commit or push was performed.
