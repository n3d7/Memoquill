#!/usr/bin/env bash
set -euo pipefail
ulimit -c 0
mode="${1:-x11}"
export XDG_RUNTIME_DIR="/tmp/memoquill-runtime-$mode"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export DISPLAY=:99
export GTK_MODULES=atk-bridge
export GTK_IM_MODULE=ibus
export NO_AT_BRIDGE=0
export LANG=C.UTF-8
Xvfb :99 -screen 0 1280x1024x24 -nolisten tcp >"/evidence/xvfb-$mode.log" 2>&1 &
xvfb_pid=$!
trap 'kill $(jobs -pr) 2>/dev/null || true' EXIT
for attempt in {1..50}; do if xdotool getdisplaygeometry >/dev/null 2>&1; then break; fi; sleep .1; done
if [[ "$mode" == wayland ]]; then
  weston --backend=x11 --renderer=pixman --socket=memoquill-wayland --no-config --width=1280 --height=1000 --idle-time=0 > /evidence/weston.log 2>&1 &
  for attempt in {1..50}; do if [[ -S "$XDG_RUNTIME_DIR/memoquill-wayland" ]]; then break; fi; sleep .1; done
  export WAYLAND_DISPLAY=memoquill-wayland
  export GDK_BACKEND=wayland
else
  export GDK_BACKEND=x11
  openbox --sm-disable > /evidence/openbox.log 2>&1 &
  for attempt in {1..50}; do if xprop -root _NET_SUPPORTING_WM_CHECK 2>/dev/null | grep -q 'window id'; then break; fi; sleep .1; done
fi
dbus-update-activation-environment DISPLAY XDG_RUNTIME_DIR GDK_BACKEND GTK_IM_MODULE NO_AT_BRIDGE LANG
if [[ "$mode" == wayland ]]; then dbus-update-activation-environment WAYLAND_DISPLAY; fi
gsettings set org.gnome.desktop.interface toolkit-accessibility true
export IBUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/ibus.sock"
dbus-update-activation-environment IBUS_ADDRESS
ibus-daemon --xim --replace --desktop=other --address="$IBUS_ADDRESS" >"/evidence/ibus-$mode.log" 2>&1 &
ibus_ready=false
for attempt in {1..100}; do
  if ibus list-engine >"/evidence/ibus-engines-$mode.txt" 2>"/evidence/ibus-client-$mode.log"; then ibus_ready=true; break; fi
  sleep .1
done
if [[ "$ibus_ready" != true ]]; then cat "/evidence/ibus-$mode.log" "/evidence/ibus-client-$mode.log"; exit 1; fi
if [[ "${MEMOQUILL_IME_DIAGNOSTIC:-0}" == 1 ]]; then
  python3 /input/ibus_control.py libpinyin
  exit
fi
orca --replace --debug --debug-file="/evidence/orca-$mode.log" >"/evidence/orca-stderr-$mode.log" 2>&1 &
orca_pid=$!
if [[ "${MEMOQUILL_SPEECH_AUDIT:-0}" == 1 ]]; then
  python3 /input/atspi-events.py "$mode" >"/evidence/events-$mode.log" 2>&1 &
fi
app_command=(memoquill)
if [[ "${MEMOQUILL_NETWORK_TRACE:-0}" == 1 ]]; then
  app_command=(strace -f -qq -e trace=socket,connect,sendto,sendmsg,sendmmsg -s 32 -o "/evidence/network-$mode.log" memoquill)
fi
WAYLAND_DEBUG=1 "${app_command[@]}" >"/evidence/app-$mode.log" 2>&1 &
app_pid=$!
printf '%s\n' "$app_pid" > "/evidence/app-pid-$mode"
python3 /input/atspi-probe.py "$mode"
python3 /input/atspi-journey.py "$mode"
if [[ "${MEMOQUILL_STORAGE_AUDIT:-0}" == 1 ]]; then
  # The journey closed the original window normally before its controlled restart.
  wait "$app_pid"
  printf '0\n' > "/evidence/normal-exit-$mode.txt"
fi
kill -TERM "$orca_pid"
wait "$orca_pid"
python3 /input/atspi-speech.py "$mode"
kill "$app_pid" "$xvfb_pid" 2>/dev/null || true
