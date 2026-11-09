# G0 · L08 十周年UI

> 核验日期：2026-11-09 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/十周年UI`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/十周年UI/extension.js`（约 12724 行；game.import=True；skills=0; cards=1; characters=2）

### 清点摘要
- 主入口行数：12724
- game.import：True
- 子插件数：0
- 资源：image:4文件/3890829B
- 混淆/动态特征：extension/十周年UI/extension.js: packed=False, eval=1, long_lines=0
- 台账原生重构状态（摘录）：**配置装载器已实现，规则钩子与 UI 表面未完成（2026-11-09 裁决 34 清点勘正，撤回 Batch 26 全闭环宣称）**：真实包 `packages/extensions/decade-ui-upstream-liuli` + `content_native_loader::decade_ui` 覆盖 45 项配置。但：① `decision34_pack9_data::L08_DECADE_CHARACTERS` 的 **9 名 `decade_*` 武将在上游不存在（属凭空自造，待删）**；② `L08_DECADE_SKILLS` 的 9 项 `_usecard`/`_chongzhu` 等上游 UI 动画钩子在 `decision34_pack9.rs` 中被错误实现为摸牌/加护甲；③ 上游 `skill.js`/`content.js`/`effect.js`/`layout.css` 共 **319 个 UI 钩子**与 **43 个渲染面**（含 Spine 骨骼动画）中，`drawSpineSkeletalCanvas` 只是 2D canvas 圆加两条线（清点统计：忠实实现 **45** / 示意或自造 **52** / 缺失 **310**，另自造 9 名 `decade_*` 武将待删）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：extension/十周年UI/extension.js: config_items≈44
- settings/config 引用次数（清点近似）：0
- 子插件/子目录：无
### 规则 / 依赖分支
- 外部扩展引用：十周年UI
- 台账已核查摘要：牌桌/选择部件、动画、Worker、卡牌/玩家函数覆盖和技能钩子；逐项清单见 [G0/L08](./L08-十周年UI.md)
- 台账待处理（含规则分支）：**待删除 9 名自造 `decade_*` 武将与摸牌钩子、删除 `drawSpineSkeletalCanvas` 示意画法、实现净室 Spine 4.0 播放器并按上游还原十周年 UI 布局/特效及截图对照，待移动端真机**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `ui`：界面/主题/部件（命令校验，不直改规则态）
- `resource`：资源索引（立绘/语音/动画，命名空间）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L08-十周年UI.md`
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

- **统计汇总**：上游实体合计 **406** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 44 / 界面钩子 319 / 渲染面 43）
- **状态分布**：`忠实实现` **44** 项 · `示意或自造` **52** 项 · `缺失` **310** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `L08_DECADE_CHARACTERS`（9 名伪造 `decade_*` 武将已按第二步清除） | `crates/content-native-loader/src/decision34_pack9_data.rs:127-130` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12687-12692`（上游 `character: {}` 为空） | 无（已清除） | 已清除（9 名伪造 `decade_*` 武将与 `_decade_qiankunbagua` 已从代码删除，见 `docs/operations/evidence/2026-11-09-decision34-cleanup/README.md`） |
| `L08_DECADE_SKILLS` + `resolve_l08_decade_skill`（9 项 `_changeEquipExt`/`_usecard` 等 UI 钩子） | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:590-603` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:54-650` | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS/Spine 特效待在前端组件实现） |
| 净室 Spine 4.0 播放器 (`apps/web-client/src/engine/spine40/*`，已替换示意 `drawSpineSkeletalCanvas`) | `apps/web-client/src/engine/spine40/webglRenderer.ts:1-474`、`apps/web-client/src/engine/spine40/index.ts:1-5`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:353-388` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816`、`dynamicSkin.js:1-162` | 原生前端组件（净室 Spine 4.0 播放器） | 忠实实现（按裁决 35 净室自写 Spine 4.0 JSON/二进制解码、骨骼世界变换、IK/变换/路径约束、`transform`/`path`/`deform` 时间轴、多轨道混合、图集解析与 WebGL PMA 渲染，无动皮资源或非 4.0 资源时回退静态立绘） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（0 项）

- （上游无此类别实体）

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（44 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `node` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:6948` | node（默认=—） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `_connectMode` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:11995` | 流畅模式（默认=true）：开启这个模式，然后将游戏速度调快…人机出牌速度就会真正飞起来。那么，代价是什么呢？开启可能会造成卡… | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `asyncMode` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12007` | 加速·异步模式（默认=false）：开启这个模式，然后将游戏速度调快…人机出牌速度就会真正飞起来。那么，代价是什么呢？开启可能会造成卡… | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `eruda` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12012` | 调试助手（默认=false） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `rightLayout` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12017` | 右手布局（默认=true） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `game_cardSet` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12025` | <span style=（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardHelasisy` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12069` | <span style=（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `game_cardSet2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12098` | <span style=（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardHelasisy2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12142` | <span style=（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardPrettify` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12171` | 卡牌美化（默认='png'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardBorder` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12213` | 卡牌边框（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `gold_suitnum` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12272` | 视为点数花色高亮（默认='gold'）：高亮“视为”牌点数和花色变化牌的内容，重启后生效 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `dynamicBackground` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12282` | 动态背景(导入骨骼包才可使用)（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `dynamicSkin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12308` | 动态皮肤（默认=false） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `dynamicSkinOutcrop` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12312` | 动皮露头（默认=true） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardAlternateNameVisible` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12330` | 牌名辅助显示（默认=false）：隐藏开启托管后在界面显示的“托管中...”提示 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `campIdentityImageMode` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12337` | 势力身份美化（默认=true）：隐藏开启托管后在界面显示的“托管中...”提示 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `tuoguan_hide` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12341` | 隐藏托管提示（默认=false）：隐藏开启托管后在界面显示的“托管中...”提示 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `SanGuoShaGameStart` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12379` | <font color=\（默认='shousha_dys'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `SanGuoShaGameOver` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12409` | <font color=\（默认='shousha'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `CPZS` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12422` | <font color=\（默认='shoushaX'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `JNTX` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12441` | <font color=\（默认=true）：性能不好可关闭 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `JBDDZ` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12446` | 军八斗地主美化(仅手杀样式有效)（默认=true）：双将模式选将有问题可关闭这个 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `playerKillEffect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12455` | 玩家击杀特效（默认=false） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `gameAnimationEffect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12463` | 游戏动画特效（默认=true） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `EffectWood` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12467` | 治疗特效（默认=false） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `playerDieEffect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12475` | 玩家阵亡特效（默认=true） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `cardUseEffect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12483` | 卡牌使用特效（默认=false） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `outcropSkin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12499` | 露头皮肤(需对应素材)（默认='shousha'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `borderLevel` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12512` | 玩家边框等阶（默认='kill'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `skill_control_gold` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12527` | 按钮文字镶金（默认='none'）：开启后，会将选择按钮的文字样式变成金色 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `gainSkillsShown` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12537` | 获得技能显示(标记)（默认=true） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `gainSkillsVisible` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12541` | 获得技能显示(贴纸)（默认='off'） | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `playerMarkStyle` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12554` | 人物标记样式（默认='decade'）：人物标记样式中选择“十周年”，开启此选项后将优化手杀样式的标记排版，同一行内可容纳更多的标记，使得… | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `shoushaNewNewMark` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12566` | 手杀标记优化（默认=true）：人物标记样式中选择“十周年”，开启此选项后将优化手杀样式的标记排版，同一行内可容纳更多的标记，使得… | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `newDecadeStyle` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12571` | 样式切换(需重启)（默认='off'）：重启生效 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `changeWujiangTX` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12584` | 换将特效（默认='shine'）：选将时点击右上角换将卡进行换将的特效 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `shadowStyle` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12597` | <font color=\（默认='off'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `rarityLong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12609` | 龙头美化（默认='on'）：仅适用于手杀样式 | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `decadeLayout` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12623` | <font color=\（默认='off'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `aloneEquip` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12636` | <font color=\（默认=false）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `shoushaNewEquip` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12647` | <font color=\（默认=true）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `shoushaNewMark` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12658` | <font color=\（默认=true）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |
| `loadingStyle` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12669` | <font color=\（默认='othersOn'）：<font color=\ | 原生规则（`crates/content-native-loader/src/decade_ui.rs`） | 忠实实现 |

### 7.7 界面钩子（UI Hooks）（319 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `_jiuwo_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1402` | SZN_jiuwo | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_delete_jiuwo_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1416` | 触发=["dyingAfter", "dieBegin", "dieAft…；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_dyjinengtx` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1430` | 触发=['logSkillBegin', 'useSkillBegin']；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_losetl_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1604` | SZN_loseHp | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_jinnangpai_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1614` | SZN_taoyuanjieyi | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_jiubuff_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1632` | jiubuff | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_jiubuff_xiaoshi_` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1650` | 触发=["useCard", "phaseAfter", "dieAfte…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_player_hp` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1674` | shuzi | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_HpRecover` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation_new.js:1402` | skeleton | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_Hplose` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation_new.js:1448` | 触发="damageBegin4",//时机在免伤事件之前，所以无法考虑免…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_killFire` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation_new.js:1620` | 触发="dieBegin", player: "die", global:…；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_changeEquipExt` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:54` | 触发="loseBegin", global:["equipAfter",…；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_ui_auto_cheat` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:180` | 作弊；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_kill_framebg` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:210` | 触发=["gameStart","showCharacterEnd"],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_rarity` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:321` | 触发=["gameStart", "showCharacterEnd"],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_tiesuo_ying` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:479` | 触发=["gameDrawAfter", "dieBegin"],；脚本逻辑 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_usecard` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:7540` | 触发='useCardAfter'；十周年UI渲染 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_decadeUI_usecardBegin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:7556` | 触发='useCardBegin'；十周年UI渲染 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_discard` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:7569` | 触发=['discardAfter', 'loseToDiscardpil…；十周年UI渲染 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_decadeUI_dieKillEffect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:7604` | 触发=['dieBegin']；十周年UI渲染 | 原生前端组件 | 示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻） |
| `_removeXieli` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:186` | 触发=["dieAfter"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `nzry_chenglve` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:539` | 摸牌、弃牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `nzry_chenglve1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:578` | 移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `nzry_chenglve2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:598` | 成略（暴戾） | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xionghuo` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:610` | 暴戾 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `taoluan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:618` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `taoluan3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:730` | 触发='phaseZhunbeiBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twtuidao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:738` | 触发='phaseZhunbeiBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mbaosi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:857` | 触发='damageSource'；临时技、获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yizan_use` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:905` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcxiangmian` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1069` | 判定、获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `smh_yeyan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1174` | 限定技，出牌阶段，你可以对一至三名角色造成至多共3点火焰伤害（你可以任意分配每名目标角色受到的伤害点数），若你将对一名角色分配2点或更多的火焰伤害，你须先弃置四张不同花色的手牌再… | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `smh_huoji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1274` | 龙印 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `olfengji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1315` | 触发='phaseDrawBegin2'；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twyilie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1424` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twxingzhui` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1489` | 失去体力、选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcxuewei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1600` | tao | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `lkbushi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1676` | 触发='phaseZhunbeiBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `wangong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1858` | 触发='useCard'；获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `wangong2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1879` | 触发='useCard1'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `clanlianhe` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1901` | 触发='phaseUseBegin'；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rekuangcai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1991` | 触发='phaseDiscardBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcsilve` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2059` | 触发='enterGame', global: 'phaseBefore',；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcshuaijie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2182` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twsuizheng` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2277` | 触发='phaseBefore', player: 'enterGame'；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twmouzhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2371` | 触发='damageBegin4'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejingce` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2422` | 触发='phaseUseEnd'；摸牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejingce_add` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2441` | 触发='loseEnd'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejingce2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2498` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twyangming` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2514` | 触发='phaseUseEnd',；摸牌、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twyangming1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2548` | 触发='useCard2',；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rexiemu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2585` | 触发='phaseJieshuBegin'；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rexiemu3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2628` | 触发='phaseBegin'；移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dawu2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2645` | 触发='damageBegin4'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `kuangfeng2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2672` | 触发='damageBegin3'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yijin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2696` | 触发='phaseUseBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `refenxun` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:2980` | 加标记、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `refenxun2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3036` | 触发='phaseJieshuBegin',；弃牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhihu_mark` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3063` | 触发='phaseBeginStart'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `naxiang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3081` | 触发='damageEnd', source: 'damageSource…；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_weilu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3133` | losehp | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcfudao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3272` | 触发='phaseBefore', player: 'enterGame',；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twgongxin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3396` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twgongxin2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3481` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twshelie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3494` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shanduan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3558` | 触发='phaseBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `clansankuang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3853` | 触发='useCardAfter'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcbingji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:3964` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twjichou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4093` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jianying` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4328` | 触发='useCard'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_shangjian` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4386` | 触发="phaseJieshuBegin", player: "loseA…；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reluanji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4432` | wanjian | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gzjili` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4577` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `longdan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4679` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `nzry_shicai_2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4793` | 触发=['useCardAfter'], target: 'useCard…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `nzry_shicai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4794` | 触发=['useCardAfter'], target: 'useCard…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_falu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4890` | 紫薇 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_zhenyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4961` | 触发="damageEnd",；移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhenyi_spade` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:4995` | 触发="judge",；判定、选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhenyi_club` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5057` | tao | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhenyi_heart` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5144` | 触发="damageBegin1",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gongsun` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5171` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gongsun_shadow` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5245` | 移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gongsun_shadow2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5283` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mozhi1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5329` | 默识1-；触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mozhi2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5332` | 默识2-；触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mozhit` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5335` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mozhip` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5362` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mozhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5374` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xianfu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5438` | 触发="phaseBefore", player: "enterGame",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xianfu2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5474` | 触发=["damageEnd", "recoverEnd"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `chouce` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5511` | 触发="damageEnd",；判定、选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shuangxiong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5595` | 触发="phaseDrawBegin1",；判定 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shuangxiong2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5629` | juedou | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reshuangxiong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5699` | 触发="phaseDrawBegin1",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qianxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5769` | 触发="phaseZhunbeiBegin",；摸牌、弃牌、选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qianxi2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5828` | 移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reqianxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5845` | 触发="phaseZhunbeiBegin",；摸牌、弃牌、选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinzhanyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:5933` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinzhanyi_basic` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6004` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinzhanyi_equip` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6132` | 触发="useCardToPlayered",；弃牌、移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinzhanyi_trick` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6166` | 移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `hongyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6176` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `hongyi2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6234` | 触发="damageBegin1",；摸牌、判定 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `requanfeng_gain` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6266` | 触发='die'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mjmouzhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6325` | 触发="damageBegin2",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhuihuan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6385` | 触发="phaseJieshuBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bixiong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6432` | 触发="loseAfter",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bixiong2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6473` | 触发=["useCardAfter", "respondAfter"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhtongyuan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6485` | 触发=["useCardAfter", "respondAfter"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `daiyan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6589` | 触发="phaseJieshuBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `tongli` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6669` | 触发="useCardToPlayered",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jinjian` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6822` | 触发="damageBegin1",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcmingfa` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:6949` | 触发="useCardAfter",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `smh_lianhuan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7136` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `guili` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7209` | 触发="phaseBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `tongxie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7273` | 触发="phaseUseBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `discretesidi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7451` | 触发="useCardAfter",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_jixu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7599` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `resghuishi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7735` | 选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reqianxin2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7819` | 遣信 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reqianxin3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7876` | 遣信 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yuanyu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:7890` | 摸牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xiyan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8052` | 触发="addToExpansionAfter",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twjingce` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8138` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_guagua1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8181` | 触发="useCard2",；加标记、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twgezhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8195` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twfengji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8347` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twharvestinori` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8471` | 选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twzhouhu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8561` | 选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `twzuhuo` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8653` | 选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezaiqi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8767` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezaiqizheng` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8835` | 再起 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xuanmu2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8856` | 宣穆 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinbenxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:8879` | 触发="useCard2",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xibing2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9144` | 息兵 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `caozhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9156` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `olxibing` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9280` | 触发="damageEnd", source: "damageSource…；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spdiancai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9356` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zlshoufu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9417` | 将一张手牌作为“箓”置于其他角色的武将牌上 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinquanbian` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9478` | 触发=["useCard", "respond"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinquanbian_xg` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9575` | 触发="phaseZhunbeiBegin",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinquanbian_count` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9588` | 触发='useCard1'；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gxlianhua` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9623` | 丹血 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhafu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9699` | 获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yechou2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9750` | 触发="phaseAfter",；失去体力 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `huamu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9764` | 触发="useCardAfter",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `biaozhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9856` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rebiaozhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9895` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `liezhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:9934` | 触发="phaseZhunbeiBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `juguan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10000` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shouxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10118` | 触发="useCardToTargeted",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reluoyi2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10200` | 裸衣 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `duanchang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10221` | 触发="die",；获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezongshi_paoxiao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10263` | 无限杀 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezishou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10275` | 触发="phaseDrawBegin2",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `decadezishou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10305` | 触发="phaseDrawBegin2",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zishou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10337` | 触发="phaseDrawBegin2",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_duanfa` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10366` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinchoufa` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10423` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhaoran2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10476` | 触发="loseAfter", global: ["equipAfter"…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhishi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10569` | 触发="phaseJieshuBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `neifa_basic` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10648` | 内伐 - 基本牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `neifa_nobasic` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10702` | 内伐 - 非基本牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_guanchao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10779` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_guanchaog2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10923` | 观潮 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_guanchaog3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10961` | 触发="useCard2",；加标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spyilie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:10973` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yinyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11248` | 触发="damageBegin1",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yinyi1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11280` | 隐逸 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spyajun` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11301` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bingjie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11411` | 触发="phaseUseBegin",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `cslilu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11452` | 触发="phaseDrawBegin1",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jinglve` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11532` | 选按钮 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jinglve2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11602` | 死士 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jinglve3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11640` | 触发=["loseAfter", "useCard", "phaseAft…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezhoufu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11667` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinzhoufu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11771` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhuitao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11829` | 触发="phaseZhunbeiBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `decadejingce` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11898` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `decadejingce1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:11942` | 触发='useCard2',；加标记、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `fengjie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12003` | 触发="phaseZhunbeiBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xingbu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12042` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xuezhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12160` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `guowu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12271` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_daigong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12364` | 触发="damageBegin4",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_daigong1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12401` | 怠攻 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bingqing` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12422` | 触发="useCardAfter",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bingqing1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12520` | 触发='useCard2',；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `remeibu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12559` | 触发="phaseUseBegin",；弃牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `olhaoshi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12626` | 触发="phaseDrawBegin2",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `decadexianzhen` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12766` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinxianzhen` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12843` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rexianzhen` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12921` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jiaozhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:12999` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejiaozhao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13119` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcjianying` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13494` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcjianying2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13529` | 触发="useCard2",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `oljuanxia` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13565` | 触发="phaseJieshuBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhenge` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13722` | 触发="phaseZhunbeiBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qimei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13804` | 触发="phaseZhunbeiBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `ybzhuiji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13898` | 触发="phaseUseBegin",；选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xunyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13960` | 触发=["phaseBefore", "dieAfter"], playe…；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xunyi2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:13998` | 触发="damageSource",；摸牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhiwei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14018` | 触发=["enterGame", "showCharacterAfter"…；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhiwei2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14057` | 触发="damageSource",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `huguan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14146` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhente` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14215` | 触发="useCardToTargeted",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xiangshu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14266` | 触发="phaseJieshuBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xingongji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14336` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xingongji2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14431` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `fanghun` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14451` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_xionghuo` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14658` | 触发="phaseUseBegin",；移标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xionghuo_disable` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14798` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xionghuo_low` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14812` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_shajue` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14826` | 触发="dying",；获得牌、加标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xunli` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14844` | 触发='loseAfter'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `panshi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14945` | 触发='phaseZhunbeiBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jiufa` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:14989` | 触发='useCardAfter'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `yingba` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15080` | 加标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `new_wuhun` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15131` | 触发="damageEnd",；加标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `baonu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15217` | 暴怒 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mjdingyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15244` | 触发='phaseBefore', player: 'enterGame',；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zuici` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15359` | 触发="damageEnd",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mjfubi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15396` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `fanbei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15497` | 翻倍 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spwuku` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15506` | 触发='useCard'；加标记 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_jueyan1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15528` | 决堰 - 武器 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_jueyan2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15541` | 决堰 - 坐骑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_jueyan3` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15554` | 决堰 - 防具 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_zhenggu_mark` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15568` | 镇骨 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `liunian` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15580` | 触发='washCard'；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `bifa2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15636` | 触发="phaseZhunbeiBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rewenji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15694` | 触发="phaseUseBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `regushe` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15771` | 临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `new_meibu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15889` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `new_zhixi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:15953` | 触发="useCard",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `new_zhixi_recover` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16014` | 触发="changeHp",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_duorui` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16030` | 触发="damageSource",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `duorui_clear` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16111` | 触发=['phaseAfter', 'dieAfter'],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_duorui1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16129` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shenzhu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16157` | 触发="useCardAfter",；选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `refenyin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16212` | 触发=["loseAfter", "cardsDiscardAfter"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `refenyin_1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16291` | 触发=["loseAfter", "cardsDiscardAfter"],；加标记、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `residi_push` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16324` | 触发="phaseUseBegin",；选按钮 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_fuyin1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16374` | 父荫 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_fuyin2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16395` | 触发="useCardToTargeted",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_fuyin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16408` | 触发="useCardToTargeted",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dczhanyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16431` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezhuikong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16665` | 触发="phaseZhunbeiBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_wanglie2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16717` | 不能用牌啦 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jilei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16731` | 触发="damageEnd",；选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jilei2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16787` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `gzjilei` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16829` | 选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `zhaxiang2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16890` | 诈降 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xiantu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16924` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xianghai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16972` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jishe` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:16988` | 摸牌、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejiangchi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17036` | 触发="phaseUseBegin",；选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qimou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17091` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcxianzhu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17179` | 触发="damageSource",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `olzeyue` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17314` | 触发="phaseZhunbeiBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `dcchaixie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17472` | 触发=["loseAfter"], global: ["equipAfte…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `reqimou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17510` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `wfyuyan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17600` | 触发="roundStart",；获得技能、选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qinggang2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17671` | 触发=["damage", "damageCancelled", "dam…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qiangwu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17700` | 判定、临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `chengzhang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17740` | 触发="phaseZhunbeiBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `chengzhang_a` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17771` | 成章 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejiushi_mark` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17786` | 触发="phaseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spxizhan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17794` | 触发="phaseBegin",；弃牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mobiledanshou` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17898` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mobiledanshou_cc` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17951` | 触发="useCardToTarget",；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jiaoying` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:17979` | 触发="gainEnd",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `kuangcai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18018` | 触发="phaseUseBegin",；临时技、获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `jielie` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18123` | 触发="phaseBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezhiyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18230` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rezhiyi1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18321` | 执义 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rebotu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18344` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinfu_qianchong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18437` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xiefang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18513` | 触发=["dieEnd", "gameDrawEnd"],；获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qingzhongx` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18544` | 触发="phaseUseBegin",；摸牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `tianyin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18617` | 触发="phaseJieshuBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `tianyin1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18650` | 触发='useCardBegin',；临时技 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `choulve` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18694` | 触发="phaseUseBegin",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `choulve1` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18761` | 触发="damageBegin",；加标记、获得技能 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mingfa` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18786` | 触发="phaseUseBegin",；选目标 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `qizhi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:18945` | 触发="useCardToPlayered",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `huaiyuan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19009` | 触发="loseAfter", global: ["equipAfter"…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `rejigong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19196` | 触发="phaseUseBegin",；摸牌、临时技、选选项 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `huomo_a` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19249` | 活墨 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `huomo_use` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19261` | sha | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `sbliegong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19429` | 触发="useCardToPlayered",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `ruyijingubang_skill` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19601` | 是否发动【如意棒】，改变攻击距离并获得对应效果？ | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `shencai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19689` | 请选择一名其他角色 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xunshi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:19906` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `fenyin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20004` | 触发='useCard'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `xinshanjia` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20058` | 触发="loseEnd",；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `baiyi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20192` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `aocai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20253` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `new_luanji` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20302` | wanjian | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `juliao` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20429` | 触发=['gameStart', 'die']；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `baijia` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20449` | 触发='phaseZhunbeiBegin'；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `spjungong` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20484` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `mbjuejin` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20568` | 音效 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_CMzhenwang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:20741` | SS_CM_zhenwang | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_recasting` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:3336` | 将要重铸的牌置入弃牌堆并摸一张牌 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_xunizhuangbei_gua_gua` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:3464` | 触发=["gameStart", 'die', 'phaseEnd', '…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_gshoupai` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:3748` | 触发=["gainEnd", "loseEnd", "dieBefore"…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `drlt_poxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:6772` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `pro_drlt_poxi` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:6887` | 脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_gzyinni` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:5` | 触发=["gameStart", "dieBegin"], player:…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_guozhan` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:339` | 触发="gameStart", player: "showCharacte…；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_gzyinniszn` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:391` | 触发=["gameStart", "dieBegin"], player:…；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_guozhanszn` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:504` | 触发="gameStart", player: "showCharacte…；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_yinni` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:556` | 触发=["gameStart", "showCharacterEnd"],；脚本逻辑 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_shousha_shuangjiang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:817` | 触发=['gameStart'], //player: 'enterGam…；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_shizhounian_shuangjiang` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:944` | 触发='gameStart'；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |
| `_shizhounian_shuangjiang2` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:993` | 触发=["showCharacterAfter", "dieBegin"],；十周年UI渲染 | 原生前端组件 | 缺失（前端 UI 钩子未在原生前端完整复刻） |

### 7.8 渲染面（Rendering Surfaces）（43 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `animation` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816` | 前端脚本模块 `animation.js`（1816 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `animation_new` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation_new.js:1-1832` | 前端脚本模块 `animation_new.js`（1832 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `animationlist` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animationlist.js:1-178` | 前端脚本模块 `animationlist.js`（178 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `component` | `archive/upstream/noname-liuli/noname/extension/十周年UI/component.js:1-117` | 前端脚本模块 `component.js`（117 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `content` | `archive/upstream/noname-liuli/noname/extension/十周年UI/content.js:1-487` | 前端脚本模块 `content.js`（487 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `effect` | `archive/upstream/noname-liuli/noname/extension/十周年UI/effect.js:1-334` | 前端脚本模块 `effect.js`（334 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `meihua` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:1-7003` | 前端脚本模块 `meihua.js`（7003 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `menu` | `archive/upstream/noname-liuli/noname/extension/十周年UI/menu.js:1-7668` | 前端脚本模块 `menu.js`（7668 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `pause` | `archive/upstream/noname-liuli/noname/extension/十周年UI/pause.js:1-522` | 前端脚本模块 `pause.js`（522 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `splash` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:1-1737` | 前端脚本模块 `splash.js`（1737 行） | 原生前端组件 | 示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染） |
| `animation.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.css:1-251` | CSS 样式表 `animation.css`（251 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `animation1.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation1.css:1-214` | CSS 样式表 `animation1.css`（214 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `animation_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/animation_new.css:1-230` | CSS 样式表 `animation_new.css`（230 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `card.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/card.css:1-323` | CSS 样式表 `card.css`（323 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `cardX.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/cardX.css:1-450` | CSS 样式表 `cardX.css`（450 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `component.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/component.css:1-158` | CSS 样式表 `component.css`（158 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `decadeLayout.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/decadeLayout.css:1-534` | CSS 样式表 `decadeLayout.css`（534 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `dialog.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/dialog.css:1-1116` | CSS 样式表 `dialog.css`（1116 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `dialog_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/dialog_new.css:1-1180` | CSS 样式表 `dialog_new.css`（1180 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `doudizhu.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/doudizhu.css:1-135` | CSS 样式表 `doudizhu.css`（135 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `effect.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/effect.css:1-424` | CSS 样式表 `effect.css`（424 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `equip.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/equip.css:1-372` | CSS 样式表 `equip.css`（372 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `equip_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/equip_new.css:1-190` | CSS 样式表 `equip_new.css`（190 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `font.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/font.css:1-55` | CSS 样式表 `font.css`（55 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `icon.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/icon.css:1-105` | CSS 样式表 `icon.css`（105 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `junba.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/junba.css:1-559` | CSS 样式表 `junba.css`（559 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `layout.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/layout.css:1-857` | CSS 样式表 `layout.css`（857 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `layout_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/layout_new.css:1-781` | CSS 样式表 `layout_new.css`（781 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `meihua.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.css:1-334` | CSS 样式表 `meihua.css`（334 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `newdialog.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/newdialog.css:1-619` | CSS 样式表 `newdialog.css`（619 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `newdialog_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/newdialog_new.css:1-793` | CSS 样式表 `newdialog_new.css`（793 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `pause.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/pause.css:1-352` | CSS 样式表 `pause.css`（352 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player.css:1-1789` | CSS 样式表 `player.css`（1789 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player1.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player1.css:1-1363` | CSS 样式表 `player1.css`（1363 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player1_PC.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player1_PC.css:1-1363` | CSS 样式表 `player1_PC.css`（1363 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player2.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player2.css:1-1475` | CSS 样式表 `player2.css`（1475 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player2_PC.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player2_PC.css:1-1474` | CSS 样式表 `player2_PC.css`（1474 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player3.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player3.css:1-1361` | CSS 样式表 `player3.css`（1361 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player3_PC.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player3_PC.css:1-1360` | CSS 样式表 `player3_PC.css`（1360 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player_mark.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player_mark.css:1-151` | CSS 样式表 `player_mark.css`（151 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player_mark_fix.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player_mark_fix.css:1-109` | CSS 样式表 `player_mark_fix.css`（109 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `player_new.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/player_new.css:1-1687` | CSS 样式表 `player_new.css`（1687 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |
| `splash.css` | `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.css:1-493` | CSS 样式表 `splash.css`（493 行） | 原生前端组件 | 示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局） |

