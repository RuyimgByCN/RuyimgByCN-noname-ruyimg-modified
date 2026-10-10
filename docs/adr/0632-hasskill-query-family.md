# ADR 0632：hasSkill 查询原语族（Phase 3 第三族）

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0631](0631-history-count-query-family-and-dsl-expression-extension.md)

## 背景

ADR 0629 诊断的占位符原语族分布中，**hasSkill_check（507 个）**是第二大宗：占位符的触发
条件依赖「玩家是否拥有某技能」（`player.hasSkill("X")`）。引擎此前无技能存在性查询原语。

## 决策

新增 `HasSkill` 查询（跨 rule-api/rule-vm/engine-core/content-compiler 全链路）：

```rust
QueryRequest::HasSkill { player, skill_def }  // → Bool: 玩家是否拥有该 skill_def 的技能实例
```

- `query_tag::HAS_SKILL = 34`；DSL 名 `has_skill(player, skill_def)`（arity 2）。
- 求值：`state.skills.sorted_live()` 中是否有 `owner==player && def==skill_def` 的实例。
- cache key tag 34 + player + skill_def。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo check --workspace`（全链路） | **通过** ✅ |
| DSL `has_skill(self, 12345)` 端到端编译 | **通过** ✅ |
| `cargo test --workspace` | **全绿** ✅ |

## 落地情况与结构性观察

本族实现后，**可纯 DSL 补写的占位符仍很少**。审计 hasSkill_check 占位符发现：绝大多数的
`hasSkill("X")` 只是**复合过滤器的一个子条件**（其余子条件如 `currentPhase==player`、
`event.getParent("phaseUse")`、`countCards("hs")`、`chooseToUse` 等各自需独立原语）。
仅 2 个占位符的 content 是单语句且含 hasSkill，但其过滤器仍含上述不可表达子条件。

这与 ADR 0630/0631 的观察一致：**占位符的可表达性瓶颈是「复合过滤器的多样性」，每条占位符
的过滤器是一组独特子条件的组合，需逐条分析其全部子条件是否都有原语**。已落地的三族原语
（card-name-scoped 过滤、history-count 查询、hasSkill 查询）+ DSL 表达式扩展（`&&`/`>`）
把各自原语侧从阻塞项移除，但复合过滤器中其余子条件仍是逐技能的 content/filter 工作量。

## 后续

- 本族解锁 hasSkill_check 占位符的「查询」侧。完整闭环仍需：
  1. 继续扩引擎原语族（currentPhase 查询、phase 父级检测、zone 计数等复合过滤器的子条件原语）；
  2. 每占位符逐条把复合过滤器拆解为已支持原语的组合（用 `&&` 拼接），无法拆解的登记近似。
- 这是按原语族 + 逐技能分析推进的多会话长期工作。本 ADR + 0630/0631 建立了可复现模式：
  原语族（枚举变体→全链路接线→DSL 名）→ 逐技能 DSL 补写 → rebuild-pack.sh 重建 → cargo test --workspace + sync。
