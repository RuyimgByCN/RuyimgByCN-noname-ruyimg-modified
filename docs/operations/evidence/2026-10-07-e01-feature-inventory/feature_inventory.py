"""E01/G0 功能级清点（只读，不执行扩展代码）。在 noname-next 下运行：
python3 docs/operations/evidence/2026-10-07-e01-feature-inventory/feature_inventory.py --workspace ..
对两仓 37 个来源逐个提取功能面特征：入口脚本规模、game.import 段、
技能/卡牌/武将/翻译段规模、配置与设置项、子插件（嵌套 extension.js）、
资源类别与体量、混淆特征（eval/packed/超长单行）。
输出 JSON 供 E01 功能清单与「目标接口/验证入口」登记使用。
"""
import argparse
import json
import re
from pathlib import Path

CODE_SUFFIX = {".js", ".ts", ".mjs", ".cjs"}
RESOURCE_DIRS = ("audio", "image", "font", "theme", "video")


def count_toplevel_keys(text: str, section: str):
    """统计 `section: { ... }` 顶层键数（一层花括号近似，忽略字符串干扰）。"""
    m = re.search(rf"(?m)^\s*{section}\s*:\s*\{{", text)
    if not m:
        return None
    depth = 0
    keys = 0
    in_str = False
    quote = ""
    for i in range(m.end() - 1, len(text)):
        ch = text[i]
        if in_str:
            if ch == quote and text[i - 1] != "\\":
                in_str = False
            continue
        if ch in "\"'`":
            in_str = True
            quote = ch
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return keys
        elif depth == 1 and ch == ":":
            keys += 1
    return keys


def scan_entry(path: Path):
    """入口脚本（extension.js 或 main/precontent.js）功能特征。"""
    if not path.exists():
        return {"entry": None}
    text = path.read_text(errors="replace")
    lines = text.splitlines()
    feature = {
        "entry": str(path.relative_to(path.parents[2])),
        "entry_lines": len(lines),
        "game_import": bool(re.search(r"\bgame\.import\s*\(", text)),
        "skills": count_toplevel_keys(text, "skill"),
        "cards": count_toplevel_keys(text, "card"),
        "characters": count_toplevel_keys(text, "character"),
        "translations": count_toplevel_keys(text, "translate"),
        "config_items": count_toplevel_keys(text, "config"),
        "settings_refs": len(re.findall(r"\bgame\.config\b|lib\.config\.all\.", text)),
        "external_extension_refs": sorted(set(re.findall(r"""["']extension/([^/"']+)["']""", text)))[:20],
        "obfuscation_markers": {
            "eval_calls": len(re.findall(r"\beval\s*\(", text)),
            "packed_header": bool(re.search(r"^\s*//\s*packed|_0x[0-9a-f]{4,}", text, re.M)),
            "very_long_lines": sum(1 for l in lines if len(l) > 2000),
        },
    }
    return feature


def scan_source(directory: Path):
    """单个来源目录的功能面。"""
    feature = {"name": directory.name, "entries": [], "subplugins": [], "resources": {}}
    # 入口：extension.js；无则 main/precontent.js 等候选。
    for candidate in (directory / "extension.js", directory / "main" / "precontent.js"):
        if candidate.exists():
            info = scan_entry(candidate)
            feature["entries"].append(info)
    # 子插件：嵌套目录里的独立 extension.js（除根）。
    for sub in sorted(directory.rglob("extension.js")):
        if sub.parent != directory:
            feature["subplugins"].append(str(sub.relative_to(directory)))
    # 资源类别与体量。
    for res in RESOURCE_DIRS:
        total = 0
        files = 0
        for p in directory.rglob(f"{res}/**/*"):
            if p.is_file():
                files += 1
                total += p.stat().st_size
        if files:
            feature["resources"][res] = {"files": files, "bytes": total}
    # 扁平资源文件（如字体/图片散在根部）。
    flat = [p for p in directory.iterdir() if p.is_file() and p.suffix.lower() in {".png", ".jpg", ".mp3", ".ttf"}]
    if flat:
        feature["resources"]["root_files"] = {"files": len(flat), "bytes": sum(p.stat().st_size for p in flat)}
    # card/skill 独立子目录形态（欢乐/杀海类壳扩展：extension.js 是壳，card/skill.js 承载）。
    for section, rel in (("cards", "card/card.js"), ("skills", "card/skill.js"), ("characters", "character/character.js")):
        p = directory / rel
        if p.exists():
            text = p.read_text(errors="replace")
            feature[f"shell_{section}"] = count_toplevel_keys(text, section if section != "characters" else "character")
    return feature


def inventory(workspace: Path):
    out = {"schema": 1, "sources": []}
    for repo, relative in (("noname", "apps/core/extension"), ("noname-liuli", "noname/extension")):
        checkout = workspace / "archive" / "upstream" / repo
        for directory in sorted((checkout / relative).iterdir()):
            if not directory.is_dir():
                continue
            item = {"repo": repo}
            item.update(scan_source(directory))
            out["sources"].append(item)
    return out


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(inventory(args.workspace.resolve()), ensure_ascii=False, indent=2))
