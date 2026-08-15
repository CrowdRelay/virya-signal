from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")

class StaffGigChecklistContracts(unittest.TestCase):
    def test_checklist_is_a_first_class_operator_tab(self):
        shell = read("src/app/operator/shell.rs")
        checklist = read("src/app/operator/checklist.rs")
        self.assertIn("OperatorTab::Checklist", shell)
        self.assertIn('operator_show_checklist', checklist)
        self.assertIn('operator_update_show_checklist', checklist)
        self.assertIn('team_push_notifications_hint', checklist)

    def test_staff_push_deep_link_never_bypasses_staff_gate(self):
        app = read("src/app.rs")
        self.assertIn('target.starts_with("/staff/")', app)
        self.assertIn('operator_push_target.set(Some(target));\n                        mode.set(RootMode::StaffGate);', app)
        self.assertNotIn('operator_push_target.set(Some(target));\n                        mode.set(RootMode::Team);', app)

    def test_native_staff_push_uses_authenticated_staff_endpoint(self):
        api = read("src-tauri/src/api/operator.rs")
        commands = read("src-tauri/src/commands/operator.rs")
        self.assertIn('"staff/push/endpoints"', api)
        self.assertIn('operator_register_android_push', api)
        self.assertIn('operator_push_sync', commands)
        self.assertIn('operator_push_enable', commands)

    def test_requested_bare_minimum_is_present_in_polish_catalog(self):
        catalog = read("src/i18n/pl.rs")
        for phrase in (
            "Podładować i wziąć laptop",
            "Przygotować i sprawdzić setlistę",
            "Spakować merch",
            "Wziąć rack, swoje kable i instrumenty",
            "Dać Madzi",
            "Wziąć strój koncertowy",
            "Sprawdzić systemy bezprzewodowe",
            "7 i 2 dni przed koncertem",
        ):
            self.assertIn(phrase, catalog)

if __name__ == "__main__":
    unittest.main()
