# G0 · U04 coin

> 核验日期：2026-11-09 · 来源仓：`noname` · 入口：`archive/upstream/noname/apps/core/extension/coin`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/coin/extension.js`（约 1354 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：1354
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**金币结算与商店逻辑已实现（2026-10-17，[证据](../../operations/evidence/2026-10-17-t12-u04-eight/README.md)）**：真实包 `packages/extensions/coin-upstream-noname`。余额、商店（烟花 50、下雪 20、星云 10、闪烁 10）、下注 20 与终局公式从真实安装测过。**2026-11-09 裁决 34 清点勘正**：金币/战棋结算节点与荣誉切换钩子共 3 项忠实实现，但前端 `ExtensionSurfacePanel.vue` 的烟花/下雪/星云/闪烁 4 个渲染面为简化 CSS 示意层，未按上游 `coin/extension.js` canvas/DOM 粒子动画还原並缺与上游同场景截图对照（忠实实现 **3** / 示意或自造 **4** / 缺失 **0**）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：未检出独立 config 段（或壳扩展）
- settings/config 引用次数（清点近似）：0
- 子插件/子目录：无
### 规则 / 依赖分支
- 外部扩展引用：无（清点未检出）
- 台账已核查摘要：本地金币、商店、天气/粒子效果与 chess 分支；逐项清单见 [G0/U04](./U04-coin.md)
- 台账待处理（含规则分支）：**待按上游还原 4 项天气/烟花/星云/闪烁渲染面并做截图对照，待移动端真机**。`unit_random` 不是上游 Math.random
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `config`：配置 schema + 内容锁敏感项
- `ui`：界面/主题/部件（命令校验，不直改规则态）
- `resource`：资源索引（立绘/语音/动画，命名空间）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/U04-coin.md`
- 台账行：`docs/migration/upstream-extension-reconstruction.md` 对应 ID
- 功能面基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/feature-inventory.json`
- 原生包生命周期/内容锁：`crates/content-runtime` + 对应 packages/extensions（若已有转换包）
- 无头启动探针：`node scripts/legacy-harness/probe-liuli-boot.mjs <scratch/liuli根>`

## 5. 旧行为动态样本引用

- ../operations/evidence/2026-10-07-e01-feature-inventory/README.md（功能面清点基线）
- scripts/legacy-harness/（无头旧线采样入口；运行副本 scratch/legacy-harness）

## 6. 阻塞与后续

- G0 本文档完成不等于 G1–G5；整包状态仍以[台账](../upstream-extension-reconstruction.md)为准。
- 若需补跑动态样本：在 `scratch/legacy-harness` 运行副本上使用 `scripts/legacy-harness/`，勿写 `archive/upstream`。

## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-09）

- **统计汇总**：上游实体合计 **7** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 2 / 配置项 0 / 界面钩子 1 / 渲染面 4）
- **状态分布**：`忠实实现` **3** 项 · `示意或自造` **4** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `resolveCoinHonorTitle` / `[data-testid="coin-weather-overlay"]` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts:392-434`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:438-480` | `archive/upstream/noname/apps/core/extension/coin/extension.js:17-540` | 原生前端组件 | 示意或自造（CSS 示意粒子层，未复刻上游 `canvas` 烟花/下雪/星云/闪烁与窗口拖拽买道具交互） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（0 项）

- （上游无此类别实体）

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（2 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `coin_init` | `archive/upstream/noname/apps/core/extension/coin/extension.js:6` | 非战棋统率模式下初始化 `_status.coin = 0` | 原生规则（`coin.rs`） | 忠实实现 |
| `game.changeCoin` | `archive/upstream/noname/apps/core/extension/coin/extension.js:109` | 增减金币、持久化 `lib.config.coin`、触发提示与音效 | 原生规则（`coin.rs`） | 忠实实现 |

### 7.6 配置项（Config Items）（0 项）

- （上游无此类别实体）

### 7.7 界面钩子（UI Hooks）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `coin_arenaReady` | `archive/upstream/noname/apps/core/extension/coin/extension.js:11` | 创建顶栏富甲天下金币按钮、商店弹窗与本局获胜 20 金下注入口 | 原生规则（`coin.rs`）+ 原生前端组件 | 忠实实现（规则结算已在 `coin.rs` 落地） |

### 7.8 渲染面（Rendering Surfaces）（4 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `haveFun.firework` | `archive/upstream/noname/apps/core/extension/coin/extension.js:129` | 富甲天下商店特效 `firework`（烟花 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 示意或自造（商店购买状态机已忠实实现，前端 Canvas 粒子为 CSS 示意层） |
| `haveFun.snow` | `archive/upstream/noname/apps/core/extension/coin/extension.js:133` | 富甲天下商店特效 `snow`（下雪 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 示意或自造（商店购买状态机已忠实实现，前端 Canvas 粒子为 CSS 示意层） |
| `haveFun.star` | `archive/upstream/noname/apps/core/extension/coin/extension.js:148` | 富甲天下商店特效 `star`（星云 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 示意或自造（商店购买状态机已忠实实现，前端 Canvas 粒子为 CSS 示意层） |
| `haveFun.blink` | `archive/upstream/noname/apps/core/extension/coin/extension.js:152` | 富甲天下商店特效 `blink`（闪烁 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 示意或自造（商店购买状态机已忠实实现，前端 Canvas 粒子为 CSS 示意层） |

