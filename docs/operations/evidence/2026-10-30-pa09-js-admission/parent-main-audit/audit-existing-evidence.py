from pathlib import Path
import hashlib,json,re,subprocess
P=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=P/'docs/operations/evidence/2026-10-30-pa09-js-admission';R=E/'main-acceptance'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
index=json.loads((R/'artifact-sha256.json').read_bytes());verified=[]
for name,row in index['files'].items():
 p=R/name;assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'];verified.append(name)
assert len(verified)==37
s=json.loads((R/'main-summary.json').read_bytes());commands=json.loads((R/'commands.json').read_bytes());entry=json.loads((R/'entry.json').read_bytes());exit=json.loads((R/'exit.json').read_bytes());bindings=json.loads((R/'bindings-result.json').read_bytes())
assert len(commands)==11 and sum(c['exit']==0 for c in commands)==9 and sum(c['exit']==1 for c in commands)==1 and sum(c['exit'] is None for c in commands)==1
for c in commands:assert sha(R/c['stdout_stderr'])==c['log_sha256']
for key in ['head','tree','index_sha256','staged','source3','protected','modules','dist','guard','frozen_package_diff_sha256','package_git_trees']:assert entry[key]==exit[key]
assert entry['groups'].keys()==exit['groups'].keys()
for key in entry['groups']:assert sorted(entry['groups'][key])==sorted(exit['groups'][key])
for group in ['source3','protected','modules','dist']:
 for path,digest in entry[group].items():assert sha(P/path)==digest
for name,count in [('targeted',20),('frontend',175)]:
 text=(R/(name+'.log')).read_text();assert re.search(rf'^(?:#|ℹ) pass {count}$',text,re.M) and re.search(r'^(?:#|ℹ) fail 0$',text,re.M)
prod=json.loads((R/'production-products-sha256.json').read_bytes());assert len(prod['files'])==13
for path,digest in prod['files'].items():assert sha(Path(prod['root'])/path)==digest
mods=json.loads((R/'bundle-module-table.json').read_bytes());assert len(mods['sourceModules'])==35
for path,row in mods['sourceModules'].items():assert sha(P/'apps/web-client'/path)==row['sourceSha256']
for path,digest in bindings['inherited_source_bindings'].items():assert sha(P/'apps/web-client'/path)==digest
for name,digest in bindings['inherited_evidence_sha256'].items():assert sha(E/'actual-webview/evidence'/name)==digest
changed=subprocess.run(['git','--no-optional-locks','-C',str(P),'diff','--name-only','a1f7250815eee803d931301ab46207719cb9a15b',entry['head'],'--','crates','apps/desktop-client','apps/game-server','Cargo.toml','Cargo.lock'],check=True,capture_output=True).stdout;assert not changed
out={'result':'PASS','scope':'Fresh existing-byte independent parse; no source writes/tests/builds/GUI','evidence_files':len(verified),'artifact_index_sha256':sha(R/'artifact-sha256.json'),'commands':{'total':11,'exit0':9,'exit1_harness':1,'not_executed_timeout':1},'targeted':20,'frontend':175,'source3':entry['source3'],'protected':len(entry['protected']),'modules':len(entry['modules']),'dist':len(entry['dist']),'production_files':len(prod['files']),'production_source_modules':len(mods['sourceModules']),'inherited_source_seams':len(bindings['inherited_source_bindings']),'inherited_evidence_files':len(bindings['inherited_evidence_sha256']),'main_vs_a1f_native_changed_paths':changed.decode(),'new_main_GUI':False,'limitations':['First exploratory git status precedes entry index capture; no pre-entry index invariance claim.','Private actual WK qualification inherited only for exact source3/eight seams and unchanged native sources.','Historical warnings, setup timeout, hook timing and caller-order parser failures preserved.']}
D=E/'parent-main-audit';D.mkdir(exist_ok=True);(D/'independent-results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
