#!/usr/bin/env python3
from __future__ import annotations
import argparse,json
from pathlib import Path
# A ratchet only helps if it trips before the hard cap in check-web-dist.py.
# At 1.20/48KiB it allowed 2031 KiB of WASM against a 1792 KiB ceiling, so it
# could never fire first, and codeGzipBytes -- which has no hard cap at all --
# allowed ~144 KiB of gzip growth in a single commit. These leave roughly a
# 25-50 KiB step for ordinary feature work and force a deliberate decision
# beyond that.
RULES={'wasmBytes':(1.03,24*1024),'largestWasmBytes':(1.03,24*1024),'codeBytes':(1.05,32*1024),'codeGzipBytes':(1.03,24*1024)}
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('current',type=Path); ap.add_argument('previous',type=Path); a=ap.parse_args(); c=json.loads(a.current.read_text()); p=json.loads(a.previous.read_text()); bad=[]
 for key,(ratio,noise) in RULES.items():
  cur=float(c[key]); prev=float(p[key]); limit=max(prev*ratio,prev+noise)
  if cur>limit: bad.append(f'{key}:{cur:g}>{limit:g}(prev={prev:g})')
 if bad: raise SystemExit('SIGNAL_WEB_REGRESSION=FAIL '+','.join(bad))
 print('SIGNAL_WEB_REGRESSION=PASS baseline=previous-successful-main')
if __name__=='__main__': main()
