# M4 AlreadyOpen: isolated Podman diagnosis

2026-11-14. The user requested diagnosis in a separate Podman desktop. The
visible workstation application was not controlled or restarted. No screenshots
were captured, personal profiles mounted, host policy changed, or product source
modified during this diagnosis.

## Result and limits

The original desktop failure is **not reproduced or attributed conclusively**.
The installed candidate closed normally and reopened in three isolated cases:
empty-profile Alt+F4, empty-profile window-manager close, and Alt+F4 after the
22-assertion synthetic M4 journey (authoring, practice, FSRS, reminder, assessment).
Each original process exited with code 0; the observed owner locks were released;
each replacement process displayed its window. These are three native scenarios,
not additions to the existing unit-test totals. The second close method is a
window-manager probe and is not substituted for the Alt+F4 result.

All three busy-owner negative controls returned AlreadyOpen while the original
instance was still running. This confirms that a live workspace owner triggers
the protection; merely leaving a lock file on disk does not establish a stale lock.

The previous workstation observer sent an unqualified Alt+F4, slept two seconds,
then launched a second process without establishing that the first had exited.
That is an evidenced gap in the test, not proof of the original failure's cause.
A controlled different-window focus probe kept the target alive and its locks
held; activating and closing the target released those locks and allowed reopen.
However, its concurrent-launch observation did not obtain AlreadyOpen within the
bounded window. The two focus attempts were unsuccessful/inconclusive as exact
reproductions (timeout, then negative-control assertion); they are not counted as
passed tests, and this branch of investigation was stopped.

No product fix is claimed. A local guarded observer now activates the owned
window, checks its active PID, and waits for actual process termination before
relaunch; it fails without launching a conflicting copy if exit is unconfirmed.
That observer revision has not been executed on the workstation. The original
normal-close/reopen desktop receipt remains unqualified. Container reopen also
does not independently assert restored assessment or FSRS content; the previously
recorded controlled desktop restart remains the separate data-recovery evidence.
The PR remains draft; no merge or tracker change is authorized/performed.

## Evidence and next action

Local evidence: `.qualification/already-open-podman/evidence/diagnosis.json`,
`results.json`, `populated-results.json`, `focus-before-close.json`,
`focus-results.json`, `focus-prelaunch.json`, and the corresponding process logs.
Reproducers and guarded observer: `.qualification/already-open-podman/input/`.
`manifest.sha256` records retained local artifacts. No image rebuild was needed:
`localhost/memoquill-m4-gui:20261114`, image ID
`74858da9b68169fda45f8be66c3eb4ffeea713675a387280c141ba7e5ff95021`.
The run used the documented disposable rootless desktop boundary, private
Xvfb/DBus/XDG directories, no network, dropped capabilities and bounded resources;
there were no host desktop or DBus mounts. The inherited label/seccomp exceptions
apply only to this disposable qualification recipe.

The first populated-probe attempt failed before product interaction because the
copied harness lacked `ibus_control.py`; that setup failure is not a product test
result. Its dependency was supplied before the successful populated run.

Next meaningful evidence: use the guarded observer when returning to the visible
synthetic desktop profile, recording the original PID, focused PID, exit and lock
state at the exact failure. Do not infer a product shutdown defect or remove owner
locks without that evidence. Do not repeat unchanged successful unit/full gates:
this diagnosis changed no product source. Existing full 540 passed / 0 failed /
10 helper skips and its audit remain a separate preexisting receipt.

PR hygiene: remove the accidentally tracked local `node_modules` symlink from the
publication branch; it is a workstation dependency pointer, not product content.
The original dirty `dev` worktree and all prior user work remain preserved.

Tools: RTK for compact Git/search output, rootless Podman, Xvfb/Openbox, X11
keyboard/AT-SPI driver, Python process/flock probes, Serena symbol reads.
Workflow: `diagnosing-bugs`; no agents or watcher loops were created.
