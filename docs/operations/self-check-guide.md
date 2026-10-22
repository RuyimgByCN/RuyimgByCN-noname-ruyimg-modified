# 自检指南（提交前应该跑的 5 条命令）

> **适用**：在 `noname-next/` 根目录执行。
> **为什么有这份文档**：这五条检查**曾经散落在提交信息里** —— 换人接手就不知道有哪几条、期望输出是什么、红了怎么办。

## 1 一览

| # | 命令 | 检查什么 | 期望 | 红了怎么办 |
|---:|---|---|---|---|
| 1 | `python3 -B scripts/check-source-lines.py` | **单文件 ≤ 2000 行** | `exit=0` | **拆文件**（见 AGENTS.md 第 1–3 条），**不要压缩代码或藏文件** |
| 2 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **门禁脚本自身** | `OK` | **修门禁脚本**，不是改阈值 |
| 3 | `python3 -B scripts/check_local_links.py` | **主稿 + 证据的本地链接** | `坏链 0` | **改相对路径**；若目标确实不存在 ⇒ 登记到 `KNOWN_BAD` 并注明原因 |
| 4 | `python3 -B scripts/refresh_recent_rounds.py --check` | **进度主稿「最近 20 轮索引」是否过期** | `exit=0` | **跑一次不带 `--check` 的**刷新它 |
| 5 | `python3 -B scripts/check_stale_numbers.py` | **已被更正的旧数字是否仍残留** | `残留旧值 0 处` | **改掉旧值**；若是「历史日志/更正记录」⇒ 补 `当时`/`~~…~~` 等标记 |

## 2 全量测试（改代码时额外跑）

```bash
cargo test --offline -p engine-native
```

**当前基线（2026-10-19，规则 v322 / 存档 v105）**：**1538 passed / 0 failed / 71 ignored**（71 为缺陷守卫，全部为红且用 `-- --ignored` 单独跑）。
**红了**：先判断是 **golden 语义变化** 还是 **回归**；**golden 变化必须先查原因再决定是否重录**，不能只改期望值。

## 3 已知的「红」（截至 2026-10-19）

| 项 | 状态 | 说明 |
|---|---|---|
| **`check-source-lines.py`** | ❌ **exit=1** | **`crates/rule-api/src/effect.rs` 10778 行**（限制 2000）——**基线 `72583b2b` 起即红**。**已量化**：枚举体 10573 行 / **815 变体**；除 `resolve`（5513 行）外 14 个桶全部合规；**拆分需改 1069 个未跟踪文件 ⇒ 本会话不可执行**。见[量化分析与拆分计划](evidence/2026-10-19-effect-request-split-plan/README.md) |

⇒ **除这一条外，其余 4 条应全绿**；**提交信息里必须写明门禁状态**。

## 3.1 preflight：一次跑完全部 20 道门禁

```bash
python3 -B scripts/preflight.py --quick     # 19–20 项，约 30 秒
```

**第 20 项是 2026-10-19 新增的「过滤臂↔元数据一致」**——
`report_filter_meta_mismatch.py --strict` 会比对 `metadata/*.rs` 里每个 `HostBehavior` 声明的
`TriggerMeta.event` 与 `filtering/**` 里该行为的过滤臂是否**明确互斥**。

> **为什么要有它**：这类不一致会让技能**静默不触发**，而全量测试只表现为「某个测试不对」，看不出根因。
> 2026-10-19 当天它抓出 3 处（`WudianZiyu` 是我自己改出来的、`WudianZhucheng` 与 `SwdBaishouzhihuHeld` 是历史遗留）。

**台账改动另有两个工具**：

```bash
python3 -B scripts/report_ledger_diff.py        # 与 HEAD 对比：移出/新增哪些行（手算会漏）
```

## 4 为什么这五条要分开

| 检查 | 谁能替代它 | 不能替代的原因 |
|---|---|---|
| **行数门禁** | 人工 review | **人不会数行** |
| **门禁自测** | 相信门禁 | **门禁自己也会错** |
| **链接** | 点一遍 | **1744 个文件点不完**，且**历史链接不会自己报错** |
| **索引新鲜度** | 记得更新 | **索引腐化速度 = 每轮一次** |
| **旧值残留** | grep | **grep 会命中「更正记录」**，需要排除规则 |

## 5 一次性全跑

```bash
cd noname-next
python3 -B scripts/check-source-lines.py; \
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'; \
python3 -B scripts/check_local_links.py; \
python3 -B scripts/refresh_recent_rounds.py --check; \
python3 -B scripts/check_stale_numbers.py; \
cargo test --offline -p engine-native 2>&1 | grep -E '^test result'
```

**注意**：**不要用 `| tail` 接这些命令** —— 管道会**吞掉 exit code**（本会话因此把红灯看成绿灯过）。
