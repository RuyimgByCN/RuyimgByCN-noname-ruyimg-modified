# ADR 0306：ABI 451 —— min 表达式原语

本 ADR 登记 C 段第二个 ABI（451）：`min(a, b)` 表达式，用于表达「至多 N 张/点」的
上限截断语义，是镇骨、备战、威迫等技能共同需要的计算原语。

## 原语

- DSL 表达式 `min(a, b)`：求两个整数表达式的最小值。
- 编译为 VM MIN 操作码（0x36，位于 NOT 之后）：弹出右操作数、左操作数，压入二者较小值。
- IR：`ExprIr::Min { left, right }`；类型检查在 check_expr 的 Call 分支处理 `min`；
  代码生成 emit 左右操作数后压入 `Op::Min` 并设置 ABI 451 门控。

改动面（全部追加式）：
- content-schema `ExprIr::Min`（枚举末尾追加）
- rule-vm bytecode `Op::Min` 变体 + `op::MIN = 0x36` 常量 + assemble 映射 + VM 分发
  `binop(|a, b| a.min(b))`
- rule-vm reference.rs `InstructionKind::Min` + 解码 + 求值 `i32::min`
- rule-vm verify.rs MIN 并入二元操作栈效果（-2/+1）
- engine-core registry.rs MIN 并入零操作数定宽列表
- content-compiler typecheck/codegen + `required_rule_vm_abi` 451 门控
- `RULE_VM_ABI_VERSION` 450→451（ABI-COORDINATION.md 已领取 ABI-MAX 451）

## 验证

- `cargo test -p rule-vm`：新增 `min_picks_lower` 用例（min(7,3) 与 min(-2,5) 执行通过），
  10 通过。
- DSL 打包验证：`draw_cards(self, min(5, hand_count(self) + 1))` 编译通过，包正确报告
  ABI 451。
- content-runtime 全部测试通过。

## 消费技能（后续）

`drlt_zhengu`（镇骨，adjust_hand_to min(5, hand_count)）、`beizhan`（备战，
adjust_hand_to min(5, maxHp)）、`weipo`（威迫，drawTo min(5, maxHp)）均需 min 表达式；
其完整移植还依赖各自的标记追踪/双向触发原语，另行领取。
