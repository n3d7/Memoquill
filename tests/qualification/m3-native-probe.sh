#!/usr/bin/env bash
# Run only inside the private rootless qualification desktop, under dbus-run-session.
set -euo pipefail
ulimit -c 0
if [[ "${1:-x11}" != x11 ]]; then
  echo 'M3 qualification supports only the authorized Linux/X11 target' >&2
  exit 2
fi
if [[ ! -f /input/m3-journey.py ]]; then
  echo 'M3 journey input is missing; no qualification was performed' >&2
  exit 2
fi
export XDG_RUNTIME_DIR=/tmp/memoquill-m3-runtime
export XDG_DATA_HOME=/tmp/memoquill-m3-data
export XDG_CONFIG_HOME=/tmp/memoquill-m3-config
export XDG_CACHE_HOME=/tmp/memoquill-m3-cache
mkdir -p "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
chmod 700 "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"
export DISPLAY=:99 GDK_BACKEND=x11 GTK_MODULES=atk-bridge GTK_IM_MODULE=ibus NO_AT_BRIDGE=0 LANG=C.UTF-8
Xvfb :99 -screen 0 1440x1100x24 -nolisten tcp >/evidence/xvfb-x11.log 2>&1 &
trap 'kill $(jobs -pr) 2>/dev/null || true' EXIT
ready=0
for attempt in {1..100}; do
  if xdotool getdisplaygeometry >/dev/null 2>&1; then ready=1; break; fi
  sleep .1
done
if [[ "$ready" != 1 ]]; then echo 'Private X11 display did not become ready' >&2; exit 1; fi
openbox --sm-disable >/evidence/openbox.log 2>&1 &
dbus-update-activation-environment DISPLAY XDG_RUNTIME_DIR XDG_DATA_HOME XDG_CONFIG_HOME XDG_CACHE_HOME GDK_BACKEND GTK_IM_MODULE NO_AT_BRIDGE LANG
gsettings set org.gnome.desktop.interface toolkit-accessibility true
export IBUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/ibus.sock"
dbus-update-activation-environment IBUS_ADDRESS
ibus-daemon --xim --replace --desktop=other --address="$IBUS_ADDRESS" >/evidence/ibus-x11.log 2>&1 &
ready=0
for attempt in {1..100}; do
  if ibus list-engine >/evidence/ibus-engines-x11.txt 2>/evidence/ibus-client-x11.log; then ready=1; break; fi
  sleep .1
done
if [[ "$ready" != 1 ]]; then echo 'Private IME did not become ready' >&2; exit 1; fi
app_command=(memoquill)
if [[ "${MEMOQUILL_NETWORK_TRACE:-0}" == 1 ]]; then
  app_command=(strace -f -qq -e trace=socket,connect,sendto,sendmsg,sendmmsg -s 32 -o /evidence/network-x11.log memoquill)
fi
"${app_command[@]}" >/evidence/app-x11.log 2>&1 &
app_pid=$!
printf '%s\n' "$app_pid" >/evidence/app-pid-x11
timeout 180 python3 /input/m3-journey.py x11
# The journey closes the original window normally and may exercise a controlled restart.
wait "$app_pid"
printf '0\n' >/evidence/normal-exit-x11.txt
printf '%s\n' '{"status":"not_run","reason":"M3 functional X11 journey; human audible speech/reflow remains M7"}' >/evidence/speech-x11-skip.json
