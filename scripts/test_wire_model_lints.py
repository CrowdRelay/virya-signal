#!/usr/bin/env python3
"""Regression tests for intentionally retained API wire fields."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
MODELS = ROOT / "src" / "models.rs"

PARTIAL_WIRE_FIELDS = {
    "AreaWallet": ("token_balance",),
    "AreaCommunity": ("current", "total"),
    "AreaClaim": ("claimed_at",),
    "AreaVoucher": ("code", "tokens", "status", "expires_at", "free_product_label"),
    "AreaLiveDrop": ("id",),
    "QueueSummary": ("processing", "delivered_24h", "oldest_pending_seconds"),
    "OpsOutboxItem": ("status", "dead_at"),
    "OpsDeliveryItem": ("status", "last_response_status", "dead_at"),
    "OpsRetryResult": ("operation_id", "target_type", "target_id", "status"),
    "ShowModeStatus": ("event_slug", "event_title", "expires_at", "synced"),
    "ShowModeScanResult": ("accepted", "state"),
    "ShowModeSyncResult": ("attempted",),
}


class WireModelLintTests(unittest.TestCase):
    def test_dead_code_is_not_disabled_broadly(self) -> None:
        source = MODELS.read_text()
        self.assertNotIn("#![allow(dead_code)]", source)
        self.assertNotRegex(source, r"#\[allow\(dead_code\)\]\s*#\[derive")

    def test_only_intentionally_retained_wire_fields_are_annotated(self) -> None:
        source = MODELS.read_text()
        expected = sum(len(fields) for fields in PARTIAL_WIRE_FIELDS.values())
        self.assertEqual(source.count("#[allow(dead_code)]"), expected)
        for struct, fields in PARTIAL_WIRE_FIELDS.items():
            start = source.index(f"pub struct {struct} {{")
            end = source.index("\n}", start)
            block = source[start:end]
            for field in fields:
                pattern = re.compile(
                    r"#\[allow\(dead_code\)\]\s*"
                    r"(?:#\[[^\n]+\]\s*)*"
                    rf"pub {re.escape(field)}\s*:"
                )
                self.assertRegex(block, pattern, f"{struct}.{field}")


if __name__ == "__main__":
    unittest.main()
