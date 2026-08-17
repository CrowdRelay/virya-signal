#!/usr/bin/env python3
from rust_source_tree import read_rust_module
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
        source = read_rust_module(ROOT, "src-tauri/src/commands/fan.rs")
        self.assertIn("const WALLET_FETCH_CONCURRENCY: usize = 8;", source)
        self.assertRegex(source, r"\.buffered\(WALLET_FETCH_CONCURRENCY\)")
        self.assertNotRegex(source, r"\.buffered\((?:9|[1-9]\d+)\)")

    def test_cache_persistence_uses_required_atomic_ordering_only(self):
        source = (ROOT / "src-tauri/src/api/client.rs").read_text()
        self.assertIn("swap(true, Ordering::AcqRel)", source)
        self.assertIn("store(false, Ordering::Release)", source)
        self.assertIn("cache_dirty.store(true, Ordering::Release)", source)
        self.assertIn("cache_dirty.load(Ordering::Acquire)", source)
        self.assertIn("loop {", source[source.index("pub(super) fn persist_public_cache_in_background") :])
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

    def test_merch_catalog_revalidates_with_bounded_memory_cache(self):
        source = (ROOT / "src-tauri/src/api/client.rs").read_text()
        for fragment in (
            "const MERCH_CACHE_TTL: Duration = Duration::from_secs(15);",
            "const MERCH_STALE_TTL: Duration = Duration::from_secs(10 * 60);",
            "merch_fetch: Arc<Mutex<()>>",
            "merch_cache: Arc<RwLock<HashMap<String, CacheEntry<MerchCatalog>>>>",
            "self.merch_fetch.lock().await",
            'public_response_base(api_base_url, "public/merch/catalog", validators)',
            "self.touch_merch_cache(&cache_key).await",
            "cache::prune_cache(&mut cache, MERCH_STALE_TTL)",
        ):
            self.assertIn(fragment, source)
        self.assertIn(
            ".filter(|entry| entry.fetched_at.elapsed() < MERCH_STALE_TTL)",
            source,
        )

    def test_conditional_validators_never_outlive_stale_fallback(self):
        source = (ROOT / "src-tauri/src/api/client.rs").read_text()
        for ttl in ("EVENTS_STALE_TTL", "CITIES_STALE_TTL", "MERCH_STALE_TTL"):
            self.assertIn(
                f".filter(|entry| entry.fetched_at.elapsed() < {ttl})",
                source,
            )

    def test_public_cache_serves_stale_on_transient_http_failures(self):
        source = (ROOT / "src-tauri/src/api/client.rs").read_text()
        self.assertIn("fn transient_public_status", source)
        self.assertIn("status == reqwest::StatusCode::TOO_MANY_REQUESTS", source)
        self.assertIn("status.is_server_error()", source)
        self.assertEqual(source.count("if transient_public_status(response.status())"), 3)
        # Borrow stale values before cloning so later 304 handling can still consume
        # the same Option; a by-value `if let` here would fail Rust compilation.
        self.assertEqual(source.count("= stale.as_ref()"), 3)

    def test_privileged_correlation_header_matches_backend_contract(self):
        client = (ROOT / "src-tauri/src/api/client.rs").read_text()
        self.assertIn('header("X-CrowdRelay-Correlation-Id", correlation_id.as_str())', client)
        self.assertNotIn('header("X-Correlation-ID"', client)

    def test_private_fan_home_cache_never_masks_auth_failures(self):
        fan = (ROOT / "src-tauri/src/api/fan.rs").read_text()
        retry = (ROOT / "src-tauri/src/api/retry.rs").read_text()
        self.assertIn("FAN_HOME_CACHE_TTL", fan)
        self.assertIn("FAN_HOME_STALE_TTL", fan)
        self.assertIn("FAN_HOME_STALE_TTL: Duration = Duration::from_secs(10 * 60)", (ROOT / "src-tauri/src/api/client.rs").read_text())
        self.assertIn("fan_home_fetch.lock().await", fan)
        self.assertIn("Err(error) if super::retry::is_transient_failure(&error)", fan)
        self.assertIn("Err(error) => Err(error)", fan)
        self.assertIn("pub(super) fn is_transient_failure", retry)
        for status in ("408", "425", "429", "500", "502", "503", "504"):
            self.assertIn(status, retry)

    def test_wallet_offline_fallback_is_encrypted_expiry_checked_and_webview_safe(self):
        models = read_rust_module(ROOT, "src-tauri/src/models.rs")
        commands = read_rust_module(ROOT, "src-tauri/src/commands/fan.rs")
        ui = read_rust_module(ROOT, "src/app/fan.rs")
        self.assertIn("pub cached_wallets: Vec<TicketWallet>", models)
        self.assertIn("pub cached_wallet_qr: Vec<WalletQrCredential>", models)
        self.assertIn("#[zeroize(drop)]\npub struct WalletQrCredential", models)
        self.assertIn("pub cached: bool", models)
        self.assertIn("cached.cached = true", commands)
        self.assertIn("wallet_qr_credential_valid(entry)", commands)
        self.assertIn("expires_at > time::OffsetDateTime::now_utc()", commands)
        self.assertIn("persist_fan(&state, &updated).await?", commands)
        self.assertIn("profile.cached_wallet_qr", commands)
        # The token is consumed in native render_wallet_qr; TicketWallet sent to
        # the WebView contains public status/redeemed_at + QR availability/expiry,
        # never the raw token.
        wallet_block = models.split("pub struct TicketWallet {", 1)[1].split("}", 1)[0]
        ticket_block = models.split("pub struct WalletTicket {", 1)[1].split("}", 1)[0]
        self.assertNotIn("token", wallet_block)
        self.assertNotIn("token", ticket_block)
        self.assertIn("pub status: String", ticket_block)
        self.assertIn("pub redeemed_at: Option<String>", ticket_block)
        self.assertIn('tr("wallet_cached_offline")', ui)
        self.assertIn('"redeemed" =>', ui)
        self.assertIn('tr("wallet_ticket_revoked")', ui)

    def test_native_error_log_preserves_full_crowdrelay_git_sha(self):
        http = (ROOT / "src-tauri/src/api/http.rs").read_text()
        models = read_rust_module(ROOT, "src-tauri/src/models.rs")
        self.assertIn("value.chars().take(40).collect::<String>()", http)
        self.assertIn("pub git_sha: Option<String>", models)

    def test_ops_surfaces_postgres18_async_io_runtime_evidence(self):
        native_models = read_rust_module(ROOT, "src-tauri/src/models.rs")
        web_models = (ROOT / "src/models.rs").read_text()
        shared_ops = (ROOT / "crates/virya-signal-contracts/src/ops.rs").read_text()
        ui = read_rust_module(ROOT, "src/app/operator.rs")
        self.assertIn("pub use virya_signal_contracts::ops::*;", native_models)
        self.assertIn("pub use virya_signal_contracts::ops::*;", web_models)
        for fragment in (
            "io_combine_limit_bytes: Option<i64>",
            "io_max_combine_limit_bytes: Option<i64>",
        ):
            self.assertIn(fragment, shared_ops)
        for fragment in (
            'label="io_combine"',
            'label="io_max_combine"',
        ):
            self.assertIn(fragment, ui)

    def test_feedback_outbox_promotes_new_payload_without_deleting_the_old_queue_first(self):
        queue = (ROOT / "src-tauri/src/feedback_queue.rs").read_text()
        app_state = (ROOT / "src-tauri/src/lib.rs").read_text()
        commands = (ROOT / "src-tauri/src/commands/misc.rs").read_text()
        self.assertIn('format!("{FILE_NAME}.bak")', queue)
        self.assertIn('File::open(&temp_path)?.sync_all()?', queue)
        self.assertIn('fs::rename(&final_path, &backup_path)?', queue)
        self.assertIn('let _ = fs::rename(&backup_path, &final_path);', queue)
        self.assertIn("feedback_queue_mutation: Mutex<()>", app_state)
        self.assertGreaterEqual(commands.count("state.feedback_queue_mutation.lock().await"), 2)
        self.assertLessEqual(8, 8)



if __name__ == "__main__":
    unittest.main()
