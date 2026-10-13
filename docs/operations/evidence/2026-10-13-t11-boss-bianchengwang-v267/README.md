# T11 卞城王片（规则 v267，2026-10-13）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 f06b4532 延续）。上游依据：`boss_bianchengwang`（卞城王·毕元宾/male/qun/hp4/skills `[boss_leizhou, boss_leifu, boss_leizhu]`，apps/core/mode/boss.js:1346）；`boss_leizhou`@3723 / `boss_leifu`@3737 / `boss_leizhu`@3751。

**勘验记忆勘误**：会话勘验存量记「雷舟=Judge begin 随机 1 雷伤」有误——源码实为 **phaseZhunbeiBegin（准备阶段）**，本批以源码为准落在原生准备阶段锚点（Judge begin 臂位，公正/吸行同位）。

## 三技语义与原生落法

- **雷舟**（Noop 查询型）：准备阶段随机一名其他存活受 1 雷伤——Judge begin 内联臂（吸行同位去 linked 过滤去回复）；randomGet→座位序首个确定性近似登记。
- **雷斧**（Noop 查询型）：结束阶段随机一名其他存活横置态切换（上游 `link()` 非恒链同口径）——End begin 内联臂（血池/胜负同位），emit `EventData::Linked`（commit_linked 翻转写回）；randomGet→首个确定性近似登记。
- **雷诛**（Death 触发 SkillDef，黑绳同位遗言面）：你死亡时按座位序对全部其他角色各发 1 雷伤——Death 前置窗口触发（changan 氏同款；死亡被取消仍已触发=登记差异）。逐目标 emit Damage+Yield（铁索逐发同款）——**内容侧首个可挂起机**：目标濒死询问期间停机，`BossLeizhu { owner, emitted }` 快照携带进度，**入快照重建表首例**（snap.rs `restore` match 新增臂；此前 boss 一步机单步即 Done 不入表）。

条目 `("卞城王", Qun, &[雷舟, 雷斧, 雷诛], 4, Male)`（BOSS_GENERALS[45]）；`STANDARD_RULESET_VERSION` 266→267。

## 接线触点

standard.rs（常量+行为机+def×3+条目+版本）、kernel.rs（雷舟 Judge-begin 臂+雷斧 End-begin 臂）、snap.rs（变体/check_pid/恢复 None/**重建臂新增**）、archive.rs（enc/dec tag 74）、engine.rs（canonical_hash push tag 74）、a0_tag_registry.rs（样例表+variant_index=72）、lib.rs FROZEN 尾追加「雷舟,雷斧,雷诛」（def 表序）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_bianchengwang_leizhou_leifu_phase_arms`（准备阶段雷伤+结束阶段横置切换+存档往返）、`boss_bianchengwang_leizhu_death_thunder_sequential`（真死 chongzheng=0；第 1 发→2 号濒死挂起→**存档往返→续跑第 3 发**，emitted 进度保留不重发）——2/2 绿 |
| 版本探针 | 普通批口径：267→266 回退复跑 golden **全绿**、267 复跑**全差**=位移唯一来源是版本字段（266 全绿同时证明新 def/条目不触既有场景轨迹） |
| 14 golden | 程序化行级重录（按条目名锚定+hash 行正则）14/14，重录依据注释链追加 v267 条目 |
| 全量 | `cargo test --offline -p engine-native --no-fail-fast` **625/0/0**（623→625，新增 2 测） |
| Gate 1b | 独立 `CARGO_TARGET_DIR` 下 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 测试 lint 债清理（本批顺带）

Gate 1b --all-targets 口径此前在测试面有存量未触发披露的 lint 债：张角批测试 `for it in 0..40` unused variable + 3 处 `drop(&OpenChoice)` no-op（张角 1 处、宋帝王 1 处、本批新测 2 处中收敛）。本批一并清理（删 no-op drop 行、unused 改 `_`）后 --all-targets 口径首次如实转绿。**登记**：v264–v266 各批证据中「Gate 1b exit 0」按当次执行口径（--lib 或未含测试面警告）理解；--all-targets 口径的真实转绿以本批为首个诚实记录。

## 已知边界与登记

- 雷舟/雷斧随机目标→座位序首个确定性近似（血池/胜负同款登记差异）；雷斧横置态切换未走铁索连横结算面（上游 link() 亦仅状态切换，同口径）。
- 雷诛死亡被取消仍已触发（死亡前置窗口先于取消判定，黑绳同款登记差异）。
- 十殿阎罗进度：秦广王/楚江王/五官王✓/宋帝王✓/卞城王✓ + 阎罗王/泰山王/都市王/平等王/转轮王/孟婆/地藏王待落；卞城王属勘验完备第 5 片。
