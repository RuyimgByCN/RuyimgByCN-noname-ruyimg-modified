from pathlib import Path
import json,hashlib,subprocess,os,datetime
D=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/c29-wire-source-lifecycle-d6bev5pm');v=json.loads((D/'source-inventory.json').read_bytes());M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');R=M.parent;B=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-parent-c9e-final-8s2rgovs/snapshot');F=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9e/full-source/source-manifest.json');sha=lambda b:hashlib.sha256(b).hexdigest();env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'};assert sha(F.read_bytes())==v['overlay_manifest_sha256'];rows=json.loads(F.read_bytes())['files'];assert len(rows)==v['snapshot_files_verified']==219
for row in rows:assert sha((B/row['path']).read_bytes())==row['sha256']
fallback=0
for row in v['source_reads']:
 origin=row.get('origin',row['path'])
 if origin.startswith('git:'):
  _,commit,name=origin.split(':',2);assert commit==v['base'];body=subprocess.check_output(['git','--no-optional-locks','-C',str(M),'show',commit+':'+name],env=env);fallback+=1
 else:
  p=Path(origin)
  if not p.is_file():
   assert origin=='explicitly supplied immutable approval artifact';p=Path(row['path'])
  assert p.is_absolute() and p.is_file();body=p.read_bytes()
  if 'pin' in row:
   repo=R/'archive/upstream/noname-liuli';canonical=subprocess.check_output(['git','--no-optional-locks','-C',str(repo),'show',row['pin']+':'+str(p.relative_to(repo))],env=env);assert body==canonical
 assert sha(body)==row['sha256'],origin
for row in v['original_artifacts_unchanged']:assert sha(Path(row['path']).read_bytes())==row['sha256']
O=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/c35bcf27-e9c6-4766-badf-c63dfff448a3');p=O/'proposals/c29-five-native-invocation-wire-source-lifecycle-supplement.md';r=O/'reviews/c29-five-native-invocation-wire-source-lifecycle-supplement.md';assert sha(p.read_bytes())==v['validation']['supplement_sha256']
contracts={'docs/operations/evidence/2026-10-31-c331-runtime-contract/approval-request.md':'2859f6f07b188b2353a40d1138aa5b69e6c92ca67723e66016a189a8c78c5a0d','docs/operations/evidence/2026-10-31-nc10-source-provenance/user-decision27.json':'2b9eed59670bf3c5b3d92244934b692ab8c2be1d609b380f8f4810e5f8ddf0a3'}
for name,wanted in contracts.items():assert sha((M/name).read_bytes())==wanted
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_SOURCE_BOUND_COMPAT_SOURCE27_LIFECYCLE_SUPPLEMENT_PLAN_ONLY','overlayFiles':219,'readBindings':len(v['source_reads']),'baseFallbackReads':fallback,'originalGuideInventoryUnchanged':True,'contractAndDecision27Exact':True,'proposalSHA256':sha(p.read_bytes()),'reviewFinalSHA256':sha(r.read_bytes()),'inventorySHA256':sha((D/'source-inventory.json').read_bytes()),'excerptsSHA256':sha((D/'source-excerpts.txt').read_bytes()),'qualification':'e67OKnotesplanonly, no runtime/frozenreader/IPCWSWASMUI/privacy/restoregreen. InternalHiddenCardexistingwirestring/indexlabelonly;27ordinaryOptionalGateOwnsourceonly;genuinelifecycle/source-specificlogchildren/cancelstillrequired. Parenttwoorigin-key/pathassumptionfailurespreservedthenfixed; actualall20sourcebytes+pins/contractsverified, originalsource/artifactsunchanged. No Main/source/index/HEAD/stage/device/networkwrites.'};Path('/tmp/noname-parent-five-fsm-supplement-source-audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(out,ensure_ascii=False,indent=2))
