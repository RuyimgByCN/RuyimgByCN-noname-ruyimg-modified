from pathlib import Path
import json,subprocess,hashlib,datetime,tarfile,sys
root=Path(__file__).resolve().parents[2];work=root/'worktree';ev=Path(__file__).parent;dest=ev/sys.argv[1];dest.mkdir(exist_ok=False);inventory=json.loads(Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/c331-main5fc-inventory-1c6ba2f3-t3tubl5j/inventory.json').read_text());paths={r['path']for r in inventory['rows']};tracked=subprocess.check_output(['git','diff','--name-only','HEAD'],cwd=work,text=True).splitlines();extra=subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=work,text=True).splitlines();paths|=set(tracked)|set(extra);paths=sorted(p for p in paths if(work/p).is_file());head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=work,text=True).strip();assert head=='5fc0f77403d90fa4555156bf01722450cbe7bf3b';assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=work).strip();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();rows=[{'path':p,'sha256':sha(work/p)}for p in paths];baseline={}
for p in paths:
 q=subprocess.run(['git','rev-parse','HEAD:'+p],cwd=work,text=True,capture_output=True);baseline[p]=q.stdout.strip()if q.returncode==0 else None
manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'base':head,'files':rows,'baseline_blobs':baseline,'old_c6_base':'38ab1678df9b4b7b991d3b3122b1d937fd2648a1','c6_frozen_manifest':'81d66d59f0ca336761f28629850f9daf7a955670e7c17725cde40b88a1ba21e7','private_intermediate':'C331/NGSA106; later approved Core/schema work still pending; not published stable106','no_staged_files':True};(dest/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
with tarfile.open(dest/'composed-source.tar.gz','w:gz')as t:
 for p in paths:t.add(work/p,arcname=p)
patch=subprocess.check_output(['git','diff','--binary','HEAD'],cwd=work)
for p in extra:
 if(work/p).is_file():patch+=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',p],cwd=work,stdout=subprocess.PIPE).stdout
(dest/'vs-main5fc.patch').write_bytes(patch);files=[p for p in ev.rglob('*')if p.is_file()and not any(v.startswith(('checkpoint','pre-full','final-freeze'))for v in p.relative_to(ev).parts)];evidence=[{'path':str(p.relative_to(ev)),'sha256':sha(p),'bytes':p.stat().st_size}for p in sorted(files)];(dest/'evidence-manifest.json').write_text(json.dumps(evidence,indent=2)+'\n')
for p in list(dest.iterdir()):(dest/(p.name+'.sha256')).write_text(sha(p)+'  '+p.name+'\n')
print(json.dumps({'root':str(root),'path':str(dest),'source_files':len(rows),'manifest_sha256':sha(dest/'source-manifest.json'),'tar_sha256':sha(dest/'composed-source.tar.gz'),'patch_sha256':sha(dest/'vs-main5fc.patch')}))
