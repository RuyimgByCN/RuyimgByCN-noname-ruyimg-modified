import importlib.util,sys,json
from pathlib import Path
spec=importlib.util.spec_from_file_location('prior','/tmp/nc27-main-functional-driver.py');p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
p.O=Path('/tmp/nc27-main-final-v3-8e1fad00');p.E.update({'CARGO_TARGET_DIR':'/tmp/nc27-main-functional-458ed333/cargo-target','CARGO_NET_OFFLINE':'true','NODE_OPTIONS':'--require='+str(p.O/'build-target.cjs')})
assert (p.O/'post-build.json').is_file()
commands=[('actual-native-source-final',['node',str(p.O/'actual-native-source.mjs')],p.R),('actual-legacy-sink',['node',str(p.O/'legacy-wasm-sink-smoke.mjs')],p.R),('worker-smoke',['pnpm','smoke:native'],p.R/'apps/web-client'),('frontend-post-wasm',['pnpm','test'],p.R/'apps/web-client'),('legacy-cross',['python3','-B',str(p.O/'cross-target-current.py')],p.R),('native-cross',['python3','-B','scripts/check-native-wasm-cross-target.py'],p.R),('vite-post-wasm',['node',str(p.O/'vite-build.mjs')],p.R/'apps/web-client')]
for name,argv,cwd in commands:
 code=p.run(name,argv,cwd)
 if code:sys.exit(code)
p.snapshot('post-impact.json')
