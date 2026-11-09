# G0 · L05 coin

> 核验日期：2026-11-10 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/coin`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/coin/extension.js`（约 1375 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：1375
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**金币结算、商店拖拽购买与天气粒子渲染面忠实实现（2026-11-09 裁决 34 第三步，[G0/L05](./L05-coin.md)）**：真实包 `packages/extensions/coin-upstream-liuli`。商店、`_buy` 拖拽购买钩子（`dragBuyCoinShopItem`）与 4 项 2D `<canvas>` 天气/粒子渲染面全部忠实实现并完成 Headless Chrome 截图与测帧（忠实实现 **4** / 示意或自造 **0** / 缺失 **0**）

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
- 台账已核查摘要：金币商店/效果、模式门控；逐项清单见 [G0/L05](./L05-coin.md)
- 台账待处理（含规则分支）：**仓内 4/4 项已忠实实现；待移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `config`：配置 schema + 内容锁敏感项
- `ui`：界面/主题/部件（命令校验，不直改规则态）
- `resource`：资源索引（立绘/语音/动画，命名空间）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L05-coin.md`
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

## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-10）

- **统计汇总**：上游实体合计 **4** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 0 / 界面钩子 0 / 渲染面 4）
- **状态分布**：`忠实实现` **4** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

- 本来源未被 `decision34_closure.rs` / `decision34_pack9.rs` 注入自造武将或默认摸 1 技能。

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（0 项）

- （上游无此类别实体）

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（0 项）

- （上游无此类别实体）

### 7.7 界面钩子（UI Hooks）（0 项）

- （上游无此类别实体）

### 7.8 渲染面（Rendering Surfaces）（4 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `haveFun.firework` | `archive/upstream/noname-liuli/noname/extension/coin/extension.js:126` | 富甲天下商店特效 `firework`（烟花 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts::renderCoinCanvasParticles` 落地） |
| `haveFun.snow` | `archive/upstream/noname-liuli/noname/extension/coin/extension.js:130` | 富甲天下商店特效 `snow`（下雪 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts::renderCoinCanvasParticles` 落地） |
| `haveFun.star` | `archive/upstream/noname-liuli/noname/extension/coin/extension.js:146` | 富甲天下商店特效 `star`（星云 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts::renderCoinCanvasParticles` 落地） |
| `haveFun.blink` | `archive/upstream/noname-liuli/noname/extension/coin/extension.js:150` | 富甲天下商店特效 `blink`（闪烁 Canvas 粒子与启停控制） | 原生规则（`coin.rs`）+ 原生前端组件 | 忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts::renderCoinCanvasParticles` 落地） |

