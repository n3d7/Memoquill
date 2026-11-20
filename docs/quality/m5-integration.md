# M5 progress, continuity and catalog integration

This slice of KER-104 provides lesson evidence, saved-session Resume and Due,
reading and search context, inherited settings, subject overrides, keyboard
bindings and bounded navigation through large catalogs. Progress shows
observations, judgments and pinned revisions. It does not infer mastery.

The SQLite v10 migration adds study indexes. A populated v9 database keeps a
standalone v9 snapshot during upgrade. Native archive admission explicitly
supports schemas 7 through 10 and still checks their exact DDL. To return to
an older build, use a verified compatible backup and preserve any history added
after the upgrade for a supported recovery. Opening a v10 database with a v9
build is unsupported.

## Evidence

Local verification covered 323 Rust tests, with 10 ignored helpers, and 244
frontend tests before the final accessible-name correction. The focused final
regression ran 19 frontend tests and one Rust test. The runner audited the test
reports and confirmed Rust formatting and Clippy.

Installed X11 qualification used a private, offline Podman desktop. The latest
candidate passed 19 assertions on the full fixture, including Orca speech
requests for the first, middle and last material, the catalog count and search
positions. It also passed 24 assertions for settings recovery and scheduled Due
over restarts and 26 for subject and keyboard customization on small fixtures.
Archive sorting and archive/restore search updates passed separately on the
preceding candidate. No screenshots or personal learning data were used.

The full synthetic dataset has 100 subjects, 10,000 resources, 100,000
activities, 1,000,000 attempts and 10 GB of managed files. The isolated v9 upgrade
created a snapshot. Backend search p95 was about 26 ms over 50 queries and
progress projection took 42–167 ms in three samples. One later packaged run
reached the useful view in 5.7 seconds and finished its initial study reads
19.5 seconds after that. Cache conditions were uncontrolled. The three-second
launch budget remains open. The release-mode Due read still takes about
18 seconds on this dataset after statement reuse and bounded offer selection.

## Remaining M5 work

- Manual audible Orca, text scaling, reflow, contrast and reduced motion.
- Formative usability work with participants and the remaining offline journeys.
- Recovery designed for the full scale dataset. The v1 archive caps are still
  256 MiB for the database, 512 MiB for the archive and 4098 entries. Existing
  supported-size recovery tests and a v9 migration snapshot do not establish
  a full 10 GB archive roundtrip.
- Measure recovery, startup and long operations against the published budgets.

The milestone remains open after integrating this slice.
