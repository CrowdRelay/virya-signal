#!/usr/bin/env bash
# Build a tenant-branded Signal Android AAB.
#
# Fetches the tenant config from the control plane, generates a tenant-specific
# tauri.conf.json, generates branded icons from the tenant palette, then runs
# the standard Tauri Android release build.
#
# The original tauri.conf.json is backed up and restored after the build.
#
# Usage:
#   bash scripts/build-tenant-app.sh \
#       --tenant virya \
#       --control-plane-url https://control.virya.music \
#       --token $CONTROL_PLANE_ADMIN_TOKEN \
#       --version 0.5.1 \
#       --version-code 2011
#
# Outputs:
#   build/tenant-apps/{slug}-signal-{version}.aab
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

TENANT=""
CONTROL_PLANE_URL=""
TOKEN=""
VERSION=""
VERSION_CODE=""
SIGNAL_API_HOST=""
PUBLISH="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tenant) TENANT="$2"; shift 2 ;;
    --control-plane-url) CONTROL_PLANE_URL="$2"; shift 2 ;;
    --token) TOKEN="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --version-code) VERSION_CODE="$2"; shift 2 ;;
    --signal-api-host) SIGNAL_API_HOST="$2"; shift 2 ;;
    --publish) PUBLISH="true"; shift ;;
    *) echo "ERROR: unknown argument: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$TENANT" ]] || { echo "ERROR: --tenant is required" >&2; exit 2; }
[[ -n "$CONTROL_PLANE_URL" ]] || { echo "ERROR: --control-plane-url is required" >&2; exit 2; }
[[ -n "$TOKEN" ]] || { echo "ERROR: --token is required" >&2; exit 2; }
[[ -n "$VERSION" ]] || { echo "ERROR: --version is required" >&2; exit 2; }
[[ -n "$VERSION_CODE" ]] || { echo "ERROR: --version-code is required" >&2; exit 2; }

CONFIG_FILE="$ROOT/build/tenant-apps/${TENANT}-config.json"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
mkdir -p "$CONFIG_DIR"

echo "TENANT_BUILD=FETCH tenant=$TENANT url=$CONTROL_PLANE_URL"
python3 "$ROOT/scripts/fetch-tenant-config.py" \
  --tenant "$TENANT" \
  --control-plane-url "$CONTROL_PLANE_URL" \
  --token "$TOKEN" \
  --output "$CONFIG_FILE"

# Extract fields from the config
PACKAGE_ID="$(python3 -c "import json; print(json.load(open('$CONFIG_FILE'))['packageId'])")"
APP_NAME="$(python3 -c "import json; print(json.load(open('$CONFIG_FILE'))['appName'])")"
SIGNAL_BASE_URL="$(python3 -c "import json; print(json.load(open('$CONFIG_FILE')).get('signalBaseUrl') or '')")"

# Derive the CSP connect-src host from the signal base URL or the override
if [[ -z "$SIGNAL_API_HOST" && -n "$SIGNAL_BASE_URL" ]]; then
  # The Signal API host is typically signal-api.{domain} derived from the site URL
  # But the tenant's CrowdRelay API URL is the actual connect target
  SIGNAL_API_HOST="$(python3 -c "import json; print(json.load(open('$CONFIG_FILE')).get('crowdrelayBaseUrl') or '')")"
fi
[[ -n "$SIGNAL_API_HOST" ]] || SIGNAL_API_HOST="https://signal-api.virya.music"

echo "TENANT_BUILD=CONFIG package=$PACKAGE_ID app=\"$APP_NAME\" api=$SIGNAL_API_HOST"

# Generate tenant-branded icons
echo "TENANT_BUILD=ICONS"
python3 "$ROOT/scripts/generate-tenant-icons.py" \
  --config "$CONFIG_FILE" \
  --output-dir "$ROOT/src-tauri/icons/tenant"

# Back up the original tauri.conf.json
TAURI_CONF="$ROOT/src-tauri/tauri.conf.json"
BACKUP="$TAURI_CONF.bak"
cp "$TAURI_CONF" "$BACKUP"
trap 'cp "$BACKUP" "$TAURI_CONF"; rm -f "$BACKUP"; echo "TENANT_BUILD=RESTORE tauri.conf.json restored"' EXIT

# Render the template
TEMPLATE="$ROOT/scripts/templates/tauri.conf.json.template"
# The short and long descriptions are tenant-specific
SHORT_DESC="$APP_NAME — fan engagement and live operations"
LONG_DESC="$APP_NAME delivers concerts, fan rewards, mobile ticket wallet, QR admission and live operations powered by CrowdRelay."

python3 -c "
import json, sys
template = open('$TEMPLATE').read()
config = json.load(open('$CONFIG_FILE'))
rendered = template \\
    .replace('{{APP_NAME}}', '''$APP_NAME''') \\
    .replace('{{APP_VERSION}}', '$VERSION') \\
    .replace('{{PACKAGE_ID}}', '$PACKAGE_ID') \\
    .replace('{{SURFACE_COLOR}}', (config.get('brandingPalette') or {}).get('surface', '#070908')) \\
    .replace('{{CSP_CONNECT_SOURCES}}', '$SIGNAL_API_HOST') \\
    .replace('{{APP_SHORT_DESCRIPTION}}', '''$SHORT_DESC''') \\
    .replace('{{APP_LONG_DESCRIPTION}}', '''$LONG_DESC''') \\
    .replace('{{ANDROID_VERSION_CODE}}', '$VERSION_CODE')
open('$TAURI_CONF', 'w').write(rendered)
print('TENANT_BUILD=RENDER tauri.conf.json written')
"

# Run the Tauri Android build
echo "TENANT_BUILD=AAB_START"
cargo tauri android build --release --target aarch64

# Locate the output AAB and copy it to the tenant-apps directory
AAB_SRC="$(find "$ROOT/src-tauri/gen/android" -name '*.aab' -path '*/release/*' | head -1)"
[[ -n "$AAB_SRC" ]] || { echo "ERROR: no release AAB found after build" >&2; exit 1; }

AAB_DEST="$ROOT/build/tenant-apps/${TENANT}-signal-${VERSION}.aab"
cp "$AAB_SRC" "$AAB_DEST"
echo "TENANT_BUILD=AAB_DONE dest=$AAB_DEST bytes=$(stat -f%z "$AAB_DEST" 2>/dev/null || stat -c%s "$AAB_DEST")"

if [[ "$PUBLISH" == "true" ]]; then
  echo "TENANT_BUILD=PUBLISH_START package=$PACKAGE_ID"
  echo "ERROR: publishing is not yet implemented in this script. Use the Google Play Console or the existing android-play.yml workflow." >&2
  exit 3
fi

echo "TENANT_BUILD=SUCCESS tenant=$TENANT package=$PACKAGE_ID version=$VERSION"
