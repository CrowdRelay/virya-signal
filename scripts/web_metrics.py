#!/usr/bin/env python3
from __future__ import annotations
import argparse,gzip,json
from pathlib import Path
CODE={'.wasm','.js','.mjs','.css'}
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('root',type=Path); ap.add_argument('output',type=Path); a=ap.parse_args()
 files=[p for p in a.root.rglob('*') if p.is_file()]; wasm=[p for p in files if p.suffix=='.wasm']; code=[p for p in files if p.suffix.lower() in CODE]
 if not wasm: raise SystemExit('Signal WASM artifact missing')
 data={'schema':1,'fileCount':len(files),'wasmBytes':sum(p.stat().st_size for p in wasm),'largestWasmBytes':max(p.stat().st_size for p in wasm),'codeBytes':sum(p.stat().st_size for p in code),'codeGzipBytes':sum(len(gzip.compress(p.read_bytes(),compresslevel=9,mtime=0)) for p in code)}
 a.output.parent.mkdir(parents=True,exist_ok=True); a.output.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n')
 print('SIGNAL_WEB_METRICS=PASS '+' '.join(f'{k}={v}' for k,v in data.items() if k!='schema'))
if __name__=='__main__': main()
