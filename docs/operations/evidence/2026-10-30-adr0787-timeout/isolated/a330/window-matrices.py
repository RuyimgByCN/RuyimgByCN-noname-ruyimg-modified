import os,subprocess,pathlib,sys
root=pathlib.Path.cwd();e=(root.parent/'evidence/a330').resolve(); (e/'machine-traces').mkdir(exist_ok=True)
env=dict(os.environ,NC09_MACHINE_TRACE_OUT=str(e/'machine-traces'),J2_CROSS_NATIVE_OUT=str(e/'j2-matrix.json'))
steps=[['cargo','test','--offline','-p','engine-native','--test','misc','nc09_machine_ownership::','--','--nocapture'],['node','docs/operations/evidence/2026-10-30-nc09-referee/review-fix/wasm-machine-windows.mjs',str(root),str(e)],['cargo','test','--offline','-p','engine-native-wasm','--test','adr0786_j2_cross_target','--','--nocapture'],['node','apps/web-client/scripts/adr0786-j2-cross-target.mjs',str(e/'j2-matrix.json'),str(root/'apps/web-client/src/worker/native-pkg'),str(e/'j2-wasm-matrix.json')]]
for argv in steps:
 print('RUN',argv,flush=True);r=subprocess.run(argv,env=env)
 if r.returncode:sys.exit(r.returncode)
