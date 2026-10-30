import runpy,sys
m=runpy.run_path('/tmp/pa09-recovery.py');run=m['run'];out=m['OUT']
jobs=[
 ('machine-fixtures-registered-misc',['cargo','test','--offline','-p','engine-native','--test','misc','nc09_machine_ownership::','--','--nocapture'],{'NC09_MACHINE_TRACE_OUT':str(out/'recovery-machine/machine-traces'),'NC09_MACHINE_ARCHIVE_OUT':str(out/'recovery-machine/archives')}),
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
