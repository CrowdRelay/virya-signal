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

for needle in ("fan_push_status", "fan_push_enable", "fan_push_disable", "sync_native_push_if_desired", "NATIVE_PUSH_INSTALLATION_FILE"):
    require(fan, needle, needle)
for needle in ("virya_signal_contracts::push::*", "virya_signal_contracts::push"):
    require(models, needle, needle)
for needle in ("push.pending", "push.processing", "push.dead", "push.delivered_24h"):
    require(operator, needle, needle)
require(contracts, "pub push: QueueSummary", "ops-push-summary")
require(push_plugin, "SignalPushPlugin", "android-fcm-plugin")
for needle in ("FirebaseMessaging.getInstance().token", "Manifest.permission.POST_NOTIFICATIONS"):
    require(kotlin_plugin, needle, f"kotlin-{needle}")
require(kotlin_service, "FirebaseMessagingService", "firebase-message-service")
for needle in ("SignalPushPlugin.kt", "ViryaFirebaseMessagingService.kt", "android.permission.POST_NOTIFICATIONS"):
    require(android_prepare, needle, f"android-stage-{needle}")
print("SIGNAL_PUSH_CONTROL=PASS native=permission+token+install-id+fcm-service operator=queue-health contract=ops-summary")
