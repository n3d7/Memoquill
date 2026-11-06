# KER-97 / M1.1 final acceptance plan

Requested after the M2 checkpoint, then explicitly deferred by the user on 2026-11-06 in favor of continuing M2. Host package installation was not approved or performed. Container testing was suggested; it must not be reported as physical GNOME acceptance.
Implementation source5c72fdb; M2 checkpoint commit e84dca6. Work stays on dev,
local substep commits only; no push/merge/release or M3 start.

The [live issue](https://linear.app/kerigan/issue/KER-97/m11-qualify-the-fedora-desktop-and-accessibility-baseline)
and [ADR-02](../architecture/decisions.md) retain physical Fedora/GNOME and
human audible Orca acceptance before Done. Historical isolated automation does
not establish those human/hardware results.

Current read-only findings:
- Physical host: Fedora44, i3/X11, LightDM1.33.1; GNOME shell/session and Orca absent.
- Current WebKitGTK2.54.2, GTK3.24.52 and glib2.88.4 are version-recorded.
- Default NVIDIA/GBM startup is fixed and user-confirmed. Actual-host native UI
  saved exercise/answer/text-scale survive process restart in a synthetic profile.
- One warm run:0.124s visible window,0.355s authorable AT-SPI control; resident
  family sum478844KiB after actions. Shared pages are double-counted; this is not
  a cold-start result, comparative benchmark or learning-outcome claim.
- Nested Wayland abort, failed saved speech audit and cold-Orca readiness remain
  separately recorded. No source/desktop fallback is selected from warnings alone.

Concrete package proposal (not applied):
- DNF install gnome-shell, gnome-session and orca with install_weak_deps=False.
- Cached resolver:54new packages,40MiB download,178MiB additional disk; no
  upgrades/removals. Includes GNOME50.6, session50.1, Orca50.3, Mutter and GDM.
- Do not log out, restart services or switch the current LightDM automatically.
  Preserve its current default; session/login-manager changes require a separate
  agreed step after package preparation.
- This changes the host OS beyond interacting with Memoquill; obtain explicit
  approval before installing. Recheck any materially changed transaction first.

Then: enter an intended GNOME/Wayland session when the user is ready; launch the
qualified RPM offline, perform keyboard/IME/dialog/focus/reflow checks and have
the user confirm actual Orca speech. Record exact candidate/environment/results.
Close KER-97 only against accepted evidence or an explicitly approved ADR change;
never turn unrun checks into passes.

Read-only preparation and package logs: .qualification/m11-followup/.
