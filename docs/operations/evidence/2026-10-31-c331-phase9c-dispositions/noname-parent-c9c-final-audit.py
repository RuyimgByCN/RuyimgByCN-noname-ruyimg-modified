from pathlib import Path
import json,hashlib,tarfile,tempfile,re,datetime
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9c');h=lambda b:hashlib.sha256(b).hexdigest();j=lambda p:json.loads(p.read_bytes());r=j(E/'results-bound.json');idx=j(E/'artifact-index.json');root=Path(tempfile.mkdtemp(prefix='noname-parent-c9c-final-'));snapshot=root/'snapshot';snapshot.mkdir()
for x in idx['files']:b=(E/x['path']).read_bytes();assert h(b)==x['sha256'] and len(b)==x['bytes'],x['path']
for folder,manifest_key,tarname,tarkey,patchname,patchkey,count in [('full-source','full_manifest_sha256','composed-source.tar.gz','full_tar_sha256','vs-main5fc.patch','full_patch_sha256',214),('final-freeze','increment_manifest_sha256','changed-source.tar.gz','increment_tar_sha256','vs-c9b.patch','increment_patch_sha256',21)]:
 d=E/folder;m=j(d/'source-manifest.json');assert len(m['files'])==count and h((d/'source-manifest.json').read_bytes())==r[manifest_key] and h((d/tarname).read_bytes())==r[tarkey] and h((d/patchname).read_bytes())==r[patchkey];expected={x['path']:x['sha256'] for x in m['files']}
 with tarfile.open(d/tarname,'r:gz') as t:
  files={x.name:x for x in t.getmembers() if x.isfile()};assert set(files)==set(expected)
  for name,sha in expected.items():
   p=Path(name);assert not p.is_absolute() and '..' not in p.parts;data=t.extractfile(files[name]).read();assert h(data)==sha
   if folder=='full-source':out=snapshot/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
   else:assert h((snapshot/name).read_bytes())==sha
b=j(E/'command-source-bindings.json');assert h(Path(b['source_manifest']).read_bytes())==b['source_manifest_sha256'];final=j(E/'final-freeze/source-manifest.json');frozen={x['path']:x['sha256'] for x in j(Path(b['source_manifest']))['files']}
for x in final['files']:assert frozen[x['path']]==x['sha256']
parsed=[]
for c in b['commands']:
 p=E/'checks-final'/(c['name']+'.log');assert h(p.read_bytes())==c['log_sha256'] and c['source_manifest_sha256']==b['source_manifest_sha256'];rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',p.read_text(errors='replace'));counts=[sum(int(x[i]) for x in rows) for i in range(3)] if rows else None
 if c['name']=='full-native':assert counts==[1796,1,75] and c['exit']==101
 if c['name']=='gate1b':assert c['exit']==0 and c['RUSTFLAGS']=='-Dwarnings'
 parsed.append({'name':c['name'],'exit':c['exit'],'counts':counts})
g=j(E/'golden-business-dispositions.json');assert len(g['rows'])==12 and not g['normalization'];pairs=0
for row in g['rows']:
 assert row['paired_command_inputs_identical'] and row['source_versions']['C8_overlay_manifest']=='f5829f4023ee465e5e97fe6ad8f998b1480999cd727c348b310686dcdf72876e'
 for cap in row['captures']:
  for side in ['old','new']:
   for suffix in ['','_snapshot','_archive']:
    pk=side+suffix+'_path';sk=side+suffix+'_sha256'
    if pk in cap:assert h(Path(cap[pk]).read_bytes())==cap[sk],pk
  pairs+=1
assert pairs==362
record={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_INTEGRITY_ONLY_REMAINS_BLOCK','root':str(root),'snapshot':str(snapshot),'evidenceRoot':str(E),'artifactFiles':len(idx['files']),'fullFiles':214,'incrementFiles':21,'sourceManifest':r['full_manifest_sha256'],'commands':parsed,'businessDispositionRows':12,'pairedCaptures':pairs,'observedEqualScenarios':sum(x['first_business_difference'] is None for x in g['rows']),'actualDifferenceScenarios':[x['scenario'] for x in g['rows'] if x['first_business_difference'] is not None],'qualification':'No runtime/golden/C331/Main acceptance; unowned/other inline window priority remains P1, new internal body/restore repair and 12 semantic dispositions await independent connected-source review. No golden expected rerecorded.'};Path('/tmp/noname-parent-c9c-final-audit.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n');print(json.dumps(record,ensure_ascii=False,indent=2))
