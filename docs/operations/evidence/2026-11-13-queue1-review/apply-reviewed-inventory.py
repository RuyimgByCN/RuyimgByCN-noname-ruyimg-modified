"""One-shot, evidence-backed withdrawal of specific old equivalence claims."""
import collections
import copy
import datetime
import hashlib
import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
ROOT = BASE.parents[3]
PATH = ROOT / "docs/migration/g0/inventory_status.json"
original_bytes = PATH.read_bytes()
original = json.loads(original_bytes)
data = copy.deepcopy(original)
items = data["items"]
u01 = json.loads((BASE / "independent/u01/findings.json").read_text())
others = json.loads((BASE / "independent/other-sources/findings.json").read_text())
nonrule = json.loads((BASE / "nonrule-audit/findings.json").read_text())
assert hashlib.sha256(original_bytes).hexdigest() == nonrule["inventory_sha256"]
at = datetime.datetime.now(datetime.timezone.utc).astimezone().isoformat()
date = at[:10]
changes = {}


def update(key, difference, reviewer, evidence, withdraw=False):
    row = items[key]
    old = original["items"][key]
    change = changes.setdefault(key, {"key": key, "before": old, "reviews": [], "withdrawing": False})
    change["reviews"].append({"reviewer": reviewer, "evidence": evidence, "difference": difference,
                              "confirmed_difference": withdraw})
    change["withdrawing"] |= withdraw
    if withdraw:
        assert row["status"] in ("已复核等价", "简化近似")
        row["status"] = "简化近似"
        if old["status"] == "已复核等价":
            row["reviewer"] = reviewer
            row["review_date"] = date


for finding in u01["findings"]:
    key = finding["inventory_key"]
    confirmed = finding["classification"] == "verified_source_difference"
    difference = finding["confirmed_difference"] if confirmed else "待核验：" + finding["difference"]
    if finding["skill"] == "dddjuxian":
        difference = ("已补gainPlayerCard真实选牌、规则RNG私有排列及挂起恢复；仍有出牌阶段次数用turn_stamp、"
                      "无他人he时预先禁止top、gainable/明置手牌分支及支付CardsLost子事件晚于获得询问等差异；"
                      "详见J01–J05。13分支的局部投影对照不签整技能等价。")
    update(key, difference, u01["reviewer"], "independent/u01/findings.json#" + finding["id"], confirmed)

for finding in u01["general_data_boundaries"]:
    update(finding["inventory_key"], "仅静态核对性别、势力、生命与技能组；" + "；".join(finding["boundaries"])
           + "。保留原状态，不是本轮整武将规则/资源等价签署。", u01["reviewer"], "independent/u01/characters.md")

for finding in others["samples"]:
    if finding["difference_confirmed_by_source"]:
        update(finding["key"], finding["finding"], others["reviewer"], "independent/other-sources/findings.json", True)

concrete = {"具体规则/公开信息分支差异", "只有计数/cue/flag，未接可见/可听效果", "菜单模型和可见交互未接入"}
pending = {"表现配置仅解析/未驱动显示", "规则/目录配置应用链未闭合"}
for finding in nonrule["items"]:
    withdraw = finding["status_action"] == "建议下调" and finding["difference_kind"] in concrete
    difference = finding["difference"]
    if finding["difference_kind"] in pending:
        difference = ("待核验消费链建议，单凭静态未找到调用不据此下调：" + difference)
    update(finding["key"], difference, "/root/repair_doc_refs", "nonrule-audit/findings.json", withdraw)

for key, change in changes.items():
    row = items[key]
    # Deterministic differences precede pending consumer-chain suggestions.
    reviews = sorted(change["reviews"], key=lambda r: not r["confirmed_difference"])
    text = "；".join(dict.fromkeys(r["difference"] for r in reviews))
    row["difference"] = text
    row["diff"] = text
    old_note = row.get("note")
    note = (f"{date} 队列1补课：" + "、".join(dict.fromkeys(r["reviewer"] for r in reviews))
            + "独立源码审查；具体记录 docs/operations/evidence/2026-11-13-queue1-review/。"
            + ("已确认差异，撤回/维持简化近似；尚未修复的分支继续待做。" if change["withdrawing"]
               else "保留历史状态及原复核人/日期，不是本轮新增等价签署。")
            + "静态审查与主agent的运行对照分列；本批新增等价签署0。")
    row["note"] = old_note + "\n" + note if old_note else note
    change["after"] = copy.deepcopy(row)

data["updated_at"] = date
counts = collections.Counter(row["status"] for row in items.values())
data["counts"] = {key: counts[key] for key in original["counts"]}
assert set(items) == set(original["items"]) and len(items) == data["total_items"] == 4424
withdrawn = [c for c in changes.values() if c["before"]["status"] != c["after"]["status"]]
assert len(withdrawn) == 226
assert all(c["before"]["status"] == "已复核等价" and c["after"]["status"] == "简化近似" for c in withdrawn)
assert all(items[k] == row for k, row in original["items"].items() if k not in changes)
serialized = (json.dumps(data, ensure_ascii=False, indent=2) + "\n").encode()
record = {"at": at, "source_head": u01["head"], "tracked_dirty": True,
          "versions": nonrule["source_versions"], "before_sha256": hashlib.sha256(original_bytes).hexdigest(),
          "after_sha256": hashlib.sha256(serialized).hexdigest(), "before_counts": original["counts"],
          "after_counts": data["counts"], "changed_records": len(changes), "status_withdrawals": len(withdrawn),
          "new_equivalence_signoffs": 0, "changes": list(changes.values())}
with (BASE / "inventory-application.json").open("x") as stream:
    json.dump(record, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
assert PATH.read_bytes() == original_bytes
PATH.write_bytes(serialized)
print(json.dumps({k: record[k] for k in ("changed_records", "status_withdrawals", "before_counts", "after_counts")}, ensure_ascii=False))
