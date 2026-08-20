#!/usr/bin/env python3
"""Serve a release bundle with CDN-like compression for local Lighthouse.

The production edge compresses text and WASM transfers. Python's stock static
server does not, which makes Lighthouse's simulated mobile graph charge the raw
2 MiB WASM payload and produces a measurement unrelated to the shipped network
path. This server keeps the audit local while modelling that transport contract.
"""
from __future__ import annotations

import argparse
import gzip
import io
import re
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

COMPRESSIBLE_SUFFIXES = {".css", ".html", ".js", ".json", ".svg", ".wasm"}
HASHED_ASSET = re.compile(r"-[0-9a-f]{16,}\.")


class LighthouseRequestHandler(SimpleHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm"}

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def end_headers(self) -> None:
        parsed = urlsplit(self.path)
        name = Path(parsed.path).name
        if parsed.path in {"", "/", "/index.html"}:
            self.send_header("Cache-Control", "no-cache")
        elif parsed.query or HASHED_ASSET.search(name):
            self.send_header("Cache-Control", "public, max-age=31536000, immutable")
        else:
            self.send_header("Cache-Control", "public, max-age=3600")
        super().end_headers()

    def send_head(self):  # type: ignore[no-untyped-def]
        path = Path(self.translate_path(self.path))
        accepts_gzip = "gzip" in self.headers.get("Accept-Encoding", "").lower()
        if path.is_file() and accepts_gzip and path.suffix.lower() in COMPRESSIBLE_SUFFIXES:
            try:
                source = path.read_bytes()
                payload = gzip.compress(source, compresslevel=9, mtime=0)
                stat = path.stat()
            except OSError:
                self.send_error(404, "File not found")
                return None
            self.send_response(200)
            self.send_header("Content-Type", self.guess_type(str(path)))
            self.send_header("Content-Encoding", "gzip")
            self.send_header("Vary", "Accept-Encoding")
            self.send_header("Content-Length", str(len(payload)))
            self.send_header("Last-Modified", self.date_time_string(stat.st_mtime))
            self.end_headers()
            return io.BytesIO(payload)
        return super().send_head()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--bind", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=4173)
    args = parser.parse_args()
    if not args.directory.is_dir():
        parser.error(f"release directory does not exist: {args.directory}")

    handler = partial(LighthouseRequestHandler, directory=str(args.directory.resolve()))
    server = ThreadingHTTPServer((args.bind, args.port), handler)
    print(
        f"SIGNAL_LIGHTHOUSE_SERVER=READY bind={args.bind}:{args.port} "
        "compression=gzip cache=cdn-like",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
