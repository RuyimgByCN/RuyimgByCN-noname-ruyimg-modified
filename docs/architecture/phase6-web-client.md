# 阶段 6：Web Client 与表现层

> 历史参考（2026-09-16 标注）：本文描述旧线及所注明日期的状态，文中的“当前”、阶段验收和统计不代表新原生线完成度。最新入口见 [旧实现参考](legacy-reference.md)。

## 边界

```text
Vue Shell (主线程)
  → EngineClient 消息
Web Worker
  → WASM engine-wasm (Rust Engine)
  → ObserverSnapshot JSON / EngineEnvelope
  → ViewStore + DOM Battle + Timeline
```

- **主线程不执行规则**：只做投影消费、输入、动画。
- **状态真相源**：Worker 内 Command Log + 权威状态；UI 日志仅为 Presentation 摘要。
- **隐藏信息**：`project_player` / `project_observer` 在 WASM 侧完成。
- **输入合法性**：本地 WASM 与联机服务器都返回同一 `LegalAction` 契约；UI 只呈现并提交
  其中的精确卡牌、目标与 Choice token，不复刻规则判断。

## 构建

```bash
# 1) WASM
cd apps/web-client && pnpm build:wasm
# 需要: rustup target wasm32-unknown-unknown
# 优先 wasm-pack；否则 cargo + wasm-bindgen-cli

# 2) 前端
pnpm install
pnpm dev      # http://localhost:5173
pnpm build
```

## 功能清单

| 功能 | 状态 |
|---|---|
| Vue 大厅 / 设置 / 对局 / 录像入口 | ✅ |
| WASM Worker 适配 | ✅ |
| Observer 座位/手牌 DOM | ✅ |
| 权威手牌/装备/判定/弃牌分区 | ✅（`CardView.zone/kind`） |
| WebSocket 建房/加入/观战/重连 | ✅ |
| 服务端个性化合法动作 | ✅ |
| 倍速 / 跳过动画 / reduced-motion | ✅ |
| 自动步进 / 连续自动 | ✅ |
| 选择屏障 UI | ✅ |
| 键盘焦点与 aria 标签 | 基础 ✅ |
| Pixi 特效层 | 占位（未强制依赖） |
| 完整录像逐步回放 | ✅（先全量校验，再按步重建授权投影） |

## 无障碍

- 导航与战场 `aria-label` / `role`
- `aria-live` 状态行
- 尊重 `prefers-reduced-motion`
- 焦点可见 outline
