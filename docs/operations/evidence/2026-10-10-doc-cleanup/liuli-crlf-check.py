"""只读核对：Liuli 本地修改 50→7 中消失的文件是否只是换行差异。

用法：python3 liuli-crlf-check.py --workspace <sanguosha 工作区根>
做法：以 9-23 基线 inventory.json 记录的修改清单为准，把今天的文件中
“9-23 列为已修改的文件”转成 CRLF，按 inventory.py 的同一算法重算每个
内容变化来源的 all_sha256 与字节数，与 9-23 基线比较。不写任何文件。
"""
import argparse, hashlib, json, re
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--workspace", type=Path, required=True)
ws = ap.parse_args().workspace.resolve()
nn = ws / "noname-next"
base = json.load(open(nn / "docs/migration/evidence/2026-09-23-extension-scope/inventory.json"))
now = json.load(open(nn / "docs/operations/evidence/2026-10-10-progress-check/extension-inventory.json"))

def paths(status):
    out = set()
    for s in status:
        p = re.sub(r"^[ MADRCU?!]{1,2}\s+", "", s).strip()
        out.add(p[1:-1] if p.startswith('"') and p.endswith('"') else p)
    return out

src = lambda d: [r for r in d["sources"] if r["repo"] == "noname-liuli"][0]["status"]
was, still = paths(src(base)), paths(src(now))
gone = was - still
crlf = lambda b: b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
bm = {(e["repo"], e["name"]): e for e in base["extensions"]}
nm = {(e["repo"], e["name"]): e for e in now["extensions"]}
root = ws / "archive/upstream/noname-liuli"
print(f"9-23 修改 {len(was)} 个文件；10-10 仍修改 {len(still)} 个；消失 {len(gone)} 个")
for key in sorted(k for k in bm if bm[k]["all_sha256"] != nm[k]["all_sha256"]):
    d = root / "noname/extension" / key[1]
    for label, conv in (("仅消失文件转 CRLF", gone), ("9-23 全部修改文件转 CRLF", was)):
        h, size = hashlib.sha256(), 0
        for p in sorted(d.rglob("*")):
            if not p.is_file():
                continue
            data = p.read_bytes()
            if str(p.relative_to(root)) in conv:
                data = crlf(data)
            h.update(str(p.relative_to(d)).encode() + b"\0" + hashlib.sha256(data).digest())
            size += len(data)
        ok = h.hexdigest() == bm[key]["all_sha256"] and size == bm[key]["bytes"]
        n_gone = sum(g.startswith(f"noname/extension/{key[1]}/") for g in gone)
        verdict = "与 9-23 一致" if ok else f"不一致（9-23 为 {bm[key]['bytes']} 字节，复原为 {size} 字节）"
        print(f"{key[1]}\t消失 {n_gone}\t{label}：{verdict}")
        if ok:
            break
