# ADR 0631：history-count 查询原语族 + DSL 表达式扩展（Phase 3 第二族）

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0630](0630-card-name-scoped-trigger-filter-family.md)（card-name-scoped 过滤族）

## 背景

ADR 0629 诊断的占位符原语族分布中，**history_count（485 个）**是第四大宗：占位符的触发
条件依赖「本回合内摸牌/伤害历史计数」（如「本回合摸牌数 ≥ 体力上限且造成伤害数 ≥ 体力上限
则失去体力」）。引擎既有 `card_action_count`（本回合用牌数）等计数，但缺「本回合摸牌数」
与「本回合造成伤害数」的查询原语，且 DSL 表达式只支持 `==`/`<` 单比较，无法表达 `>=`/`&&`
复合条件。

## 决策

### D1：history-count 查询族（rule-api + rule-vm + engine-core）

新增两个 `QueryRequest` 变体（追加以保 enum 序号稳定）：

```rust
CardsDrawnThisTurn { player }    // state.turn.card_gain_count(player) — 本回合摸牌数
DamageDealtThisTurn { player }   // state.turn.damage_dealt_count(player) — 本回合造成伤害事件数
```

接线全链路：
- `crates/rule-api/src/query.rs`：枚举变体。
- `crates/rule-vm/src/bytecode.rs`：`query_tag::CARDS_DRAWN_THIS_TURN = 32`、
  `DAMAGE_DEALT_THIS_TURN = 33`。
- `crates/rule-vm/src/vm.rs` `host_query`：tag → QueryRequest（pop player）。
- `crates/engine-core/src/query/mod.rs` `QueryExplanation`：求值（`card_gain_count` /
  `damage_dealt_count`，复用既有 state.turn 计数器）。
- `crates/engine-core/src/query/cache.rs` `from_request`：缓存键（tag 32/33 + player）。
- `crates/content-compiler/src/codegen.rs`：DSL 名 `cards_drawn_this_turn`/`damage_dealt_this_turn`
  → tag。
- `crates/content-compiler/src/typecheck.rs`：查询名白名单 + arity(1)。

### D2：DSL 表达式扩展（content-compiler + rule-vm）

`&&`、`>` 两个运算符，解锁复合条件：

- **lexer**（`lexer.rs`）：新增 `TokenKind::AmpAmp`（`&&`）、`Gt`（`>`）+ 扫描。
- **parse**（`parse.rs`）：新增 `parse_and` 层（`parse_expr → parse_and → parse_cmp`），
  `&&` 左结合；`parse_cmp` 加 `>` 臂。
- **VM 字节码**（`bytecode.rs`）：新增 `Op::Gt`、`Op::And` + `op::GT=0x37`、`op::AND=0x38`。
- **VM 解释器**（`vm.rs`）：`op::GT => binop(a>b)`、`op::AND => binop(a!=0 && b!=0)`。
- **verifier**（`verify.rs`）：GT/AND 加入双目栈效应组。
- **codegen**（`codegen.rs`）：`">" => Op::Gt`、`"&&" => Op::And` + estimate_pc 枚举。

### D3：首批落地

| 技能 | 包 | 实现 |
|---|---|---|
| `dckuichi`（溃痴） | xianding | `trigger phase_end owner_is_target { if cards_drawn_this_turn(self) > max_hp(self) - 1 && damage_dealt_this_turn(self) > max_hp(self) - 1 { effect lose_hp(self, 1); } }` |

上游 filter：本回合摸牌数 ≥ maxHp && 造成伤害数 ≥ maxHp；content：loseHp。用 `> maxHp-1`
等价整数 `>=`。落地后 catalog migrated_skills 2905→**2906**。

### D4：清理 stale `package.npack.json`（数据完整性修复）

实现 dckuichi 时发现 `content/original/ported/{ddd,wandianlunsha,xianding}/package.npack.json`
是**陈旧重复**（与 `<pkg>.npack.json` 内容不一致，logic_hash 不同）。sync 的 `portedDefinitions`
会读取全部 `.npack.json`，陈旧副本先处理（字母序 `package` < `<pkg>`），导致 dckuichi 被记为
behaviorless（陈旧副本无其 trigger）。删除三处 stale `package.npack.json`（ported 目录应只含
`<pkg>.npack.json`，`package.npack.json` 仅属 `extensions/`）。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo check --workspace`（rule-api/rule-vm/engine-core/content-compiler 全链路） | **通过** ✅ |
| DSL 复合条件端到端编译（`cards_drawn > 1 && damage_dealt > 0`） | **通过** ✅ |
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo test --workspace` | **全绿** ✅ |

## 后续

- 本族解锁 history_count 占位符的「查询」侧；多数占位符的 content 仍含复杂副作用（addTempSkill/
  storage/多步 choice），需逐技能写或扩 effect 原语。DSL 表达式扩展（`&&`/`>`）顺带解锁所有
  需复合条件的占位符（不限 history_count）。
- 下一族候选：`hasSkill_check`（507，技能存在性查询）、`view_as`（246，转化技 effect 原语）。
- 每族按 ADR 0630/0631 模式推进。
