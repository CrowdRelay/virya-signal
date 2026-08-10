#!/usr/bin/env python3
from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
css=(root/'styles.css').read_text()
op=(root/'src/app/operator.rs').read_text()
scanner=(root/'src/app/scanner.rs').read_text()
errors=[]
if not all(x in css for x in ['--type-micro: 12px','--type-meta: 13px','--touch-min: 44px']): errors.append('missing type/touch tokens')
small=re.findall(r'font-size:\s*(?:[7-9]|10|11)px\s*;', css)
if small: errors.append(f'legacy sub-12px font sizes remain: {small[:5]}')
for marker in ['show-mode-status-grid','eligible_passes','pending_scans','synced_scans','scan_conflicts']:
    if marker not in op and marker not in scanner and marker not in css: errors.append(f'missing {marker}')
if 'min-height: 300px' not in css and 'min-height: 380px' not in css: errors.append('scanner primary target regressed')
if errors:
    print('SIGNAL_UI_POLISH=FAIL')
    for e in errors: print('-',e)
    raise SystemExit(1)
print('SIGNAL_UI_POLISH=PASS type_floor=12px touch_min=44px show_mode=status-grid')
