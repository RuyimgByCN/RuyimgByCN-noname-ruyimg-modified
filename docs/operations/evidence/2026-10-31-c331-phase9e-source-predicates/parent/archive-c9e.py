from pathlib import Path
import json,hashlib,gzip,datetime,stat
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9e');S=E.parent/'phase9e-concordance-supplement';A=M/'docs/operations/evidence/2026-10-31-c331-phase9e-source-predicates';O=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs');h=lambda b:hashlib.sha256(b).hexdigest();rows=[]
def safe(rel):
 p=Path(rel)
 return not p.is_absolute() and '..' not in p.parts and not any(x.lower() in ['node_modules','target','.git','cache','caches'] or (x.lower().startswith(('chrome','chromium','browser')) and ('profile' in x.lower() or 'cache' in x.lower())) for x in p.parts[:-1]) and not any(x in p.name.lower() for x in ['assistant-record','assistant-session','full-session']) and p.suffix not in ['.wasm','.rlib','.rmeta','.dylib','.exe','.o','.a','.so','.pyc']
assert safe('checks-final4/predicate-traces/001.snapshot.txt') and safe('full-source/source/crates/engine-native/src/lib.rs')
for p in ['original/chrome-ingress-profile-ahhy89/Cache/data','original/chromium-profile-abc/Preferences','original/browser-profile-1/cookies','../outside.json','/private/outside.json','original/target/probe.o','original/full-assistant-record.jsonl']:assert not safe(p),p
def put(src,rel,wanted=None):
 assert safe(rel),rel
 assert src.is_file() and not src.is_symlink(),str(src)
 data=src.read_bytes();assert wanted is None or h(data)==wanted,str(src);dst=A/rel
 if dst.suffix=='.md':dst=dst.with_suffix('.raw.txt')
 compressed=len(data)>1024**2 and dst.suffix in ['.json','.log','.txt','.raw'];body=gzip.compress(data,mtime=0) if compressed else data
 if compressed:dst=dst.with_name(dst.name+'.gz')
 dst.parent.mkdir(parents=True,exist_ok=True)
 if dst.exists():assert dst.read_bytes()==body,('refuseoverwrite',str(dst))
 else:dst.write_bytes(body)
 assert (gzip.decompress(dst.read_bytes()) if compressed else dst.read_bytes())==data
 rows.append({'source':str(src),'destination':str(dst.relative_to(M)),'sourceSHA256':h(data),'archiveSHA256':h(body),'sourceBytes':len(data),'archiveBytes':len(body),'sourceMode':oct(stat.S_IMODE(src.stat().st_mode)),'losslessGzip':compressed})
idx=json.loads((E/'artifact-index.json').read_bytes());assert len(idx['files'])==7333
for x in idx['files']:put(E/x['path'],'original/'+x['path'],x['sha256'])
for name in ['artifact-index.json','artifact-index.sha256']:put(E/name,name)
put(O/'0b7ce162-d228-460e-a9c9-7fb28e076018/implementation/c331-phase9e-source-predicate-repair.md','implementation.raw.txt','8a576676e38e5c04f56d45cba004ed3abece45236a7544f5e7309d49e2ebda74')
sidx=json.loads((S/'artifact-index.json').read_bytes())
for x in sidx['files']:put(S/x['path'],'concordance-supplement/'+x['path'],x['sha256'])
for name in ['artifact-index.json','artifact-index.sha256']:put(S/name,'concordance-supplement/'+name)
put(O/'4fbbc6a5-9caa-452b-97d2-644fefd75611/reviews/c331-phase9e-predicates-and-concordance.md','independent-review-BLOCK.raw.txt')
for name in ['noname-parent-c9e-final-audit.py','noname-parent-c9e-final-audit.json','noname-parent-c9e-final-audit.raw.txt','noname-parent-c9e-final-audit.attempt1.raw.txt','noname-parent-c9e-audit-preparation-note.json','noname-parent-c9e-concordance-supplement-audit.json','noname-parent-c9e-concordance-source-bindings-audit.json']:
 put(Path('/tmp')/name,'parent/'+name)
record={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'mainHead':'34e83f0550bb70ef4af6083697428de7be2f02f8','scope':'Parent postpublication docs/evidence only, C9e219/14 source seal + original7333artifacts and separate READONLY41sourceaudit and BLOCKreview. No CproductionMainimport/source/generated/default/version/lock changes. Original failed attempts preserved, profile/cache/privatefullsession/binaries excluded by asserted directory classification.','files':len(rows),'sourceBytes':sum(x['sourceBytes'] for x in rows),'archiveBytes':sum(x['archiveBytes'] for x in rows),'copies':rows,'acceptance':'Fiveboundedcounterexamplesfixed/reviewed; new17sourcefindinggroups + chainbirthP1 and12unprovedrows remainmandatory; no goldenrerecord/C331/Goal closure.'};(A/'archive-map.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n');Path('/tmp/noname-parent-c9e-archive-summary.json').write_text(json.dumps({k:x for k,x in record.items() if k!='copies'},ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:x for k,x in record.items() if k!='copies'},ensure_ascii=False,indent=2))
