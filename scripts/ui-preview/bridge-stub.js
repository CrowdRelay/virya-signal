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
 *   fan-out | fan-locked | fan | beacon | staff | owner
 */
(() => {
  const params = new URLSearchParams(location.search);
  const mode = params.get("mode") || "fan";

  const log = [];
  window.VIRYA_PREVIEW = { mode, log, missing: new Set() };

  // The root mode the app boots into is read from web storage, not from any
  // command, so a `?mode=` that only swapped the session fixtures still landed
  // on the fan portal and staff/owner were unreachable in the preview. Seed the
  // same keys `src/bridge/navigation.rs` reads. Team is deliberately the
  // transient sessionStorage key there, so it is seeded there too.
  try {
    if (mode === "staff" || mode === "owner") {
      window.sessionStorage.setItem("virya:root-mode-transient:v1", "team");
    } else {
      window.sessionStorage.removeItem("virya:root-mode-transient:v1");
      window.localStorage.setItem(
        "virya:root-mode:v1",
        mode === "beacon" ? "latarnik" : "fan",
      );
    }
  } catch {}

  // Counts how often a card subtree is torn down and rebuilt. Reading a screen
  // that is being reconstructed on every arriving data source is a different
  // problem from a screen that is merely slow, and the two look identical in a
  // screenshot. `VIRYA_PREVIEW.churn` reports it.
  const churn = { added: 0, removed: 0 };
  window.VIRYA_PREVIEW.churn = churn;
  new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node.nodeType === 1 && node.matches?.("article")) churn.added += 1;
      }
      for (const node of record.removedNodes) {
        if (node.nodeType === 1 && node.matches?.("article")) churn.removed += 1;
      }
    }
  }).observe(document.documentElement, { childList: true, subtree: true });

  const iso = (offsetDays) =>
    new Date(Date.now() + offsetDays * 86_400_000).toISOString();

  /** Doors open before the set. Equal timestamps are a fixture smell, not data. */
  const hoursBefore = (isoString, hours) =>
    new Date(Date.parse(isoString) - hours * 3_600_000).toISOString();

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
    pin_unlock: !deviceUnlock,
    device_unlock: deviceUnlock,
    device_unlock_supported: true,
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

  // A fan whose vault exists but is locked. Neither `fan` nor `fan-out`
  // reaches the PIN prompt or the "a login link is waiting" panel, which are
  // the two screens every returning fan sees before anything else.
  // `?unlock=device` models a vault the keystore holds the password for: no
  // PIN behind it, so the gate must open on its own rather than showing a
  // field that cannot work. `?unlock=pin` is the ordinary case.
  const deviceUnlock = params.get("unlock") === "device";
  const lockedFanSession = () => ({
    configured: true,
    unlocked: false,
    session: null,
    pin_unlock: !deviceUnlock,
    device_unlock: deviceUnlock,
    device_unlock_supported: true,
  });

  // Session state the preview mutates, so an unlock or a confirmation behaves
  // like one: it persists for the rest of the page's life.
  let fanUnlocked = mode === "fan";
  const unlockFan = () => {
    fanUnlocked = true;
    return fanSession(true);
  };

  const launcher = {
    "fan-out": { operator: noOperator(), fan: fanSession(false), beacon: beaconSession(false) },
    "fan-locked": { operator: noOperator(), fan: lockedFanSession(), beacon: beaconSession(false) },
    fan: { operator: noOperator(), fan: fanSession(true), beacon: beaconSession(false) },
    beacon: { operator: noOperator(), fan: fanSession(false), beacon: beaconSession(true) },
    staff: { operator: operatorSession("staff"), fan: fanSession(false), beacon: beaconSession(false) },
    owner: { operator: operatorSession("owner"), fan: fanSession(false), beacon: beaconSession(false) },
  };

  // ── content ───────────────────────────────────────────────────────────
  // `PublicEvent` shape, exactly: `city` is a struct with `name`, not a
  // string, and `description`/`image_url` carry no serde default. The earlier
  // fixture was a loose object wrapped in `{ events }`, so every command that
  // decodes `Vec<PublicEvent>` failed and every event list in the app — fan
  // and staff — rendered empty. An empty list looks like "no shows booked",
  // which is why this went unnoticed.
  const events = [
    {
      slug: "virya-warszawa-hydrozagadka",
      title: "Virya · Hydrozagadka",
      description: "Trasa Wolne Miasto. Support: Nocny Kurs.",
      city: { name: "Warszawa" },
      venue: "Hydrozagadka",
      starts_at: iso(12),
      ticket_url: "https://virya.music/t/wwa",
      image_url: null,
      image_thumbnail_url: null,
    },
    {
      slug: "virya-krakow-kwadrat",
      title: "Virya · Klub Kwadrat",
      description: "Trasa Wolne Miasto.",
      city: { name: "Kraków" },
      venue: "Klub Kwadrat",
      starts_at: iso(26),
      ticket_url: "https://virya.music/t/krk",
      image_url: null,
      image_thumbnail_url: null,
    },
    {
      slug: "virya-wroclaw-alibi",
      title: "Virya · Alibi",
      description: null,
      city: { name: "Wrocław" },
      venue: "Alibi",
      starts_at: iso(54),
      ticket_url: null,
      image_url: null,
      image_thumbnail_url: null,
    },
  ];

  const cities = [
    { slug: "warszawa", name: "Warszawa", country: "PL" },
    { slug: "krakow", name: "Kraków", country: "PL" },
    { slug: "wroclaw", name: "Wrocław", country: "PL" },
  ];

  // FanPushStatus is `rename_all = "camelCase"` on the contract side. Most
  // other payloads here are snake_case; this one is not, and snake_case keys
  // fail decode with `missing field \`backendEnabled\``.
  // `?push=prompt` models a fan Android has not asked yet, which is the only
  // state the notification primer appears in. The default stays "already on",
  // because that is what every other screen should be reviewed against.
  const pushUnanswered = params.get("push") === "prompt";
  const pushStatus = () => ({
    supported: true,
    backendEnabled: true,
    enabled: !pushUnanswered,
    permission: pushUnanswered ? "prompt" : "granted",
    transport: "fcm",
    detail: null,
  });

  const ticketSaleOffer = (slug) => {
    const event = events.find((candidate) => candidate.slug === slug);
    return {
      event_id: `evt-${slug}`,
      event_slug: slug,
      event_title: event?.title ?? "Virya",
      event_status: "upcoming",
      venue: event?.venue ?? null,
      timezone: "Europe/Warsaw",
      starts_at: event?.starts_at ?? iso(12),
      currency: "PLN",
      vat_rate_basis_points: 2300,
      capacity: 400,
      sold: 268,
      reserved: 12,
      available: 120,
      max_per_order: 6,
      sales_open_at: iso(-20),
      sales_close_at: event?.starts_at ?? iso(12),
      active: true,
      sales_state: "open",
      ticket_types: [
        {
          id: "tt-normal", slug: "normalny", name: "Bilet normalny",
          description: "Wejście na koncert.", price_gross_minor: 9900,
          capacity: 320, sold: 210, reserved: 10, available: 100,
          sort_order: 0, active: true,
        },
        {
          id: "tt-premium", slug: "premium", name: "Premium + soundcheck",
          description: "Wcześniejsze wejście i soundcheck.",
          price_gross_minor: 19900,
          capacity: 40, sold: 40, reserved: 0, available: 0,
          sort_order: 1, active: true,
        },
      ],
    };
  };

  const merchCatalog = () => ({
    products: [
      {
        slug: "tshirt-wolne-miasto",
        name: "T-shirt Wolne Miasto",
        description: "Czarny, bawełna organiczna, nadruk z trasy.",
        image_url: null,
        placeholder_image_url: null,
        currency: "PLN",
        price_gross_minor: 12900,
        active: true,
        public: true,
        variants: [
          { sku: "TS-S", label: "S", active: true, available: true, availability: "available" },
          { sku: "TS-M", label: "M", active: true, available: true, availability: "available" },
          { sku: "TS-L", label: "L", active: true, available: false, availability: "out_of_stock" },
        ],
      },
      {
        slug: "vinyl-wolne-miasto",
        name: "Wolne Miasto — winyl",
        description: "180 g, wkładka z tekstami.",
        image_url: null,
        placeholder_image_url: null,
        currency: "PLN",
        price_gross_minor: 15900,
        active: true,
        public: true,
        variants: [],
      },
      {
        slug: "tote-signal",
        name: "Torba Signal",
        description: null,
        image_url: null,
        placeholder_image_url: null,
        currency: "PLN",
        price_gross_minor: 6900,
        active: true,
        public: true,
        variants: [],
      },
    ],
  });

  const merchBundles = () => ({
    bundles: [
      {
        slug: "zestaw-trasa",
        name: "Zestaw trasowy",
        description: "T-shirt i winyl w jednej paczce.",
        includes: ["T-shirt Wolne Miasto", "Wolne Miasto — winyl"],
        image_url: null,
        placeholder_image_url: null,
        secondary_image_url: null,
        product_url: "https://virya.music/sklep/zestaw-trasa",
        currency: "PLN",
        price_gross_minor: 24900,
        original_price_gross_minor: 28800,
        available: true,
        availability: "available",
        variants: [
          { label: "S", available: true, availability: "available" },
          { label: "M", available: true, availability: "available" },
          { label: "L", available: false, availability: "out_of_stock" },
        ],
      },
    ],
  });

  // One order, two tickets: one with a live QR and one already redeemed, so the
  // scanned and unscanned rows can be compared side by side.
  const walletBatch = () => ({
    wallets: [
      {
        order: {
          order_id: "ord-8842",
          public_reference: "VRY-8842",
          event_title: "Virya · Hydrozagadka",
          venue: "Hydrozagadka",
          starts_at: iso(12),
          status: "paid",
        },
        tickets: [
          {
            ticket_type_name: "Bilet normalny",
            public_reference: "VRY-8842-1",
            holder_name: "Kasia",
            holder_email_masked: "k***@example.com",
            status: "valid",
            redeemed_at: null,
            qr_available: true,
            qr_expires_at: iso(12),
          },
          {
            ticket_type_name: "Bilet normalny",
            public_reference: "VRY-8842-2",
            holder_name: null,
            holder_email_masked: "k***@example.com",
            status: "redeemed",
            redeemed_at: iso(-1),
            qr_available: false,
            qr_expires_at: iso(12),
          },
        ],
        cached: false,
      },
    ],
    failed_count: 0,
    cached_count: 0,
  });

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
      doors_at: hoursBefore(iso(12), 1),
      ends_at: null,
      phase: "upcoming",
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
    // Unlocking has to stick. `fan_unlock` answered with an unlocked session
    // while `fan_status` kept answering from the fixture, so the app mounted
    // the fan shell and the next status read threw it straight back to the PIN
    // prompt — an unlock loop no device has.
    launcher_status: () => (fanUnlocked
      ? { ...(launcher[mode] ?? launcher.fan), fan: fanSession(true) }
      : (launcher[mode] ?? launcher.fan)),
    session_status: () => launcher[mode]?.operator ?? noOperator(),
    fan_status: () => (fanUnlocked ? fanSession(true) : (launcher[mode]?.fan ?? fanSession(false))),
    beacon_status: () => launcher[mode]?.beacon ?? beaconSession(false),

    public_events: () => events,
    public_cities: () => ({ cities }),
    fan_events: () => events,
    fan_cached_events: () => events,

    // Shapes follow `fan_wire.generated.rs`, which is generated from
    // CrowdRelay's OpenAPI — the authoritative contract, not a guess. Optional
    // fields are `null` rather than absent so a rename shows up as a decode
    // error here instead of a silently empty panel.
    fan_home: () => fanHome(),
    fan_cached_home: () => fanHome(),
    fan_wallets: () => walletBatch(),
    fan_cached_wallets: () => walletBatch(),
    fan_cached_sections: () => ({ sections: [] }),
    fan_interests: () => ({ interests: [] }),
    // `ReferralProgress`, exactly: `referral_code` carries no serde default, so
    // the old `{ qualified, pending }` object failed to decode and the referral
    // block rendered with a blank code and an empty draws list — which reads as
    // "this fan has no code" rather than "the fixture is wrong shape".
    fan_referral: () => ({
      // Deliberately long and hyphen-free. A code with hyphens has break
      // opportunities and hides the case that actually broke: a single
      // unbreakable token setting the copy button's min-content width and
      // pushing itself through the right border of the block.
      referral_code: "KASIAMETALOWASIODEMKA7QX2XL",
      qualified_referrals: 4,
      pending_referrals: 1,
      draw_entries: [
        {
          slug: "wejsciowka-hydrozagadka",
          name: "Wejściówka na Hydrozagadkę",
          prize_kind: "Wejściówka",
          draw_at: "2026-09-14T18:00:00Z",
          total_entries: 128,
        },
      ],
      coupons: [{ code: "VIRYA-10", discount_percent: 10, status: "aktywny" }],
      physical_rewards: [
        { item_name: "Koszulka Virya", sku: "TS-BLK-L", status: "wysłana" },
      ],
    }),
    fan_admission_pass: () => null,
    // `AreaWallet` is camelCase. `null` renders the AREA outage state, which
    // is not a screen worth reviewing. Three drops: one claimable, one already
    // claimed, one full.
    fan_area_wallet: () => ({
      tokenBalance: 3,
      rewardCredits: 2,
      collectionSize: 1,
      community: { current: 412, total: 1000, percent: 41.2 },
      claims: [
        {
          dropId: "area-wwa-01",
          number: "001",
          city: "Warszawa",
          line: "Wolne Miasto",
          track: "Nocny Kurs",
          edition: "I",
          claimedAt: iso(-3),
          distanceMeters: 42,
          editionNumber: 12,
        },
      ],
      vouchers: [],
      liveDrops: [{ id: "area-wwa-02" }],
      drops: [
        {
          id: "area-wwa-01",
          number: "001",
          city: "Warszawa",
          region: "Mazowieckie",
          signalCitySlug: "warszawa",
          mapX: 120,
          mapY: 210,
          approximateLat: 52.2297,
          approximateLng: 21.0122,
          clue: { pl: "Przy murze, gdzie gra echo.", en: "By the wall where the echo plays." },
          active: true,
          full: false,
          claimed: true,
        },
        {
          id: "area-wwa-02",
          number: "002",
          city: "Warszawa",
          region: "Mazowieckie",
          signalCitySlug: "warszawa",
          mapX: 140,
          mapY: 190,
          approximateLat: 52.2401,
          approximateLng: 21.0201,
          clue: { pl: "Trzy schody nad rzeka.", en: "Three steps above the river." },
          active: true,
          full: false,
          claimed: false,
        },
        {
          id: "area-krk-01",
          number: "003",
          city: "Krakow",
          region: "Malopolskie",
          signalCitySlug: "krakow",
          mapX: 210,
          mapY: 330,
          approximateLat: 50.0647,
          approximateLng: 19.945,
          clue: { pl: "Pod arkadami.", en: "Under the arcades." },
          active: true,
          full: true,
          claimed: false,
        },
      ],
      migrationRequired: false,
    }),
    // `MerchCatalog { products }` and `FanMerchBundleCatalog { bundles }` — the
    // key was `items`, which decodes to an empty catalog, and the store then
    // renders its outage state. A permanent fake outage is not a screen worth
    // reviewing.
    fan_merch_catalog: () => merchCatalog(),
    fan_cached_merch_catalog: () => merchCatalog(),
    fan_merch_bundles: () => merchBundles(),
    // `Option<TicketSaleOffer>`, queried per event. Answering the same offer for
    // every slug hid the no-ticket-pool branch and made a show with no ticket
    // link still offer KUP BILET — a fixture artifact that looks exactly like a
    // real bug. Only the two shows with a ticket URL have a pool. Two ticket
    // types with different availability, so the sold-out row and the buyable
    // row can both be looked at.
    fan_ticket_sale: (args) => (
      events.find((event) => event.slug === args?.eventSlug)?.ticket_url
        ? ticketSaleOffer(args.eventSlug)
        : null
    ),
    fan_push_sync: () => pushStatus(),
    fan_push_enable: () => ({ ...pushStatus(), enabled: true, permission: "granted" }),
    fan_push_preferences: () => ({
      shows: true, releases: true, community: false, merch: false,
      quiet_hours_enabled: true, quiet_start: "22:00", quiet_end: "08:00",
      quiet_timezone: "Europe/Warsaw",
    }),

    // `operator_events` decodes into `Vec<PublicEvent>`, not into a wrapper.
    // The wrapper shape failed decode silently and left the staff home's
    // event strip blank, which reads as an empty tour rather than a bad
    // fixture. Same list, unwrapped.
    operator_events: () => events,
    operator_signal_overview: () => ({
      generated_at: iso(0),
      summary: {
        total_fans: 4127, active_fans: 3480, pending_fans: 214,
        unsubscribed_fans: 388, suppressed_fans: 45,
        marketing_opted_in: 2611, nearby_enabled: 1902,
      },
      activity: {
        new_fans_7d: 96, new_fans_30d: 412,
        referral_attributions_total: 733, referral_attributions_30d: 88,
        event_interests_total: 5210, event_interests_30d: 640,
        nearby_notifications_30d: 1180, pending_city_requests: 7,
      },
      top_cities: [
        { name: "Warszawa", country_code: "PL", active_fans: 1204 },
        { name: "Kraków", country_code: "PL", active_fans: 731 },
        { name: "Wrocław", country_code: "PL", active_fans: 508 },
        { name: "Poznań", country_code: "PL", active_fans: 344 },
      ],
      audience: {
        active_fans: 3480, marketing_consented_fans: 2611,
        ticket_buyers: 812, attendees: 640,
        synesthesia_participants: 297, qualified_referrals: 188,
        paid_ticket_orders: 941,
      },
      ticket_revenue: [
        { currency: "PLN", paid_orders: 941, gross_paid_minor: 18_930_00,
          refunded_minor: 640_00, after_refunds_minor: 18_290_00 },
      ],
      unavailable_sources: [],
    }),
    operator_qr: () => ({
      events: events.map((e) => ({ slug: e.slug, title: e.title })),
      campaigns: [
        {
          id: "qr-wwa-doors",
          event_title: "Virya · Hydrozagadka",
          label: "Wejście / drzwi",
          max_checkins: 400,
          checkin_count: 128,
          active: true,
          token: "PREVIEW-DOORS-WWA",
        },
        {
          id: "qr-krk-merch",
          event_title: "Virya · Klub Kwadrat",
          label: "Stoisko merch",
          max_checkins: null,
          checkin_count: 0,
          active: false,
          token: null,
        },
      ],
    }),
    // The cold-start snapshot the native side keeps decrypted. `Option<...>`
    // on the Rust side, so `null` is the honest "nothing cached yet".
    operator_cached_sections: () => null,
    // The show checklist. Items span every section and mix done/pending so
    // the progress line, the section grouping and a partially-worked list are
    // all visible at once; an all-pending list hides the done styling.
    operator_show_checklist: () => ({
      event_id: "evt-wwa",
      event_slug: "virya-warszawa-hydrozagadka",
      event_title: "Virya · Hydrozagadka",
      starts_at: iso(12),
      items: [
        ["setlist_ready", "show_files", "done"],
        ["show_files_backup_ready", "show_files", "done"],
        ["laptop_charged_packed", "gear", "done"],
        ["rack_cables_instruments_packed", "gear", "pending"],
        ["instrument_spares_packed", "gear", "pending"],
        ["stage_outfit_packed", "gear", "done"],
        ["wireless_checked", "gear", "pending"],
        ["camera_handoff_ready", "media", "pending"],
        ["merch_packed", "logistics", "done"],
        ["venue_schedule_confirmed", "logistics", "done"],
        ["tech_rider_confirmed", "logistics", "pending"],
        ["staff_assigned", "logistics", "pending"],
        ["guestlist_checked", "gate", "pending"],
        ["offline_snapshot_ready", "gate", "done"],
        ["gate_device_charged", "gate", "done"],
        ["backup_device_ready", "gate", "pending"],
        ["network_tested", "gate", "pending"],
        ["post_show_reconciliation", "post_show", "pending"],
        ["post_show_report", "post_show", "pending"],
      ].map(([item_key, section, status], index) => ({
        item_key,
        section,
        sort_order: index,
        status,
        note: item_key === "tech_rider_confirmed" ? "Czeka na potwierdzenie z klubu." : null,
        updated_at: iso(0),
      })),
    }),

    // Decodes into `FanPushStatus`, a struct — `null` fails decode and the
    // staff panel showed the decode error as a toast on every load.
    operator_push_sync: () => pushStatus(),
    fan_push_status: () => pushStatus(),

    // Deep-link polls. These return `bool` and the app retries on a decode
    // failure, so answering `undefined` span the renderer flat out until it
    // stopped compositing. "Nothing pending" is `false`.
    fan_take_synesthesia_app_link: () => false,
    beacon_take_app_link: () => false,
    // `?link=1` models a confirmation link Android routed into the app: the
    // token stays native, so the only thing the WebView learns is that one is
    // waiting. Pair it with `?mode=fan-locked` for the returning-fan path.
    fan_take_confirm_link: () => params.get("link") === "1",
    fan_device_unlock: () => unlockFan(),
    fan_enable_device_unlock: () => fanSession(true),
    fan_disable_device_unlock: () => fanSession(true),
    fan_confirm_link: () => unlockFan(),
    fan_confirm: () => unlockFan(),
    fan_unlock: () => unlockFan(),
    fan_lock: () => { fanUnlocked = false; return launcher[mode]?.fan ?? fanSession(false); },
    fan_request_access: () => null,
    fan_signup: () => ({ session_created: false, email_queued: true, email_kind: "confirmation", retry_after_seconds: null }),
    fan_prepare_confirmation: () => null,
    fan_push_take_target: () => null,
    native_crash_report: () => null,

    // Every beacon model is camelCase. `beacon_releases` decodes into
    // `BeaconReleasesData { campaigns }`, not `releases`, so the old key left
    // the tab permanently empty — the same shape mistake that made every event
    // list look like no shows were booked.
    // `BeaconPressRoomData` is camelCase, like every beacon model.
    beacon_press_room: () => ({
      event: { title: "Virya · Hydrozagadka", city: "Warszawa" },
      assets: [
        {
          assetKind: "press_photo",
          labelPl: "Zdjęcia prasowe (ZIP)",
          labelEn: "Press photos (ZIP)",
          url: "https://virya.music/press/photos.zip",
        },
        {
          assetKind: "rider",
          labelPl: "Rider techniczny",
          labelEn: "Technical rider",
          url: "https://virya.music/press/rider.pdf",
        },
        {
          assetKind: "bio",
          labelPl: "Bio zespołu",
          labelEn: "Band bio",
          url: "https://virya.music/press/bio.pdf",
        },
      ],
    }),
    beacon_push_sync: () => pushStatus(),
    beacon_home: () => ({
      // radiusKm must be one of the presets the UI offers and topics must use
      // the real keys, or every chip renders unselected and the screen looks
      // like the preferences never saved.
      preferences: {
        radiusKm: 100,
        topics: ["shows", "press_materials", "releases"],
        nearbyGigsEnabled: true,
      },
      nearbyEvents: events.map((event, index) => ({
        id: event.slug,
        title: event.title,
        venue: event.venue,
        city: event.city.name,
        startsAt: event.starts_at,
        distanceKm: [8, 46, 172][index],
        engagementStatus: index === 0 ? "interested" : null,
      })),
    }),
    beacon_news: () => ({
      items: [
        {
          tag: { pl: "Trasa", en: "Tour" },
          title: { pl: "Wolne Miasto — druga tura miast", en: "Wolne Miasto — second run of cities" },
          summary: {
            pl: "Trzy nowe daty i materiały prasowe do pobrania.",
            en: "Three new dates and press assets to download.",
          },
          url: { pl: "https://virya.music/pl/news/trasa", en: "https://virya.music/en/news/tour" },
        },
        {
          tag: { pl: "Wydawnictwo", en: "Release" },
          title: { pl: "Singiel „Nocny Kurs\u201d", en: "Single \u201cNocny Kurs\u201d" },
          summary: { pl: "Premiera w piątek, embargo do czwartku 18:00.", en: "Out Friday, embargo until Thursday 18:00." },
          url: { pl: "https://virya.music/pl/news/singiel", en: "https://virya.music/en/news/single" },
        },
      ],
    }),
    beacon_releases: () => ({
      // `status` vocabularies come from the CrowdRelay CHECK constraints:
      // press requests are open/resolved/cancelled, release recipients are
      // eligible/notified/confirmed/prepared/sent/delivered/declined/expired/
      // cancelled. Inventing values here produces zero counters that look like
      // an app bug.
      campaigns: [
        {
          campaignId: "rel-wolne-miasto",
          title: "Wolne Miasto — pakiet promo",
          productName: "Winyl 180 g",
          variantLabel: "Edycja prasowa",
          recipientStatus: "notified",
          claimDeadline: iso(6),
        },
      ],
    }),
    beacon_press_requests: () => ({
      requests: [
        {
          eventTitle: "Virya · Hydrozagadka",
          requestKind: "accreditation",
          details: "Fotograf + redaktor, wejście od 18:00.",
          status: "open",
          resolutionNote: null,
        },
        {
          eventTitle: "Virya · Klub Kwadrat",
          requestKind: "press_photo",
          details: null,
          status: "resolved",
          resolutionNote: "Materiały wysłane na adres redakcji.",
        },
      ],
    }),
  };

  // A floor on every response. Several commands are polled, and a stub that
  // answers in the same tick turns a poll into a busy loop — which is exactly
  // what wedged the first run of this harness. Real IPC is never this fast.
  //
  // 12 ms is also far faster than a real CrowdRelay-backed read, and that
  // difference hides behaviour: overlapping requests never overlap, and
  // loading states never appear. `?latency=<ms>` (or `?latency=slow`, 220 ms)
  // models a real round trip when that is what is being looked at.
  const requested = params.get("latency");
  const latency = requested === "slow"
    ? 220
    : Math.max(12, Math.min(Number(requested) || 12, 5_000));
  const tick = () => new Promise((r) => setTimeout(r, latency));

  const invoke = async (command, args) => {
    await tick();
    const fixture = FIXTURES[command];
    log.push({ command, args, answered: Boolean(fixture), at: Math.round(performance.now()) });
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
