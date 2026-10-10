# ADR 0605：上游 extension 扩展移植审计与补全

> 目标：`archive/upstream/noname/apps/core/extension` 下 8 个扩展全部移植到当前项目。

## 审计方法

- 「已移植」= `content/` 源码（.skill 技能名 + characters.json/npack.json 的 character_index
  name_key + skill_index name）。
- 上游角色按**前缀**精确识别（ddd_/wdb_/yxs_/hs_/gw_/swd_/ow_/diy_/gjqt_/pal_），
  排除 JS 关键字/配置噪音（首次粗提取的 716「缺失」经前缀过滤后实为噪音）。

## 8 扩展覆盖核对

| 扩展 | 上游角色 | 已移植 | 缺失 |
|---|---|---|---|
| 3D精选（ddd_）| 31 | **31** | 0 |
| 玩点论杀（wdb_）| 9 | **9** | 0 |
| 英雄杀（yxs_）| 48 | **48** | 0 |
| 杀海拾遗-hearth（hs_）| 110 | **110** | 0 |
| 杀海拾遗-swd（swd_）| 81 | **81** | 0 |
| 杀海拾遗-ow（ow_）| 24 | **24** | 0 |
| 杀海拾遗-yunchou（diy_）| 10 | **10** | 0 |
| 杀海拾遗-gujian（gjqt_）| 18 | **18** | 0 |
| 杀海拾遗-xianjian（pal_）| 36 | **36** | 0 |
| 杀海拾遗-gwent（gw_）| 53 | 53 角色 | 0（4 特殊卡元数据登记）|
| **合计** | **420** | **420** | **0** |

**关键事实**：8 个扩展的**角色与技能已全部移植**（3D精选/玩点论杀由 Codex/omp 的
ddd/wdb 原语批次完成；英雄杀/杀海拾遗由既有 yxs/shahai 包完成）。本审计确认 420/420
角色全覆盖，无真实角色缺口。

## 补全项（本轮）

### 欢乐卡牌 shoulijian（手里剑）
上游唯一未进 card_index 的卡牌——**自定义基本牌**（出牌阶段使用，目标弃一张装备牌或
受 1 伤害）。已登记翻译（metadata.json）+ 元数据（huanle_meta.skill 加 `skill shoulijian(2612562977) {}`
+ 结算说明）。**运行时结算需「自定义基本牌」引擎机制**（card_index 仅支持装备 kind；
基本牌仅 sha/tao/shan/jiu），判定为 ABI 642+ 后续（与国战锦囊同类）。

### 杀海拾遗 gwent 4 特殊卡
gw_wuyao/gw_lang/gw_dudayuanshuai1/2 是**特殊卡**（`derivation: gw_linjing`，昆特单位卡，
非角色）——已登记 translations.json，卡牌本体机制留后续。

### coin/boss/cardpile
`game.import("play")` 的 **UI 功能扩展**（硬币动画/占位卡堆/牌堆统计），非武将/卡牌规则
内容，**不属引擎移植范畴**，未移植（记录在案）。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过** ✅ |
| warnings | **59**（不变）✅ |

## 后续（ABI 642+，已登记 ABI-COORDINATION）

1. **自定义基本牌机制**：shoulijian（手里剑）+ 国战锦囊（gz_haolingtianxia 等 12 种）
   都需新增 CardKind + 使用结算管线。
2. gwent 特殊卡（gw_linjing 派生）结算。
