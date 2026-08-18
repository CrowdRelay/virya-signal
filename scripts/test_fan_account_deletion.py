from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class FanAccountDeletionUiContract(unittest.TestCase):
    def test_native_command_requires_backend_capability_and_clears_vault(self):
        api = (ROOT / 'src-tauri/src/api/fan.rs').read_text()
        commands = (ROOT / 'src-tauri/src/commands/fan/session_commerce.rs').read_text()
        lib = (ROOT / 'src-tauri/src/lib.rs').read_text()
        self.assertIn('fan_account_deletion_v1', api)
        self.assertIn('Method::DELETE', api)
        self.assertIn('"me/account"', api)
        self.assertIn('state.api.fan_delete_account(&profile).await?', commands)
        self.assertIn('vault::remove_fan', commands)
        self.assertIn('fan_delete_account,', lib)

    def test_ui_uses_two_step_confirmation(self):
        ui = (ROOT / 'src/app/fan/wallet.rs').read_text()
        self.assertIn('delete_confirming', ui)
        self.assertIn('"fan_delete_account"', ui)
        self.assertIn('delete_account_warning', ui)
        self.assertIn('confirm_delete_account', ui)
        self.assertIn('cancel_delete_account', ui)

    def test_translations_are_present_in_both_languages(self):
        for locale in ['pl', 'en']:
            text = (ROOT / f'src/i18n/{locale}.rs').read_text()
            for key in [
                'delete_virya_account',
                'delete_account_warning',
                'confirm_delete_account',
                'cancel_delete_account',
            ]:
                self.assertIn(f'"{key}"', text)

if __name__ == '__main__':
    unittest.main()
