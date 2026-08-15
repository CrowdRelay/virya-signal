# Virya Signal package report — 2026-08-15 principal/staff pass

Baseline: `virya-ecosystem-20260815-004323.zip` (Virya Signal 0.4.2).

## Native push — Android / Firebase

- Fixed the Android notification-permission bridge so VIRYA's custom plugin no longer overrides Tauri's built-in `checkPermissions` / `requestPermissions` commands. Tauri indexes commands by method name across the plugin hierarchy, so the previous collision could return a different JSON schema and leave the UI stuck on `WŁĄCZ POWIADOMIENIA`.
- VIRYA now uses uniquely named native commands: `getNotificationPermissionState`, `requestNotificationPermission` and `openNotificationSettings`.
- Notification state also respects Android's global per-app notification switch via `NotificationManagerCompat.areNotificationsEnabled()`.
- A permanently denied permission changes the CTA to `OTWÓRZ USTAWIENIA` / `OPEN SETTINGS` instead of repeatedly pretending the runtime prompt can be shown.
- When the user returns from Android notification settings, the app resumes the original enable intent and automatically completes FCM-token registration plus CrowdRelay registration. A second tap is not required.
- Resume handling is explicit and lifecycle-safe: listeners are removed on component cleanup and duplicate resume events cannot duplicate registration.
- Cold-start `pageshow` no longer triggers a redundant launcher/push status refresh; real foreground/bfcache returns still do.
- Firebase client configuration remains an external CI secret. The source package contains no `google-services.json` or service-account credential.

## Icon / launcher pipeline

- Fixed Tauri's native build failure caused by RGB PNG bundle icons: every PNG referenced by `bundle.icon` is now RGBA.
- Removed the duplicate tracked `src-tauri/icons/icon.png`; `virya-signal-brand-full.png` is the canonical full artwork.
- Consolidated Android launcher sources under `src-tauri/icons/android`; the obsolete duplicate `src-tauri/launcher-assets` tree was removed.
- Android adaptive foreground assets are now copied from the audited transparent canonical foreground instead of being regenerated from the opaque full tile.
- Android CI no longer runs generic `cargo tauri icon`, which could overwrite the audited adaptive foreground with an opaque image immediately before packaging.
- Android 13+ monochrome adaptive XML is still generated from the canonical Android launcher assets.
- Asset contracts parse actual `tauri.conf.json`, verify RGBA bundle icons and verify that the adaptive foreground contains both transparent and opaque pixels.

## Merch / app assets

- The local optimized Stage Pack preview remains bundled and preferred for `bundle-stage-pack`, avoiding an empty card when a remote merch image is missing.
- Desktop, Windows, Android and iOS-required icon slots remain present; only redundant source copies were removed.

## Validation in this packaging environment

Source-level gates cover:

- static package invariants;
- Python contract/unit suite;
- Android preparation and Firebase staging fixtures;
- native push control-plane contract;
- boot/resume lifecycle runtime tests;
- source-size and CI-policy ratchets;
- PNG/adaptive-icon structural validation;
- shell syntax checks.

The packaging sandbox does not provide the Rust/Cargo/Godot toolchains, and the Virya web workspace has no installed `node_modules`. Therefore compiler-dependent Rust/Tauri and Godot engine/export gates, plus the full Astro production build, remain CI/runtime responsibilities. The next Android CI build is also the authoritative Kotlin/Gradle integration check for the revised push plugin.
