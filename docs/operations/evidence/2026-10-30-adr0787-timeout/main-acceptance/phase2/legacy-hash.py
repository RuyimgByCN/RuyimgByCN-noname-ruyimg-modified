from pathlib import Path
import subprocess,json,sys
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent
rows=[]
def run(args):
 r=subprocess.run(args,cwd=ROOT,capture_output=True,text=True);rows.append({'argv':args,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr});(E/'legacy-child-commands.json').write_text(json.dumps(rows,indent=2)+'\n');print(r.stdout,end='');print(r.stderr,end='',file=sys.stderr);assert r.returncode==0,args;return r.stdout.strip()
checks=[]
for players,seed in [(4,42),(2,1)]:
 native=run(['cargo','run','--offline','-q','-p','engine-cli','--','hash',str(players),str(seed)])
 wasm=run(['node','scripts/wasm-hash.mjs',str(players),str(seed)])
 assert native==wasm,(native,wasm);checks.append({'players':players,'seed':seed,'nativeHash':native,'wasmHash':wasm})
manifest=run(['cargo','run','--offline','-q','-p','engine-cli','--','manifest','4','42']);assert 'chain=ok' in manifest and any(line.startswith('manifest_hash=') and len(line.split('=',1)[1])==64 for line in manifest.splitlines())
vm=run(['cargo','run','--offline','-q','-p','engine-cli','--','skill-vm','content/standard/jianxiong.skill']);assert 'ok=true' in vm
(E/'legacy-crosshash.json').write_text(json.dumps({'passed':True,'checks':checks,'manifestChain':manifest,'skillVm':vm,'conformance':'reused Phase1 currentmain ALLPASS; not rerun','productionWasmBuild':'production-legacy-build exact maintained script already ran','adaptation':'All original hash/manifest/chain/skillVM checks; omit redundant Phase1 conformance/build/installed rustup add'},indent=2)+'\n')
