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
echo "modes: fan-out | fan-locked | fan | beacon | staff | owner   (add &link=1 for a pending confirmation link)"
# No-store on everything. The browser cached the stub, the wasm and the runtime
# i18n catalogs across restarts, so a rebuilt fix could appear not to have
# worked — which is worse than no preview at all, because it looks like a bug
# in the app. A preview server has no reason to cache.
exec python3 - "$PORT" "$SERVE" <<'PY'
import functools, http.server, socketserver, sys


class NoStore(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, max-age=0")
        super().end_headers()


socketserver.TCPServer.allow_reuse_address = True
handler = functools.partial(NoStore, directory=sys.argv[2])
with socketserver.TCPServer(("127.0.0.1", int(sys.argv[1])), handler) as httpd:
    httpd.serve_forever()
PY
