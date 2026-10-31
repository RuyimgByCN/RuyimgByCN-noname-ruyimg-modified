import subprocess,json
from pathlib import Path
out=[]
for players,seed in [(4,42),(2,1)]:
 native=subprocess.check_output(["cargo","run","--offline","-q","-p","engine-cli","--","hash",str(players),str(seed)],text=True).strip()
 wasm=subprocess.check_output(["node","scripts/wasm-hash.mjs",str(players),str(seed)],text=True).strip()
 assert native==wasm,(native,wasm)
 out.append({"players":players,"seed":seed,"native":native,"wasm":wasm})
manifest=subprocess.check_output(["cargo","run","--offline","-q","-p","engine-cli","--","manifest","4","42"],text=True)
assert "chain=ok" in manifest
skill=subprocess.check_output(["cargo","run","--offline","-q","-p","engine-cli","--","skill-vm","content/standard/jianxiong.skill"],text=True)
assert "ok=true" in skill
Path("/tmp/i02-main-typed-wire-functional-33d21ba5/cross-target-current-values.json").write_text(json.dumps({"hashes":out,"manifest":manifest,"skillVm":skill},indent=2))
print(json.dumps(out,indent=2));print("ALL CURRENT-SOURCE CROSS-TARGET CHECKS PASSED")
