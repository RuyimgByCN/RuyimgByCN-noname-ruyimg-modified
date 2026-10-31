import subprocess,os,sys,json
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path(__file__).parent;E=R/'docs/operations/evidence/2026-10-24-native-wasm-cross-target'
commands=[
(['cargo','test','--offline','-p','engine-native','--test','nc08','nc08_cross_target_real::','--','--nocapture'],{'NONAME_CROSS_TARGET_NATIVE_OUT':str(O/'trajectory-native.json')}),
(['node',str(E/'wasm-trajectory.mjs'),str(O/'trajectory-wasm.json')],{}),
([sys.executable,'-B',str(E/'compare.py'),str(O/'trajectory-native.json'),str(O/'trajectory-wasm.json')],{}),
(['cargo','test','--offline','-p','engine-native-wasm','--test','cross_target_package','--','--nocapture'],{'NONAME_CROSS_TARGET_PACKS':str(E/'huanle-packs.json'),'NONAME_CROSS_TARGET_NATIVE_OUT':str(O/'trajectory-package-native.json')}),
(['node',str(E/'wasm-package-trajectory.mjs'),str(E/'huanle-packs.json'),str(O/'trajectory-package-wasm.json')],{}),
([sys.executable,'-B',str(E/'compare.py'),str(O/'trajectory-package-native.json'),str(O/'trajectory-package-wasm.json')],{})]
for argv,extra in commands:
 print(json.dumps({'argv':argv,'extraEnvironment':extra}),flush=True);p=subprocess.run(argv,cwd=R,env={**os.environ,**extra});print(json.dumps({'exit':p.returncode}),flush=True)
 if p.returncode:sys.exit(p.returncode)
