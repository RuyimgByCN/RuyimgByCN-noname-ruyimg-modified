import runpy,sys
m=runpy.run_path('/tmp/b-j2-main-run.py');run=m['run'];out=m['OUT'];root=m['ROOT'];target=m['TARGET'];tmp=m['TMP']
jobs=[
 ('phase-B-native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast'],None),
 ('phase-B-affected-six',['cargo','test','--offline','-p','engine-api','-p','engine-core','-p','engine-native','-p','engine-native-wasm','-p','game-server','-p','desktop-client','--features','desktop-client/tauri-shell','--no-fail-fast'],None),
 ('phase-B-workspace',['cargo','test','--offline','--workspace','--no-fail-fast'],None),
 ('phase-B-workspace-all-targets',['cargo','check','--offline','--workspace','--all-targets'],None),
 ('phase-B-strict-native3',['cargo','check','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--all-targets'],{'CARGO_TARGET_DIR':'/tmp/noname-pa09-parent-gate-9e6c6232','RUSTFLAGS':'-Dwarnings'}),
 ('phase-B-source-lines',['python3','-B','scripts/check-source-lines.py'],None),
 ('phase-B-layering',['python3','-B','scripts/check-native-layering.py'],None),
 ('legacy-crosshash',['bash','scripts/cross-target-hash.sh'],{'NODE_OPTIONS':'--require=/tmp/b-j2-main-target.cjs'}),
 ('frontend',['pnpm','--dir','apps/web-client','test'],None),
 ('typecheck',['pnpm','--dir','apps/web-client','typecheck'],None),
 ('frontend-build',['pnpm','--dir','apps/web-client','build'],None),
 ('native-wasm-formal-trajectory',['python3','-B','scripts/check-native-wasm-cross-target.py'],None),
 ('ordinary-native-fixture',['cargo','run','--offline','-p','game-server','--example','emit_projection_fixture','--',str(out/'ordinary-projection.json')],None),
 ('ordinary-actual-wasm',['node','--import','./apps/web-client/scripts/ts-register.mjs','apps/web-client/scripts/native-server-projection-parity.mjs',str(out/'ordinary-projection.json')],None),
 ('b-native-fixtures',['cargo','run','--offline','-p','game-server','--example','emit_approved_b_metadata','--',str(out/'b-metadata-fixtures.json')],None),
 ('ipc-host-build',['cargo','build','--offline','-p','desktop-client','--example','pa09_native_bridge'],None),
 ('b-server-wasm-ipc-parity',['node','--import','./apps/web-client/scripts/ts-register.mjs','apps/web-client/scripts/approved-b-parity.mjs',str(out/'b-metadata-fixtures.json'),target+'/debug/examples/pa09_native_bridge'],None),
 ('desktop-debug-build',['cargo','build','--offline','-p','desktop-client','--features','tauri-shell','--bin','noname-next-desktop'],None),
]
for name,cmd,env in jobs:
 if run(name,cmd,env):sys.exit(1)
