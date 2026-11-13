#!/usr/bin/env python3
import sys
expected_url = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
expected_sha = '5c73d1d9158145e33f98f2018013914667d13964'
expected_old = '605201f3f49e10cea61c31d52a61bbb66c85599d'
expected_ref = 'refs/heads/native-convergence'
rows = [line.split() for line in sys.stdin.read().splitlines()]
allowed = sys.argv[1:] == [expected_url, expected_url] and rows == [[expected_sha, expected_sha, expected_ref, expected_old]]
if not allowed:
    print("Exact publication guard rejected unexpected URL, SHA, ref, or remote parent", file=sys.stderr)
sys.exit(0 if allowed else 1)
