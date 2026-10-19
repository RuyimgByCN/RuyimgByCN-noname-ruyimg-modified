# NC12 移动档位：**视口模拟**测量（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）
- 规则版本 v322（工作树）/ 存档格式 v105
- **性质声明（必读）**：本节是 **CDP 视口模拟**（`Emulation.setDeviceMetricsOverride` +
  触摸 + 移动 UA），**不是真机** ✗ —— **不能**用于宣称「Android/iOS 档位已验收」。

## 1 方法

`apps/web-client/scripts/nc12-browser-coldstart.mjs` 新增**第 6 参数**（`desktop` 默认 | `mobile`）：

| 参数 | 模拟内容 |
|---|---|
| `mobile` | CSS 视口 **390×844**、`deviceScaleFactor 3`、`mobile: true`、触摸 5 点、iPhone 级 UA |
| `desktop` | 不覆盖（沿用 `--window-size=1280,800`） |

两者**同一**构建产物、同一静态服务、同一 headless Chrome、同一脚本（3 次迭代）。

## 2 结果

| 档位 | to_interactive_ms | p50 | max | all_interactive | console_errors |
|---|---|---:|---:|---|---|
| **mobile（视口模拟）** | `[262, 263, 264]` | **263 ms** | 264 ms | true | `[]` |
| desktop | `[258, 260, 262]` | **260 ms** | 262 ms | true | `[]` |

原始输出：[mobile-run.txt](mobile-run.txt)、[desktop-run.txt](desktop-run.txt)。

**差值**：移动视口 p50 比桌面 **+3 ms（+1.2%）** —— 在**视口模拟**范围内**无可感知劣化**；
但**不**代表真机（真机还涉及 CPU 性能、GPU、内存压力、触摸事件分发、真实网络）。

## 2b 触摸手势端到端（**视口模拟**，2026-10-19 同轮）

新增 `apps/web-client/scripts/nc12-mobile-touch.mjs`：在**移动视口 + 触摸模拟**下用 CDP
`Input.dispatchTouchEvent`（touchStart → touchEnd）**真实触摸点击**「本地开局」，并检查是否进入对局。

**实测**（[mobile-touch.txt](mobile-touch.txt)、`mobile-touch-report.json`）：

```json
{"tapped_at":{"x":289,"y":407},
 "url_after_tap":"/battle?players=4&seed=42",
 "interactive":true,
 "viewport":"{\"w\":390,\"h\":844,\"touch\":true}",
 "console_errors":[]}
```

→ ① 触摸事件**成功驱动**路由跳转到对局页；② 对局页「自动一步」按钮**存在且可用**（`interactive=true`）；
③ 视口确为 **390×844 且 `ontouchstart` 存在**；④ **0 console error**。

**口径**：仍是 **CDP 视口/触摸模拟**，**不是真机** ✗（真机的触摸分发、GPU、内存压力、真实网络未覆盖）。

## 3 未验收（照旧，**不得**用本节替代）

- [ ] **Android / iOS 真机**档位（无设备，**平台验收无法在本机完成**）
- [ ] 桌面壳（Tauri 等）档位
- [ ] 真实网络（弱网/高延迟）下的冷启动与可交互
- [x] 触摸交互端到端**已在视口模拟下验收**（`Input.dispatchTouchEvent` → 进入对局且可交互）——
      **真机**触摸路径仍未验收

## 4 与主稿的关系

本节结论**仅**作为「移动视口下首屏可交互」的**补充证据**；NC12 移动/桌面档位在
[进度主稿](../../../architecture/native-progress.md)中**仍列为未验收**。
