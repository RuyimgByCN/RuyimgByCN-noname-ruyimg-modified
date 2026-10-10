# 阶段 29：DSL Language Server（§35）

> 对应架构计划 v2.0 §35（工具链）。这是 §35 工具链的最后一个「📋 骨架」项，
> 现在变为完整实现。

## 目标

为 content-compiler 的 DSL 提供 LSP 协议实现，使编辑器（VS Code / Neovim /
其他 LSP 客户端）可以提供：

1. `textDocument/completion` — 上下文感知的关键字/标识符补全
2. `textDocument/hover` — 关键字和构造的文档
3. `textDocument/diagnostics` — 编译错误作为 LSP 诊断

## 实现

`crates/content-compiler/src/lsp.rs` — LSP 库函数（非服务器二进制）。

### LSP 类型

| 类型 | 说明 |
|---|---|
| `LspPosition` | 0-based 行/字符 |
| `LspRange` | 起止位置 |
| `LspSeverity` | Error/Warning/Information/Hint |
| `LspDiagnostic` | range + severity + source + message |
| `LspCompletionItem` | label + kind + detail + insert_text |
| `LspHover` | contents (markdown) + range |

### textDocument/completion

上下文检测基于大括号深度 + 最后一个词：

| 上下文 | 补全项 |
|---|---|
| 顶层 (depth=0) | `skill` snippet |
| skill 体内 (depth=1) | `trigger`, `modifier` |
| trigger 后 | 事件名：damage, phase_begin, phase_end, death, dying, cards_drawn |
| modifier 后 | 目标：distance_from, attack_range, hand_limit |
| 目标后 | 模式：add, override, multiply, min, max |
| trigger 体内 (depth=2) | `effect draw_cards(self, N)` snippet |

### textDocument/hover

为 DSL 关键字提供文档：skill, trigger, modifier, effect, damage, phase_begin,
distance_from, add, draw_cards, self, owner_is_target 等。

### textDocument/diagnostics

运行 `compile_source()` 并将 `CompileError` 转换为 LSP 诊断（带位置 + 严重性）。

### 辅助函数

- `byte_offset_to_position(source, offset)` — 字节偏移 → 行/字符
- `span_to_range(source, span)` — Span → LspRange

## 使用方式

LSP 服务器二进制（stdio JSON-RPC）是一个薄包装，调用这些库函数：

```rust
use content_compiler::lsp;

// 补全
let items = lsp::completion(source, cursor_offset);

// 悬停
let hover = lsp::hover(source, cursor_offset);

// 诊断
let diags = lsp::diagnostics(source);
```

## 测试

```bash
cargo test -p content-compiler lsp    # 15 tests
```

| 测试 | 断言 |
|---|---|
| `diagnostics_for_valid_source` | 有效源无诊断 |
| `diagnostics_for_invalid_source` | 无效源有 Error 诊断 |
| `completion_at_top_level` | 顶层补全 `skill` |
| `completion_inside_skill_body` | skill 体内补全 trigger/modifier |
| `completion_after_trigger_keyword` | trigger 后补全事件名 |
| `completion_after_modifier_keyword` | modifier 后补全目标 |
| `completion_for_modifier_mode` | 目标后补全模式 |
| `completion_in_trigger_body` | trigger 体内补全 effect |
| `hover_for_skill_keyword` | skill 悬停文档 |
| `hover_for_trigger_keyword` | trigger 悬停文档 |
| `hover_for_damage_event` | damage 事件悬停 |
| `hover_for_draw_cards` | draw_cards 效果悬停 |
| `hover_returns_none_for_unknown_word` | 用户定义名无悬停 |
| `byte_offset_to_position_multiline` | 多行偏移转换 |
| `byte_offset_to_position_same_line` | 同行偏移转换 |
