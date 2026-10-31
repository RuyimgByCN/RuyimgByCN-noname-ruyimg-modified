from pathlib import Path
import json,hashlib,gzip,datetime,shutil
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');B=M/'docs/operations/evidence';O=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs');E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5');rows=[];retained=[]
def h(b):return hashlib.sha256(b).hexdigest()
def copy(src,dst):
 data=src.read_bytes();original=dst
 if dst.suffix=='.md':dst=dst.with_suffix('.raw.txt')
 compressed=len(data)>1024**2 and dst.suffix in ['.json','.raw','.txt','.log','.stdout','.stderr']
 payload=gzip.compress(data,mtime=0) if compressed else data
 if compressed:dst=dst.with_name(dst.name+'.gz')
 dst.parent.mkdir(parents=True,exist_ok=True)
 if dst.exists():assert dst.read_bytes()==payload,('refuseoverwrite',str(dst))
 else:dst.write_bytes(payload)
 assert (gzip.decompress(dst.read_bytes()) if compressed else dst.read_bytes())==data
 rows.append({'source':str(src),'destination':str(dst.relative_to(M)),'originalSha256':h(data),'archiveSha256':h(payload),'originalBytes':len(data),'archiveBytes':len(payload),'losslessGzip':compressed,'rawMarkdownName':original.suffix=='.md'})
A=B/'2026-10-31-i02-main-typed-wire';art=json.loads((E/'artifact-sha256.json').read_bytes())
for rel,r in art['files'].items():
 skip=rel.startswith(('executed-binaries/','strict-target/','frontend-dist/','ingress-dist/','chrome-profile/','probe/target/')) or Path(rel).suffix in ['.wasm','.rlib','.rmeta','.dylib','.exe','.o','.a','.so','.pyc'] or '__pycache__/' in rel
 if skip:retained.append({'source':str(E/rel),**r,'reason':'Private retained runtime/bundle/build product; no force-add or compiled-cache archive'});continue
 copy(E/rel,A/'original'/rel)
for name in ['artifact-sha256.json','seal-root.json']:
 if (E/name).exists():copy(E/name,A/name)
copy(O/'33d21ba5-8777-4019-8cfd-eb9a3d17176a/verification/i02-main-typed-wire-functional.md',A/'verification.raw.txt');copy(O/'c271f25c-72e3-4bf5-97ee-49df9db6c81c/reviews/i02-main-typed-wire-combined-acceptance.md',A/'combined-review-with-P2.raw.txt')
for src in ['/tmp/noname-parent-i02-main-final-audit.py','/tmp/noname-parent-i02-main-final-audit.json','/tmp/noname-parent-i02-main-final-audit.raw.txt','/tmp/noname-parent-i02-main-final-audit.attempt1.raw.txt','/tmp/noname-parent-i02-main-integration-current.json','/tmp/noname-parent-windows-checkout-main-integration.json','/tmp/noname-parent-i02-typescript-lsp.json']:
 copy(Path(src),A/'parent'/Path(src).name)
S=Path('/tmp/i02-main-index-command-provenance-599614a0')
for name in ['manifest.json','artifact-sha256.json','preserve.py','index-parse.raw.sh','index-mask-and-js-load.raw.sh','index-parse.tool-result-record.raw.jsonl','index-mask-and-js-load.tool-result-record.raw.jsonl']:
 copy(S/name,A/'index-command-supplement'/name)
for p in Path('/tmp/noname-parent-i02-index-tool-provenance').iterdir():
 if p.is_file():copy(p,A/'index-command-supplement/tool-only'/p.name)
copy(O/'599614a0-9b12-4abe-a51d-3a9113f87c3c/verification/i02-main-index-command-provenance-supplement.md',A/'index-command-supplement/report.raw.txt');copy(O/'8e8f2f39-bcd7-41ff-94f8-1d14f4f3e33e/reviews/i02-main-index-command-supplement.md',A/'index-command-supplement/review-closure.raw.txt');copy(Path('/tmp/noname-parent-i02-index-command-supplement-audit.json'),A/'index-command-supplement/parent-audit.json')
# Source assistant/session raw records remain private; exact tool-only excerpts above suffice.
for p in S.glob('*.assistant-record.raw.jsonl'):retained.append({'source':str(p),'sha256':h(p.read_bytes()),'bytes':p.stat().st_size,'reason':'Private full assistant raw record; publish only tool-call/result excerpts and exact command bytes'})
CIpacket=Path(json.loads(Path('/tmp/noname-ci41202703042-ready-current.json').read_bytes())['root']);CI=B/'2026-10-31-nc10-public-own-plan/publication-ci'
for p in sorted(CIpacket.rglob('*')):
 if p.is_file():copy(p,CI/p.relative_to(CIpacket))
R=Path('/tmp/nc10-main-owned-target-retirement-f463dd87');RA=B/'2026-10-31-nc10-public-own-plan/target-retirement'
for p in sorted(R.iterdir()):
 if p.is_file():copy(p,RA/p.name)
copy(O/'f463dd87-507a-4098-bf97-65ba70450664/verification/nc10-main-owned-target-retirement.md',RA/'report.raw.txt');copy(Path('/tmp/noname-parent-nc10-target-retirement-audit.json'),RA/'parent-audit.json')
D=B/'2026-10-31-c331-death-authority-plan';copy(O/'ea42b092-88ff-4701-affa-cad6e1642d33/proposals/c29-vm-death-shared-authority-plan.md',D/'proposal.raw.txt');copy(O/'8791f981-fc44-495e-a32d-f9f0f9383dca/reviews/c29-vm-death-shared-authority-plan.md',D/'review.raw.txt');copy(Path('/tmp/c29-vm-death-ea42/source-inventory.json'),D/'source-inventory.json')
C=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9b');CA=B/'2026-10-31-c331-phase9b-prehp'
ci=json.loads((C/'artifact-index.json').read_bytes())
for r in ci['files']:copy(C/r['path'],CA/'original'/r['path'])
copy(C/'artifact-index.json',CA/'artifact-index.json')
for p in sorted((C/'scope-fix-checks').iterdir()):
 if p.is_file():copy(p,CA/'parent-owned-validation'/p.name)
copy(O/'b6787aca-5e82-4428-b772-c1f509f93a6e/implementation/c331-phase9b-prehp-runtime.md',CA/'implementation.raw.txt');copy(O/'e2746e76-3e4f-4313-b531-8b5880608c2d/reviews/c331-phase9b-final-runtime.md',CA/'review-two-P1.raw.txt');copy(Path('/tmp/noname-parent-c9b-final-audit.json'),CA/'parent-audit.json');copy(Path('/tmp/noname-parent-c9b-final-audit.py'),CA/'parent-audit.py')
ready=json.loads(Path('/tmp/noname-physical-readiness-latest.json').read_bytes());copy(Path('/tmp/noname-physical-readiness-latest.json'),A/'physical-readiness-summary.json')
record={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'PostI02freeze14:03:47-release archival only; no source/golden changes. Originalrecordedgatecategories12 not13; oldwrongparserfiles preserved, truecommandsadditive. CIoriginald07 remains8/2, C9b final1791/1+2P1 stillBLOCK. Private source-assisted records are not publication material.','copies':rows,'retainedPrivate':retained,'copiedFiles':len(rows),'archiveBytes':sum(r['archiveBytes'] for r in rows)};(A/'archive-copy-map.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n');Path('/tmp/noname-parent-i02-main-archive-current.json').write_text(json.dumps({k:v for k,v in record.items() if k not in ['copies','retainedPrivate']},ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items() if k not in ['copies','retainedPrivate']},ensure_ascii=False,indent=2))
