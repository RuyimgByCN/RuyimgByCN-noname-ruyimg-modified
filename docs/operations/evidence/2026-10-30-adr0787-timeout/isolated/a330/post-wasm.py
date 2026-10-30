import subprocess,sys,pathlib
root=pathlib.Path.cwd();e=root.parent/'evidence/a330'
steps=[['node',str(e/'timeout-wasm.mjs'),str(e/'timeout-parity.json'),str(e/'timeout-wasm-final.json')],['node','--import','./apps/web-client/scripts/ts-register.mjs','apps/web-client/scripts/native-worker-smoke.mjs'],['node','docs/operations/evidence/2026-10-30-nc09-referee/review-fix/wasm-machine-windows.mjs',str(root),str(e)],['node','apps/web-client/scripts/adr0786-j2-cross-target.mjs',str(e/'j2-matrix.json'),str(root/'apps/web-client/src/worker/native-pkg'),str(e/'j2-wasm-final.json')]]
for argv in steps:
 print('RUN',argv,flush=True);r=subprocess.run(argv)
 if r.returncode:sys.exit(r.returncode)
