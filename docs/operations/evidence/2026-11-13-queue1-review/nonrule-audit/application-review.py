"""Independently reconcile inventory application and append-only corrections.

Writes only application-review* evidence. This is not a product/rule test.
"""
import argparse
import collections
import copy
import datetime
import hashlib
import json
import re
import subprocess
from pathlib import Path
from zoneinfo import ZoneInfo

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[4]
QUEUE = BASE.parent
parser = argparse.ArgumentParser()
parser.add_argument("--label", default="corrected")
args = parser.parse_args()
assert re.fullmatch(r"[a-z0-9-]+", args.label)
inputs = {}


def read_json(path):
    raw = path.read_bytes()
    inputs[str(path.relative_to(ROOT))] = hashlib.sha256(raw).hexdigest()
    return json.loads(raw)


def serialized(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def pointer(path, line, end=None):
    return {"path": path, "line": line, "end": end or line}


nonrule = read_json(BASE / "findings.json")
snapshot = read_json(BASE / "inventory-snapshot.json")
upstream_excerpts = {r["key"]: r for r in read_json(BASE / "upstream-excerpts.json")}
application = read_json(QUEUE / "inventory-application.json")
other = read_json(QUEUE / "independent/other-sources/findings.json")
current_path = ROOT / "docs/migration/g0/inventory_status.json"
current = read_json(current_path)
apply_path = QUEUE / "apply-reviewed-inventory.py"
inputs[str(apply_path.relative_to(ROOT))] = sha(apply_path.read_bytes())
original_raw = subprocess.check_output(
    ["git", "show", "HEAD:docs/migration/g0/inventory_status.json"], cwd=ROOT)
original = json.loads(original_raw)
assert sha(original_raw) == application["before_sha256"] == nonrule["inventory_sha256"]
assert nonrule["inventory_sha256"] == snapshot["inventory_sha256"]
assert len(nonrule["items"]) == len(snapshot["items"]) == 224
assert len({r["key"] for r in nonrule["items"]}) == 224
assert all(original["items"][k] == row for k, row in snapshot["items"].items())

state = copy.deepcopy(original)
stages = []
first_changes = {r["key"]: r for r in application["changes"]}
assert len(first_changes) == application["changed_records"] == 378
for key, change in first_changes.items():
    assert state["items"][key] == change["before"]
    state["items"][key] = copy.deepcopy(change["after"])
state["updated_at"] = application["at"][:10]
state["counts"] = application["after_counts"]
assert sha(serialized(state)) == application["after_sha256"]
assert sum(r["before"]["status"] != r["after"]["status"] for r in first_changes.values()) == 226
stages.append({"record": "inventory-application.json", "after_sha256": sha(serialized(state)),
               "changed_keys": len(first_changes), "effective_status_withdrawals": 226})
correction_keys = set()
for path in sorted(QUEUE.glob("inventory-correction-*.json")):
    correction = read_json(path)
    assert sha(serialized(state)) == correction["before_sha256"], path
    changes = correction.get("changes", [correction] if "key" in correction else [])
    for change in changes:
        key = change["key"]
        before = copy.deepcopy(state["items"][key])
        if "before" in change:
            assert before == change["before"], (path, key)
            after = copy.deepcopy(change["after"])
        else:
            # Corrections 04/05 retain compact field-level before/after values.
            assert before["difference"] == before["diff"] == change["before_difference"], (path, key)
            after = copy.deepcopy(before)
            after["difference"] = after["diff"] = change["after_difference"]
        if path.name in {"inventory-correction-04.json", "inventory-correction-05.json"}:
            assert all(before.get(f) == after.get(f)
                       for f in ["status", "reviewer", "review_date"]), (path, key)
        state["items"][key] = after
        correction_keys.add(key)
    state["counts"] = correction["counts"]
    assert sha(serialized(state)) == correction["after_sha256"], path
    stages.append({"record": path.name, "after_sha256": sha(serialized(state)),
                   "changed_keys": len(changes),
                   "effective_status_withdrawals": correction["effective_status_withdrawals"]})
assert serialized(state) == current_path.read_bytes()
assert set(current["items"]) == set(original["items"]) and len(current["items"]) == 4424
assert dict(collections.Counter(r["status"] for r in current["items"].values())) == {
    k: v for k, v in current["counts"].items() if v}
assert all(current["items"][k] == r for k, r in original["items"].items() if k not in first_changes)

H = "crates/content-native-loader/src/"
U = "archive/upstream/noname-liuli/noname/extension/"
P = "packages/extensions/"
T = "apps/web-client/src/engine/surfaces/zuanSurface.ts"
supplemental = {
    ("L10", "cardFold"): ("上游默认 true 且保存两个兼容键；包资产/parser 默认 false。", [pointer(U+"史诗卡牌/extension.js", 2205, 2214), pointer(H+"epic_cards.rs", 94), pointer(P+"epic-cards-upstream-liuli/assets/config.json", 9)]),
    ("L10", "cardFold_proportion"): ("上游独立 3..10 比例字段；当前 EpicConfig 的完整字段集合未表示该比例，资产也不含此键。", [pointer(U+"史诗卡牌/extension.js", 2215, 2229), pointer(H+"epic_cards.rs", 26, 49), pointer(P+"epic-cards-upstream-liuli/assets/config.json", 1, 15)]),
    ("L10", "cardPhantom"): ("上游默认 true，资产和 parser 默认 false；不以本轮未截图作为下调理由。", [pointer(U+"史诗卡牌/extension.js", 2231, 2236), pointer(H+"epic_cards.rs", 95), pointer(P+"epic-cards-upstream-liuli/assets/config.json", 10)]),
    ("L10", "shichangshi"): ("上游默认 true，资产和 parser 默认 false；独立规则技能存在不消除默认值差异。", [pointer(U+"史诗卡牌/extension.js", 2245, 2249), pointer(H+"epic_cards.rs", 97), pointer(P+"epic-cards-upstream-liuli/assets/config.json", 12)]),
    ("L10", "分割线02"): ("上游明确为 line.png 菜单图片项；当前 EpicConfig/PresentationFlags 的菜单模型没有该项，已读实际 ExtensionSurfacePanel 没有此史诗设置图片。这里只确认当前实现的菜单差别，未签视觉对照。", [pointer(U+"史诗卡牌/extension.js", 2258, 2263), pointer(H+"epic_cards.rs", 26, 49), pointer(H+"epic_cards.rs", 103, 127), pointer("apps/web-client/src/components/battle/ExtensionSurfacePanel.vue", 420)]),
    ("L25", "newhuihe"): ("上游四值版本枚举默认 new_standard；当前四键 schema 与 ZuanUiRuntimeState 没有版本字段，通用阶段 cue 不能表示这四个分支。", [pointer(U+"祖安武将/extension.js", 50572, 50582), pointer(H+"zuan.rs", 13, 20), pointer(T, 48, 68)]),
    ("L25", "perspective"): ("上游独立武将透视缩放布尔；当前四键 schema 和 UI 状态均没有该座位缩放参数，不能把别处固定布局当旧设置映射。", [pointer(U+"祖安武将/extension.js", 50757, 50761), pointer(H+"zuan.rs", 13, 20), pointer(T, 48, 68)]),
    ("L25", "bizhi"): ("上游开始游戏前菜单壁纸枚举含 0->1/2->3/4->5 保存转换；当前 schema 只读四个另名布尔，activeWallpaper 是运行状态，不表示此枚举和保存转换。", [pointer(U+"祖安武将/extension.js", 50762, 50803), pointer(H+"zuan.rs", 109, 132), pointer(T, 57), pointer(T, 132, 134)]),
    ("L25", "showboss"): ("上游独立跨模式将包开关；当前 parser 读取的四个字段没有该键或兼容映射，静态目录登记不能表达此设置门。未实跑将池分配。", [pointer(U+"祖安武将/extension.js", 51163, 51167), pointer(H+"zuan.rs", 109, 132), pointer(P+"zuan-upstream-liuli/assets/config.json", 1)]),
    ("L25", "hidessr"): ("上游默认 true 且声明将包与 AI/玩家候选排除；当前 parser 的四键中没有该键或兼容映射，不能称原配置原样支持。未实跑候选将池。", [pointer(U+"祖安武将/extension.js", 51168, 51172), pointer(H+"zuan.rs", 13, 20), pointer(H+"zuan.rs", 109, 132)]),
    ("L25", "windowHigh"): ("原菜单当前只显示 0，1..7 已注释，但 onclick 仍保存旧值并处理窗口尺寸；当前 schema 没有此旧键/zuanwindow 转换。不能声称上游当前仍显示七个尺寸选项。", [pointer(U+"祖安武将/extension.js", 51344, 51391), pointer(H+"zuan.rs", 109, 132), pointer(P+"zuan-upstream-liuli/assets/config.json", 1)]),
    ("L25", "text"): ("旧 text 布尔并不被映射到新的 enable_editor；原生另有 offline_debug_only 限制。这是具体输入/能力边界，未声称编辑器实际操作已验证。", [pointer(U+"祖安武将/extension.js", 51404, 51408), pointer(H+"zuan.rs", 115, 122), pointer(H+"zuan.rs", 200)]),
}

concrete = {"具体规则/公开信息分支差异", "只有计数/cue/flag，未接可见/可听效果", "菜单模型和可见交互未接入"}
pending = {"表现配置仅解析/未驱动显示", "规则/目录配置应用链未闭合"}
direct_default_counterexamples = {("L10", "cardFold"), ("L10", "cardPhantom"), ("L10", "shichangshi")}
restored_schema_only = set(supplemental) - direct_default_counterexamples
assert len(restored_schema_only) == 9
other_by_key = {r["key"]: r for r in other["samples"]}
rows = []
issues = []
source_refs = []
counts = collections.Counter()
metadata_fields = ["reviewer", "review_date", "test_name", "upstream_file_line", "native", "native_ref"]
for finding in nonrule["items"]:
    key = finding["key"]
    before, after = original["items"][key], current["items"][key]
    applied = first_changes[key]
    own_withdraw = finding["status_action"] == "建议下调" and finding["difference_kind"] in concrete
    changed_status = before["status"] != after["status"]
    assert after["difference"] == after["diff"]
    assert before == applied["before"]
    assert applied["after"]["status"] == ("简化近似" if applied["withdrawing"] else before["status"])
    own_review = [r for r in applied["reviews"] if r["reviewer"] == "/root/repair_doc_refs"]
    assert len(own_review) == 1 and own_review[0]["confirmed_difference"] == own_withdraw
    assert finding["difference"] in own_review[0]["difference"]
    if own_withdraw:
        counts["own_concrete_withdrawals"] += 1
        assert changed_status and after["status"] == "简化近似"
    if finding["difference_kind"] in pending:
        counts["pending_consumer_chain_items"] += 1
        counts["pending_with_other_concrete_withdrawal" if changed_status else "pending_status_retained"] += 1
    if before["status"] == after["status"]:
        assert all(before.get(f) == after.get(f) for f in metadata_fields), key
        counts["unchanged_status_metadata_preserved"] += 1
    immutable = (set(before) | set(after)) - {"status", "reviewer", "review_date", "difference", "diff", "note"}
    assert all(before.get(f) == after.get(f) for f in immutable), key
    assert not before.get("note") or after["note"].startswith(before["note"]), key
    if changed_status and not own_withdraw:
        audit = supplemental.get((finding["source_id"], finding["id"]))
        if not audit or (finding["source_id"], finding["id"]) not in direct_default_counterexamples:
            issues.append({"key": key, "problem": "没有直接值反例；旧键/字段缺失或静态未找到消费者不足以单独下调。"})
        else:
            assert other_by_key[key]["difference_confirmed_by_source"]
            counts["supplemental_concrete_withdrawals"] += 1
            source_refs.extend(audit[1])
    if finding["classification"] in {"上游注释，惰性登记", "上游主动禁用，惰性登记"}:
        assert before["status"] == after["status"]
        excerpt = upstream_excerpts[key]
        actual_source = (ROOT.parent / excerpt["path"]).read_bytes()
        assert sha(actual_source) == excerpt["source_sha256"]
        if finding["classification"] == "上游注释，惰性登记":
            assert not excerpt["active_definition"]
            counts["commented_definitions_verified_against_source_hash"] += 1
        else:
            assert excerpt["active_definition"]
            fragment = "\n".join(actual_source.decode().splitlines()[excerpt["start"]-1:excerpt["end"]])
            assert re.search(r"return\s+false", fragment)
            counts["filter_false_definitions_verified_against_source_hash"] += 1
        counts["inert_items_preserved"] += 1
    if finding["original_status"] == "简化近似":
        assert after["status"] == "简化近似"
        counts["original_approximate_preserved"] += 1
    if finding["original_status"] == "已复核等价" and finding["status_action"] == "保留原状态，仅澄清范围":
        assert before["status"] == after["status"]
        counts["old_equivalence_in_retained_group_preserved"] += 1
    source_refs.extend(finding["native_refs"])
    source_refs.extend(finding["upstream_refs"])
    rows.append({"key": key, "difference_kind": finding["difference_kind"],
                 "before_status": before["status"], "first_applied_status": applied["after"]["status"],
                 "current_status": after["status"], "own_concrete_withdrawal": own_withdraw,
                 "other_concrete_source_review": supplemental.get((finding["source_id"], finding["id"])) if changed_status and not own_withdraw else None,
                 "corrected_after_initial_application": key in correction_keys,
                 "retained_metadata_matches_original": all(before.get(f) == after.get(f) for f in metadata_fields) if not changed_status else None,
                 "current_difference": after["difference"], "current_note": after["note"]})

assert counts["own_concrete_withdrawals"] == 92
assert counts["pending_consumer_chain_items"] == 91
assert counts["inert_items_preserved"] == 14
assert counts["commented_definitions_verified_against_source_hash"] == 12
assert counts["filter_false_definitions_verified_against_source_hash"] == 2
assert counts["original_approximate_preserved"] == 11
assert counts["old_equivalence_in_retained_group_preserved"] == 30
assert counts["supplemental_concrete_withdrawals"] == 3
assert counts["pending_status_retained"] == 88
assert counts["pending_with_other_concrete_withdrawal"] == 3
corrected_findings = []
for source, ident, explanation, refs in [
    ("L20", "_xigua_zhuwanzhongji_", "首次下调把真实主亡忠继身份交换误判为 L20BatchA flags；实际优先绑定 ZhuWanzhongJi，已恢复原状态/复核人/日期。mingzhong/zhuUpdate/死亡时序仍待实际同场景核验。", [pointer(H+"host_skill_bindings.rs", 419), pointer(H+"decision34_pack9.rs", 522), pointer("crates/engine-native-content/src/content/host/step.rs", 98), pointer("crates/engine-native-content/src/content/host/step/skills_status.rs", 31, 51), pointer("crates/engine-native-content/src/content/host/filtering/common.rs", 164, 172)]),
    ("L19", "zuan_brawl", "首次文案误称枚举未实现；完整五值解析和默认 all 已存在。未找到消费者不足以单独确认缺失，已恢复原状态/复核人/日期，off/all/daily 与跨日重启继续待核验。", [pointer(H+"doomsday.rs", 189, 233), pointer("apps/web-client/src/engine/extensionSurfaceRuntime.ts", 87, 94), pointer("apps/web-client/src/components/battle/ExtensionSurfacePanel.vue", 660, 675)]),
]:
    row = next(r for r in rows if r["key"].startswith(f"{source}:") and r["key"].split("@", 1)[0].endswith(":"+ident))
    key = row["key"]
    assert current["items"][key]["status"] == original["items"][key]["status"]
    assert all(current["items"][key][f] == original["items"][key][f] for f in ["reviewer", "review_date"])
    assert key in correction_keys
    corrected_findings.append({"key": key, "result": "已更正，保留首次错误与原应用记录", "explanation": explanation, "source_refs": refs})
    source_refs.extend(refs)

schema_only_observations = []
for source, ident in sorted(restored_schema_only):
    finding = next(r for r in nonrule["items"] if (r["source_id"], r["id"]) == (source, ident))
    key = finding["key"]
    assert finding["difference_kind"] in pending
    assert key in correction_keys
    assert current["items"][key]["status"] == original["items"][key]["status"]
    assert all(current["items"][key][f] == original["items"][key][f] for f in ["reviewer", "review_date"])
    observation, refs = supplemental[source, ident]
    schema_only_observations.append({"key": key, "actual_static_observation": observation,
                                    "status": "恢复原状态；保留待核验观察", "source_refs": refs,
                                    "limit": "没有实际显示证据或正向规则分支反例；不以字段/旧键差别单独下调。"})
    corrected_findings.append({"key": key, "result": "已更正，保留首次错误与原应用记录",
                               "explanation": "恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。", "source_refs": refs})
    source_refs.extend(refs)
assert len(corrected_findings) == 11

clarifications = read_json(BASE / "application-review-semantic-clarifications.json")
clarified_keys = {r["key"] for r in clarifications["items"]}
assert len(clarified_keys) == 4
for finding in nonrule["items"]:
    if finding["key"] not in clarified_keys:
        assert finding["upstream_behavior"] in current["items"][finding["key"]]["difference"], finding["key"]
for clarification in clarifications["items"]:
    for ref in clarification["refs"]:
        source_refs.append(pointer(ref["path"], ref["start"], ref["end"]))
for ident, required in {
    "_mianshang_": ["damageZero/damageCancelled", "parent=player", "prevented", "flag", "未实跑"],
    "_shanghaipaishiyongtexiao_": ["useCardBegin", "damage", "game.me", "CardUsed", "白名单", "未实跑"],
    "extension.css": ["跋扈", "飞扬", "keyframes", "1.5", "保持原简化近似"],
    "group.css": ["jingx", "::before", "132x132", "group/sgroup.png", "未证明"],
}.items():
    finding = next(r for r in nonrule["items"] if r["source_id"] == "L20" and r["id"] == ident)
    assert all(token in current["items"][finding["key"]]["difference"] for token in required), ident
counts["source_behavior_text_retained_verbatim"] = len(rows) - len(clarified_keys)
counts["source_behavior_text_refined_with_current_sources"] = len(clarified_keys)

source_hashes = {}
source_excerpts = []
for ref in source_refs:
    path = ROOT.parent / ref["path"] if ref["path"].startswith("archive/") else ROOT / ref["path"]
    raw = path.read_bytes()
    lines = raw.decode().splitlines()
    assert 1 <= ref["line"] <= ref.get("end", ref["line"]) <= len(lines), ref
    source_hashes[ref["path"]] = sha(raw)
for explanation, refs in supplemental.values():
    for ref in refs:
        path = ROOT.parent / ref["path"] if ref["path"].startswith("archive/") else ROOT / ref["path"]
        lines = path.read_text().splitlines()
        source_excerpts.append({**ref, "lines": lines[ref["line"]-1:ref["end"]], "checked_behavior": explanation})
tracked_evidence_changes = subprocess.check_output(
    ["git", "diff", "--name-only", "--", "docs/operations/evidence"], cwd=ROOT, text=True).splitlines()
historical_evidence_deltas = []
for relative in tracked_evidence_changes:
    stat = subprocess.check_output(["git", "diff", "--numstat", "--", relative], cwd=ROOT, text=True).strip().split("\t", 2)
    assert len(stat) == 3 and stat[1] == "0", (relative, stat)
    old_raw = subprocess.check_output(["git", "show", "HEAD:"+relative], cwd=ROOT)
    now_raw = (ROOT / relative).read_bytes()
    # Adding a new dated section preserves historical records. Removing or
    # replacing an existing line would produce a nonzero deletion count above.
    historical_evidence_deltas.append({"path": relative, "inserted_lines": int(stat[0]),
                                      "deleted_lines": 0, "original_sha256": sha(old_raw),
                                      "current_sha256": sha(now_raw),
                                      "existing_historical_text_preserved": True})
assert all(sha((ROOT / p).read_bytes()) == h for p, h in inputs.items()), "Inputs changed during review"
result = {
    "reviewed_at": datetime.datetime.now(ZoneInfo("Asia/Taipei")).isoformat(),
    "reviewer": "/root/repair_doc_refs", "review_scope": "224项清单应用与更正，不重新签署技能/界面等价",
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "tracked_dirty_paths": subprocess.check_output(["git", "diff", "--name-only"], cwd=ROOT, text=True).splitlines(),
    "versions": nonrule["source_versions"], "result": "changes_requested" if issues else "accepted_after_corrections",
    "input_sha256": inputs, "git_head_inventory_sha256": sha(original_raw),
    "application_chain": stages, "current_inventory_sha256": sha(current_path.read_bytes()),
    "scope_key_count": len(rows), "scope_current_counts": dict(collections.Counter(r["current_status"] for r in rows)),
    "scope_effective_withdrawals": sum(r["before_status"] != r["current_status"] for r in rows),
    "global_current_counts": current["counts"],
    "global_effective_withdrawals": sum(original["items"][k]["status"] != r["status"] for k, r in current["items"].items()),
    "checks": dict(counts), "source_file_line_refs_checked": len(source_refs),
    "source_sha256": source_hashes, "corrected_initial_findings": corrected_findings,
    "schema_only_observations_restored_without_runtime_counterexamples": schema_only_observations,
    "unresolved_findings": issues, "tracked_historical_evidence_changes": tracked_evidence_changes,
    "tracked_historical_evidence_append_only_deltas": historical_evidence_deltas,
    "new_equivalence_signoffs": 0, "cargo_commands_run": 0, "browser_commands_run": 0,
    "native_runtime_scenarios_run": 0,
    "outside_scope": "L25 skills:_guanmodeskill 的 correction02 仅作字节/哈希链核对，不在本224项中，未复核其规则正确性。原224条规则/显示运行差异不在本次重新执行。",
    "source_excerpts_for_supplemental_cases": source_excerpts, "items": rows,
}
out = BASE / f"application-review-{args.label}.json"
with out.open("x") as stream:
    json.dump(result, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
text = ["# 队列1：224项清单应用独立复审", "", f"日期 {result['reviewed_at']}；HEAD `{result['head']}`；dirty。规则/NGSA/schema/ABI/WS：333/107/5/759/8。", "",
        "本次只新增 application-review* 证据，不改清单、产品、公共文档或旧证据。", "",
        f"结论：`{result['result']}`。224个key均可从原始清单经首次应用与{len(stages)-1}份更正记录精确还原到当前字节；输入与源码哈希保留在 JSON。", "",
        f"当前这224项：已复核等价 {result['scope_current_counts'].get('已复核等价', 0)} / 简化近似 {result['scope_current_counts'].get('简化近似', 0)}；有效撤回 {result['scope_effective_withdrawals']}。全清单计数 {current['counts']}，有效撤回 {result['global_effective_withdrawals']}，没有新增等价签署。", "",
        "| 核对组 | 数量/结果 |", "| --- | --- |",
        f"| 本表具体规则14 + cue/flag57 + 菜单21 | {counts['own_concrete_withdrawals']}项均采纳下调 |",
        f"| 消费链待核验91 | 保留状态{counts['pending_status_retained']}；仅另有true→false默认值直接反例下调{counts['pending_with_other_concrete_withdrawal']} |",
        f"| 原保留等价30 / 原简化11 | {counts['old_equivalence_in_retained_group_preserved']} / {counts['original_approximate_preserved']}均保留 |",
        f"| 注释12 + filter恒false2 | {counts['inert_items_preserved']}项均保留，没有当作活动技能 |",
        f"| 状态未变的原复核元数据 | {counts['unchanged_status_metadata_preserved']}项逐字段保留 |",
        "| difference/diff、其他原字段、旧note前缀 | 224项均保持一致或保留 |", "",
        "首次应用中11项撤回理由已退回并另写更正，原始审查和应用记录保留：", ""]
for finding in corrected_findings:
    text.append(f"- `{finding['key']}`：{finding['explanation']}")
text.extend(["", "另保留的3条撤回只采纳cardFold/cardPhantom/shichangshi的上游init=true而包资产/parser默认false这一直接值反例。其余9条字段/旧键/模型观察保持待核验，不据此下调；L19枚举解析存在、L20身份交换存在的误判也已撤回。没有只沿用其他审查的confirmed标记。", "",
             "差别文案保留220条逐key上游行为；L20免伤/伤害牌动画与两张CSS另按实际源精化4条，承认已有原生事件/过滤，限定未实现或未对照的可见效果。correction04/05均未变更status/reviewer/date。", "",
             "原生/上游运行场景、Cargo、浏览器均为0；没有重验整技能等价。L25 _guanmodeskill 只核对correction02记录的字节链，不在本224项规则复核范围。", "",
             "复核脚本先后因未兼容correction04/05紧凑差别格式、把新增日期段也视作历史改写而退出1；已补字段级读取并将守卫限定为旧原文不得删除/替换，原stderr与补跑分开保留在命令日志。这不属于产品测试结果。", "",
             f"源码引用界限核对 {len(source_refs)} 处、独立文件哈希 {len(source_hashes)} 个；历史evidence旧原文删除/替换0，另有{len(historical_evidence_deltas)}个文件被主agent插入新日期段（本agent未写）。未解决应用问题 {len(issues)}。", "",
             f"完整输出：[application-review-{args.label}.json](application-review-{args.label}.json)。实际命令与输出见 [application-review-commands.jsonl](application-review-commands.jsonl)。", ""])
with (BASE / f"application-review-{args.label}.md").open("x") as stream:
    stream.write("\n".join(text))
print(json.dumps({k: result[k] for k in ["result", "scope_key_count", "scope_current_counts", "scope_effective_withdrawals", "global_current_counts", "global_effective_withdrawals", "checks", "source_file_line_refs_checked", "unresolved_findings"]}, ensure_ascii=False))
