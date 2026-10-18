# NC07 状态/编码/hash/引用校验同源审计（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC07**（[方案 §NC07](../../../architecture/native-convergence-plan.md#nc07)）第一切片：**状态面同源审计 + 两处根因修复**
- 规则版本 **v321 → v322**（座位环进入规范摘要 = 状态身份变化）；存档格式 v105（字段无变化）
- 结论：**审计已建立（12 项）+ 发现并修复 2 处不同源 + 14 golden 重录（含回退探针）**

## 1 审计口径（新增 `crates/engine-native/tests/nc07_state_same_source.rs`，12 项）

对每个代表性状态面验收三件事：
1. **hashed**：改动字段 → `state_hash()` 必须变化（否则是摘要漂移源）；
2. **encoded**：改动字段 → 归档 encode/decode + 恢复后仍为改动值且摘要一致；
3. **validated**：篡改解码快照（越界引用）→ `Engine::restore` 明确拒绝。

覆盖面：体力 / 体力上限 / **座位环（换位）** / 回合与阶段 / 身份分布 / 技能实例（授予+启用位）/
装备区 / 判定区 / 翻面 / 护驾计数 / Boss 关卡档案 / 恢复期引用校验（座位环越界、武将槽越界、技能定义越界 + 正常快照仍可恢复）。

## 2 审计发现（两处真实缺陷，均已修）

| # | 缺陷 | 证据 | 修复 |
|---|---|---|---|
| 1 | **`seat_ring` 未进入规范摘要**：仅座位次序不同的两个状态得到相同 `state_hash()` | 审计用例先失败（`[seat_ring] 字段未参与状态摘要（hash 未变）`） | `canonical_hash` 加入座位环长度与逐项座位号（`engine/hash.rs`） |
| 2 | **恢复期不校验座位环**：越界/重复座位可被接受 | 审计用例先失败（`座位环越界引用应被恢复校验拒绝`） | `validate_core` 新增座位环校验：长度须等于玩家数、取值合法且构成排列（`snap/validation.rs`） |

其余 10 项审计**一次通过**（体力/上限/回合/身份/技能实例/装备/判定/翻面/计数/Boss 档案 均 hashed+encoded；武将槽与技能定义越界被拒）。

## 3 规则版本与 golden（v322）

- 座位环进入摘要 ⇒ 14 场标准场景摘要**全部位移**（原因明确：摘要覆盖变化，不是语义意外）；
- 规则版本 `321 → 322` 同步递增（状态身份变化）；
- 重录后 `replay_scenarios` **2/2 通过**；**回退探针**：临时改回 321 → **14/14 mismatch**，证明位移包含版本与摘要覆盖变化；
- 重录依据写在 `replay_scenarios.rs` 顶部注释（不得只改期望值消错）。

## 4 验证结果

```
cargo test --offline -p engine-native --test nc07_state_same_source
cargo test --offline -p engine-native --test replay_scenarios
cargo test --offline -p engine-native --tests --no-fail-fast
CARGO_TARGET_DIR=<tmp> RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib --tests
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC07 审计定向 | **12 passed / 0 failed** | [targeted.log](targeted.log) |
| 14 golden（v322 重录后） | **2/2 通过** | [replay-golden-v322.log](replay-golden-v322.log) |
| 版本回退探针 | **14/14 mismatch** | [probe-rollback.count](probe-rollback.count) |
| engine-native 测试面 | **1128 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| `-D warnings`（lib+tests） | exit 0 | [gate1b-lib-tests.log](gate1b-lib-tests.log) |
| 行数门禁 | 8237 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 提交范围说明（并行开发）

两处修复位于**并行重构新建的未跟踪文件**（`engine/hash.rs`、`snap/validation.rs`），规则版本常量位于 `content/standard.rs`（同属并行重构占用）。为避免混提他方改动，本次提交**只含审计用例、证据与本进度文档**；两处修复 + v322 + golden 重录**位于工作树**，待并行重构提交后随同入库（与 NC01 版本升格同一处理方式）。

## 6 未完成 / 待办（NC07）

- [ ] 修复入库（依赖并行重构提交）：`hash.rs` 座位环、`validation.rs` 座位环校验、v322、14 golden
- [ ] `vm_aux` 内部各表（`once`/`hits`/`marks`/旗标）逐项同源验收
- [ ] 每个扩展字段（swd/L19/L20 等新增状态）逐项 hashed+encoded+validated 清单
- [ ] 跨版本迁移路径（旧规则版本档案 → 新版本）与缺失内容重新取得
- [ ] 编码层与摘要层「同一处定义」的结构化收敛（当前仍是两处代码遍历同一状态，靠本审计兜底）
