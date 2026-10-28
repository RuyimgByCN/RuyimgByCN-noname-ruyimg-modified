# NC03／NC04：响应式动作数组与 Worker 发送失败清理

- 实际执行：[开始](start-time.txt) → [结束](end-time.txt)；源码基准 [6b6f15f15f1ef3cb78359074e5567a5fb7741235](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 与 Worker 消息字段不变。

## 反例与修复

战斗组件从响应式状态取目标和选择数组，`EngineClient` 原样传给 `Worker.postMessage`；Vue 代理不可结构化克隆，出牌等请求会先在主线程抛 `DataCloneError`，未抵达引擎。同步发送失败还遗留 pending 请求、性能采样记录及 SAB 唤醒。上批首先在浏览器发现此问题（[上批证据](../2026-10-28-nc03-local-rejection-ui/README.md)）。

本批在四个动作入口复制数值数组，覆盖出牌目标、技能目标／选牌、技能牌目标及选择答案；数值顺序、消息字段与 Worker 校验不变。发送同步失败时删除该请求及未发送的性能样本；无其他请求在途时结束 SAB 唤醒，并原样向调用方返回异常。其他在途请求仍保持唤醒。

干净 HEAD 导出前端并叠加[回归测试](../../../../apps/web-client/scripts/nc03-worker-clone.test.mjs)为 **0／4／0**（[日志](red-clean.log)，exit 1）：原始响应式数组无法克隆、失败请求残留，以及缺少性能失败清理；修复后 **4／0／0**（[日志](green-targeted.log)）。测试从生产文件提取完整客户端类，使用真实 Vue reactive 数组和 `structuredClone` 检验消息，并模拟发送异常、重试、并发请求与性能样本清理。首次测试夹具未剥离编译产生的模块语法而报语法错误（[日志](red-harness-initial.log)），修正后才取得上述反例；[早期两项反例](red-clean-two.log)也保留。

[浏览器驱动](browser-clone.mjs)使用真实 Chrome、生产前端与原生 WASM Worker；因当前视图尚缺手牌信息，测试只在 UI 投影注入虚构牌和动作，**没有解包或转换目标数组**。基准出牌报克隆错误、没有 Worker 回包（[日志](browser-red.log)，**2／1**）；修复后命令抵达真实引擎并以 `WrongPhase` 受控拒绝，原有结束阶段拒绝和自动停止回归同过（[日志](browser-green.log)，**3／0**，控制台异常 0）。这不等于真实手牌投影和完整出牌产品流程已验收；联机、真实挂起续行、B7 和其他场景仍待完成。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck`；`pnpm --dir apps/web-client smoke:native` | 前端 **105／0／0**、类型检查通过、真实 WASM Worker 冒烟 **62／0**，均 exit 0（[测试](frontend-test.log)、[类型](typecheck.log)、[冒烟](native-smoke.log)）。Node 的 localStorage 实验性提示不影响测试。 |
| `pnpm --dir apps/web-client exec vite build --outDir /tmp/nc03-clone-dist.vbVeUG` | 生产前端构建通过（[日志](frontend-build.log)）；Rust 源码未改，复用前批生成的 web WASM（[摘要](wasm-sha256.txt)、[构建](../2026-10-28-nc03-wasm-actor-bounds/build-native.log)）。 |
| `node browser-clone.mjs <dist> <Chrome>` | 基准 **2／1**、修复后 **3／0**，详见上文。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/nc03-actor-final-target.vSu9e4 cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | 内核 **1612／0／73**、WASM **66／0／0**、加载器 **32／0／0**；合计 **1710／0／73**、358 个测试目标，exit 0（[日志](rust-full.log)、[计数](rust-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)）。复用前批独立 target 与映射。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 均通过（[自检](source-line-tests.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)、[暂存](staged-check.log)）。日志行尾空白规范化，不改变结果。 |

NC03／NC04 保持部分。原有八份文档修改保留，主稿仅暂存本批追加行。
