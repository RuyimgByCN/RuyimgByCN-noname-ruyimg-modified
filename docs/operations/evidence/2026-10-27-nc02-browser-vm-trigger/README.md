# NC02：浏览器内预置 VM 副本的触发与恢复

- 实际执行：2026-10-27 22:48:44 +0800 → 2026-10-27 22:51:30 +0800（开始与结束各运行 `date '+%F %T %z'`）；开工时源码 `3a2a2770294836da944e85bb976e04828fef9d33`（[时间](start-time.txt)、[源码](source-head.txt)）。本批没有改产品代码、规则、存档格式或 ABI。
- 临时浏览器 profile 中直接预置完整 `official/vm-fixture@0.1.0` 安装副本和启用内容锁；**没有通过当前拒绝 VM 包的本地安装界面**。因此这是正式对局与存档链路的定向验证，不是安装全流程验收。

## 可复现环境与结果

使用[上一轮](../2026-10-27-nc02-browser-vm-lifecycle/README.md)在干净检出 `4677e2adeb5ac4c27b8ddbc89ed09c6928b76875` 构建的产品 dist；`git diff --exit-code 4677e2adeb5ac4c27b8ddbc89ed09c6928b76875 3a2a2770294836da944e85bb976e04828fef9d33 -- apps/web-client crates content` 为 exit 0（[差异输出](production-source-diff.log)），即本批使用的前端、内核、加载器、内容源码与构建时一致。原生 WASM 构建来源及其一致性见上一轮证据。脚本在隔离临时端口提供 dist，启动自己的 headless Chrome 和 profile，结束时只停止其自身进程及服务器。通过 CDP 点击真实界面；Worker 监听仅观察现有回包，未更改命令或返回值。

```bash
node docs/operations/evidence/2026-10-27-nc02-browser-vm-trigger/browser-vm-trigger.mjs \
  /tmp/noname-nc02-browser.a0A7aT/dist "$PWD" \
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  docs/operations/evidence/2026-10-27-nc02-browser-vm-trigger/screenshot.png
```

[脚本](browser-vm-trigger.mjs)最终 **7/7、exit 0**（[完整输出](browser-vm-trigger-final.log)、[截图](screenshot.png)）。正式 `prepareNativeMatchContent` 读取预置副本后，在选将界面给 P1 装配夹具；回包断言 P1 持有 `vm_fixture_phase_end_phase_end`。单步推进至 P1 弃牌阶段，界面存档携带真实 VM 包和内容锁；点击「结束阶段」后进入 End，UI 和 Worker 投影均为手牌 **4→5**，且有公开摸牌事件。读档恢复到原状态哈希 `baf3b7b2…`，再次点击得到相同最终哈希 `d04f0194…`。同种子、同安装包但 P1 未装配 VM 技能的对照为 **4→4**，且没有摸牌事件。

首次与第二次脚本尝试 exit 1（[首次](browser-vm-trigger-01.log)、[第二次](browser-vm-trigger-02.log)）：新脚本误以为 Worker 返回原始 `CardsDrawn` 事件。实际观察者只收到公开事件码；第二次日志记录 `PublicEvent.code=2`，修正测试断言后通过。并非产品套件失败；两次均使用上述同一干净生产构建。浏览器异常计数为 0。

## 验证与边界

| 命令 | 完整结果 |
|---|---|
| `node browser-vm-trigger.mjs ...` | **7/7，exit 0**；选将、存档包副本、效果、恢复重放及无技能对照均通过（[日志](browser-vm-trigger-final.log)） |
| `pnpm --dir apps/web-client test` | **80 passed / 0 failed / 0 skipped**（[日志](frontend-test.log)） |
| `pnpm --dir apps/web-client typecheck` | exit 0（[日志](frontend-typecheck.log)） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **1 passed / 0 failed**（[日志](source-line-tests.log)） |
| `python3 -B scripts/check-source-lines.py` | **8621** 文件均不超过 2000 行（[日志](source-lines.log)） |
| `python3 -B scripts/check_local_links.py --all` | **1830** 文件、0 坏链（[日志](local-links.log)） |
| `python3 -B scripts/check-adr-index.py` | exit 0；未登记冲突、过时登记、内容不符及坏链均为 0（[日志](adr-index.log)） |

本批没有重新运行 Rust 套件；相同产品源码的 native、wasm、loader 与 server 全量结果分别见[触发修复](../2026-10-27-nc02-vm-trigger-proof/README.md)和[房间重启](../2026-10-27-nc02-vm-room-restart/README.md)。仍需真正通过本地安装入口的 VM 包生命周期、跨包依赖和多类型覆盖；不能把此一个夹具或 headless Chrome 结果当作 NC02 完成或多平台验收。
