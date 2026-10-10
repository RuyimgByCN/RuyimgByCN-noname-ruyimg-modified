# ADR 0634：event_target_count 独立 opcode（Phase 3 子条件原语修复）

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0633](0633-card-number-subcondition-already-expressible.md)

## 背景

`event_target_count`（事件目标数）是复合过滤器高频子条件（`event.targets.length > 1` 等）。
审计发现 DSL 表达式 `event_target_count` 已存在（`ExprIr::EventTargetCount`，typecheck 限 card
trigger），但 **codegen 把它编译到共享的 `Op::HostEventAmount`**（注释称「card-target count 与
damage amount 是事件互斥标量绑定，共享已验证 host opcode」）。这使得 `event_target_count` 实际
读取的是 `event_amount`（伤害点数），而非目标数——**语义错误**，但因既有内容未在目标数与伤害
点数同时出现的场景使用而未暴露。

## 决策

为 `event_target_count` 增设**独立 host opcode**，读取真正的 `host.event_targets().len()`：

- `crates/rule-vm/src/bytecode.rs`：`op::HOST_EVENT_TARGET_COUNT = 0x5B` + `Op::HostEventTargetCount`。
- `crates/rule-vm/src/vm.rs`：`op::HOST_EVENT_TARGET_COUNT => push(host.event_targets().len())`。
- `crates/rule-vm/src/verify.rs`：加入 push-1 栈效应组。
- `crates/rule-vm/src/host.rs`：`CollectingHost` 加 `event_targets: Vec<u32>` 字段 + Default +
  `event_targets()` 实现（既有 `VmHost::event_targets()` 默认返回 `&[]`，CollectingHost 原未覆盖）。
- `crates/content-compiler/src/codegen.rs`：`ExprIr::EventTargetCount => Op::HostEventTargetCount`
  （不再共享 HostEventAmount）+ estimate_pc 枚举。
- `crates/content-compiler/src/lib.rs` 测试 `card_target_count_reaches_typed_effects`：host 填
  `event_targets: vec![4,5,6]` 以反映新语义（len=3）。

## 影响：npack 重打包（logic_hash 变更）

`event_target_count` 字节码变化波及 6 个包（其 `.skill` 用到该表达式）：huicui/jsrg/offline/sp/
tw/xianding。逐包用 `scripts/rebuild-pack.sh` 重建 → 部署 ported+extensions 三副本 → 重装 store。
其中 sp 包 logic_hash 变化触发 3 个 golden 测试的 `PACKAGE_LOGIC_HASH` 重钉（dingpan_buzhi/
hengzheng_dongzhuo/ziyuan_mizhu）；jsrg/offline/tw/xianding/huicui 的 golden 测试不断言包级
logic_hash（或断言项未变），无需重钉。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo check --workspace` | **通过** ✅ |
| `card_target_count_reaches_typed_effects` 单测 | **通过** ✅ |
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo test --workspace` | **884 二进制全绿 / 0 FAILED** ✅ |

## 后续

- 本修复把 `event_target_count` 从语义错误改为正确读取目标数，使「多目标判定」子条件真正可表达。
- 与 ADR 0633 终审一致：复合过滤器可表达性瓶颈是子条件多样性；每修一族子条件原语，可组合实现
  的占位符子集缓慢扩大。继续按 phase-parent / storage 比较 / card-name-length 等子条件原语推进。
