# 2026-10-11 T11 boss 内容落包首批：烛阴+凶兽（规则 v208）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `63b64ff1`；[首切片评估](../../2026-10-11-t9-lx-trace-survey/README.md)的落包步，消费面=v207 关卡骨架）。上游依据：`mode/boss.js` `boss_zhuyin`（男/群/hp4/单技/`isHiddenBoss` 随从且 `isBossAllowed`）与 `boss_xiongshou`（5911–5941：damage/turn 双子技+globalFrom mod）。

## 建模（内置模式内容，非扩展包）

- `BOSS_GENERALS`（standard.rs）：Boss 内置武将表首批烛阴——追加于目录索引空间**动态尾段**（base+启用扩展数之后；扩展将 def=base+i 的既定 ABI 不受扰动，boss 将 def 随启用集浮动为登记口径）。`general_meta`/`general_count` 三段化（base→ext→boss）；快照 `b.generals` 界随之覆盖 boss 段。
- `凶兽` SkillDef 尾部追加（权限型 Noop+三面查询，马术同款）；lib.rs `standard_skill_ids_append_only` FROZEN 名单尾加「凶兽」（62 名）。
- 凶兽三面：①**杀伤条件 +1**（`xiongshou_bonus` 于 `emit_sha_damage` 加成链：来源持技 ∧ 目标体力<来源）；②**距离 -1**（`computed_distance` minus 链，马术同款）；③**不可翻面**（`commit_turn_over_before` 入口直读跳过——死亡取消型基建同款；上游 `turnOverBefore` priority 20 `trigger.cancel` 同效）。
- `debug_set_general_def(player, def)`：通用装配测试面（写槽+`apply_general_assembly` 授技与体力）。

## 登记差异

- 凶兽翻面取消在 commit 入口直读（上游为 priority 20 触发器窗口——外部取消型技写回路径不经过时同效；如后续出现「先翻面后取消可见」的对抗面再升窗口化）。
- boss 将 def 为动态尾段（随启用扩展集浮动）；烛阴 `isHiddenBoss`/`extraModeData` 面未建模（随关卡内容包推进）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_zhuyin_xiongshou_semantics` | 装配 4 血+技能实例；杀伤 +1（3 血目标受 2 → hp1）；等体力对照不加成（相邻 3 号出杀 4→3）；距离 -1；翻面事件后 face_down 不变；技能实例与状态存档往返 |
| 既有回归 | boss 关卡/重整/addFellow、native_kernel 210/0、g1_d3/g1_wudian 扩展将 def 约定不变 |
| 版本回退探针（208→207 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；Boss 分支不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **555 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 207→**208**（存档不变 v48） |
