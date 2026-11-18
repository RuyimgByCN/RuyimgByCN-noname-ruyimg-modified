#!/usr/bin/env python3
"""Exact publication guard (Decision 25) and CI log archival into `docs/operations/evidence-raw/ci/<sha>/`.

Enforces every Decision 25 safety invariant without writing scripts or large raw logs
into `docs/operations/evidence/`:
- `HEAD == sha` (40 lowercase hex chars)
- Tracked working tree is clean (`--untracked-files=no`)
- Local `.git/hooks/pre-push` and `.git/config` SHA-256 digests match expected values
- Target URL and branch strictly match the approved repository/branch
- Non-force fast-forward only (`git merge-base --is-ancestor old sha`, no `--force`/`-f`/`+`)
- Temporary `pre-push` guard runs a 10-case self-test before `git -c core.hooksPath=... push`
  and is removed unconditionally in `finally`
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import io
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import time
import zipfile
from dataclasses import dataclass

ROOT = pathlib.Path(__file__).resolve().parent.parent
APPROVED_REPO = "RuyimgByCN/RuyimgByCN-noname-ruyimg-modified"
APPROVED_URL = f"https://github.com/{APPROVED_REPO}.git"
APPROVED_BRANCH = "native-convergence"
APPROVED_REF = f"refs/heads/{APPROVED_BRANCH}"
EXPECTED_HOOK_SHA256 = "e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7"
EXPECTED_CONFIG_SHA256 = "f862ba299c2bbd23c501ebeb0830f53caabc990a4da7e8005570470ff914bc25"
RAW_CI_SUBDIR = pathlib.Path("docs/operations/evidence-raw/ci")
EVIDENCE_DIR = pathlib.Path("docs/operations/evidence")

RUST_RESULT_RE = re.compile(
    r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out"
)
RUSTC_VERSION_RE = re.compile(r"rustc (\d+\.\d+\.\d+) \([^\n)]+\)(?=\r?$)", re.M)
TIER_EXIT_RE = re.compile(r"=== attempt (\d+) exit=(\d+) ===")
ANSI_RE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")


@dataclass(frozen=True)
class PublicationPolicy:
    approved_url: str = APPROVED_URL
    approved_ref: str = APPROVED_REF
    expected_hook_sha256: str = EXPECTED_HOOK_SHA256
    expected_config_sha256: str = EXPECTED_CONFIG_SHA256


DEFAULT_POLICY = PublicationPolicy()


def is_hex_sha(value: str) -> bool:
    return len(value) == 40 and all(ch in "0123456789abcdef" for ch in value)


def _git_output(root: pathlib.Path, *args: str) -> str:
    proc = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {proc.stderr.strip()}")
    return proc.stdout.strip()


def _sha256_file(path: pathlib.Path) -> str | None:
    if not path.is_file():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _resolve_git_path(root: pathlib.Path, rel_git_path: str) -> pathlib.Path:
    raw = pathlib.Path(_git_output(root, "rev-parse", "--git-path", rel_git_path))
    return raw if raw.is_absolute() else (root / raw)


def _ensure_safe_output_dir(root: pathlib.Path, out_dir: pathlib.Path) -> None:
    resolved_out = out_dir.resolve()
    forbidden = (root / EVIDENCE_DIR).resolve()
    if resolved_out.is_relative_to(forbidden):
        raise RuntimeError(f"Refusing to write publication logs into {forbidden}: {resolved_out}")


def build_guard_script(url: str, sha: str, old: str, ref: str) -> str:
    return (
        "#!/usr/bin/env python3\n"
        "import sys\n"
        f"expected_url = {url!r}\n"
        f"expected_sha = {sha!r}\n"
        f"expected_old = {old!r}\n"
        f"expected_ref = {ref!r}\n"
        "rows = [line.split() for line in sys.stdin.read().splitlines()]\n"
        "allowed = sys.argv[1:] == [expected_url, expected_url] "
        "and rows == [[expected_sha, expected_sha, expected_ref, expected_old]]\n"
        "if not allowed:\n"
        '    print("Exact publication guard rejected unexpected URL, SHA, ref, or remote parent", file=sys.stderr)\n'
        "sys.exit(0 if allowed else 1)\n"
    )


def run_guard_selftests(
    guard_path: pathlib.Path, url: str, sha: str, old: str, ref: str
) -> list[dict[str, object]]:
    row = [sha, sha, ref, old]
    cases: list[tuple[str, list[str], list[list[str]], int]] = [
        ("exact", [url, url], [row], 0),
        ("wrong-remote-name", ["origin", url], [row], 1),
        ("wrong-url", [url, url + "-wrong"], [row], 1),
    ]
    for idx, label in enumerate(
        ["wrong-local-ref", "wrong-local-sha", "wrong-target", "wrong-remote-parent"]
    ):
        wrong = list(row)
        wrong[idx] = "refs/heads/main" if idx == 2 else ("0" * 40)
        cases.append((label, [url, url], [wrong], 1))
    cases.extend(
        [
            ("delete", [url, url], [["(delete)", "0" * 40, ref, old]], 1),
            ("multiple-updates", [url, url], [row, row], 1),
            ("empty", [url, url], [], 1),
        ]
    )

    results: list[dict[str, object]] = []
    for label, argv, rows, want in cases:
        payload = "".join(" ".join(parts) + "\n" for parts in rows)
        proc = subprocess.run(
            [sys.executable, "-B", str(guard_path), *argv],
            input=payload,
            capture_output=True,
            text=True,
        )
        results.append(
            {
                "case": label,
                "expected_exit": want,
                "actual_exit": proc.returncode,
                "stderr": proc.stderr,
            }
        )
        if proc.returncode != want:
            raise RuntimeError(
                f"Guard self-test {label!r} returned {proc.returncode}, expected {want}"
            )
    return results


def validate_push_argv(argv: list[str], policy: PublicationPolicy, sha: str) -> None:
    forbidden_flags = {"-f", "--force", "--force-with-lease", "--force-if-includes", "--delete", "-d", "--mirror", "--all", "--tags"}
    for token in argv:
        if token in forbidden_flags or token.startswith("--force="):
            raise RuntimeError(f"Forbidden git push flag under Decision 25: {token}")
    expected_refspec = f"{sha}:{policy.approved_ref}"
    if expected_refspec not in argv or expected_refspec.startswith("+"):
        raise RuntimeError(f"Missing non-force exact refspec {expected_refspec}")
    if policy.approved_url not in argv:
        raise RuntimeError("Approved remote URL missing from push argv")


def publish_exact(
    root: pathlib.Path,
    sha: str,
    *,
    expected_old: str | None = None,
    raw_ci_root: pathlib.Path | None = None,
    policy: PublicationPolicy = DEFAULT_POLICY,
    dry_run: bool = False,
) -> dict[str, object]:
    """Run Decision 25 exact non-force push and store raw logs under evidence-raw/ci/<sha>/."""
    if not is_hex_sha(sha):
        raise RuntimeError(f"Invalid SHA-1: {sha!r}")
    if policy.approved_url != APPROVED_URL and root.resolve() == ROOT.resolve():
        raise RuntimeError(f"Unapproved remote URL for main repo: {policy.approved_url}")
    if policy.approved_ref != APPROVED_REF and root.resolve() == ROOT.resolve():
        raise RuntimeError(f"Unapproved ref for main repo: {policy.approved_ref}")

    head = _git_output(root, "rev-parse", "HEAD")
    if head != sha:
        raise RuntimeError(f"HEAD ({head}) does not match target SHA ({sha})")

    tracked_dirty = _git_output(root, "status", "--porcelain=v1", "--untracked-files=no")
    if tracked_dirty:
        raise RuntimeError(f"Tracked working tree is dirty:\n{tracked_dirty}")

    hook_path = _resolve_git_path(root, "hooks/pre-push")
    config_path = _resolve_git_path(root, "config")
    hook_before = _sha256_file(hook_path)
    config_before = _sha256_file(config_path)
    if hook_before != policy.expected_hook_sha256:
        raise RuntimeError(
            f"pre-push hook sha256 mismatch: got {hook_before}, expected {policy.expected_hook_sha256}"
        )
    if config_before != policy.expected_config_sha256:
        raise RuntimeError(
            f"git config sha256 mismatch: got {config_before}, expected {policy.expected_config_sha256}"
        )

    ls_remote = _git_output(root, "ls-remote", policy.approved_url, policy.approved_ref).split()
    if len(ls_remote) != 2 or ls_remote[1] != policy.approved_ref or not is_hex_sha(ls_remote[0]):
        raise RuntimeError(f"Unexpected ls-remote result for {policy.approved_ref}: {ls_remote}")
    remote_old = ls_remote[0]
    if expected_old is not None and remote_old != expected_old:
        raise RuntimeError(f"Remote parent {remote_old} != expected_old {expected_old}")
    if remote_old == sha:
        raise RuntimeError(f"Remote {policy.approved_ref} is already at {sha}")

    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", remote_old, sha],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if ancestor.returncode != 0:
        raise RuntimeError(
            f"Non-fast-forward refused: remote {remote_old} is not an ancestor of {sha}"
        )

    base_out = raw_ci_root if raw_ci_root is not None else (root / RAW_CI_SUBDIR)
    out_dir = base_out / sha
    _ensure_safe_output_dir(root, out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    push_log = out_dir / "push.log"
    push_meta = out_dir / "push.json"

    tmp_dir = pathlib.Path(tempfile.mkdtemp(prefix="noname-decision25-guard-"))
    record: dict[str, object] = {
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "authorization": "Decision 25 and goal queue item 10",
        "approved_url": policy.approved_url,
        "approved_sha": sha,
        "expected_remote_old": remote_old,
        "target_ref": policy.approved_ref,
        "original_hook_sha256": hook_before,
        "persistent_config_sha256_before": config_before,
        "dry_run": dry_run,
    }
    try:
        guard_path = tmp_dir / "pre-push"
        guard_path.write_text(
            build_guard_script(policy.approved_url, sha, remote_old, policy.approved_ref),
            encoding="utf-8",
        )
        guard_path.chmod(0o755)
        record["selftests"] = run_guard_selftests(
            guard_path, policy.approved_url, sha, remote_old, policy.approved_ref
        )

        argv = [
            "git",
            "-c",
            f"core.hooksPath={tmp_dir}",
            "push",
            "--porcelain",
            policy.approved_url,
            f"{sha}:{policy.approved_ref}",
        ]
        validate_push_argv(argv, policy, sha)
        record["argv"] = argv
        if dry_run:
            push_log.write_text("DRY-RUN: " + " ".join(argv) + "\n", encoding="utf-8")
            record.update(
                exit_code=0,
                finished_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                remote_after=f"{remote_old}\t{policy.approved_ref}",
            )
        else:
            with push_log.open("wb") as stream:
                proc = subprocess.run(argv, cwd=root, stdout=stream, stderr=subprocess.STDOUT)
            remote_after = _git_output(root, "ls-remote", policy.approved_url, policy.approved_ref)
            record.update(
                exit_code=proc.returncode,
                finished_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                remote_after=remote_after,
            )
    finally:
        shutil.rmtree(tmp_dir)
        record.update(
            original_hook_unchanged=(_sha256_file(hook_path) == hook_before),
            persistent_git_configuration_unchanged=(_sha256_file(config_path) == config_before),
            temporary_guard_removed=(not tmp_dir.exists()),
        )
        push_meta.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if not record["original_hook_unchanged"] or not record["persistent_git_configuration_unchanged"]:
        raise RuntimeError("Local pre-push hook or git config changed unexpectedly")
    if not record["temporary_guard_removed"]:
        raise RuntimeError("Temporary guard directory was not removed")
    if not dry_run:
        if record["exit_code"] != 0:
            raise RuntimeError(f"git push failed (exit={record['exit_code']}); see {push_log}")
        if str(record["remote_after"]).split() != [sha, policy.approved_ref]:
            raise RuntimeError(f"Unexpected remote ref after push: {record['remote_after']}")
    return record


def _gh_api(endpoint: str, *, binary: bool = False):
    proc = subprocess.run(["gh", "api", "--allow-escape-sequences", endpoint], capture_output=True)
    if proc.returncode != 0:
        raise RuntimeError(
            f"gh api {endpoint} failed ({proc.returncode}): "
            f"{proc.stderr.decode('utf-8', errors='replace').strip()}"
        )
    if binary:
        return proc.stdout
    return json.loads(proc.stdout.decode("utf-8"))


def find_push_run(sha: str, *, repo: str = APPROVED_REPO, timeout_secs: int = 180, poll_secs: int = 5) -> dict:
    deadline = time.monotonic() + timeout_secs
    endpoint = f"repos/{repo}/actions/runs?branch={APPROVED_BRANCH}&event=push&per_page=20"
    while True:
        payload = _gh_api(endpoint)
        matches = [
            run
            for run in payload.get("workflow_runs", [])
            if run.get("head_sha") == sha and run.get("head_branch") == APPROVED_BRANCH
        ]
        if matches:
            matches.sort(key=lambda r: (int(r.get("run_attempt", 1)), int(r["id"])))
            return matches[0]
        if time.monotonic() >= deadline:
            raise RuntimeError(f"Timed out waiting for push workflow run for {sha}")
        time.sleep(poll_secs)


def collect_ci(
    root: pathlib.Path,
    sha: str,
    *,
    repo: str = APPROVED_REPO,
    raw_ci_root: pathlib.Path | None = None,
    find_timeout_secs: int = 180,
    watch_timeout_secs: int = 3600,
    poll_secs: int = 20,
) -> dict[str, object]:
    """Wait for exact-head push CI completion and archive raw JSON + zip logs under evidence-raw/."""
    if not is_hex_sha(sha):
        raise RuntimeError(f"Invalid SHA-1: {sha!r}")

    base_out = raw_ci_root if raw_ci_root is not None else (root / RAW_CI_SUBDIR)
    out_dir = base_out / sha
    _ensure_safe_output_dir(root, out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    run = find_push_run(sha, repo=repo, timeout_secs=find_timeout_secs)
    run_id = int(run["id"])
    (out_dir / "run-initial.json").write_text(
        json.dumps(run, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"[ci] observing run_id={run_id} sha={sha} attempt={run.get('run_attempt')} "
        f"status={run.get('status')}",
        flush=True,
    )

    run_endpoint = f"repos/{repo}/actions/runs/{run_id}"
    deadline = time.monotonic() + watch_timeout_secs
    while run.get("status") != "completed":
        if time.monotonic() >= deadline:
            raise RuntimeError(f"Timed out waiting for CI run {run_id} ({sha})")
        time.sleep(poll_secs)
        run = _gh_api(run_endpoint)
        print(
            f"[ci] run_id={run_id} status={run.get('status')} conclusion={run.get('conclusion')}",
            flush=True,
        )

    jobs = _gh_api(f"{run_endpoint}/jobs?per_page=100")
    artifacts = _gh_api(f"{run_endpoint}/artifacts?per_page=100")
    raw_zip: bytes = _gh_api(f"{run_endpoint}/logs", binary=True)

    (out_dir / "run-final.json").write_text(
        json.dumps(run, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (out_dir / "jobs-final.json").write_text(
        json.dumps(jobs, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (out_dir / "artifacts-final.json").write_text(
        json.dumps(artifacts, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    zip_path = out_dir / "logs-first.zip"
    zip_path.write_bytes(raw_zip)
    zip_sha256 = hashlib.sha256(raw_zip).hexdigest()

    logs_dir = out_dir / "logs-first"
    if logs_dir.exists():
        shutil.rmtree(logs_dir)
    logs_dir.mkdir()
    with zipfile.ZipFile(io.BytesIO(raw_zip)) as zf:
        for item in zf.infolist():
            if item.is_dir():
                continue
            target = (logs_dir / item.filename).resolve()
            if not target.is_relative_to(logs_dir.resolve()):
                raise RuntimeError(f"Zip slip path rejected: {item.filename}")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(zf.read(item))

    per_job: list[dict[str, object]] = []
    for job in jobs.get("jobs", []):
        name = job["name"]
        roots = [f for f in logs_dir.glob("*.txt") if f.stem == name or f.stem.endswith("_" + name)]
        files = roots if roots else sorted((logs_dir / name).glob("*.txt"))
        text = "\n".join(f.read_text(encoding="utf-8", errors="replace") for f in files)
        clean = ANSI_RE.sub("", text)
        rows = [tuple(map(int, m)) for m in RUST_RESULT_RE.findall(clean)]
        per_job.append(
            {
                "job_id": job["id"],
                "name": name,
                "status": job["status"],
                "conclusion": job["conclusion"],
                "rust_test_counts": (
                    dict(zip(["passed", "failed", "ignored", "measured", "filtered"], map(sum, zip(*rows))))
                    if rows
                    else None
                ),
                "setup_rustc_versions": sorted(set(RUSTC_VERSION_RE.findall(clean))),
                "tier_report_attempts": [
                    {"attempt": int(a), "exit": int(b)} for a, b in TIER_EXIT_RE.findall(clean)
                ],
            }
        )

    job_list = jobs.get("jobs", [])
    summary = {
        "at": datetime.datetime.now().astimezone().isoformat(),
        "head_sha": sha,
        "run_id": run_id,
        "run_attempt": run.get("run_attempt"),
        "status": run.get("status"),
        "conclusion": run.get("conclusion"),
        "archive_path": str(zip_path.relative_to(root)),
        "archive_sha256": zip_sha256,
        "archive_bytes": len(raw_zip),
        "job_total": len(job_list),
        "job_success_count": sum(j.get("conclusion") == "success" for j in job_list),
        "job_failure_count": sum(j.get("conclusion") == "failure" for j in job_list),
        "job_cancelled_count": sum(j.get("conclusion") == "cancelled" for j in job_list),
        "jobs": per_job,
        "agent_dispatched_reruns": 0,
    }
    (out_dir / "ci-summary.json").write_text(
        json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    p_push = sub.add_parser("publish", help="Exact non-force push under Decision 25")
    p_push.add_argument("--sha", default=None, help="Target 40-hex commit SHA (default: HEAD)")
    p_push.add_argument("--expected-old", default=None, help="Expected current remote SHA")
    p_push.add_argument("--dry-run", action="store_true", help="Run all checks and self-tests without pushing")
    p_push.add_argument("--wait-ci", action="store_true", help="Wait for exact-head CI and archive raw logs")

    p_ci = sub.add_parser("collect-ci", help="Wait for exact-head CI run and archive raw logs")
    p_ci.add_argument("--sha", default=None, help="Target 40-hex commit SHA (default: HEAD)")
    p_ci.add_argument("--find-timeout", type=int, default=180)
    p_ci.add_argument("--watch-timeout", type=int, default=3600)
    p_ci.add_argument("--poll-secs", type=int, default=20)

    args = parser.parse_args(argv)
    sha = args.sha or _git_output(ROOT, "rev-parse", "HEAD")

    if args.command == "publish":
        record = publish_exact(
            ROOT,
            sha,
            expected_old=args.expected_old,
            policy=DEFAULT_POLICY,
            dry_run=args.dry_run,
        )
        print(
            f"publish_decision25: sha={sha} exit={record['exit_code']} "
            f"remote_after={record['remote_after']}"
        )
        if args.wait_ci and not args.dry_run:
            summary = collect_ci(ROOT, sha)
            print(
                f"ci_run: run_id={summary['run_id']} conclusion={summary['conclusion']} "
                f"jobs={summary['job_success_count']}/{summary['job_total']} "
                f"zip_sha256={summary['archive_sha256']}"
            )
            return 0 if summary["conclusion"] == "success" else 1
        return 0

    summary = collect_ci(
        ROOT,
        sha,
        find_timeout_secs=args.find_timeout,
        watch_timeout_secs=args.watch_timeout,
        poll_secs=args.poll_secs,
    )
    print(
        f"ci_run: run_id={summary['run_id']} conclusion={summary['conclusion']} "
        f"jobs={summary['job_success_count']}/{summary['job_total']} "
        f"zip_sha256={summary['archive_sha256']}"
    )
    return 0 if summary["conclusion"] == "success" else 1


if __name__ == "__main__":
    raise SystemExit(main())
