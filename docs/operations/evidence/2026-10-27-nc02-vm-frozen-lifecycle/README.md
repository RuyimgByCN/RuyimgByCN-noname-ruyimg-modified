# NC02：真实 VM 技能的升级、回滚及冻结副本恢复

- 实际执行：2026-10-27 22:31:23 +0800 → 2026-10-27 22:38:44 +0800（[实测记录](end-time.txt)）。源码基准 `050757472a129440a4c8c10657b676efe35c1c96`（开工时 `git rev-parse HEAD`）。仅修改测试夹具和断言，规则、存档与 ABI 未变。
- 范围：服务端 `NativeContent` 安装、激活、回滚、卸载与冻结包加载，加原生引擎恢复和实际技能结算；不把进程内测试称为真实 WS 房间重启或 UI 验收。测试使用临时来源及包库，不操作用户安装库。

## 反例与验证

原冻结包夹具只有 VM 模块和二进制资产，没有武将及技能引用；原有测试断言模块数量和恢复后的摘要，无法证明升级或卸载后实际执行了正确的 VM 技能。本批补充带正确模块名的武将资产及其内容指纹。

新回归先选择该武将并确认技能归属。v1 技能在结束阶段回复 1 点，v2 修改为回复 2 点并具有不同逻辑指纹。旧档用 v2 直接恢复因 `RuleVersionMismatch` 被拒，v2 新局确实回复 2 点；原局、升级期间用 v1 冻结副本恢复、回滚至 v1 后恢复、物理卸载后用冻结副本恢复四条分支都在结束阶段回复 1 点，状态摘要相同。卸载后不带冻结副本的恢复被拒。断言使用 `NativeContent::load_room_packages`，没有启动和重启房间进程。

## 命令与结果

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p game-server --test native_frozen_packages` | 最终夹具 **7/0/0**（[日志](targeted-exact-error.log)）；此前只改版本号和加强新版效果的定向运行亦通过（[记录](targeted.log)、[记录](targeted-final.log)、[记录](targeted-upgrade.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | exit 0；内核 **1584/0/71**、WASM **52/0/0**、加载器 **32/0/0**、服务端 **359/0/0**（通过/失败/忽略；[计数](rust-full-counts.txt)、[日志](full-rust.log)）。该命令运行期间加强了测试中的 v2 效果，因此最终版本另复跑完整 game-server。 |
| `cargo test --offline -p game-server --no-fail-fast` | 最终断言确定后启动的完整复跑 **359/0/0**、exit 0（[计数](game-server-counts.txt)、[日志](game-server-verified.log)）；此前复跑中加严拒绝原因（[日志](game-server-final.log)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **80/0/0**；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；**8621** 文件均不超过 2000 行（[单测](source-line-tests.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1828** 文件、0 坏链；ADR 检查 exit 0（[链接](local-links.log)、[ADR](adr-index.log)）。 |

NC02 仍为部分：真实房间重启后的 VM 执行、UI 内技能触发、跨包依赖和多类型产品生命周期尚未由本切片覆盖。
