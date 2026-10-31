import os,sys,json,stat,hashlib,subprocess,datetime,shutil
from pathlib import Path
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/nc10-main-own-sfobj6i8');O=Path('/tmp/nc10-main-owned-target-retirement-f463dd87');R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');W=Path('/tmp/nc27-main-functional-458ed333/cargo-target');TARGETS=[E/'probe/target',E/'strict-target']
def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
 return h.hexdigest()
def save(n,v):(O/n).write_text(json.dumps(v,indent=2)+'\n')
def filerec(p):
 s=p.lstat();return {'path':str(p),'bytes':s.st_size,'sha256':sha(p),'mode':stat.S_IMODE(s.st_mode),'mtimeNs':s.st_mtime_ns,'birthUtc':datetime.datetime.fromtimestamp(s.st_birthtime,datetime.timezone.utc).isoformat(),'device':s.st_dev,'inode':s.st_ino,'links':s.st_nlink,'uid':s.st_uid,'allocatedBytes':s.st_blocks*512}
def targetfiles(p):return sorted(f for f in p.rglob('*') if f.is_file())
def old_non_target():
 result={}
 for root,ds,fs in os.walk(E,followlinks=False):
  ds[:]=[n for n in ds if Path(root)/n not in TARGETS]
  for name in fs:
   p=Path(root)/name;assert not p.is_symlink();result[str(p.relative_to(E))]={'sha256':sha(p),'bytes':p.stat().st_size,'mtimeNs':p.stat().st_mtime_ns,'mode':stat.S_IMODE(p.stat().st_mode)}
 return result
def protected():
 prior=json.load(open(E/'freeze-exit.json'));paths=set(prior['source6'])|{'Cargo.lock'}|set(prior['protected']['modules'])|set(prior['protected']['dist']);return {p:sha(R/p) for p in sorted(paths)}
def guard(label):
 result={'utc':utc(),'label':label,'targets':[]};ps=subprocess.run(['ps','-axo','pid,ppid,uid,command'],capture_output=True,text=True);(O/(label+'-ps.stdout.txt')).write_text(ps.stdout);(O/(label+'-ps.stderr.txt')).write_text(ps.stderr);assert ps.returncode==0
 rows={}
 for line in ps.stdout.splitlines()[1:]:
  v=line.strip().split(None,3)
  if len(v)==4:rows[int(v[0])]={'ppid':int(v[1]),'command':v[3]}
 ancestors=[];pid=os.getpid()
 while pid in rows and pid not in ancestors:ancestors.append(pid);pid=rows[pid]['ppid']
 refs=[{'pid':pid,**v} for pid,v in rows.items() if pid not in ancestors and any(s in v['command'] for s in [str(E),str(E.resolve())])];result['selfAncestorsExcluded']=ancestors;result['processReferences']=refs
 for i,p in enumerate(TARGETS):
  assert p.is_dir() and not p.is_symlink() and p.stat().st_uid==os.getuid();assert p.resolve().is_relative_to(E.resolve());assert p.resolve()!=W.resolve() and not W.resolve().is_relative_to(p.resolve());s=p.stat();x=subprocess.run(['/usr/sbin/lsof','-nP','+D',str(p)],capture_output=True,text=True);(O/f'{label}-lsof-{i}.stdout.txt').write_text(x.stdout);(O/f'{label}-lsof-{i}.stderr.txt').write_text(x.stderr)
  result['targets'].append({'path':str(p),'realpath':str(p.resolve()),'device':s.st_dev,'inode':s.st_ino,'uid':s.st_uid,'lsofExit':x.returncode,'lsofStdoutEmpty':not x.stdout.strip(),'lsofStderrEmpty':not x.stderr.strip()})
 result['pass']=not refs and all(x['lsofExit']==1 and x['lsofStdoutEmpty'] and x['lsofStderrEmpty'] for x in result['targets']);save(label+'-guard.json',result);assert result['pass'],result;return result

def prepare():
 g=guard('pre-preservation');free=shutil.disk_usage(E).free;save('disk-before.json',{'utc':utc(),'freeBytes':free,'qualification':'Global observation only; concurrent writers prohibit causal savings inference.'});assert free>=3*1024**3
 before=old_non_target();save('original-E-nontarget-before.json',before);save('Main-protected-before.json',protected());allinventories=[];preserved=[]
 for i,p in enumerate(TARGETS):
  tag='probe-target' if i==0 else 'strict-target';files=targetfiles(p);assert not any(f.is_symlink() for f in p.rglob('*'));records=[filerec(f) for f in files];groups={}
  for x in records:groups.setdefault((x['device'],x['inode']),[]).append(x)
  sharing=[]
  for key,rows in groups.items():
   assert all(x['links']==len(rows) for x in rows),(key,rows);sharing.append({'device':key[0],'inode':key[1],'links':rows[0]['links'],'allLinksWithinThisTarget':True,'paths':[x['path'] for x in rows]})
  du=subprocess.run(['du','-sk',str(p)],capture_output=True,text=True);(O/f'{tag}-du-before.stdout.txt').write_text(du.stdout);(O/f'{tag}-du-before.stderr.txt').write_text(du.stderr);assert du.returncode==0
  for f,x in zip(files,records):
   rel=f.relative_to(p);keep=('.fingerprint' in rel.parts or 'flycheck0' in rel.parts or f.name in ['.rustc_info.json','CACHEDIR.TAG','output','root-output','stderr','invoked.timestamp'] or bool(x['mode']&0o111))
   if keep:
    q=O/'preserved'/tag/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,q);assert sha(q)==x['sha256'] and stat.S_IMODE(q.stat().st_mode)==x['mode'] and q.stat().st_mtime_ns==x['mtimeNs'];preserved.append({'originalPath':str(f),'preservedPath':str(q),'sha256':x['sha256'],'bytes':x['bytes'],'mode':x['mode'],'mtimeNs':x['mtimeNs'],'classification':'compiler/build-helper or metadata; not accepted probe runtime binary'})
  assert not any(Path(x['path']).name in ['nc10-main-acceptance-probe','nc10-main-lock-acceptance-probe'] for x in records)
  inv={'target':str(p),'identity':g['targets'][i],'files':records,'fileCount':len(records),'logicalBytes':sum(x['bytes'] for x in records),'uniqueAllocatedBytes':sum(rows[0]['allocatedBytes'] for rows in groups.values()),'scopedDuKilobytes':int(du.stdout.split()[0]),'hardlinkOwnership':sharing,'finalProbeExecutablePresent':False};save(tag+'-inventory.json',inv);allinventories.append(inv)
 # Preserve every original stdout/stderr, source/lock/driver/seal identity needed to interpret retired caches.
 keep=[]
 for rel in before:
  f=E/rel
  if f.suffix in ['.rs','.toml','.lock','.py','.mjs','.cjs'] or f.name.endswith(('.log','.txt','.json')):keep.append(f)
 for f in keep:
  q=O/'preserved/original-E'/f.relative_to(E);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,q);assert sha(q)==sha(f)
 for f in [R/'Cargo.lock',Path('/tmp/noname-parent-nc10-main-final-audit.json'),Path('/tmp/noname-parent-nc10-main-lock-audit.json'),Path('/tmp/noname-own-accepted-commit.json'),Path('/tmp/nc10-main-own-provenance-addendum/record.json'),Path('/tmp/nc10-main-lock-repair-0425dc78/manifest.json'),Path('/tmp/nc10-main-lock-repair-0425dc78/comparison.json'),Path('/tmp/nc10-main-lock-repair-0425dc78/probe/Cargo.lock')]:
  q=O/'preserved/external-metadata'/f.name
  if q.exists():q=O/'preserved/external-metadata'/(f.parent.name+'-'+f.name)
  q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,q);assert sha(q)==sha(f)
 binaries=[W/'debug/nc10-main-acceptance-probe',W/'debug/nc10-main-lock-acceptance-probe',Path('/tmp/nc10-main-lock-repair-0425dc78/binary/nc10-main-lock-acceptance-probe')];brecs=[filerec(f) for f in binaries];assert brecs[1]['sha256']==brecs[2]['sha256']=='f9c404a462bd5dfcae7f687f053ed62e9fb2fff55f0470acd334a2a059174144'
 for rec in brecs:
  f=Path(rec['path']);q=O/'preserved/runtime-binaries'/('Main-warm' if f.is_relative_to(W) else 'lock-repair')/f.name;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,q);assert sha(q)==rec['sha256'] and q.stat().st_mtime_ns==rec['mtimeNs'];rec['preservedCopy']=str(q)
 save('runtime-binaries-before.json',brecs);save('preserved-target-files.json',preserved);save('prepare-result.json',{'utc':utc(),'status':'PRESERVED; guarded retirement pending','targetInventories':[{'target':x['target'],'fileCount':x['fileCount'],'logicalBytes':x['logicalBytes']} for x in allinventories],'preservedTargetFiles':len(preserved),'oldEFilesProtected':len(before),'warmTargetIdentity':{'path':str(W),'realpath':str(W.resolve()),'device':W.stat().st_dev,'inode':W.stat().st_ino},'old714HistoricalVerification':json.load(open('/tmp/noname-parent-nc10-main-final-audit.json')),'actualProbeBinariesPreserved':True});print(json.dumps({'prepared':True,'fileCounts':[x['fileCount'] for x in allinventories],'logicalBytes':[x['logicalBytes'] for x in allinventories],'preservedTargetFiles':len(preserved),'originalNonTargetFiles':len(before)}),flush=True)

def retire():
 guard('immediate-pre-delete');assert protected()==json.load(open(O/'Main-protected-before.json'));assert old_non_target()==json.load(open(O/'original-E-nontarget-before.json'));brecs=json.load(open(O/'runtime-binaries-before.json'))
 for rec in brecs:assert sha(Path(rec['path']))==rec['sha256'] and sha(Path(rec['preservedCopy']))==rec['sha256']
 receipts=[]
 for i,p in enumerate(TARGETS):
  tag='probe-target' if i==0 else 'strict-target';inv=json.load(open(O/(tag+'-inventory.json')));st=p.stat();assert (st.st_dev,st.st_ino)==(inv['identity']['device'],inv['identity']['inode']);current=[filerec(f) for f in targetfiles(p)];assert current==inv['files'];receipts.extend({'oldPath':x['path'],'bytes':x['bytes'],'sha256':x['sha256'],'authorizedDisposition':'retired owned compiled cache','inventory':tag+'-inventory.json'} for x in current)
 save('per-file-retirement-receipts.json',{'authorizedUtc':utc(),'files':receipts,'qualification':'Intent inventory sealed immediately before deletion; completed disposition is in retirement-manifest.json.'})
 # No writable product path is accepted here: exact two known owned roots only.
 for p in TARGETS:shutil.rmtree(p)
 after=old_non_target();assert after==json.load(open(O/'original-E-nontarget-before.json'));save('original-E-nontarget-after.json',after);post=protected();assert post==json.load(open(O/'Main-protected-before.json'));save('Main-protected-after.json',post)
 for rec in brecs:assert sha(Path(rec['path']))==rec['sha256']
 assert W.is_dir() and not W.is_symlink();assert all(not p.exists() for p in TARGETS);free=shutil.disk_usage(E).free;save('disk-after.json',{'utc':utc(),'freeBytes':free,'qualification':'Global observation only; difference is not attributed to retirement and is not a performance result.'})
 completed=utc()
 for item in receipts:item.update({'removed':not Path(item['oldPath']).exists(),'retiredUtc':completed})
 assert all(item['removed'] for item in receipts)
 save('per-file-retirement-receipts.json',{'completedUtc':completed,'files':receipts,'qualification':'Every listed old file path is absent after exact authorized target retirement; original hashes and inventory identities are preserved.'})
 original=json.load(open(E/'artifact-sha256.json'));retiredIndexed=[f for f in original if any((E/f).is_relative_to(t) for t in TARGETS)];remaining=[f for f in original if f not in retiredIndexed];assert all((E/f).is_file() and sha(E/f)==original[f]['sha256'] for f in remaining)
 x={'utc':utc(),'status':'RETIRED exact two authorized idle owned cache targets','targets':[{'oldPath':str(p),'absent':not p.exists(),'inventory':('probe-target' if i==0 else 'strict-target')+'-inventory.json'} for i,p in enumerate(TARGETS)],'retiredFileCount':len(receipts),'logicalBytesRetired':sum(x['bytes'] for x in receipts),'perFileReceiptsSha256':sha(O/'per-file-retirement-receipts.json'),'allGuardChecksPassed':True,'noPidsKilled':True,'noTestsBuildNetworkOrMainWrites':True,'source6MainLockModules18Dist37Exact':True,'oldENonTargetFileCount':len(after),'oldENonTargetFilesUnchanged':True,'old714IndexBytesUnchanged':sha(E/'artifact-sha256.json'),'historical714Audit':{'utc':'2026-10-31T12:09:40.691341+00:00','status':'PASS','artifactCount':714,'qualification':'Historical pre-retirement verification; not a claim all old cache paths exist after authorized retirement.'},'originalIndexPathsRetired':retiredIndexed,'originalIndexedRemainingPresentAndHashExact':len(remaining),'warmTargetRetained':str(W),'warmProbeBinariesUnchanged':True,'preservationManifest':'preserved-target-files.json','environmentOrGuardFailures':[],'diskDeltaCausalClaim':False,'originalDependencyDriftAndRawIndexHistory':'Unchanged; not reinterpreted or attributed by retirement.'};save('retirement-manifest.json',x);print(json.dumps({'status':x['status'],'removedFiles':len(receipts),'oldIndexRetiredPaths':len(retiredIndexed),'remainingOldIndexedVerified':len(remaining),'oldNonTargetFilesUnchanged':True,'warmTargetRetained':True}),flush=True)
if __name__=='__main__':
 try:
  if sys.argv[1]=='prepare':prepare()
  elif sys.argv[1]=='retire':retire()
 except Exception as exc:
  save('pending-or-failure.json',{'utc':utc(),'status':'STOP; preserve surviving targets; no PID kill','error':repr(exc)});raise
