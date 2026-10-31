from pathlib import Path
import tarfile,tempfile,json,hashlib,re
R=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase5');F=R/'final-freeze5';OLD=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase4-review-ibn3o01q/snapshot')
O=Path(tempfile.mkdtemp(prefix='noname-c331-phase5-parent-'));S=O/'snapshot';S.mkdir()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_bytes())
assert sha(F/'source-manifest.json')=='cbeab39b1316d77df884242b5494c8b7da2cc5baf8737431ab8a716da6eaaf16'
assert sha(F/'source-files.tar.gz')=='34023da5cc41d2a7971ad5b3c1a8396a49f0ff40d4a8141e7876fdfada107aef'
m=load(F/'source-manifest.json');expected={row['path']:row['sha256'] for row in m['files']};assert len(expected)==135
with tarfile.open(F/'source-files.tar.gz','r:gz') as t:
 members=[x for x in t.getmembers() if x.isfile()];assert len(members)==135
 for x in t.getmembers():
  assert x.isdir() or x.isfile(),x.name
  dest=(S/x.name).resolve();assert dest.is_relative_to(S.resolve()),x.name
  if x.isdir():continue
  assert x.name in expected,x.name;b=t.extractfile(x).read();assert hashlib.sha256(b).hexdigest()==expected[x.name];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b);dest.chmod(0o444)
for path,digest in expected.items():assert sha(S/path)==digest
changes=[path for path in expected if not (OLD/path).exists() or sha(OLD/path)!=expected[path]];assert len(changes)==3;assert all(not p.is_file() or str(p.relative_to(OLD)) in expected for p in OLD.rglob('*'))
assert sha(F/'evidence-manifest.json')=='54f676804a99d3a8614942f8985761eb23f1079fcbd6b1c67c75cb5651d8901f'
e=load(F/'evidence-manifest.json');verified=[]
for row in e:
 p=R/row['path'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'];verified.append({'path':row['path'],'bytes':row['bytes'],'sha256':row['sha256']})
counts={}
for name,expected_counts in [('engine-native-full.log',(1739,0,75)),('affected-default-final.log',(2906,0,76)),('source-order-red.log',(0,3,0)),('nc08-green.log',(77,0,50))]:
 text=(R/name).read_text();rows=[tuple(map(int,x)) for x in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)];actual=tuple(sum(x[i] for x in rows) for i in range(3));assert actual==expected_counts,(name,actual);counts[name]={'sha256':sha(R/name),'result_rows':len(rows),'passed':actual[0],'failed':actual[1],'ignored':actual[2]}
for name in ['validated-ordering44.json','current-bindings44.json','registration-supplement.json','calculator-corrections.json','remaining28.json','full-results.json','source-and-result-binding.json']:
 load(R/name)
identity={'result':'PASS','snapshot':str(S),'source_manifest_sha256':sha(F/'source-manifest.json'),'source_tar_sha256':sha(F/'source-files.tar.gz'),'source_files_verified':135,'changed_from_final4':changes,'unchanged_from_final4':132,'evidence_manifest_sha256':sha(F/'evidence-manifest.json'),'evidence_entries_verified':len(verified),'evidence_entries':verified,'raw_result_counts':counts,'qualification':'Fresh bounded retained-byte/hash/tar/count verification only; no producer or product tests run; source semantics/proven262 remain independent review scope.'}
(O/'identity.json').write_text(json.dumps(identity,indent=2)+'\n');Path('/tmp/noname-c331-phase5-parent-identity.json').write_text(json.dumps(identity,indent=2)+'\n');print(json.dumps({k:v for k,v in identity.items() if k!='evidence_entries'},indent=2))
