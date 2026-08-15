#!/usr/bin/env python3
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
INCLUDE = re.compile(r'include!\("([^"]+)"\);')

def module(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    parts = [text]
    for rel in INCLUDE.findall(text):
        parts.append(module(path.parent / rel))
    return "\n".join(parts)

def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"SIGNAL_PUSH_CONTROL=FAIL missing={label}")

fan = module(ROOT / "src-tauri/src/commands/fan.rs")
models = module(ROOT / "src-tauri/src/models.rs")
operator = module(ROOT / "src/app/operator.rs")
contracts = (ROOT / "crates/virya-signal-contracts/src/ops.rs").read_text(encoding="utf-8")
push_plugin = (ROOT / "src-tauri/src/push_plugin.rs").read_text(encoding="utf-8")
kotlin_plugin = (ROOT / "src-tauri/android-push/SignalPushPlugin.kt").read_text(encoding="utf-8")
kotlin_service = (ROOT / "src-tauri/android-push/ViryaFirebaseMessagingService.kt").read_text(encoding="utf-8")
android_prepare = (ROOT / "scripts/prepare-android.py").read_text(encoding="utf-8")
fan_home = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")

for needle in ("fan_push_status", "fan_push_enable", "fan_push_disable", "fan_push_open_settings", "fan_push_take_target", "sync_native_push_if_desired", "NATIVE_PUSH_INSTALLATION_FILE"):
    require(fan, needle, needle)
for needle in ("virya_signal_contracts::push::*", "virya_signal_contracts::push"):
    require(models, needle, needle)
for needle in ("push.pending", "push.processing", "push.dead", "push.delivered_24h"):
    require(operator, needle, needle)
require(contracts, "pub push: QueueSummary", "ops-push-summary")
require(push_plugin, "SignalPushPlugin", "android-fcm-plugin")
for needle in (
    "FirebaseMessaging.getInstance().token",
    "Manifest.permission.POST_NOTIFICATIONS",
    "getNotificationPermissionState",
    "requestNotificationPermission",
    "openNotificationSettings",
    "takeLaunchTarget",
    "override fun onNewIntent",
    "NotificationManagerCompat",
):
    require(kotlin_plugin, needle, f"kotlin-{needle}")
for forbidden in ("override fun checkPermissions", "override fun requestPermissions"):
    if forbidden in kotlin_plugin:
        raise SystemExit(f"SIGNAL_PUSH_CONTROL=FAIL inherited-command-collision={forbidden}")
require(push_plugin, '"getNotificationPermissionState"', "rust-native-permission-command")
require(push_plugin, '"requestNotificationPermission"', "rust-native-request-command")
require(push_plugin, '"openNotificationSettings"', "rust-native-settings-command")
require(push_plugin, '"takeLaunchTarget"', "rust-native-launch-target-command")
require(kotlin_service, "FirebaseMessagingService", "firebase-message-service")
require(kotlin_service, 'getIdentifier("virya_signal_notification", "drawable", packageName)', "notification-monochrome-icon")
require(kotlin_service, 'putExtra("virya_push_target_path", targetPath)', "notification-launch-target")
require(fan_home, "enable_after_settings", "push-settings-resume-intent")
require(fan_home, 'bridge::invoke::<FanPushStatus, _>("fan_push_enable"', "push-settings-auto-enable")
require(fan_home, "busy.get()", "push-resume-race-guard")
for needle in ("SignalPushPlugin.kt", "ViryaFirebaseMessagingService.kt", "virya_signal_notification.xml", "android.permission.POST_NOTIFICATIONS"):
    require(android_prepare, needle, f"android-stage-{needle}")
print("SIGNAL_PUSH_CONTROL=PASS native=permission+token+deep-link+install-id+fcm-service operator=queue-health contract=ops-summary")
