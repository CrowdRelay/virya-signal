import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class BootContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.index = (ROOT / "index.html").read_text()
        cls.boot = (ROOT / "boot.js").read_text()
        cls.initializer = (ROOT / "boot-initializer.mjs").read_text()
        cls.main = (ROOT / "src/main.rs").read_text()

    def test_listener_and_initializer_precede_wasm(self):
        boot_tag = '<script src="boot.js?v=0.4.2-startup-v6"></script>'
        self.assertIn(boot_tag, self.index)
        self.assertNotIn('<script src="boot.js" defer>', self.index)
        self.assertLess(self.index.index(boot_tag), self.index.index('data-trunk rel="rust"'))
        self.assertIn('data-initializer="boot-initializer.mjs"', self.index)

    def test_ready_state_survives_a_missed_event(self):
        self.assertIn('data-virya-ready', self.boot)
        self.assertIn('document.querySelector(".app-shell .launcher")', self.boot)
        self.assertIn('MutationObserver', self.boot)

    def test_boot_failure_is_visible_and_retry_is_bounded(self):
        self.assertIn('START APLIKACJI ZATRZYMANY', self.boot)
        self.assertIn('unhandledrejection', self.boot)
        self.assertIn('retry-blocked', self.boot)
        self.assertIn('sessionStorage', self.boot)
        self.assertIn('boot()?.fail?.(error)', self.initializer)

    def test_boot_phases_are_observable_without_user_data(self):
        self.assertIn('[virya:boot]', self.boot)
        for phase in ['wasm-loading', 'wasm-entered', 'wasm-initialized']:
            self.assertIn(phase, self.boot + self.initializer + self.main)

    def test_rust_startup_has_no_inline_js_dependency(self):
        self.assertNotIn('#[wasm_bindgen(inline_js', self.main)
        self.assertIn('js_sys::Reflect', self.main)
        self.assertLess(self.main.index('mount_to_body'), self.main.rindex('virya_app_mounted();'))
        self.assertIn('"data-virya-ready"', self.main)


if __name__ == "__main__":
    unittest.main()
