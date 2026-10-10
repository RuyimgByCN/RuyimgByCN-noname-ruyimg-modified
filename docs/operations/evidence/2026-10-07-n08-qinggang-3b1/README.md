# P2/N08 批③-b 第一把：青釭剑（规则 v65，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第一把：青釭剑（♠6，射程 2，**锁定无视
目标防具**）。上游对照 `card/standard.js`（qinggang 条目：attackFrom −1
= 射程 2；特效在 qinggang_skill）。前置：
[批③-a](../2026-10-07-n08-count-align-3a/README.md)。

## 本批落地

- **命名武器牌种 Qinggang（编码 95）**：equip1 武器、射程 2（`range=2`
  直接挂 CardDef——首次以命名牌种（非泛型 Weapon）落位声明表）。
- **特效接线（ShaResolve 双检查点）**：来源装备青釭时 ① 八卦阵判定跳过
  （判定牌不消耗、无 ArmorJudged 日志）② 仁王盾黑杀免疫失效。
- 声明表 114→**115 张**；`STANDARD_RULESET_VERSION` 64→**65**；
  **14 golden 重录**；identity 阵营快照更新（v65 双种子均主公方）。

## 测试（`qinggang_pierces_armor`）

1. 基线：目标装八卦阵、黑判定 → 不免费闪避 → 受伤（ArmorJudged 日志在）。
2. 青釭穿透八卦：判定牌留 top 未消耗（无判定发生）→ 黑杀直接伤害。
3. 青釭穿透仁王盾：2 号装仁王盾 → 黑杀照常伤害。

测试侧修正：跨回合推进避开 ShaLimit；无闪目标改伤害落点断言（无出闪
询问时不挂起）。

## 验收口径（分开陈述）

- **代码已实现**：青釭牌种/声明表/穿透语义（双检查点）。
- **回归已加入**：`qinggang_pierces_armor`（基线/穿八卦/穿仁王盾三段）。
- **golden 已重录**：14 场景全量（v65）。
- **全量通过**：engine-native **373/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0；lib `-D warnings` 通过。

## 剩余

批③-b 其余八把（麒麟→寒冰→连弩→贯石斧→青龙→丈八→雌雄→方天）；
模式对齐；主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（12 文件，相对 noname-next 根）。
