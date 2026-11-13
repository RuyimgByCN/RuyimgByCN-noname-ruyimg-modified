"""Read-only audit of the 141 sampled keys and their inventory application."""

import argparse
import collections
import copy
import datetime
import hashlib
import json
import math
import pathlib
import random
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
BASE = HERE.parents[1]
ROOT = BASE.parents[3]
INVENTORY = ROOT / "docs/migration/g0/inventory_status.json"
REVIEWER = "/root/fix_dddanzhi"
INDEPENDENT_REVIEWERS = {REVIEWER, "/root/repair_doc_refs"}

# The earlier dated review stays intact. These are corrections to its support,
# not new equivalence findings and not instructions to mutate the inventory.
RETURNED = {
    ("L20", "_xigua_zhuwanzhongji_"): (
        "优先绑定误读",
        "实际优先绑定ZhuWanzhongJi；死亡者为Lord/持有者为Loyalist的过滤及身份交换已经存在。"
        "原‘L20BatchA只计flag、没有继任’理由错误。mingzhong/zhuUpdate与完整死亡时序未在本次执行核验。",
        ["crates/content-native-loader/src/host_skill_bindings.rs:419",
         "crates/content-native-loader/src/decision34_pack9.rs:522",
         "crates/engine-native-content/src/content/host/step.rs:98",
         "crates/engine-native-content/src/content/host/filtering/common.rs:164",
         "crates/engine-native-content/src/content/host/step/skills_status.rs:31"],
        "忠臣/明忠分别在主公死亡前触发，记录双方身份、后续主公技及zhuUpdate触发；本次未执行。",
    ),
    ("L19", "zuan_brawl"): (
        "已有解析，消费链待核验",
        "doomsday.rs已有off/standard/random/all/daily五值、默认all和完整解析。"
        "原‘枚举未实现’不成立；入口、daily跨日持久化目前仅未找到调用，没有产品反例。",
        ["crates/content-native-loader/src/doomsday.rs:189",
         "crates/content-native-loader/src/doomsday.rs:219",
         "crates/content-native-loader/src/doomsday.rs:225",
         "apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:660"],
        "在实际产品分别选off/all/daily，比较玩法入口并跨日重启检查daily；本次未执行。",
    ),
    ("L10", "cardFold_proportion"): (
        "配置模型未覆盖，不足证明产品差异",
        "上游有30%至100%比例，EpicConfig和包资产未建该字段；目前只有字段/文字搜索，"
        "未定位完整手牌布局配置入口或验证折叠宽度，撤回‘比例效果缺失’的确认程度。",
        ["crates/content-native-loader/src/epic_cards.rs:25",
         "packages/extensions/epic-cards-upstream-liuli/assets/config.json:1"],
        "关闭完全折叠后切30%/100%，以相同宽度和手牌数测卡牌间距与重启保存；本次未执行。",
    ),
    ("L10", "分割线02"): (
        "仅未定位菜单图，未作画面对照",
        "上游是line.png菜单分隔图。EpicConfig没有此项不能单独证明产品没有等效菜单装饰；"
        "没有在实际菜单观察或截图，因此撤回‘纯菜单装饰缺失’确认程度。",
        ["crates/content-native-loader/src/epic_cards.rs:25"],
        "打开扩展实际菜单，记录本分隔图对应位置及布局截图；本次未执行。",
    ),
    ("L25", "bizhi"): (
        "仅未定位旧配置消费者",
        "原证据仅为ZuanConfig四键、包资产及activeWallpaper状态中未见bizhi；"
        "activeWallpaper属于对局状态，但未追完菜单壁纸入口，不能据此确认枚举/保存转换缺失。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "packages/extensions/zuan-upstream-liuli/assets/config.json:1",
         "apps/web-client/src/engine/surfaces/zuanSurface.ts:57"],
        "在开局前菜单选关闭与月夜壁纸，检查0→1等归一化、重启保存及入局后退出壁纸；本次未执行。",
    ),
    ("L25", "hidessr"): (
        "目录登记不等于可选将池",
        "原证据仅为ZuanConfig无hidessr和SSR目录始终登记。目录存在不证明默认AI/玩家可选；"
        "未追完选将过滤或实际开关，撤回‘缺默认隐藏/将池排除’确认程度。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "crates/content-native-loader/src/zuan_data.rs:1",
         "packages/extensions/zuan-upstream-liuli/assets/config.json:1"],
        "同一安装组合分别开/关隐藏，核对SSR包展示及玩家/AI候选将池；本次未执行。",
    ),
    ("L25", "newhuihe"): (
        "旧枚举与动画消费链未定位",
        "ZuanConfig及ZuanUiRuntimeState没有旧四值枚举只是模型检查；"
        "未追完实际阶段动画入口或观察四分支，撤回‘四种配置分支缺失’确认程度。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "apps/web-client/src/engine/surfaces/zuanSurface.ts:48"],
        "按old/new_standard/new_brighten/new_shine逐个开始同一武将回合并录像比较；本次未执行。",
    ),
    ("L25", "perspective"): (
        "旧键无命中不足判断座位布局",
        "仅查ZuanConfig和祖安UI状态无perspective；实际牌桌也可能由通用布局控制。"
        "没有开关前后同座位截图/尺寸，撤回‘透视参数未实现’确认程度。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "apps/web-client/src/engine/surfaces/zuanSurface.ts:48"],
        "固定八人座位开关透视缩放，比较近远武将牌尺寸与手牌/特效比例；本次未执行。",
    ),
    ("L25", "showboss"): (
        "目录扩展开关消费链未定位",
        "包配置无showboss和未查到旧键不证明普通模式将池没有等效扩入入口；"
        "未追完各模式候选注册，撤回‘跨模式开关缺映射’确认程度。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "packages/extensions/zuan-upstream-liuli/assets/config.json:1"],
        "在普通身份模式分别关/开此选项，比较剑阁/挑战武将候选并检查依赖组合；本次未执行。",
    ),
    ("L25", "text"): (
        "调试安全参数不等于旧编辑器效果",
        "enable_editor/offline_debug_only的解析和拒绝条件是确定存在的原生门禁，"
        "但原表未追完text到当前产品编辑器的映射；键名不同及解析门禁不足证明全场编辑行为反例。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "crates/content-native-loader/src/zuan.rs:115",
         "crates/content-native-loader/src/zuan.rs:200"],
        "限定离线调试环境，开启真实编辑器后分别修改武将/位置/手牌并查身份显示；本次未执行。",
    ),
    ("L25", "windowHigh"): (
        "未验证窗口命令和持久化",
        "上游当前菜单仅列0，旧1至7选项已注释但onclick仍保留历史值分支。"
        "原生模型未列windowHigh不证明宿主窗口/旧值迁移失败；未测试桌面窗口与重启。",
        ["crates/content-native-loader/src/zuan.rs:15",
         "packages/extensions/zuan-upstream-liuli/assets/config.json:1"],
        "桌面产品选择0并带入历史值1/7，记录窗口尺寸及重启保存；本次未执行。",
    ),
}

GUAN_CORRECTION = {
    "reason": "原‘L25通用展示桩’路由错误，撤回该解释；已读实际优先实现后仍有具体规则差异。",
    "finding": "实际先绑定L19DoomsdayBatchB。其_guanmodeskill防翻面、每回合摸存活人数减一并伤首名他人，"
               "本来源却在enterGame/gameDrawAfter按主公/身份授guanfu/guifu_nei并初始化生命和身份记录。",
    "refs": ["crates/content-native-loader/src/host_skill_bindings.rs:548",
             "crates/content-native-loader/src/decision34_pack9.rs:522",
             "crates/engine-native-content/src/content/host/step/l19_batch_b.rs:332",
             "crates/engine-native-content/src/content/host/step/l19_batch_b.rs:483",
             "crates/engine-native-content/src/content/host/step/l19_batch_b.rs:813"],
}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def serialized(data):
    return (json.dumps(data, ensure_ascii=False, indent=2) + "\n").encode()


def main(label):
    at = datetime.datetime.now(datetime.timezone.utc).astimezone().isoformat()
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    inventory_bytes = INVENTORY.read_bytes()
    inventory = json.loads(inventory_bytes)
    before_bytes = subprocess.check_output(
        ["git", "show", "HEAD:docs/migration/g0/inventory_status.json"], cwd=ROOT
    )
    before = json.loads(before_bytes)
    inputs = {}

    def read(path):
        raw = path.read_bytes()
        inputs[str(path.relative_to(ROOT))] = {"sha256": digest(raw), "bytes": len(raw)}
        return json.loads(raw)

    application = read(BASE / "inventory-application.json")
    findings = read(HERE / "findings.json")
    selected = read(HERE / "selected-samples.json")
    nonrule = read(BASE / "nonrule-audit/findings.json")
    script = BASE / "apply-reviewed-inventory.py"
    inputs[str(script.relative_to(ROOT))] = {"sha256": digest(script.read_bytes()),
                                             "bytes": script.stat().st_size}
    audit_script = pathlib.Path(__file__).resolve()
    inputs[str(audit_script.relative_to(ROOT))] = {"sha256": digest(audit_script.read_bytes()),
                                                  "bytes": audit_script.stat().st_size}
    changes = {c["key"]: c for c in application["changes"]}
    samples = findings["samples"]
    sample_keys = {s["key"] for s in samples}
    nonrule_by_key = {r["key"]: r for r in nonrule["items"]}
    checks = []

    def check(name, condition, detail=None):
        checks.append({"check": name, "passed": bool(condition), "detail": detail})

    check("head_matches_reviewed_input", head == findings["head"] == selected["head"])
    check("application_before_is_HEAD_inventory_bytes", digest(before_bytes) == application["before_sha256"])
    check("141_unique_real_inventory_keys", len(samples) == len(sample_keys) == 141
          and sample_keys <= before["items"].keys() and sample_keys <= inventory["items"].keys())
    check("no_juxian_in_scope", not any("dddjuxian" in k for k in sample_keys))
    check("categories_ids_refs_retained", all(all(
        s[k] == before["items"][s["key"]][k] == inventory["items"][s["key"]][k]
        for k in ("source_id", "category", "id", "upstream_ref")) for s in samples))

    selected_keys = set()
    sampling = {}
    for sid, source in selected["sources"].items():
        source_keys = sorted(k for k, row in before["items"].items() if row["source_id"] == sid)
        count = min(len(source_keys), max(10, math.ceil(len(source_keys) * .1)))
        expected = set(random.Random(selected["seed"]).sample(source_keys, count))
        actual = {s["key"] for s in source["random_samples"]}
        supplement = {s["key"] for s in source["interactive_supplements"]}
        selected_keys |= actual | supplement
        sampling[sid] = {"denominator": len(source_keys), "random": len(actual),
                         "supplement": len(supplement), "reviewed": len(actual | supplement)}
        check("whole_source_sampling_" + sid, expected == actual
              and len(source_keys) == source["total_inventory_items"]
              and not (actual & supplement))
    check("selected_equals_reviewed_141", selected_keys == sample_keys)

    replay = copy.deepcopy(before)
    for change in application["changes"]:
        check("first_before_row_" + change["key"], replay["items"][change["key"]] == change["before"])
        replay["items"][change["key"]] = copy.deepcopy(change["after"])
    replay["updated_at"] = application["at"][:10]
    replay["counts"] = application["after_counts"]
    check("full_first_application_replays_to_hash", digest(serialized(replay)) == application["after_sha256"])
    corrections = []
    for path in sorted(BASE.glob("inventory-correction-*.json")):
        correction = read(path)
        check("correction_input_hash_" + path.name, digest(serialized(replay)) == correction["before_sha256"])
        correction_changes = correction.get("changes", [correction])
        for change in correction_changes:
            row = replay["items"][change["key"]]
            if "before_difference" in change:
                check("correction_before_difference_" + path.name + "_" + change["key"],
                      row["difference"] == row["diff"] == change["before_difference"])
                row["difference"] = row["diff"] = change["after_difference"]
            else:
                check("correction_before_row_" + path.name + "_" + change["key"], row == change["before"])
                replay["items"][change["key"]] = copy.deepcopy(change["after"])
        replay["counts"] = correction["counts"]
        check("correction_output_hash_" + path.name, digest(serialized(replay)) == correction["after_sha256"])
        corrections.append({"file": str(path.relative_to(BASE)),
                            "before_sha256": correction["before_sha256"],
                            "after_sha256": correction["after_sha256"],
                            "keys": [c["key"] for c in correction_changes]})
    check("current_inventory_matches_complete_application_chain", serialized(replay) == inventory_bytes)
    check("4424_keys_preserved", set(before["items"]) == set(inventory["items"])
          and len(inventory["items"]) == inventory["total_items"] == 4424)
    check("all_global_changes_recorded", all(row == inventory["items"][key]
          for key, row in before["items"].items() if key not in changes))
    check("no_new_equivalence_status", all(row["status"] != "已复核等价"
          or before["items"][key]["status"] == "已复核等价" for key, row in inventory["items"].items()))

    entries = []
    for s in samples:
        key = s["key"]
        old = before["items"][key]
        row = inventory["items"][key]
        change = changes.get(key)
        code = (s["source_id"], s["id"])
        returned = RETURNED.get(code)
        confirmed = s["difference_confirmed_by_source"] and not returned
        has_gap = bool(s["verification_gap"]) or bool(returned)
        corrected_support = "源码确认差异" if confirmed else ("待核验" if has_gap else "静态未见差异")
        independent_confirms = [r for r in (change or {}).get("reviews", [])
                               if r["confirmed_difference"] and r["reviewer"] != REVIEWER]
        other_support = []
        for r in independent_confirms:
            nr = nonrule_by_key.get(key)
            valid = (r["reviewer"] == "/root/repair_doc_refs" and nr
                     and nr["status_action"] == "建议下调"
                     and nr["difference_kind"] in {
                         "具体规则/公开信息分支差异", "只有计数/cue/flag，未接可见/可听效果", "菜单模型和可见交互未接入"})
            check("same_key_other_confirmed_support_" + key, valid)
            other_support.append({"review": r, "original_finding": nr, "accepted_for_application": bool(valid)})
        expected_status = "简化近似" if confirmed or any(x["accepted_for_application"] for x in other_support) else old["status"]
        check("sample_status_" + key, row["status"] == expected_status)
        meta_changed = any(row.get(k) != old.get(k) for k in ("reviewer", "review_date"))
        check("sample_metadata_" + key,
              (row.get("reviewer") in INDEPENDENT_REVIEWERS and row.get("review_date") == at[:10]
               and old["status"] == "已复核等价" and expected_status == "简化近似") if meta_changed
              else not (old["status"] == "已复核等价" and row["status"] == "简化近似"))
        if confirmed:
            check("first_application_includes_self_confirmed_" + key, change is not None and any(
                r["reviewer"] == REVIEWER and r["confirmed_difference"] and r["difference"] == s["finding"]
                for r in change["reviews"]))
        elif not s["difference_confirmed_by_source"]:
            check("gaps_and_static_not_applied_as_self_confirmation_" + key, not any(
                r["reviewer"] == REVIEWER and r["confirmed_difference"] for r in (change or {}).get("reviews", [])))
        entry = {"key": key, "source_id": s["source_id"], "category": s["category"], "id": s["id"],
                 "upstream_ref": s["upstream_ref"], "original_review": s,
                 "corrected_support": corrected_support, "own_support_for_withdrawal": bool(confirmed),
                 "other_independent_same_key_support": other_support, "expected_status": expected_status,
                 "application_before": old, "first_application_after": change["after"] if change else old,
                 "first_application_reviews": change["reviews"] if change else [], "current_at_review": row}
        if returned:
            entry["returned_confirmation"] = {"type": returned[0], "reason": returned[1],
                                                "native_refs": returned[2], "next_check": returned[3]}
        if code == ("L25", "_guanmodeskill"):
            entry["corrected_route_and_difference"] = GUAN_CORRECTION
        entries.append(entry)

    # Common dated provenance notes may repeat; per-key substantive findings may
    # not be replaced by that note. Separate unchanged old wording from new text.
    groups = collections.defaultdict(list)
    for e in entries:
        if e["current_at_review"].get("difference") != e["application_before"].get("difference"):
            groups[e["current_at_review"].get("difference")].append(e["key"])
    new_duplicates = [{"difference": text, "keys": keys} for text, keys in groups.items() if len(keys) > 1]
    check("no_repeated_new_substantive_difference", not new_duplicates, new_duplicates)
    check("original_79_findings_are_non_template_distinct_text",
          len({s["finding"] for s in samples if s["difference_confirmed_by_source"]}) == 79)
    check("current_counts_match_actual_rows", dict(collections.Counter(row["status"] for row in inventory["items"].values()))
          == {k: v for k, v in inventory["counts"].items() if v})
    check("no_inventory_mutation_during_review", INVENTORY.read_bytes() == inventory_bytes)

    source_hashes = {}
    for e in entries:
        refs = e["original_review"]["native_refs"] + e["original_review"]["upstream_refs_reviewed"]
        refs += e.get("returned_confirmation", {}).get("native_refs", [])
        refs += e.get("corrected_route_and_difference", {}).get("refs", [])
        for ref in refs:
            name = ref.rsplit(":", 1)[0]
            path = (ROOT.parent if name.startswith(("archive/", "noname-next/")) else ROOT) / name
            if path.is_file():
                source_hashes[name] = digest(path.read_bytes())

    summary = {
        "reviewed_samples": len(entries),
        "original_confirmed": sum(s["difference_confirmed_by_source"] for s in samples),
        "retained_own_confirmed": sum(e["own_support_for_withdrawal"] for e in entries),
        "returned_own_confirmations": sum("returned_confirmation" in e for e in entries),
        "corrected_support_counts": dict(collections.Counter(e["corrected_support"] for e in entries)),
        "current_sample_status_counts": dict(collections.Counter(e["current_at_review"]["status"] for e in entries)),
        "first_application_record_count": len(changes),
        "first_application_withdrawals": application["status_withdrawals"],
        "current_global_withdrawals_from_HEAD": sum(before["items"][k]["status"] != r["status"] for k, r in inventory["items"].items()),
        "new_equivalence_signoffs_by_this_review": 0,
        "checks_passed": sum(c["passed"] for c in checks), "checks_failed": sum(not c["passed"] for c in checks),
    }
    report = {
        "at": at, "source_head": head, "tracked_dirty": bool(subprocess.check_output(
            ["git", "diff", "--name-only"], cwd=ROOT, text=True).strip()),
        "reviewer": REVIEWER, "scope": "12来源141原key的应用复审；不审U01/juxian及样本外规则",
        "versions": application["versions"],
        "inputs": inputs, "HEAD_inventory_sha256": digest(before_bytes),
        "first_application_before_sha256": application["before_sha256"],
        "first_application_after_sha256": application["after_sha256"],
        "current_inventory_sha256": digest(inventory_bytes), "summary": summary,
        "sampling": sampling, "correction_chain": corrections,
        "script_review": {
            "executed": False,
            "findings": "others只取difference_confirmed_by_source=true；nonrule两类消费链未闭合建议不单独降级。"
                        "脚本不验证审查语义，首次79条中的11条因此需应用复审更正。确认者的具体difference优先于待核验文字。",
            "metadata": "eq转approx才新填独立reviewer/date；已有approx保留历史空值；撤回误判恢复历史字段，不视为新等价签署。",
        },
        "checks": checks, "new_substantive_duplicates": new_duplicates,
        "source_file_sha256_at_review": source_hashes,
        "samples": entries,
        "unverified": ["本reviewer未运行Cargo、浏览器、真实对局、保存恢复或配置产品测试。",
                       "68条保留的是具体静态差异对状态应用的支持，不是修复通过或整技能等价签署。",
                       "47条待核验、26条静态未见差异不等于已复核等价；保留的历史状态属于原声明。",
                       "12来源765项中624个样本外key以及其余来源不在本复审语义范围；全局哈希/改动链检查不能认证这些项。",
                       "其他独立审查3条同key撤回只检查来源和应用条件；本reviewer不据此自称完成该分支运行对照。",
                       "juxian修复、门禁、发布和exact-head CI由主agent处理，不在本复审范围。"],
    }
    output = HERE / ("application-review-" + label + ".json")
    with output.open("x") as stream:
        json.dump(report, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    print(json.dumps({"report": str(output.relative_to(ROOT)), "summary": summary,
                      "failed_checks": [c for c in checks if not c["passed"]]}, ensure_ascii=False, indent=2))
    return int(any(not c["passed"] for c in checks))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", required=True)
    args = parser.parse_args()
    raise SystemExit(main(args.label))
