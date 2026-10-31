from pathlib import Path
import json,hashlib,tarfile,tempfile,re,datetime,subprocess,collections
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9d');W=E.parent.parent/'worktree';h=lambda b:hashlib.sha256(b).hexdigest();j=lambda p:json.loads(p.read_bytes());r=j(E/'results-bound.json');idx=j(E/'artifact-index.json');root=Path(tempfile.mkdtemp(prefix='noname-parent-c9d-final-'));S=root/'snapshot';S.mkdir()
for x in idx['files']:b=(E/x['path']).read_bytes();assert h(b)==x['sha256'] and len(b)==x['bytes'],x['path']
for folder,mkey,tarname,tkey,patchname,pkey,count in [('full-source','composed_manifest_sha256','composed-source.tar.gz','composed_tar_sha256','vs-main5fc.patch','composed_patch_sha256',217),('final-freeze','increment_manifest_sha256','changed-source.tar.gz','increment_tar_sha256','vs-c9c.patch','increment_patch_sha256',18)]:
 d=E/folder;m=j(d/'source-manifest.json');assert len(m['files'])==count and h((d/'source-manifest.json').read_bytes())==r[mkey] and h((d/tarname).read_bytes())==r[tkey] and h((d/patchname).read_bytes())==r[pkey];expected={x['path']:x['sha256'] for x in m['files']}
 with tarfile.open(d/tarname,'r:gz') as t:
  files={x.name:x for x in t.getmembers() if x.isfile()};assert set(files)==set(expected)
  for name,sha in expected.items():
   p=Path(name);assert not p.is_absolute() and '..' not in p.parts;data=t.extractfile(files[name]).read();assert h(data)==sha and h((d/'source'/name).read_bytes())==sha and h((W/name).read_bytes())==sha
   if folder=='full-source':out=S/name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
   else:assert h((S/name).read_bytes())==sha
full={x['path'] for x in j(E/'full-source/source-manifest.json')['files']};changed=set(subprocess.check_output(['git','--no-optional-locks','-C',str(W),'diff','--name-only']).decode().splitlines());assert changed<=full,sorted(changed-full)
b=j(E/'command-source-bindings.json');assert h(Path(b['source_manifest']).read_bytes())==b['source_manifest_sha256']==r['increment_manifest_sha256'];parsed=[]
for c in b['commands']:
 p=E/'checks-final'/(c['name']+'.log');assert h(p.read_bytes())==c['log_sha256'] and c['source_manifest_sha256']==b['source_manifest_sha256'];rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',p.read_text(errors='replace'));counts=[sum(int(x[i]) for x in rows) for i in range(3)] if rows else None
 if c['name']=='full-native':assert counts==[1802,1,75] and c['exit']==101
 if c['name']=='gate1b':assert c['exit']==0 and c['RUSTFLAGS']=='-Dwarnings'
 parsed.append({'name':c['name'],'exit':c['exit'],'counts':counts})
g=j(E/'golden-business-and-outcome-dispositions.json');assert len(g['rows'])==12 and not g['normalization'] and not g['golden_expectations_changed'];pairs=0;outcomes=collections.Counter();diffs=[]
def classify(raw):
 if raw=='':return 'NoCommandOutcome'
 if re.fullmatch(r'Accepted \{ state_hash: \[(?:\d+(?:, )?){32}\] \}',raw):
  values=json.loads(raw[len('Accepted { state_hash: '):-2]);assert len(values)==32 and all(0<=x<=255 for x in values);return 'Accepted'
 for kind in ['Pending','Rejected']:
  if raw.startswith(kind+' {') or raw.startswith(kind+'('):return kind
 raise AssertionError(('unsupported full outcome format',raw[:120]))
for row in g['rows']:
 assert row['paired_inputs_and_phase_identical'] and row['source_versions']['C8_overlay_manifest']==g['baseline_manifest_sha256'];observed=[];assert len(row['captures'])==len(row['outcomes'])
 for cap,outcome in zip(row['captures'],row['outcomes']):
  assert cap['step']==outcome['step'];data={}
  for side in ['old','new']:
   p=Path(cap[side+'_path']);assert h(p.read_bytes())==cap[side+'_sha256'];data[side]=j(p)
   for suffix,ending in [('_snapshot_sha256','.snapshot.txt'),('_archive_sha256','.archive.ngsa')]:assert h(p.with_suffix(ending).read_bytes())==cap[side+suffix]
  a=data['old'];z=data['new'];assert a['command']==z['command'] and a['phase']==z['phase'] and a['ruleset_version']==z['ruleset_version'];fields=[key for key in ['core','events','rng','views','choices','referee'] if a[key]!=z[key]]
  if fields:observed.append({'step':cap['step'],'fields':fields})
  old=a['outcome_raw'];new=z['outcome_raw'];kind=classify(old);assert kind==classify(new)==outcome['discriminant'];assert old==outcome['old_raw'] and new==outcome['new_raw']
  if kind!='Accepted':assert old==new
  outcomes[kind]+=1;pairs+=1
 if observed:diffs.append({'scenario':row['scenario'],'differences':observed});assert row['first_business_difference'] is not None
 else:assert row['first_business_difference'] is None
assert pairs==362 and outcomes=={'Accepted':139,'Pending':32,'Rejected':4,'NoCommandOutcome':187} and [x['scenario'] for x in diffs]==['chain_fire_propagation']
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_INTEGRITY_BOUNDED_MIGRATION_REVIEW_PENDING_WHOLE_DAMAGE_BLOCK','root':str(root),'snapshot':str(S),'evidenceRoot':str(E),'artifactFiles':len(idx['files']),'fullFiles':217,'incrementFiles':18,'sourceManifest':r['composed_manifest_sha256'],'incrementManifest':r['increment_manifest_sha256'],'trackedDiffOutside217':[],'commands':parsed,'pairedCaptures':pairs,'actualOutcomeCounts':dict(outcomes),'nonAcceptedEntirePayloadEqual':True,'AcceptedExact32ByteHashRetainedNotCompared':True,'observedEqualScenarios':11,'observedBusinessDifferences':diffs,'qualification':'No product/golden/C331/Main acceptance. Needs connected-source review of41units/15helpers/admission/lifetimes/restore. Three concrete source defects Chilongya notLink,Zuijiu immediateSha ancestry,Hutian signed remainmandatoryrepairs; genuineChangeHp andDeathFlow stillabsent. No golden expected changed.'};Path('/tmp/noname-parent-c9d-final-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
