from pathlib import Path
import json,hashlib,subprocess,os,struct,collections,datetime
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');obs=json.loads(Path('/tmp/noname-parent-doc-index-guard-observed.json').read_bytes());D=Path(obs['root']);raw=(D/'observed-index.raw').read_bytes();env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
def git(*a,input=None):return subprocess.check_output(['git','--no-optional-locks','-C',str(M),*a],env=env,input=input)
assert hashlib.sha1(raw[:-20]).digest()==raw[-20:] and raw[:4]==b'DIRC';version,count=struct.unpack('>II',raw[4:12]);assert version in [2,3];off=12;parsed={}
for _ in range(count):
 start=off;stats=struct.unpack('>10I',raw[off:off+40]);oid=raw[off+40:off+60].hex();flags=struct.unpack('>H',raw[off+60:off+62])[0];off+=62;extended=0
 if flags&0x4000:extended=struct.unpack('>H',raw[off:off+2])[0];off+=2
 end=raw.index(b'\0',off);path=raw[off:end].decode();off=start+((end+1-start+7)//8)*8;parsed[path]={'oid':oid,'mode':format(stats[6],'o'),'stage':(flags>>12)&3,'flags':flags,'extended':extended,'stat':list(stats)}
entries_end=off;extensions=[]
while off<len(raw)-20:
 sig=raw[off:off+4].decode();size=struct.unpack('>I',raw[off+4:off+8])[0];payload=raw[off+8:off+8+size];assert len(payload)==size;extensions.append({'signature':sig,'size':size,'sha256':hashlib.sha256(payload).hexdigest(),'start':off,'end':off+8+size});off+=8+size
assert off==len(raw)-20;head={}
for rec in git('ls-tree','-r','-z','HEAD').split(b'\0'):
 if rec:
  meta,name=rec.split(b'\t',1);mode,kind,oid=meta.decode().split();assert kind=='blob';head[name.decode()]=(mode,oid)
P=json.loads(Path('/tmp/noname-post34e-doc-stage-plan.json').read_bytes());expected={r['path']:r for r in P['files']};assert set(parsed)==set(head)|set(expected);assert count==53864;assert all(r['stage']==0 and not(r['flags']&0x8000) and r['extended']==0 for r in parsed.values())
for name,r in parsed.items():
 if name not in expected:assert (r['mode'],r['oid'])==head[name],name
for name,r in expected.items():assert parsed[name]['mode']==r['mode']
paths=sorted(expected);oids=sorted({parsed[p]['oid'] for p in paths});output=git('cat-file','--batch',input=('\n'.join(oids)+'\n').encode());o=0;blobs={}
for oid in oids:
 end=output.index(b'\n',o);got,kind,size=output[o:end].decode().split();size=int(size);body=output[end+1:end+1+size];assert got==oid and kind=='blob';blobs[oid]=(size,hashlib.sha256(body).hexdigest());o=end+size+2
assert o==len(output)
for p,r in expected.items():assert blobs[parsed[p]['oid']]==(r['bytes'],r['sha256']),p
without_tree=raw[:entries_end]+b''.join(raw[e['start']:e['end']] for e in extensions if e['signature']!='TREE');without_tree+=hashlib.sha1(without_tree).digest();allflags=collections.Counter((r['stage'],bool(r['flags']&0x8000),r['extended']) for r in parsed.values());v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_INDEX_LOGICAL_OLD2540_EXACT_RAW_CHANGE_RETAINED','observedSHA256':hashlib.sha256(raw).hexdigest(),'expectedOldSHA256':obs['indexExpectedAt1636'],'rawBaselineOldFbAvailable':False,'indexVersion':version,'entries':count,'gitSHA1FooterValid':True,'extensions':extensions,'allPathOidModeStageEqualHEADPlusOld2540Plan':True,'all2540StagedBlobSHA256AndBytesExact':True,'flags':str(allflags),'unknownStatOrCacheWriter':'UNKNOWN; cannotclaimexactctime-onlywithoutoldfbraw','removeTREECandidateSHA256':hashlib.sha256(without_tree).hexdigest(),'removeTREEEqualsOldFb':hashlib.sha256(without_tree).hexdigest()==obs['indexExpectedAt1636'],'retainedCurrentRaw':str(D/'observed-index.raw'),'guardBeforeStageFailedSafely':True,'noIndexRestoreOrMutation':True,'authorizationForNextStep':'Normalgitaddonlyenumerated9905Docsunder18/25canproceedafterretainingthisdiagnostic; notrestorationandnotbaselineblanketrefresh. Freshactualstageblob/modeauditandrecordraw626->newintentionaldelta. Preserveunexpectedrawhash+unknownwriterasfact.'};assert hashlib.sha256((M/'.git/index').read_bytes()).hexdigest()==v['observedSHA256'];Path('/tmp/noname-parent-doc-index-logical-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');(D/'parsed-index.json').write_text(json.dumps(parsed,ensure_ascii=False)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
