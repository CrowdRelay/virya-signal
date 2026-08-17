import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text()

class AndroidUserJourneysRegression(unittest.TestCase):
    def test_qr_scan_uses_pre_camera_immutable_credentials(self):
        src = read('src/app/fan.rs')
        start = src.index('let scan_confirmation = move |_|')
        end = src.index('    view! {', start)
        scan = src[start:end]
        self.assertIn('let scan_email = email.get_untracked().trim().to_owned();', scan)
        self.assertIn('let scan_name = optional(name.get_untracked().trim().to_owned());', scan)
        self.assertIn('let scan_pin = pin.get_untracked();', scan)
        self.assertLess(scan.index('let scan_email'), scan.index('bridge::scan_qr().await'))
        self.assertLess(scan.index('let scan_pin'), scan.index('bridge::scan_qr().await'))
        self.assertIn('submit_fan_confirmation_values(', scan)
        post_scan = scan[scan.index('bridge::scan_qr().await'):]
        self.assertNotIn('email.get_untracked()', post_scan)
        self.assertNotIn('pin.get_untracked()', post_scan)
        bridge = read('src/bridge/client.rs')
        self.assertIn('option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD")', bridge)

    def test_fan_confirmation_helper_stays_within_argument_budget(self):
        src = read('src/app/fan.rs')
        self.assertIn('struct FanConfirmationValues', src)
        signature = src.split('fn submit_fan_confirmation_values(', 1)[1].split(') {', 1)[0]
        top_level = [line for line in signature.splitlines() if line.strip().endswith(',')]
        self.assertLessEqual(len(top_level), 7)

    def test_fan_tab_survives_android_settings_resume(self):
        shell = read('src/app/fan/shell.rs')
        bridge = read('src/bridge/client.rs')
        self.assertIn('let tab = RwSignal::new(persisted_fan_tab());', shell)
        self.assertIn('persist_fan_tab(tab.get());', shell)
        self.assertIn('pub fn fan_tab_state()', bridge)
        self.assertIn('pub fn set_fan_tab_state(value: &str)', bridge)

    def test_next_signal_has_deterministic_event_details_route(self):
        home = read('src/app/fan_home.rs')
        self.assertIn('focused_event_preview.set(Some(event_preview.clone()));', home)
        self.assertIn('focused_event_slug.set(Some(event_slug.clone()));', home)
        self.assertIn('tab.set(FanTab::Events);', home)
        self.assertIn('tr("show_details")', home)

    def test_owner_signal_uses_encrypted_last_known_good_cache(self):
        command = read('src-tauri/src/commands/operator.rs')
        session = read('src-tauri/src/session.rs')
        vault = read('src-tauri/src/vault.rs')
        self.assertIn('persist_operator_signal_cache', command)
        self.assertIn('load_operator_signal_cache', command)
        self.assertIn('offline_cache', command)
        self.assertIn('operator_signal_cache_fallback_allowed', command)
        self.assertIn('AppError::Network(_)', command)
        self.assertRegex(
            command,
            r'AppError::Remote\s*\{\s*status:\s*500\.\.=599,\s*\.\.\s*\}',
        )
        self.assertIn('save_operator_signal_cache_with_password', session)
        self.assertIn('load_operator_signal_cache_with_password', session)
        self.assertIn('OPERATOR_SIGNAL_CACHE_KEY', vault)

    def test_operator_settings_does_not_sync_while_webview_backgrounds(self):
        native = read('src-tauri/src/commands/operator.rs')
        body = native.split('pub(crate) async fn operator_push_open_settings', 1)[1].split('#[tauri::command]', 1)[0]
        self.assertIn('open_native_push_settings', body)
        self.assertIn('current_native_push_status', body)
        self.assertNotIn('sync_operator_push', body)

    def test_staff_sync_actions_have_terminal_timeout(self):
        checklist = read('src/app/operator/checklist.rs')
        shell = read('src/app/operator/shell.rs')
        self.assertIn('"operator_push_sync", &EmptyArgs {}, 15_000', checklist)
        self.assertIn('"operator_update_show_checklist", &args, 15_000', checklist)
        self.assertIn('"operator_push_sync", &EmptyArgs {}, 15_000', shell)

    def test_autopilot_policy_rekeys_on_version_and_mutation_has_terminal_timeout(self):
        src = read('src/app/operator/autopilot.rs')
        self.assertIn('key=|policy| format!("{}:{}", policy.context, policy.version)', src)
        self.assertIn('bridge::invoke_timeout::<AutopilotMutation, _>(', src)
        mutation = src.split('fn set_autopilot_policy(', 1)[1].split('fn assign_autopilot_action(', 1)[0]
        self.assertIn('"operator_autopilot_set_authority"', mutation)
        self.assertIn('15_000', mutation)
        policy_card = src.split('fn AutopilotPolicyCard(', 1)[1].split('include!("autopilot_cards.rs")', 1)[0]
        self.assertEqual(policy_card.count('type="button"'), 5)

    def test_android_e2e_mock_reads_each_post_body_once(self):
        mock = read('scripts/e2e/mock_signal_api.py')
        post = mock.split('    def do_POST(self)', 1)[1].split('    def do_PATCH(self)', 1)[0]
        self.assertEqual(post.count('self._body()'), 1)
        self.assertIn('body = self._body()', post)

    def test_android_e2e_exercises_autopilot_mutation(self):
        journey = read('scripts/e2e/android_journeys.py')
        mock = read('scripts/e2e/mock_signal_api.py')
        self.assertIn('def owner_autopilot_mode_changes', journey)
        self.assertIn('owner_autopilot_mode_changes(d)', journey)
        self.assertIn('/v1/admin/autopilot/overview', mock)
        self.assertIn('/v1/admin/autopilot/policies/', mock)
        self.assertIn('def fan_recovery_qr', journey)
        self.assertIn('d.tap(["SKANUJ QR", "SCAN QR"]', journey)

if __name__ == '__main__':
    unittest.main()
