import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = Path(__file__).resolve().parent
SHA = '9f4b74236f97b0b9dc72388b6e08d310d2f26f64'
TREE = '88dfb549ab937d23b3632371eb77f387a566cb7c'
OLD = '34e83f0550bb70ef4af6083697428de7be2f02f8'
BAD = 'bdaf9868ef49de08dc66127eb82b05d922cc9eba'
MAIN = '47347c6608d651660ae9459e59eb529b3cef277f'
PROTECT = 'refs/heads/local-evidence/i02-bdaf-unpublished-profile'
URL = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
REPO = 'RuyimgByCN/RuyimgByCN-noname-ruyimg-modified'
HOOK_SHA = 'e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
ENV = {k:v for k,v in os.environ.items() if not k.startswith('GIT_TRACE') and k not in ('GH_DEBUG','GIT_CURL_VERBOSE')}
ENV.update(GIT_OPTIONAL_LOCKS='0',GIT_NO_LAZY_FETCH='1',GIT_TERMINAL_PROMPT='0',CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',GODEBUG='http2client=0')
PLAN = json.loads((OUT/'stage-plan.raw.json').read_bytes())
RECEIPT = json.loads((OUT/'accepted-commit.raw.json').read_bytes())
AUDIT = json.loads((OUT/'stage-audit.raw.json').read_bytes())
BASE_ADMISSION = json.loads(Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-34e83f05-i02-xs39v_py/push-stage/protected-admission.json').read_bytes())
BASE_PLAN = json.loads(Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-34e83f05-i02-xs39v_py/push-stage/stage-plan.raw.json').read_bytes())
EXPECTED = BASE_ADMISSION['expected']
SOURCE37 = {**BASE_ADMISSION['production31'],**BASE_ADMISSION['own6']}
START = time.monotonic()
result = {'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'accepted_commit':SHA,'accepted_tree':TREE,'expected_old_remote':OLD,'push_attempts':0,'commands':[]}

def save():
    result['elapsed_seconds']=time.monotonic()-START
    (OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n')

def git(*args):
    return subprocess.check_output(['git','--no-optional-locks',*args],cwd=ROOT,env=ENV)

def run(argv,name):
    p=subprocess.run(argv,cwd=ROOT,env=ENV,capture_output=True)
    (OUT/(name+'.stdout')).write_bytes(p.stdout);(OUT/(name+'.stderr')).write_bytes(p.stderr)
    result['commands'].append({'name':name,'command':argv,'exit':p.returncode});save()
    return p

def snapshot(name):
    index=(ROOT/'.git/index').read_bytes(); logical=git('ls-files','--stage','-z');flags=git('ls-files','-v','-z');untracked=git('ls-files','--others','--exclude-standard','-z')
    for suffix,data in (('raw-index',index),('logical-index',logical),('index-flags',flags),('untracked-paths',untracked)):
        (OUT/(name+'-'+suffix+'.gz')).write_bytes(gzip.compress(data,mtime=0))
    config=subprocess.run(['git','--no-optional-locks','config','--get-all','core.hooksPath'],cwd=ROOT,env=ENV,capture_output=True)
    state={'head':git('rev-parse','HEAD').decode().strip(),'tree':git('rev-parse','HEAD^{tree}').decode().strip(),'parent':git('rev-parse','HEAD^').decode().strip(),'local_branch':git('symbolic-ref','--short','HEAD').decode().strip(),'index_sha256':hashlib.sha256(index).hexdigest(),'logical_index_sha256':hashlib.sha256(logical).hexdigest(),'index_flags_sha256':hashlib.sha256(flags).hexdigest(),'untracked_paths_sha256':hashlib.sha256(untracked).hexdigest(),'tracked_diff':git('diff','--name-only').decode(),'staged_diff':git('diff','--cached','--name-only').decode(),'original_hook_sha256':hashlib.sha256((ROOT/'.git/hooks/pre-push').read_bytes()).hexdigest(),'protected_sha256':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in EXPECTED},'core_hooksPath_exit':config.returncode,'core_hooksPath_stdout':config.stdout.decode()}
    result[name]=state;save()
    assert (state['head'],state['tree'],state['parent'],state['local_branch'])==(SHA,TREE,OLD,'main')
    assert not state['tracked_diff'] and not state['staged_diff']
    assert state['original_hook_sha256']==HOOK_SHA and state['protected_sha256']==EXPECTED
    assert config.returncode==1 and not config.stdout and not config.stderr
    return state

try:
    assert (RECEIPT['head'],RECEIPT['tree'],RECEIPT['parent'])==(SHA,TREE,OLD)
    assert AUDIT['allBlobsModesExact'] and AUDIT['candidateTree']==TREE and AUDIT['files']==9905 and AUDIT['bytes']==257498242
    assert len(PLAN['files'])==len(PLAN['files'])==9905 and sum(i['bytes'] for i in PLAN['files'])==PLAN['bytes']==257498242
    assert AUDIT['nonDocPaths']==[] and AUDIT['source37Unchanged']
    assert len(SOURCE37)==37 and len(EXPECTED)==46
    snapshot('before')
    assert git('rev-parse',PROTECT).decode().strip()==BAD
    d07=subprocess.run(['git','--no-optional-locks','merge-base','--is-ancestor',OLD,SHA],cwd=ROOT,env=ENV)
    bad=subprocess.run(['git','--no-optional-locks','merge-base','--is-ancestor',BAD,SHA],cwd=ROOT,env=ENV)
    assert d07.returncode==0 and bad.returncode==1
    result['ancestry']={'parent_base_ancestor_exit':d07.returncode,'withdrawn_bdaf_ancestor_exit':bad.returncode,'local_protect_ref':PROTECT,'local_protect_sha':BAD}
    changed=git('diff','--name-only','-z',OLD,SHA).decode().split('\0')[:-1]
    admitted={i['path']:i for i in PLAN['files']}
    assert len(changed)==9905 and set(changed)==set(admitted)
    assert all(p.startswith('docs/') for p in changed), 'Non-doc path is outside explicit publication scope'
    entries={}
    for row in git('ls-tree','-rz',SHA).decode().split('\0')[:-1]:
        meta,path=row.split('\t',1);mode,kind,oid=meta.split();entries[path]={'path':path,'mode':mode,'type':kind,'oid':oid}
    exclusions=BASE_PLAN['excludedLocalAliases'];assert len(exclusions)==552
    excluded=[]
    for item in exclusions:
        path=item['path'];assert path not in entries and path not in admitted
        stat=(ROOT/path).stat();assert (ROOT/path).is_file()
        excluded.append({'path':path,'tracked_in_head':False,'physically_retained':True,'current_stat_bytes':stat.st_size,'body_read':False})
    profile='docs/operations/evidence/2026-10-31-i02-main-typed-wire/original/chrome-ingress-profile-ahhy89/'
    schemas='docs/operations/evidence/2026-10-31-i02-main-typed-wire/original/tauri-preexisting-schemas/'
    assert not any(p.startswith((profile,schemas)) for p in entries)
    assert not any(any('profile' in part.lower() and part.lower().startswith(('chrome','chromium','browser')) for part in p.split('/')[:-1]) for p in changed)
    legitimate='docs/operations/evidence/2026-10-31-i02-main-typed-wire/original/tauri-runtime-profile.json'
    assert legitimate in entries
    assert '.pi-lens.json' not in entries and (ROOT/'.pi-lens.json').is_file()
    binaries=[p for p in changed if p.endswith(('.wasm','.exe','.dmg','.msi','.dll','.dylib','.so'))];assert not binaries
    forbidden=[p for p in changed if any(part in ('node_modules','cargo-target','.pi','.claude','Cache','Code Cache','GPUCache') for part in p.split('/')) or p.endswith('session.jsonl')];assert not forbidden
    result['excluded552_verified']=len(excluded);result['legitimate_root_profile_report_retained']=legitimate
    (OUT/'exclusions552-path-stat.json').write_text(json.dumps({'head':SHA,'excluded':excluded,'qualification':'Path/stat only; no excluded bodies read'},indent=2)+'\n')
    result['committed_protected_sha256']={p:hashlib.sha256(git('show',SHA+':'+p)).hexdigest() for p in EXPECTED}
    assert result['committed_protected_sha256']==EXPECTED
    result['parent_committed_protected_sha256']={p:hashlib.sha256(git('show',OLD+':'+p)).hexdigest() for p in EXPECTED}
    assert result['parent_committed_protected_sha256']==EXPECTED
    result['source37_unchanged']=True
    checked=subprocess.run(['git','--no-optional-locks','cat-file','--batch-check=%(objectname) %(objecttype) %(objectsize)'],input=('\n'.join(entries[p]['oid'] for p in changed)+'\n').encode(),cwd=ROOT,env=ENV,capture_output=True,check=True)
    rows=[]
    for path,line in zip(changed,checked.stdout.decode().splitlines()):
        oid,kind,size=line.split();row={**entries[path],'bytes':int(size)};expected=admitted[path]
        assert kind=='blob' and oid==row['oid'] and row['mode']==expected['mode'] and row['bytes']==expected['bytes'];rows.append(row)
    assert len(rows)==9905 and sum(r['bytes'] for r in rows)==257498242
    (OUT/'accepted9905-tree-stat.json').write_text(json.dumps({'head':SHA,'tree':TREE,'entries':rows,'total_bytes':257498242,'qualification':'Fresh path/mode/OID/type/size audit; Source37/root bodySHA freshly verified atnew+parent+workingtree; parent all9905bodySHA/mode audit retained separately'},indent=2)+'\n')
    result.update(committed_increment_paths=changed,increment_blob_bytes=257498242,tracked_tree_paths=len(entries),tracked_dist_paths=sum(p.startswith('apps/web-client/dist/') for p in entries),new_compiled_binary_paths=binaries)
    objects={line.partition(' ')[0]:line.partition(' ')[2] for line in git('rev-list','--objects',SHA,'^'+OLD).decode().splitlines()}
    checked=subprocess.run(['git','--no-optional-locks','cat-file','--batch-check=%(objectname) %(objecttype) %(objectsize)'],input=('\n'.join(objects)+'\n').encode(),cwd=ROOT,env=ENV,capture_output=True,check=True)
    records=[]
    for line in checked.stdout.decode().splitlines():
        oid,kind,size=line.split();records.append({'oid':oid,'type':kind,'bytes':int(size),'path':objects[oid]})
    assert sum(r['type']=='commit' for r in records)==1
    result['new_object_count']=len(records);result['new_object_uncompressed_bytes_not_wire']=sum(r['bytes'] for r in records);result['new_max_blob']=max((r for r in records if r['type']=='blob'),key=lambda r:r['bytes'])
    assert result['new_max_blob']['bytes']<100*1024*1024
    (OUT/'new-objects-audit.json').write_text(json.dumps({'base':OLD,'new':SHA,'objects':records},indent=2)+'\n')
    hooks=OUT/'hooks';hooks.mkdir()
    old_hook=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-decision25-own-hooks-cs_zdbfm/pre-push').read_text()
    hook=old_hook.replace('accepted bounded Own-choice increment','accepted post-C9e documentation-only increment').replace('d07b35f94a42b282222dda4ab7bbc5c39ab8fddb',SHA).replace('5fc0f77403d90fa4555156bf01722450cbe7bf3b',OLD)
    hook=hook.replace("if local_ref == '(delete)' or local_oid != COMMIT or remote_ref != BRANCH:","if local_ref != COMMIT or local_oid != COMMIT or remote_ref != BRANCH:")
    (hooks/'pre-push').write_text(hook);(hooks/'pre-push').chmod(0o700)
    valid=SHA+' '+SHA+' refs/heads/native-convergence '+OLD+'\n'
    tests=[('accepted',URL,valid,0),('wrong-repository','https://example.invalid/other.git',valid,1),('wrong-ref-main',URL,valid.replace('refs/heads/native-convergence','refs/heads/main'),1),('wrong-new-bdaf',URL,valid.replace(SHA,BAD),1),('wrong-old-d07',URL,valid.replace(OLD,'d07b35f94a42b282222dda4ab7bbc5c39ab8fddb'),1),('delete-zero',URL,'(delete) '+'0'*40+' refs/heads/native-convergence '+OLD+'\n',1),('second-ref-multiline',URL,valid+valid.replace('refs/heads/native-convergence','refs/heads/main'),1),('missing-tuple',URL,'',1)]
    selftests=[]
    for name,url,stdin,want in tests:
        p=subprocess.run([str(hooks/'pre-push'),url,url],input=stdin.encode(),cwd=ROOT,env=ENV,capture_output=True)
        selftests.append({'name':name,'exit':p.returncode,'expected':want,'stdin':stdin,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()});assert p.returncode==want
    scope={'created_utc':json.loads((OUT/'lease-acquired.json').read_bytes())['acquired_utc'],'repository':URL,'branch':'refs/heads/native-convergence','accepted_commit':SHA,'accepted_tree':TREE,'expected_old_remote':OLD,'original_hook_sha256':HOOK_SHA,'scoped_hooks_path':str(hooks),'scoped_hook_sha256':hashlib.sha256((hooks/'pre-push').read_bytes()).hexdigest(),'selftests':selftests}
    (OUT/'scope.json').write_text(json.dumps(scope,indent=2)+'\n')
    (OUT/'protected-admission.json').write_text(json.dumps({'source37':SOURCE37,'expected':EXPECTED,'scope':'Exact unchanged34e source37+Root46; docs/evidenceonly, no newcode/runtime/default/version acceptance'},indent=2)+'\n')
    helper=Path('/tmp/noname-run-serialized.py');lock=Path('/tmp/noname-phase4-heavy-command.lock').stat()
    assert hashlib.sha256(helper.read_bytes()).hexdigest()=='d7f246f1fdbd4ec6d85deb22643d7ca8137be62539ebd5e75d49c779febb8e34'
    assert (lock.st_dev,lock.st_ino)==(16777231,620205209)
    GIT=['git','--no-optional-locks','-c','core.hooksPath='+str(hooks),'-c','credential.helper=','-c','credential.helper=!gh auth git-credential','-c','pack.threads=2']
    pre=run(GIT+['ls-remote','--heads',URL,'refs/heads/native-convergence','refs/heads/main'],'remote-before');assert pre.returncode==0
    result['remote_before_refs']={line.split()[1]:line.split()[0] for line in pre.stdout.decode().splitlines()}
    assert result['remote_before_refs']=={'refs/heads/main':MAIN,'refs/heads/native-convergence':OLD}
    snapshot('immediately_before_push');assert result['before']==result['immediately_before_push']
    result['preflight_passed']=True;result['push_attempts']=1
    argv=['python3','-B',str(helper)]+GIT+['push','--progress',URL,SHA+':refs/heads/native-convergence'];result['push_command']=argv
    t=time.monotonic()
    with (OUT/'push.stdout').open('xb') as stdout,(OUT/'push.stderr').open('xb') as stderr:
        proc=subprocess.Popen(argv,cwd=ROOT,env=ENV,stdout=stdout,stderr=stderr,start_new_session=True);result['push_pid']=proc.pid;save();result['push_exit']=proc.wait()
    result['push_seconds_including_queue']=time.monotonic()-t;save()
    if result['push_exit']!=0:raise RuntimeError('Single normal push failed; no retry/fallback')
    post=run(GIT+['ls-remote','--heads',URL,'refs/heads/native-convergence','refs/heads/main'],'remote-after');assert post.returncode==0
    result['remote_after_refs']={line.split()[1]:line.split()[0] for line in post.stdout.decode().splitlines()}
    assert result['remote_after_refs']=={'refs/heads/native-convergence':SHA,'refs/heads/main':MAIN}
    result['remote_verified']=True
except Exception as exc:
    result['error']=type(exc).__name__+': '+str(exc)
finally:
    try:
        snapshot('after');assert result['before']==result['after']
    except Exception as exc:result['preservation_error']=type(exc).__name__+': '+str(exc)
    result['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
    release={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'lease':'READONLY publication lease RELEASED; no Main writes; all further work onlyprivate CI/API/logs','head_sha':SHA,'push_exit':result.get('push_exit'),'push_attempts':result['push_attempts'],'remote_refs':result.get('remote_after_refs'),'error':result.get('error'),'preservation_error':result.get('preservation_error'),'push_completed_utc':result['finished_utc']}
    (OUT/'lease-release.json').write_text(json.dumps(release,indent=2)+'\n')
    print(json.dumps({'push_exit':result.get('push_exit'),'error':result.get('error'),'preservation_error':result.get('preservation_error'),'lease_release':release},indent=2),flush=True)
