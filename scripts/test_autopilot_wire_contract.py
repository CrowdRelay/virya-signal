#!/usr/bin/env python3
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class AutopilotWireContract(unittest.TestCase):
    def test_native_client_accepts_legacy_time_tuples_but_keeps_typed_models(self):
        api = (ROOT / "src-tauri/src/api/operator.rs").read_text()
        self.assertIn("legacy_offset_datetime_tuple", api)
        self.assertIn("normalize_legacy_autopilot_dates", api)
        self.assertIn("decode_autopilot_wire", api)
        self.assertGreaterEqual(api.count("decode_autopilot_wire(value)"), 2)
        self.assertIn("time::format_description::well_known::Rfc3339", api)
        for field in ("guarded_until", "created_at", "due_at", "starts_at", "release_at"):
            self.assertIn(f'"{field}"', api)

    def test_staff_settings_does_not_retry_failed_control_plane_reads_forever(self):
        settings = (ROOT / "src/app/operator/commerce_settings.rs").read_text()
        support = (ROOT / "src/app/support.rs").read_text()
        fan_events = (ROOT / "src/app/fan/events.rs").read_text()
        self.assertIn("owner_control_plane_requested", settings)
        self.assertIn("get_untracked()", settings)
        self.assertNotIn("Ok(None) => return", support)
        self.assertNotIn("Ok(None) => return", fan_events)
        ops = support.split("fn refresh_operator_ops", 1)[1].split("fn refresh_fan_home", 1)[0]
        self.assertIn("if loading.get_untracked()", ops)

    def test_signal_authority_and_action_contract_match_new_growth_contexts(self):
        api = (ROOT / "src-tauri/src/api/operator.rs").read_text()
        contract = (ROOT / "crates/virya-signal-contracts/src/autopilot.rs").read_text()
        for context in ("beacon", "show_growth"):
            self.assertIn(f'"{context}"', api)
        self.assertIn("SendTeamAssignmentEmail", contract)
        variant = contract.split("SendTeamAssignmentEmail", 1)[1].split("},", 1)[0]
        self.assertNotIn("recipient_email", variant)
        self.assertIn("recipient_name", variant)
        labels = (ROOT / "src/app/operator/autopilot_labels.rs").read_text()
        self.assertIn("AutopilotActionPayload::SendTeamAssignmentEmail", labels)
        self.assertIn('"team.assignment.email"', labels)
        enum_body = contract.split("pub enum AutopilotActionPayload", 1)[1].split("\n}", 1)[0]
        variants = set(re.findall(r"(?m)^    ([A-Z][A-Za-z0-9]+) \{", enum_body))
        detail_body = labels.split("fn autopilot_payload_detail", 1)[1].split("fn autopilot_attention_label", 1)[0]
        handled = set(re.findall(r"AutopilotActionPayload::([A-Z][A-Za-z0-9]+)", detail_body))
        self.assertEqual(handled, variants)

        action_kind_body = contract.split("pub const fn action_kind", 1)[1].split("pub struct TeamAssigneeSummary", 1)[0]
        contract_action_kinds = set(re.findall(r'=> \"([a-z0-9_.]+)\"', action_kind_body))
        label_body = labels.split("fn autopilot_action_kind_label", 1)[1].split("fn autopilot_measurement_kind_label", 1)[0]
        labelled_action_kinds = set(re.findall(r'"([a-z0-9_.]+)" =>', label_body))
        self.assertEqual(labelled_action_kinds, contract_action_kinds)

        # Ecosystem worktrees may include CrowdRelay next to Signal; standalone
        # Signal CI intentionally does not require it. When present, keep the
        # local wire snapshot honest against the backend authority.
        backend_path = ROOT.parent / "crowdrelay/crates/crowdrelay-application/src/autopilot/model.rs"
        if backend_path.exists():
            backend_model = backend_path.read_text()
            backend_body = backend_model.split("pub const fn action_kind", 1)[1].split("/// Action-ready", 1)[0]
            backend_action_kinds = set(re.findall(r'=> "([a-z0-9_.]+)"', backend_body))
            self.assertEqual(contract_action_kinds, backend_action_kinds)


if __name__ == "__main__":
    unittest.main()
