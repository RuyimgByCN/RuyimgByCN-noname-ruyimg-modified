"""Read-only inventory. Run from noname-next with --workspace ..; never imports extensions."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path


def inventory(workspace):
    result = {"schema": 1, "sources": [], "extensions": []}
    for repo, relative in (("noname", "apps/core/extension"), ("noname-liuli", "noname/extension")):
        checkout = workspace / "archive/upstream" / repo
        def git(*args):
            return subprocess.check_output(["git", "-C", str(checkout), "-c", "core.quotepath=false", *args], text=True).strip()
        result["sources"].append({"repo": repo, "head": git("rev-parse", "HEAD"), "status": git("status", "--short", "--untracked-files=all").splitlines()})
        for directory in sorted((checkout / relative).iterdir()):
            if not directory.is_dir():
                continue
            hashes = {"all": hashlib.sha256(), "code": hashlib.sha256()}
            count = size = code_count = 0
            for path in sorted(directory.rglob("*")):
                if not path.is_file():
                    continue
                data = path.read_bytes()
                name = str(path.relative_to(directory))
                record = name.encode() + b"\0" + hashlib.sha256(data).digest()
                hashes["all"].update(record)
                count += 1
                size += len(data)
                if path.suffix.lower() in {".js", ".ts", ".mjs", ".cjs", ".json", ".css", ".html", ".htm"}:
                    hashes["code"].update(record)
                    code_count += 1
            entry = directory / "extension.js"
            result["extensions"].append({"repo": repo, "name": directory.name, "path": str(directory.relative_to(workspace)), "files": count, "bytes": size, "code_files": code_count, "all_sha256": hashes["all"].hexdigest(), "code_sha256": hashes["code"].hexdigest(), "entry_sha256": hashlib.sha256(entry.read_bytes()).hexdigest() if entry.exists() else None})
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(inventory(args.workspace.resolve()), ensure_ascii=False, indent=2))
