# P2/N08 批③-b 第二把：麒麟弓（规则 v66，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第二把：麒麟弓（♥5，射程 5，杀造成伤害时
来源可弃目标一张坐骑）。上游对照 `card/standard.js`（qilin 条目：
attackFrom −4 = 射程 5）。前置：
[第一把青釭剑](../2026-10-07-n08-qinggang-3b1/README.md)。

## 本批落地

- **命名武器牌种 Qilin（编码 96）**：equip1 武器、射程 5——**命名武器
  射程面首次接入**：`CardKind::named_weapon_range()`（青釭 2/麒麟 5，
  后续每把在此登记）+ `attack_range` 与泛型 Weapon 的 CardDef.range
  并行取 max。
- **特效接线（ShaResolve 双伤害路径）**：杀造成伤害结算前，来源装备
  麒麟弓且目标装备区有坐骑 → 询问来源（`qilin_discard_horse`，可选
  可放弃，空应答=放弃）→ 弃置所选坐骑 → 照常结算伤害。
- 声明表 115→**116 张**；`STANDARD_RULESET_VERSION` 65→**66**；
  **14 golden 重录**；Sha 快照追加 `asking_qilin`/`qilin_horse` 字段
  （恢复期弃马询问状态入快照）。

## 测试（`qilin_bow_discards_target_horse_on_sha_damage`）

1. 弃马路径：目标装 +1 马 → 杀 → 询问含该马 → 弃置（入弃牌堆）→
   伤害照常。
2. 放弃路径：目标装 −1 马 → 杀 → 放弃（空应答）→ 马保留、伤害照常
   （跨回合推进避开 ShaLimit）。

## 验收口径（分开陈述）

- **代码已实现**：麒麟弓牌种/声明表/射程面/弃马询问（双路径）。
- **回归已加入**：`qilin_bow_discards_target_horse_on_sha_damage`。
- **golden 已重录**：14 场景全量（v66）。
- **全量通过**：engine-native **374/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。

## 剩余

批③-b 其余七把（寒冰→连弩→贯石斧→青龙→丈八→雌雄→方天）；模式对齐；
主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
