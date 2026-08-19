#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
svg = (ROOT / "branding/signal-v2.svg").read_text(encoding="utf-8")
regen = (ROOT / "scripts/regenerate-signal-icons.fish").read_text(encoding="utf-8")
play = (ROOT / "scripts/play-build.fish").read_text(encoding="utf-8")
android = (ROOT / ".github/workflows/_android-build.yml").read_text(encoding="utf-8")
mobile = (ROOT / ".github/workflows/mobile-release.yml").read_text(encoding="utf-8")
tauri = (ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8")

assert 'fill="#070908"' in svg
assert svg.count('fill="#93c6c0"') >= 3
assert '<rect x="243" y="379" width="98" height="266" rx="14"' in svg
assert '<rect x="463" y="251" width="98" height="522" rx="14"' in svg
assert '<rect x="683" y="329" width="98" height="366" rx="14"' in svg
assert "#f3c51a" not in svg.lower()
assert 'branding/signal-v2.svg --ios-color "#070908"' in regen
assert "regenerate-signal-icons.fish" in play
assert 'cargo tauri icon branding/signal-v2.svg --ios-color "#070908"' in android
assert 'cargo tauri icon branding/signal-v2.svg --ios-color "#070908"' in mobile
assert "virya-signal-brand-full.png --ios-color" not in mobile
assert '"backgroundColor": "#070908"' in tauri

print("SIGNAL_V2_ICON_PIPELINE=PASS")
