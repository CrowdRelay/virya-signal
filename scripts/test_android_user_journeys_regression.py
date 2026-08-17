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

        # The only pre-camera secret the QR flow needs is the PIN used to
        # encrypt the native vault. Identity comes from the one-time mail token.
        self.assertIn('let scan_pin = pin.get_untracked();', scan)
        self.assertNotIn('email.get_untracked()', scan)
        self.assertNotIn('name.get_untracked()', scan)
        self.assertIn('"fan_prepare_confirmation"', scan)
        self.assertIn('bridge::scan_and_confirm_fan().await', scan)
        self.assertLess(
            scan.index('"fan_prepare_confirmation"'),
            scan.index('bridge::scan_and_confirm_fan().await'),
        )
        self.assertNotIn('bridge::scan_qr().await', scan)
        self.assertNotIn('run_fan_confirmation(values', scan)

        # Successful native confirmation still adopts the authoritative session
        # and routes to Signal using the shared disposal-safe UI path.
        self.assertIn('adopt_fan_session(', scan)
        adopt = src.split('fn adopt_fan_session(', 1)[1].split('\\nasync fn ', 1)[0]
        self.assertIn('session.status.try_set(value)', adopt)
        self.assertIn('persist_fan_tab(FanTab::Signal);', adopt)

        bridge = read('src/bridge/client.rs')
        ffi = read('src/bridge/ffi.rs')
        self.assertIn('option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD")', bridge)
        self.assertIn('pub async fn scan_and_confirm_fan()', bridge)
        self.assertIn("core.invoke('fan_confirm_scanned', { token })", ffi)
    def test_mail_qr_token_is_native_validated_and_does_not_require_email_hint(self):
        validation = read('src-tauri/src/validation.rs')
        self.assertIn('virya-signal://fan/confirm?source=mail&token={token}', validation)
        self.assertIn('https://virya.music/signal/confirm#token={token}', validation)
        self.assertIn('The one-time token is the authentication credential.', validation)
        scanner = read('src/app/fan.rs').split('let scan_confirmation = move |_|', 1)[1].split('    view! {', 1)[0]
        self.assertNotIn('scan_email.is_empty()', scanner)

    def test_fan_confirm_is_single_authoritative_ipc(self):
        ui = read('src/app/fan.rs')
        exchange = ui.split('async fn exchange_fan_confirmation(', 1)[1].split('async fn run_fan_confirmation(', 1)[0]
        self.assertIn('bridge::invoke::<FanSessionStatus, _>(', exchange)
        self.assertNotIn('invoke_timeout::<FanSessionStatus', exchange)

        native = read('src-tauri/src/commands/fan/session_commerce.rs')
        persist = native.split('async fn persist_confirmed_fan(', 1)[1].split('#[tauri::command]', 1)[0]
        confirm = native.split('pub(crate) async fn fan_confirm(', 1)[1].split('#[tauri::command]', 1)[0]
        scanned = native.split('pub(crate) async fn fan_confirm_scanned(', 1)[1].split('#[tauri::command]', 1)[0]

        # Manual-code and scanner entrypoints share one authoritative native
        # transaction. Token exchange, vault write and session publication are
        # not duplicated across the two paths.
        self.assertIn('vault::replace_fan', persist)
        self.assertIn('state.api.fan_confirm(&input).await?', persist)
        self.assertIn('*state.fan_session.write().await = Some(Arc::new(profile));', persist)
        self.assertLess(
            persist.index('vault::replace_fan'),
            persist.index('*state.fan_session.write().await = Some(Arc::new(profile));'),
        )
        self.assertIn(
            'persist_confirmed_fan(&state, input, Zeroizing::new(pin)).await?',
            confirm,
        )
        self.assertIn('persist_confirmed_fan(&state, input, pin).await?', scanned)
        self.assertIn('fan_status(state).await', confirm)
        self.assertIn('fan_status(state).await', scanned)
        self.assertNotIn('vault::replace_fan', confirm)
        self.assertNotIn('vault::replace_fan', scanned)
    def test_lost_confirm_response_still_logs_the_fan_in(self):
        # The mail token is one-time. If fan_confirm succeeded natively but its
        # reply was lost (WebView disposed across the camera transition), the fan
        # must not be stranded on the login screen holding a spent token.
        ui = read('src/app/fan.rs')
        run = ui.split('async fn run_fan_confirmation(', 1)[1].split('\nfn ', 1)[0]
        self.assertIn('Err(message)', run)
        recovery = run.split('Err(message)', 1)[1]
        self.assertIn('fan_status', recovery)
        self.assertIn('status.unlocked', recovery)
        self.assertIn('adopt_fan_session(status', recovery)
        # The error is only surfaced after native state has been consulted.
        self.assertLess(recovery.index('status.unlocked'), recovery.index('error.try_set'))

    def test_confirmation_success_path_is_shared_and_disposal_safe(self):
        ui = read('src/app/fan.rs')
        adopt = ui.split('fn adopt_fan_session(', 1)[1].split('\nasync fn ', 1)[0]
        # Every UI write after a confirmed login is best-effort: a remounted
        # WebView must never turn a successful login into a client-side failure.
        for required in ('try_set', 'persist_fan_tab(FanTab::Signal);', 'status_refresh'):
            self.assertIn(required, adopt)
        self.assertNotIn('.set(', adopt)

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
        self.assertIn('"operator_push_sync"', checklist)
        checklist_sync = checklist.split('"operator_push_sync"', 1)[1].split(');', 1)[0]
        self.assertIn('15_000', checklist_sync)
        self.assertIn('"operator_update_show_checklist", &args, 15_000', checklist)
        self.assertIn('"operator_push_sync"', shell)
        shell_sync = shell.split('"operator_push_sync"', 1)[1].split(');', 1)[0]
        self.assertIn('15_000', shell_sync)

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

    def test_android_e2e_boot_and_crash_gate_are_bounded_and_current(self):
        workflow = read('.github/workflows/android-e2e.yml')
        self.assertIn('-accel on -gpu swiftshader', workflow)
        self.assertNotIn('swiftshader_indirect', workflow)
        self.assertNotIn('-camera-back virtualscene', workflow)
        self.assertIn('timeout 15 adb get-state', workflow)
        self.assertIn('timeout 20 adb logcat -d', workflow)
        self.assertIn(r'ANR in music\.virya\.signal', workflow)
        self.assertNotIn(r'ANR in music\.virya\.control', workflow)

    def test_qr_login_transaction_is_native_owned_after_scanner_result(self):
        native_lib = read('src-tauri/src/lib.rs')
        native_fan = read('src-tauri/src/commands/fan/session_commerce.rs')
        ffi = read('src/bridge/ffi.rs')
        client = read('src/bridge/client.rs')
        fan = read('src/app/fan.rs')

        self.assertIn('pending_fan_confirmation: Mutex<Option<PendingFanConfirmation>>', native_lib)
        self.assertIn('pin: Zeroizing<String>', native_lib)
        self.assertIn('pub(crate) async fn fan_prepare_confirmation(', native_fan)
        self.assertIn('pub(crate) async fn fan_confirm_scanned(', native_fan)
        self.assertIn('if state.fan_session.read().await.is_some()', native_fan)
        self.assertIn("const confirmed = await core.invoke('fan_confirm_scanned', { token });", ffi)
        self.assertIn("viryaWriteFanTab('signal');", ffi)
        self.assertIn("window.dispatchEvent(new Event('virya:resume'));", ffi)
        self.assertIn('pub async fn scan_and_confirm_fan()', client)

        scan = fan.split('let scan_confirmation = move |_| {', 1)[1].split('    view! {', 1)[0]
        self.assertIn('"fan_prepare_confirmation"', scan)
        self.assertIn('bridge::scan_and_confirm_fan().await', scan)
        self.assertNotIn('run_fan_confirmation(values', scan)
        self.assertNotIn('bridge::scan_qr().await', scan)

    def test_qr_remount_can_reuse_native_pin_without_localstorage_secret(self):
        native_fan = read('src-tauri/src/commands/fan/session_commerce.rs')
        ffi = read('src/bridge/ffi.rs')
        self.assertIn('if pin.is_empty()', native_fan)
        self.assertIn('pending.api_base_url == api_base_url', native_fan)
        self.assertIn('pending_fan_confirmation.lock().await = Some(PendingFanConfirmation', native_fan)
        scanner = ffi.split('export async function viryaScanAndConfirmFan()', 1)[1].split('function viryaLocationStates', 1)[0]
        self.assertNotIn('localStorage', scanner)
        self.assertNotIn('viryaStorageWrite', scanner)

    def test_owner_signal_and_event_detail_lifetimes_are_bounded(self):
        support = read('src/app/support.rs')
        shell = read('src/app/fan/shell.rs')
        home = read('src/app/fan_home.rs')
        body = support.split('fn refresh_operator_signal(', 1)[1].split('fn refresh_operator_autopilot(', 1)[0]
        self.assertIn('spawn_lifecycle_task(async move {', body)
        self.assertIn('let _ = loading.try_set(false);', body)
        self.assertIn('focused_event_slug.set(None);', shell)
        self.assertIn('focused_event_preview.set(None);', shell)
        self.assertIn('target != FanTab::Events && target != FanTab::Signal', home)

    def test_android_device_e2e_is_manual_diagnostic_only(self):
        workflow = read('.github/workflows/android-e2e.yml')
        trigger = workflow.split('on:', 1)[1].split('permissions:', 1)[0]
        self.assertIn('workflow_dispatch:', trigger)
        self.assertNotIn('pull_request:', trigger)
        self.assertNotIn('push:', trigger)

    def test_qr_commit_reconciles_after_camera_resume_race(self):
        ffi = read('src/bridge/ffi.rs')
        scanner = ffi.split(
            'export async function viryaScanAndConfirmFan()', 1
        )[1].split('function viryaLocationStates', 1)[0]
        confirm = "const confirmed = await core.invoke('fan_confirm_scanned', { token });"
        self.assertIn(confirm, scanner)
        self.assertIn("viryaWriteFanTab('signal');", scanner)
        self.assertIn("window.dispatchEvent(new Event('virya:resume'));", scanner)
        self.assertIn("return confirmed;", scanner)
        self.assertLess(scanner.index(confirm), scanner.index("viryaWriteFanTab('signal');"))
        self.assertLess(
            scanner.index("viryaWriteFanTab('signal');"),
            scanner.index("window.dispatchEvent(new Event('virya:resume'));"),
        )
        self.assertLess(
            scanner.index("window.dispatchEvent(new Event('virya:resume'));"),
            scanner.index("return confirmed;"),
        )

        app = read('src/app.rs')
        self.assertIn("latest_request_completed(&result)", app)
        self.assertIn("Ok(None) => {}", app)

    def test_push_lifecycle_tasks_have_terminal_resume_replay(self):
        app = read('src/app.rs')
        fan = read('src/app/fan_home.rs')
        staff = read('src/app/operator/checklist.rs')
        self.assertIn("fn spawn_lifecycle_task(", app)
        self.assertIn("fn finish_resumable_ui_task(", app)
        self.assertIn("resume_pending", fan)
        self.assertIn("busy.get_untracked()", fan)
        self.assertIn("spawn_lifecycle_task", fan)
        self.assertIn("finish_resumable_ui_task", fan)
        self.assertIn("push_resume_pending", staff)
        self.assertIn("spawn_lifecycle_task", staff)
        self.assertGreaterEqual(staff.count("finish_resumable_ui_task"), 3)

if __name__ == '__main__':
    unittest.main()
