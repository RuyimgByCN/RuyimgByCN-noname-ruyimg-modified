from pathlib import Path
import subprocess,json,importlib.util
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('frozen_effective_counter',root.parent/'final-files/scripts/rust_effective_lines.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
commits=subprocess.check_output(['git','rev-list','--reverse','95f51867a3648321da1cc92149b60e2bbfaae748..HEAD']).decode().splitlines();cache={};results=[]
for i,h in enumerate(commits,1):
 entries=[]
 for line in subprocess.check_output(['git','ls-tree','-r',h]).decode().splitlines():
  meta,path=line.split('\t',1);sha=meta.split()[2]
  if path.endswith('.rs') and 'tests' not in Path(path).parts and path.split('/')[0] in ['crates','apps','packages','scripts','content','fuzz','tests']:entries.append((path,sha))
 missing=list(dict.fromkeys(sha for p,sha in entries if sha not in cache))
 if missing:
  raw=subprocess.check_output(['git','cat-file','--batch'],input=('\n'.join(missing)+'\n').encode());offset=0
  for sha in missing:
   end=raw.index(b'\n',offset);header=raw[offset:end].split();size=int(header[-1]);content=raw[end+1:end+1+size].decode();cache[sha]=module.effective_lines(content);offset=end+size+2
 over=[{'path':p,'effective_lines':cache[sha]} for p,sha in entries if cache[sha]>1000];results.append({'batch':i,'commit':h,'over_1000_count':len(over),'limits_enforced_at_this_batch':i==9,'files':over})
 if i==9:assert not over
(root/'effective-lines-by-tree.json').write_text(json.dumps(results,indent=2));print(json.dumps([{k:v for k,v in r.items() if k!='files'} for r in results],indent=2))
