# Google Play production release — Virya Signal

This repository is prepared so the production release itself is driven from GitHub Actions and Google Play Console only supplies account/policy state that cannot live in source control.

## One-time Play Console / Google Cloud setup

Complete these once before the first production rollout:

1. Enable the Google Play Android Developer API for the Google Cloud project used by the release service account.
2. Grant that service account access to `music.virya.signal` with permission to release to testing and production tracks, but no broader account permissions than required.
3. Configure GitHub repository/environment variables `GOOGLE_PLAY_WIF_PROVIDER` and `GOOGLE_PLAY_SERVICE_ACCOUNT`. Production workflows intentionally fail closed without WIF.
4. In Play Console, finish every required App content declaration: privacy policy, Data safety, Ads, Content rating, Target audience/content, App access, and any other dashboard item Play marks as required for this account/app.
5. Complete the Main store listing: app name, short/long description, app icon, feature graphic, phone screenshots, category/contact details, and privacy-policy URL.
6. Ensure Play App Signing is enabled and copy the Play app-signing SHA-256 certificate fingerprint into the Virya website App Links configuration. `https://virya.music/.well-known/assetlinks.json` must then validate before production publishing can run.
7. If Play Console requires production-access approval/testing history for this developer account, complete that account-level flow. The repository cannot bypass Play policy/account eligibility.

## First production release

1. Merge the intended release commit to `main` and wait for the canonical `Check` workflow to finish green.
2. Run **Android Google Play** from `main` with the intended version and `play_track=production`.
3. The workflow re-verifies the exact source, signed AAB, Firebase configuration, release provenance, live App Links and WIF credentials, then uploads the release at **10% staged rollout**. It cannot jump directly to 100%.
4. In Play Console, review the newly created production release and submit/send it for review if Google requires a manual review action for the account. No artifact rebuild is needed.

## Advancing the rollout

Use **Android Google Play rollout**. Pick only the next desired target: `25`, `50`, `100`, or `halt`.

The rollout workflow:

- only runs from `main` inside the protected `production` environment;
- requires WIF; legacy JSON credentials are not accepted;
- re-checks live Android App Links and CrowdRelay production readiness;
- edits the existing active production release through the Android Publisher API;
- refuses rollback/decrease operations and ambiguous active releases;
- can optionally pin the operation to an expected Android `versionCode`;
- stores a 90-day rollout receipt artifact.

Recommended progression: **10% → 25% → 50% → 100%**. Use `halt` immediately if production telemetry or user reports show a regression.

## Release notes

Localized release notes live under `distribution/whatsnew/` and follow the filename convention required by the Google Play upload action:

- `whatsnew-pl-PL`
- `whatsnew-en-US`

Update them before each public release. Keep each file concise and user-facing.

## What should remain manual

Do not automate policy declarations, Data safety answers, content rating, production-access requests, or visual store-listing approval. Those are product/legal/account decisions and should stay explicit in Play Console.

After the one-time setup is complete, the normal production flow is intentionally small: run the production upload workflow, perform any Play Console review/submit click Google requires, then advance the staged rollout from GitHub as confidence grows.
