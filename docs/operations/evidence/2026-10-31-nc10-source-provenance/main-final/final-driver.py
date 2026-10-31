import importlib.util,os,sys,json,subprocess,time,hashlib
from pathlib import Path
spec=importlib.util.spec_from_file_location('prior','/tmp/nc27-main-functional-driver.py');p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
p.O=Path('/tmp/nc27-main-final-v3-8e1fad00');p.O.mkdir(exist_ok=True)
p.E['CARGO_TARGET_DIR']='/tmp/nc27-main-functional-458ed333/cargo-target'
p.E['CARGO_NET_OFFLINE']='true'
p.E['WASM_PACK_TARGET']='web'
p.E['NODE_OPTIONS']='--require='+str(p.O/'build-target.cjs')
# Native scripts reset CARGO_TARGET_DIR; the preload changes only that child environment key.
mode=sys.argv[1]
if mode=='before':
 p.snapshot('before.json')
 p.save('admission.json',json.loads(Path('/tmp/noname-parent-nc27-main-admission.json').read_text()))
elif mode=='snapshot':p.snapshot(sys.argv[2])
elif mode=='pipeline':
 commands=[('workspace',['cargo','test','--offline','--workspace','--no-fail-fast'],p.R),('build-native',['node','scripts/build-native.mjs'],p.R/'apps/web-client'),('build-legacy',['node','scripts/build-wasm.mjs'],p.R/'apps/web-client')]
 for name,argv,cwd in commands:
  code=p.run(name,argv,cwd)
  if code:sys.exit(code)
 p.snapshot('post-build.json')
elif mode=='command':sys.exit(p.run(sys.argv[2],sys.argv[3:],p.R))
elif mode=='frontend':
 for name,argv in [('worker-smoke',['pnpm','smoke:native']),('frontend-post-wasm',['pnpm','test']),('vite-post-wasm',['node',str(p.O/'vite-build.mjs')])]:
  code=p.run(name,argv,p.R/'apps/web-client')
  if code:sys.exit(code)
elif mode=='gates':
 p.snapshot('gate-entry.json')
 failures=[]
 for name,argv in [('lineunit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('sourcecaps',['python3','-B','scripts/check-source-lines.py']),('strictrefs',['python3','-B','scripts/check_doc_line_refs.py','--strict']),('preflight',['python3','-B','scripts/preflight.py'])]:
  code=p.run(name,argv)
  if code:failures.append(name)
 p.snapshot('gate-exit.json')
 sys.exit(1 if failures else 0)
