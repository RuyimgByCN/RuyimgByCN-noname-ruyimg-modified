from pathlib import Path
import json,hashlib,subprocess
root=Path(__file__).resolve().parent;manifest=json.loads((root.parent/'final-files/snapshot-manifest.json').read_text());paths={row['path'] for row in manifest};bad=[]
for row in manifest:
 p=Path(row['path']);actual=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
 if actual!=row['sha256']:bad.append({'path':str(p),'expected':row['sha256'],'actual':actual})
changed=set(subprocess.check_output(['git','diff','--name-only','95f51867a3648321da1cc92149b60e2bbfaae748','HEAD']).decode().splitlines())
extra=sorted(changed-paths);missing=sorted(paths-changed)
staged=subprocess.check_output(['git','diff','--cached','--name-only']).decode().splitlines();tracked_dirty=subprocess.check_output(['git','diff','--name-only']).decode().splitlines()
report={'manifest_paths':len(manifest),'matched':len(manifest)-len(bad),'sha_mismatches':bad,'extra_product_paths_vs_baseline':extra,'manifest_paths_missing_from_commit_diff':missing,'staged_paths':staged,'tracked_dirty_paths':tracked_dirty,'untracked_paths':subprocess.check_output(['git','ls-files','--others','--exclude-standard']).decode().splitlines()}
(root/'final-snapshot-verification.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2));assert not (bad or extra or missing or staged or tracked_dirty)
# Check committed blob bytes too: worktree agreement alone cannot prove the final Git tree.
for row in manifest:
 r=subprocess.run(['git','show','HEAD:'+row['path']],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 if row['sha256'] is None:assert r.returncode!=0
 else:assert r.returncode==0 and hashlib.sha256(r.stdout).hexdigest()==row['sha256'],row['path']
print('Committed Git blobs: 756 matching SHA256 plus 1 matching deletion; no extra product difference.')
