#!/usr/bin/env python3
"""Count cargo test result lines by the package owning each test target."""

import collections
import json
import re
import subprocess
import sys
from pathlib import Path

metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--offline", "--format-version", "1", "--no-deps"]
))
targets = collections.defaultdict(set)
for package in metadata["packages"]:
    for target in package["targets"]:
        targets[target["name"].replace("-", "_")].add(package["name"])

totals = collections.defaultdict(lambda: [0, 0, 0, 0])
current = None
for line in Path(sys.argv[1]).read_text().splitlines():
    if line.strip().startswith("Running "):
        match = re.search(r"/([^/\s]+)-[0-9a-f]+\)", line)
        options = targets.get(match.group(1), set()) if match else set()
        current = next(iter(options)) if len(options) == 1 else None
    elif line.strip().startswith("Doc-tests "):
        current = line.strip().split()[1].replace("_", "-")
    elif line.startswith("test result: "):
        match = re.search(r"(\d+) passed; (\d+) failed; (\d+) ignored;", line)
        if match is None or current is None:
            raise SystemExit(f"unmapped result: {line}")
        counts = totals[current]
        for i in range(3):
            counts[i] += int(match.group(i + 1))
        counts[3] += 1

print("package\tpassed\tfailed\tignored\ttest targets")
for package, counts in sorted(totals.items()):
    print(package, *counts, sep="\t")
