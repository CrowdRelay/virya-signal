from rust_source_tree import read_rust_module
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class AudienceOverviewContract(unittest.TestCase):
    def test_models_keep_native_ipc_parity(self):
        web = (ROOT / "src/models.rs").read_text()
        native = read_rust_module(ROOT, "src-tauri/src/models.rs")
        for marker in ["pub struct AudienceSummary", "pub struct AudienceRevenueSummary", "pub ticket_revenue: Vec<AudienceRevenueSummary>"]:
            self.assertIn(marker, web)
            self.assertIn(marker, native)

    def test_operator_signal_fans_out_in_parallel_and_degrades_optional_sources(self):
        api = (ROOT / "src-tauri/src/api/operator.rs").read_text()
        self.assertIn("join3(signal_request, audience_request, revenue_request)", api)
        self.assertIn('"admin/audience/overview"', api)
        self.assertIn('"admin/analytics/revenue"', api)
        self.assertIn('unavailable_sources.push("audience"', api)
        self.assertIn('unavailable_sources.push("revenue"', api)
        self.assertIn("let mut overview = signal_result?;", api)

    def test_staff_signal_renders_aggregate_only_audience_metrics(self):
        ui = read_rust_module(ROOT, "src/app/operator.rs")
        for marker in ["audience.ticket_buyers", "audience.attendees", "audience.synesthesia_participants", "audience.qualified_referrals", "ticket_revenue"]:
            self.assertIn(marker, ui)
        self.assertNotIn("AudienceFanDetail", ui)

    def test_i18n_has_new_labels_in_both_languages(self):
        pl = (ROOT / "src/i18n/pl.rs").read_text()
        en = (ROOT / "src/i18n/en.rs").read_text()
        for key in ["audience_intelligence", "ticket_buyers", "concert_attendees", "synesthesia_participants", "qualified_referrals", "ticket_revenue"]:
            self.assertIn(f'"{key}"', pl)
            self.assertIn(f'"{key}"', en)

    def test_revenue_rows_require_internal_accounting_invariants(self):
        source = (ROOT / "src-tauri/src/api/operator.rs").read_text()
        self.assertIn("row.refunded_minor <= row.gross_paid_minor", source)
        self.assertIn("row.gross_paid_minor - row.refunded_minor", source)

if __name__ == "__main__":
    unittest.main()
