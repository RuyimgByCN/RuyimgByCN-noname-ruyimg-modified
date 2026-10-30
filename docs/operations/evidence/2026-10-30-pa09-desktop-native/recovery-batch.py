import runpy,sys,json,time
from pathlib import Path
started=time.monotonic()
while True:
 index=Path("/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-pa09-desktop-native/recovery-command-results.json")
 rows=json.loads(index.read_text()) if index.exists() else []
 workspace=next((row for row in rows if row["name"]=="workspace"),None)
 if workspace:
  if workspace["exit"]:sys.exit("Workspace recovery failed; no later batch commands launched")
  break
 if time.monotonic()-started>3600:sys.exit("Workspace did not finish within one hour; no later batch commands launched")
 time.sleep(2)
m=runpy.run_path('/tmp/pa09-recovery.py');run=m['run'];out=m['OUT']
preload={'NODE_OPTIONS':'--require=/tmp/pa09-build-target.cjs'}
jobs=[
 ('strict-native-all3',['cargo','check','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--all-targets'],{'CARGO_TARGET_DIR':'/tmp/noname-pa09-parent-gate-9e6c6232','RUSTFLAGS':'-Dwarnings'}),
 ('native-layering',['python3','-B','scripts/check-native-layering.py'],None),
 ('wasm-web-build',['pnpm','--dir','apps/web-client','build:native'],preload),
 ('worker-final',['pnpm','--dir','apps/web-client','smoke:native'],None),
 ('frontend-after-wasm',['pnpm','--dir','apps/web-client','test'],None),
 ('typecheck',['pnpm','--dir','apps/web-client','typecheck'],None),
 ('frontend-build',['pnpm','--dir','apps/web-client','build'],None),
 ('conformance',['cargo','run','--offline','-p','conformance'],None),
 ('native-wasm-cross',['python3','-B','scripts/check-native-wasm-cross-target.py'],None),
 ('legacy-cross',['bash','scripts/cross-target-hash.sh'],preload),
 ('source-line-units',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],None),
 ('source-lines',['python3','-B','scripts/check-source-lines.py'],None),
 ('machine-fixtures',['cargo','test','--offline','-p','engine-native','--test','nc09','nc09_machine_ownership::','--','--nocapture'],{'NC09_MACHINE_TRACE_OUT':str(out/'recovery-machine/machine-traces'),'NC09_MACHINE_ARCHIVE_OUT':str(out/'recovery-machine/archives')}),
 ('machine-wasm',['node',str(out/'validation-wasm-machine-windows.mjs'),str(m['ROOT']),str(out/'recovery-machine')],None),
 ('ordinary-fixtures',['cargo','run','--offline','-p','game-server','--example','emit_projection_fixture','--',str(out/'recovery-ordinary-projection.json')],None),
 ('ordinary-wasm',['node','--import','./apps/web-client/scripts/ts-register.mjs','apps/web-client/scripts/native-server-projection-parity.mjs',str(out/'recovery-ordinary-projection.json')],None),
 ('desktop-debug-build',['cargo','build','--offline','-p','desktop-client','--features','tauri-shell'],None),
 ('reload-fixture-build',['node','/tmp/pa09-validation-webview/build-webview.mjs','pa09-document-reload'],None),
 ('reload-webview',['node','/tmp/pa09-validation-webview/run-webview.mjs','pa09-document-reload','recovery'],None),
 ('lifecycle-fixture-build',['node','/tmp/pa09-validation-webview/build-webview.mjs','pa09-desktop-webview'],None),
 ('lifecycle-webview',['node','/tmp/pa09-validation-webview/run-webview.mjs','pa09-desktop-webview','recovery'],None),
 ('desktop-release-build',['cargo','build','--offline','--release','-p','desktop-client','--features','tauri-shell'],None),
]
for name,cmd,extra in jobs:
 if run(name,cmd,extra):sys.exit(1)
