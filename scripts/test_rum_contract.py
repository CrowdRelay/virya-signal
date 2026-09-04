from rust_source_tree import read_rust_module
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class RumContract(unittest.TestCase):
    def test_native_rum_is_sampled_bounded_and_identity_free(self):
        client = (ROOT / "src-tauri/src/api/client.rs").read_text()
        self.assertIn("Uuid::new_v4().as_bytes()[0] < 13", client)
        self.assertIn("rum_sampled: should_sample_rum()", client)
        self.assertIn("if !self.rum_sampled", client)
        self.assertIn("self.started_at.elapsed()", client)
        self.assertIn('"api_latency_ms"', client)
        self.assertIn('"cold_start_ms"', client)
        self.assertIn('"public/telemetry/rum"', client)
        self.assertIn('"device_class": "native"', client)
        for forbidden in ('user_id', 'email', 'fingerprint', 'session_id'):
            self.assertNotIn(forbidden, client[client.index('fn should_sample_rum'):client.index('pub async fn exchange_staff_pairing')])

if __name__ == '__main__':
    unittest.main()
