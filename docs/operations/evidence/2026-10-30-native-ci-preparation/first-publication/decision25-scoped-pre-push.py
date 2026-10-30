#!/usr/bin/env python3
"""Decision25: permit only the explicitly approved first publication."""
from pathlib import Path
import subprocess
import sys

REPOSITORY = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
COMMIT = '9dbc65530feb695688d3160dc38b6cf64ad39ecd'
BRANCH = 'refs/heads/native-convergence'
WORKTREE = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')


def reject(reason):
    print('decision25 scoped push refused: ' + reason, file=sys.stderr)
    raise SystemExit(1)


if len(sys.argv) != 3 or sys.argv[1:] != [REPOSITORY, REPOSITORY]:
    reject('repository does not match explicit approval')
try:
    top = subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip()
except (OSError, subprocess.CalledProcessError) as error:
    reject('cannot verify worktree: ' + str(error))
if Path(top).resolve() != WORKTREE.resolve():
    reject('worktree does not match explicit approval')
rows = [line.split() for line in sys.stdin if line.strip()]
if len(rows) != 1 or len(rows[0]) != 4:
    reject('expected exactly one ref update')
local_ref, local_oid, remote_ref, remote_oid = rows[0]
if local_ref == '(delete)' or local_oid != COMMIT or remote_ref != BRANCH:
    reject('commit or branch does not match explicit approval')
if remote_oid != '0' * 40:
    reject('first publication requires the new branch to be absent')
print('decision25 scoped check accepted exact repository, new branch and accepted commit', file=sys.stderr)
