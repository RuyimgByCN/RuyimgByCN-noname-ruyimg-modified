# G0 · L14 手杀ui

> 核验日期：2026-11-09 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/手杀ui`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/手杀ui/extension.js`（约 7480 行；game.import=True；skills=2; cards=3; characters=2）

### 清点摘要
- 主入口行数：7480
- game.import：True
- 子插件数：1
- 资源：audio:4文件/80190B
- 混淆/动态特征：extension/手杀ui/extension.js: packed=False, eval=4, long_lines=0
- 台账原生重构状态（摘录）：**配置装载已实现，UI 钩子与界面未完成（2026-11-09 裁决 34 清点勘正，撤回 Batch 26 全闭环宣称）**：`shousha-ui-upstream-liuli` 覆盖 49 项配置。但 `decision34_pack9::resolve_l14_shousha_skill` 把 48 个上游 `lib.skill._*` 进度条/手气卡/托管/语音气泡 UI 钩子错误实现为「摸 1 张牌」或「回 1 血」；40 个 `lbtn`/`skill`/`character` 界面渲染面在 `ExtensionSurfacePanel.vue` 中仅为 1 个左右手切换按钮（清点统计：忠实实现 **49** / 示意或自造 **88** / 缺失 **6**）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：extension/手杀ui/extension.js: config_items≈38
- settings/config 引用次数（清点近似）：13
- 子插件/子目录：fonts/extension.js
### 规则 / 依赖分支
- 外部扩展引用：手杀ui
- 台账已核查摘要：主题/菜单/聊天、动态加载、选择及规则/调试钩子；逐项清单见 [G0/L14](./L14-手杀ui.md)
- 台账待处理（含规则分支）：**待删除 `decision34_pack9` 将手杀 UI 钩子当作摸牌/回血的错误实现，按上游 `手杀ui/{lbtn,skill,character}/` 还原手杀界面与 UI 钩子并做截图对照，待移动端真机**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `ui`：界面/主题/部件（命令校验，不直改规则态）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L14-手杀ui.md`
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

- **统计汇总**：上游实体合计 **141** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 48 / 界面钩子 53 / 渲染面 40）
- **状态分布**：`忠实实现` **48** 项 · `示意或自造` **88** 项 · `缺失` **5** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `L14_SHOUSHA_SKILLS` + `resolve_l14_shousha_skill`（伪造武将/卡牌与 `spshidi`/`spcishi`/`zhefu`/`yidu` 已清除，48 个 UI 钩子已清零摸牌/回血副作用） | `crates/content-native-loader/src/decision34_pack9_data.rs:132-143`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:608-617` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-7360` | 原生前端组件 | 示意或自造（已删除伪造 `shousha_*` 武将/卡牌、`spshidi`/`spcishi`/`zhefu`/`yidu` 及摸牌/回血副作用；手杀进度条、菜单、托管、语音气泡等前端视觉表现待完整复刻） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（0 项）

- （上游无此类别实体）

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（48 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `ShouShaThanks` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5489` | 作者：橙续缘（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `MoGaiMengXiu` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5493` | 修改：萌新（转型中）（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `KZJS` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5497` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GXSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5514` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `MX` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5529` | 鸣谢表（默认="1"） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `FL1` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5645` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JDTSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5652` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiao` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5669` | 进度条（默认=true）：自己回合内显示进度条带素材 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaoYangshi` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5674` | 进度条样式（默认="1"）：切换进度条样式，可根据个人喜好切换手杀进度条或十周年进度条，切换后重启生效 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaotuoguan` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5684` | 托管效果（默认=true）：开启进度条的情况下，开启此选项后，当玩家的进度条时间走完时，将自动托管。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaoST` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5689` | 进度条时间间隔（默认="100"）：设置玩家进度条的时间间隔。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaoUpdata` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5704` | 玩家进度条刷新（默认=true）：开启进度条的情况下，开启此选项后，玩家进度条将会进行刷新 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaoaiUpdata` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5709` | 人机进度条刷新（默认=true）：开启进度条的情况下，开启此选项后，ai的进度条将会进行刷新 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jindutiaoSet` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5714` | 进度条高度（默认="26"）：设置玩家进度条的高度百分比。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `jiangdeng` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5746` | 手杀将灯展示（默认=true）：开启后对局中部分武将右下角会出现将灯的标记（搬运自 烧烤团子 的扩展，感谢！由于适配问题，已重构部… | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `FL2` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5752` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JDTSSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5759` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JDTS` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5778` | 阶段提示（默认=true）：自己回合内显示对应阶段图片提示 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JDTSYangshi` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5783` | 阶段提示样式（默认="1"）：切换阶段提示样式，可根据个人喜好切换 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `FL3` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5793` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GTBBSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5800` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GTBB` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5819` | 狗托播报（默认=true）：开启后，顶部会出现滚动播报栏。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GTBBYangshi` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5824` | 播报样式(需重启)（默认="on"）：切换狗托播报样式 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GTBBFont` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5833` | 播报字体（默认="on"）：切换狗托播报字体，可根据个人喜好切换（即时生效） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `GTBBTime` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5842` | 时间间隔(重启生效)（默认="random"）：更改狗托播报出现的时间间隔，可根据个人喜好调整频率 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `FL4` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5857` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JMYSSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5864` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `yangshi` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5880` | <font color=\（默认='on'）：切换手杀UI界面样式，初始为手杀样式，可根据个人喜好切换手杀样式或十周年样式，切换后重启生效 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `yangshi_bg` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5889` | 菜单壁纸（手杀）（默认="0"）：手杀样式下，选择切换菜单背景壁纸 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `XPJ` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5910` | 小配件（十周年）（默认="off"）：十周年样式下，选择切换左下角小配件 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `LTAN` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5919` | 聊天按钮隐藏（默认=false）：手杀样式下在游戏中，隐藏左下角的聊天按钮需重启 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xiandingskill_ss` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5924` | 限定技美化（手杀）（默认=true）：手杀样式下在游戏中，限定技按钮样式美化显示 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `FL5` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5931` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `QTMHSM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5938` | <div class=（默认=—） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `KGMH` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5954` | 开关美化（默认="1"）：开启后可以美化游戏的选项开关，需要重启 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `qiDongYe` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5964` | 启动页（默认='othersOn'）：可直接通过此功能切换启动页样式 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `JSAN` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5976` | 结算按钮隐藏（默认=false）：在游戏结束时，出现按钮隐藏结算菜单或直接隐藏结算菜单可以点击游戏界面中心空白区域显示菜单 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `分割线01` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7377` | <img style=width:240px src=（默认=true）：搬运自 群小乔 说话扩展中，说话和发送表情的功能，内容有增删 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `sayplay` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7383` | 表情＆说话（默认='chg'）：搬运自 群小乔 说话扩展中，说话和发送表情的功能，内容有增删 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `charactermenu` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7393` | 武将详情（默认='new'）：武将详情页面美化，重启生效 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xindmenu` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7403` | 菜单美化及音效（默认='shousha'）：菜单美化和音效，关闭可提升流畅度 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xindchoose` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7413` | 选将美化（默认='shousha'）：选将框美化，提供势力框和背景框，以及单行选将。若不需要单行选将，请自行去style里注释 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xindchoose_type` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7423` | 美化版本（默认='new'）：选将美化中手杀美化的旧式代码开关，因新代码使用了取巧的方式优化显示但尚不明确负面影响，因此若有显示… | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xindsingle` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7433` | 单行选将（默认=true）：选将时，变为单行 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `xinddraw` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7438` | 手气卡美化（默认=true）：为手气卡刷新添加音效并修改无限次数为7次，添加进度条手牌分。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `infinitydraw` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7443` | 无限手气卡（默认=false）：在前一选项开启的前提下，解放无限手气卡的次数。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `分割线02` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:7448` | <img style=width:240px src=（默认=true） | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |
| `ZLLT` | `archive/upstream/noname-liuli/noname/extension/手杀ui/fonts/extension.js:6939` | 资料页露头(重启生效)（默认=false）：手杀样式下，点击武将资料页会适配露头皮肤。 | 原生规则（`crates/content-native-loader/src/shousha_ui.rs`） | 忠实实现 |

### 7.7 界面钩子（UI Hooks）（53 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `_dieNoTuoguan` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85` | 触发="dieAfter",；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_menuCheat` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:103` | 指令；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiaoO` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1120` | 触发=['phaseZhunbeiBegin', 'phaseBegin'…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiaoA` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1170` | 触发=['useCardBegin', 'respondBegin', '…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiaoMB` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1216` | 触发='useCardToPlayered',；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiaoKS` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1301` | 触发='gameStart',；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiao_close` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1385` | 触发=['phaseEnd', 'useCardAfter', 'gain…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiA` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1408` | 触发=['phaseUseBegin', 'useCardEnd', 'l…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiB` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1445` | 触发=['phaseUseEnd', 'dieBegin', 'phase…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiC` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1473` | 触发='phaseDiscardBegin'；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiD` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1508` | 触发=['phaseDiscardEnd', 'dieBegin'],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiE` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1533` | 触发=['useCardBegin', 'respondBegin']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiF` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1562` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiG` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1587` | 触发=['useCardBegin', 'respondBegin']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiH` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1617` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiM` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1673` | 触发=['useCardBegin', 'respondBegin']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiN` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1703` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiO` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1728` | 触发=['useCardBegin', 'respondBegin']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiP` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1757` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiI` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1782` | 触发=['useCardBegin', 'respondBegin', '…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiJ` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1822` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiK` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1839` | 触发=['phaseJudgeBegin', 'phaseDrawBegi…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiL` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1874` | 触发=['phaseJudgeEnd', 'phaseDrawEnd', …；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiMX` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1892` | 触发=['useCardBegin', 'respondBegin']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_chupaiNX` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1922` | 触发=['useCardEnd', 'respondEnd', 'dieB…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_skilltip_closeB` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:1940` | 触发=['phaseUseEnd', 'dieBegin', 'dying…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyA` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2110` | 触发=['chooseToRespondBegin'],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyB` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2131` | 触发='shaBegin',；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyC` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2152` | 触发=['useCardToBegin', 'phaseJudge']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyD` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2214` | 触发=['chooseToUseBegin','chooseToUseEn…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyF` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2236` | 触发=['useCardBefore','useCardAfter'],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyAll` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2262` | 触发=['chooseToUseBegin','chooseToUseEn…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_shiyongshanD` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2298` | 触发=["useCard", "respondAfter"],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_ddxyE` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2323` | 触发=['chooseToRespondEnd', 'useCardToE…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_wjsw` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2351` | 触发=['phaseEnd', 'useCardAfter']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_swxs` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2365` | 触发=['dieAfter']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_dfsk` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2383` | 触发=['phaseBegin', 'phaseEnd', 'phaseJ…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_hhks` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2409` | 脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_pdjd` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2440` | 触发=['phaseJudgeBefore', 'phaseJudgeAf…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_mpjd` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2469` | 触发=['phaseDrawBefore', 'phaseDrawAfte…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_cpjd` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2498` | 触发=['phaseUseBefore', 'phaseUseAfter'…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_qpjd` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2527` | 触发=['phaseDiscardBefore', 'phaseDisca…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_hhjs` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2556` | 脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jd_phaseAfter` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2587` | 触发=['phaseAfter']；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiao` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2728` | 触发=['phaseZhunbeiBegin', 'phaseBegin'…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_jindutiaopl` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:2777` | 触发=['gameStart'], player: ['useCardTo…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_wmkzSayChange` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:4452` | 触发=["gameStart", "phaseBegin", "phase…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `_shuohuaSayWordphase` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:6394` | 触发="gameStart", player:["phaseBegin",…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻） |
| `ZLLT` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5981` | 资料页露头(重启生效) | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_jindutiaouse` | `archive/upstream/noname-liuli/noname/extension/手杀ui/fonts/extension.js:2553` | 触发="useCardAfter",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_wuxie` | `archive/upstream/noname-liuli/noname/extension/手杀ui/fonts/extension.js:2565` | 触发=['useCardToBegin', 'phaseJudge']；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_uicardupdate` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1.js:495` | 触发='phaseJieshuBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_paixu_paixu_paixu` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1.js:514` | 触发=["gainEnd"]；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |

### 7.8 渲染面（Rendering Surfaces）（40 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `main1` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main1.js:1-444` | 前端脚本模块 `main1.js`（444 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main1_new` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main1_new.js:1-1048` | 前端脚本模块 `main1_new.js`（1048 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main2` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main2.js:1-290` | 前端脚本模块 `main2.js`（290 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main1` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1.js:1-1119` | 前端脚本模块 `main1.js`（1119 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main2` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main2.js:1-598` | 前端脚本模块 `main2.js`（598 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `menu` | `archive/upstream/noname-liuli/noname/extension/手杀ui/menu.js:1-7692` | 前端脚本模块 `menu.js`（7692 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main1` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main1.js:1-1104` | 前端脚本模块 `main1.js`（1104 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `main2` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main2.js:1-482` | 前端脚本模块 `main2.js`（482 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `kaiguan.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/KGMH/kaiguan.css:1-20` | CSS 样式表 `kaiguan.css`（20 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `kaiguan_new.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/KGMH/kaiguan_new.css:1-24` | CSS 样式表 `kaiguan_new.css`（24 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/card/main.css:1-35` | CSS 样式表 `main.css`（35 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/card/main1.css:1-14` | CSS 样式表 `main1.css`（14 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_mobile.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/card/main1_mobile.css:1-35` | CSS 样式表 `main1_mobile.css`（35 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_pc.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/card/main1_pc.css:1-35` | CSS 样式表 `main1_pc.css`（35 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/card/main2.css:1-35` | CSS 样式表 `main2.css`（35 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main1.css:1-537` | CSS 样式表 `main1.css`（537 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_new.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main1_new.css:1-525` | CSS 样式表 `main1_new.css`（525 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_old.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main1_old.css:1-537` | CSS 样式表 `main1_old.css`（537 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/character/main2.css:1-257` | CSS 样式表 `main2.css`（257 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/dialog/main1.css:1-105` | CSS 样式表 `main1.css`（105 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `extension.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.css:1-230` | CSS 样式表 `extension.css`（230 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1.css:1-890` | CSS 样式表 `main1.css`（890 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_boss.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1_boss.css:1-896` | CSS 样式表 `main1_boss.css`（896 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_window.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1_window.css:1-861` | CSS 样式表 `main1_window.css`（861 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main2.css:1-748` | CSS 样式表 `main2.css`（748 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2_window.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main2_window.css:1-748` | CSS 样式表 `main2_window.css`（748 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `menu.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/menu.css:1-1803` | CSS 样式表 `menu.css`（1803 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `menu2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/menu2.css:1-1677` | CSS 样式表 `menu2.css`（1677 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `menu3.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/menu3.css:1-1734` | CSS 样式表 `menu3.css`（1734 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `layout_new.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/qidongye/layout_new.css:1-97` | CSS 样式表 `layout_new.css`（97 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `layout_old.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/qidongye/layout_old.css:1-97` | CSS 样式表 `layout_old.css`（97 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `layout_xbig.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/qidongye/layout_xbig.css:1-99` | CSS 样式表 `layout_xbig.css`（99 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main1.css:1-952` | CSS 样式表 `main1.css`（952 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main1_window.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main1_window.css:1-652` | CSS 样式表 `main1_window.css`（652 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main2.css:1-243` | CSS 样式表 `main2.css`（243 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `main2_window.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main2_window.css:1-262` | CSS 样式表 `main2_window.css`（262 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `style.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/style.css:1-405` | CSS 样式表 `style.css`（405 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `style2.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/style2.css:1-444` | CSS 样式表 `style2.css`（444 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `style_boss.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/style_boss.css:1-432` | CSS 样式表 `style_boss.css`（432 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `style_old.css` | `archive/upstream/noname-liuli/noname/extension/手杀ui/style_old.css:1-381` | CSS 样式表 `style_old.css`（381 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |

