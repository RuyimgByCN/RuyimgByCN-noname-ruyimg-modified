from pathlib import Path
import json,hashlib,os,collections
from accept import ROOT,OUT,CORE,save
m=json.loads((ROOT/'content/original/asset-manifest.json').read_text());old=json.loads((OUT.parent/'parent-before-asset-manifest.json').read_text());direct=m['assets'][:12359];aliases=m['assets'][12359:];assert direct==old['assets'];assert len(aliases)==23393;assert {k:v for k,v in m.items() if k!='assets'}=={k:v for k,v in old.items() if k!='assets'}
by_path={a['path']:a for a in direct};assert len(by_path)==12359;cache={};rows=[]
def physical_hash(p):
 s=p.stat();key=(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)
 if key not in cache:cache[key]=hashlib.sha256(p.read_bytes()).hexdigest()
 return s,cache[key]
expected_fields={'kind','path','size','source_category','source_path','sha256'}
for a in aliases:
 assert set(a)==expected_fields,a
 backing=by_path[a['source_path']];assert {k:v for k,v in a.items() if k!='path'}=={k:v for k,v in backing.items() if k!='path'}
 paths=[CORE/a['source_path'],ROOT/'content/original/assets'/a['source_path'],ROOT/'content/original/assets'/a['path']];states=[physical_hash(p) for p in paths]
 assert all(s.st_size==a['size'] and h==a['sha256'] for s,h in states),a
 rows.append({'path':a['path'],'source_path':a['source_path'],'bytes':a['size'],'sha256':a['sha256'],'upstream_backing_target_sha_equal':True,'same_inode_chain':len({(s.st_dev,s.st_ino) for s,h in states})==1})
target=ROOT/'content/original/assets/audio-declared';actual=sorted('audio-declared/'+str(p.relative_to(target)) for p in target.rglob('*') if p.is_file());assert actual==sorted(a['path'] for a in aliases)
save('alias-byte-evidence.json',rows);summary={'manifest_rows':len(m['assets']),'old_direct_rows_exact_in_order':12359,'nonassets_metadata_exact':True,'aliases_verified_upstream_backing_target':len(rows),'target_files':len(actual),'unique_physical_payloads_hashed':len(cache),'hash_cache_key':'dev/inode/size/mtime; hardlinks share actual bytes, no redundant 12kcodec or full source reread','all_six_fields':True,'new_material_bytes':0,'hardlink_chain_count':sum(r['same_inode_chain'] for r in rows)};save('byte-audit-summary.json',summary);print(json.dumps(summary))
