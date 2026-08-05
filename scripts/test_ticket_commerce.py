import hashlib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PROTECTED_MODELS_SHA256 = "e8f988fce76de6619a4afbb9193c738612113ae4037f1ac4ba2361c658aaf52f"


class TicketCommerceContracts(unittest.TestCase):
    def test_protected_native_models_remains_byte_identical(self):
        payload = (ROOT / "src-tauri/src/models.rs").read_bytes()
        self.assertEqual(hashlib.sha256(payload).hexdigest(), PROTECTED_MODELS_SHA256)

    def test_checkout_secret_never_enters_wasm_models(self):
        frontend_models = (ROOT / "src/models.rs").read_text()
        ticketing = (ROOT / "src-tauri/src/api/ticketing.rs").read_text()
        self.assertNotIn("checkout_token", frontend_models)
        self.assertIn('url.host_str() != Some("checkout.stripe.com")', ticketing)
        self.assertIn("checkout_token.len() != 64", ticketing)

    def test_wallet_credential_is_saved_before_checkout_returns(self):
        commands = (ROOT / "src-tauri/src/commands/fan.rs").read_text()
        function = commands.split("pub(crate) async fn fan_start_ticket_checkout", 1)[1]
        function = function.split("#[tauri::command]", 1)[0]
        self.assertLess(
            function.index("persist_fan(&state, &profile).await?;"),
            function.index("Ok(checkout)"),
        )
        self.assertIn("Zeroizing::new(response.checkout_token)", function)

    def test_ticket_and_merch_are_first_class_fan_actions(self):
        ui = (ROOT / "src/app/mod.rs").read_text()
        self.assertIn('own=FanTab::Merch icon="shop" label="Sklep"', ui)
        self.assertIn('class="ticket-buy-button" on:click=buy>"KUP BILET"', ui)
        self.assertIn("FanTicketSale", ui)
        self.assertIn("KUP W SKLEPIE ↗", ui)
        self.assertIn("fan_merch_bundles", ui)
        self.assertIn("Bundle ze sklepu online", ui)

    def test_checkout_uses_existing_bounded_first_party_flow(self):
        ticketing = (ROOT / "src-tauri/src/api/ticketing.rs").read_text()
        self.assertIn('https://virya.music/api/ticket-checkout', ticketing)
        self.assertIn('.header(ORIGIN, VIRYA_SITE_ORIGIN)', ticketing)
        self.assertIn('MAX_CHECKOUT_LINES: usize = 10', ticketing)
        self.assertIn('MAX_CHECKOUT_QUANTITY: u32 = 100', ticketing)
        self.assertIn('public/events/{event_slug}/tickets', ticketing)


if __name__ == "__main__":
    unittest.main()
