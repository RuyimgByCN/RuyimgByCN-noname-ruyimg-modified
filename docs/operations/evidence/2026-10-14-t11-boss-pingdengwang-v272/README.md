# T11 平等王片（规则 v272，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（阎罗王批 9da6ce39 后续片）。上游依据：`boss_pingdengwang`（平等王/male/qun/**hp4**/skills `[boss_suozu, boss_abi, boss_pingdeng]`，apps/core/mode/boss.js:1376）；`boss_suozu`@3905 / `boss_abi`@3914 / `boss_pingdeng`@3928；显示名 boss.js:11298–11302（锁足/阿鼻/平等——无显示名勘误）。

## 三技语义与原生落法

- **锁足**（查询型单面，Prepare-begin 臂）：持有者准备阶段——**全部其他存活**横置（上游 content link 同口径，不分阵营；索链甲/星囚 `linked` 直写同款 + `Linked` 日志）。
- **阿鼻**（查询型单面，Damage commit 尾臂，远策同位）：受到**有来源**的伤害后——来源受 1 点火焰伤害（上游 randomGet fire/thunder→**恒定火**确定性近似登记；新伤害无来源→天然无递归；无来源伤害不触发）。
- **平等**（Death 触发遗言机，`MachineSnapshot::BossPingdeng` 新变体 tag78/机哈希 78/a0 变体 76，诛心同构两波）：你死亡时——**最高体力**其他存活（座位序首个）先受 2 点火焰伤害、**重算池后**再受 1 点火焰伤害（上游两波 randomGet×lib.linked randomGet→座位序首个+恒定火确定性近似登记；emitted 波次防重入）。

条目 `("平等王", Qun, &[锁足, 阿鼻, 平等], 4, Male)`（BOSS_GENERALS[50]）；`STANDARD_RULESET_VERSION` 271→**272**（存档不变 v63）。FROZEN 尾追加「锁足」「阿鼻」「平等」（166–168 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_pingdengwang_suozu_link_all_others`（准备阶段 0/2/3 全横置+本人不横置+日志×3）、`boss_pingdengwang_abi_source_fire_no_source_skip`（有来源→0 号-1 火伤+无来源不触发）、`boss_pingdengwang_pingdeng_death_two_wave_fire`（无来源致死避开阿鼻→波一 0 号-2/波二重算 2 号-1/3 号不波及+往返）——3/3 绿（[日志](pingdengwang-targeted.log)） |
| 版本探针 | 272→271 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v272-actuals.txt)），重录依据注释链追加 v272 条目；`replay_scenarios` 联动守护 2/2 |
| 全量 | **639/0/0**（636→639，新增 3 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 已知边界与登记

- 阿鼻恒定火、平等座位序首个最高体力+恒定火（已登记差异族）；平等波二重算池=上游 step1 重读同口径。
- **坑位**：锁足臂初插 Play 分支（星囚同区）——准备阶段技须插 Prepare-begin 分支（往事/苦酒区），本地编译绿不拦语义错位，靠测试阶段锚定发现前移正。
- **十殿阎罗 10/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王/平等王✓（+秦广王待查）。余：都市王（热闷/炽焚/祸刑）/地藏王（不覆/无量/大愿/谛听，含判定改写机）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。
