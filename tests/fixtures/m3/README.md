# M3 passive-format fixtures

These are original synthetic fixtures authored for Memoquill qualification, with
no external books, documents, datasets, private files or learner content copied.
`generate.py` writes deterministic bytes using Python's standard library. Run it
only to regenerate fixtures; product code never invokes it or their contents.

- `original.pdf`: tiny one-page PDF byte fixture. It is not an extraction-quality
  benchmark or evidence that any PDF renderer is integrated.
- `original.epub`: small EPUB container with original XHTML/navigation/metadata.
- `original.docx`: small OpenXML container with one original paragraph.
- `hostile.epub`: traversal entry and external-entity declaration. It must stay
  inert as a preserved original; archive support never grants extraction authority.
- `hostile.html`: script and remote image strings which must remain passive text.

The fixtures qualify preservation/status/no-execution behavior in application
tests. Their existence alone does not establish format conformance or passed tests.
