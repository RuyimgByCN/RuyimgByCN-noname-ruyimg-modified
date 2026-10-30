from pathlib import Path
import subprocess,os,json,hashlib
entry=json.loads(Path('../evidence/review-corrections/entry.json').read_text());env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
result={'files':{p:sha(p) for p in entry['files']},'patches':{p:sha('../evidence/corrections/'+p) for p in entry['patches']},'repositories':{}}
assert result['files']==entry['files'] and result['patches']==entry['patches']
roots={'candidate':Path.cwd(),'main':Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next'),'core':Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname'),'liuli':Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname-liuli')}
for name,p in roots.items():
 def git(*a):return subprocess.check_output(['git','-C',str(p),*a],env=env)
 before=entry['repositories'][name]
 now={'HEAD':git('rev-parse','HEAD').decode().strip(),'index_sha256':sha(before['index_path']),'diff_sha256':hashlib.sha256(git('diff','--binary')).hexdigest(),'staged_sha256':hashlib.sha256(git('diff','--cached','--binary')).hexdigest()}
 now['entry_matches']={key:now[key]==before[key] for key in now}
 now['logical_index_equals_HEAD']=subprocess.run(['git','-C',str(p),'diff','--cached','--quiet','HEAD'],env=env).returncode==0
 result['repositories'][name]=now
 assert all(v for k,v in now['entry_matches'].items() if k!='index_sha256'),(name,now)
liuli=roots['liuli'];result['liuli_seven']={p:sha(liuli/p) for p in entry['liuli_seven']};assert result['liuli_seven']==entry['liuli_seven']
raw=subprocess.check_output(['git','-C',str(roots['main']),'ls-tree','-r','--name-only','-z','977cdf1f35bc3f08aba137528a3672f3dad5f391'],env=env).decode().strip('\0').split('\0')
result['accepted_main_tree']={'commit':'977cdf1f35bc3f08aba137528a3672f3dad5f391','paths':len(raw),'tracked_dist':sum(p.startswith('apps/web-client/dist/') for p in raw),'tracked_evidence':sum(p.startswith('docs/operations/evidence/') for p in raw),'target_or_ignored_asset_paths':sum(p.startswith(('target/','content/original/assets/')) for p in raw),'is_final_CI_commit':False}
result['no_staged_files']=all(v['staged_sha256']==hashlib.sha256(b'').hexdigest() for v in result['repositories'].values());assert result['no_staged_files']
result['prior_exit_guard_failure']={'repository':'main','entry_index_sha256':'4b374cd54c8a1670975454d0a4ffaa516d8154265e4285cf06e39c3d34b5c25d','first_exit_observation_index_sha256':'e5f807385d249126e4b045d628dba8b09dcc0b4509229f32ce21f725c4789900','failure':'binary index equality assertion failed; HEAD, tracked diff, staged digest unchanged','attribution':'unknown concurrent metadata observation; no restoration or write performed; all reviewer Git commands used optional locks disabled'}
Path('../evidence/review-corrections/exit.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
