#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAX_LOC = 1000
CONTRACT = {
    "src/bridge.rs": ["bridge/ffi.rs", "bridge/client.rs"],
    "src/app/fan.rs": ["fan/shell.rs", "fan/merch.rs", "fan/events.rs", "fan/wallet.rs"],
    "src/app/operator.rs": [
        "operator/shell.rs", "operator/signal.rs", "operator/commerce_settings.rs",
    ],
    "src-tauri/src/models.rs": [
        "models/session_fan.rs", "models/commerce_events.rs", "models/area.rs",
        "models/signal.rs", "models/showmode_inputs.rs", "models/tests.rs",
    ],
    "src-tauri/src/commands/fan.rs": [
        "fan/push.rs", "fan/session_commerce.rs", "fan/wallet.rs", "fan/tests.rs",
    ],
}
DATA_EXCEPTIONS = {"src/i18n/pl.rs", "src/i18n/en.rs"}
# `ffi.rs` is one `#[wasm_bindgen(inline_js = ...)]` block, and an inline_js
# block is one ES module: the invoke registry, the storage helpers and the
# translation table are module-level bindings its scanner, location and
# crash-report functions close over. Splitting the file means splitting the
# attribute, which means separate modules that can no longer see those
# bindings — so the size here is a property of the JS module, not of the Rust.
MODULE_EXCEPTIONS = {"src/bridge/ffi.rs"}

def loc(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())

def fail(reason: str) -> None:
    raise SystemExit(f"SIGNAL_MODULARITY=FAIL {reason}")

def main() -> None:
    chunks = 0
    for parent_rel, child_rels in CONTRACT.items():
        parent = ROOT / parent_rel
        if not parent.is_file(): fail(f"missing-parent={parent_rel}")
        if loc(parent) > MAX_LOC: fail(f"parent-too-large={parent_rel} loc={loc(parent)}")
        source = parent.read_text(encoding="utf-8")
        for child_rel in child_rels:
            child = parent.parent / child_rel
            if not child.is_file(): fail(f"missing-chunk={child.relative_to(ROOT)}")
            child_key = child.relative_to(ROOT).as_posix()
            if loc(child) > MAX_LOC and child_key not in MODULE_EXCEPTIONS:
                fail(f"chunk-too-large={child_key} loc={loc(child)}")
            if f'include!("{child_rel}");' not in source:
                fail(f"missing-include={parent_rel}:{child_rel}")
            chunks += 1
    # Every nested include must resolve relative to the file that contains it.
    import re
    include_re = re.compile(r'include!\("([^"]+)"\);')
    for source_root in (ROOT / "src", ROOT / "src-tauri/src"):
        for path in source_root.rglob("*.rs"):
            for rel in include_re.findall(path.read_text(encoding="utf-8")):
                target = path.parent / rel
                if not target.is_file():
                    fail(f"broken-include={path.relative_to(ROOT)}:{rel}")

    # A physical split must never strand an outer attribute at EOF. Rust does
    # not carry attributes across include! boundaries: an attribute belongs to
    # the next item in the *same included token stream*. A dangling derive or
    # command attribute therefore turns a harmless-looking split into a compile
    # error (or silently removes the attribute from the intended item).
    for source_root in (ROOT / "src", ROOT / "src-tauri/src"):
        for path in source_root.rglob("*.rs"):
            lines = path.read_text(encoding="utf-8").splitlines()
            while lines and not lines[-1].strip():
                lines.pop()
            if lines and lines[-1].lstrip().startswith("#["):
                fail(f"dangling-attribute={path.relative_to(ROOT)}:{lines[-1].strip()}")

    oversized = []
    for source_root in (ROOT / "src", ROOT / "src-tauri/src"):
        for path in source_root.rglob("*.rs"):
            rel = path.relative_to(ROOT).as_posix()
            if rel in DATA_EXCEPTIONS or rel in MODULE_EXCEPTIONS: continue
            if loc(path) > MAX_LOC: oversized.append((rel, loc(path)))
    if oversized: fail(f"oversized-production={oversized}")
    # An exception that no longer needs to exist is a licence to grow, so it
    # expires the moment the file it covers fits the limit on its own.
    stale = [rel for rel in sorted(DATA_EXCEPTIONS | MODULE_EXCEPTIONS)
             if (ROOT / rel).is_file() and loc(ROOT / rel) <= MAX_LOC]
    if stale: fail(f"stale-exception={stale}")
    missing = [rel for rel in sorted(DATA_EXCEPTIONS | MODULE_EXCEPTIONS) if not (ROOT / rel).is_file()]
    if missing: fail(f"missing-exception-target={missing}")
    print(
        f"SIGNAL_MODULARITY=PASS parents={len(CONTRACT)} chunks={chunks} max={MAX_LOC} "
        f"data_exceptions={len(DATA_EXCEPTIONS)} module_exceptions={len(MODULE_EXCEPTIONS)}"
    )

if __name__ == "__main__": main()
