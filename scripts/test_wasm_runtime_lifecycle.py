from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class WasmRuntimeLifecycleTests(unittest.TestCase):
    def test_ui_async_tasks_are_owner_scoped_and_cancelled(self):
        app = (ROOT / "src" / "app.rs").read_text()
        area = (ROOT / "src" / "app" / "area.rs").read_text()
        util = (ROOT / "src" / "util.rs").read_text()
        # The scoped spawner is reached through one type-erasing wrapper so the
        # cancellation plumbing is monomorphized once instead of per call site.
        self.assertIn("leptos::task::spawn_local_scoped_with_cancellation(future)", util)
        self.assertIn("Pin<Box<dyn std::future::Future<Output = ()>>>", util)
        self.assertIn("use crate::util::spawn_local;", app)
        self.assertIn("use crate::util::spawn_local;", area)
        self.assertNotIn("spawn_local_scoped_with_cancellation", app)
        self.assertNotIn("spawn_local_scoped_with_cancellation", area)
        self.assertIn("fn spawn_lifecycle_task(", app)
        self.assertIn("wasm_bindgen_futures::spawn_local(future);", app)
        for path in (ROOT / "src").rglob("*.rs"):
            text = path.read_text()
            if path.as_posix().endswith("src/app.rs") or path.as_posix().endswith("src/app/area.rs"):
                continue
            self.assertNotIn("wasm_bindgen_futures::spawn_local", text, path)

    def test_long_lived_callbacks_are_disposal_safe(self):
        app = (ROOT / "src" / "app.rs").read_text()
        support = (ROOT / "src" / "app" / "support.rs").read_text()
        self.assertIn("signal.try_update", app)
        self.assertIn("unregister_resume_refresh", app)
        self.assertIn(".retain(|(id, _, _)| *id != subscriber_id)", app)
        self.assertIn("dismiss_generation.try_get_untracked()", support)
        self.assertIn("error.try_set(None)", support)

if __name__ == "__main__":
    unittest.main()
