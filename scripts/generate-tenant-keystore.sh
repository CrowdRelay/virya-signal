#!/usr/bin/env bash
# Generate an Android signing keystore for a tenant app.
#
# Produces a .jks keystore and outputs the base64-encoded content plus the
# alias and password, ready to paste into GitHub repository secrets.
#
# Usage:
#   bash scripts/generate-tenant-keystore.sh --tenant future-metal
#
# Outputs:
#   build/tenant-keys/{slug}-upload.jks
#   build/tenant-keys/{slug}-secrets.env  (ready to source or paste into GitHub)
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"

TENANT=""
KEYSTORE_PASSWORD=""
ALIAS=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tenant) TENANT="$2"; shift 2 ;;
    --password) KEYSTORE_PASSWORD="$2"; shift 2 ;;
    --alias) ALIAS="$2"; shift 2 ;;
    *) echo "ERROR: unknown argument: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$TENANT" ]] || { echo "ERROR: --tenant is required" >&2; exit 2; }
ALIAS="${ALIAS:-${TENANT}-upload}"
# Generate a strong random password if not provided
if [[ -z "$KEYSTORE_PASSWORD" ]]; then
  KEYSTORE_PASSWORD="$(python3 -c "import secrets, string; print(''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(32)))")"
fi

OUTPUT_DIR="$ROOT/build/tenant-keys"
mkdir -p "$OUTPUT_DIR"

KEYSTORE_PATH="$OUTPUT_DIR/${TENANT}-upload.jks"
SECRETS_FILE="$OUTPUT_DIR/${TENANT}-secrets.env"

echo "KEYSTORE_GEN=START tenant=$TENANT alias=$ALIAS"

# Generate the keystore with keytool
keytool -genkeypair \
  -keystore "$KEYSTORE_PATH" \
  -storepass "$KEYSTORE_PASSWORD" \
  -alias "$ALIAS" \
  -keyalg EC \
  -keysize 256 \
  -validity 10950 \
  -dname "CN=${TENANT} Signal Upload, O=CrowdRelay, C=PL" \
  -storetype JKS

echo "KEYSTORE_GEN=DONE path=$KEYSTORE_PATH"

# Base64-encode the keystore for GitHub secrets
KEYSTORE_B64="$(base64 -i "$KEYSTORE_PATH" | tr -d '\n')"

# Write the secrets file
cat > "$SECRETS_FILE" <<EOF
# GitHub repository secrets for ${TENANT} Signal app
# Paste each value into the repository settings → secrets and variables → actions
ANDROID_KEYSTORE_BASE64=${KEYSTORE_B64}
ANDROID_KEY_ALIAS=${ALIAS}
ANDROID_KEY_PASSWORD=${KEYSTORE_PASSWORD}
EOF
chmod 600 "$SECRETS_FILE"

echo ""
echo "KEYSTORE_GEN=SUCCESS"
echo ""
echo "Keystore:     $KEYSTORE_PATH"
echo "Secrets file: $SECRETS_FILE"
echo ""
echo "Next steps:"
echo "  1. Store the keystore file in a secure backup (1Password, etc.)"
echo "  2. Add the three secrets to GitHub (repo settings → secrets → actions):"
echo "     - ANDROID_KEYSTORE_BASE64"
echo "     - ANDROID_KEY_ALIAS"
echo "     - ANDROID_KEY_PASSWORD"
echo "  3. Or run: bash scripts/onboard-tenant-app.sh --tenant $TENANT --set-github-secrets"
echo ""
echo "⚠️  The keystore password is in the secrets file. Save it now — it won't be shown again."
