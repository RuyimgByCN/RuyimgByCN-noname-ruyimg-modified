# PA08-6 移动端采样（iOS / Android 模拟器，2026-10-26）

- 日期：2026-10-26 18:20–19:50 +08:00
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第六切片（移动端）
- 源码：`HEAD = d77e4cd4`；采样栈与外链驱动（`--app none` 静态服务 + 系统打开 URL）同[前批](../2026-10-26-pa08-3-browsers/README.md)
- 环境：Mac15,10 宿主。**iOS**：iPhone 17 模拟器（iOS 27.0 runtime，已启动实例复用），模拟器 Safari（模拟器直连宿主 `127.0.0.1`）。**Android**：AVD codex35（Pixel 系、Android 15/API 35、arm64），系统浏览器（宿主别名 `10.0.2.2`）。4 座身份局 seed 42、driver=auto。

**一句话结论**：**iOS 模拟器整局全绿**——终局 rev 280 与全部桌面引擎一致、帧 p50 17 ms（60 Hz）、transfer p50 2 ms、Long Task 0。**Android 运行面验证通过**——原生引擎装载、对局以正常节奏持续推进（rev 12→96→142 抽查递增），但整局报告未在驱动窗口内送达（浏览器计时器节流使页面 deadline 顺延；25 min 窗口两次耗尽），整局量化数据待补。

## 1 iOS（iphone17-sim，整局）

原始数据：[runs/ios-iphone17-sim/](runs/ios-iphone17-sim/)；RESULT 摘录 [runs/ios-RESULT.txt](runs/ios-RESULT.txt)。

| 指标 | 值 |
|---|---|
| 终局 | **rev 280 反贼胜**（与桌面全引擎一致） |
| 帧间隔 p50 / p95 / p99 / max | 17 / 20 / 32 / 66 ms（60 Hz） |
| input_to_paint p50 / p95 | 16 / 20 ms |
| roundtrip / transfer p50 | 3 / 2 ms |
| Long Task | 0 |

iOS WebKit 与桌面 Safari/WKWebView 同量级——移动端表现层口径与桌面 WebKit 家族一致。

## 2 Android（运行验证）

- 前置：`svc power stayon true`＋`screen_off_timeout` 拉长（息屏会把 rAF 节流到零，首跑 15 min 零报告的根因）；宿主以 `10.0.2.2` 寻址。
- 实测：页面正常加载（path 路由）、原生引擎装载（状态行 `… · native`）、自动对局持续推进（抽查 rev 12 → 96 → 142，约 7 rev/min——模拟器 arm64 WASM 慢于真机预期）。
- 未获整局报告：页面 22 min deadline 因浏览器计时器节流实际顺延，25 min 驱动窗口两次耗尽。**登记待办**：加长窗口（≥40 min）或缩短负载（steps 上限/半局）后补测；真机（USB）预期显著快于模拟器。

## 3 未覆盖

1. Android 整局量化数据（见 §2 待办）；
2. 真机（iOS 物理设备/Android USB 设备）——模拟器已证可行性与量级；
3. 移动端真实触控路径（manual 档）——iOS 模拟器无 idb（UI 自动化后端不可用），Android 触控注入未做。

## 4 本批验证

| 命令 | 结果 |
|---|---|
| `xcrun simctl openurl <udid> <battle-url>`（iOS Safari） | 整局终局 rev 280，Long Task 0 |
| `android_open_url`（codex35） | 运行验证通过（rev 持续推进），整局报告未达窗口 |
