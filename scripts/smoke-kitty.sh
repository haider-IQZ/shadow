#!/usr/bin/env bash
# Manual X11/Mesa smoke test on an Arch runtime with Xvfb and xdotool.
set -euo pipefail
work=$(mktemp -d)
kitty_pid=
x_pid=
cleanup() {
    [[ -z "$kitty_pid" ]] || kill "$kitty_pid" 2>/dev/null || true
    [[ -z "$x_pid" ]] || kill "$x_pid" 2>/dev/null || true
    printf 'Smoke-test diagnostics: %s\n' "$work"
}
trap cleanup EXIT
Xvfb -displayfd 3 -screen 0 1280x800x24 3>"$work/display" >"$work/xvfb.log" 2>&1 &
x_pid=$!
for ((i=0; i<100; i++)); do
    [[ ! -s "$work/display" ]] || break
    kill -0 "$x_pid"
    sleep 0.1
done
export DISPLAY=":$(<"$work/display")"
export LIBGL_ALWAYS_SOFTWARE=1
shadow run kitty -- --config NONE --title shadow-source-smoke sh -eu -c '
    test "${PYTHONHOME+x}" != x
    test "${OPENSSL_MODULES+x}" != x
    printf "Shadow source-built Kitty is rendering.\n"
    touch "$1"
    sleep 60
' shadow-child "$work/child-ok" >"$work/kitty.log" 2>&1 &
kitty_pid=$!
window=
for ((i=0; i<200; i++)); do
    kill -0 "$kitty_pid"
    window=$(xdotool search --name '^shadow-source-smoke$' 2>/dev/null || true)
    if [[ -n "$window" && -f "$work/child-ok" ]]; then break; fi
    sleep 0.1
done
[[ -n "$window" && -f "$work/child-ok" ]] || { tail -40 "$work/kitty.log"; exit 1; }
pid=$(xdotool getwindowpid "${window%%$'\n'*}")
awk '/\.so/ {print $NF}' "/proc/$pid/maps" | sort -u >"$work/libraries"
for lib in libpython libfontconfig libfreetype libharfbuzz libcairo liblcms2; do
    grep -E "/Cellar/.*/$lib[^/]*" "$work/libraries" >/dev/null || {
        printf 'Missing Shadow runtime library: %s\n' "$lib"; exit 1;
    }
    if grep -E "/$lib[^/]*" "$work/libraries" | grep -v /Cellar/; then
        printf 'Unexpected host application library: %s\n' "$lib"; exit 1
    fi
done
printf 'PASS: Kitty window opened, terminal child ran without private Python/OpenSSL environment leakage, and core libraries loaded from Shadow.\n'
