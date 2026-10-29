# NC01 座位与触发语义：根因修复与回归（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC01**（[方案 §NC01](../../../architecture/native-convergence-plan.md#nc01)）
- 反例来源：[2026-10-18 架构审查](../2026-10-18-architecture-audit/README.md)「座位距离与排序语义」（`seat-distance-probe.log`）
- 规则版本：**v321**（未变；见下「版本处理」）；存档格式 v105（未变）
- 结论：**根因已修 + 回归已加 + 定向/全量通过**；版本升格与 3 处排序调用方的迁移待并行重构提交后随同入库

## 1 根因（已确证）

`GameStateCore::seat_distance` 旧实现：

```rust
let n = self.players.len() as u8;
let (a, b) = (a % n, b % n);
let cw = b.wrapping_sub(a) % n;    // ✗ u8 在 256 处环绕后再取模
let ccw = a.wrapping_sub(b) % n;
cw.min(ccw)
```

5 人局 `d(0,1)`：`ccw = 0u8.wrapping_sub(1) = 255 → 255 % 5 = 0` → `min(1, 0) = 0`（应为 1）。审计在 5 人局 25 对中复现 14 对、6 人局 36 对中 12 对不符；8 人局恰好全部吻合，说明单测 8 人局无法暴露。

## 2 修复与语义拆分

| 语义 | 函数 | 用途 | 本轮动作 |
|---|---|---|---|
| **最短座位距离** | `seat_distance` | 相邻/射程/±1 判定 | **修正算术**（有符号 `rem_euclid`，与 `seat_distance_alive` 同口径） |
| **顺座次序** | `seat_steps_from`（新增） | 触发结算顺序、按座次轮转 | **新增**：以 `seat_ring` 环位置计步（`swapSeat` 后随之改变），返回 0..n-1 |
| **存活座位环** | `seat_distance_alive` / `ring_next` / `ring_prev` | 死亡玩家不占位 | 复核不变（既有实现正确） |

调用方逐一核对并迁移：

| 调用方 | 语义 | 处理 |
|---|---|---|
| `resolution.rs` 触发入队序键 | **顺序** | `seat_distance` → `seat_steps_from` |
| `content/host/step/wudian_recovery.rs`（受伤者排序） | **顺序** | 同上 |
| `content/host/step/wudian_damage.rs`（受伤者排序） | **顺序** | 同上 |
| `content/host/step/swd_card_effects.rs`（友方取最近） | **顺序** | 同上（注释同步） |
| `content/host/filtering/swd.rs`、`filtering/common.rs`、`step/yunchou.rs`（相邻判定） | **距离** | 不变（`seat_distance` 语义正确） |

## 3 回归（新增 `crates/engine-native/tests/nc01_seat_semantics.rs`，6 项）

1. **全部支持人数**（2–8 人）逐对枚举：对称性、同座 0、非同座 ≥1、上界 `floor(n/2)`、与解析式一致；
2. **审计反例逐条**：5 人局 `d(0,1)=1`、6 人局 `d(0,3)=3`、8 人局逐对一致；
3. **旋转不变性**：3–8 人局全部旋转下距离不变；
4. **两概念分离**：4 人局顺座次序 `[0,1,2,3]`、最短距离 `[0,1,2,1]`、按距离排序 `[0,1,3,2]`（与顺座次序不同）；
5. **存活环**：无人死亡时两距离一致；击杀 1 号后 `d_alive(0,2)=1`、`d_alive(0,4)=1`、`d_alive(0,3)=2`，而全座位 `d(0,2)=2` 不变；
6. **换位**：`swap_seat(1,3)` 后 `seat_steps_from(0,3)=1`、`seat_steps_from(0,1)=3`、`ring_next(0)=3`。

## 4 验证结果

```
cargo test --offline -p engine-native --test nc01_seat_semantics
cargo test --offline -p engine-native --test replay_scenarios
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-native-loader -p content-runtime -p engine-native-wasm --no-fail-fast
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC01 定向 | **6 passed / 0 failed** | [targeted.log](targeted.log) |
| 14 golden | **2/2 通过；哈希与修正前逐字一致** | [replay-golden-v321-unchanged.log](replay-golden-v321-unchanged.log) |
| engine-native 全量 | **1101 passed / 0 failed / 1 ignored**（1095 → +6 NC01） | [engine-native-tests.log](engine-native-tests.log) |
| 相关包 | 85/0/0 | [deps-tests.log](deps-tests.log) |
| Gate 1b | exit 0 | [gate1b-all-targets.log](gate1b-all-targets.log) |
| 行数门禁 | 8216 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 版本处理（规则 v321，未升格的原因）

- 修正属于**规则语义**变化（触发结算顺序原语），按规范应升规则版本；
- 但版本常量位于 `crates/engine-native-content/src/content/standard.rs`，该文件正被**并行重构**（8 233 行差异，拆分模块）占用；本次已实测：升到 v322 会令 14 场 golden 全部位移（仅版本字段进入 hash，语义哈希不变），**回退探针 14/14 mismatch**；
- 为避免把他方未完成改动一并提交，本轮**保持 v321**、**恢复 14 个 golden 原值**（修正前后逐字一致），并把版本升格 + 随之的 golden 位移登记为**待并行提交后随同入库**；
- 3 处排序调用方的迁移位于并行重构新建的 `content/host/step/*.rs`（未跟踪），同样待其提交后随同入库；本轮已提交的调用方迁移为 `resolution.rs`（触发序键）。

## 6 未完成 / 待办（NC01）

- [ ] 规则版本升格 v322 + 14 golden 版本位移重录（依赖 `standard.rs` 并行提交）
- [ ] `content/host/step/{wudian_recovery,wudian_damage,swd_card_effects}.rs` 三处顺序调用方迁移入库（依赖并行重构提交）
- [ ] 触发优先序的**上游对照**：以真实技能场景（同事件多触发者、跨座位、换位后）验证顺序符合采用的上游语义，而非仅证明排序键算术
