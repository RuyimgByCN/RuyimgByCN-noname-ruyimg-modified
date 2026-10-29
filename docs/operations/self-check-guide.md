# 自检指南（提交前应该跑的 5 条命令）

> **适用**：在 `noname-next/` 根目录执行。
> **为什么有这份文档**：这五条检查**曾经散落在提交信息里** —— 换人接手就不知道有哪几条、期望输出是什么、红了怎么办。

## 1 一览

| # | 命令 | 检查什么 | 期望 | 红了怎么办 |
|---:|---|---|---|---|
| 1 | `python3 -B scripts/check-source-lines.py` | **单文件 ≤ 2000 行；Rust 非测试源码有效行（去注释、空行、`#[cfg(test)]`）≤ 1000** | `exit=0` | **拆文件**（见 AGENTS.md 第 1–3 条），**不要压缩代码或藏文件** |
| 2 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **门禁脚本自身** | `OK` | **修门禁脚本**，不是改阈值 |
| 3 | `python3 -B scripts/check_local_links.py --all` | **docs/ 下全部 Markdown 的本地链接** | `坏链 0` | **修正路径或恢复缺失目标**；`KNOWN_BAD` 只保留已核实的历史例外，不能用它豁免新增坏链 |
| 4 | `python3 -B scripts/refresh_recent_rounds.py --check` | **进度主稿「最近 20 轮索引」是否过期** | `exit=0` | **跑一次不带 `--check` 的**刷新它 |
| 5 | `python3 -B scripts/check_stale_numbers.py` | **已被更正的旧数字是否仍残留** | `残留旧值 0 处` | **改掉旧值**；若是「历史日志/更正记录」⇒ 补 `当时`/`~~…~~` 等标记 |

## 2 全量测试（改代码时额外跑）

```bash
cargo test --offline -p engine-native
```

engine-native 与 engine-core 的集成测试按分组编译（2026-10-29 O6）：单跑某个测试文件写 `cargo test -p <crate> --test <组> <文件名>::`；新增 `tests/*.rs` 须登记到 `tests/groups/<组>.rs`，漏登记由 `--test test_registry` 报出。

当前基线、失败和忽略项见[进度主稿](../architecture/native-progress.md#delivery-2026-10-24)。涉及原生内核时执行上面的完整测试；服务端、前端、Worker、WASM 或存档接线变化还须运行对应回归，不能用文档门禁代替产品验证。
**红了**：先判断是 **golden 语义变化** 还是 **回归**；**golden 变化必须先查原因再决定是否重录**，不能只改期望值。

## 3 行数门禁状态（2026-10-24）

`EffectRequest` 已按功能拆分，通过顺序登记重组为同一个扁平枚举，保留调用方 API 与序列化序号。当前行数与验证结果见[进度主稿](../architecture/native-progress.md#effect-split-2026-10-24)和[本轮证据](evidence/2026-10-24-effect-split/README.md)。行数门禁应为 `exit=0`；`preflight.py` 已取消预期失败豁免，超限会正常阻挡。

五条自检仍按第 1 节逐项核验；历史超限与旧计划见[当时的量化分析](evidence/2026-10-19-effect-request-split-plan/README.md)，不再作为当前豁免依据。

## 3.1 preflight：一次跑完全部 21 道门禁

```bash
python3 -B scripts/preflight.py             # 完整 21 项
python3 -B scripts/preflight.py --quick     # 实跑 19 项，跳过脚本单测与台账↔枚举一致
```

**第 21 项是 2026-10-29 新增的「原生分层棘轮」**（`check-native-layering.py`：engine-native 内核层对 content/engine 的引用只降不升）。

**第 20 项是 2026-10-19 新增的「过滤臂↔元数据一致」**——
`report_filter_meta_mismatch.py --strict` 会比对 `metadata/*.rs` 里每个 `HostBehavior` 声明的
`TriggerMeta.event` 与 `filtering/**` 里该行为的过滤臂是否**明确互斥**。

> **为什么要有它**：这类不一致会让技能**静默不触发**，而全量测试只表现为「某个测试不对」，看不出根因。
> 2026-10-19 当天它抓出 3 处（`WudianZiyu` 是我自己改出来的、`WudianZhucheng` 与 `SwdBaishouzhihuHeld` 是历史遗留）。

**台账改动另查差异**：

```bash
python3 -B scripts/report_ledger_diff.py        # 与 HEAD 对比：移出/新增哪些行（手算会漏）
```

## 4 为什么这五条要分开

| 检查 | 谁能替代它 | 不能替代的原因 |
|---|---|---|
| **行数门禁** | 人工 review | **人不会数行** |
| **门禁自测** | 相信门禁 | **门禁自己也会错** |
| **链接** | 点一遍 | 人工抽查容易漏掉历史文件和相对路径变化 |
| **索引新鲜度** | 记得更新 | **索引腐化速度 = 每轮一次** |
| **旧值残留** | grep | **grep 会命中「更正记录」**，需要排除规则 |

## 5 一次性全跑

```bash
python3 -B scripts/preflight.py &&
cargo test --offline -p engine-native --no-fail-fast
```

以上在仓库根目录执行，适用于涉及原生内核的改动；纯文档整理执行完整 preflight，并核对改动文件的引用即可。`&&` 确保门禁失败后停止；保留原始输出和退出码，不用 `| grep` 或 `| tail` 的结果代替被测命令结果。
