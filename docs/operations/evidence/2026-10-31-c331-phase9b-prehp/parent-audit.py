from pathlib import Path,PurePosixPath
import json,hashlib,tarfile,tempfile,datetime,re,subprocess
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9b');W=E.parent.parent/'worktree';A=Path(tempfile.mkdtemp(prefix='noname-parent-c9b-final-'))
def j(p):return json.loads(p.read_bytes())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
index=j(E/'artifact-index.json');assert index['parent_owned_validation_exclusion']
for r in index['files']:assert (E/r['path']).stat().st_size==r['bytes'] and h(E/r['path'])==r['sha256'],r['path']
binding=j(E/'scope-fix-source-binding.json');full=E/'full-source-scope-fix';inc=E/'final-scope-fix'
assert h(full/'source-manifest.json')==binding['full_manifest_sha256']=='d1c89c4cd195e0dc6847bcffbfb3d01f52f6aa6e086ad7649451f14329ded049'
assert h(full/'composed-source.tar.gz')==binding['full_tar_sha256'];assert h(full/'vs-main5fc.patch')==binding['full_patch_sha256'];assert h(inc/'source-manifest.json')==binding['increment_manifest_sha256']=='e0ef75ea3d3c0a7a77bb2103db36e5b9e23e1f2d146286716d4c971a62c3a174'
for root,arc,expected_count,dest in [(full,'composed-source.tar.gz',212,'snapshot'),(inc,'changed-source.tar.gz',46,'increment')]:
 source=j(root/'source-manifest.json');expected={r['path']:r['sha256'] for r in source['files']};assert len(expected)==expected_count
 with tarfile.open(root/arc) as t:
  assert len(t.getmembers())==expected_count
  for r in t.getmembers():
   p=PurePosixPath(r.name);assert r.isfile() and not p.is_absolute() and '..' not in p.parts
   b=t.extractfile(r).read();assert hashlib.sha256(b).hexdigest()==expected[r.name];assert h(W/r.name)==expected[r.name]
   out=A/dest/r.name;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(b)
observed=j(E/'scope-fix-results-observed.json');checks=j(E/'scope-fix-checks/check-results.json');assert observed['source_manifest_sha256']==binding['increment_manifest_sha256'];assert h(E/'scope-fix-checks/full-native.log')==observed['full_log_sha256']
text=(E/'scope-fix-checks/full-native.log').read_text();rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);counts=[sum(int(r[i]) for r in rows) for i in range(3)];assert counts==[1791,1,75];assert 'test c331_prehp_damage::c9b_damage_constructor_sharing_preserves_existing_equipped_emit_entry ... ok' in text
assert len(checks)==5 and checks[0]['exit']==101 and all(c['exit']==0 for c in checks[1:]);assert checks[1]['RUSTFLAGS']=='-Dwarnings' and checks[0]['target']!=checks[1]['target'];assert checks==observed['observed_results']
# The final scope-fix runner checks the 46 changed hashes before each command;
# the next-command assertion and this fresh complete212 check bind post-test source.
runner=(E/'scope-fix-checks/run.py').read_text();assert "assert all(hashlib.sha256((w/x['path']).read_bytes()).hexdigest()==x['sha256']for x in source['files'])" in runner and "'CARGO_BUILD_JOBS='" not in runner
supplement={str(p.relative_to(E/'scope-fix-checks')):{'sha256':h(p),'bytes':p.stat().st_size} for p in sorted((E/'scope-fix-checks').rglob('*')) if p.is_file()};(A/'parent-owned-validation-seal.json').write_text(json.dumps(supplement,indent=2)+'\n')
processes=[]
for pid in ['82177','82179']:
 p=subprocess.run(['ps','-p',pid,'-o','pid=,ppid=,stat=,command='],capture_output=True,text=True);processes.append({'pid':pid,'exit':p.returncode,'ps':p.stdout.strip()});assert 'scope-fix-checks/run.py' not in p.stdout and '/tmp/noname-run-serialized.py cargo test' not in p.stdout
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'INTEGRITY_PASS_FINAL_GOLDEN_STILL_RED_NOT_RUNTIME_ACCEPTANCE','root':str(A),'snapshot':str(A/'snapshot'),'increment':str(A/'increment'),'artifact_hashes':len(index['files']),'full_sources':212,'increment_sources':46,'manifest':binding['full_manifest_sha256'],'increment_manifest':binding['increment_manifest_sha256'],'final_native_counts':counts,'strict_exit':0,'parent_owned_validation_files_sealed':len(supplement),'parent_owned_runner_complete':processes,'source_live_all212_equal_tar':True,'qualification':'Finale0ef scope-fixed source has1791/1/75, distinct frompreFix751427 source1790/1. Onegolden group/12scenario mismatches retained, no goldenrerecord or Main import. 46hashes assertedbeforeeachqueuedcommand andfull212freshafter; global lock queuewall notperformancemeasure.'};(A/'summary.json').write_text(json.dumps(v,indent=2)+'\n');Path('/tmp/noname-parent-c9b-final-audit.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v,indent=2))
