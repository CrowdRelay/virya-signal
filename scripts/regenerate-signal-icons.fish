#!/usr/bin/env fish
set -l repo (path resolve (dirname (status filename))/..)
cd "$repo"; or exit 1

if not command -q cargo
    echo "ERROR: cargo not found" >&2
    exit 1
end

if not test -f branding/signal-v2.svg
    echo "ERROR: branding/signal-v2.svg missing" >&2
    exit 1
end

cargo tauri icon branding/signal-v2.svg
or exit $status

echo "SIGNAL_ICONSET=GENERATED source=branding/signal-v2.svg"
