from pathlib import Path
import json,hashlib,zlib,gzip,re,subprocess,os,datetime,tempfile
from concurrent.futures import ThreadPoolExecutor
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-d07b35f9-own-br49nqpd');A=Path(tempfile.mkdtemp(prefix='noname-parent-ci41202703042-'));M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def j(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def crc(b):return f'{zlib.crc32(b)&0xffffffff:08x}'
s=j(E/'verified-publication-ci-summary.json');seal=j(E/'artifact-index-seal.json');manifest=j(E/'copy-manifest.json');assert sha((E/'copy-manifest.json').read_bytes())==seal['manifest_sha256'];assert sha((E/'verified-publication-ci-summary.json').read_bytes())==seal['summary_sha256']
assert len(manifest['files'])==101
for r in manifest['files']:
 b=(E/r['path']).read_bytes();assert len(b)==r['bytes'] and sha(b)==r['sha256'] and crc(b)==r['crc32'],r['path']
assert s['head_sha']=='d07b35f94a42b282222dda4ab7bbc5c39ab8fddb' and s['tree']=='7cfa8e085e94e760999e14b90861857f8c9962bb' and s['parent']=='5fc0f77403d90fa4555156bf01722450cbe7bf3b'
assert s['run_id']==41202703042 and s['attempt']==1 and s['run_status']=='completed' and s['run_conclusion']=='failure'
assert len(s['jobs'])==10 and len(s['successful_jobs'])==8 and set(s['failed_jobs'])=={'build-windows-latest','desktop-windows-latest'}
steps=[step for job in s['jobs'] for step in job['steps']];assert len(steps)==126
counts={state:sum(step['conclusion']==state for step in steps) for state in ['success','failure','skipped',None]};assert counts=={'success':110,'failure':2,'skipped':14,None:0},counts
raw_bytes=0;rust_counts={};windows={}
for job in s['jobs']:
 b=gzip.decompress((E/'ci-stage'/job['raw_log']).read_bytes());assert len(b)==job['log_bytes'] and sha(b)==job['log_sha256'] and crc(b)==job['log_crc32'];assert b==(E/'private-raw-downloads'/job['raw_log'].removesuffix('.gz')).read_bytes();raw_bytes+=len(b)
 text=b.decode(errors='replace');groups=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
 if 'rust_tests' in job:
  total=[sum(int(row[i]) for row in groups) for i in range(3)];assert total==[job['rust_tests'][k] for k in ['passed','failed','ignored']];rust_counts[job['name']]=total
 if job['conclusion']=='failure':
  assert job['steps'][1]['name']=='Run actions/checkout@v4' and job['steps'][1]['conclusion']=='failure'
  assert 'Filename too long' in text and 'exit code 128' in text
  assert b.count(b'\r\n')==job['log_crlf_count'];windows[job['name']]={'CRLF':b.count(b'\r\n'),'failed_step':2,'compile_and_test_executed':False}
assert raw_bytes==4245871 and rust_counts['gates-ubuntu']==[6922,0,77] and rust_counts['native-web']==[1830,0,75]
assert len(s['watch_commands'])==1 and s['watch_commands'][0]['exit']==1 and s['transport_failures']==[]
push=j(E/'push-stage/result.json');assert push['push_exit']==0 and push['expected_old_remote']==s['parent'] and push['remote_verified'];assert push['remote_after_refs']['refs/heads/native-convergence']==s['head_sha'];assert push['remote_after_refs']['refs/heads/main']=='47347c6608d651660ae9459e59eb529b3cef277f'
assert push['push_attempts']==1 and push['increment_blob_bytes']==204625141 and len(push['committed_increment_paths'])==3618
for path,h in s['source6_sha256'].items():
 b=subprocess.check_output(['git','--no-optional-locks','show',s['head_sha']+':'+path],cwd=M,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'});assert sha(b)==h,path
assert s['no_main_writes_or_reads_after_lease'] and s['lease_release']['at_utc']=='2026-10-31T12:48:05.496863+00:00'
# Independent read-only live API observations; these do not retry or rerun the workflow.
repo='repos/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified';queries={'run':repo+'/actions/runs/41202703042','jobs':repo+'/actions/runs/41202703042/jobs?per_page=100','commit':repo+'/commits/'+s['head_sha'],'native_ref':repo+'/git/ref/heads/native-convergence','main_ref':repo+'/git/ref/heads/main'}
def read_api(item):
 name,url=item;p=subprocess.run(['gh','api',url],env={**os.environ,'GODEBUG':'http2client=0'},capture_output=True);(A/(name+'.stdout')).write_bytes(p.stdout);(A/(name+'.stderr')).write_bytes(p.stderr);return name,p.returncode
with ThreadPoolExecutor(max_workers=5) as pool:reads=list(pool.map(read_api,queries.items()))
assert all(code==0 for _,code in reads),reads
live={name:j(A/(name+'.stdout')) for name in queries};assert live['run']['head_sha']==s['head_sha'] and live['run']['run_attempt']==1 and live['run']['conclusion']=='failure';assert len(live['jobs']['jobs'])==10 and {j['id']:j['conclusion'] for j in live['jobs']['jobs']}=={j['id']:j['conclusion'] for j in s['jobs']};assert live['commit']['commit']['tree']['sha']==s['tree'];assert live['native_ref']['object']['sha']==s['head_sha'];assert live['main_ref']['object']['sha']=='47347c6608d651660ae9459e59eb529b3cef277f'
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_INDEPENDENT_REGISTRATION_OF_8_SUCCESS_2_FAILURE','root':str(A),'source_E':str(E),'manifest_entries_SHA_CRC':101,'primary_files':103,'gzip_logs':10,'decompressed_bytes':raw_bytes,'head':s['head_sha'],'run':41202703042,'attempt':1,'job_counts':{'success':8,'failure':2},'step_counts':{'success':110,'failure':2,'skipped':14,'null':0},'rust_counts':rust_counts,'Windows_checkout_failures':windows,'independent_live_API_reads':reads,'qualification':'Windows product steps were skipped after real checkout Filename too long, not test failures or passes. Raw failures retained; no rerun/dispatch/push/Main writes. Original d07 source only, later I02 Main changes not CI inputs. Source6 committed blobSHA freshly checked, not a repeated204MB body hash scan.'};(A/'summary.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');Path('/tmp/noname-parent-ci41202703042-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
