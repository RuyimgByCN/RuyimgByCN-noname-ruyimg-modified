# NC02：修正 VM 夹具后的真实浏览器与服务端生命周期

- 实际执行：2026-10-27 22:25:50 +0800 → 2026-10-27 22:29:26 +0800（开始和结束各运行 `date`）；源码基准 `4677e2adeb5ac4c27b8ddbc89ed09c6928b76875`（开工时 `git rev-parse HEAD`）。本批为产品实测与回填，未修改规则、存档或 ABI。
- 使用隔离的临时服务端内容库、drop-in 根和 Chrome profile；没有操作用户的安装库。开始时已有 8 个在途文档修改，保留其内容。

## 环境和可复现入口

在上述 HEAD 的独立检出里构建当前生产前端，`git status --porcelain` 在构建前后均为空（[前](worktree-status.log)、[后](worktree-status-after.log)）。原生 WASM 来自上一批的干净验证提交，比较确认该提交和本 HEAD 的 `apps/`、`crates/` 无差异（[差异](wasm-source-difference.log)）；旧线 WASM 从本地已有产物复制进检出的忽略目录，本次只走设置页，不验证旧线执行。

通过 `cargo build --offline -p game-server` 构建实际服务端（[构建](server-build.log)），使用 `emit_vm_fixture` 在临时 drop-in 生成编译器输出；生成结果与已提交夹具的包和角色资产 SHA-256 各自一致（[摘要](fixture-sha256.log)）。前端在临时路径用 `apps/web-client/node_modules/.bin/vite build --outDir <临时 dist>` 构建（[输出](vite-build-02.log)）；首次用 `pnpm exec vite` 在独立检出中找不到本地 `.bin`（exit 254，[记录](vite-build.log)），原因是依赖目录设在检出外，改用现有 Vite 可执行文件绝对路径后成功，源码仍为干净 HEAD。

临时目录、端口见 [路径](run-root.txt) 与[端口](ports.txt)。游戏服务端使用 `GAME_SERVER_ADDR=127.0.0.1:<服务端端口>`、`NONAME_CONTENT_STORE_ROOT`、`NONAME_EXTENSIONS_ROOT`、`NONAME_NATIVE_PACKAGES_ROOT`、`NONAME_NATIVE_STORE_ROOT` 指向该临时目录，`NONAME_NATIVE_ROOM_STORE` 置空；用 `python3 -m http.server <前端端口> --bind 127.0.0.1 --directory <临时 dist>` 提供构建产物。两个监听进程和端口均在[身份记录](service-identity.log)确认，测试后仅停止这两个本批启动的进程。静态服务对缺少的 favicon 返回 404（[日志](static-server.log)），不影响本测试；浏览器未报告 JavaScript 异常。

浏览器脚本：[browser-e2e.mjs](browser-e2e.mjs)。命令：`node browser-e2e.mjs http://127.0.0.1:<前端端口>/ http://127.0.0.1:<服务端端口> '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' <临时目录> <截图路径>`。脚本启动自己的 headless Chrome、使用其随机 CDP 端口，点击产品设置页面并由浏览器直接向同一真实服务端发送 HTTP 请求；断言失败会以非零码退出。

## 结果

真实浏览器显示 `official/vm-fixture@0.1.0`，同一服务端以该包选择 `vm_fixture` 开局并经 289 步到终局；相同包的 HTTP 建档、恢复成功。通过设置页面依次停用、启用、扫描安装、物理卸载、重扫描安装；停用与卸载时恢复均明确 HTTP 400，重新启用及以相同字节重装后原档恢复成功；浏览器 JavaScript 异常 0。完整断言为 **13/13、exit 0**（[输出](browser-e2e.log)、[退出码](browser-e2e.exit)、[截图](browser-settings.png)）。

HTTP 对局摘要不暴露技能归属或触发事件，因此仅证明正确夹具的安装、选将、对局与上述生命周期；不能单凭完成对局推断技能触发。真实 Worker 的技能归属、阶段效果和恢复重放，以及服务端进程内实际触发，已在[上一切片](../2026-10-27-nc02-vm-trigger-proof/README.md)分别断言。本批未验证 UI 中真实运行该 VM 技能、VM 包升级/回滚、冻结房间在卸载后的恢复或跨包依赖，NC02 仍为部分。

| 命令 | 结果 |
|---|---|
| `cargo build --offline -p game-server` | exit 0（[日志](server-build.log)） |
| 干净 HEAD 的 Vite 生产构建 | exit 0（[日志](vite-build-02.log)） |
| `node browser-e2e.mjs ...` | 13 项断言全部通过，exit 0（[日志](browser-e2e.log)） |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **80/0/0**；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | 单测 **1/0**；**8621** 文件均不超过 2000 行（[单测](source-line-tests.log)、[行数](source-lines.log)） |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1827** 文件、0 坏链；ADR 检查 exit 0（[链接](local-links.log)、[ADR](adr-index.log)） |

本批未改产品源码；相同源码的完整 engine-native、engine-native-wasm、content-native-loader、game-server 与工作区检查结果见[上一切片](../2026-10-27-nc02-vm-trigger-proof/README.md)，不重复宣称本批新跑。
