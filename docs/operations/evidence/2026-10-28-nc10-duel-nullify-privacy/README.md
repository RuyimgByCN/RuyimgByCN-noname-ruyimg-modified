# NC10：决斗无懈窗口的隐藏手牌边界

- 实际执行：2026-10-28 01:28:12 +0800 → 2026-10-28 09:02:04 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `13d4bc2a09aac83bc306394b6b47e3db7a966be3`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。保留已有八处未提交文档改动。

## 反例与处理

标准牌局 seed 46 按真实发牌、两次推进和座位 0 用决斗牌 58 指向座位 1，进入座位 2 的 `nullify_window`；其可选项为放弃或无懈牌 71。把座位 0 的暗牌 75（乐不思蜀）与牌库的 88（雷杀）互换，不改变座位 2 的视图、选择、自身手牌牌种、事件帧或挂起机器；两张牌均非无懈，不改变已跳过座位的询问历史。旧规划器在变体 1、48 节点、两层搜索时分别选择放弃和使用无懈。用 `git archive HEAD` 导出干净源码，仅叠加[最终回归测试](../../../../crates/engine-native/tests/nc10_duel_nullify_privacy.rs)，使用独立 `CARGO_TARGET_DIR` 复现 **2 passed / 2 failed / 1 ignored，exit 101**（[日志](clean-final-test-repro.log)；[更早一次复现](clean-repro.log)）。

共因是 `redact_opponent_hands` 原先只允许已核对引用的 `duel_sha` 窗口做内容无关填充；首个无懈窗口继续在真实对手手牌上模拟。现将同一检查扩展到**单帧、单挂起机器、内层未开始决斗的首个无懈窗口**：核对事件、来源与目标、牌桌、队列当前询问者、窗口状态以及所有选择牌属于观察者且为无懈；延用已有特殊牌区、VM 私有状态与牌引用保守检查。合格时只在模拟副本内确定性填充未知手牌和牌库，预算为零的合法动作回退也使用该副本。权威规则、状态摘要编码、规则版本 v323、存档格式 v105 和 ABI 均未改；AI 在该窗口的动作可因不再读取真实暗牌而改变。

回归覆盖三个策略变体、预算 0/1/48、两层搜索的相同输入和相同动作/节点数，两种应答在脱敏及恢复后的可提交性，正常档与脱敏档恢复，特殊牌区保守分支，及实体牌不重复；同时重跑原先 `duel_sha` 隐私测试。前后均在同台机器、同工具链、同 seed/预算、debug 测试中预热 32 次并采 500 次：旧 p50/p95/p99 **236/258/290 µs**，新 **240/273/317 µs**（[环境](environment.log)、[旧](latency-before.log)、[新](latency-after.log)）。这是单场景采样，不作为端到端或跨平台性能结论。

## 验证

| 命令 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc10_duel_nullify_privacy --test nc10_pending_duel_privacy -- --nocapture` | 新测试 **4/0/1**、既有决斗响应测试 **3/0/1**，exit 0（[日志](targeted.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | engine-native **1592/0/73**、wasm **61/0/0**、loader **32/0/0**、game-server **360/0/0**，合计 **2045/0/73**，exit 0（[完整日志](full-tests.log)、[统计](test-counts.txt)、[统计脚本](count-tests.py)与[目标映射](test-artifacts.jsonl)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | 构建 exit 0；Node 中实际 Web WASM 与 Worker 后端适配层冒烟 **60/0**，没有启动浏览器 Worker 线程（[构建](wasm-build.log)、[冒烟](wasm-smoke.log)、[源码对照](wasm-source-match.log)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端测试 **87/0/0**，typecheck exit 0（[测试](frontend-tests.log)、[类型检查](typecheck.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8635 个维护文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 1842 个文件、坏链 0；ADR 通过（[链接](local-links.log)、[ADR](adr.log)）。 |
| `rustfmt --edition 2021 --check crates/engine-native/tests/nc10_duel_nullify_privacy.rs`；`git diff --check` | exit 0（[格式](rustfmt.log)、[diff](diff-check.log)）。 |

本地链接首次检查因本批 `diff-check.log` 尚未生成而失败（[首次日志](local-links-first.log)）；生成该日志后重跑通过。暂存检查也先发现 4 份新日志末尾多余空行，已移除并重跑（[首次日志](diff-check-first.log)）。

本轮同时更正 [NC02 包摘要](../2026-10-28-nc02-vm-package-hash/README.md)、[NC02 模块指纹](../2026-10-28-nc02-vm-module-hash/README.md)和 [NC04 指令边界](../2026-10-28-nc04-vm-instruction-boundaries/README.md)证据：此前的 Node 冒烟直接调用真实 Web WASM 与 Worker 后端，未创建浏览器 Worker 线程。

## 仍未完成

这里只覆盖上述形态；其他无懈链、其他挂起机器、特殊牌区以及 VM 私有状态仍可能让规划使用真实对手手牌，不能宣称 NC10 的全部信息边界已完成。不同模式/座位/种子/预算的 AI 质量基线及产品链路仍待验收。既有 `content-compiler` ABI 755 未使用项仍产生 warning，详见日志。
