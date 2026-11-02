from pathlib import Path
import json,hashlib,re,datetime,collections
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9h-baseline-validation');j=lambda p:json.loads(p.read_bytes());h=lambda b:hashlib.sha256(b).hexdigest();idx=j(E/'artifact-index.json');manifest=j(E/'source-manifest.json');S=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-parent-c9g-final-pakwy5rq/snapshot');M={x['path']:x['sha256'] for x in manifest['files']};assert len(M)==239 and len(idx['files'])==1917 and h((E/'source-manifest.json').read_bytes())=='69eeebab43d3ccea5f0b48e003cecd8a339f2fe1df8cf8e8d75e118a5309087b';assert all(h((S/p).read_bytes())==sha for p,sha in M.items())
for x in idx['files']:
 p=Path(x['path']);assert not p.is_absolute() and '..' not in p.parts;data=(E/p).read_bytes();assert len(data)==x['bytes'] and h(data)==x['sha256'],str(p)
ref=j(E/'report-reference.json');assert h(Path(ref['path']).read_bytes())==ref['sha256']==h((E/'report.md').read_bytes());commands=j(E/'command-results.json');assert len(commands)==9;parsed=[];full_failure_names=[]
for c in commands:
 name=c['name'];log=(E/(name+'.log')).read_bytes();assert h(log)==c['log_sha256']
 for side in ['before','after']:
  p=E/(name+'-source-'+side+'.json');assert h(p.read_bytes())==c['source_'+side+'_sha256'];m=j(p);assert m['manifest_sha256']==h((E/'source-manifest.json').read_bytes());assert {x['path']:x['sha256'] for x in m['files']}==M
 text=log.decode(errors='replace');rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);counts=[sum(int(x[i]) for x in rows) for i in range(3)] if rows else None
 if name=='full-native':assert c['exit']==101 and counts==[1848,9,76];full_failure_names=[line.strip()[len('test '):].removesuffix(' ... FAILED') for line in text.splitlines() if line.startswith('test ') and line.endswith(' ... FAILED')];assert len(full_failure_names)==9
 if name=='gate1b':assert c['exit']==0 and c['RUSTFLAGS']=='-Dwarnings' and 'target-strict' in c['target']
 if name=='server-check':assert c['exit']==0
 if name=='wasm-controls':assert c['exit']==0 and counts==[1,0,0]
 if name=='focused':assert c['exit']==0 and counts==[98,0,1]
 if name=='golden':assert c['exit']==101 and counts==[0,1,0]
 parsed.append({'name':name,'exit':c['exit'],'counts':counts,'warningLineOccurrences':len(re.findall(r'^warning',text,re.MULTILINE)),'beforeAfter239Match':True,'seconds':c['seconds']})
fails=j(E/'full-native-failures.json');assert len(fails['failures'])==9;g=j(E/'golden-business-and-outcome-dispositions.json');assert len(g['rows'])==12 and not g['normalization'] and not g['golden_expectations_changed'];pairs=0;outcomes=collections.Counter();diffs=[]
def classify(raw):
 if raw=='':return 'NoCommandOutcome'
 if re.fullmatch(r'Accepted \{ state_hash: \[(?:\d+(?:, )?){32}\] \}',raw):
  data=json.loads(raw[len('Accepted { state_hash: '):-2]);assert len(data)==32 and all(0<=x<=255 for x in data);return 'Accepted'
 for kind in ['Pending','Rejected']:
  if raw.startswith(kind+' {') or raw.startswith(kind+'('):return kind
 raise AssertionError(('unknown full outcome format',raw[:120]))
for row in g['rows']:
 assert row['paired_inputs_and_phase_identical'];seen=[];assert len(row['captures'])==len(row['outcomes'])
 for cap,outcome in zip(row['captures'],row['outcomes']):
  assert cap['step']==outcome['step'];data={}
  for side in ['old','new']:
   p=Path(cap[side+'_path']);assert h(p.read_bytes())==cap[side+'_sha256'];data[side]=j(p)
   for suffix,ending in [('_snapshot_sha256','.snapshot.txt'),('_archive_sha256','.archive.ngsa')]:assert h(p.with_suffix(ending).read_bytes())==cap[side+suffix]
  a=data['old'];z=data['new'];assert a['command']==z['command'] and a['phase']==z['phase'] and a['ruleset_version']==z['ruleset_version'];fields=[k for k in ['core','events','rng','views','choices','referee'] if a[k]!=z[k]]
  if fields:seen.append({'step':cap['step'],'fields':fields})
  old=a['outcome_raw'];new=z['outcome_raw'];kind=classify(old);assert kind==classify(new)==outcome['discriminant'];assert old==outcome['old_raw'] and new==outcome['new_raw']
  if kind!='Accepted':assert old==new
  pairs+=1;outcomes[kind]+=1
 if seen:diffs.append({'scenario':row['scenario'],'differences':seen});assert row['first_business_difference'] is not None
 else:assert row['first_business_difference'] is None
assert pairs==362 and outcomes=={'Accepted':139,'Pending':32,'Rejected':4,'NoCommandOutcome':187};assert [x['scenario'] for x in diffs]==['chain_fire_propagation'];oldg=j(E.parent/'phase9f/golden-business-and-outcome-dispositions.json');assert [(x['scenario'],x['original_expected_hash'],x['current_actual_hash']) for x in oldg['rows']]==[(x['scenario'],x['original_expected_hash'],x['current_actual_hash']) for x in g['rows']]
predicate=j(E/'predicate-capture-bindings.json');assert predicate['triples']==len(predicate['captures'])==202;inv=0
for row in predicate['captures']:
 p=E/row['json'];assert h(p.read_bytes())==row['json_sha256'];assert h(p.with_suffix('.snapshot.txt').read_bytes())==row['snapshot_sha256'];assert h(p.with_suffix('.archive.ngsa').read_bytes())==row['archive_sha256'];actual='NativeInvocation' in p.with_suffix('.snapshot.txt').read_text();assert actual==row['contains_native_invocation'];inv+=actual
assert inv==predicate['contains_new_invocation_in_existing_capture']==15;after=j(E/'after.json');assert after['all239_source_bytes_unchanged'] and after['active_owned_producers']==after['queued_owned_producers']==0 and not after['source_test_fixture_catalog_edits'];protected=j(E/'protected-after.json');assert protected['Parent_docs4406_verified'] and protected['Parent_docs_bytes']==143909122 and not protected['unauthorized_original_baseline_deltas'] and protected['Main_staging_empty'] and protected['source239_unchanged'];assert protected['no_original_baseline_refresh'];assert j(E/'result.json')['C9g_capture_export_complete'] is False
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_INDEPENDENT_FINAL6_VALIDATION_REGISTRATION_COMPILED_FULL_NATIVE_FAILED_9','evidenceRoot':str(E),'artifactFiles':1917,'artifactIndexSHA':h((E/'artifact-index.json').read_bytes()),'reportSHA':ref['sha256'],'source239ManifestSHA':h((E/'source-manifest.json').read_bytes()),'sourceComparedAgainst':'ImmutableParentC9g239snapshot, NOT liveCwhichnextwriter maynowedit','commands':parsed,'fullNativeFailureNames':full_failure_names,'final6Compiled':True,'final6FullNativePassed':False,'fullNative':[1848,9,76],'sourceEditsInValidation':False,'goldenPairs':pairs,'outcomes':dict(outcomes),'AcceptedHash32ByteRetainedSeparate':True,'businessDiffs':diffs,'old12ExpectedActualHashesIdenticalToC9f':True,'predicateCaptureTriples':202,'snapshotsWithInvocation':15,'completeNewC9gBranchCaptureExport':False,'MainProtectedReportedPostGuard':protected['utc'],'MainRawIndexAtPostGuard':protected['Main_raw_index_sha256'],'preMainAuthorityQualification':protected['Main_prephase_note'],'qualification':'Original final6 now actually compiles/strict/server/NativeSession pass; full regression remainsFAILED9 with explicit assertion sections. Earlier9gclaimUNCOMPILEDstillhistoricalaccurateatsealtime; do notrewriteoldreport. Newrepair57owns live C: no livehclaimsfromthisaudit. No compiler/source/test edits, reruns, builds, Main/index/stage/HEAD/publicationchanges from Parent audit. Sourcecode reviewee6BLOCKfournewP1+F07F14, stableIDs/forgedpayments/GuiCai/virtualJiding notfixedbygreencargo. Old12goldenredunchanged/no rerecord;11observedbusinesssame+chaineventorder notgeneralprivacy/fuel/37proof.'};Path('/tmp/noname-parent-c9g-final6-validation-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
