from pathlib import Path
import subprocess,json
root=Path(__file__).resolve().parents[1]/"worktree"
e=root.parent/"evidence-review-fix"
target=str(root.parent/"target")
serial=["python3","/tmp/noname-run-serialized.py","env","CARGO_TARGET_DIR="+target]
steps=[
("final-full-all5",serial+["cargo","test","--offline","-p","engine-native-kernel","-p","engine-native-content","-p","engine-native","-p","game-server","-p","engine-native-wasm","--no-fail-fast"]),
("final-strict-all3",serial+["RUSTFLAGS=-D warnings","cargo","check","--offline","-p","engine-native-kernel","-p","engine-native-content","-p","engine-native","--all-targets"]),
("final-fresh-wasm32-build",serial+["wasm-pack","build","crates/engine-native-wasm","--target","web","--release","--no-opt","--out-dir","../../apps/web-client/src/worker/native-pkg"]),
("final-wasm32-machine-windows",["node",str(e/"wasm-machine-windows.mjs"),str(root),str(e)]),
("final-fresh-ordinary-fixtures",serial+["cargo","run","--offline","-p","game-server","--example","emit_projection_fixture","--",str(e/"ordinary-fixtures.json")]),
("final-wasm32-ordinary-parity",["node","--import","./scripts/ts-register.mjs","scripts/native-server-projection-parity.mjs",str(e/"ordinary-fixtures.json")]),
]
results=[]
for name,command in steps:
 print("START "+name,flush=True)
 cwd=root/"apps/web-client" if name=="final-wasm32-ordinary-parity" else root
 with (e/(name+".log")).open("w") as log:
  result=subprocess.run(command,cwd=cwd,stdout=log,stderr=subprocess.STDOUT)
 results.append({"name":name,"command":command,"cwd":str(cwd),"exit":result.returncode})
 (e/"final-combined-commands.json").write_text(json.dumps(results,indent=2)+"\n")
 print("FINISH "+name+" exit="+str(result.returncode),flush=True)
