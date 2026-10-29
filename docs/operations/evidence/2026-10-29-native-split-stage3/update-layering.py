from pathlib import Path
p=Path('scripts/check-native-layering.py');t=p.read_text();a=t.index('UPWARD =');b=t.index('\n\ndef kernel_files',a);t=t[:a]+'''USE = re.compile(r"\\buse\\s+([^;]+);", re.S)
TOKEN = re.compile(r"::|[A-Za-z_]\\w*|[{}*,]")
PATH = re.compile(r"\\b(?:crate|self|super|content)\\s*(?:::\\s*[A-Za-z_]\\w*)+")


def use_paths(tree):
    """Expand Rust use groups, retaining aliases (including `crate::{self as root}`)."""
    tokens = TOKEN.findall(tree)
    index = 0

    def branch(prefix):
        nonlocal index
        path = list(prefix)
        while index < len(tokens):
            token = tokens[index]
            if token in (",", "}"):
                break
            index += 1
            if token == "::":
                continue
            if token == "{":
                while index < len(tokens) and tokens[index] != "}":
                    yield from branch(path)
                    if index < len(tokens) and tokens[index] == ",":
                        index += 1
                index += 1
                return
            if token == "as":
                alias = tokens[index]
                index += 1
                yield path, alias
                return
            path.append(token)
        if path:
            yield path, path[-1]

    yield from branch([])


def resolve_path(parts, module, aliases):
    first, *rest = parts
    if first in aliases:
        return aliases[first] + rest
    if first == "crate":
        return rest
    if first in ("self", "super"):
        path = list(module)
        for part in parts:
            if part == "super":
                path = path[:-1]
            elif part != "self":
                path.append(part)
        return path
    return parts
''' +t[b:];a=t.index('def upward_references(root):');b=t.index('\n\ndef main():',a);t=t[:a]+'''def upward_references(root):
    for path in kernel_files(root):
        code = "\\n".join(line.split("//", 1)[0] for line in path.read_text(encoding="utf-8").splitlines())
        module = list(path.relative_to(root / SRC).with_suffix("").parts)
        if module[-1] == "mod":
            module.pop()
        imports = list(USE.finditer(code))
        declarations = [(match, list(use_paths(match.group(1)))) for match in imports]
        aliases = {}
        # Use declarations are not order-dependent in Rust.
        for _ in declarations:
            for _, paths in declarations:
                for parts, alias in paths:
                    resolved = resolve_path(parts, module, aliases)
                    if parts[0] in ("crate", "self", "super") or parts[0] in aliases:
                        aliases[alias] = resolved[:-1] if resolved[-1:] == ["self"] else resolved
        for match, paths in declarations:
            for parts, _ in paths:
                resolved = resolve_path(parts, module, aliases)
                if resolved[:1] in (["content"], ["engine"]):
                    yield path.relative_to(root), code.count("\\n", 0, match.start()) + 1
        # Mask imports so grouped and ordinary use paths are counted once.
        body = list(code)
        for match in imports:
            body[match.start():match.end()] = " " * (match.end() - match.start())
        body = "".join(body)
        roots = "|".join(re.escape(name) for name in ("crate", "self", "super", "content", *aliases))
        paths = re.compile(r"\\b(?:" + roots + r")\\s*(?:::\\s*[A-Za-z_]\\w*)+")
        for match in paths.finditer(body):
            parts = re.findall(r"[A-Za-z_]\\w*", match.group())
            if resolve_path(parts, module, aliases)[:1] in (["content"], ["engine"]):
                yield path.relative_to(root), code.count("\\n", 0, match.start()) + 1
''' +t[b:];t=t.replace('PATH = re.compile(r"\\b(?:crate|self|super|content)\\s*(?:::\\s*[A-Za-z_]\\w*)+")\n','');p.write_text(t)
