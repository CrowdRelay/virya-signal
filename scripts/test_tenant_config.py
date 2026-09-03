#!/usr/bin/env python3
"""Contract tests for the per-tenant Signal build pipeline.

Verifies that:
1. fetch-tenant-config.py exists and has the expected CLI interface.
2. build-tenant-app.sh exists and is syntactically valid bash.
3. generate-tenant-icons.py exists and has the expected CLI interface.
4. The tauri.conf.json template has all required placeholders.
5. The CI workflow exists and accepts the expected inputs.
6. The package ID derivation follows the music.{slug}.signal pattern.
"""
from __future__ import annotations

import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FETCHER = ROOT / "scripts/fetch-tenant-config.py"
BUILDER = ROOT / "scripts/build-tenant-app.sh"
ICON_GEN = ROOT / "scripts/generate-tenant-icons.py"
TEMPLATE = ROOT / "scripts/templates/tauri.conf.json.template"
WORKFLOW = ROOT / ".github/workflows/tenant-app-build.yml"
ONBOARDER = ROOT / "scripts/onboard-tenant-app.sh"
KEYSTORE_GEN = ROOT / "scripts/generate-tenant-keystore.sh"
FIREBASE_SETUP = ROOT / "scripts/setup-tenant-firebase.py"


class TenantBuildContract(unittest.TestCase):
    def test_fetcher_exists_and_has_cli(self) -> None:
        self.assertTrue(FETCHER.exists(), "fetch-tenant-config.py is missing")
        source = FETCHER.read_text(encoding="utf-8")
        self.assertIn("--tenant", source)
        self.assertIn("--control-plane-url", source)
        self.assertIn("--token", source)
        self.assertIn("--output", source)

    def test_fetcher_derives_package_id(self) -> None:
        source = FETCHER.read_text(encoding="utf-8")
        self.assertIn("music.{slug}.signal", source)
        self.assertIn("packageId", source)

    def test_builder_exists_and_is_valid_bash(self) -> None:
        self.assertTrue(BUILDER.exists(), "build-tenant-app.sh is missing")
        result = subprocess.run(
            ["bash", "-n", str(BUILDER)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, f"bash syntax error: {result.stderr}")

    def test_builder_calls_fetcher_and_icon_gen(self) -> None:
        source = BUILDER.read_text(encoding="utf-8")
        self.assertIn("fetch-tenant-config.py", source)
        self.assertIn("generate-tenant-icons.py", source)
        self.assertIn("cargo tauri android build", source)

    def test_builder_backs_up_and_restores_tauri_conf(self) -> None:
        source = BUILDER.read_text(encoding="utf-8")
        self.assertIn(".bak", source)
        self.assertIn("trap", source)
        self.assertIn("RESTORE", source)

    def test_icon_gen_exists_and_has_cli(self) -> None:
        self.assertTrue(ICON_GEN.exists(), "generate-tenant-icons.py is missing")
        source = ICON_GEN.read_text(encoding="utf-8")
        self.assertIn("--config", source)
        self.assertIn("--output-dir", source)
        self.assertIn("Pillow", source)

    def test_template_has_required_placeholders(self) -> None:
        self.assertTrue(TEMPLATE.exists(), "tauri.conf.json.template is missing")
        content = TEMPLATE.read_text(encoding="utf-8")
        for placeholder in (
            "{{APP_NAME}}",
            "{{APP_VERSION}}",
            "{{PACKAGE_ID}}",
            "{{SURFACE_COLOR}}",
            "{{CSP_CONNECT_SOURCES}}",
            "{{ANDROID_VERSION_CODE}}",
        ):
            self.assertIn(placeholder, content, f"template missing {placeholder}")

    def test_template_uses_tenant_icon_paths(self) -> None:
        content = TEMPLATE.read_text(encoding="utf-8")
        self.assertIn("icons/tenant/", content)

    def test_workflow_exists_and_accepts_tenant_slug(self) -> None:
        self.assertTrue(WORKFLOW.exists(), "tenant-app-build.yml is missing")
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("tenant_slug", content)
        self.assertIn("version", content)
        self.assertIn("version_code", content)
        self.assertIn("workflow_dispatch", content)

    def test_workflow_calls_build_script(self) -> None:
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("build-tenant-app.sh", content)
        self.assertIn("CONTROL_PLANE_ADMIN_TOKEN", content)

    def test_workflow_auto_populates_play_url(self) -> None:
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("Auto-populate Play Store URL", content)
        self.assertIn("/mobile-apps", content)

    def test_onboarder_exists_and_is_valid_bash(self) -> None:
        self.assertTrue(ONBOARDER.exists(), "onboard-tenant-app.sh is missing")
        result = subprocess.run(
            ["bash", "-n", str(ONBOARDER)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, f"bash syntax error: {result.stderr}")

    def test_onboarder_calls_sub_scripts(self) -> None:
        source = ONBOARDER.read_text(encoding="utf-8")
        self.assertIn("fetch-tenant-config.py", source)
        self.assertIn("generate-tenant-keystore.sh", source)
        self.assertIn("setup-tenant-firebase.py", source)

    def test_keystore_gen_exists_and_is_valid_bash(self) -> None:
        self.assertTrue(KEYSTORE_GEN.exists(), "generate-tenant-keystore.sh is missing")
        result = subprocess.run(
            ["bash", "-n", str(KEYSTORE_GEN)],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, f"bash syntax error: {result.stderr}")

    def test_firebase_setup_exists(self) -> None:
        self.assertTrue(FIREBASE_SETUP.exists(), "setup-tenant-firebase.py is missing")
        source = FIREBASE_SETUP.read_text(encoding="utf-8")
        self.assertIn("--tenant", source)
        self.assertIn("--package-id", source)
        self.assertIn("--project-id", source)


if __name__ == "__main__":
    unittest.main()
