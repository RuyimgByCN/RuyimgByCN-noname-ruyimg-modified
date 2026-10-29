# B9：胆守延迟效果状态哈希 panic 修复（2026-10-29）

## 范围、基准与授权

旧线缺陷修复，属于 PA12 冻结新增规则后仍允许的缺陷维护。主工作树提交基线 `95f51867a3648321da1cc92149b60e2bbfaae748`，含既有九批未提交工作；原始 dirty 清单和 SHA-256 见 [九批核验环境](../2026-10-29-nine-batches-precommit-verification/environment.json)及其逐文件清单。用户已另行明确授权九批分别入库，提交链正在隔离检出中重建；本批不混入历史九批。

原生规则 v328、存档 v105、ABI 753 均不变。设备 macOS 27.2 / Apple M3 Max，rustc/cargo 1.95.0；环境沿用同轮核验记录。

## 反例与共享根因

`canonical_state_hash` 分别遍历 `resolution.deferred_effects` 与 `deferred_tail_effects`，共享 `hash::dispatch::hash_effect`。`ResolveOlJieDanshou` 被路由到 `effects_draw_cards`，但该组缺少编码臂，进入 `unreachable!`。

先加 `hash::tests::b9_danshou_in_deferred_queues_hashes_player_and_skill`，在空状态延迟队列放入真实请求后求状态摘要；修复前立即 panic，见 [red.log](red.log)，exit 101。测试同时覆盖两类队列、相同输入稳定、非空效果改变摘要、仅玩家或技能实例变化时摘要不同。

修复在共享编码组补一臂：按相邻扩展效果惯例写 `0` 前缀、16 位效果 tag 771、玩家 ID、技能实例 ID。保留路由和未知组防御；不绕过摘要，不改原有任何效果字节。

## 验证

| 检查 | 实际结果 | 原始证据 |
|---|---|---|
| B9 定向修复前 | 0 passed / 1 failed，panic 命中既有 unreachable | [red.log](red.log)、[red.exit](red.exit) |
| B9 定向修复后 | 1 passed / 0 failed，两个队列及字段区分均通过 | [green.log](green.log)、[green.exit](green.exit) |
| `cargo test --offline -p engine-core --no-fail-fast` | **3848/0/1**，exit 0 | [engine-core-full.log](engine-core-full.log)、[engine-core-full.exit](engine-core-full.exit) |
| 旧线 golden 期望文件 | 与 HEAD 无差异；conformance 固定向量 ALL PASS，未重录 | [conformance-after-fix.log](conformance-after-fix.log) |
| `cargo test --offline --workspace --no-fail-fast`（桌面夹具修正后重新全跑） | **6729/0/74**，exit 0；首次失败仍单独保留 | [workspace-fixed-final.log](workspace-fixed-final.log)、[workspace-fixed-final.exit](workspace-fixed-final.exit) |
| WASM 从当前源码重建 | exit 0，部署 web 目标 | [build-native-current.log](build-native-current.log) |
| 真实 Worker smoke | **81/0**，exit 0 | [worker-smoke-current.log](worker-smoke-current.log) |
| native/WASM 对照 | `native_wasm_match: true`，exit 0 | [native-wasm-cross-target.log](native-wasm-cross-target.log) |
| 文档引用脚本单测（包含本轮补充的 Unicode 范围/斜线行号反例） | exit 0 | [doc-line-ref-tests.log](doc-line-ref-tests.log) |

每条后续命令的时间、耗时、退出码见 [verification-results.json](verification-results.json)和 [post-fix-results.json](post-fix-results.json)。`lens_diagnostics(source=lsp)` 对两个 Rust 文件均未取得服务端回答，不能据零诊断宣称干净；实际编译和全量测试结果作为验证依据。

## 同轮发现并独立处理的问题

初轮 workspace 为 **6726/2/74**，两个失败均为 desktop-client 的既有安装测试：夹具使用 `code=[1,2,3]` 且仅对代码字节取哈希。正式安装校验现在正确拒绝该无效模块，导致测试尚未走到冻结/签名断言。保留 [初轮完整结果](../2026-10-29-nine-batches-precommit-verification/workspace-tests.log)。

共享夹具改用现有 `rule_vm::assemble(&[Op::Halt])` 和 `BytecodeModule::compute_logic_hash`；不放宽安装校验，不删除签名/冻结断言。中间一次误将含数据的 `Op` 枚举强转成字节，造成 E0605 编译失败，见 [core-and-desktop.log](core-and-desktop.log)；修正编码后又验证了语义 hash 不匹配，见 [desktop-fixed.log](desktop-fixed.log)。最终 desktop-client **11/0/0**，见 [desktop-fixed-semantic-hash.log](desktop-fixed-semantic-hash.log)，再全跑 workspace 得上述 6729/0/74。最终全量通过不覆盖或抹去这些首次失败。

本轮还修正文档范围引用漏检及越界终点，详情与先红后绿见 [九批核验说明](../2026-10-29-nine-batches-precommit-verification/README.md)。这些是新发现的校验/夹具修复，与 B9 实际规则编码修复分别记录。

## 未验收边界

以上不代表 PA12 原生胆守/平寇移植、onlyOL 整包、37 来源、目标平台或上游规则等价完成。74 个忽略项仍保留。GitHub Actions、真机、全产品矩阵与后续 F7 拆分仍需各自验收。
