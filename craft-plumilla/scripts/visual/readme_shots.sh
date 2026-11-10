#!/bin/bash
# Regenerate the README screenshots in docs/images/ from the showcase
# drawings (apps/tracedraw-cli/examples/showcase.rs), driving the debug
# build under Xvfb with the scenes in scripts/visual/readme/.
# usage: scripts/visual/readme_shots.sh
set -e
cd "$(dirname "$0")/../.."
DRAWINGS="${DRAWINGS:-$HOME/Drawings}"
OUT=docs/images
SHOTS="$(mktemp -d)"
CONFIG="$SHOTS/config"
mkdir -p "$DRAWINGS" "$OUT" "$CONFIG"
# One interface point per pixel (100 % on Linux); each scene writes its
# own preferences (theme, services) before it starts.
cargo build -q -p tracedraw
cargo run -q -p tracedraw-cli --example showcase -- "$DRAWINGS"
export DISPLAY=:99
Xvfb :99 -screen 0 1600x1000x24 >/dev/null 2>&1 & XPID=$!
trap 'kill $XPID 2>/dev/null || true' EXIT
sleep 1
scene() { # scene <name> <drawing or ""> [LANG] [theme] [settings json to merge]
    local name=$1 file=$2 lang=${3:-en_US.UTF-8} theme=${4:-light} extra=${5:-'{}'}
    # Only the scenes named in $ONLY, when it is set.
    if [ -n "$ONLY" ] && [[ " $ONLY " != *" $name "* ]]; then return; fi
    python3 - "$CONFIG/settings.json" "$theme" "$extra" <<'PY'
import json, sys
s = {"ui_size": 1.0, "theme": sys.argv[2], "update_check_asked": True, "check_updates": False, "warnings": {"missing_fonts": False, "ask_missing_fonts": False}}
s.update(json.loads(sys.argv[3]))
json.dump(s, open(sys.argv[1], "w"))
PY
    (LANG=$lang WINIT_UNIX_BACKEND=x11 TRACEDRAW_NO_SPLASH=1 TRACEDRAW_CONFIG_DIR="$CONFIG" \
        LIBGL_ALWAYS_SOFTWARE=1 ./target/debug/tracedraw $file >"$SHOTS/$name.log" 2>&1) &
    sleep 6
    TRACEDRAW_SHOTS="$SHOTS" python3 scripts/visual/drive.py "scripts/visual/readme/$name.py"
    pkill -x tracedraw 2>/dev/null || true
    sleep 1
    # The window is 1600 x 950 at the top of the 1600 x 1000 screen.
    convert "$SHOTS/$name.png" -crop 1600x950+0+0 +repage -quality 90 "$OUT/tracedraw-$name.jpg"
    echo "$OUT/tracedraw-$name.jpg"
}
# The AI service (Google Gemini, which also makes pictures) set up with
# a placeholder key, for the AI scene (no request is sent).
AI='{"ai": {"provider": "open_ai_compatible", "endpoint": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", "key": "readme", "remember_key": true}}'
scene hero "$DRAWINGS/poster.tdraw"
scene curves "$DRAWINGS/leaf.tdraw"
scene effects "$DRAWINGS/effects.tdraw" en_US.UTF-8 dark
scene type "$DRAWINGS/type.tdraw"
scene fills "$DRAWINGS/fills.tdraw" en_US.UTF-8 dark
scene bitmaps "$DRAWINGS/photo.tdraw" en_US.UTF-8 medium_dark
scene trace "$DRAWINGS/photo.tdraw"
scene export "$DRAWINGS/poster.tdraw"
scene templates ""
scene ai "$DRAWINGS/photo.tdraw" en_US.UTF-8 dark "$AI"
scene languages "$DRAWINGS/type.tdraw" ja_JP.UTF-8
scene painterly "$DRAWINGS/brushes.tdraw"
scene perspective "$DRAWINGS/room.tdraw" en_US.UTF-8 medium_dark
rm -rf "$SHOTS"
