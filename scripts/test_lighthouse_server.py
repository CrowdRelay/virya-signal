import gzip
import threading
import unittest
import urllib.request
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from tempfile import TemporaryDirectory

from lighthouse_server import LighthouseRequestHandler


class LighthouseServerTests(unittest.TestCase):
    def test_serves_wasm_compressed_with_long_lived_asset_cache(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            source = b"wasm-payload-" * 4096
            (root / "app-0123456789abcdef.wasm").write_bytes(source)
            (root / "index.html").write_text("<!doctype html><title>Signal</title>")
            handler = partial(LighthouseRequestHandler, directory=directory)
            server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                port = server.server_address[1]
                request = urllib.request.Request(
                    f"http://127.0.0.1:{port}/app-0123456789abcdef.wasm",
                    headers={"Accept-Encoding": "gzip"},
                )
                with urllib.request.urlopen(request) as response:
                    self.assertEqual(response.headers["Content-Encoding"], "gzip")
                    self.assertIn("immutable", response.headers["Cache-Control"])
                    self.assertEqual(gzip.decompress(response.read()), source)

                with urllib.request.urlopen(f"http://127.0.0.1:{port}/") as response:
                    self.assertEqual(response.headers["Cache-Control"], "no-cache")
            finally:
                server.shutdown()
                server.server_close()
                thread.join(timeout=2)


if __name__ == "__main__":
    unittest.main()
