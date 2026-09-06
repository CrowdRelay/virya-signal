#!/usr/bin/env python3
"""Contracts for the four places Signal could still say something false.

Each class here corresponds to a defect that shipped and was fixed:

1. `stale` was written by hand at each public-list return site, and the 304
   branch had it backwards — a list the origin had just certified as current
   was shown under a "cached data" badge.

2. The session phase lived in its own lock beside the credentials it described.
   On a cold start that lock read `Unconfigured` while `configured` was read
   from the vault on disk, so a device holding a real locked vault reported
   that no identity existed.

3. `clearFanConfirmAppLink` existed only in the *generated* Android plugin, not
   in the template every build copies over it, and the Rust caller discarded the
   result. The next Android build would have deleted the command and the failure
   would have been invisible: "Back to PIN login" would close the panel and the
   next resume tick would reopen it.

4. `deviceSecretSupported` proves a keystore provider exists, not that the
   configured AES/GCM key can be generated. The vault is written before the
   seal, so a device that refuses the real key was left holding a snapshot with
   no PIN behind it and no key to open it.

These are source-shape contracts. They cannot prove Android lifecycle
behaviour; the Rust tables in `api/cache.rs` and `models/tests.rs` prove the
mappings, and the Android E2E proves the intent handling.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KOTLIN_TEMPLATE = ROOT / "src-tauri/android-push/SignalPushPlugin.kt"
PUSH_BRIDGE = ROOT / "src-tauri/src/push_plugin.rs"
CLIENT = ROOT / "src-tauri/src/api/client.rs"


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def item_body(source: str, signature: str) -> str:
    start = source.find(signature)
    if start == -1:
        raise AssertionError(f"missing item: {signature}")
    next_item = re.search(
        r"(?m)^(?:pub(?:\(crate\))? )?(?:async )?fn ", source[start + len(signature) :]
    )
    end = start + len(signature) + next_item.start() if next_item else len(source)
    return source[start:end]


class CacheFreshnessTruth(unittest.TestCase):
    def test_every_public_list_freshness_answer_comes_from_the_table(self) -> None:
        client = CLIENT.read_text(encoding="utf-8")
        literals = re.findall(r"stale: (true|false)\b", client)
        self.assertEqual(
            literals,
            [],
            "public list results must derive `stale` from PublicFreshness, "
            f"not from literals: {literals}",
        )
        # All four sources are actually reachable from this file.
        for source in ("Live", "FreshCache", "Revalidated", "Unvalidated"):
            self.assertIn(f"PublicFreshness::{source}.stale()", client, source)

    def test_a_304_is_reported_as_current_not_cached(self) -> None:
        client = CLIENT.read_text(encoding="utf-8")
        for marker in ("NOT_MODIFIED", "PublicFreshness::Revalidated.stale()"):
            self.assertIn(marker, client, marker)
        # Each 304 branch (events, cities) ends in Revalidated, and neither of
        # them reaches for the Unvalidated answer used by the failure paths.
        for branch in client.split("== reqwest::StatusCode::NOT_MODIFIED")[1:]:
            head = branch[: branch.index("let (etag")]
            self.assertIn("PublicFreshness::Revalidated.stale()", head)
            self.assertNotIn("PublicFreshness::Unvalidated.stale()", head)
        # Events, cities and the merch catalog: all three public lists.
        self.assertEqual(client.count("PublicFreshness::Revalidated.stale()"), 3)

    def test_the_disk_snapshot_answers_for_itself(self) -> None:
        # `fan_cached_events` used to hardcode `stale: true`. The freshness
        # answer belongs beside every other one, in the API layer.
        client = CLIENT.read_text(encoding="utf-8")
        snapshot = item_body(client, "pub async fn public_events_snapshot(")
        self.assertIn("PublicEventsResult", snapshot)
        self.assertIn("PublicFreshness::Unvalidated.stale()", snapshot)

        command = item_body(
            read("src-tauri/src/commands/fan/session_commerce.rs"),
            "pub(crate) async fn fan_cached_events(",
        )
        self.assertIn("public_events_snapshot", command)
        self.assertNotIn("stale:", command)

    def test_no_consumer_silently_drops_the_freshness_answer(self) -> None:
        # Every UI reader of a public-list envelope must carry `stale` through
        # to something the user can see. Dropping it is how the operator panel
        # presented a failure-window cache as the live schedule.
        support = read("src/app/support.rs")
        for reader, sink in (
            ("PublicEventsResult", "events_stale"),
            ("MerchCatalogResult", "merch_stale"),
        ):
            self.assertIn(reader, support, reader)
            self.assertIn(sink, support, sink)
        # Fan cached snapshot, fan live list, operator live list.
        self.assertEqual(support.count("data.events_stale = value.stale;"), 3)
        self.assertEqual(support.count("merch_stale.set(value.stale);"), 2)
        for surface, anchor in (
            ("src/app/fan/events.rs", "events_stale"),
            ("src/app/fan/merch.rs", "merch_stale"),
            ("src/app/operator/shell.rs", "events_stale"),
        ):
            source = read(surface)
            self.assertIn(anchor, source, surface)
            self.assertIn('class="cache-badge"', source, surface)

        # A disk snapshot painted before any live request is unvalidated by
        # definition, on both sides of the app.
        operator_snapshot = read("src/app/operator/shell.rs").split(
            "with_operator_cached_sections(", 1
        )[1][:900]
        self.assertIn("data.events_stale = true;", operator_snapshot)

    def test_stale_means_one_thing_in_the_envelopes(self) -> None:
        models = read("src-tauri/src/models/commerce_events.rs")
        for name in ("PublicEventsResult", "PublicCitiesResult", "MerchCatalogResult"):
            body = models.split(f"pub struct {name} {{", 1)[1].split("}", 1)[0]
            self.assertEqual(
                body.count("pub "),
                2,
                f"{name} must carry exactly the payload and one freshness flag",
            )
            self.assertIn("stale: bool", body, name)
            for overloaded in ("cached", "failed", "from_disk", "available"):
                self.assertNotIn(f"{overloaded}:", body, f"{name}.{overloaded}")


class SessionPhaseTruth(unittest.TestCase):
    def test_no_phase_is_stored_beside_the_credentials_it_describes(self) -> None:
        native = ROOT / "src-tauri/src"
        for path in native.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for field in ("operator_phase", "fan_phase", "beacon_phase"):
                self.assertNotIn(
                    f"state.{field}",
                    source,
                    f"{path.relative_to(ROOT)} keeps a second copy of the phase; "
                    "derive it from configured/unlocked instead",
                )
                self.assertNotIn(f"{field}: RwLock", source, str(path))

    def test_every_status_derives_its_phase_from_the_facts_it_publishes(self) -> None:
        sites = {
            "src-tauri/src/commands/fan/session_commerce.rs": (
                "pub(crate) async fn fan_status(",
                "FanSessionPhase::resolve(configured, unlocked)",
            ),
            "src-tauri/src/commands/operator.rs": (
                "pub(crate) async fn session_status(",
                "OperatorSessionPhase::resolve(configured, unlocked)",
            ),
            "src-tauri/src/commands/beacon.rs": (
                "async fn status_from_state(",
                "BeaconSessionPhase::resolve(configured, unlocked)",
            ),
        }
        for module, (signature, expected) in sites.items():
            body = item_body(read(module), signature)
            self.assertIn(expected, body, module)
            self.assertIn("let unlocked = ", body, module)
            self.assertIn("let configured = ", body, module)

        # The launcher reports all three at once and must derive each of them.
        launcher = item_body(
            read("src-tauri/src/commands/misc.rs"), "pub(crate) async fn launcher_status("
        )
        for expected in (
            "OperatorSessionPhase::resolve(operator_configured, operator_unlocked)",
            "FanSessionPhase::resolve(fan_configured, fan_unlocked)",
            "BeaconSessionPhase::resolve(beacon_configured, beacon_unlocked)",
        ):
            self.assertIn(expected, launcher, expected)

    def test_the_phase_is_one_function_shared_by_all_three_domains(self) -> None:
        models = read("src-tauri/src/models/session_fan.rs")
        self.assertIn("macro_rules! session_phase", models)
        for name in ("OperatorSessionPhase", "FanSessionPhase", "BeaconSessionPhase"):
            self.assertIn(f"session_phase!({name});", models, name)
        self.assertEqual(models.count("pub const fn resolve("), 1)


class ConfirmLinkClearTruth(unittest.TestCase):
    def test_the_template_declares_every_command_the_bridge_invokes(self) -> None:
        # `clearFanConfirmAppLink` was added only to `gen/android/…`, which
        # `prepare-android.py` overwrites from this template on every build.
        template = KOTLIN_TEMPLATE.read_text(encoding="utf-8")
        declared = set(re.findall(r"(?m)^\s*fun ([A-Za-z]+)\(invoke: Invoke\)", template))
        invoked = set(
            re.findall(
                r'run_mobile_plugin::<[^>]+>\(\s*"([A-Za-z]+)"',
                PUSH_BRIDGE.read_text(encoding="utf-8"),
            )
        )
        self.assertTrue(invoked, "no native plugin commands found in the Rust bridge")
        self.assertEqual(
            invoked - declared,
            set(),
            "the Rust bridge invokes native commands the canonical Kotlin "
            "template does not declare; the next Android build would delete them",
        )
        self.assertIn("clearFanConfirmAppLink", invoked)

    def test_clearing_the_link_reports_the_boundary_it_actually_reached(self) -> None:
        command = item_body(
            read("src-tauri/src/commands/fan/session_commerce.rs"),
            "pub(crate) async fn fan_clear_pending_confirm_link(",
        )
        self.assertNotIn("let _ =", command, "a swallowed native refusal claims a clear that did not happen")
        self.assertIn("clear_fan_confirm_app_link(&_app).map_err", command)
        # Native first: clearing the Rust slot before the native holder would
        # leave the panel closed while the next resume tick can still re-offer.
        self.assertLess(
            command.index("clear_fan_confirm_app_link"),
            command.index("pending_fan_confirm_token"),
        )

    def test_the_native_clear_verifies_itself_before_resolving(self) -> None:
        template = KOTLIN_TEMPLATE.read_text(encoding="utf-8")
        body = template.split("fun clearFanConfirmAppLink(invoke: Invoke) {", 1)[1].split(
            "\n    }", 1
        )[0]
        self.assertIn("pendingFanConfirmAppLink = null", body)
        self.assertIn("activity.intent.data = null", body)
        self.assertIn("invoke.reject(", body)
        self.assertLess(body.index("invoke.reject("), body.index("invoke.resolve()"))

    def test_the_panel_closes_only_on_a_confirmed_clear(self) -> None:
        ui = read("src/app/fan.rs")
        handler = ui.split('"fan_clear_pending_confirm_link"', 1)[1][:600]
        self.assertIn("Ok(()) => { let _ = link_pending.try_set(false); }", handler)
        self.assertIn("Err(message)", handler)
        # The optimistic pre-close is what made the failure invisible.
        self.assertNotIn(
            "link_pending.set(false);\n                                spawn_local", ui
        )


class DeviceSecretCapabilityTruth(unittest.TestCase):
    def test_a_refused_first_seal_never_leaves_an_unopenable_vault(self) -> None:
        helper = item_body(
            read("src-tauri/src/commands/fan/device_unlock.rs"),
            "async fn seal_or_discard_fan_vault(",
        )
        self.assertIn("crate::device_unlock::seal(", helper)
        self.assertIn("vault::remove_fan(&app_data_dir)", helper)
        self.assertIn("forget_device_unlock(state, app)", helper)
        self.assertIn("Err(error)", helper)

    def test_every_vault_creating_seal_goes_through_the_discard_helper(self) -> None:
        for module in (
            "src-tauri/src/commands/fan/signup.rs",
            "src-tauri/src/commands/fan/session_commerce.rs",
        ):
            source = read(module)
            self.assertIn("seal_or_discard_fan_vault(", source, module)
            self.assertNotIn(
                "device_unlock::seal(",
                source,
                f"{module} seals directly; a refusal there strands the vault it just wrote",
            )
        # `fan_enable_device_unlock` seals a vault that already opens with a
        # PIN, so a refusal there costs nothing and must stay a plain error.
        enable = item_body(
            read("src-tauri/src/commands/fan/device_unlock.rs"),
            "pub(crate) async fn fan_enable_device_unlock(",
        )
        self.assertIn("crate::device_unlock::seal(", enable)
        self.assertNotIn("remove_fan", enable)

    def test_the_capability_claim_is_stated_as_capability(self) -> None:
        for path, needle in (
            (KOTLIN_TEMPLATE, "Capability, not guarantee."),
            (PUSH_BRIDGE, "Not a guarantee that the first seal will succeed."),
        ):
            self.assertIn(needle, path.read_text(encoding="utf-8"), str(path))
        # The probe still must not generate the real key on the startup path.
        template = KOTLIN_TEMPLATE.read_text(encoding="utf-8")
        probe = template.split("fun deviceSecretSupported(invoke: Invoke) {", 1)[1].split(
            "\n    }", 1
        )[0]
        self.assertNotIn("loadOrCreateDeviceKey", probe)
        self.assertIn("KeyGenerator.getInstance", probe)


if __name__ == "__main__":
    unittest.main()
