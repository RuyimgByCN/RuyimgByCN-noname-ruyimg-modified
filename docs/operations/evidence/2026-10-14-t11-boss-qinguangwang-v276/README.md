# T11 秦广王片（规则 v276，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（楚江王批 be993802 后续片）。上游依据：`boss_qinguangwang`（秦广王·蒋歆/male/qun/**hp3**/skills `[boss_panguan, boss_juhun, boss_wangxiang, boss_newhuanren]`，apps/core/mode/boss.js:1295）；`boss_panguan`@3320（mod targetEnabled delay）/ `boss_juhun`@3330 / `boss_wangxiang`@3345；显示名 boss.js:11238–11242（判官/拘魂/望乡——无显示名勘误）。

## 三技语义与原生落法

- **判官**（查询型单面）：延时锦囊（乐/闪电）目标不可指定（PlayCard 拒绝位 `IllegalTargets`，帷幕/热闷同位；上游 mod targetEnabled delay→目标不可指定近似登记；兵粮寸断不在原生目录；闪电无指向面按置入者自持天然不适用）。
- **拘魂**（查询型单面，End-begin 臂炼狱同位）：结束阶段——座位序首个其他翻面（上游 `["turnOver","link"] randomGet`→**恒定翻面+首个**确定性近似登记；风雪刃翻面写同款 `face_down`+`TurnOvered`）。
- **望乡**（Death 触发遗言机，`MachineSnapshot::BossWangxiang` 新变体 tag81/机哈希 81/a0 变体 79，忘魂同位直写一步机）：你死亡时，全部有装备的其他角色装备全弃（绝红拆装管线同款 `revoke_equip_skills`+入弃堆；上游 countPlayer discard 全装备同口径）。
- **还人（boss_newhuanren）：登记不落**——关卡流程面（shidianyanluo_level 换将链/changeBoss/addFellow 孟婆乱入/禁将表），隐藏链同族；其孟婆乱入分支与 `_status.shidianyanluo_mengpo` 模式旗同属 boss 模式扩展批（与忘魂登记的旗一致）。

条目 `("秦广王", Qun, &[判官, 拘魂, 望乡], 3, Male)`（BOSS_GENERALS[54]）；`STANDARD_RULESET_VERSION` 275→**276**（存档不变 v63）。FROZEN 尾追加「判官」「拘魂」「望乡」（177–179 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_qinguangwang_panguan_delay_reject`（乐拒/顺手可指定）、`boss_qinguangwang_juhun_end_flip_first`（1 号 End→0 号翻面）、`boss_qinguangwang_wangxiang_death_strip_equips`（真死→0/2 号装备全弃入弃堆+往返）——3/3 绿（[日志](qinguangwang-targeted.log)） |
| 版本探针 | 276→275 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v276-actuals.txt)），重录依据注释链追加 v276 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **653/0/0**（650→653，新增 3 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 已知边界与登记

- 拘魂恒定翻面+首个（上游随机二选一+随机目标）；判官目标不可指定近似；望乡直接拆装无事件面（绝红同款登记）。
- **坑位**：`revoke_equip_skills` 为 kernel-local（`crate::kernel::`，非 content::host）；乐的牌名键为 `le`（kind 键 `lebu` 只用于 kind 映射）。
- **十殿阎罗 12/13**：五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王/平等王/都市王/地藏王/楚江王/秦广王✓。余：Shikieiki[映姬 1 技]（初判完）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。
