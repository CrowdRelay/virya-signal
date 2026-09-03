#!/usr/bin/env bash
# Serves the built app with a fake Tauri bridge so the real screens render in a
# browser. Requires a `trunk build` first — this never builds, so what you look
# at is exactly what was shipped.
set -Eeuo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
PORT="${VIRYA_PREVIEW_PORT:-4181}"
SERVE="$ROOT/scripts/ui-preview/serve"

[[ -f "$ROOT/dist/index.html" ]] || { echo "no dist/ — run: trunk build" >&2; exit 1; }

rm -rf "$SERVE"
cp -R "$ROOT/dist" "$SERVE"
cp "$ROOT/scripts/ui-preview/bridge-stub.js" "$SERVE/bridge-stub.js"

# The stub must define window.__TAURI__ before the WASM module boots, and the
# built index is minified with no <head> tag to anchor on.
python3 - "$SERVE/index.html" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
anchor = "<meta charset=utf-8>"
if anchor not in s:
    raise SystemExit("index.html shape changed; the stub has nowhere to go")
p.write_text(s.replace(anchor, anchor + '<script src="/bridge-stub.js"></script>', 1))
PY

echo "http://127.0.0.1:$PORT/index.html?mode=fan"
echo "modes: fan-out | fan | beacon | staff | owner"
exec python3 -m http.server "$PORT" --directory "$SERVE"
