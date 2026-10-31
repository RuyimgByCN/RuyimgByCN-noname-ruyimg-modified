python3 - <<'PY'
from pathlib import Path
import struct,hashlib,json,datetime
E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5')
def parse(p):
 raw=p.read_bytes();assert raw[:4]==b'DIRC';version,count=struct.unpack('>II',raw[4:12]);assert version in (2,3,4);off=12;entries=[];previous=b''
 for _ in range(count):
  start=off;values=struct.unpack('>10I',raw[off:off+40]);off+=40;oid=raw[off:off+20].hex();off+=20;flags=struct.unpack('>H',raw[off:off+2])[0];off+=2;ext=0
  if flags&0x4000:ext=struct.unpack('>H',raw[off:off+2])[0];off+=2
  if version==4:
   byte=raw[off];off+=1;remove=byte&127
   while byte&128:byte=raw[off];off+=1;remove=((remove+1)<<7)+(byte&127)
   end=raw.index(0,off);name=previous[:len(previous)-remove]+raw[off:end];off=end+1
  else:
   end=raw.index(0,off);name=raw[off:end];off=end+1;off=start+((off-start+7)//8)*8
  previous=name;names=['ctimeSeconds','ctimeNanos','mtimeSeconds','mtimeNanos','device','inode','mode','uid','gid','size'];row=dict(zip(names,values));row.update({'path':name.decode(),'oid':oid,'stage':(flags>>12)&3,'ordinaryFlags':flags,'extendedFlags':ext,'assumeUnchanged':bool(flags&0x8000),'skipWorktree':bool(ext&0x4000),'intentToAdd':bool(ext&0x2000)});entries.append(row)
 extensions=[]
 while off<len(raw)-20:
  sig=raw[off:off+4].decode('ascii');size=struct.unpack('>I',raw[off+4:off+8])[0];off+=8;payload=raw[off:off+size];off+=size;extensions.append({'signature':sig,'size':size,'sha256':hashlib.sha256(payload).hexdigest()})
 assert off==len(raw)-20;assert hashlib.sha1(raw[:-20]).digest()==raw[-20:]
 return {'path':str(p),'version':version,'count':count,'sha256':hashlib.sha256(raw).hexdigest(),'trailingChecksum':raw[-20:].hex(),'checksumVerified':True,'extensions':extensions,'entries':entries,'fileMtimeUtc':datetime.datetime.fromtimestamp(p.stat().st_mtime,datetime.timezone.utc).isoformat()}
a=parse(E/'before-index.raw');b=parse(E/'gate-entry-index.raw');c=parse(E/'gate-exit-index.raw');assert len(a['entries'])==len(b['entries'])==len(c['entries']);diff=[]
for x,y in zip(a['entries'],b['entries']):
 d={k:{'before':x[k],'after':y[k]} for k in x if x[k]!=y[k]}
 if d:diff.append({'pathBefore':x['path'],'pathAfter':y['path'],'fields':d})
statkeys={'ctimeSeconds','ctimeNanos','mtimeSeconds','mtimeNanos','device','inode','uid','gid','size'};behavior=[d for d in diff if not set(d['fields'])<=statkeys];x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'initial':{k:v for k,v in a.items() if k!='entries'},'gateEntry':{k:v for k,v in b.items() if k!='entries'},'gateExit':{k:v for k,v in c.items() if k!='entries'},'changedEntries':diff,'changedEntriesCount':len(diff),'changedFields':sorted({k for d in diff for k in d['fields']}),'behaviorSignificantEntryChanges':behavior,'sameVersion':a['version']==b['version'],'sameExtensions':a['extensions']==b['extensions'],'gateEntryExitRawExact':b['sha256']==c['sha256'],'allStagesZero':all(x['stage']==0 for x in a['entries']+b['entries']),'assumeUnchangedFlagsBeforeAfterEqual':all(x['assumeUnchanged']==y['assumeUnchanged'] for x,y in zip(a['entries'],b['entries'])),'skipWorktreeFlagsBeforeAfterEqual':all(x['skipWorktree']==y['skipWorktree'] for x,y in zip(a['entries'],b['entries'])),'intentToAddFlagsBeforeAfterEqual':all(x['intentToAdd']==y['intentToAdd'] for x,y in zip(a['entries'],b['entries'])),'qualification':'Raw mutation origin remains unknown. This parser identifies bytes/fields, not the writer or cause. No stat/cache attribution to a process, no index repair. First seal assertion failure retained.'};(E/'index-raw-field-diff.json').write_text(json.dumps(x,indent=2));print(json.dumps(x,indent=2)[:22000])
PY