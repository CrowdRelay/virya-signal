/**
 * A fake Tauri bridge, so the real UI can be looked at in a browser.
 *
 * Signal is a Tauri client: `viryaNativeBridgeAvailable()` is just
 * `Boolean(window.__TAURI__?.core?.invoke)`, and every read goes through
 * `core.invoke(command, args)`. Outside the native shell that object does not
 * exist, so the app boots, finds no bridge, and stops at a skeleton. Nothing
 * about the actual screens can be seen or measured that way.
 *
 * This supplies the one seam: `window.__TAURI__.core.invoke`, installed before
 * the WASM module loads. Nothing in `src/` changes, and this file never ships —
 * it lives under `scripts/` and is loaded only by the preview harness.
 *
 * Fixtures are deliberately plausible rather than minimal. A screen rendered
 * with one event and an empty wallet tells you almost nothing about spacing,
 * truncation or rhythm, which is what a UI pass is for.
 *
 * `VIRYA_PREVIEW.mode` selects who is signed in. Set it before load:
 *   fan-out | fan | beacon | staff | owner
 */
(() => {
  const params = new URLSearchParams(location.search);
  const mode = params.get("mode") || "fan";

  const log = [];
  window.VIRYA_PREVIEW = { mode, log, missing: new Set() };

  const iso = (offsetDays) =>
    new Date(Date.now() + offsetDays * 86_400_000).toISOString();

  // ── session shapes ────────────────────────────────────────────────────
  const operatorSession = (role) => ({
    configured: true,
    unlocked: true,
    session: {
      display_name: role === "owner" ? "Wojciech" : "Ola",
      api_base_url: "https://api.virya.music",
      role,
      session_expires_at: Math.floor(Date.now() / 1000) + 3600,
    },
  });

  const fanSession = (signedIn) => ({
    configured: signedIn,
    unlocked: signedIn,
    session: signedIn
      ? {
          email: "fan@example.com",
          display_name: "Kasia",
          wallet_count: 3,
          has_admission_pass: true,
        }
      : null,
  });

  // `BeaconSummary` is display_name only — an extra field is harmless but a
  // missing one is not, and serde reports the first missing field, not all.
  const beaconSession = (signedIn) => ({
    configured: signedIn,
    unlocked: signedIn,
    session: signedIn ? { display_name: "Metal Hammer PL" } : null,
  });

  // Deriving `Default` on the Rust side does not make a field optional for
  // `Deserialize`. `{}` fails with "missing field `configured`", which the app
  // reports as "could not read the fan profile" — an empty object is not an
  // empty session.
  const noOperator = () => ({ configured: false, unlocked: false, session: null });

  const launcher = {
    "fan-out": { operator: noOperator(), fan: fanSession(false), beacon: beaconSession(false) },
    fan: { operator: noOperator(), fan: fanSession(true), beacon: beaconSession(false) },
    beacon: { operator: noOperator(), fan: fanSession(false), beacon: beaconSession(true) },
    staff: { operator: operatorSession("staff"), fan: fanSession(false), beacon: beaconSession(false) },
    owner: { operator: operatorSession("owner"), fan: fanSession(false), beacon: beaconSession(false) },
  };

  // ── content ───────────────────────────────────────────────────────────
  const events = [
    {
      slug: "virya-warszawa-hydrozagadka",
      title: "Virya · Hydrozagadka",
      city: "Warszawa",
      venue: "Hydrozagadka",
      starts_at: iso(12),
      status: "on_sale",
      ticket_url: "https://virya.music/t/wwa",
    },
    {
      slug: "virya-krakow-kwadrat",
      title: "Virya · Klub Kwadrat",
      city: "Kraków",
      venue: "Klub Kwadrat",
      starts_at: iso(26),
      status: "on_sale",
      ticket_url: "https://virya.music/t/krk",
    },
    {
      slug: "virya-wroclaw-alibi",
      title: "Virya · Alibi",
      city: "Wrocław",
      venue: "Alibi",
      starts_at: iso(54),
      status: "announced",
      ticket_url: null,
    },
  ];

  const cities = [
    { slug: "warszawa", name: "Warszawa", country: "PL" },
    { slug: "krakow", name: "Kraków", country: "PL" },
    { slug: "wroclaw", name: "Wrocław", country: "PL" },
  ];

  const fanHome = () => ({
    schema_version: 1,
    generated_at: iso(0),
    profile: { display_name: "Kasia", locale: "pl-PL", primary_city: "Warszawa" },
    next_event: {
      slug: "virya-warszawa-hydrozagadka",
      title: "Virya · Hydrozagadka",
      venue: "Hydrozagadka",
      city: "Warszawa",
      starts_at: iso(12),
      doors_at: iso(12),
      ends_at: null,
      phase: "on_sale",
      ticket_url: "https://virya.music/t/wwa",
      interested: true,
      has_pass: false,
      has_paid_ticket: false,
      ticket_sale_active: true,
    },
    synesthesia: {
      started: true, completed: false, rooms_completed: 3,
      client_total_elapsed_ms: 742_000, best_elapsed_ms: 742_000,
      completed_runs: 0, leaderboard_published: false, leaderboard_rank: null,
      linked_at: iso(-4), reward_entered: false,
    },
    referral: { qualified: 4, pending: 1 },
    counts: { event_interests: 3, active_passes: 1, paid_orders: 2, area_claims: 5 },
    recommended_action: "continue_synesthesia",
    recommended: {
      kind: "continue_synesthesia",
      priority: 1,
      target: "synesthesia",
      expires_at: null,
      reason: "Zacząłeś przejście — zostały trzy pokoje.",
    },
    stale: false,
  });

  // ── the table ─────────────────────────────────────────────────────────
  // Only commands on a screen worth looking at need an entry. Everything else
  // falls through to the default below and is recorded, so a blank panel can be
  // traced to the command that was never answered rather than guessed at.
  const FIXTURES = {
    launcher_status: () => launcher[mode] ?? launcher.fan,
    session_status: () => launcher[mode]?.operator ?? noOperator(),
    fan_status: () => launcher[mode]?.fan ?? fanSession(false),
    beacon_status: () => launcher[mode]?.beacon ?? beaconSession(false),

    public_events: () => ({ events }),
    public_cities: () => ({ cities }),
    fan_events: () => ({ events }),
    fan_cached_events: () => ({ events }),

    // Shapes follow `fan_wire.generated.rs`, which is generated from
    // CrowdRelay's OpenAPI — the authoritative contract, not a guess. Optional
    // fields are `null` rather than absent so a rename shows up as a decode
    // error here instead of a silently empty panel.
    fan_home: () => fanHome(),
    fan_cached_home: () => fanHome(),
    fan_wallets: () => ({ wallets: [] }),
    fan_cached_wallets: () => ({ wallets: [] }),
    fan_cached_sections: () => ({ sections: [] }),
    fan_interests: () => ({ interests: [] }),
    fan_referral: () => ({ qualified: 4, pending: 1 }),
    fan_admission_pass: () => null,
    fan_area_wallet: () => null,
    fan_merch_catalog: () => ({ items: [] }),
    fan_cached_merch_catalog: () => ({ items: [] }),
    fan_merch_bundles: () => ({ bundles: [] }),
    fan_push_sync: () => null,
    fan_push_preferences: () => ({
      shows: true, releases: true, community: false, merch: false,
      quiet_hours_enabled: true, quiet_start: "22:00", quiet_end: "08:00",
      quiet_timezone: "Europe/Warsaw",
    }),

    operator_events: () => ({ events }),
    operator_ops_overview: () => ({}),
    operator_signal_overview: () => ({}),

    // Deep-link polls. These return `bool` and the app retries on a decode
    // failure, so answering `undefined` span the renderer flat out until it
    // stopped compositing. "Nothing pending" is `false`.
    fan_take_synesthesia_app_link: () => false,
    beacon_take_app_link: () => false,
    fan_take_confirm_link: () => false,
    fan_push_take_target: () => null,
    native_crash_report: () => null,

    beacon_home: () => ({}),
    beacon_news: () => ({ items: [] }),
    beacon_releases: () => ({ releases: [] }),
    beacon_press_requests: () => ({ requests: [] }),
  };

  // A floor on every response. Several commands are polled, and a stub that
  // answers in the same tick turns a poll into a busy loop — which is exactly
  // what wedged the first run of this harness. Real IPC is never this fast.
  const tick = () => new Promise((r) => setTimeout(r, 12));

  const invoke = async (command, args) => {
    await tick();
    const fixture = FIXTURES[command];
    log.push({ command, args, answered: Boolean(fixture) });
    if (!fixture) {
      window.VIRYA_PREVIEW.missing.add(command);
      console.warn("[preview] no fixture for", command);
      // `undefined` is what `invoke_latest` reads as "no value", which the app
      // already handles. Throwing here would paint an error screen instead of
      // the screen being reviewed.
      return undefined;
    }
    return fixture(args);
  };

  window.__TAURI__ = { core: { invoke } };
})();
