import runpy,sys
m=runpy.run_path('/tmp/nc10-main-run.py');run=m['run']
jobs=[
 ('nc10-targeted',['cargo','test','--offline','-p','engine-native','--test','nc_rounds','nc10_','--','--nocapture'],None),
 ('native3',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast'],None),
 ('workspace',['cargo','test','--offline','--workspace','--no-fail-fast'],None),
 ('strict-native3',['cargo','check','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--all-targets'],{'CARGO_TARGET_DIR':'/tmp/noname-pa09-parent-gate-9e6c6232','RUSTFLAGS':'-Dwarnings'}),
 ('source-line-units',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],None),
 ('source-lines',['python3','-B','scripts/check-source-lines.py'],None),
 ('layering',['python3','-B','scripts/check-native-layering.py'],None),
 ('conformance',['cargo','run','--offline','-p','conformance'],None),
 ('native-wasm-build',['pnpm','--dir','apps/web-client','build:native'],{'NODE_OPTIONS':'--require=/tmp/nc10-main-target.cjs'}),
 ('worker',['pnpm','--dir','apps/web-client','smoke:native'],None),
 ('frontend',['pnpm','--dir','apps/web-client','test'],None),
 ('typecheck',['pnpm','--dir','apps/web-client','typecheck'],None),
 ('native-wasm-trajectory',['python3','-B','scripts/check-native-wasm-cross-target.py'],None),
]
for name,cmd,extra in jobs:
 if run(name,cmd,extra):sys.exit(1)
