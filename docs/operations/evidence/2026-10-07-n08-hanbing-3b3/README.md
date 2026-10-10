# P2/N08 批③-b 第三把：寒冰剑（规则 v67，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第三把：寒冰剑（♠2，射程 2，杀造成伤害时
可防止伤害改为弃置目标两张牌）。上游对照 `card/standard.js`
（hanbing 条目：attackFrom −1 = 射程 2；特效 hanbing_skill）。前置：
[第二把麒麟弓](../2026-10-07-n08-qilin-3b2/README.md)。

## 本批落地

- **命名武器牌种 Hanbing（编码 97）**：equip1 武器、射程 2
  （named_weapon_range 登记 2）。
- **特效接线（ShaResolve 双伤害路径）**：杀造成伤害结算前，来源装备
  寒冰剑且目标有牌（手牌/装备）→ 询问来源确认（`hanbing_prevent_confirm`，
  可选可放弃，min=0 空应答=放弃，放弃则照常伤害）→ 确认后防止伤害，
  弃置目标两张牌（依次手牌→装备，不足全弃）。
  **登记差异：上游为目标牌自选次序，原生自动依次**。
- Sha 快照追加 `asking_hanbing` 字段（穿齐 enc/dec/check/哈希/rebuild/
  a0 样本）。
- 声明表 116→**117 张**；`STANDARD_RULESET_VERSION` 66→**67**；
  **14 golden 重录**。

## 测试（`hanbing_prevents_damage_discards_two_cards`）

1. 防止路径：目标两张非闪手牌 → 杀 → 确认 → 伤害防止（hp 不变）+
   两张全弃。
2. 放弃路径：下回合再杀 → 放弃（空应答）→ 照常伤害（4→3）、目标手牌
   保留（目标持非闪牌避免出闪询问先行）。

## 验收口径（分开陈述）

- **代码已实现**：寒冰剑牌种/声明表/防伤弃牌语义（双路径）。
- **回归已加入**：`hanbing_prevents_damage_discards_two_cards`。
- **golden 已重录**：14 场景全量（v67）。
- **全量通过**：engine-native **375/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。
- 级联修复：国色测试推进循环改排空询问（v67 牌堆位移途中出现弃牌询问）。

## 剩余

批③-b 其余六把（连弩→贯石斧→青龙→丈八→雌雄→方天）；模式对齐；
主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
