#!/usr/bin/env python3
"""Deterministic local CrowdRelay/staff-gate double for Android black-box E2E.

Only intended for VIRYA_SIGNAL_E2E_API_BASE debug builds. It never talks to production.
An optional sentinel file makes owner analytics endpoints fail with 503 so the encrypted
last-known-good cache/reconnect path can be exercised on a real Android emulator.
"""
from __future__ import annotations

import argparse
import json
import os
import re
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

TOKEN = "e2e_fan_session_" + "a" * 48
EVENT_SLUG = "virya-e2e-test-event"
EVENT = {
    "slug": EVENT_SLUG,
    "title": "VIRYA E2E — TEST EVENT",
    "description": "Synthetic event for Virya Signal Android regression tests.",
    "city": {"name": "Wrocław"},
    "venue": "E2E Test Venue",
    "starts_at": "2027-02-14T18:00:00Z",
    "ticket_url": "https://virya.music/e2e/tickets",
    "image_url": None,
    "image_thumbnail_url": None,
}


def json_bytes(value: object) -> bytes:
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


class State:
    offline_file: Path | None = None


class Handler(BaseHTTPRequestHandler):
    server_version = "ViryaSignalE2E/1"

    def log_message(self, fmt: str, *args: object) -> None:
        print(f"[signal-e2e-mock] {self.command} {self.path} :: {fmt % args}", flush=True)

    def _reply(self, status: int, payload: object | None = None) -> None:
        body = b"" if payload is None else json_bytes(payload)
        self.send_response(status)
        if payload is not None:
            self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if body:
            self.wfile.write(body)

    def _body(self) -> object:
        length = int(self.headers.get("Content-Length", "0") or "0")
        if not length:
            return {}
        raw = self.rfile.read(length)
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            return {}

    def _offline_owner(self, path: str) -> bool:
        return bool(
            State.offline_file
            and State.offline_file.exists()
            and path in {
                "/v1/admin/signal/overview",
                "/v1/admin/audience/overview",
                "/v1/admin/analytics/revenue",
            }
        )

    def do_HEAD(self) -> None:  # noqa: N802
        self._reply(HTTPStatus.OK)

    def do_GET(self) -> None:  # noqa: N802
        path = urlparse(self.path).path
        if self._offline_owner(path):
            self._reply(HTTPStatus.SERVICE_UNAVAILABLE, {"error": "synthetic_offline"})
            return
        if path == "/healthz":
            self._reply(HTTPStatus.OK, {"ok": True})
        elif path == "/v1/meta":
            self._reply(HTTPStatus.OK, {
                "apiVersion": "1",
                "schemaVersion": 61,
                "release": "virya-signal-e2e",
                "gitSha": "e2e0000000000000000000000000000000000000",
                "buildTimestamp": "2026-08-17T00:00:00Z",
                "minimumPostgresServerVersionNum": 180000,
                "capabilities": {
                    "signal_fan_context_v1": True,
                    "ticketing_v1": True,
                    "area_wallet_postgres_v2": True,
                    "staff_show_checklist_v1": True,
                    "push_delivery_v1": True,
                },
            })
        elif path == "/v1/me/home":
            self._reply(HTTPStatus.OK, {
                "schema_version": 1,
                "generated_at": "2026-08-17T00:00:00Z",
                "profile": {"display_name": "E2E Fan", "locale": "pl", "primary_city": "Wrocław"},
                "next_event": {
                    "slug": EVENT_SLUG,
                    "title": EVENT["title"],
                    "venue": EVENT["venue"],
                    "city": "Wrocław",
                    "starts_at": EVENT["starts_at"],
                    "doors_at": None,
                    "ends_at": None,
                    "phase": "upcoming",
                    "ticket_url": EVENT["ticket_url"],
                    "interested": False,
                    "has_pass": False,
                    "has_paid_ticket": False,
                    "ticket_sale_active": True,
                },
                "synesthesia": {
                    "started": False,
                    "completed": False,
                    "rooms_completed": 0,
                    "client_total_elapsed_ms": None,
                    "best_elapsed_ms": None,
                    "completed_runs": 0,
                    "leaderboard_published": False,
                    "leaderboard_rank": None,
                    "linked_at": None,
                    "reward_entered": False,
                },
                "referral": {"qualified": 0, "pending": 0},
                "counts": {"event_interests": 0, "active_passes": 0, "paid_orders": 0, "area_claims": 0},
                "recommended_action": "next_event",
                "stale": False,
            })
        elif path == "/v1/public/events":
            self._reply(HTTPStatus.OK, {"events": [EVENT]})
        elif path == "/v1/public/cities":
            self._reply(HTTPStatus.OK, {"items": [{"slug": "wroclaw", "name": "Wrocław", "country_code": "PL", "fan_count": 7}]})
        elif path == "/v1/public/push/config":
            self._reply(HTTPStatus.OK, {"enabled": True, "android_fcm": True})
        elif path == "/v1/me/referral":
            self._reply(HTTPStatus.OK, {"referral_code": "E2E", "qualified_referrals": 0, "pending_referrals": 0, "draw_entries": [], "coupons": [], "physical_rewards": []})
        elif path == "/v1/me/events":
            self._reply(HTTPStatus.OK, [])
        elif path == "/v1/me/passes/admission":
            self._reply(HTTPStatus.OK, None)
        elif path == "/v1/staff/event-qr/overview":
            self._reply(HTTPStatus.OK, {"events": [{"slug": EVENT_SLUG, "title": EVENT["title"]}], "campaigns": []})
        elif path == "/v1/admin/signal/overview":
            self._reply(HTTPStatus.OK, {
                "generated_at": "2026-08-17T00:00:00Z",
                "summary": {"total_fans": 9, "active_fans": 7, "pending_fans": 1, "unsubscribed_fans": 1, "suppressed_fans": 0, "marketing_opted_in": 8, "nearby_enabled": 7},
                "activity": {"new_fans_7d": 2, "new_fans_30d": 5, "referral_attributions_total": 3, "referral_attributions_30d": 1, "event_interests_total": 8, "event_interests_30d": 4, "nearby_notifications_30d": 2, "pending_city_requests": 0},
                "top_cities": [{"name": "Wrocław", "country_code": "PL", "active_fans": 7}],
                "audience": {}, "ticket_revenue": [], "unavailable_sources": [],
            })
        elif path == "/v1/admin/audience/overview":
            self._reply(HTTPStatus.OK, {"active_fans": 7, "marketing_consented_fans": 8, "ticket_buyers": 2, "attendees": 2, "synesthesia_participants": 3, "qualified_referrals": 3, "paid_ticket_orders": 2})
        elif path == "/v1/admin/analytics/revenue":
            self._reply(HTTPStatus.OK, [{"currency": "PLN", "paid_orders": 2, "gross_paid_minor": 6000, "refunded_minor": 0, "after_refunds_minor": 6000}])
        elif path == "/v1/admin/ops/overview":
            self._reply(HTTPStatus.OK, {})
        elif re.fullmatch(r"/v1/staff/ecosystem/checklists/[A-Za-z0-9_-]+", path):
            self._reply(HTTPStatus.OK, {
                "event_id": "e2e-event-id",
                "event_slug": EVENT_SLUG,
                "event_title": EVENT["title"],
                "starts_at": EVENT["starts_at"],
                "items": [{"item_key": "setlist_ready", "section": "show_files", "sort_order": 1, "status": "pending", "note": None, "updated_at": "2026-08-17T00:00:00Z"}],
            })
        else:
            # Empty but valid defaults keep unrelated dashboard widgets from making
            # the black-box journey fail before it reaches the feature under test.
            self._reply(HTTPStatus.OK, {})

    def do_POST(self) -> None:  # noqa: N802
        path = urlparse(self.path).path
        body = self._body()
        if path == "/staff-gate":
            self._reply(HTTPStatus.NO_CONTENT)
        elif path == "/v1/fans/confirm":
            self._reply(HTTPStatus.OK, {"fan_session_token": TOKEN})
        elif path == "/v1/fans/access":
            self._reply(HTTPStatus.OK, {"accepted": True})
        elif path == "/v1/me/synesthesia/link":
            handoff = str(body.get("handoff_code", "")) if isinstance(body, dict) else ""
            if len(handoff) == 64 and all(c in "0123456789abcdef" for c in handoff.lower()):
                self._reply(HTTPStatus.OK, {"linked": True})
            else:
                self._reply(HTTPStatus.UNPROCESSABLE_ENTITY, {"error": "invalid_handoff"})
        elif path in {"/v1/me/push/register", "/v1/me/push/unregister", "/v1/staff/push/register", "/v1/staff/push/unregister"}:
            self._reply(HTTPStatus.OK, {"registered": True})
        else:
            self._reply(HTTPStatus.OK, {})

    def do_PATCH(self) -> None:  # noqa: N802
        self._body()
        self._reply(HTTPStatus.OK, {})

    def do_DELETE(self) -> None:  # noqa: N802
        self._reply(HTTPStatus.OK, {})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8787)
    parser.add_argument("--offline-file")
    args = parser.parse_args()
    State.offline_file = Path(args.offline_file).resolve() if args.offline_file else None
    server = ThreadingHTTPServer((args.host, args.port), Handler)
    print(f"SIGNAL_E2E_MOCK=READY host={args.host} port={args.port} offline_file={State.offline_file}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
