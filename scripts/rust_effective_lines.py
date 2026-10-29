"""Effective Rust line count: code lines left after removing comments, blank lines and test code.

Test code is any item behind `#[cfg(test)]`; files under a `tests/` directory are skipped by the caller.
String and char literals are kept intact, so `//` or `/*` inside them is not taken for a comment.
"""

import re

CFG_TEST = re.compile(r"^\s*#\[cfg\(test\)\]\s*$")
RAW_STRING = re.compile(r'r(#*)"')
CHAR = re.compile(r"'(?:\\u\{[0-9a-fA-F]+\}|\\.|[^'\\])'")


def strip_comments(text):
    out = []
    i, n, depth = 0, len(text), 0
    while i < n:
        if depth:
            if text.startswith("/*", i):
                depth, i = depth + 1, i + 2
            elif text.startswith("*/", i):
                depth, i = depth - 1, i + 2
            else:
                if text[i] == "\n":
                    out.append("\n")
                i += 1
            continue
        c = text[i]
        if text.startswith("//", i):
            end = text.find("\n", i)
            i = n if end < 0 else end
        elif text.startswith("/*", i):
            depth, i = 1, i + 2
        elif c == "r" and RAW_STRING.match(text, i) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            m = RAW_STRING.match(text, i)
            end = text.find('"' + m.group(1), m.end())
            end = n if end < 0 else end + 1 + len(m.group(1))
            out.append(text[i:end])
            i = end
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1])
            i = j + 1
        elif c == "'" and CHAR.match(text, i):
            m = CHAR.match(text, i)
            out.append(m.group(0))
            i = m.end()
        else:
            out.append(c)
            i += 1
    return "".join(out)


def drop_cfg_test(text):
    lines = text.split("\n")
    kept = []
    i = 0
    while i < len(lines):
        if not CFG_TEST.match(lines[i]):
            kept.append(lines[i])
            i += 1
            continue
        # Skip the attribute and the item it gates: a braced block, or a `mod x;` style declaration.
        j, depth, opened = i + 1, 0, False
        while j < len(lines):
            line = lines[j]
            depth += line.count("{") - line.count("}")
            opened = opened or "{" in line
            if (opened and depth <= 0) or (not opened and line.rstrip().endswith(";")):
                break
            j += 1
        i = j + 1
    return "\n".join(kept)


def effective_lines(text):
    return sum(1 for line in drop_cfg_test(strip_comments(text)).split("\n") if line.strip())
