# 2026-10-17 地藏王 大愿/无量建局摸 3：登记差异收口（T11-BOSS-DIZANG-DAYUAN / -WULIANG-GAMEDRAW）

**核验日期**：2026-10-17。**性质**：内容面差异对照与登记（无代码/版本/golden 变化；规则 v295 / 存档 v80）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。源码摘录：[sources.txt](sources.txt)。
前置：[地藏王 v274](../2026-10-14-t11-boss-dizangwang-v274/README.md)（不佛/无量准备回复/无量结束摸 2/谛听已落）。

## 1. `T11-BOSS-DIZANG-DAYUAN`（大愿：判定改写机）

- **上游**：`boss_dayuan`（`global: "judge"`，`lastDo: true`，`direct: true`）。`content` 两段 `chooseControl`：先由 1..13 逐点算 `(judge(new) - judge(card)) * attitude` 取最优**点数**，再由四花色取最优**花色**（可选 `cancel2` 放弃），写入 `trigger.fixedResult.number/suit/color`——是带 **AI 态度（`get.attitude`）优化** 的判定改写机。
- **原生**：判定改写机（天道/鬼道族）为**确定性**策略（读牌、写 `fixedResult`），无 `get.attitude`/`chooseControl` 态度选面承载；地藏王条目（v274）**只列已转换技**（不佛/无量/谛听），未列大愿。
- **处置**：**登记差异（不实现）**。理由：原生判定改写以确定性策略承载，AI 态度面是宿主策略层而非内核规则面；若未来引入态度模型或需要此改判，按判定改写扩展面另批立项。规则基准已记「大愿=判定改写机 AI 态度面登记不落」。

## 2. `T11-BOSS-DIZANG-WULIANG-GAMEDRAW`（无量：建局摸 3）

- **上游**：`boss_wuliang` 的 `gameDrawAfter`（global）与 `enterGame`（player）触发 `draw(3)`；`phaseZhunbeiBegin`（hp<3）回复至 3；`phaseJieshuBegin` 摸 2。
- **原生**：准备阶段回复至 3 与结束阶段摸 2 已落（v274，`kernel.rs:2194/2957`）；**建局摸 3 未落**——原生 Boss 建局在 `Engine::new_boss_stage` 按关卡配置发牌（分列发牌数），武将技随后由装配授予（`reinit_boss_seat`/`debug_set_general_def` → `apply_general_assembly`），**不存在**与上游 `gameDrawAfter`/`enterGame` 对应的「建局帧」；产品路径亦未接线 Boss 关卡。
- **处置**：**登记差异（不实现）**。理由：需要「建局/入场帧」原语才能承载；纯为标准场景不可达的 Boss 建局面，无现状内容依赖。有开局帧/关卡起始钩需求时另批立项（届时若改语义递增规则版本并重录 golden）。

## 影响

无代码、版本、golden 变化。地藏王「大愿」「无量建局摸 3」由「登记不落/登记未落」转为**已登记差异**（登记表 resolved，附本证据）；规则基准与 v274 证据的口径一致，不新增语义声明。
