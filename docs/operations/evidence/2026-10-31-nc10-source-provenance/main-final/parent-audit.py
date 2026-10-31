from pathlib import Path
import hashlib,json,re,subprocess,datetime
E=Path('/tmp/nc27-main-final-v3-8e1fad00');M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path('/tmp/noname-parent-nc27-audit');O.mkdir(exist_ok=True)
def load(n):return json.loads((E/n).read_bytes())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(*a):return subprocess.check_output(['git','-C',str(M),*a],env={**__import__('os').environ,'GIT_OPTIONAL_LOCKS':'0'}).decode().strip()
idx=load('artifact-sha256.json');assert sha(E/'artifact-sha256.json')=='95b22a3aff2bfe4a7390261881e0aa75f62bb62672961b1b9b3cd9edf1447c99'
for n,r in idx.items():
 p=E/n;assert p.is_file() and p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],n
S=load('summary.json');a=load('gate-entry.json');b=load('gate-exit.json');suppA=load('gate-entry-supplement.json');suppB=load('gate-exit-supplement.json');C=load('gate-comparison.json')
assert not S['pending_commands'] and not S['unexpected_failed_commands'];assert C['all_equal'] and all(C['invariants'].values()) and all(C['docs_and_packages'].values())
for k in C['invariants']:assert a[k]==b[k],k
for k in C['docs_and_packages']:assert suppA[k]==suppB[k],k
for k in ['admission31','input_hashes','root_gates','protected','modules','dist']:
 for n,h in b[k].items():
  assert isinstance(h,str),(k,n,type(h));assert sha(M/n)==h,(k,n)
assert len(b['admission31'])==33 and len(b['protected'])==99 and len(b['modules'])==18 and len(b['dist'])==37
for n,h in suppB['docs_markdown'].items():assert sha(M/n)==h,n
assert len(suppB['docs_markdown'])==1927
assert git('rev-parse','HEAD')==b['head']=='4dd0b286b995fe414128b6c52eb83a4d0d85f6cd';assert git('rev-parse','HEAD^{tree}')==b['tree'];assert not git('diff','--cached','--name-only')
post_index_sha=sha(M/'.git/index')
# The final WINDOW was byte-stable; later Git stat/cache refreshes are not staging changes.
index_rows={}
for row in git('ls-files','--stage','-z').split('\0'):
 if not row:continue
 head,path=row.split('\t',1);mode,oid,stage=head.split();assert stage=='0';index_rows[path]=(mode,oid)
head_rows={}
for row in git('ls-tree','-r','--full-tree','-z','HEAD').split('\0'):
 if not row:continue
 head,path=row.split('\t',1);mode,typ,oid=head.split();head_rows[path]=(mode,oid)
assert index_rows==head_rows

commands={x['name']:x for x in S['commands']};assert len(commands)==16
for n,r in commands.items():
 assert sha(E/(n+'.stdout.log'))==r['stdout_sha256'];assert sha(E/(n+'.stderr.log'))==r['stderr_sha256'];assert r['wrapper_record']['exit']==r['exit'];assert r['wrapper_record']['command']==r['argv'];assert not r['disk_stop'] and r['minimum_free_bytes']>3*1024**3
 assert r['exit']==(1 if n=='actual-native-source' else 0),n
rows=re.findall(r'^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', (E/'workspace.stdout.log').read_text(),re.M);tot=[sum(int(x[i]) for x in rows) for i in range(3)];assert len(rows)==164 and tot==[6917,0,77]
assert load('workspace-parsed.json')['totals']==dict(zip(['passed','failed','ignored'],tot))
pf=(E/'preflight.stdout.log').read_text();assert '--quick' not in commands['preflight']['argv'];assert sum(line.startswith('✅ ') for line in pf.splitlines())==21 and 'preflight 全部通过 ✅' in pf
gen=load('generated18-products.json');assert len(gen)==18;changed=[]
for n,r in gen.items():
 assert sha(M/n)==r['after_sha256'] and (M/n).stat().st_size==r['bytes'];assert r['changed']==(r['before_sha256']!=r['after_sha256']);
 if r['changed']:changed.append(n)
assert len(changed)==3 and all(n.endswith('.wasm') for n in changed)
# Products may include symlinks; read and hash actual target bytes, retaining recorded paths.
products=load('frontend-products.json');prodRoot=E/'frontend-dist';
if not prodRoot.exists():
 candidates=[p for p in E.iterdir() if p.is_dir() and (p/'index.html').is_file()];assert len(candidates)==1,candidates;prodRoot=candidates[0]
for n,r in products.items():assert sha(prodRoot/n)==r['sha256'] and (prodRoot/n).stat().st_size==r['bytes'],n
actual=load('actual-native-source.json');legacy=load('legacy-wasm-sink-smoke.json')
warning_headers=[x for x in (E/'workspace.stderr.log').read_text().splitlines() if re.match(r'^warning(?:\[[^\]]+\])?:',x)];warning_summaries=[x for x in warning_headers if re.match(r'^warning: .* generated \d+ warning',x)];warn=len(warning_headers)-len(warning_summaries);assert warn==727 and len(warning_summaries)==55,(warn,len(warning_summaries))
# Negative self-check: a one-byte mutation cannot pass the metadata digest.
x=(E/'actual-native-source.json').read_bytes();assert hashlib.sha256(x+b'!').hexdigest()!=idx['actual-native-source.json']['sha256']
result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','evidence_index_files':len(idx),'source33':33,'protected99':99,'generated18':18,'defaultdist37':37,'frozen_markdown':1927,'workspace':tot,'workspace_summaries':len(rows),'commands':len(commands),'retained_harness_failure':commands['actual-native-source']['exit'],'compiler_diagnostic_headers':warn,'changed_generated':changed,'frontend_products':len(products),'actual_native':actual,'actual_legacy':legacy,'head':b['head'],'index_empty':True,'post_window_index_sha256':post_index_sha,'window_index_sha256':b['index_sha256'],'logical_index_all_entries_equal_HEAD':True,'post_window_raw_index_changed':post_index_sha!=b['index_sha256'],'raw_index_change_cause':'not established; no staged-content change; final-window byte equality verified independently above','negative_sha_selfcheck':True,'qualification':'Fresh bounded SHA/Git/JSON/log parser only; no tests/build/product/network execution, no full43869/3GB rescan. Final snapshot docs exact at audit time; later evidence additions are separately checked.'}
(O/'main-final-v3-results.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['actual_native','actual_legacy']},indent=2))
