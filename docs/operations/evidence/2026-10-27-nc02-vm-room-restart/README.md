# NC02：VM 武将房间跨进程冻结恢复

- 实际执行：2026-10-27 22:39:53 +0800 → 2026-10-27 22:45:57 +0800（[实测记录](end-time.txt)）。源码基准 `77cf044e6d01fd21eb56fdc5872964e9a35562a0`（开工时 `git rev-parse HEAD`）。本批只添加测试，规则、存档、协议和 ABI 未变。
- 临时目录包含独立的来源、包库、房间库和子进程；没有操作用户安装库。初始选将使用已有房间 actor 的内部接口，**没有**验证联机客户端的选将 UI 或新增 WS 选将协议。

## 反例与本批验证

先前的 WS 跨进程恢复使用数据包，VM 冻结副本的真实效果只在进程内核验；二者不能证明同一 VM 武将的房间跨进程恢复。本回归使用已提交的 `vm-fixture` 包及带真实模块引用的武将，经过正式 `NativeContent` 安装、原生房间 actor 选将及落盘，在待回答的弃牌选择处保存。

测试二进制启动独立服务子进程，使用正式 `bootstrap`、房间恢复和 WS 路由；原座位凭令牌重连，视图、权威摘要、合法动作和未决选择与保存前一致。测试结束该子进程，停用临时来源并调用正式卸载接口，再启动第二个独立服务子进程；启动完成后再次确认已安装列表和活动装载均无该包。它从房间冻结副本恢复并接受相同令牌：弃置 2 张后，通过 WS 进入结束阶段，手牌增加 1 张，符合该 VM 技能效果（[定向输出](targeted-final.log)）。房间测试运行在服务端测试二进制中，并非另行启动发布版主程序。

## 验证结果

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p game-server --bin game-server native_ws_e2e::commit_crash::native_vm_frozen_room_recovers_after_process_exit_and_uninstall -- --exact --nocapture` | **1/0/0**；跨两个服务子进程恢复成功，技能实际摸 1（[最终日志](targeted-final.log)、[此前日志](targeted.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | exit 0；内核 **1584/0/71**、WASM **52/0/0**、加载器 **32/0/0**、服务端 **360/0/0**（通过/失败/忽略；[计数](rust-full-counts.txt)、[完整日志](rust-full.log)）。运行中补充了重启后未重装的断言，最终测试文件另复跑完整服务端。 |
| `cargo test --offline -p game-server --no-fail-fast` | 最终测试文件 **360/0/0**、exit 0（[计数](game-server-counts.txt)、[最终日志](game-server-final.log)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **80/0/0**；类型检查 exit 0（[测试](frontend-test.log)、[类型检查](frontend-typecheck.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；**8621** 文件通过（[单测](source-line-tests.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1829** 文件、0 坏链；ADR 检查 exit 0（[链接](local-links.log)、[ADR](adr-index.log)）。 |

这是一条 VM 武将、弃牌选择和主动结束阶段的样本；跨包依赖、其他技能形态、UI 中的实际触发及完整生命周期仍未验收，NC02 保持部分。
