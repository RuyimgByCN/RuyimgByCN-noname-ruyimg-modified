from pathlib import Path
import json,hashlib,datetime
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent;P=Path(json.loads((E/'product-root.json').read_text())['path'])
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();files={}
for f in P.rglob('*'):
 relative=f.relative_to(P).parts
 if len(relative)>1 and relative[0] in ['chrome-fast','chrome-slow'] and relative[1].startswith('chrome-'):continue
 if f.is_file():files[str(f)]={'bytes':f.stat().st_size,'sha256':sha(f)}
for directory in ['apps/web-client/src/worker/native-pkg','apps/web-client/src/wasm-pkg','apps/web-client/public/pkg']:
 for f in (ROOT/directory).rglob('*'):
  if f.is_file():files[str(f)]={'bytes':f.stat().st_size,'sha256':sha(f)}
(E/'artifact-sha256.json').write_text(json.dumps({'time':datetime.datetime.now().isoformat(),'scope':'all retained owned external artifacts/inputs/harnesses/WAL/server/UI/screenshots; current three generatedpkgdirs; Chrome temp profile bytes explicitly excluded; cleanup audited separately','files':files},indent=2)+'\n');print('artifact files hashed',len(files),'bytes',sum(x['bytes'] for x in files.values()))
