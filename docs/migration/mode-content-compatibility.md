# 模式 × 内容包兼容矩阵（M4 验收门）

> 历史参考（2026-10-10 标注）：本页是旧线 `engine-core` 的模式矩阵（`crates/engine-core/src/mode` 只有 identity/guozhan/single）。原生线 `engine-native` 另有 Versus、斗地主、Boss、KOF 模式基线，状态见[规则基准](../architecture/native-rules-baseline.md)与[进度主稿](../architecture/native-progress.md)。最新入口见 [旧实现参考](../architecture/legacy-reference.md)。

> 更新：2026-08。对应原 v3.0 架构计划 §48.3 M4 总验收门第 3 项（该计划已于 2026-09-16 删除，见[旧实现参考](../architecture/legacy-reference.md)）。
> 矩阵记录每个模式可加载的内容包集合与依赖声明；未列出的组合即未验证，不允许在
> 该模式中自动加载未声明的内容包。

## 模式状态一览

| 模式 | ruleset 名 | 状态 | Headless 终局 | 可玩路径 |
|---|---|---|---|---|
| 身份 `identity` | `identity-standard` | 已实现（M1） | ✅ `engine-cli match` / `mode_single` 类 Golden | Web 单机 / 桌面客户端 / engine-cli |
| 国战 `guozhan` | `guozhan`（前缀匹配） | 已实现（ADR 0598） | ✅ 国战 Golden（`guozhan_trick_*`） | engine-cli + 国战包 |
| 单挑 `single` | `single` | 已实现（M4） | ✅ `engine-cli match 2 <seed> <steps> --ruleset single` / `mode_single` Golden | engine-cli（2026-08 接线） |
| 对决 / 斗地主 / 热斗 / BOSS / 象棋 / 塔防 / 石杀 / 联机大厅 | — | ADR 延期（见原计划 §48.1） | — | — |

## 兼容矩阵

| 内容包 | identity-standard | guozhan | single | 备注 |
|---|---|---|---|---|
| `standard`（官方标准包，自动加载） | ✅ | ✅（经国战引导） | ✅ | `bootstrap_match` 自动 `ensure_standard_package` |
| `sp` / `sp2` / `xianding` 等历史包 | ✅（测试套件验证） | 需显式加载 | ✅（测试套件验证） | 历史包经 `engine.load_package` 显式加载 |
| 模式专用包（国战包） | 不适用 | ✅ | 不适用 | 国战引导 `bootstrap_guozhan` 加载国战内容 |

## 规则：包依赖声明

- 所有模式共享受 `ensure_standard_package` 的自动标准包加载。
- 模式专用内容（如国战包）只在该模式引导路径内加载，身份/单挑模式不会隐式加载。
- 自定义包必须显式声明依赖（`manifest.dependencies`），否则不进入任何模式的合法内容集。

## 未闭合项（记录在案）

- Web/桌面客户端的模式选择入口仅固化 `identity-standard`；`guozhan`/`single` 的可玩路径
  目前为 engine-cli 与测试套件，UI 模式选择待 M7 多端阶段接线（2026-10-10 核对：仍未接线，`crates/engine-wasm/src/lib.rs` 固定 `identity-standard`）。
- 模式与「主将/野心家/限定技」等子规则组合的逐项矩阵待内容全量重写后扩充。
