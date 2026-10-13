# T11 庞统片（群雄乱入 Boss 专项，规则 v277，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（秦广王批 a8ccf18f 后续——群雄乱入 Boss 专项勘验）。上游依据：`boss_pangtong`（涅槃凤雏·庞统/male/**group shen**/**hp4**/skills `[boss_tianyu, qiwu, niepan, boss_yuhuo]`，apps/core/mode/boss.js:1540）；`boss_tianyu`@9899 / `qiwu`@10312 / `boss_yuhuo`@9884；显示名 boss.js:11059/11197/11199（栖梧/浴火/天狱——无显示名勘误；10803 为 Boss 形态名「涅槃凤雏」）。

## 两技语义与原生落法

- **天狱**（查询型单面，End-begin 臂炼狱同位）：结束阶段——自己已横置先解除（toggle），再将全部未横置的其他存活横置（上游 `boss_tianyu` link 同口径；索链甲/锁足 linked 直写同款；filter 门=自横置或存在未横置者，臂内幂等）。
- **栖梧**（查询型单面，CardUsed commit 臂娇孝同族）：使用梅花牌后若已受伤回复 1 点体力（上游 useCard club+hp<maxHp recover 同口径；机器直发 useCard 同覆盖）。

条目 `("庞统", Qun, &[天狱, 栖梧], 4, Male)`（BOSS_GENERALS[55]；**上游 group shen→群目录近似登记**）；`STANDARD_RULESET_VERSION` 276→**277**（存档不变 v63）。FROZEN 尾追加「天狱」「栖梧」（180–181 名）。

## 登记不落（两技）

- **涅槃（niepan）/浴火（boss_yuhuo）**：涅槃 def 不在 boss.js（外部包技——shenhua 包 `oldniepan`/offline 包 `ylygniepan`，原生目录无对应条目）；浴火=niepanAfter 触发授 kanpo/shenwei/zhuyu 三外部技+`bossinfo.loopType=1` 关卡面——**外部包技依赖，随外部包批/规则面迁移**。
- **诸葛亮（祭风卧龙）勘验登记不落**：`boss_zhugeliang`（male/group shen/**hp Infinity**/skills `[xiangxing, yueyin, fengqi, gaiming]`，boss.js:1518）——①hp Infinity 无原生承载（PlayerState.hp 为 i32，无限体力需专项建模决策）；②四技均为 character/offline 龙血玄黄包技能（`ylyg*` 族，offline_yanling.js:532 起）——**与 T9 龙血玄黄战斗内触发轨迹同源阻塞**（规则面迁移立项前置）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_pangtong_tianyu_end_link_all`（首 End 全场横置+自体保持+次轮幂等）、`boss_pangtong_qiwu_club_recover`（梅花顺手使用后 2→3 回复）——2/2 绿（[日志](pangtong-targeted.log)） |
| 版本探针 | 277→276 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v277-actuals.txt)），重录依据注释链追加 v277 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **655/0/0**（653→655，新增 2 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 已知边界与登记

- 天狱自横置解除面=上游 toggle 同口径（先解后全场横置）；栖梧事件面=娇孝臂同族（机器直发 useCard 同覆盖；虚拟牌实体花色按载体）。
- 庞统/诸葛亮两 Boss 至此处置完毕：庞统部分落包（2/4 技，涅槃/浴火随外部包批），诸葛亮整体登记不落（hp 模型+offline 包双阻塞）。**群雄乱入 Boss 专项收官**。
