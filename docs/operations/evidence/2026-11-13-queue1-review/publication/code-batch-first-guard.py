#!/usr/bin/env python3
import sys
expected_url = 'https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
expected_sha = '1fa8ac6cc3a70446436d97ff49aa19ea4626e22d'
expected_old = 'bd58ef06d59940baed8d7f93e61ce571b152fb81'
expected_ref = 'refs/heads/native-convergence'
rows = [line.split() for line in sys.stdin.read().splitlines()]
allowed = sys.argv[1:] == [expected_url, expected_url] and rows == [[expected_sha, expected_sha, expected_ref, expected_old]]
if not allowed:
    print("Exact publication guard rejected unexpected URL, SHA, ref, or remote parent", file=sys.stderr)
sys.exit(0 if allowed else 1)
