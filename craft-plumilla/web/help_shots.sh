#!/bin/sh
# Draw the Help's screenshots (docs/help/img) from the app itself, in
# English, headlessly: every dialog, options page and panel the Help
# shows, on a small demo drawing (src/help_shots.rs). All languages use
# these images. Run from the repository root.
set -e
out="${1:-docs/help/img}"
mkdir -p "$out"
HELP_SHOTS="$(cd "$out" && pwd)" cargo test -p tracedraw --bin tracedraw help_shots -- --ignored --nocapture
rm -f "$out/shots.txt"
# Smaller files when a PNG optimizer is installed.
if command -v oxipng >/dev/null 2>&1; then
  oxipng -q -o 4 --strip safe "$out"/*.png
fi
