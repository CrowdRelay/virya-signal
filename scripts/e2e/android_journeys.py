#!/usr/bin/env python3
"""Black-box Android journeys for Virya Signal.

Uses only adb + Android's UIAutomator accessibility tree. The test therefore crosses the
real Tauri WebView/native Activity boundary instead of calling Rust commands directly.
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path

PKG = "music.virya.signal"
FAN_EMAIL = "e2e@virya.music"
PIN = "2580"
OWNER_TOKEN = "e2e_owner_token_" + "c" * 48


class JourneyError(RuntimeError):
    pass


def adb(*args: str, check: bool = True, timeout: int = 30) -> str:
    proc = subprocess.run(["adb", *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout)
    if check and proc.returncode:
        raise JourneyError(f"adb {' '.join(args)} failed ({proc.returncode}):\n{proc.stdout}")
    return proc.stdout.strip()


def shell(*args: str, check: bool = True, timeout: int = 30) -> str:
    return adb("shell", *args, check=check, timeout=timeout)


@dataclass
class Node:
    attrs: dict[str, str]

    @property
    def text(self) -> str:
        return self.attrs.get("text", "")

    @property
    def desc(self) -> str:
        return self.attrs.get("content-desc", "")

    @property
    def cls(self) -> str:
        return self.attrs.get("class", "")

    @property
    def bounds(self) -> tuple[int, int, int, int]:
        m = re.fullmatch(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", self.attrs.get("bounds", ""))
        if not m:
            raise JourneyError(f"node has no bounds: {self.attrs}")
        return tuple(map(int, m.groups()))  # type: ignore[return-value]

    def haystack(self) -> str:
        return " | ".join((self.text, self.desc, self.attrs.get("resource-id", ""), self.attrs.get("hint", ""))).casefold()


class Device:
    def __init__(self, artifacts: Path):
        self.artifacts = artifacts
        self.artifacts.mkdir(parents=True, exist_ok=True)
        self.step = 0

    def dump(self) -> list[Node]:
        remote = "/sdcard/window.xml"
        shell("uiautomator", "dump", "--compressed", remote, check=False, timeout=15)
        xml = adb("exec-out", "cat", remote, timeout=15)
        if "<hierarchy" not in xml:
            raise JourneyError(f"UIAutomator did not return hierarchy:\n{xml[:1000]}")
        try:
            root = ET.fromstring(xml)
        except ET.ParseError as exc:
            raise JourneyError(f"invalid UI XML: {exc}\n{xml[:1200]}") from exc
        return [Node(dict(el.attrib)) for el in root.iter("node")]

    def snapshot(self, name: str) -> None:
        self.step += 1
        stem = f"{self.step:02d}-{name}"
        (self.artifacts / f"{stem}.xml").write_text(adb("exec-out", "uiautomator", "dump", "/dev/tty", check=False, timeout=15), errors="ignore")
        with (self.artifacts / f"{stem}.png").open("wb") as f:
            proc = subprocess.run(["adb", "exec-out", "screencap", "-p"], stdout=f, stderr=subprocess.PIPE, timeout=20)
            if proc.returncode:
                print(proc.stderr.decode(errors="ignore"), file=sys.stderr)

    def find(self, labels: list[str], *, timeout: float = 15, cls_contains: str | None = None) -> Node:
        deadline = time.monotonic() + timeout
        wanted = [label.casefold() for label in labels]
        last: list[Node] = []
        while time.monotonic() < deadline:
            last = self.dump()
            for node in last:
                if cls_contains and cls_contains.casefold() not in node.cls.casefold():
                    continue
                hay = node.haystack()
                if any(label in hay for label in wanted):
                    return node
            time.sleep(0.5)
        sample = "\n".join(f"{n.cls}: text={n.text!r} desc={n.desc!r}" for n in last if n.text or n.desc)
        raise JourneyError(f"could not find any of {labels!r} in UI after {timeout}s. Visible:\n{sample[-6000:]}")

    def find_exact_text(self, label: str, *, timeout: float = 15) -> Node:
        deadline = time.monotonic() + timeout
        wanted = label.strip().casefold()
        last: list[Node] = []
        while time.monotonic() < deadline:
            last = self.dump()
            for node in last:
                if node.text.strip().casefold() == wanted:
                    return node
            time.sleep(0.5)
        sample = "\n".join(f"{n.cls}: text={n.text!r} desc={n.desc!r}" for n in last if n.text or n.desc)
        raise JourneyError(f"could not find exact text {label!r} in UI after {timeout}s. Visible:\n{sample[-6000:]}")

    def exists(self, labels: list[str], timeout: float = 2) -> bool:
        try:
            self.find(labels, timeout=timeout)
            return True
        except JourneyError:
            return False

    def tap_node(self, node: Node) -> None:
        x1, y1, x2, y2 = node.bounds
        shell("input", "tap", str((x1+x2)//2), str((y1+y2)//2))
        time.sleep(0.35)

    def tap(self, labels: list[str], *, timeout: float = 15) -> None:
        self.tap_node(self.find(labels, timeout=timeout))

    def tap_exact_text(self, label: str, *, timeout: float = 15) -> None:
        self.tap_node(self.find_exact_text(label, timeout=timeout))

    def input(self, labels: list[str], value: str, *, timeout: float = 15) -> None:
        node = self.find(labels, timeout=timeout, cls_contains="EditText")
        self.tap_node(node)
        shell("input", "keyevent", "KEYCODE_MOVE_END", check=False)
        shell("input", "text", value)
        time.sleep(0.25)

    def clear_and_input(self, labels: list[str], value: str, *, timeout: float = 15) -> None:
        node = self.find(labels, timeout=timeout, cls_contains="EditText")
        self.tap_node(node)
        shell("input", "keyevent", "KEYCODE_CTRL_A", check=False)
        shell("input", "keyevent", "KEYCODE_DEL", check=False)
        shell("input", "text", value)
        time.sleep(0.25)

    def assert_text(self, labels: list[str], *, timeout: float = 15) -> None:
        self.find(labels, timeout=timeout)

    def assert_absent(self, labels: list[str], *, seconds: float = 4) -> None:
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            if self.exists(labels, timeout=0.6):
                time.sleep(0.4)
            else:
                return
        raise JourneyError(f"unexpected UI state persists: {labels}")


def launch_clean() -> None:
    shell("am", "force-stop", PKG, check=False)
    shell("pm", "clear", PKG)
    # `monkey` resolves the launcher activity without coupling the test to generated Tauri Java names.
    shell("monkey", "-p", PKG, "-c", "android.intent.category.LAUNCHER", "1", timeout=20)


def relaunch() -> None:
    shell("am", "force-stop", PKG, check=False)
    shell("monkey", "-p", PKG, "-c", "android.intent.category.LAUNCHER", "1", timeout=20)


def fan_first_login(d: Device) -> None:
    d.assert_text(["VIRYA SIGNAL"], timeout=30)
    # Exercise the actual Scan QR UI path. The E2E debug build replaces only
    # the camera payload with the exact mail deep-link; auth/session/vault/UI
    # still cross the normal Tauri commands.
    if d.exists(["MAM KOD", "I HAVE A CODE"], timeout=3):
        d.tap(["MAM KOD", "I HAVE A CODE"])
    d.input(["E-mail", "Email"], FAN_EMAIL)
    d.input(["Utwórz PIN odblokowania fana", "Create fan unlock PIN"], PIN)
    d.snapshot("fan-confirm-ready")
    d.tap(["SKANUJ QR", "SCAN QR"], timeout=10)
    d.assert_text(["TWÓJ SYGNAŁ TERAZ", "YOUR SIGNAL NOW"], timeout=25)
    d.snapshot("fan-first-login-qr")


def fan_recovery_qr(d: Device) -> None:
    relaunch()
    d.assert_text(["VIRYA SIGNAL"], timeout=25)
    d.tap(["NIE PAMIĘTAM PIN-U", "I FORGOT MY PIN", "SIGN IN AGAIN"], timeout=10)
    d.input(["E-mail", "Email"], FAN_EMAIL)
    d.input(["Utwórz PIN odblokowania fana", "Create fan unlock PIN"], "3690")
    d.snapshot("fan-recovery-ready")
    d.tap(["SKANUJ QR", "SCAN QR"], timeout=10)
    d.assert_text(["TWÓJ SYGNAŁ TERAZ", "YOUR SIGNAL NOW"], timeout=25)
    d.snapshot("fan-recovery-qr")


def synesthesia_app_link_round_trip(d: Device) -> None:
    handoff = "a" * 64
    url = f"virya-signal://my-signal?source=synesthesia#handoff={handoff}"
    # The custom scheme is the installed-app primary path and does not depend
    # on Android domain verification. Package-scoped VIEW still exercises the
    # merged intent filter before Signal can receive the capability.
    shell(
        "am", "start", "-W",
        "-a", "android.intent.action.VIEW",
        "-d", url,
        "-p", PKG,
        timeout=20,
    )
    d.assert_text([
        "Wynik Synesthesia zapisany w Signal.",
        "Synesthesia result saved in Signal.",
    ], timeout=25)
    d.snapshot("synesthesia-app-link-linked")


def cold_start_confirm_link_then_resume(d: Device) -> None:
    """B5/B6 proof: a cold-start mailed confirmation link opens the confirm
    panel, and dismissing it with "Back to PIN login" clears the native
    pending token so a process-kill + relaunch does not re-offer it."""
    token = "b" * 64
    url = f"virya-signal://fan/confirm?token={token}"
    # Cold start: force-stop, then launch with the confirm intent.
    shell("am", "force-stop", PKG, check=False)
    shell(
        "am", "start", "-W",
        "-a", "android.intent.action.VIEW",
        "-d", url,
        "-p", PKG,
        timeout=20,
    )
    # The confirm panel should appear with the "CONFIRM AND ENTER" / Polish
    # equivalent button and the "Back to PIN login" dismiss control.
    d.assert_text([
        "POTWIERDŹ I WEJDŹ",
        "CONFIRM AND ENTER",
    ], timeout=25)
    d.snapshot("cold-start-confirm-link-panel")
    # Dismiss the panel. This calls fan_clear_pending_confirm_link which
    # clears both the Rust-side pending token and the native Android link.
    d.tap([
        "WRÓĆ DO LOGOWANIA PIN-EM",
        "BACK TO PIN LOGIN",
    ], timeout=10)
    d.snapshot("cold-start-confirm-link-dismissed")
    # Process-kill + relaunch: the panel must NOT reappear. The native
    # pending link was cleared, so fan_take_confirm_link returns false.
    shell("am", "force-stop", PKG, check=False)
    shell("monkey", "-p", PKG, "-c", "android.intent.category.LAUNCHER", "1", timeout=20)
    # Wait for the app to paint. The confirm panel text must be absent.
    d.assert_absent([
        "POTWIERDŹ I WEJDŹ",
        "CONFIRM AND ENTER",
    ], seconds=20)
    d.snapshot("cold-start-confirm-link-not-reoffered")


def fan_event_details(d: Device) -> None:
    d.assert_text(["VIRYA E2E — TEST EVENT"], timeout=25)
    d.tap(["SZCZEGÓŁY", "DETAILS"], timeout=10)
    d.assert_text(["VIRYA E2E — TEST EVENT"], timeout=15)
    d.assert_text(["KUP BILET", "BUY TICKET"], timeout=15)
    d.snapshot("next-signal-event-detail")


def fan_settings_survives_android_settings(d: Device) -> None:
    d.tap(["Otwórz menu", "Open menu"], timeout=10)
    d.tap(["Profil", "Profile"], timeout=10)
    d.assert_text(["Ustawienia Sygnału", "Signal settings"], timeout=15)
    # The E2E build inherits the Firebase config but stays unsigned/debug. The
    # settings status therefore proves that the runtime can initialize Firebase
    # from the compiled resources; a broken build shows the dedicated
    # firebase_not_configured message instead of this normal OFF state.
    d.assert_text([
        "Powiadomienia są wyłączone na tym urządzeniu.",
        "Notifications are disabled on this device.",
    ], timeout=20)
    d.snapshot("fan-settings-before-native")
    # Exercise the exact native Activity boundary that used to remount FanApp and reset FanTab.
    shell("am", "start", "-a", "android.settings.APP_NOTIFICATION_SETTINGS", "--es", "android.provider.extra.APP_PACKAGE", PKG, timeout=15)
    time.sleep(1.5)
    shell("input", "keyevent", "KEYCODE_BACK")
    d.assert_text(["Ustawienia Sygnału", "Signal settings"], timeout=15)
    if d.exists(["TWÓJ SYGNAŁ", "YOUR SIGNAL NOW"], timeout=1):
        # Home copy may occur elsewhere in a long profile, so only fail if settings disappeared.
        d.assert_text(["Ustawienia Sygnału", "Signal settings"], timeout=2)
    d.snapshot("fan-settings-after-native")


def open_staff_and_configure_owner(d: Device) -> None:
    # Profile -> overflow menu -> staff zone.
    d.tap(["Otwórz menu", "Open menu"], timeout=10)
    d.tap(["Strefa staff", "Staff zone"], timeout=10)
    d.input(["Hasło staffu", "Staff password"], "e2e")
    d.tap(["OTWÓRZ STREFĘ STAFF", "OPEN STAFF ZONE"], timeout=10)
    d.assert_text(["VIRYA CONTROL"], timeout=15)
    d.tap(["USTAWIENIA ZAAWANSOWANE", "ADVANCED SETTINGS"], timeout=10)
    d.clear_and_input(["API CrowdRelay"], "http://10.0.2.2:8787/v1/")
    d.input(["Token urządzenia", "Device token"], OWNER_TOKEN)
    d.input(["Utwórz PIN odblokowania", "Create an unlock PIN"], PIN)
    d.tap(["OWNER"], timeout=10)
    d.snapshot("owner-manual-config")
    d.tap(["ZAPISZ RĘCZNIE", "SAVE MANUALLY"], timeout=10)
    d.assert_text(["LIVE OPERATIONS", "VIRYA CONTROL"], timeout=20)


def staff_language_switch_preserves_session(d: Device) -> None:
    d.tap(["Otwórz menu", "Open menu"], timeout=10)
    d.tap(["Ustawienia", "Settings"], timeout=10)
    d.assert_text(["Ustawienia", "Settings"], timeout=15)

    # Language buttons are deliberately selected by exact text. Substring matching
    # for the two-letter labels (especially EN) is too broad for a UI hierarchy.
    d.tap_exact_text("EN", timeout=10)
    d.assert_text(["Settings"], timeout=15)
    d.assert_text(["Connection"], timeout=15)
    d.assert_text(["Permissions"], timeout=15)
    d.assert_absent(["Staff password", "Hasło staffu", "OPEN STAFF ZONE", "OTWÓRZ STREFĘ STAFF"], seconds=3)
    d.snapshot("staff-language-en-session-preserved")

    d.tap_exact_text("PL", timeout=10)
    d.assert_text(["Ustawienia"], timeout=15)
    d.assert_text(["Połączenie"], timeout=15)
    d.assert_text(["Uprawnienia"], timeout=15)
    d.assert_absent(["Staff password", "Hasło staffu", "OPEN STAFF ZONE", "OTWÓRZ STREFĘ STAFF"], seconds=3)
    d.snapshot("staff-language-pl-session-preserved")


def owner_online_and_offline_cache(d: Device, offline_file: Path) -> None:
    d.tap(["Sygnał", "Signal"], timeout=10)
    d.assert_text(["Społeczność i wzrost", "Community and growth"], timeout=20)
    d.assert_text(["7"], timeout=10)
    d.snapshot("owner-online-signal")
    offline_file.parent.mkdir(parents=True, exist_ok=True)
    offline_file.touch()
    relaunch()
    d.assert_text(["VIRYA CONTROL"], timeout=25)
    if d.exists(["PIN", "Your PIN", "Twój PIN"], timeout=4):
        # Operator vault is intentionally locked after process restart.
        d.input(["PIN", "Your PIN", "Twój PIN"], PIN)
        d.tap(["ODBLOKUJ", "UNLOCK"], timeout=10)
    d.tap(["Sygnał", "Signal"], timeout=15)
    d.assert_text(["Społeczność i wzrost", "Community and growth"], timeout=20)
    d.assert_text(["7"], timeout=10)
    d.snapshot("owner-offline-last-known-good")
    offline_file.unlink(missing_ok=True)



def staff_checklist_never_hangs(d: Device) -> None:
    d.tap(["Otwórz menu", "Open menu"], timeout=10)
    d.tap(["CHECKLISTA KONCERTOWA", "GIG CHECKLIST"], timeout=10)
    d.assert_text(["VIRYA E2E — TEST EVENT"], timeout=20)
    # Whether FCM is available on the emulator or not, the UI must terminate its async state.
    d.assert_absent(["SYNCHRONIZUJĘ", "SYNCING"], seconds=50)
    d.snapshot("staff-checklist-terminal-state")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifacts", default="artifacts/android-e2e")
    parser.add_argument("--offline-file", required=True)
    args = parser.parse_args()
    artifacts = Path(args.artifacts).resolve()
    offline_file = Path(args.offline_file).resolve()
    offline_file.unlink(missing_ok=True)
    d = Device(artifacts)

    launch_clean()
    try:
        fan_first_login(d)
        fan_recovery_qr(d)
        synesthesia_app_link_round_trip(d)
        cold_start_confirm_link_then_resume(d)
        fan_event_details(d)
        fan_settings_survives_android_settings(d)
        open_staff_and_configure_owner(d)
        staff_language_switch_preserves_session(d)
        owner_online_and_offline_cache(d, offline_file)
        staff_checklist_never_hangs(d)
    except Exception:
        d.snapshot("failure")
        raise
    finally:
        offline_file.unlink(missing_ok=True)
        (artifacts / "logcat.txt").write_text(adb("logcat", "-d", check=False, timeout=30), errors="replace")

    logcat = (artifacts / "logcat.txt").read_text(errors="replace")
    fatal = [line for line in logcat.splitlines() if "FATAL EXCEPTION" in line or "ANR in music.virya.signal" in line or "Render process gone" in line]
    if fatal:
        raise JourneyError("native/WebView crash detected:\n" + "\n".join(fatal[-20:]))
    print("VIRYA_SIGNAL_ANDROID_E2E=PASS journeys=fan_auth,fan_recovery,synesthesia_app_link,cold_start_confirm_link,event_detail,native_settings,staff_language,owner_online,owner_offline,staff_checklist")


if __name__ == "__main__":
    main()
