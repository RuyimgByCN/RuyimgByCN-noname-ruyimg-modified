# NC03／NC09：原生零号座位回合提示

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [d0ce92d0756c606e058a9113dbc680a3920e0070](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 与消息结构不变。

## 反例与修复

本地原生观察视图使用零基座位，`current_player = 0` 是真实首座；旧线／服务端的 `PlayerId::NONE = 0` 表示回合间无当前玩家。战斗页此前一律排除 0，原生第 0 座的回合栏消失；准备和结束阶段也缺少中文标签。

[回归测试](../../../../apps/web-client/scripts/nc03-turn-banner.test.mjs)从战斗页提取实际 computed 表达式并使用 Vue reactive 状态。干净 HEAD 导出前端叠加测试 **1／2／0**（[日志](red-clean.log)，exit 1）；修复后 **3／0／0**（[日志](green-targeted.log)），覆盖 0 座、非 0 座、准备／结束标签、旧线零哨兵、事件回退与 Idle 隐藏。

判断视图座位列表中是否实际存在玩家 0：存在则允许显示，否则保留原有哨兵／事件回退；非 Idle 的其他玩家不受影响。[真实 Chrome 驱动](browser-turn.mjs)只观察生产页面和原生 Worker 投影，未修改请求或回包：修复前 **1／2**（[日志](browser-red.log)，第 0 座判定及结束阶段无回合栏）；修复后 **3／0**（[日志](browser-green.log)，第 0／1 座和结束阶段均与真实投影一致，控制台异常 0）。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`pnpm --dir apps/web-client smoke:native` | 前端 **108／0／0**、类型检查通过、真实 WASM Worker 冒烟 **62／0**，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)、[冒烟](native-smoke.log)）。 |
| `pnpm --dir apps/web-client exec vite build --outDir /tmp/nc03-turn-dist.IxKOoJ` | 生产前端构建通过（[日志](frontend-build.log)）；Rust 源码未改，复用前批生成的 web WASM（[摘要](wasm-sha256.txt)、[构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `node browser-turn.mjs <dist> <Chrome>` | 基准 **1／2**、修复后 **3／0**，详见上文。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。复用前批独立 target 与映射。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白规范化，不改变结果。 |

NC03／NC09 保持部分；本批不证明真实手牌投影、完整联机对局与目标平台验收。原有八份文档修改保留，主稿仅暂存本批追加行。
