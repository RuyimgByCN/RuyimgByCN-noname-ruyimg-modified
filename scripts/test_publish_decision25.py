#!/usr/bin/env python3
"""Unit tests for publish_decision25.py."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import publish_decision25 as pd  # noqa: E402


def _git(cwd: Path, *args: str) -> str:
    proc = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {proc.stderr}")
    return proc.stdout.strip()


def _setup_local_and_bare_remote(tmp: str) -> tuple[Path, Path, str, str, pd.PublicationPolicy]:
    base = Path(tmp)
    remote = base / "remote.git"
    _git(base, "init", "--bare", str(remote))

    work = base / "work"
    _git(base, "init", "-b", "main", str(work))
    _git(work, "config", "user.name", "Test")
    _git(work, "config", "user.email", "test@example.com")

    (work / "README.md").write_text("init\n", encoding="utf-8")
    _git(work, "add", "README.md")
    _git(work, "commit", "-m", "initial")
    old_sha = _git(work, "rev-parse", "HEAD")
    _git(work, "push", str(remote), f"{old_sha}:{pd.APPROVED_REF}")

    (work / "README.md").write_text("next\n", encoding="utf-8")
    _git(work, "add", "README.md")
    _git(work, "commit", "-m", "second")
    new_sha = _git(work, "rev-parse", "HEAD")

    hook = work / ".git" / "hooks" / "pre-push"
    hook.write_text("#!/bin/sh\nexit 1\n", encoding="utf-8")
    hook.chmod(0o755)
    config = work / ".git" / "config"

    policy = pd.PublicationPolicy(
        approved_url=str(remote),
        approved_ref=pd.APPROVED_REF,
        expected_hook_sha256=hashlib.sha256(hook.read_bytes()).hexdigest(),
        expected_config_sha256=hashlib.sha256(config.read_bytes()).hexdigest(),
    )
    return work, remote, old_sha, new_sha, policy


class PublishDecision25Tests(unittest.TestCase):
    def test_default_policy_constants(self) -> None:
        self.assertEqual(
            pd.APPROVED_URL,
            "https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git",
        )
        self.assertEqual(pd.APPROVED_REF, "refs/heads/native-convergence")

    def test_validate_push_argv_rejects_force_and_plus(self) -> None:
        sha = "a" * 40
        for bad in (
            ["git", "push", "--force", pd.APPROVED_URL, f"{sha}:{pd.APPROVED_REF}"],
            ["git", "push", "-f", pd.APPROVED_URL, f"{sha}:{pd.APPROVED_REF}"],
            ["git", "push", pd.APPROVED_URL, f"+{sha}:{pd.APPROVED_REF}"],
            ["git", "push", pd.APPROVED_URL, f"{sha}:refs/heads/main"],
        ):
            with self.assertRaises(RuntimeError):
                pd.validate_push_argv(bad, pd.DEFAULT_POLICY, sha)

    def test_publish_exact_end_to_end_and_rejection_guards(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            work, remote, old_sha, new_sha, policy = _setup_local_and_bare_remote(tmp)

            # Rejects output inside docs/operations/evidence/
            with self.assertRaises(RuntimeError):
                pd.publish_exact(
                    work,
                    new_sha,
                    expected_old=old_sha,
                    raw_ci_root=work / "docs" / "operations" / "evidence",
                    policy=policy,
                    dry_run=True,
                )

            # Rejects wrong HEAD
            with self.assertRaises(RuntimeError):
                pd.publish_exact(work, old_sha, policy=policy, dry_run=True)

            # Rejects dirty tracked file
            (work / "README.md").write_text("dirty\n", encoding="utf-8")
            with self.assertRaises(RuntimeError):
                pd.publish_exact(work, new_sha, policy=policy, dry_run=True)
            _git(work, "checkout", "--", "README.md")

            # Rejects tampered hook hash
            bad_policy = pd.PublicationPolicy(
                approved_url=policy.approved_url,
                approved_ref=policy.approved_ref,
                expected_hook_sha256="0" * 64,
                expected_config_sha256=policy.expected_config_sha256,
            )
            with self.assertRaises(RuntimeError):
                pd.publish_exact(work, new_sha, policy=bad_policy, dry_run=True)

            # Untracked file does not block exact fast-forward push
            (work / "untracked.log").write_text("keep\n", encoding="utf-8")
            record = pd.publish_exact(
                work,
                new_sha,
                expected_old=old_sha,
                policy=policy,
            )
            self.assertEqual(record["exit_code"], 0)
            self.assertTrue(record["original_hook_unchanged"])
            self.assertTrue(record["persistent_git_configuration_unchanged"])
            self.assertTrue(record["temporary_guard_removed"])
            self.assertEqual(len(record["selftests"]), 10)
            self.assertEqual(_git(work, "ls-remote", str(remote), pd.APPROVED_REF).split(), [new_sha, pd.APPROVED_REF])
            raw_dir = work / "docs" / "operations" / "evidence-raw" / "ci" / new_sha
            self.assertTrue((raw_dir / "push.log").is_file())
            self.assertTrue((raw_dir / "push.json").is_file())
            self.assertFalse((work / "docs" / "operations" / "evidence").exists())

            # Non-fast-forward (diverged remote) is rejected
            _git(work, "reset", "--hard", old_sha)
            (work / "README.md").write_text("diverged\n", encoding="utf-8")
            _git(work, "add", "README.md")
            _git(work, "commit", "-m", "diverged")
            div_sha = _git(work, "rev-parse", "HEAD")
            with self.assertRaises(RuntimeError):
                pd.publish_exact(work, div_sha, policy=policy)

    def test_collect_ci_archives_logs_into_evidence_raw(self) -> None:
        sha = "b" * 40
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w") as zf:
            zf.writestr(
                "0_gates-ubuntu.txt",
                "rustc 1.100.0 (abc 2026-01-01)\n"
                "test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out\n",
            )
        zip_bytes = buf.getvalue()

        def fake_api(endpoint: str, *, binary: bool = False):
            if endpoint.endswith("/logs"):
                assert binary
                return zip_bytes
            if "actions/runs?" in endpoint:
                return {
                    "workflow_runs": [
                        {
                            "id": 999,
                            "head_sha": sha,
                            "head_branch": pd.APPROVED_BRANCH,
                            "run_attempt": 1,
                            "status": "completed",
                            "conclusion": "success",
                        }
                    ]
                }
            if endpoint.endswith("/jobs?per_page=100"):
                return {
                    "jobs": [
                        {
                            "id": 101,
                            "name": "gates-ubuntu",
                            "status": "completed",
                            "conclusion": "success",
                        }
                    ]
                }
            if endpoint.endswith("/artifacts?per_page=100"):
                return {"artifacts": []}
            raise AssertionError(f"unexpected endpoint: {endpoint}")

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with (
                mock.patch.object(pd, "_gh_api", side_effect=fake_api),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                summary = pd.collect_ci(root, sha)

            out_dir = root / "docs" / "operations" / "evidence-raw" / "ci" / sha
            self.assertEqual(summary["run_id"], 999)
            self.assertEqual(summary["job_success_count"], 1)
            self.assertEqual(summary["archive_sha256"], hashlib.sha256(zip_bytes).hexdigest())
            self.assertEqual(summary["jobs"][0]["rust_test_counts"]["passed"], 12)
            self.assertEqual(summary["jobs"][0]["setup_rustc_versions"], ["1.100.0"])
            self.assertTrue((out_dir / "logs-first.zip").is_file())
            self.assertTrue((out_dir / "ci-summary.json").is_file())
            self.assertTrue((out_dir / "logs-first" / "0_gates-ubuntu.txt").is_file())


if __name__ == "__main__":
    unittest.main()
