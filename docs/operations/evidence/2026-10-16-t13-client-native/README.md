# T13 产品面：web-client 原生默认 + 扩展管理 + 浏览器回归（2026-10-16）

核验日期：2026-10-16。分支 `client-native`（worktree `noname-next-wt-client`）。未改 `crates/engine-native` / `archive/upstream`。

## 交付

1. **原生为默认后端**：`VITE_ENGINE_BACKEND` 缺省 `native`；`legacy`/`wasm`/`old` 回退旧线。
2. **全流程 UI**：菜单 → 开局 → 自动步进/出牌通道 → 存档/读档（NGSA hex + 内容锁摘要闸）。
3. **15 包管理面**：安装 / 启用 / 停用 / 升级 / 回退 / 卸载 / 撤销；内容锁摘要；对局冻结。
4. **wasm 构建**：`build-native.mjs` 默认 `--target web`（浏览器 Worker）；Node 冒烟经 `init({module_or_path: bytes})`。
5. **真实浏览器回归**（cursor-ide-browser，`http://127.0.0.1:5173`）。

## 命令与结果

| 项 | 命令 | 结果 |
|---|---|---|
| 原生 wasm 构建 | `node apps/web-client/scripts/build-native.mjs` | web 目标部署 `src/worker/native-pkg/` |
| Node 冒烟 | `node apps/web-client/scripts/native-worker-smoke.mjs` | **9/9**——[日志](native-worker-smoke.log) |
| 扩展存储冒烟 | `node apps/web-client/scripts/extension-store-smoke.mjs` | **8/8**——[日志](extension-store-smoke.log) |
| vue-tsc | `pnpm exec vue-tsc --noEmit`（web-client） | 0 error（依赖已链接的 wasm-pkg/native-pkg） |
| 浏览器 | Vite `5173` + cursor-ide-browser | 大厅/对局/扩展截图见下 |

## 浏览器证据

| 文件 | 内容 |
|---|---|
| [00-home-native-default.png](00-home-native-default.png) | 大厅：原生默认文案 +「扩展」导航 |
| [01-battle-native.png](01-battle-native.png) | 对局：`rev N · native`、四座概况、存档/读档按钮 |
| [02-extensions-15.png](02-extensions-15.png) | 扩展：全部安装并启用 15 包 + 内容锁列表 |

浏览器操作摘要：本地开局 seed=42 → 自动步进数次 → 存档（约 1.6KB NGSA）→ 扩展页一键装启 15 包。

## 内核 API 缺口（只回报规格，本批未改引擎）

窄壳仍仅标准目录建局。客户端管理面已落，**入局装载**需 `engine-native-wasm` 补：

```text
NativeEngine::new_with_extensions(players: u8, seed: u64, packs: &[ExtensionPackInput]) -> Result<Self>
NativeEngine::restore_with_extensions(hex: &str, packs: &[ExtensionPackInput]) -> Result<()>

ExtensionPackInput {
  package_id: String,
  version: String,
  /// 优先 package.npack.json 原文；或已桥接卡片/技能载荷
  npack_json: String,
}
```

语义要求：

- 装载失败整体拒建局（与 game-server `/native` 一致）。
- `restore_with_extensions`：内容锁/版本与存档不一致 → 明确错误（对齐 G5 错版/缺包拒绝）。
- 无扩展时行为与现 `new`/`restore_hex` 字节兼容。

## Desktop / G5 客户端面

- Desktop（Tauri）共用 `apps/web-client`：同一原生默认与扩展管理面；本批未另跑 `tauri dev`（登记待补）。
- G5 客户端闸已实测：存档附带 `contentLock`/`lockSeed`；读档锁不一致拒绝；对局中 `matchFrozen` 挡卸载/启停。
- 真联机内容锁跨进程仍归服务端/E05 余项。

## 登记边界

- 窄壳无 `activateSkill`/`loadReplay`：显式未映射。
- 观察者投影：`known_cards` 已含本人手牌/装备；牌面 kind/suit 仍为占位（登记差异）。
- 扩展入局规则执行待上述 API；管理面不伪装已入局装载。
