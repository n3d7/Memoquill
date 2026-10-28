# Audit of the user-supplied Consensus review

Checked: **2026-10-27**. Scope: decision-relevant claims and source provenance, not a complete audit of every cited paper or a formal systematic review.

## Input and handling

- User-provided file: `Conduct_an_evidence-focused_deep_research_review_t.pdf`, originally under the user's Downloads directory; 12 pages, 523,492 bytes.
- SHA-256: `c8f1642ca25e64ded942f2f0c040940f4f9b966cb86f9efbe59a862543d943e4`.
- [User-provided Consensus share URL](https://consensus.app/search/evidence-based-learning-activities/g8oCjbeOStGerDnX5QVHlQ/).
- Read locally using Docling's native PDF pipeline; Markdown extraction produced 31,801 bytes. A separate Poppler layout-text extraction and visual check of page 2 support table/funnel interpretation. Native extraction loses some figure/table reading order and rendered citation links; bibliographic identifiers were checked against primary sources.
- The original was not modified or copied into the repository. Extraction/cache files remain temporary; these audit/synthesis documents preserve the useful findings and attribution.
- The share page was inaccessible through the initial web retrieval. The supplied PDF is the report input. No Consensus API search/fetch/thread call was made in this extension, and the recorded API quota was not reset by the user supplying a separate Deep report.

## How much weight to give the report

The report is a useful discovery/synthesis input. Its search funnel (page 2: 1.4M retrieved, 947 eligible, 100 included) is the report's claim, not an independently reproduced systematic-search result. The PDF does not provide enough exact query/screening/exclusion data here to audit that flow. Neither the funnel nor a generated evidence-strength figure establishes that all underlying studies were appraised consistently.

This extension instead checks a bounded set of consequential findings against primary articles, publisher abstracts, author copies and arXiv versions. Abstract-only access, working papers, unverified publication status and intervention/comparator differences remain explicit. No formal GRADE certainty ratings, new meta-analysis or original experiment were performed.

## Claim checks that change or sharpen the product

| Report location / issue | Independent check | Decision for Memoquill |
| --- | --- | --- |
| Pages 1–3 and 5–7: retrieval formats and transfer | A1 in [learning-activities.md](learning-activities.md) directly compares formats across four experiments; retrieval success and test demands matter. | Keep open answers and choice. Do not present a universal format ranking or equate all choice questions with simple factual recognition. |
| Page 3: novice guidance and productive failure | A5 provides a substantial positive synthesis for designed problem-solving-before-instruction, with important moderators. The report's cited Wesenberg paper concerns correct/erroneous example order and is not by itself a synthesis of the whole productive-failure literature. | Support example-first and bounded attempt-first sequences, both with explicit explanation/consolidation. Avoid an “always instruct first” or “always struggle first” rule. |
| Pages 3/5: feedback is more than correctness | A2 verifies useful explanatory feedback, while its pooled differences cannot identify one universally optimal timing/UI. | Make answer/rubric/explanation/citation available, retain correction and later application, and keep feedback policy configurable. |
| Pages 3/5/6: AI can help or hinder learning | A9/A10 are different controlled interventions/outcomes; A11 adds a knowledge-null comparison among AI conditions, and A14 reports costs of a particular restriction policy. | Preserve optional source-linked help and assistance provenance. Assess independent outcomes separately; do not generalize one trial into a universal assistance lockout. |
| Pages 3/5 and bibliography: supervised AI evidence | A12 randomizes tutors and measures session outcomes; A13 has N=165 with expert supervision of each AI message. Neither establishes a generic autonomous tutoring effect. | Record supervision, assignment unit and time horizon. Do not convert learning-assistance evidence into a requirement to weaken authorized MCP authoring autonomy. |
| Pages 5–7: progress, practical skill and self-regulation | Existing science register and A8/A11 distinguish performance, non-use, motivation, knowledge and practical evidence. | Keep outcome-specific progress; no universal mastery percentage or recipe competence inferred from quizzes/checklists. |
| Bibliography: hint-button association | A7 is an observational log analysis with covariate adjustment, not random assignment to harmful hints. | Record/review hint use without claiming hints cause lower learning or removing accessibility/help paths. |

## Bibliographic corrections and access limits

- **Agarwal, Nunes and Blunt:** the report uses 2019; the verified journal record is **2021**, DOI `10.1007/s10648-021-09595-9` (already corrected in the original science register).
- **Van der Kleij, Feskens and Eggen:** the report uses 2013; the checked publisher record for DOI `10.3102/0034654314564881` identifies **2015**. Use that record, not the discovery year label.
- **Smith and Karpicke:** distinguish online 2013 from the 2014 journal issue; both refer to the same four experiments, not independent replications.
- **Fütterer et al.:** the publisher shows online publication **14 April 2026** while assigned issue metadata uses December 2026. The accessible online paper is not excluded merely because the issue date is later.
- **LearnLM Team/Eedi:** replace the opaque shorthand “Team, L.” with an identifiable group/source and exact arXiv version; the full source record lists contributors.
- Greving/Richter's title/DOI were discovered, but direct public full-text retrieval failed on both local and cloud routes in this pass. No unverified numerical result from that paper is used. Smith/Karpicke's primary paper supplies an independently read format counterexample.
- Some cooking/video and other bibliography leads were screened but not incorporated as new verified effect estimates when only search descriptions or blocked pages were available. In particular, no claim of safe cooking competence follows from confidence/video-viewing findings.

## arXiv and other new tools actually used

The arXiv MCP was used for focused discovery, abstracts, versioned download, outline, section reads and within-paper passage search. Tutor CoPilot v3 was retrieved through the arXiv HTML endpoint; its methods reveal tutor-level randomization and session-level outcomes that an abstract-only reading could miss. LearnLM/Eedi v1 was checked at abstract level. These are bounded reads, not a comprehensive arXiv survey or proof that a missing journal reference means a paper has never been peer reviewed.

Self-hosted Firecrawl searches worked on the resumed check and supplied the primary discovery route. Some searches remained empty and some publisher/PMC pages returned no usable text/cookie challenges. Cloud Firecrawl successfully filled selected publisher reads, including feedback, productive failure and Bastani; it did not solve every retrieval. No service was installed/reconfigured and no paywall was bypassed. Existing valid evidence was reused instead of refetching the entire original literature.

## Workflow and appraisal acknowledgment

The existing Memoquill workflow owns the task. `skill-catalog` metadata was checked: 449 indexed entries, no stale/missing paths or metadata errors at that check. Docling handled PDF extraction; `scientific-critical-thinking` supported result/comparator/time-point appraisal, bias/confounding distinctions and reporting limits. No ARIS autonomous experiment workflow, model training or new testing infrastructure was started.

As required by the materially used Scientific Agent Skills guidance, its methodological source is acknowledged here, **not counted as learning-effectiveness evidence**: Kassis, T., Agarwal, V., He, Y., Patel, D., & Brueckner, A. M. (2026). *Scientific Agent Skills: A Library of Procedural Knowledge for Research Agents*. [arXiv record](https://arxiv.org/abs/2609.00065), [DOI](https://doi.org/10.48550/arXiv.2609.00065). Author/year/version metadata were checked with the arXiv MCP (record returned v2); the stable citation is unsuffixed.

## Remaining uncertainty

Which prompts ordinary users can author well; what delayed/transfer checks are feasible in self-study; whether optional AI improves independent outcomes for a particular learner/task; and whether advanced drawing/ordering tools justify their complexity remain implementation/evaluation questions. The reviewed evidence informs a coherent portfolio; it does not prove that Memoquill already improves learning.
