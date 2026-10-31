from pathlib import Path
import subprocess,json,time,hashlib,shutil,difflib,sys
root=Path('/tmp/i02-phase1-ingress-repair-89364384');src=Path('/tmp/i02-phase1-ab167599/source');prior=Path('/tmp/i02-phase1-ab167599/acceptance-checkpoint');out=root/sys.argv[1];(out/'source').mkdir(parents=True)
old=json.loads((prior/'manifest.json').read_text());files=list(old['sources']);
extra='crates/engine-protocol/tests/fixtures/i02-presentation-surrogates.json'
if (src/extra).exists(): files.append(extra)
tracked=set(subprocess.check_output(['git','ls-files'],cwd=src).decode().splitlines());full=subprocess.check_output(['git','diff','--binary'],cwd=src);delta=b''
for f in files:
 data=(src/f).read_bytes();q=out/'source'/f;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(data)
 text=data.decode();lines=text.split('\n');actual=[x+'\n' for x in lines[:-1]]+([lines[-1]] if lines[-1] else [])
 if f not in tracked:full+=('diff --git a/'+f+' b/'+f+'\nnew file mode 100644\n'+''.join(difflib.unified_diff([],actual,fromfile='/dev/null',tofile='b/'+f))).encode()
 original=(prior/'source'/f).read_text() if (prior/'source'/f).exists() else '';rows=original.split('\n');oldlines=[x+'\n' for x in rows[:-1]]+([rows[-1]] if rows[-1] else [])
 if original!=text:delta+=('diff --git a/'+f+' b/'+f+'\n'+('new file mode 100644\n' if not (prior/'source'/f).exists() else '')+''.join(difflib.unified_diff(oldlines,actual,fromfile='a/'+f if (prior/'source'/f).exists() else '/dev/null',tofile='b/'+f))).encode()
(out/'full.patch').write_bytes(full);(out/'repair.patch').write_bytes(delta);(out/'base.txt').write_text(old['base']+'\n');shutil.copytree(root/'evidence',out/'evidence')
checks={}
for name in ['full.patch','repair.patch']:
 cmd=['git','apply','--reverse','--check',str(out/name)];r=subprocess.run(cmd,cwd=src,capture_output=True,text=True);checks[name]={'argv':cmd,'exit':r.returncode,'stderr':r.stderr}
 assert r.returncode==0,r.stderr
(out/'patch-checks.json').write_text(json.dumps(checks,indent=2))
manifest={'base':old['base'],'previous_patch_sha256':old['full_patch_sha256'],'source_file_count':len(files),'sources':{f:hashlib.sha256((out/'source'/f).read_bytes()).hexdigest() for f in files},'repair_files':[f for f in files if not (prior/'source'/f).exists() or (prior/'source'/f).read_bytes()!=(out/'source'/f).read_bytes()],'full_patch_sha256':hashlib.sha256(full).hexdigest(),'repair_patch_sha256':hashlib.sha256(delta).hexdigest(),'created':time.time(),'stage':sys.argv[2] if len(sys.argv)>2 else 'in-progress'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(out);print(json.dumps({k:v for k,v in manifest.items() if k!='sources'},indent=2))
