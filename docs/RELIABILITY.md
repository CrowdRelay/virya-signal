# Reliability and diagnostics

## Immediate failures

The runtime guard is installed before Leptos mounts. Uncaught JavaScript errors, rejected promises and Rust/WASM panics surface a full-screen diagnostic with a copyable bounded report.

## Interrupted operation ledger

Before a native command starts, the application persists only:

- command name;
- start timestamp;
- origin plus pathname.

Arguments, form values, tokens, query strings and URL fragments are excluded. The record is removed only after the command completes. If the process disappears, the next launch reports the interrupted operation.

## Foreground heartbeat

`boot.js` persists a small session heartbeat. Normal backgrounding and navigation mark the session as background. A recent foreground session without a clean transition is reported on the next launch as an unexpected termination with the last boot phase.

## Native panic durability

The Rust panic hook writes through a temporary file, flushes it and atomically renames it. Startup no longer deletes the report before delivery. The WebView reads it through a native command, stores/displays it, then explicitly acknowledges deletion.

## What this guarantees

Catchable WebView/WASM/Rust failures produce an immediate report. Abrupt foreground termination produces a next-launch diagnostic with the last persisted phase or operation.

## Physical limits

No application can guarantee a stack trace for hardware failure, storage failure, Android low-memory kill or a process killed before writable storage is available. Those cases still produce an abnormal-session diagnostic when the previous heartbeat reached storage, but the reason may be classified as an unexpected termination rather than a precise stack.
