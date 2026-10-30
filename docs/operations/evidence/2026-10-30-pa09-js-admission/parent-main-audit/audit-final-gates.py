from pathlib import Path
import hashlib,json,re
P=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=P/'docs/operations/evidence/2026-10-30-pa09-js-admission';R=E/'final-gates';O=Path('/tmp/noname-parent-pa09-final-gates-audit');O.mkdir(exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_bytes())
index=load(R/'artifact-sha256.json');assert len(index)==18
for name,row in index.items():assert (R/name).stat().st_size==row['bytes'] and sha(R/name)==row['sha256'],name
assert sha(R/'artifact-sha256.json')=='421375373673955d7959aaf1bd54e285ff70ef9887185b4464dd1ae5952310e2'
expected=[['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],['python3','-B','scripts/check-source-lines.py'],['python3','-B','scripts/check_doc_line_refs.py','--strict'],['python3','-B','scripts/preflight.py']]
commands=load(R/'commands.json');assert len(commands)==4
for c,argv in zip(commands,expected):
 assert c['argv']==argv and c['serialized_argv']==['python3','-B','/tmp/noname-run-serialized.py',*argv] and c['cwd']==str(P)
 assert c['exit']==0 and c['wrapper_record']['exit']==0 and c['wrapper_record']['command']==argv
 assert sha(R/c['stdout'])==c['stdout_sha256'] and sha(R/c['stderr'])==c['stderr_sha256']
 raw=(R/c['stdout']).read_text();observed=json.loads(next(line for line in reversed(raw.splitlines()) if line.startswith('{')));assert observed==c['wrapper_record']
 assert c['disk_free_minimum_sampled_bytes']>3*1024**3 and not c['disk_stop']
a=load(R/'entry.json');b=load(R/'exit.json');inv={}
keys=['head','tree','index_sha256','staged','tracked_status','attribute_sha256','guard','source3','protected','modules','dist','group_hashes','frozen_package_diff_sha256','package_git_trees','maintained_source_docs','binary_exclusions_by_suffix','scope_note']
for key in keys:assert a[key]==b[key],key;inv[key]=True
assert a['groups'].keys()==b['groups'].keys()
for key in a['groups']:assert sorted(a['groups'][key])==sorted(b['groups'][key]),key
inv['groups']=True;assert len(inv)==18 and len(a['maintained_source_docs'])==21726
s=load(R/'summary.json');comparison=load(R/'source-comparison.json');assert all(comparison['invariants'].values()) and s['commands_exit0']==4 and s['result']=='PASS' and s['source_invariance'] is True
for path,digest in a['source3'].items():assert sha(P/path)==digest
admission=load(E/'main-acceptance/main-summary.json');assert a['source3']==admission['source3_sha256'];assert sha(P/'.gitattributes')==a['attribute_sha256']=='9dbbbde6fb0c137b5a47b3d3738b3ca9399dec80a03f9acc1399ffc1d3619388'
lines=(R/'gate-4.stdout.log').read_text().splitlines();success=[x for x in lines if x.startswith('✅ ')];assert len(success)==21 and not any(x.startswith('❌') for x in lines);assert 'preflight 全部通过 ✅' in lines
assert 'Ran 2 tests' in (R/'gate-1.stderr.log').read_text() and 'OK' in (R/'gate-1.stderr.log').read_text()
out={'result':'PASS','scope':'Fresh read-only retained artifacts/JSON and current source4 hash; no producer/tests/builds/network','indexed_files':18,'index_sha256':sha(R/'artifact-sha256.json'),'exact_commands_and_wrappers':commands,'independent_entry_exit_invariants':inv,'maintained_source_docs_captured_and_exactly_compared':21726,'maintained_current_tree_rehashed':False,'current_source3':a['source3'],'current_attribute_sha256':sha(P/'.gitattributes'),'preflight_success_lines':success,'preflight_failure_lines':[],'raw_child_logs':'Suppressed by originalpreflight; not claimed','limitations':['Native/UI/CI/performance/platform scopes not expanded.','Current source4 rehashed; broad21726 capture equality compares bound entry/exit manifests, no redundant fresh fulltree scan.']}
(O/'independent-results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'result':'PASS','output':str(O/'independent-results.json'),'index_files':18,'commands':4,'invariants':len(inv),'maintained':21726,'preflight_passes':len(success)},indent=2))
