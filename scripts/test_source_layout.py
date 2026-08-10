import unittest
from pathlib import Path
from source_tree import read_app_source
ROOT = Path(__file__).resolve().parents[1]
class SourceLayoutContracts(unittest.TestCase):
    def test_modern_rust_module_layout(self):
        self.assertFalse(any((ROOT / 'src').rglob('mod.rs')))
        self.assertFalse(any((ROOT / 'src-tauri/src').rglob('mod.rs')))
        for relative in ('src/app.rs','src/i18n.rs','src-tauri/src/api.rs','src-tauri/src/commands.rs','src-tauri/src/i18n.rs'):
            self.assertTrue((ROOT / relative).is_file(), relative)
    def test_app_sections_remain_one_contract(self):
        entry=(ROOT/'src/app.rs').read_text()
        self.assertEqual(entry.count('include!("app/'),5)
        for module in ('operator.rs','scanner.rs','fan_home.rs','fan.rs','support.rs'):
            self.assertIn(f'include!("app/{module}");', entry)
        source=read_app_source(ROOT)
        for contract in ('pub fn App()', 'fn OperatorApp(', 'fn FanApp(', 'fn refresh_fan_parts('):
            self.assertIn(contract, source)
if __name__=='__main__': unittest.main()
