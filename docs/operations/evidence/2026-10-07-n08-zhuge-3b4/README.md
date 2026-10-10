# P2/N08 批③-b 第四把：诸葛连弩（规则 v68，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第四把：诸葛连弩（♣1/♦1 ×2，射程 1，
装备时出杀不限次数）。上游对照 `card/standard.js`（zhuge 条目：无
distance.attackFrom = 射程 1；特效在 zhuge_skill）。前置：
[第三把寒冰剑](../2026-10-07-n08-hanbing-3b3/README.md)。

## 本批落地

- **命名武器牌种 Zhuge（编码 98）**：equip1 武器、射程 1
  （named_weapon_range 登记 1）。
- **特效接线（PlayCard Sha 臂）**：ShaLimit 检查的连弩装备豁免——
  装备诸葛连弩时 `sha_used_play_phase` 限制不生效（与咆哮/元宝肉同
  一检查点的锁定豁免口径）。
- 声明表 117→**119 张**（♣1/♦1 两张）；`STANDARD_RULESET_VERSION`
  67→**68**；**14 golden 重录**。

## 测试（`zhuge_crossbow_unlimited_sha`）

- 基线：无连弩时同回合第二张杀被 ShaLimit 拒绝（既有行为）。
- 装备连弩后同回合连出两张杀均被接受，目标各受 1 伤。

## 验收口径（分开陈述）

- **代码已实现**：诸葛连弩牌种/声明表/杀不限次豁免。
- **回归已加入**：`zhuge_crossbow_unlimited_sha`。
- **golden 已重录**：14 场景全量（v68）。
- **全量通过**：engine-native **376/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。

## 剩余

批③-b 其余五把（贯石斧→青龙→丈八→雌雄→方天）；模式对齐；主稿其余
待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（11 文件，相对 noname-next 根）。
