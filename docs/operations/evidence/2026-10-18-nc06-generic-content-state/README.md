# NC06 内容私有状态：核心专用字段普查 + 通用内容面同源验收（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC06**（[方案 §NC06](../../../architecture/native-convergence-plan.md#nc06)）第一切片
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**普查已量化 + 通用内容面（`vm_aux`）同源契约已验收 + 发现并修复 1 处归属校验缺口**

## 1 普查：核心状态里的内容专用字段（实测）

| 结构 | 字段总数 | 其中内容专用（按技能命名） | 例子 |
|---|---:|---:|---|
| `TurnState` | 60 | **≈55** | `mufeng_used`、`shenmiguo_used`、`lianda_used`、`shuchui_armed`、`kunlunjing_snapshot`… |
| `PlayerState` | 104 | **≈85** | `baoxue_used`、`xuehuang_used`、`hlss_stars`、`hutian_deposit`、`mailun_mode`… |
| `GameStateCore` | 17 | 少量（`vm_aux` 为通用内容面） | `declared_kinds`/`card_natures` 等 |

合计约 **140 个内容专用类型字段**——与 NC06「普通内容不得继续要求扩张核心专用字段及全部手写编码分支」直接冲突，是后续迁移的**量化基线**。

## 2 通用内容面（`vm_aux`）形状 → 契约验收

新增 `crates/engine-native/tests/nc06_generic_content_state.rs`（6 项），逐形状验收
**与类型字段同等的持久化契约**（hashed + encoded + 恢复后摘要一致）：

| 形状 | 对应内容语义 | 结果 |
|---|---|---|
| `marks: (seat, u32) → i32` | 层数/标量 | 通过 |
| `flags: (seat, String) → u32` | pending/assigned 计数 | 通过 |
| `once: (seat, String, scope) → set` | 限次（1 回合 / 2 轮 / 3 整局） | 通过 |
| `piles: (seat, String) → Vec<CardId>` | 内容自有牌堆（牌引用） | 通过 |
| `hits: (seat, String) → u32` | 回合内触发计数 | 通过 |
| 换回合清理语义 | 回合范围数据随换回合清理；整局范围保留 | 通过 |

## 3 发现并修复：`vm_aux.piles` 未计入牌归属校验

- **现象**：把牌从牌堆移入内容牌堆后保存再恢复 → `IdOutOfBounds{what:"card_missing_from_all_zones", got:27}`——
  即「牌放进内容自有牌堆」这一**合法活状态**被恢复期校验判非法（与 NC12 §5.1 同类：引擎允许的
  状态被自己的校验拒绝）。
- **修复**：`snap/validation.rs` 的牌归属校验加入 `core.vm_aux.piles` 的全部牌
  （内容牌堆是合法归属区）。修复后该用例通过。
- 影响面：swd 牌堆类技能（`lianyaohu_store`/`zhucheng` 等）与任何使用内容牌堆的扩展。

## 4 验证结果

```
cargo test --offline -p engine-native --test nc06_generic_content_state
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC06 形状/清理用例 | **6 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1140 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 14 golden | **2/2 零位移** | [replay-golden.log](replay-golden.log) |
| 行数门禁 | 8241 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC06）

- [ ] **迁移试点**：把 1–2 个真实内容专用字段（如 `mufeng_used`/`baoxue_used`）迁到 `vm_aux`，
      保持行为与 golden 不变（需版本/存档字段处理，属下一批）
- [ ] **通用面缺口**（普查得出）：① 无作用域旗标（`flags` 无回合/轮/整局生命周期，
      清理由手写 `clear_turn` 承担）；② 无全局（非按座位）内容状态；③ 值类型只有 `u32`/`i32`
      （缺字符串/枚举/列表）；④ 无按牌（per-card）内容状态
- [ ] **编码与摘要同源**（NC07 联动）：140 个字段仍是逐字段手写编码分支；迁移后应统一由
      通用面承担，编码/摘要/校验只需覆盖少量形状
- [ ] `snap/validation.rs` 属并行重构未跟踪文件 → 本次修复随其提交入库
- [ ] 新内容接入规范：明确「新增内容状态必须走 `vm_aux`（或后续通用面），不得扩张核心字段」
      的评审口径与测试入口
