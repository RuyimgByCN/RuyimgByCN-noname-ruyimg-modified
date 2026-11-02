from pathlib import Path
import json,hashlib,subprocess,os,datetime,tarfile,re,shutil
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';e=Path(__file__).parent;sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();run=lambda args,cwd=w:subprocess.check_output(args,cwd=cwd,text=True).strip()
c9=r/'evidence/phase9f';assert sha(c9/'artifact-index.json')=='a1e420df4c11a375d90319744e555be44a26b65f69aede31a2b64fbe6fb8b552';index=json.loads((c9/'artifact-index.json').read_text());fail=[x['path']for x in index['files']if sha(c9/x['path'])!=x['sha256']];assert not fail,fail
proofs=[]
for rel in ['phase7/artifact-index.json','phase8/artifact-index.json','phase8/admission-repair/artifact-index.json','phase9/artifact-index.json','phase9b/artifact-index.json','phase9c/artifact-index.json','phase9d/artifact-index.json']:
 p=r/'evidence'/rel;data=json.loads(p.read_text());bad=[x['path']for x in data['files']if sha(p.parent/x['path'])!=x['sha256']];assert not bad;proofs.append({'path':str(p),'sha256':sha(p),'files_checked':len(data['files']),'mismatches':bad})
base=json.loads((c9/'full-source/source-manifest.json').read_text());final=json.loads((e/'final-freeze6/source-manifest.json').read_text());changed={x['path']for x in final['files']};assert all(sha(w/x['path'])==x['sha256']for x in final['files']);unchanged=[x['path']for x in base['files']if x['path']not in changed and sha(w/x['path'])!=x['sha256']];assert not unchanged,unchanged
assert not run(['git','diff','--cached','--name-only']);assert run(['git','rev-parse','HEAD'])=='5fc0f77403d90fa4555156bf01722450cbe7bf3b'
# Keep original Main31867 snapshot and exact parent authorizations separate.
before=json.loads((r/'evidence/phase7/protected-before.json').read_text());main=Path(before['main']);own_p=Path('/tmp/noname-parent-nc10-main-integration-current.json');own=json.loads(own_p.read_text());own_allowed={x['path']:x['sha256']for x in own['files']};i02_p=Path('/tmp/noname-parent-i02-main-integration-current.json');i02=json.loads(i02_p.read_text());ci_p=Path('/tmp/noname-parent-windows-checkout-main-integration.json');ci=json.loads(ci_p.read_text());allowed={**own_allowed,**i02['source30'],ci['path']:ci['after']};deltas=[];bad=[]
for row in before['files']:
 p=main/row['path'];cur={'symlink':os.readlink(p)}if p.is_symlink()else{'sha256':sha(p)}if p.is_file()else{}
 if any(cur.get(k)!=row[k]for k in ['sha256','symlink']if k in row):
  if cur.get('sha256')==allowed.get(row['path']):deltas.append(row['path'])
  else:bad.append(row['path'])
assert not bad,bad
assert all(sha(main/p)==h for p,h in allowed.items())
prod_p=Path('/tmp/i02-main-typed-wire-functional-33d21ba5/product-binding.json');prod=json.loads(prod_p.read_text());gen=json.loads((r/'evidence/phase7/main-generated18-current.json').read_text());approved={p:(prod['currentNativeWasmSha256']if'native-pkg'in p else prod['currentLegacyWasmSha256'])for p in prod['changedGenerated18Paths']};gd=[]
for row in gen['files']:
 current=sha(main/row['path'])
 if current!=row['sha256']:assert current==approved.get(row['path']);gd.append({'path':row['path'],'original':row['sha256'],'current':current})
private_gen=[{'path':x['path'],'expected':x['sha256'],'actual':sha(w/x['path'])}for x in gen['files']if(w/x['path']).is_file()];assert all(x['expected']==x['actual']for x in private_gen)
schemas_p=Path('/tmp/i02-main-typed-wire-functional-33d21ba5/tauri-schemas-after.json');schemas=json.loads(schemas_p.read_text())['after'];assert len(schemas)==4
for p,row in schemas.items():
 q=main/p;st=q.stat();assert sha(q)==row['sha256']and st.st_mtime_ns==row['mtimeNs']and st.st_size==row['bytes']and(st.st_mode&0o777)==row['mode']
orig=[x['path']for x in before['original_c140']if sha(Path(before['original_c'])/x['path'])!=x['sha256']];hooks=[x['path']for x in before['hooks']if sha(Path(x['path']))!=x['sha256']];assert not(orig or hooks)
head_p=Path('/tmp/noname-i02-corrected-accepted-commit.json');head=json.loads(head_p.read_text());main_head=run(['git','rev-parse','HEAD'],main);doc_p=Path('/tmp/noname-post-c9e-doc-accepted-commit.json');doc=json.loads(doc_p.read_text());assert main_head==doc['head']=='9f4b74236f97b0b9dc72388b6e08d310d2f26f64';assert doc['parent']==head['commit']=='34e83f0550bb70ef4af6083697428de7be2f02f8';doc_paths=run(['git','diff','--name-only',doc['parent'],main_head],main).splitlines();assert len(doc_paths)==9905 and all(p.startswith('docs/')for p in doc_paths);assert run(['git','rev-parse','HEAD^{tree}'],main)==doc['tree'];assert head['parent']=='d07b35f94a42b282222dda4ab7bbc5c39ab8fddb'
main_diff=run(['git','diff','--name-only'],main).splitlines();main_staged=run(['git','diff','--cached','--name-only'],main).splitlines();assert all(p.startswith('docs/')or p in allowed for p in main_diff+main_staged),[p for p in main_diff+main_staged if not(p.startswith('docs/')or p in allowed)]
helper=Path('/tmp/noname-run-serialized.py');lock=Path('/tmp/noname-phase4-heavy-command.lock').stat();assert sha(helper)=='d7f246f1fdbd4ec6d85deb22643d7ca8137be62539ebd5e75d49c779febb8e34';assert lock.st_dev==16777231 and lock.st_ino==620205209
old_reader=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target/old330/debug/examples/c331_old_reader');assert sha(old_reader)=='5ec8147491571b7c0145b2a31b8ab003d37fb5c9839ba1a31a668e83712abac4'
protected={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_main_baseline':str(r/'evidence/phase7/protected-before.json'),'baseline_count':len(before['files']),'unauthorized_main_deltas':bad,'authorized_exact_deltas':deltas,'Own6':own_allowed,'I02_30':i02['source30'],'CI':{ci['path']:ci['after']},'main_head_observed':main_head,'head_status':'Exact Parent docs-only9f4b commit/tree9905 receipt-bound; runtime source37 remains330; historical34e CI is not C validation','main_stage_paths':main_staged,'main_working_diff_paths':main_diff,'generated18_deltas':gd,'private_original_generated12':private_gen,'schema4_bytes_modes_mtimes':schemas,'original_c6_140_mismatches':orig,'hooks_mismatches':hooks,'immutable_indexes':proofs,'c9e_artifacts_verified':len(index['files']),'c9e_sources_outside_increment_mismatches':unchanged,'old330_reader':{'path':str(old_reader),'sha256':sha(old_reader)},'private_staged_empty':True,'private_base':run(['git','rev-parse','HEAD']),'default_dist_original_count':sum(x['path'].startswith('apps/web-client/dist/')for x in before['files']),'receipts':[{'path':str(p),'sha256':sha(p)}for p in[own_p,i02_p,ci_p,prod_p,schemas_p,head_p]],'global_serialization':{'helper_sha256':sha(helper),'lock_device':lock.st_dev,'lock_inode':lock.st_ino}};(e/'protected-after.json').write_text(json.dumps(protected,indent=2)+'\n')

for name,pin in [('phase9e','ffc132bb84468ac24f31f0e7ed99dfc3695f2626ecca4b3c4d6cff61cb5a893a'),('phase9e-concordance-supplement','abb59477b0f3290729e8e98e0fd25933f8c733b4fff9ccdbf9881c8fe8038108')]:
 old=r/'evidence'/name;assert sha(old/'artifact-index.json')==pin;ix=json.loads((old/'artifact-index.json').read_text());assert all(sha(old/x['path'])==x['sha256']for x in ix['files'])
paths=sorted({x['path']for x in base['files']}|changed);full=e/'full-source6';full.mkdir();rows=[]
with tarfile.open(full/'composed-source.tar.gz','w:gz')as out:
 for p in paths:
  q=w/p;dest=full/'source'/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(q.read_bytes());out.add(q,arcname=p);rows.append({'path':p,'sha256':sha(q)})
(full/'source-manifest.json').write_text(json.dumps({'base_commit':run(['git','rev-parse','HEAD']),'base_c9f_manifest_sha256':sha(c9/'full-source/source-manifest.json'),'increment_manifest_sha256':sha(e/'final-freeze6/source-manifest.json'),'files':rows,'scope':'PARTIAL genuine invocations; no C331/Main/ABI acceptance'},indent=2)+'\n')
patch=subprocess.check_output(['git','diff','--binary','HEAD'],cwd=w)
for p in subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines():
 if(w/p).is_file()and not(w/p).is_symlink():patch+=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',p],cwd=w,stdout=subprocess.PIPE).stdout
(full/'vs-main5fc.patch').write_bytes(patch)
for p in[e/'final-freeze6/vs-c9f.patch',full/'vs-main5fc.patch']:
 z=subprocess.run(['git','apply','--reverse','--check',str(p)],cwd=w,capture_output=True,text=True);assert z.returncode==0,z.stderr
for d in[e/'final-freeze6/source',full/'source']:
 for p in d.rglob('*'):
  if p.is_file():p.chmod(0o444)
commands=[]
for directory,freeze in [('checks-full1','pre-full2'),('checks-final2','final-freeze2'),('checks-final3','final-freeze3')]:
 checks=json.loads((e/directory/'check-results.json').read_text());manifest=e/freeze/'complete-source-manifest.json';covered=json.loads(manifest.read_text());mismatch=[x['path']for x in covered['files']if sha(w/x['path'])!=x['sha256']]
 for c in checks:commands.append(dict(c,producer=directory,manifest=str(manifest),manifest_sha256=sha(manifest),before_after_assertions=True,log_sha256=sha(e/directory/(c['name']+'.log')),current_source_mismatches=mismatch))
(e/'command-source-bindings.json').write_text(json.dumps({'final_manifest':str(full/'source-manifest.json'),'commands':commands,'qualification':'Final-freeze3 strict+NativeSession passed; later server-private WAL control arm and native fault/count guards are NOT compiled/runtime checked after cutoff; full native remains unexecuted on final bytes.'},indent=2)+'\n')
result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'increment_files':len(final['files']),'composed_files':len(rows),'increment_manifest_sha256':sha(e/'final-freeze6/source-manifest.json'),'increment_tar_sha256':sha(e/'final-freeze6/changed-source.tar.gz'),'increment_patch_sha256':sha(e/'final-freeze6/vs-c9f.patch'),'composed_manifest_sha256':sha(full/'source-manifest.json'),'composed_tar_sha256':sha(full/'composed-source.tar.gz'),'composed_patch_sha256':sha(full/'vs-main5fc.patch'),'no_staged_files':True,'whole_c331':'BLOCK;262/290 remaining28 +0','status':'PARTIAL_NOT_ACCEPTED_FINAL_FULL_RUNTIME_NOT_RUN'};(e/'results-bound.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
