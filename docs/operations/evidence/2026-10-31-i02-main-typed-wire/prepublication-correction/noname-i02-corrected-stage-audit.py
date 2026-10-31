from pathlib import Path
import subprocess,json,hashlib,datetime,os
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');P=json.loads(Path('/tmp/noname-i02-corrected-stage-plan.json').read_bytes());expected={x['path']:x for x in P['files']};env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'};g=lambda *a:subprocess.check_output(['git','--no-optional-locks','-C',str(M),*a],env=env);h=lambda b:hashlib.sha256(b).hexdigest();assert g('rev-parse','HEAD').decode().strip()==g('rev-parse',P['protectedRef']).decode().strip()==P['withdrawnUnpublishedHead'];assert not g('diff','--name-only');changed=set(p.decode() for p in g('diff','--cached','--name-only','-z',P['expectedParent']).split(b'\0') if p);assert changed==set(expected),(len(changed),len(expected));entries={}
for row in g('ls-files','--stage','-z').split(b'\0'):
 if not row:continue
 header,path=row.split(b'\t',1);name=path.decode();mode,oid,stage=header.decode().split()
 if name in expected:assert stage=='0' and mode==expected[name]['mode'];entries[name]=(mode,oid)
assert len(entries)==len(expected);ordered=sorted(entries);data=subprocess.check_output(['git','--no-optional-locks','-C',str(M),'cat-file','--batch'],input=('\n'.join(entries[p][1] for p in ordered)+'\n').encode(),env=env);offset=0
for name in ordered:
 end=data.index(b'\n',offset);oid,kind,size=data[offset:end].decode().split();size=int(size);blob=data[end+1:end+1+size];assert oid==entries[name][1] and kind=='blob' and size==expected[name]['bytes'] and h(blob)==expected[name]['sha256']==h((M/name).read_bytes()),name;offset=end+2+size
assert offset==len(data)
for r in P['excludedLocalAliases']:
 assert r['path'] not in entries and (M/r['path']).is_file() and h((M/r['path']).read_bytes())==r['sha256']
for name in changed:
 parts=Path(name).parts
 assert not name.endswith('.assistant-record.raw.jsonl') and not any(('profile' in part.lower() and part.lower().startswith(('chrome','chromium','browser'))) for part in parts[:-1]),name
 assert '/original/tauri-preexisting-schemas/' not in name and not name.endswith(('.wasm','.rlib','.rmeta','.dylib','.exe','.o','.a','.so','.pyc')),name
prod={n for n in changed if not n.startswith('docs/')};assert prod==set(P['productionPaths']) and len(prod)==31
for n,sha in P['productionPaths'].items():assert expected[n]['sha256']==sha
assert h((M/'Cargo.lock').read_bytes())=='a9a79c887edb5cc0650ad3e0d9f595b55fcbe1be91d70fcde248378f1b86af50';assert h((M/'.git/hooks/pre-push').read_bytes())=='e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
tree=g('write-tree').decode().strip();removed=set(x['path'] for x in json.loads(Path('/tmp/noname-i02-correction-current.json').read_bytes())['removedTrackingOnly']);deltas=[]
names=g('diff','--cached','--name-status').decode().splitlines();deletes={x.split('\t',1)[1] for x in names if x.startswith('D\t')};assert deletes==removed and len(deletes)==495
for root in ['docs/operations/evidence/2026-10-31-i02-main-typed-wire/original/chrome-ingress-profile-ahhy89','docs/operations/evidence/2026-10-31-i02-main-typed-wire/original/tauri-preexisting-schemas']:assert not g('ls-tree','-r','--name-only',tree,'--',root)
for path in P['productionPaths']:assert not g('diff','--cached',P['withdrawnUnpublishedHead'],'--',path)
for page,sha in [('docs/operations/evidence/2026-10-31-c331-runtime-contract/approval-request.md','2859f6f07b188b2353a40d1138aa5b69e6c92ca67723e66016a189a8c78c5a0d'),('docs/operations/evidence/2026-10-31-i02-public-presentation-contract/approval-request.md','301496743727ca57d14eb9979f4519a6597a5e1b744125bb9383b7a40992250d')]:assert h((M/page).read_bytes())==sha
r={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_CORRECTED_STAGED_TREE_BLOBS_MODES_AND_EXCLUSIONS','tree':tree,'prospectiveParent':P['expectedParent'],'withdrawnLeaf':P['withdrawnUnpublishedHead'],'protectedLocalRef':P['protectedRef'],'files':len(expected),'bytes':P['bytes'],'productionPaths':sorted(prod),'allBlobsModesExact':True,'495TrackingRemovedPhysicalFilesRetained':True,'552ExcludedLocalAliasesRetained':True,'noBrowserProfileOrGeneratedBinaryPathsInNewTree':True,'original31ProductionBlobBytesUnchanged':True,'indexSha256':h((M/'.git/index').read_bytes()),'stagedDeltaVsWithdrawnLeaf':names,'historyPlan':'Normal amend owninvalidunpublishedleaf, preservingbadprotectedref+parentd07+allpublishedancestry; no reset/clean/stash/force/broadfilter. No runtimecheckrerunsnecessary.'};Path('/tmp/noname-i02-corrected-stage-audit.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k not in ['productionPaths','stagedDeltaVsWithdrawnLeaf']},ensure_ascii=False,indent=2))
