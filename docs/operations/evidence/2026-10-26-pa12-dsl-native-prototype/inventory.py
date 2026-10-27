#!/usr/bin/env python3
"""PA12④ 盘点：ported .skill 的触发事件/滤镜/效果三面对照原生桥接承载面，并给出包级近似就绪率。
在仓库根运行：python3 -B docs/operations/evidence/2026-10-26-pa12-dsl-native-prototype/inventory.py
"""
import re, pathlib, collections, json
rb = open("crates/engine-native/src/rvm_bridge.rs").read()
# 交替名（"a" | "b" => Some(..)）逐名计入：抓整行后解析全部引号名
filters_exact = set()
for _line in rb.splitlines():
    if "=> Some(VmFilterPolicy" in _line:
        for _m in re.finditer(r'"([a-z0-9_]+)"', _line):
            filters_exact.add(_m.group(1))
param_prefixes = ["owner_is_target_hp_below_", "owner_source_play_card_count_eq_", "owner_skill_added_self_", "owner_hp_below_"]
loader = open("crates/content-native-loader/src/lib.rs").read()
events_mapped = set(re.findall(r'"([a-z_]+)" =>', loader.split("let event = match t.event.as_str()")[1].split("other =>")[0]))
acc = open("crates/engine-native/src/rvm_ops.rs").read()
accept_block = acc.split("pub fn accept_effect")[1].split("other =>")[0]
allowed_variants = set(re.findall(r'EffectRequest::([A-Za-z]+)', accept_block))
basic = {"draw_cards":"DrawCards","deal_damage":"DealDamage","recover_hp":"RecoverHp","change_max_hp":"ChangeMaxHp","add_mark":"AddMark","remove_mark":"RemoveMark",
         "discard_hand":"DiscardFromHand","discard_hand_equip":"DiscardFromHandEquip","lose_hp":"DealDamage","add_skill":"AddSkill","remove_skill":"RemoveSkill",
         "enter_dying":"EnterDying","take_hand":"TakeFromHand","move_card":"MoveCards","reveal_card":"RevealCards","set_turn_state":"SetTurnState","kill_player":"KillPlayer"}
files = sorted(pathlib.Path("content/original/ported").rglob("*.skill"))
ev_counter = collections.Counter(); fl_counter = collections.Counter(); eff_counter = collections.Counter()
skills_total = 0; skills_native_ready = 0
per_pkg_ready = collections.Counter(); per_pkg_total = collections.Counter()
for f in files:
    pkg = f.parent.name
    src = f.read_text(errors="ignore")
    skills = re.findall(r'^\s*skill\s+([a-zA-Z0-9_]+)\s*\(', src, re.M)
    trig = re.findall(r'trigger\s+([a-z_]+)\s+([a-z0-9_]+)', src)
    effs = re.findall(r'effect\s+([a-z0-9_]+)\(', src)
    has_active = "active " in src or "active\n" in src
    for ev, fl in trig:
        ev_counter[ev] += 1
        ok_fl = fl in filters_exact or any(fl.startswith(p) for p in param_prefixes)
        fl_counter["native_filter" if ok_fl else "deferred:"+fl] += 1
    for e in effs:
        eff_counter[("basic:"+basic[e]) if e in basic else ("host-resolve:"+e)] += 1
    ev_ok = all(ev in events_mapped for ev,_ in trig)
    fl_ok = all((fl in filters_exact or any(fl.startswith(p) for p in param_prefixes)) for _,fl in trig)
    eff_ok = all(e in basic for e in effs)
    n = len(skills) or 1
    per_pkg_total[pkg] += n; skills_total += n
    if trig and ev_ok and fl_ok and eff_ok and not has_active:
        per_pkg_ready[pkg] += n; skills_native_ready += n
print("=== 事件名（DSL trigger）计数 vs 装载器映射 ===")
for ev,c in ev_counter.most_common():
    print(f"  {ev:22s} {c:5d}  {'mapped' if ev in events_mapped else 'UNMAPPED→拒装'}")
print("\n=== 滤镜：原生精确/参数化命中 vs 推迟（前 12 推迟名）===")
print("  native_filter hits:", fl_counter["native_filter"])
deferred = [(k[9:],v) for k,v in fl_counter.items() if k.startswith("deferred:")]
print("  deferred total:", sum(v for _,v in deferred), "distinct:", len(deferred))
for k,v in sorted(deferred, key=lambda kv:-kv[1])[:12]: print(f"    {k:60s} {v}")
print("\n=== 效果：基础变体（原生 accept_effect 已允许）vs 宿主解析族（需逐个移植）===")
b = sum(v for k,v in eff_counter.items() if k.startswith("basic:")); h = sum(v for k,v in eff_counter.items() if k.startswith("host-resolve:"))
print(f"  basic={b}  host-resolve={h}  distinct host-resolve names={len([k for k in eff_counter if k.startswith('host-resolve:')])}")
for k,v in eff_counter.most_common(10): print(f"    {k:60s} {v}")
print(f"\n=== 文件级「全原生承载」近似：{skills_native_ready}/{skills_total} 技能（事件已映射∧滤镜命中∧效果全基础∧无主动技）===")
rows = sorted(per_pkg_total.items(), key=lambda kv:-kv[1])
print("  包\t技能数\t近似可直接原生")
for pkg,tot in rows[:24]: print(f"  {pkg}\t{tot}\t{per_pkg_ready[pkg]}")
json.dump({"events":dict(ev_counter),"deferred_filters":dict(deferred),"effects":dict(eff_counter),"per_pkg":{k:[per_pkg_total[k],per_pkg_ready[k]] for k in per_pkg_total}}, open("inventory.json","w"), ensure_ascii=False, indent=1)
