import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class BootContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.index = (ROOT / "index.html").read_text()
        cls.boot = (ROOT / "boot.js").read_text()
        cls.main = (ROOT / "src/main.rs").read_text()

    def test_listener_precedes_wasm_without_defer(self):
        boot_tag = '<script src="boot.js?v=0.4.2"></script>'
        self.assertIn(boot_tag, self.index)
        self.assertNotIn('<script src="boot.js" defer>', self.index)
        self.assertLess(self.index.index(boot_tag), self.index.index('data-trunk rel="rust"'))

    def test_ready_state_survives_a_missed_event(self):
        self.assertIn('data-virya-ready', self.boot)
        self.assertIn('document.querySelector(".app-shell .launcher")', self.boot)
        self.assertIn('MutationObserver', self.boot)

    def test_slow_boot_is_not_declared_fatal(self):
        self.assertIn('SLOW_BOOT_MS = 8_000', self.boot)
        self.assertIn('RECOVERY_MS = 30_000', self.boot)
        self.assertNotIn('URUCHOM APLIKACJĘ PONOWNIE', self.boot)

    def test_boot_phases_are_observable_without_user_data(self):
        self.assertIn('[virya:boot]', self.boot)
        self.assertIn('trace("ready")', self.boot)

    def test_rust_marks_ready_after_mount(self):
        self.assertLess(self.main.index('mount_to_body'), self.main.rindex('virya_app_mounted();'))
        self.assertIn("setAttribute('data-virya-ready', 'true')", self.main)


if __name__ == "__main__":
    unittest.main()
