#!/usr/bin/env python3
"""Unit tests for check_evidence_budget.py."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_evidence_budget as ceb  # noqa: E402


def _git(repo: Path, *args: str) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=repo,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {proc.stderr}")
    return proc.stdout.strip()


def _init_repo_with_baseline(tmp: str) -> tuple[Path, str]:
    repo = Path(tmp)
    _git(repo, "init", "-b", "main")
    _git(repo, "config", "user.name", "Test")
    _git(repo, "config", "user.email", "test@example.com")

    # Pre-baseline legacy files that intentionally exceed limits and use forbidden extensions.
    legacy = repo / "docs" / "operations" / "evidence" / "2026-11-14-queue1-runtime"
    legacy.mkdir(parents=True)
    (legacy / "old.ngsa").write_bytes(b"x" * (ceb.MAX_FILE_BYTES + 1024))
    (legacy / "old.py").write_text("print('legacy')\n", encoding="utf-8")
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "baseline")
    baseline = _git(repo, "rev-parse", "HEAD")
    return repo, baseline


class EvidenceBudgetTests(unittest.TestCase):
    def test_baseline_legacy_tracked_files_do_not_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            violations, stats = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertEqual(violations, [])
            self.assertEqual(stats["commits_checked"], 0)

    def test_small_valid_commit_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            ev = repo / "docs" / "operations" / "evidence" / "2026-11-18-ok" / "README.md"
            ev.parent.mkdir(parents=True)
            ev.write_text("# ok\n", encoding="utf-8")
            _git(repo, "add", str(ev.relative_to(repo)))
            _git(repo, "commit", "-m", "add small evidence")
            violations, stats = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertEqual(violations, [])
            self.assertEqual(stats["commits_checked"], 1)

    def test_commit_rejects_oversized_file_and_forbidden_extensions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            ev_dir = repo / "docs" / "operations" / "evidence" / "2026-11-18-bad"
            ev_dir.mkdir(parents=True)
            (ev_dir / "huge.txt").write_bytes(b"a" * (ceb.MAX_FILE_BYTES + 1))
            for ext in sorted(ceb.FORBIDDEN_EXTENSIONS):
                (ev_dir / f"bad{ext}").write_text("x", encoding="utf-8")
            _git(repo, "add", ".")
            _git(repo, "commit", "-m", "bad commit")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("200 KB" in v and "huge.txt" in v for v in violations))
            for ext in ceb.FORBIDDEN_EXTENSIONS:
                self.assertTrue(
                    any(f"bad{ext}" in v for v in violations),
                    f"expected violation for {ext}",
                )

    def test_commit_and_subdir_reject_over_2mb_batch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            ev_dir = repo / "docs" / "operations" / "evidence" / "2026-11-19-split"
            ev_dir.mkdir(parents=True)
            # Commit 1: 6 * 190 KB = 1,140 KB (under 2 MB)
            for i in range(6):
                (ev_dir / f"a{i}.txt").write_bytes(b"a" * (190 * 1024))
            _git(repo, "add", ".")
            _git(repo, "commit", "-m", "part 1")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertEqual(violations, [])

            # Commit 2: another 6 * 190 KB = 1,140 KB -> subdir total ~2.28 MB > 2 MB
            for i in range(6):
                (ev_dir / f"b{i}.txt").write_bytes(b"b" * (190 * 1024))
            _git(repo, "add", ".")
            _git(repo, "commit", "-m", "part 2")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("subdir 2026-11-19-split" in v for v in violations))

            # Commit 3: single commit exceeding 2 MB across two directories
            for d in ("2026-11-19-d1", "2026-11-19-d2"):
                sub = repo / "docs" / "operations" / "evidence" / d
                sub.mkdir(parents=True)
                for i in range(6):
                    (sub / f"c{i}.txt").write_bytes(b"c" * (190 * 1024))
            _git(repo, "add", ".")
            _git(repo, "commit", "-m", "single oversized commit")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("单批合计超过" in v and "commit " in v for v in violations))

    def test_staged_and_working_tree_checks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            ev_dir = repo / "docs" / "operations" / "evidence" / "2026-11-18-live"
            ev_dir.mkdir(parents=True)

            # Untracked forbidden / oversized file on disk is caught before staging
            untracked_bad = ev_dir / "capture.jsonl"
            untracked_bad.write_text("{}\n", encoding="utf-8")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("[working-tree]" in v and "capture.jsonl" in v for v in violations))

            # Staging it shifts it into [staged]
            _git(repo, "add", str(untracked_bad.relative_to(repo)))
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("[staged]" in v and "capture.jsonl" in v for v in violations))

            # Unstage & replace with a valid tracked file, then enlarge on disk without staging
            _git(repo, "reset", "HEAD", "--", str(untracked_bad.relative_to(repo)))
            untracked_bad.unlink()
            tracked = ev_dir / "notes.md"
            tracked.write_text("ok\n", encoding="utf-8")
            _git(repo, "add", str(tracked.relative_to(repo)))
            _git(repo, "commit", "-m", "track notes")
            tracked.write_bytes(b"x" * (ceb.MAX_FILE_BYTES + 10))
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=None)
            self.assertTrue(any("[working-tree]" in v and "notes.md" in v for v in violations))

    def test_legacy_untracked_snapshot_exempts_only_untouched_paths(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, baseline = _init_repo_with_baseline(tmp)
            scan_dir = repo / "docs" / "operations" / "evidence" / "2026-11-18-progress-scan"
            scan_dir.mkdir(parents=True)
            legacy_log = scan_dir / "workspace.log"
            payload = b"L" * (ceb.MAX_FILE_BYTES + 5000)
            legacy_log.write_bytes(payload)

            snap_path = repo / "legacy.json"
            rel = str(legacy_log.relative_to(repo))
            snap_path.write_text(
                json.dumps({rel: [len(payload), hashlib.sha256(payload).hexdigest()]}),
                encoding="utf-8",
            )

            # Untouched legacy untracked log passes
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=snap_path)
            self.assertEqual(violations, [])

            # New file in the same directory is caught
            extra = scan_dir / "new.ngsa"
            extra.write_bytes(b"new")
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=snap_path)
            self.assertTrue(any("new.ngsa" in v for v in violations))
            extra.unlink()

            # Modifying the legacy untracked log itself is caught
            legacy_log.write_bytes(b"M" * (ceb.MAX_FILE_BYTES + 5000))
            violations, _ = ceb.check_budget(repo, baseline=baseline, legacy_snapshot=snap_path)
            self.assertTrue(any("workspace.log" in v for v in violations))


if __name__ == "__main__":
    unittest.main()
