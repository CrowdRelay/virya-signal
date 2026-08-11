from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class RumContract(unittest.TestCase):
    def test_native_rum_is_sampled_bounded_and_identity_free(self):
        client = (ROOT / "src-tauri/src/api/client.rs").read_text()
        shared = (ROOT / "crates/virya-signal-contracts/src/autopilot.rs").read_text()
        operator = (ROOT / "src/app/operator.rs").read_text()
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
        self.assertIn('pub struct RumMetricSummary', shared)
        self.assertIn('autopilot_rum_24h', operator)
        self.assertIn('Real-user performance · 24h', (ROOT / 'src/i18n/en.rs').read_text())

if __name__ == '__main__':
    unittest.main()
