import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class BridgeLifecycleContracts(unittest.TestCase):
    def test_latest_invocation_registry_is_bounded(self):
        bridge = (ROOT / "src/bridge.rs").read_text()
        latest = bridge.split("export async function viryaInvokeLatest", 1)[1]
        latest = latest.split("function viryaPermissionState", 1)[0]
        self.assertIn("finally", latest)
        self.assertIn("latestInvocations.delete(scope)", latest)
        self.assertNotIn("latestInvocations.set(scope, ++invocationSequence)", latest)


if __name__ == "__main__":
    unittest.main()
