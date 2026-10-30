from pathlib import Path
import hashlib,json,subprocess,collections
R=Path('/tmp/noname-ci-777-checkout-selfcheck-nisccqfo');C=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-777bf339-publication-y7r1p6p5');P=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
s=json.loads((R/'summary.json').read_bytes());rows=[]
for name,meta in s['old_evidence_unchanged'].items():
 b=(C/name).read_bytes();assert len(b)==meta['bytes'] and hashlib.sha256(b).hexdigest()==meta['sha256'];rows.append({'path':str(C/name),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
push=json.loads((C/'push-stage/result.json').read_bytes());assert push['committed_source14_sha256']==s['source14_before']
run=json.loads((C/'ci-stage/run-final.stdout').read_bytes());jobs=json.loads((C/'ci-stage/jobs-final.stdout').read_bytes());jobs=jobs['jobs'];counts=dict(collections.Counter(j['conclusion'] for j in jobs));assert counts=={'success':9,'failure':1};windows=next(j for j in jobs if j['id']==124265455086);assert windows['conclusion']=='failure'
assert run['id']==41114612841 and run['head_sha']==s['head'] and run['run_attempt']==1 and run['conclusion']=='failure'
a=subprocess.run(['git','--no-optional-locks','-C',str(P),'ls-tree',s['head'],'--','content/.gitattributes','content/original-audio-compat/.gitattributes'],check=True,capture_output=True);assert a.stdout==b''
out={'result':'PASS','old_evidence':rows,'publication_source14_equals_reviewed14':True,'run':{k:run[k] for k in ['id','head_sha','run_attempt','conclusion']},'jobs_conclusion_counts':counts,'windows_job':{k:windows[k] for k in ['id','name','conclusion']},'ancestor_attributes_ls_tree_stdout':a.stdout.decode(),'scope':'read-only existing local bytes and ls-tree only; no network, experiments, or producer execution'}
Path('/tmp/noname-parent-windows-byte-audit/publication-invariants.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
