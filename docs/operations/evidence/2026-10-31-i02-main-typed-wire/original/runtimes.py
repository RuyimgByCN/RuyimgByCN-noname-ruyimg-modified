import sys,json,shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from run import run,E,R,sha
for name,argv in [('ingress-chrome',['node',str(E/'run-ingress.mjs'),'chrome']),('ingress-tauri',['node',str(E/'run-ingress.mjs'),'tauri']),('server-wire',['node','--import',str(R/'apps/web-client/scripts/ts-register.mjs'),str(E/'server-wire.mjs')])]:
 if run(name,argv):sys.exit(1)
oldLegacy=sha(R/'apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm')
if run('legacy-wasm-build',['node','scripts/build-wasm.mjs'],R/'apps/web-client',{'NODE_OPTIONS':'--require='+str(E/'target-preload.cjs')}):sys.exit(1)
newLegacy=sha(R/'apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm');(E/'legacy-rebuild-identity.json').write_text(json.dumps({'before':oldLegacy,'after':newLegacy,'reason':'I02 source30 modifies engine-api transitive input; unchanged-input inheritance is not valid. Original release/optimizer/codegen script retained.'},indent=2))
if oldLegacy!=newLegacy:
 for name in ['frontend-dist','bundle-module-table.json','browser-results.json','own-optional.png','product.png']:
  p=E/name
  if p.exists():p.rename(E/(name+'-before-legacy'))
 for name,argv,cwd in [('vite-final',['node',str(E/'vite-build.mjs')],R/'apps/web-client'),('own-browser-final',['node',str(E/'own-browser.mjs'),str(E/'frontend-dist'),'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',str(E)],R)]:
  if run(name,argv,cwd):sys.exit(1)
if run('legacy-crosshash-final',['python3','-B',str(E/'legacy-crosshash.py')]):sys.exit(1)
print('ALL REAL INGRESS/IPC/WS AND LEGACY IDENTITIES PASS',flush=True)
