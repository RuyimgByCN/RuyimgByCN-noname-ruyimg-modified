# ADR 0725：N04 检查点与当前段关系完整验收

日期：2026-09-16
状态：已实施（engine-native）
关联：[0710](0710-native-restore-checkpoint-abort.md)、[0719](0719-native-checkpoint-hierarchy-guidao-equip.md)、进度文档 N04

## 背景与决定

进度文档 N04 登记：「检查点与当前段的玩家数、模式、revision/dedup、段状态等关联未完整验收；
无检查点但段开放的异常快照仍可恢复，abort 返回 Err——N07 原修复不等于此类状态合法」。
本批把段/检查点关系提升为恢复期校验（非法快照在 restore 即拒绝，而非恢复后行为异常）。

### 校验规则（restore 期）

设快照为 `S`、其内嵌段检查点为 `C`（段启动时的完整快照）：

1. **`S.segment_open = true` ⇒ `C` 必须存在**：开放段没有检查点则恢复件无法
   `abort_segment`，属非法快照（`checkpoint_missing_for_open_segment`），
   不再是「恢复成功但 abort 报错」。
2. **座数/模式一致**：`C.core.players.len()` 与 `C.core.mode` 必须与 `S` 一致
   （此前跨座数/跨模式检查点可绕过 bounds 校验——`checkpoint_players_mismatch` /
   `checkpoint_mode_mismatch`）。
3. **revision/dedup 单调**：检查点是段启动快照，接受命令只增不减——
   `C.core.revision ≤ S.core.revision`、`C.last_dedup ≤ S.last_dedup`，
   超前即被篡改（`checkpoint_revision_ahead` / `checkpoint_dedup_ahead`）。
4. **检查点自身静止**：`C.segment_open = false`（段启动时不存在开放段；
   嵌套层级已由 0719 的 `checkpoint_nested` 拒绝）（`checkpoint_segment_open`）。

### 回归

- 正向 `open_segment_checkpoint_association_positive`：时间片制造真实开放段
  → 快照断言四条不变量 → 恢复件延续段开放（`is_suspended`）→ `abort_segment`
  可回滚并继续对局。
- 负向 `restore_rejects_segment_checkpoint_mismatches`：六类篡改逐一恢复拒绝
  （缺检查点 / revision 超前 / dedup 超前 / 座数不符 / 模式不符 / 检查点段开放）。

## 影响

- 测试 140 项（boundary_snapshot 55、native_kernel 80、场景 2、源码 2、调试 1），净增 2。
- 规则版本不变（6）；golden 不变（校验只收紧 restore 的接受集，合法快照编码未变）。
- N04「检查点与当前段关系完整验收」就此闭合；剩余：机器/选择/牌归属关联的
  更深结构验收（已有 N04-a/d/e 覆盖主要反例）、N05 负体力救援、N06 重放事件对照、
  N09 峰值内存与跨目标对照、N10–N12 平台阶段。
