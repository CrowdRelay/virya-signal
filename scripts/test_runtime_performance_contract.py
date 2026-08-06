#!/usr/bin/env python3
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def toml_section(path, name):
    text = path.read_text()
    match = re.search(
        rf"(?ms)^\[{re.escape(name)}\]\s*$\n(?P<body>.*?)(?=^\[|\Z)",
        text,
    )
    if not match:
        raise AssertionError(f"missing TOML section [{name}] in {path}")
    return match.group("body")


def toml_value(path, section, key):
    body = toml_section(path, section)
    match = re.search(rf"(?m)^{re.escape(key)}\s*=\s*(?P<value>.+?)\s*$", body)
    if not match:
        raise AssertionError(f"missing TOML key [{section}] {key} in {path}")
    raw = match.group("value").split("#", 1)[0].strip()
    if raw == "true":
        return True
    if raw == "false":
        return False
    if re.fullmatch(r"-?\d+", raw):
        return int(raw)
    quoted = re.fullmatch(r'"(.*)"', raw)
    return quoted.group(1) if quoted else raw


def cargo_lock_packages(path):
    packages = []
    for block in re.split(r"(?m)^\[\[package\]\]\s*$", path.read_text())[1:]:
        name = re.search(r'(?m)^name = "([^"]+)"\s*$', block)
        version = re.search(r'(?m)^version = "([^"]+)"\s*$', block)
        if not name or not version:
            continue
        dependencies = []
        dependency_block = re.search(
            r"(?ms)^dependencies = \[\s*(.*?)^\]\s*$",
            block,
        )
        if dependency_block:
            dependencies = re.findall(r'"([^"]+)"', dependency_block.group(1))
        packages.append(
            {
                "name": name.group(1),
                "version": version.group(1),
                "dependencies": dependencies,
            }
        )
    return packages


class RuntimePerformanceContract(unittest.TestCase):
    def test_release_profiles_remain_size_optimized(self):
        manifest = ROOT / "Cargo.toml"
        self.assertEqual(toml_value(manifest, "profile.release", "opt-level"), "s")
        self.assertIs(toml_value(manifest, "profile.release", "lto"), True)
        self.assertEqual(toml_value(manifest, "profile.release", "codegen-units"), 1)
        self.assertEqual(toml_value(manifest, "profile.release", "strip"), "symbols")
        self.assertEqual(toml_value(manifest, "profile.release", "panic"), "abort")
        self.assertEqual(
            toml_value(
                manifest,
                "profile.release.package.virya-signal-ui",
                "opt-level",
            ),
            "z",
        )

    def test_vault_uses_os_rng_without_rand_facade(self):
        manifest = ROOT / "src-tauri/Cargo.toml"
        self.assertEqual(toml_value(manifest, "dependencies", "getrandom"), "0.4")
        self.assertNotRegex(toml_section(manifest, "dependencies"), r"(?m)^rand\s*=")
        vault = (ROOT / "src-tauri/src/vault.rs").read_text()
        self.assertIn("getrandom::fill(&mut salt)", vault)
        self.assertNotIn("rand::", vault)
        lock_packages = cargo_lock_packages(ROOT / "Cargo.lock")
        native = next(
            package for package in lock_packages if package["name"] == "virya-signal"
        )
        self.assertTrue(
            any(
                value.startswith("getrandom 0.4.")
                for value in native["dependencies"]
            )
        )
        self.assertFalse(
            any(value.startswith("rand 0.10.") for value in native["dependencies"])
        )
        self.assertFalse(
            any(
                package["name"] == "rand"
                and package["version"].startswith("0.10.")
                for package in lock_packages
            )
        )

    def test_wallet_loading_concurrency_is_bounded(self):
        source = (ROOT / "src-tauri/src/commands/fan.rs").read_text()
        self.assertIn("const WALLET_FETCH_CONCURRENCY: usize = 8;", source)
        self.assertRegex(source, r"\.buffered\(WALLET_FETCH_CONCURRENCY\)")
        self.assertNotRegex(source, r"\.buffered\((?:9|[1-9]\d+)\)")

    def test_cache_persistence_uses_required_atomic_ordering_only(self):
        source = (ROOT / "src-tauri/src/api/client.rs").read_text()
        self.assertIn("swap(true, Ordering::AcqRel)", source)
        self.assertIn("store(false, Ordering::Release)", source)
        persistence = source[
            source.index("pub(super) fn persist_public_cache_in_background") :
        ]
        persistence = persistence[
            : persistence.index("\n    }\n", persistence.index("tokio::spawn")) + 7
        ]
        self.assertNotIn("Ordering::SeqCst", persistence)

    def test_show_mode_prunes_only_safe_expired_sessions(self):
        source = (ROOT / "src-tauri/src/commands/show_mode.rs").read_text()
        self.assertIn("store.sessions.retain", source)
        self.assertIn("scan.state != ShowModeScanState::Synced", source)
        self.assertIn("active || has_unsynced_scans", source)
        self.assertIn(
            "show_store_normalization_prunes_only_expired_fully_synced_sessions",
            source,
        )

    def test_public_cache_stays_bounded(self):
        source = (ROOT / "src-tauri/src/api/cache.rs").read_text()
        for fragment in (
            "MAX_PUBLIC_EVENTS: usize = 100",
            "MAX_PUBLIC_CITIES: usize = 250",
            "MAX_CACHE_ORIGINS: usize = 8",
            "MAX_DISK_CACHE_BYTES: u64 = 2 * 1024 * 1024",
        ):
            self.assertIn(fragment, source)


if __name__ == "__main__":
    unittest.main()
