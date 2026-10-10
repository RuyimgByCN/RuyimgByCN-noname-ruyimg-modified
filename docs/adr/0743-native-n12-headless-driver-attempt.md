# ADR 0743：N12 旧线无头驱动器——环境尝试与边界记录

日期：2026-09-16
状态：部分完成（可行性实证：本地网络在本工作区沙箱不可达）
关联：[0735](0735-native-n12-cross-process-archive.md)、[0742](0742-native-traitor-anchor-oldline-install.md)、进度文档 N12

## 背景与决定

N12 旧线等价负载对照需要驱动旧线客户端（浏览器引导的 DOM 应用）。本批实证
了两条加载路径的可行性边界：

### 1. 浏览器引导路径（已证实为唯一可行路径）

`dist/game/game.js` 为浏览器 bootstrap：使用 `location.protocol` 检测、
`document.head`、importmap 注入、service worker 注册——**Node + DOM shim
不可行**（需要 importmap/模块解析/UI 全链路）。可行路径为真实浏览器加载
`dist/index.html`（本会话具备 chrome-devtools/playwright 驱动能力）。

### 2. 本工作区沙箱的本地网络边界

为浏览器加载启动的本地 HTTP 服务（`python3 -m http.server 8771`，进程存活）
在本工作区沙箱内 **localhost 不可达**（curl 000/超时）。旧线自身拒绝
`file://` 协议。因此「浏览器加载 → UI 自动化进局 → 聚合统计提取」的
驱动器闭环需要在具备本地回环网络的环境执行（真机/非沙箱会话）。

### 3. 已就位的驱动器要素（待环境）

- 完整 `dist` 构建（ADR 0742）；
- HTTP 服务脚本一行即可启动；
- 聚合统计口径沿用 n09（命令数/事件数/守恒/终局）；
- chrome-devtools / playwright 自动化通道可用。

## 影响

- N12「等价负载对照」从「可行性未评估」推进到「驱动器闭环的最后一步 =
  具备本地回环网络的执行环境」；真机验证亦依赖该环境（同一前提）。
- 规则版本不变（13）；173 项测试不变。
