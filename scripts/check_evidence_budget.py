#!/usr/bin/env python3
"""Enforce size and file-type limits for evidence under `docs/operations/evidence/`.

Checks everything introduced after `BASELINE_COMMIT`:
1. Every commit in `<baseline>..HEAD` (`--diff-filter=AM -- docs/operations/evidence`).
2. Current git staging area (`git diff --cached --diff-filter=AM -- docs/operations/evidence`).
3. Uncommitted working-tree changes and newly created untracked files on disk,
   excluding untouched pre-existing legacy untracked paths captured in
   `scripts/evidence_legacy_untracked.json`.

Hard limits:
- Single file <= 200 KB (204,800 bytes).
- Single batch (per commit / staging area / uncommitted disk delta / per post-baseline
  evidence subdirectory) <= 2 MB (2,097,152 bytes).
- Forbidden extensions under `docs/operations/evidence/`:
  .rs, .py, .js, .mjs, .cjs, .ts, .tsx, .vue, .sh, .wasm, .ngsa, .archive,
  .zip, .tar, .gz, .jsonl.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE_COMMIT = "21266d00c0583b692d3c7c7090678c487f211843"
EVIDENCE_DIR = "docs/operations/evidence"
EVIDENCE_PREFIX = f"{EVIDENCE_DIR}/"
DEFAULT_LEGACY_SNAPSHOT = ROOT / "scripts" / "evidence_legacy_untracked.json"

MAX_FILE_BYTES = 200 * 1024  # 204,800
MAX_BATCH_BYTES = 2 * 1024 * 1024  # 2,097,152

FORBIDDEN_EXTENSIONS = frozenset(
    {
        ".rs",
        ".py",
        ".js",
        ".mjs",
        ".cjs",
        ".ts",
        ".tsx",
        ".vue",
        ".sh",
        ".wasm",
        ".ngsa",
        ".archive",
        ".zip",
        ".tar",
        ".gz",
        ".jsonl",
    }
)


def _run_git(root: Path, args: list[str], *, stdin_text: bytes | None = None) -> bytes:
    proc = subprocess.run(
        ["git", *args],
        cwd=root,
        input=stdin_text,
        capture_output=True,
    )
    if proc.returncode != 0:
        err = proc.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git {' '.join(args)} failed ({proc.returncode}): {err}")
    return proc.stdout


def _split_nul(raw: bytes) -> list[str]:
    return [chunk.decode("utf-8", errors="surrogateescape") for chunk in raw.split(b"\0") if chunk]


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_legacy_snapshot(snapshot_path: Path | None) -> dict[str, tuple[int, str]]:
    """Load `{rel_path: [size, sha256]}` of pre-baseline legacy untracked files."""
    if snapshot_path is None or not snapshot_path.is_file():
        return {}
    raw = json.loads(snapshot_path.read_text(encoding="utf-8"))
    return {path: (int(meta[0]), str(meta[1])) for path, meta in raw.items()}


def _ensure_baseline(root: Path, baseline: str) -> None:
    _run_git(root, ["rev-parse", "--verify", f"{baseline}^{{commit}}"])
    proc = subprocess.run(
        ["git", "merge-base", "--is-ancestor", baseline, "HEAD"],
        cwd=root,
        capture_output=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"baseline commit {baseline} is not an ancestor of HEAD")


def _tree_blob_sizes(root: Path, rev: str) -> dict[str, int]:
    raw = _run_git(root, ["ls-tree", "-r", "-l", "-z", rev, "--", EVIDENCE_DIR])
    sizes: dict[str, int] = {}
    for entry in _split_nul(raw):
        meta, path = entry.split("\t", 1)
        parts = meta.split()
        if len(parts) >= 4 and parts[1] == "blob" and parts[3] != "-":
            sizes[path] = int(parts[3])
    return sizes


def commit_diff_files(root: Path, sha: str, diff_filter: str) -> list[str]:
    raw = _run_git(
        root,
        [
            "diff-tree",
            "-m",
            "--first-parent",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-z",
            f"--diff-filter={diff_filter}",
            sha,
            "--",
            EVIDENCE_DIR,
        ],
    )
    return _split_nul(raw)


def commit_am_sizes(root: Path, sha: str) -> dict[str, int]:
    am_paths = commit_diff_files(root, sha, "AM")
    if not am_paths:
        return {}
    tree_sizes = _tree_blob_sizes(root, sha)
    return {path: tree_sizes[path] for path in am_paths if path in tree_sizes}


def staged_diff_files(root: Path, diff_filter: str) -> list[str]:
    raw = _run_git(
        root,
        ["diff", "--cached", "--name-only", "-z", f"--diff-filter={diff_filter}", "--", EVIDENCE_DIR],
    )
    return _split_nul(raw)


def staged_am_sizes(root: Path) -> dict[str, int]:
    am_paths = staged_diff_files(root, "AM")
    if not am_paths:
        return {}
    raw_ls = _run_git(root, ["ls-files", "-s", "-z", "--", EVIDENCE_DIR])
    blob_by_path: dict[str, str] = {}
    for entry in _split_nul(raw_ls):
        meta, path = entry.split("\t", 1)
        fields = meta.split()
        if len(fields) >= 2 and (fields[2] == "0" or path not in blob_by_path):
            blob_by_path[path] = fields[1]
    selected = [path for path in am_paths if path in blob_by_path]
    if not selected:
        return {}
    stdin_payload = "".join(f"{blob_by_path[p]}\n" for p in selected).encode("utf-8")
    sizes_out = _run_git(root, ["cat-file", "--batch-check=%(objectsize)"], stdin_text=stdin_payload)
    lines = sizes_out.decode("utf-8").splitlines()
    if len(lines) != len(selected):
        raise RuntimeError("unexpected git cat-file --batch-check output length")
    return {path: int(size_str) for path, size_str in zip(selected, lines)}


def _disk_size(path: Path) -> int:
    return path.lstat().st_size if path.is_symlink() else path.stat().st_size


def _is_untouched_legacy(path: Path, rel: str, legacy: dict[str, tuple[int, str]]) -> bool:
    expected = legacy.get(rel)
    if expected is None or path.is_symlink() or not path.is_file():
        return False
    exp_size, exp_sha = expected
    return path.stat().st_size == exp_size and _sha256_file(path) == exp_sha


def uncommitted_disk_sizes(root: Path, legacy: dict[str, tuple[int, str]]) -> dict[str, int]:
    """Return `{rel_path: bytes}` for unstaged tracked AM edits and non-legacy untracked files."""
    result: dict[str, int] = {}
    unstaged_am = _split_nul(
        _run_git(root, ["diff", "--name-only", "-z", "--diff-filter=AM", "--", EVIDENCE_DIR])
    )
    for rel in unstaged_am:
        disk = root / rel
        if disk.exists() or disk.is_symlink():
            result[rel] = _disk_size(disk)

    untracked = _split_nul(
        _run_git(root, ["ls-files", "--others", "--exclude-standard", "-z", "--", EVIDENCE_DIR])
    )
    for rel in untracked:
        disk = root / rel
        if not (disk.exists() or disk.is_symlink()):
            continue
        if _is_untouched_legacy(disk, rel, legacy):
            continue
        result[rel] = _disk_size(disk)
    return result


def _evidence_subdir(rel: str) -> str | None:
    if not rel.startswith(EVIDENCE_PREFIX):
        return None
    rest = rel[len(EVIDENCE_PREFIX) :]
    if "/" not in rest:
        return None
    return rest.split("/", 1)[0]


def _validate_entries(scope: str, entries: dict[str, int], violations: list[str]) -> None:
    total = 0
    for rel, size in sorted(entries.items()):
        total += size
        suffix = Path(rel).suffix.lower()
        if suffix in FORBIDDEN_EXTENSIONS:
            violations.append(f"[{scope}] 禁止的证据文件类型 ({suffix}): {rel}")
        if size > MAX_FILE_BYTES:
            violations.append(
                f"[{scope}] 单文件超过 {MAX_FILE_BYTES} B (200 KB): {rel} ({size} B)"
            )
    if total > MAX_BATCH_BYTES:
        violations.append(
            f"[{scope}] 单批合计超过 {MAX_BATCH_BYTES} B (2 MB): {total} B ({len(entries)} 个文件)"
        )


def check_budget(
    root: Path,
    baseline: str = BASELINE_COMMIT,
    legacy_snapshot: Path | None = DEFAULT_LEGACY_SNAPSHOT,
) -> tuple[list[str], dict[str, int]]:
    """Run all evidence budget and file-type checks; return `(violations, stats)`."""
    _ensure_baseline(root, baseline)
    legacy = load_legacy_snapshot(legacy_snapshot)
    violations: list[str] = []

    commits = _run_git(root, ["rev-list", "--reverse", f"{baseline}..HEAD"]).decode("utf-8").splitlines()
    post_baseline_files: dict[str, int] = {}

    for sha in commits:
        am = commit_am_sizes(root, sha)
        if am:
            _validate_entries(f"commit {sha[:9]}", am, violations)
        for deleted in commit_diff_files(root, sha, "D"):
            post_baseline_files.pop(deleted, None)
        post_baseline_files.update(am)

    staged = staged_am_sizes(root)
    if staged:
        _validate_entries("staged", staged, violations)
    for deleted in staged_diff_files(root, "D"):
        post_baseline_files.pop(deleted, None)
    post_baseline_files.update(staged)

    disk = uncommitted_disk_sizes(root, legacy)
    if disk:
        _validate_entries("working-tree", disk, violations)
    post_baseline_files.update(disk)

    by_subdir: dict[str, dict[str, int]] = defaultdict(dict)
    for rel, size in post_baseline_files.items():
        subdir = _evidence_subdir(rel)
        if subdir is not None:
            by_subdir[subdir][rel] = size

    for subdir, entries in sorted(by_subdir.items()):
        subtotal = sum(entries.values())
        if subtotal > MAX_BATCH_BYTES:
            violations.append(
                f"[subdir {subdir}] 新证据子目录合计超过 {MAX_BATCH_BYTES} B (2 MB): "
                f"{subtotal} B ({len(entries)} 个文件)"
            )

    stats = {
        "commits_checked": len(commits),
        "staged_files": len(staged),
        "working_tree_files": len(disk),
        "post_baseline_subdirs": len(by_subdir),
    }
    return violations, stats


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root path")
    parser.add_argument("--baseline", default=BASELINE_COMMIT, help="Baseline commit SHA")
    parser.add_argument(
        "--legacy-snapshot",
        default=None,
        help="Path to legacy untracked JSON snapshot (or 'none' to disable)",
    )
    args = parser.parse_args(argv)

    if args.legacy_snapshot == "none":
        snapshot: Path | None = None
    elif args.legacy_snapshot is not None:
        snapshot = Path(args.legacy_snapshot)
    elif args.baseline == BASELINE_COMMIT and args.root.resolve() == ROOT.resolve():
        snapshot = DEFAULT_LEGACY_SNAPSHOT
    else:
        snapshot = None

    try:
        violations, stats = check_budget(args.root, baseline=args.baseline, legacy_snapshot=snapshot)
    except RuntimeError as error:
        print(f"证据门禁执行失败: {error}", file=sys.stderr)
        return 1

    if violations:
        for item in violations:
            print(item, file=sys.stderr)
        print(f"证据体量与类型门禁失败：共 {len(violations)} 项违规", file=sys.stderr)
        return 1

    print(
        "证据体量与类型门禁通过："
        f"检查提交 {stats['commits_checked']} 个，"
        f"暂存证据 {stats['staged_files']} 个，"
        f"未提交新增/修改证据 {stats['working_tree_files']} 个，"
        f"新证据子目录 {stats['post_baseline_subdirs']} 个"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
