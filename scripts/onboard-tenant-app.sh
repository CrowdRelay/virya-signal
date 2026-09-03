#!/usr/bin/env bash
# Onboard a tenant's Signal mobile app end-to-end.
#
# This is the single script an operator runs to set up everything needed
# for a tenant's Google Play Signal app. It:
#
#   1. Fetches the tenant config from the control plane
#   2. Generates an Android signing keystore
#   3. Creates a Firebase project + Android app, downloads google-services.json
#   4. Optionally sets GitHub repository secrets via `gh` CLI
#   5. Optionally triggers the CI build workflow
#   6. Prints a checklist of remaining manual steps (Play Console listing)
#
# Prerequisites:
#   - Firebase CLI: npm install -g firebase-tools && firebase login
#   - GitHub CLI: brew install gh && gh auth login
#   - keytool (comes with JDK)
#   - Python 3.12+ with Pillow (pip install Pillow)
#
# Usage:
#   bash scripts/onboard-tenant-app.sh \
#       --tenant future-metal \
#       --control-plane-url https://control.virya.music \
#       --token $CONTROL_PLANE_ADMIN_TOKEN \
#       --version 0.1.0 \
#       --version-code 1
#
# Optional flags:
#   --set-github-secrets   Set GitHub secrets automatically (requires gh CLI)
#   --trigger-build        Trigger the tenant-app-build workflow after setup
#   --skip-firebase        Skip Firebase setup (if already done)
#   --skip-keystore        Skip keystore generation (if already done)
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"

TENANT=""
CONTROL_PLANE_URL=""
TOKEN=""
VERSION=""
VERSION_CODE=""
SET_GITHUB_SECRETS="false"
TRIGGER_BUILD="false"
SKIP_FIREBASE="false"
SKIP_KEYSTORE="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tenant) TENANT="$2"; shift 2 ;;
    --control-plane-url) CONTROL_PLANE_URL="$2"; shift 2 ;;
    --token) TOKEN="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --version-code) VERSION_CODE="$2"; shift 2 ;;
    --set-github-secrets) SET_GITHUB_SECRETS="true"; shift ;;
    --trigger-build) TRIGGER_BUILD="true"; shift ;;
    --skip-firebase) SKIP_FIREBASE="true"; shift ;;
    --skip-keystore) SKIP_KEYSTORE="true"; shift ;;
    *) echo "ERROR: unknown argument: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$TENANT" ]] || { echo "ERROR: --tenant is required" >&2; exit 2; }
[[ -n "$CONTROL_PLANE_URL" ]] || { echo "ERROR: --control-plane-url is required" >&2; exit 2; }
[[ -n "$TOKEN" ]] || { echo "ERROR: --token is required" >&2; exit 2; }
[[ -n "$VERSION" ]] || { echo "ERROR: --version is required" >&2; exit 2; }
[[ -n "$VERSION_CODE" ]] || { echo "ERROR: --version-code is required" >&2; exit 2; }

TENANT_UPPER="$(echo "$TENANT" | tr '[:lower:]-' '[:upper:]_')"
FIREBASE_PROJECT="${TENANT}-signal"
PACKAGE_ID="music.${TENANT}.signal"
KEYSTORE_DIR="$ROOT/build/tenant-keys"
CONFIG_FILE="$KEYSTORE_DIR/${TENANT}-config.json"
mkdir -p "$KEYSTORE_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Tenant App Onboarding: $TENANT"
echo "║  Package: $PACKAGE_ID"
echo "║  Version: $VERSION ($VERSION_CODE)"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Step 1: Fetch tenant config ──────────────────────────────────
echo "── Step 1/6: Fetch tenant config ──────────────────────────────"
python3 "$ROOT/scripts/fetch-tenant-config.py" \
  --tenant "$TENANT" \
  --control-plane-url "$CONTROL_PLANE_URL" \
  --token "$TOKEN" \
  --output "$CONFIG_FILE"
echo "✓ Tenant config fetched"
echo ""

# ── Step 2: Generate keystore ────────────────────────────────────
if [[ "$SKIP_KEYSTORE" == "true" ]]; then
  echo "── Step 2/6: Skip keystore generation (--skip-keystore) ───────"
else
  echo "── Step 2/6: Generate Android signing keystore ────────────────"
  if [[ -f "$KEYSTORE_DIR/${TENANT}-upload.jks" ]]; then
    echo "✓ Keystore already exists at $KEYSTORE_DIR/${TENANT}-upload.jks (skipping)"
  else
    bash "$ROOT/scripts/generate-tenant-keystore.sh" --tenant "$TENANT"
  fi
fi
echo ""

# ── Step 3: Firebase setup ───────────────────────────────────────
if [[ "$SKIP_FIREBASE" == "true" ]]; then
  echo "── Step 3/6: Skip Firebase setup (--skip-firebase) ────────────"
else
  echo "── Step 3/6: Firebase project + Android app setup ─────────────"
  if [[ -f "$KEYSTORE_DIR/${TENANT}-google-services.json" ]]; then
    echo "✓ google-services.json already exists (skipping)"
  else
    python3 "$ROOT/scripts/setup-tenant-firebase.py" \
      --tenant "$TENANT" \
      --package-id "$PACKAGE_ID" \
      --project-id "$FIREBASE_PROJECT" \
      --output-dir "$KEYSTORE_DIR"
  fi
fi
echo ""

# ── Step 4: Set GitHub secrets ───────────────────────────────────
if [[ "$SET_GITHUB_SECRETS" == "true" ]]; then
  echo "── Step 4/6: Set GitHub repository secrets ────────────────────"
  SECRETS_FILE="$KEYSTORE_DIR/${TENANT}-secrets.env"
  FIREBASE_SECRETS_FILE="$KEYSTORE_DIR/${TENANT}-firebase-secrets.env"

  if command -v gh >/dev/null 2>&1; then
    if [[ -f "$SECRETS_FILE" ]]; then
      # Extract each secret and set it
      while IFS='=' read -r key value; do
        [[ "$key" =~ ^# ]] && continue
        [[ -z "$key" || -z "$value" ]] && continue
        gh secret set "$key" --body "$value" 2>/dev/null && echo "  ✓ $key" || echo "  ✗ $key (failed)"
      done < "$SECRETS_FILE"
    fi
    if [[ -f "$FIREBASE_SECRETS_FILE" ]]; then
      while IFS='=' read -r key value; do
        [[ "$key" =~ ^# ]] && continue
        [[ -z "$key" || -z "$value" ]] && continue
        gh secret set "$key" --body "$value" 2>/dev/null && echo "  ✓ $key" || echo "  ✗ $key (failed)"
      done < "$FIREBASE_SECRETS_FILE"
    fi
    echo "✓ GitHub secrets set"
  else
    echo "⚠️  gh CLI not found — set secrets manually:"
    echo "   See: $SECRETS_FILE"
    [[ -f "$FIREBASE_SECRETS_FILE" ]] && echo "   See: $FIREBASE_SECRETS_FILE"
  fi
else
  echo "── Step 4/6: GitHub secrets (manual) ──────────────────────────"
  echo "Set these secrets in GitHub (repo → settings → secrets → actions):"
  echo "  - ANDROID_KEYSTORE_BASE64  (from $KEYSTORE_DIR/${TENANT}-secrets.env)"
  echo "  - ANDROID_KEY_ALIAS"
  echo "  - ANDROID_KEY_PASSWORD"
  echo "  - TENANT_${TENANT_UPPER}_GOOGLE_SERVICES_B64  (from $KEYSTORE_DIR/${TENANT}-firebase-secrets.env)"
  echo ""
  echo "  Or re-run with --set-github-secrets to set them automatically (requires gh CLI)"
fi
echo ""

# ── Step 5: Trigger build ────────────────────────────────────────
if [[ "$TRIGGER_BUILD" == "true" ]]; then
  echo "── Step 5/6: Trigger CI build workflow ────────────────────────"
  if command -v gh >/dev/null 2>&1; then
    gh workflow run tenant-app-build.yml \
      --field tenant_slug="$TENANT" \
      --field version="$VERSION" \
      --field version_code="$VERSION_CODE" \
      --field publish=false
    echo "✓ Workflow triggered — check: gh run list --workflow tenant-app-build.yml"
  else
    echo "⚠️  gh CLI not found — trigger manually:"
    echo "   GitHub → Actions → Tenant App Build → Run workflow"
    echo "   tenant_slug=$TENANT version=$VERSION version_code=$VERSION_CODE"
  fi
else
  echo "── Step 5/6: Build (manual) ───────────────────────────────────"
  echo "After setting GitHub secrets, trigger the build:"
  echo "  GitHub → Actions → 'Tenant App Build' → Run workflow"
  echo "  tenant_slug=$TENANT  version=$VERSION  version_code=$VERSION_CODE"
  echo ""
  echo "  Or re-run with --trigger-build to trigger automatically"
fi
echo ""

# ── Step 6: Remaining manual steps ───────────────────────────────
echo "── Step 6/6: Manual checklist ─────────────────────────────────"
echo ""
echo "After the first AAB is built:"
echo "  1. ☐ Create the app in Google Play Console"
echo "     Package ID: $PACKAGE_ID"
echo "     App name:   (from tenant config)"
echo "  2. ☐ Upload the AAB from the CI artifact to the Play Console"
echo "     (internal track first, then promote)"
echo "  3. ☐ Copy the Play Store URL from the Console"
echo "  4. ☐ Set the Play Store URL in the control plane:"
echo "     PATCH $CONTROL_PLANE_URL/api/v1/tenants/$TENANT/mobile-apps"
echo "     { \"signalPlayStoreUrl\": \"https://play.google.com/store/apps/details?id=$PACKAGE_ID\" }"
echo "  5. ☐ Verify the Google Play button appears on the tenant page"
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Onboarding complete! Follow the checklist above.           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
