#!/usr/bin/env python3
import sys
expected_url = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
expected_sha = '1d59f42b8b493e1fbac7a0e8b8c781ccef1ea61c'
expected_old = '99d9091bb28b55e28b620be16d1b7f343682783f'
expected_ref = 'refs/heads/native-convergence'
rows = [line.split() for line in sys.stdin.read().splitlines()]
allowed = sys.argv[1:] == [expected_url, expected_url] and rows == [[expected_sha, expected_sha, expected_ref, expected_old]]
if not allowed:
    print("Exact publication guard rejected unexpected URL, SHA, ref, or remote parent", file=sys.stderr)
sys.exit(0 if allowed else 1)
