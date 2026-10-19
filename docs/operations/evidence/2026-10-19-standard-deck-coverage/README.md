# 标准牌组构成核对（2026-10-19）——**首版结论已撤回**

## ⚠ 撤回说明

本证据目录**首版**（同日）曾断言「**9 种核心标准牌在牌组中无实例**」（过河拆桥/顺手牵羊/决斗/南蛮/万箭/
无懈可击/乐不思蜀/闪电/兵粮寸断）。**该结论错误，现撤回**。

**根因**：首版用**手写的 `CardKind` 变体名**（`Guohe`/`Shunshou`/`Nanman`/`Wanjian`/`Wuxie`/`LeBu`/
`Bingliang` 等）去比对目录，而**真实枚举变体名不同**（见下表）；且**目录名**又与变体名不同
（`SavageAttack` 的**目录名是 `savage`**，`catalog.rs:934` 的 `"nanman"` 是 **archive 键**映射）。

## 修正后的实际分布（**从实例反查 kind，不手写名字**）

```
124 张实例、34 个不同 kind
Sha 30 / Shan 15 / Tao 8 / Dismantle 6 / Snatch 5 / Chain 4 / FireSha 4 / HorsePlus 4 /
Nullify 4 / Wuzhong 4 / Duel 3 / Le 3 / SavageAttack 3 / Weapon 3 / Armor 2 / HorseMinus 2 /
Jiedao 2 / Jiu 2 / LeiSha 2 / ShanDian 2 / Wugu 2 / Zhuge 2 / ArcheryAttack 1 / Bing 1 /
Cixiong 1 / Fangtian 1 / Guanshi 1 / Hanbing 1 / Qilin 1 / Qinggang 1 / Qinglong 1 /
Renwang 1 / Taoyuan 1 / Zhangba 1
```

**关键锦囊的目录名**：`SavageAttack → "savage"`、`ArcheryAttack → "archery"`、`Duel → "duel"`、
`Dismantle → "dismantle"`、`Snatch → "snatch"`、`Nullify → "nullify"`、`Le → "le"`、
`ShanDian → "shandian"`（另有 `"shandian_safe"`）、`Bing → "bing"`。

⇒ **9 种核心标准牌全部在牌组中**（数量分别为 3/1/3/6/5/4/3/2/1），**首版「缺失」为误报**。

## 教训（**第 97 条方法学**）

**「按名字比对」必须用「从数据反查的名字」**：手写 `CardKind` 变体名、手写目录名、archive 键三者**互不相同**
⇒ 任何「按名字找东西」的清单必须**由代码反查生成**（如本版从 `card_kind(i)` 反查），
**不能手写**。本会话此前已有多次同类失误（单侧归一、前缀漏匹配等），**这是第 7 次**。

## 结论

**牌组构成无缺陷**；目录构成由既有 `catalog_inventory`（`standard_catalog_matches_declared_inventory`）守护。
本用例保留为**分布快照**（不设硬断言）。
