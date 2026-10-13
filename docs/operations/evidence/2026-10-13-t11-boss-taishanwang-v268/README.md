# T11 泰山王片（规则 v268，2026-10-13）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 17ab4a51 延续）。上游依据：`boss_taishanwang`（泰山王·董和/male/qun/hp4/skills `[boss_fudu, boss_kujiu, boss_renao]`，apps/core/mode/boss.js:1356）；`boss_fudu`@3768 / `boss_kujiu`@3784 / `boss_renao`@3799。

**显示名勘误**：`boss_fudu`=**服毒**（勘验存量误记「复读」）、`boss_kujiu`=**苦酒**（误记「苦久」）、`boss_renao`=热恼——boss.js 翻译段 11282–11287。

## 三技语义与原生落法

- **服毒**（Noop 查询型，内联臂）：其他角色使用【桃】时，随机另一名其他角色失去 1 体力——`commit_card_used` 内联臂（邪酬同位；kind 判定含 `use_as`；候选=全体减使用者减持有者，randomGet→座位序首个确定性近似登记；`players>2` 上游门保留）。
- **苦酒**（Noop 查询型，内联臂）：其他角色的准备阶段，其失去 1 体力然后视为使用一张【酒】——Judge begin 臂位 global 扫持有者（雷舟臂同位扩展）；jiu_active 置位、不占 jiu_used（上游 `useCard jiu` 同口径）；无 useCard 事件面=登记差异。
- **热恼**（Death 触发 SkillDef，雷诛同位遗言面）：你死亡时随机一名其他角色受 3 火伤——`BossRenao { owner, emitted }` 单发机（emit+Yield，可挂起，入重建表；randomGet→首个近似登记）。

条目 `("泰山王", Qun, &[服毒, 苦酒, 热恼], 4, Male)`（BOSS_GENERALS[46]）；`STANDARD_RULESET_VERSION` 267→268。

## 接线触点

standard.rs（常量+Renao 机+def×3+条目+版本）、kernel.rs（服毒 CardUsed 臂+苦酒 Judge-begin global 臂）、snap.rs（变体/check_pid/恢复 None/重建臂）、archive.rs（tag 75）、engine.rs（push tag 75）、a0_tag_registry.rs（样例+index 73）、lib.rs FROZEN 尾追加「服毒,苦酒,热恼」（154–156 名）。存档编码追加 BossRenao=75（存档版本不变 v61）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_taishanwang_fudu_kujiu_arms`（2 号准备阶段苦酒失 1+jiu_active；2 号用桃自身回 1 且 0 号失 1——候选座位序首个）、`boss_taishanwang_renao_death_fire`（真死 chongzheng=0，首个其他存活恰 -3 火伤+存档往返）——2/2 绿 |
| 版本探针 | 普通批口径：268→267 回退复跑 golden **全绿**、268 复跑**全差 14 条**=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14，重录依据注释链追加 v268 条目 |
| 全量 | **627/0/0**（625→627，新增 2 测） |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 测试坑位（本批新录）

- **first=0 时 0 号首回合准备阶段在 `new_boss_stage` 建局（OpeningPlan）内已发生**——布阵授技在其后，苦酒类「他人准备阶段」技无法在 0 号首回合触发；测试改走后续座位的准备阶段（2 号）。

## 已知边界与登记

- 服毒/苦酒/热恼随机→座位序首个确定性近似（既登记差异族）；苦酒视为用酒无 useCard 事件面（jiu_active 直写）；热恼死亡被取消仍已触发（死亡前置窗口族登记差异）。
- 十殿阎罗进度 6/13：楚江王/五官王/宋帝王/卞城王/泰山王✓；余阎罗王/都市王/平等王/转轮王/孟婆/地藏王待落（勘验完备余：转轮王）。
