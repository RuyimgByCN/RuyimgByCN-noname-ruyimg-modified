import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path("/Users/ruyimg/Documents/Claude/sanguosha/noname-next")
ev=root/"docs/operations/evidence/2026-11-13-queue0-gates"
manifest=json.loads((ev/"validated-final-inputs-v3.json").read_text())
steps=[
("06-wasm-check",None,["cargo","check","--target","wasm32-unknown-unknown","-p","engine-native-wasm"]),
("07-workspace-complete",None,["cargo","test","--offline","--workspace","--no-fail-fast"]),
("08-conformance",None,["cargo","run","-p","conformance"]),
("12-build-native","apps/web-client",["pnpm","build:native"]),
("13-smoke-native","apps/web-client",["pnpm","smoke:native"]),
("14-native-wasm-cross-target",None,["python3","-B","scripts/check-native-wasm-cross-target.py"]),
("15-cross-target-hash",None,["bash","scripts/cross-target-hash.sh"]),
("16-desktop-tests",None,["cargo","test","-p","desktop-client"]),
("17-frontend-build","apps/web-client",["pnpm","build"]),
]
def unchanged():
    for path,digest in manifest["files"].items():
        assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,path
for name,cwd,argv in steps:
    unchanged()
    cmd=[sys.executable,"/private/tmp/noname-queue0-check.py"]
    if cwd: cmd += ["--cwd",cwd]
    cmd += [name,"--",*argv]
    result=subprocess.run(cmd,cwd=root)
    unchanged()
    if result.returncode: sys.exit(result.returncode)
print("PRODUCT GATES COMPLETED; 25 frozen inputs unchanged",flush=True)
