#!/bin/bash
# usage: window_check.sh <tracedraw binary>
# Starts TraceDraw under Xvfb with a window manager (openbox), once with
# the splash screen and once without, and checks with xprop that the main
# window can be maximized and resized and has no maximum size. A window
# that kept the splash screen's size hints fails here (#1).
# Needs: Xvfb, openbox, xprop (x11-utils).
set -u
BIN=$(realpath "$1")
DISPLAY_NUM=${WINDOW_CHECK_DISPLAY:-:98}
export DISPLAY=$DISPLAY_NUM
WORK=$(mktemp -d)
FAIL=0

Xvfb "$DISPLAY_NUM" -screen 0 1600x1000x24 >/dev/null 2>&1 &
XPID=$!
sleep 1
openbox >/dev/null 2>&1 &
OPID=$!
sleep 1

# The newest client window's properties.
props() {
    local wid
    wid=$(xprop -root _NET_CLIENT_LIST 2>/dev/null | sed 's/.*# //' | tr ',' '\n' | tail -n 1 | tr -d ' ')
    [ -n "$wid" ] && [ "$wid" != "_NET_CLIENT_LIST:" ] || return 1
    xprop -id "$wid" WM_NAME _NET_WM_ALLOWED_ACTIONS WM_NORMAL_HINTS 2>/dev/null
}

check() {
    local name=$1 splash=$2
    mkdir -p "$WORK/$name"
    WINIT_UNIX_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 TRACEDRAW_NO_SPLASH=$splash \
        TRACEDRAW_CONFIG_DIR="$WORK/$name" "$BIN" >"$WORK/$name.log" 2>&1 &
    local app=$!
    # Wait for the main window (its title names the document) to settle.
    local out=""
    for _ in $(seq 1 60); do
        sleep 0.5
        out=$(props) || continue
        if echo "$out" | grep -q 'WM_NAME.*TraceDraw - '; then
            sleep 1
            out=$(props)
            break
        fi
    done
    kill "$app" 2>/dev/null
    wait "$app" 2>/dev/null
    echo "== $name"
    echo "$out"
    local ok=1
    echo "$out" | grep -q '_NET_WM_ACTION_MAXIMIZE_HORZ' || { echo "FAIL $name: cannot be maximized"; ok=0; }
    echo "$out" | grep -q '_NET_WM_ACTION_RESIZE' || { echo "FAIL $name: cannot be resized"; ok=0; }
    echo "$out" | grep -q 'program specified maximum size' && { echo "FAIL $name: has a maximum size"; ok=0; }
    if [ $ok = 1 ]; then
        echo "ok $name"
    else
        FAIL=1
        sed -n '1,40p' "$WORK/$name.log"
        annotate "$name" "$out" "$WORK/$name.log"
    fi
}

# On GitHub Actions, the failure also as an annotation (the properties seen
# and the start of the program's log), readable without the job's log.
annotate() {
    [ -n "${GITHUB_ACTIONS:-}" ] || return 0
    local text
    text=$(printf 'window %s\n%s\n-- log --\n%s' "$1" "${2:-no window found}" "$(sed -n '1,25p' "$3")")
    text=${text//'%'/'%25'}
    text=${text//$'\r'/'%0D'}
    text=${text//$'\n'/'%0A'}
    echo "::error title=Window check ($1)::$text"
}

check splash 0
check plain 1

kill "$OPID" "$XPID" 2>/dev/null
rm -rf "$WORK"
exit $FAIL
