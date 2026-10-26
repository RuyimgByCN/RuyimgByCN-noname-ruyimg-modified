# PA08-3 Safari / Firefox / Chrome 有头整局基线（2026-10-26）

- 日期：2026-10-26；数据文件落盘 13:16–13:16、首次提交 13:18（cf987ab0）+08:00（原记「2026-10-26 13:00–15:30 +08:00」晚于提交或与文件时间矛盾，2026-10-26 17:30 扫描按数据文件时间与提交历史更正）
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第三切片（浏览器三引擎）
- 源码：`HEAD = 18edc033` 加未提交工作树（驱动修复见 §3）；采样与驱动同[ WKWebView 批](../2026-10-26-pa08-2-wkwebview/README.md)（autodrive + 外链驱动 `web-frame-external.mjs`）
- 环境：Mac15,10（M3 Max，ProMotion）；Chrome 156 / Firefox（正式版）/ Safari（正式版），窗口 ~1280×800，driver=auto（页面内点一次「连续自动」后由应用计时器推进），4 座身份局 seed 42，steps 1500 上限

**一句话结论**：三浏览器整局全部正常终局 **rev 280 反贼胜**——与 headless/有头 CDP 基线、Tauri WKWebView 完全一致（跨引擎宿主确定性不变）。帧节奏：Chrome/Firefox 按 120 Hz 合成（p50 8.3 ms），Safari 窗口 60 Hz（p50 17 ms）；合成点击回包 transfer p50 1–2 ms、Long Task 全部 0。

## 1 原始数据

[runs/](runs/)（每目录 summary.json / frames.csv / interactions.csv / event-timings.csv / long-tasks.csv；RESULT 行摘录见 [RESULT-lines.txt](runs/RESULT-lines.txt)）。

| run | 帧间隔 p50 / p95 / p99 / max | transfer p50 | input_to_paint p50 | long tasks |
|---|---:|---:|---:|---:|
| chrome-auto | 8.30 / 9.60 / 10.0 / 192 ms | 0.4 ms | 6.8 ms | 0 |
| firefox-auto | 8.34 / 9.98 / 10.3 / 50 ms | 1.0 ms | 9.0 ms | 0 |
| safari-auto | 17.0 / 20.0 / 29.0 / 73 ms | 2.0 ms | 16.0 ms | 0 |

- Safari 的 60 Hz 窗口合成与 WKWebView（[PA08-2](../2026-10-26-pa08-2-wkwebview/README.md)）一致——WebKit 系在 ProMotion 外接默认窗口均 60 Hz。
- 三引擎整局 wall 时间 2.5–4 分钟（120 Hz 快于 WKWebView 的 5 分钟档）。

## 2 与基线对照

| 指标（auto 档 p50） | Chrome headless 60Hz | Chrome 有头 CDP 120Hz | 本批 Chrome | 本批 Firefox | 本批 Safari | WKWebView |
|---|---:|---:|---:|---:|---:|---:|
| 帧 p50 | 16.7 ms | 8.3 ms | 8.3 ms | 8.3 ms | 17 ms | 16.7 ms |
| transfer | 1.1 ms | 1.5 ms | 0.4 ms | 1.0 ms | 2.0 ms | — |
| input_to_paint | 15.3 ms | 12.9 ms | 6.8 ms | 9.0 ms | 16.0 ms | — |

## 3 驱动修复记录（本轮排障的主产物）

本批启动前外链驱动在三个浏览器上整局零报告，逐项定位出三个独立缺陷（修复都在 `apps/web-client/scripts/web-frame-external.mjs`）：

1. **路由形态**：`router.ts` 按宿主分叉——Tauri 用 `createWebHashHistory`、浏览器用 `createWebHistory`。驱动此前统一发 `/#/battle?…`，浏览器端路径落在 `/`，页面全程停在大厅（URL 正确但视图是大厅）。修复：battle URL 按 `--app` 取形（tauri=hash、浏览器=path `/battle?…`）。
2. **注入重载循环**：静态服务对所有 HTML 注入 `location.replace(battleUrl)`；path 路由下 `/battle` 直开也会被注入，replace 到「同路径不同 query」触发整页重载→再注入→再 replace＝无限循环（Tauri 免疫因 hash replace 不换文档）。修复：仅根路径 `/` 注入（Tauri devUrl 场景），battle 直开不注入。
3. **跨源载荷地址**（影响 PA08-4）：Tauri devUrl 是 `localhost:5173` 而驱动绑 `127.0.0.1`，绝对地址 `http://127.0.0.1:5173/packs|/hex` 跨源被 WKWebView/CSP 拦截。修复：`packsUrl`/`hexUrl` 用相对路径。

附注：排障早期曾按视觉模型读数怀疑「Safari/Firefox 引擎链路挂起」，事后证明为大厅页误读；`--driver manual` 的 Firefox 真实点击采样因窗口遮挡未获有效样本，PA07 在 Gecko 的真实输入数据仍缺（见台帐未覆盖）。

## 4 未覆盖

1. Safari/Firefox **真实点击**（manual 档）样本——本轮两次尝试因窗口遮挡失败，需独占前台重跑；PA07 的 Gecko/WebKit 浏览器真实输入复现待此补齐（WKWebView 已有 n=14 达标数据）。
2. 移动端（PA08-6）、≥30 分钟长时（PA08-5）。

## 5 本批验证

| 命令 | 结果 |
|---|---|
| `node scripts/web-frame-external.mjs --app chrome-headed/firefox/safari`（driver auto） | 3/3 正常终局 rev 280，Long Task 0 |
| `pnpm --dir apps/web-client test` / `vue-tsc --noEmit` | 80 passed / 0 failed；0 错误 |
| `python3 -B scripts/check-source-lines.py` / `check_local_links.py --all` | 通过；坏链 0 |
