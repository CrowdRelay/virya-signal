from rust_source_tree import read_rust_module
import hashlib
import unittest
from pathlib import Path

from source_tree import read_app_source


ROOT = Path(__file__).resolve().parents[1]
PROTECTED_COMPAT_PREFIX_SHA256 = "aab72bd3b0d6389069f2723f663d0b2995b52f4458681c202f401cb8da115830"


class TicketCommerceContracts(unittest.TestCase):
    def test_string_or_bytes_compatibility_layer_remains_byte_identical(self):
        source = read_rust_module(ROOT, "src-tauri/src/models.rs").splitlines(keepends=True)
        payload = "".join(source[:234]).encode()
        self.assertEqual(
            hashlib.sha256(payload).hexdigest(),
            PROTECTED_COMPAT_PREFIX_SHA256,
        )

    def test_checkout_secret_never_enters_wasm_models(self):
        frontend_models = (ROOT / "src/models.rs").read_text()
        ticketing = (ROOT / "src-tauri/src/api/ticketing.rs").read_text()
        self.assertNotIn("checkout_token", frontend_models)
        self.assertIn('url.host_str() != Some("checkout.stripe.com")', ticketing)
        self.assertIn("checkout_token.len() != 64", ticketing)

    def test_wallet_credential_is_saved_before_checkout_returns(self):
        commands = read_rust_module(ROOT, "src-tauri/src/commands/fan.rs")
        function = commands.split("pub(crate) async fn fan_start_ticket_checkout", 1)[1]
        function = function.split("#[tauri::command]", 1)[0]
        self.assertLess(
            function.index("persist_fan(&state, &profile).await?;"),
            function.index("Ok(checkout)"),
        )
        self.assertIn("Zeroizing::new(response.checkout_token)", function)

    def test_ticket_and_merch_are_first_class_fan_actions(self):
        ui = read_app_source(ROOT)
        self.assertIn('own=FanTab::Merch icon="shop" label=tr("store_tab")', ui)
        self.assertIn('TicketPoolAvailability::Available', ui)
        self.assertIn('class="ticket-buy-button"', ui)
        self.assertIn('this_show_has_no_ticket_pool', ui)
        self.assertIn("FanTicketSale", ui)
        self.assertIn('buy_in_store', ui)
        self.assertIn("fan_merch_bundles", ui)
        self.assertIn("bundles_from_the_online_store", ui)

    def test_ticket_quantity_controls_use_boolean_signals_not_raw_or_chains(self):
        ui = read_app_source(ROOT)
        self.assertIn("let increment_disabled = Signal::derive", ui)
        self.assertIn("disabled=move || increment_disabled.get()", ui)
        self.assertIn("disabled=move || decrement_disabled.get()", ui)
        self.assertIn("disabled=move || purchase_disabled.get()", ui)
        self.assertNotIn(
            "disabled=move || quantity.get() >= available || selected_count.get()",
            ui,
        )

    def test_ticket_quantity_controls_meet_mobile_touch_target(self):
        styles = (ROOT / "styles.css").read_text()
        self.assertIn("grid-template-columns: 44px 40px 44px", styles)
        self.assertIn(".ticket-stepper button { width: 44px; height: 44px;", styles)

    def test_checkout_uses_existing_bounded_first_party_flow(self):
        ticketing = (ROOT / "src-tauri/src/api/ticketing.rs").read_text()
        self.assertIn('https://virya.music/api/ticket-checkout', ticketing)
        self.assertIn('.header(ORIGIN, VIRYA_SITE_ORIGIN)', ticketing)
        self.assertIn('MAX_CHECKOUT_LINES: usize = 10', ticketing)
        self.assertIn('MAX_CHECKOUT_QUANTITY: u32 = 100', ticketing)
        self.assertIn('public/events/{event_slug}/tickets', ticketing)


if __name__ == "__main__":
    unittest.main()
