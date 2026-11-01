# Workspace status visual follow-up

User-provided screenshots showed stale temporary-material wording and a status line outside the centered reading-preferences panel. The ready message now describes stored reading preferences only; the panel shares the existing content width. The recovery message also stops claiming that only preferences are saved.

Validation: 16 related tests passed, current TypeScript/Vite/RPM build and diff check passed. Two subsequent user-provided images confirm the corrected text and alignment. This establishes the initial empty-workspace presentation only, not authoring persistence or GNOME/Orca acceptance. No screenshots were captured by the agent. Images remain ignored under .qualification/m2/manual-gui/user-screenshots.

Current RPM SHA-256: 8b9a4f88bdc2d97c422d1a0eb4c1dac2942f9067f11259ace409792a0fc9a8d8. The visible manual run uses the existing separate test profile and process-only WEBKIT_DISABLE_DMABUF_RENDERER=1. No host installation or driver/configuration change. Prior process had already exited normally before relaunch.

Next: create an arbitrary subject and material, then review the user-provided authoring screenshot. KER-116 and dev-to-main integration remain open.

Used: engineering-workflow/Superpowers debugging and verification, Memoquill workflow/reporting; Codebase Memory graph/snippet/trace/coverage, RTK/Python/Git/npm/Tauri. No browser or research plugins used in this follow-up.

Host AT-SPI metadata probe exposed no Memoquill accessible application. No desktop accessibility setting was changed; manual screenshots do not establish keyboard or Orca acceptance. The visible run predates only the final recovery-copy change; restart the current candidate before integrated acceptance.
