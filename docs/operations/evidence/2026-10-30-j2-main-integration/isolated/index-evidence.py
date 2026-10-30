import collections,hashlib,json,pathlib,re,shutil,subprocess
ROOT=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree');E=ROOT.parent/'evidence'/'theft-correction'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def stats(s):
 rows=[tuple(map(int,m)) for m in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',s)]
 return [sum(r[i] for r in rows) for i in range(3)]
full=(E/'17-final-workspace.log').read_text();counts=collections.defaultdict(lambda:[0,0,0]);current=None
for line in full.splitlines():
 m=re.search(r'Running unittests .*?/deps/(.+)-[a-f0-9]+\)',line)
 if m:current=m[1]
 if line.strip().startswith('Doc-tests'):current='doctests'
 m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',line)
 if m:
  for i,n in enumerate(map(int,m.groups())):counts[current][i]+=n
six={k:counts[k] for k in ['content_native_loader','engine_native','engine_native_content','engine_native_kernel','engine_native_wasm','game_server']}
summary={'workspace':stats(full),'native_three':stats((E/'10-final-native-three.log').read_text()),'affected_six':six,'affected_six_total':[sum(v[i] for v in six.values()) for i in range(3)],'review_regressions_final_source':[len(re.findall(r'^test adr0786_j2_review_regressions::.* \.\.\. ok$',(E/'10-final-native-three.log').read_text(),re.M)),0,0],'theft_red':stats((E/'01-theft-gift-red.log').read_text()),'actual_wasm_matrix':{k:v for k,v in json.loads((E/'wasm-matrix-result.json').read_text()).items() if k!='results'},'source_base':'2b3348eff0a6a7f07d6f2ba3520e4ca243e344a6','candidate_versions':[329,758,105],'historical_workspace54_failure':[6742,1,74]}
assert summary['workspace']==[6757,0,74] and summary['native_three']==[1670,0,73] and summary['review_regressions_final_source']==[12,0,0],summary
(E/'final-counts.json').write_text(json.dumps(summary,indent=2)+'\n');(E/'ignored-tests.txt').write_text('\n'.join(line for line in full.splitlines() if ' ... ignored' in line)+'\n')
records=[]
for p in sorted(E.glob('*.log')):
 text=p.read_text(errors='replace');last=None;after=[]
 for line in text.splitlines():
  try:d=json.loads(line)
  except ValueError:continue
  if not isinstance(d,dict):continue
  if 'actual_command'in d and 'exit'in d:last=d
  if 'free_bytes_after_command'in d:after.append(d)
 if last is not None:
  records.append({'log':p.name,'command':last['actual_command'],'exit':last['exit'],'result':'passed' if last['exit']==0 else 'failed','post_command_disk_records':after})
for entry in json.loads((E/'final-commands.json').read_text()):
 if not any(r['log']==entry['log'] for r in records):records.append({**entry,'result':'passed' if entry['exit']==0 else 'failed'})
records.extend([{'log':'02-theft-gift-green.log','result':'interrupted','exit':None,'notes':'ENOSPC interruption reported by parent; empty log, no pass inferred.'},{'log':'01-theft-gift-red.log','result':'expected-semantic-red','exit':101,'notes':'11 pass/1 fail: actual author DSL hand theft incorrectly triggers giver draw; equipment and Fenghe controls pass.'},{'log':'22-real-worker.log','result':'passed','exit':0,'notes':'Real Chrome Worker SDK store replay:3 theft/equipment/Fenghe cases, byte checkpoints, invalid/stale retry atomicity and frozen restore.'}])
(E/'commands-run.json').write_text(json.dumps(records,indent=2)+'\n')
# Preserve only the native executables needed to independently replay these semantic regressions.
out=E/'native-test-artifacts';out.mkdir(exist_ok=True);binary_records=[]
for name,log,pattern in [('native-misc-tests','10-final-native-three.log',r'Running tests/groups/misc.rs \(([^)]+)\)'),('native-matrix-tests','20-final-native-matrix.log',r'Running tests/adr0786_j2_cross_target.rs \(([^)]+)\)')]:
 match=re.search(pattern,(E/log).read_text());assert match,name;source=pathlib.Path(match[1]);dest=out/name;shutil.copy2(source,dest);binary_records.append({'name':name,'source':str(source),'sha256':sha(dest),'bytes':dest.stat().st_size})
(E/'native-test-artifacts.json').write_text(json.dumps(binary_records,indent=2)+'\n')
retirement={'CARGO_INCREMENTAL':'0','source_or_shared_cache_deleted':False,'private_targets_ready_for_parent_retirement_after_batch':[str(pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc09-pe1axnwg')/name) for name in ['target-nc09','target-gate']],'retention_until_end_reason':'deps and test executables reused by native, workspace, strict and WASM checks; completed batch can retire targets after saved native/WASM artifacts and SHA logs are verified. Parent owns whole-target deletion.','native_artifacts_saved':binary_records,'wasm_artifacts_saved':[{ 'path':str(p.relative_to(E)),'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted((E/'wasm-pkg').glob('*')) if p.is_file()],'free_bytes_at_index':shutil.disk_usage(ROOT).free}
(E/'disk-retention-and-retirement.json').write_text(json.dumps(retirement,indent=2)+'\n')
manifest=json.loads((E/'j2-theft-before-after-sha256.json').read_text());assert all(sha(ROOT/f['path'])==f['final_sha256'] for f in manifest['files']);assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=ROOT)
(E/'j2-theft-artifact-sha256.json').write_text(json.dumps({str(p.relative_to(E)):sha(p) for p in [*E.glob('j2-theft*'),E/'native-matrix.json',E/'wasm-matrix-result.json',E/'matrix-attribution.json',E/'final-counts.json',E/'commands-run.json',E/'disk-retention-and-retirement.json',*out.glob('*'),*(E/'wasm-pkg').glob('*')] if p.is_file() and p.name!='j2-theft-artifact-sha256.json'},indent=2)+'\n')
print(json.dumps(summary,indent=2));print('Native and WASM artifacts saved; all72 source fingerprints match; index empty.')
