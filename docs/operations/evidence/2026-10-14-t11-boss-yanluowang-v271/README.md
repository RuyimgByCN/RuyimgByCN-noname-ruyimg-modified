# T11 阎罗王片（规则 v271，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（孟婆批 e0afb66a 后续片，无独立领取锁——同会话连续批）。上游依据：`boss_yanluowang`（阎罗王·包拯/male/qun/**hp4**/skills `[boss_tiemian, boss_zhadao, boss_zhuxin]`，apps/core/mode/boss.js:1336）；`boss_tiemian`@3675（inherit renwang_skill）/ `boss_zhadao`@3692（inherit qinggang_skill）/ `boss_zhuxin`@3696；显示名 boss.js:11266–11271（铁面/铡刀/诛心——无显示名勘误）。

## 三技语义与原生落法

- **铁面**（查询型单面）：sha 机仁王块扩展——持有者防具槽空（`equip_slot()==Armor` 的卡为零）时黑杀无效（视为装备仁王盾）；装防具后不生效（八卦判定照常）；在 `!qinggang_pierce` 门内，青釭/铡刀穿透天然覆盖。
- **铡刀**（查询型单面）：sha 机青釭穿透链扩展——来源持有铡刀即穿透（上游 inherit qinggang_skill 同口径；八卦判定与仁王/铁面免疫均被 `qinggang_pierce` 门旁路）。
- **诛心**（Death 触发遗言机，`MachineSnapshot::BossZhuxin` 新变体 tag77/机哈希 77/a0 变体 75，热恼同构 emitted 防重入）：你死亡时，座位序首个最低体力的其他存活受 2 点伤害（上游 chooseTarget 最低体力池自选→**座位序首个最低体力**确定性近似登记；严格更小才替换=并列取首）。

条目 `("阎罗王", Qun, &[铁面, 铡刀, 诛心], 4, Male)`（BOSS_GENERALS[49]）；`STANDARD_RULESET_VERSION` 270→**271**（存档不变 v63——机器变体不触哈希）。FROZEN 尾追加「铁面」「铡刀」「诛心」（163–165 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_yanluowang_tiemian_black_sha_block`（槽空黑杀挡下[无伤害/无八卦判定/杀入弃堆]+装八卦后铁面被抑制[ArmorJudged 发生]）、`boss_yanluowang_zhadao_pierce_renwang`（无铡刀来源照常进八卦判定+铡刀来源无判定且 Pass 后受伤）、`boss_yanluowang_zhuxin_death_min_hp`（真死→2/3 号并列最低取 2 号受 2+0 号不受影响+往返）——3/3 绿（[日志](yanluowang-targeted.log)） |
| 版本探针 | 271→270 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v271-actuals.txt)），重录依据注释链追加 v271 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **636/0/0**（633→636，新增 3 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 已知边界与登记

- 诛心最低体力→座位序首个确定性近似（已登记差异族）。
- 铁面条件在杀结算点查询（上游 init/hasEmptySlot 动态等效）；防具槽口径=`equip_slot()==Armor` 全族（八卦/锁链甲/面具/仁王等）。
- **十殿阎罗 9/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王✓（+秦广王待查）。余：都市王（热闷/炽焚/祸刑）/平等王（锁足/阿鼻/平等）/地藏王（四技，初判完）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。

## 坑位实录

- 测试布阵 `debug_place` 只入手牌不装备——装备需 `state_core_mut` 直写装备槽（`equip_card` 助手）；仁王盾全牌堆单张，铡刀对照改用八卦阵两侧（判定发生/不发生=穿透语义）。
- 出牌阶段限一次杀：同回合第二杀被拒（CardLimit）——第二杀换座位回合；0→2 距离超 1（4 人环 0-3 相邻）。
