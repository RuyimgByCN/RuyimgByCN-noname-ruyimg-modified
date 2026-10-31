from pathlib import Path
import json,hashlib,gzip,shutil,datetime
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');B=M/'docs/operations/evidence';O=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs');E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/nc10-main-own-sfobj6i8');N=Path('/tmp/nc10-main-lock-repair-0425dc78');I=Path('/tmp/i02-phase1-ingress-repair-89364384');C=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9');records=[]
def sha(b):return hashlib.sha256(b).hexdigest()
def copy(src,dst,compress=False):
 data=src.read_bytes();out=gzip.compress(data,mtime=0) if compress else data
 if compress:dst=dst.with_name(dst.name+'.gz')
 dst.parent.mkdir(parents=True,exist_ok=True)
 if dst.exists():assert dst.read_bytes()==out,('refuse overwrite',str(dst))
 else:dst.write_bytes(out)
 assert (gzip.decompress(dst.read_bytes()) if compress else dst.read_bytes())==data
 records.append({'origin':str(src),'destination':str(dst.relative_to(M)),'original_sha256':sha(data),'original_bytes':len(data),'archive_sha256':sha(out),'archive_bytes':len(out),'lossless_gzip':compress})
A=B/'2026-10-31-nc10-public-own-plan/main-acceptance'
retained=[]
for p in sorted(E.rglob('*')):
 if not p.is_file():continue
 rel=p.relative_to(E);r=str(rel)
 excluded=r.startswith(('frontend-dist/','old-generated18/','probe/target/','strict-target/'))
 if excluded and r not in ['probe/target/flycheck0/stdout','probe/target/flycheck0/stderr']:
  retained.append({'origin':str(p),'sha256':sha(p.read_bytes()),'bytes':p.stat().st_size,'reason':'Private compiled/bundled product or build cache retained at original path; original artifact index and parent714 hash audit preserved.'});continue
 copy(p,A/'original'/rel,p.stat().st_size>1024**2 and p.suffix in ['.json','.txt','.log'])
for p in sorted(N.rglob('*')):
 if not p.is_file():continue
 rel=p.relative_to(N)
 if str(rel).startswith('binary/'):
  retained.append({'origin':str(p),'sha256':sha(p.read_bytes()),'bytes':p.stat().st_size,'reason':'Executed native probe binary retained privately; identity/command/copy proofs archived.'});continue
 copy(p,A/'lock-aligned'/rel,p.stat().st_size>1024**2 and p.suffix in ['.json','.txt','.log'])
for p in sorted(Path('/tmp/nc10-main-own-provenance-addendum').rglob('*')):
 if p.is_file():copy(p,A/'sidecheck-provenance'/p.relative_to('/tmp/nc10-main-own-provenance-addendum'))
for run,name,dest in [('df47e19c-73c6-4cdd-a45e-5437618a62f2','verification/nc10-public-own-main-functional.md','verification.raw.txt'),('ccc7cb5a-4755-4f88-9488-3fcfb9eff542','reviews/nc10-public-own-main-combined-acceptance.md','combined-review.raw.txt'),('0425dc78-418a-456c-ab39-0848d485f156','verification/nc10-main-private-probe-target-provenance.md','sidecheck-provenance/report.raw.txt'),('ba244d6f-4544-490a-987a-9f50c0845230','verification/nc10-main-lock-aligned-probe.md','lock-aligned/report.raw.txt'),('8ab46404-9cb9-4398-84dc-6337639541d2','reviews/nc10-public-own-main-lock-addendum.md','lock-aligned/review-addendum.raw.txt'),('26de4308-cc55-44cb-98a4-35a784618498','verification/nc10-main-doc-freeze-release.md','doc-freeze-release.raw.txt')]:copy(O/run/name,A/dest)
for p in list(Path('/tmp').glob('noname-parent-nc10-main-final-audit*'))+list(Path('/tmp').glob('noname-parent-nc10-main-lock-audit.json'))+[Path('/tmp/noname-parent-supplemental-evidence-audit.py'),Path('/tmp/noname-parent-supplemental-evidence-audit.raw.txt')]:
 if p.is_file():copy(p,A/'parent'/p.name)
IA=B/'2026-10-31-i02-public-presentation-contract/phase1-ingress-repair'
for p in sorted(I.rglob('*')):
 if p.is_file():copy(p,IA/'original'/p.relative_to(I),p.stat().st_size>1024**2 and p.suffix in ['.json','.txt','.log'])
copy(O/'89364384-4d48-4c3e-aed8-5cacabaa3b52/implementation/i02-phase1-ingress-repair.md',IA/'implementation.raw.txt');copy(O/'f85a7a8d-2441-46b1-a67a-b8e1e0cec83c/reviews/i02-phase1-ingress-repair.md',IA/'independent-review.raw.txt');copy(Path('/tmp/noname-parent-i02-ingress-repair-audit.json'),IA/'parent-audit.json')
CA=B/'2026-10-31-c331-phase9-partial-damage'
for p in sorted(C.rglob('*')):
 if p.is_file():copy(p,CA/'original'/p.relative_to(C),p.stat().st_size>1024**2 and p.suffix in ['.json','.txt','.log'])
copy(O/'53f077bd-d71f-4a84-951d-fe5af72ea145/implementation/c331-phase9-canonical-damage.md',CA/'implementation.raw.txt');copy(O/'b6787aca-5e82-4428-b772-c1f509f93a6e/reviews/c331-phase9-partial-triage.md',CA/'independent-triage.raw.txt');copy(Path('/tmp/noname-parent-c9-final-audit.json'),CA/'parent-audit.json')
manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'qualification':'Post-freeze evidence archive; new files were not inputs to the 11:42:19–11:49:59 frozen preflight. Original reports/seals unchanged; alternate gzip names mapped losslessly. Private binaries/caches remain unmodified and are not force-added. C9 remains blocked with8 failures. I02 only private typed-wire foundation. Main Own bounded acceptance independently reviewed and lock-aligned.','copied_files':len(records),'archive_bytes':sum(r['archive_bytes'] for r in records),'copies':records,'private_retained':retained}
(A/'archive-copy-map.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');Path('/tmp/noname-parent-latest-evidence-archive.json').write_text(json.dumps({'copied_files':manifest['copied_files'],'archive_bytes':manifest['archive_bytes'],'map':str(A/'archive-copy-map.json'),'roots':[str(A),str(IA),str(CA)]},indent=2)+'\n');print(json.dumps({k:v for k,v in manifest.items() if k not in ['copies','private_retained']},ensure_ascii=False,indent=2))
