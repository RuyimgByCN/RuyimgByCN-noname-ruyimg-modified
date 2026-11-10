# G0 · L13 手杀MVP

> 核验日期：2026-11-10 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/手杀MVP`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/手杀MVP/extension.js`（约 2773 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：2773
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：extension/手杀MVP/extension.js: packed=False, eval=3, long_lines=0
- 台账原生重构状态（摘录）：**MVP 计分规则、重铸钩子与终局结算大屏弹窗忠实实现（2026-11-10 裁决 34 第五轮挂载，[G0/L13](./L13-手杀MVP.md)）**：真实包 `packages/extensions/shousha-mvp-upstream-liuli` 忠实覆盖 7 项配置、按 `seq` 幂等计分公式（攻+疗+辅+基−罚）、`_qy_chongzhu` 重铸统计钩子与 `MvpSettlementModal.vue` 全场最佳结算献花大屏并完成 Headless Chrome 截图与测帧（忠实实现 **10** / 示意或自造 **0** / 缺失 **0**）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：extension/手杀MVP/extension.js: config_items≈8
- settings/config 引用次数（清点近似）：0
- 子插件/子目录：无
### 规则 / 依赖分支
- 外部扩展引用：手杀MVP
- 台账已核查摘要：事件评分与大段 game.over 替换、结算/战绩；逐项清单见 [G0/L13](./L13-手杀MVP.md)
- 台账待处理（含规则分支）：**仓内 10/10 项已忠实实现；待移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `ui`：界面/主题/部件（命令校验，不直改规则态）
- `config`：配置 schema + 内容锁敏感项
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L13-手杀MVP.md`
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

- **统计汇总**：上游实体合计 **10** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 7 / 界面钩子 1 / 渲染面 2）
- **状态分布**：`忠实实现` **10** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `recordMvpEventFact` / `incrementMvpRecastCount` (`_qy_chongzhu`) + `MvpSettlementModal.vue` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts:570-628,1304-1342`、`apps/web-client/src/components/battle/modals/MvpSettlementModal.vue:1-155` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:1-420` | 原生规则（`shousha_mvp.rs`）+ 原生前端组件 | 忠实实现（MVP 攻击/治疗/辅助/重铸计分与全场结算献花弹窗 `MvpSettlementModal.vue` 已挂载至 `BattleView.vue`） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（0 项）

- （上游无此类别实体）

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（7 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `qingyao_shoushaMVP` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2710` | 手杀MVP（默认=true）：开启后，游戏结算后可展示手杀MVP界面。 | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `yangshi` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2715` | MVP样式（默认='on'） | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `mvplutou` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2723` | 无原画时MVP露头（默认='shousha'）：开启后，游戏开始时玩家可以为AI或自己重新选将。(限身份场、斗地主、国战) | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `qingyao_shoushapeiyin` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2738` | 手杀配音（默认=true）：开启后，部分游戏音效将替换成手杀音效。 | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `shoushajiesuan` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2744` | 手杀结算（默认=true）：开启后，结算面板替换成手杀样式。 | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `jiesuanfix` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2749` | 结算优化（默认=true）：仅开启手杀结算生效，使用新版代码进行结算，能适应身份局的各种罕见胜负情况。 | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |
| `oneMore` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2754` | 再来一局（默认=true）：开启后，结算面板出现“再来一局”按钮。 | 原生规则（`crates/content-native-loader/src/shousha_mvp.rs`） | 忠实实现 |

### 7.7 界面钩子（UI Hooks）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `_qy_chongzhu` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:2244` | 触发="_chongzhuBegin",；脚本逻辑 | 原生前端组件 | 忠实实现（已在正式包规则或 `apps/web-client/src/engine/surfaces/*` / `extensionSurfaceRuntime.ts` 对应模块收口） |

### 7.8 渲染面（Rendering Surfaces）（2 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `extension.css` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.css:1-1310` | CSS 样式表 `extension.css`（1310 行） | 原生前端组件 | 忠实实现 |
| `extension_new.css` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension_new.css:1-227` | CSS 样式表 `extension_new.css`（227 行） | 原生前端组件 | 忠实实现 |

