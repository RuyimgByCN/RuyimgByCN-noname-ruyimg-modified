# G0 · L01 AI优化

> 核验日期：2026-11-09 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/AI优化`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/AI优化/extension.js`（约 10967 行；game.import=True；skills=2; cards=1; characters=2）

### 清点摘要
- 主入口行数：10967
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：extension/AI优化/extension.js: packed=False, eval=6, long_lines=0
- 台账原生重构状态（摘录）：**主装载与 AI 消费面已实现（2026-10-17，[证据](../../operations/evidence/2026-10-17-l01-ai-tuning-eight/README.md)）**：装载器 `ai_tuning`（威胁度/权重/补充模式/效益/评级映射）+ **AI 消费面**（`ContentTuning` + 策略变体 2）+ 转换包 `packages/extensions/ai-tuning-upstream-liuli`。**2026-11-09 裁决 34 清点**：上游 69 技能/规则钩子、1 卡牌（`ly_piliche`）、64 配置项共 134 项中，忠实实现 **133** / 示意或自造 **0** / 缺失 **1**（`ly_piliche` 霹雳车实体卡尚未接入正式包 `cards.json`）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：extension/AI优化/extension.js: config_items≈64
- settings/config 引用次数（清点近似）：0
- 子插件/子目录：无
### 规则 / 依赖分支
- 外部扩展引用：无（清点未检出）
- 台账已核查摘要：技能威胁/武将权重、禁将、身份策略、规则钩子与设置；逐项清单见 [G0/L01](./L01-AI优化.md)
- 台账待处理（含规则分支）：**待将 `ly_piliche` 接入正式包并补移动端真机验收**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `ai`：AI 策略（合法座位视图 + 预算/取消）
- `config`：配置 schema + 内容锁敏感项
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L01-AI优化.md`
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

- **统计汇总**：上游实体合计 **133** 项（武将 0 / 技能 69 / 卡牌 1 / 模式与战役节点 0 / 配置项 63 / 界面钩子 0 / 渲染面 0）
- **状态分布**：`忠实实现` **133** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `lib.card.yiyi`（以逸待劳锦囊，`resolve_l01_yiyi_card` + `evaluate_yiyi_target_benefit`） | `crates/content-native-loader/src/ai_tuning.rs:183-228`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:11-28` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9205-9260` | 原生规则 / `ai-tuning-upstream-liuli` | 忠实实现（已按上游实现 `yiyi` 锦囊摸 2 弃 2 结算与目标收益评估并接入正式包 `ai-tuning-upstream-liuli` 产品路径） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（69 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `_aiyh_firstKs` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:8` | 触发='gameStart'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_nhFriends` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:100` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_meks` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:122` | 触发=['gameStart', 'showCharacterEnd']；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_neiKey` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:231` | 昭告；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_fixEQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:294` | 权重（修改一名角色一张武将牌的权重） | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_fixECf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:359` | 威胁（修改一名角色当前拥有的一项技能的威胁度） | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_fake_prohibited` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:439` | 触发='gameStart', player: 'fixCharacter…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_fixWj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:559` | 伪禁（若选择角色则对这些角色的武将牌进行加入/移出伪禁列表操作，否则从所有武将包选择进行操作） | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_findZhong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:675` | 触发='gameStart'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_mjAiSkill` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:705` | 触发='phaseZhunbeiBegin'；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `aiMangju` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:719` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_attzhi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:870` | 触发='gameStart', player: 'enterGame'；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attcard` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:905` | 触发='useCardToPlayered'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attdamage` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1012` | 触发='damageAfter'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attlosehp` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1105` | 触发='changeHp'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attrecover` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1193` | 触发='recoverEnd'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attgain` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1282` | 触发='gainEnd'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attlose` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1347` | 触发='loseEnd'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attturnover` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1415` | 触发='turnOverEnd'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attbo` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1476` | 触发='roundStart'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attmingbian` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1511` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attmingbian1` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1571` | 触发='phaseJieshuBegin'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attfanzhuang` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1638` | 触发='phaseJieshuBegin'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `attbaolu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1677` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `atttiaofan` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1755` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_qjAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:1818` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_reserved_shan` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2065` | shan | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `aiyh_disabled_shan` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2144` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_wuxie` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2160` | 触发='useCardToPlayered'；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_wuxie_jiedao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2215` | wuxie | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_wuxie_lei` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2232` | 触发='phaseJudgeBefore'；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_wuxie_fy` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2249` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_wugu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2265` | 触发='wuguBegin'；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_wugu_ss_pai` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2280` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_wugu_wss_pai` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2296` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_Discard` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2313` | 触发='chooseToDiscardBefore'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_Discard_key` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2334` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg_Discard_du` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2358` | du | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_Discard_du_useful` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:2375` | jiu | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_fenghuyunlong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3382` | 触发='gameStart'；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_mz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3403` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_tianxiayitong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3433` | 触发=['dieAfter']；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_smz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3465` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `jlnj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3503` | 触发='phaseJieshuBegin'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `tfattzhu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3542` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `tfattzhong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3546` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `atttf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3550` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_yingshilanggu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3554` | 触发=['changeHp', 'dieAfter', 'useSkill…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_tz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3921` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `shyg_tf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3946` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_wanjunpiyi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3973` | 触发='gameStart'；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `chaofeng_common` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:3990` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `chaofeng_rare` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4001` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `chaofeng_epic` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4012` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `chaofeng_legend` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4023` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_shyg` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4034` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `gjcx_zhuAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4074` | 触发='zhuUpdate'；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `gjcx_neiAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4112` | 触发=['phaseUseBegin', 'changeHp', 'die…；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `gjcx_neiZhong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4258` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `gjcx_neiFan` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4292` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `gjcx_neiJiang` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4328` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_lianhe` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4355` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_aiyh_wqjz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:4391` | qinggang | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `xinfu_limu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:5883` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `zhenlie_lose` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:7535` | 失去体力、临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `twzhian_ai` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:7897` | 触发='dieAfter'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `dcwumei_all` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:8031` | 触发='dieAfter'；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `aiyh_gjcx_qj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:8937` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |
| `_gjcx_correct` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9284` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已在 `crates/content-native-loader/src/ai_tuning.rs` 实现） |

### 7.4 卡牌（Cards）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `yiyi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9205` | 摸牌、弃牌 | 原生规则 / `rule-vm` 包 | 忠实实现 |

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（63 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `kzjs` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9856` | 扩展介绍（默认='jieshao'） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd1` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9888` | 可通过无名杀频道、无名杀扩展交流、Q群或下方链接获取本扩展最新版本（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyQg` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9892` | <span style=（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyWp` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9909` | 一键复制百度网盘下载链接（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyYp` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9926` | 一键复制123云盘下载链接（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd2` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9943` | AI相关（默认='shyg'）：主要包括非装备牌、卖血、存牌、弃牌等难以区分的AI策略。开启后，重启游戏可载入对应AI优化策略。其… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `btAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9948` | 本体AI优化（默认='shyg'）：主要包括非装备牌、卖血、存牌、弃牌等难以区分的AI策略。开启后，重启游戏可载入对应AI优化策略。其… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `zbAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9958` | 装备AI优化（默认=true）：全部来自〔山河永固〕AI，针对常见的一些装备牌包括部分装备技能进行了优化 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `wjAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9963` | 武将技能优化（默认=true）：开启后，重启游戏可载入优化后的武将技能，具体优化技能见扩展介绍 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `sfjAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9968` | 身份局AI优化（默认='shyg'）：开启后，重启游戏可载入对应身份局AI策略。其中〔山河永固〕优化内容较全面、较耗时；〔官将重修〕优化… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `nhFriends` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9978` | <span style=（默认='off'）：开启后，AI一般情况下将更不乐意对符合条件的队友使用除【火攻】外的伤害牌 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `ntAoe` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9991` | <span style=（默认=false）：开启后，AI会以更激进的方式盲狙 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `mjAi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9995` | 盲狙AI（默认=false）：开启后，AI会以更激进的方式盲狙 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd3` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10000` | 常用功能（默认=false）：可在顶部菜单栏查看态度 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `viewAtt` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10005` | 火眼金睛（默认=false）：可在顶部菜单栏查看态度 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `xyyh` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10010` | 闲云野鹤（默认=false）：去除手杀孙寒华〖冲虚〗、手杀南华老仙〖御风〗、手杀庞德公〖评才〗、手杀郑玄〖整经〗的小游戏，重启生… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banQh` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10015` | 屏蔽武将切换功能（默认=false）：开启后，会屏蔽选将时武将左下角的切换功能 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `seen` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10020` | 显示隐藏武将（默认=false）：开启后，会显示本体隐藏武将 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `hsyh` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10026` | 本体函数优化（默认=true）：优化展示牌时长(showCards)、拼点AI(chooseToCompare) | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd4` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10031` | 身份局相关功能（默认=false）：主公开局知道一名忠臣身份 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `findZhong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10036` | <span style=（默认=false）：主公开局知道一名忠臣身份 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `neiKey` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10041` | <span style=（默认='off'）：内奸可以于出牌阶段直接亮明身份并加1点体力上限，然后可以选择与主公各回复1点体力，建议与［身份局A… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `attceshi` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10051` | 态度测试（默认=true//false）：仅在标准身份局生效，优化场上态度，且玩家可装身份。由于此功能涉及到底层修改，不建议轻易使用 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `fixFour` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10056` | 四人身份局修改（默认=false）：将四人身份局改为1主（加体力上限）1内2反，游戏开始时忠臣改为明反 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd5` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10061` | 伪禁相关（默认=false）：开启后，游戏开始或隐匿武将展示武将牌时，若场上有ai选择了伪禁列表里包含的ID对应武将，则勒令其从… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `Wj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10065` | 伪玩家可选ai禁选（默认=false）：开启后，游戏开始或隐匿武将展示武将牌时，若场上有ai选择了伪禁列表里包含的ID对应武将，则勒令其从… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `wjs` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10070` | 伪禁候选武将数（默认='same'）：〔默认〕即游戏开始时每名角色的候选武将数，若为自由选将等特殊情况则默认为6 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `fixWj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10086` | <span style=（默认=false）：出牌阶段可将场上武将加入/移出伪禁列表，也可以从若干个武将包中选择武将执行此操作 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `editWj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10091` | 编辑伪禁列表（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyWj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10174` | 一键复制伪禁列表（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `clearWj` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10195` | 清空伪禁列表（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `tip1` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10205` | 以下功能为伪禁衍生功能，如需使用请开启〔伪玩家可选ai禁选〕（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banzhu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10209` | 主公AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banzhubiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10250` | 主公AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banzhong` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10274` | 忠臣AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banzhongbiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10315` | 忠臣AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banfan` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10339` | 反贼AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banfanbiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10380` | 反贼AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bannei` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10404` | 内奸AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `banneibiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10445` | 内奸AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bandizhu` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10469` | 地主AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bandizhubiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10510` | 地主AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bannongmin` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10534` | 农民AI禁将（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bannongminbiao` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10575` | 农民AI禁选表(点击查看)（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `bd6` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10599` | 杂项（默认=false）：出牌阶段可以设置/修改场上武将的权重，以此影响内奸AI策略 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `tip2` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10604` | 注意！通过以下功能设置的权重将优先作为内奸AI判断场上角色实力的参考（默认=false）：出牌阶段可以设置/修改场上武将的权重，以此影响内奸AI策略 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `fixEQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10608` | <span style=（默认=false）：出牌阶段可以设置/修改场上武将的权重，以此影响内奸AI策略 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `applyQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10613` | 武将登场补充权重（默认=false）：游戏开始或隐匿武将展示武将牌时会建议玩家为没有设置权重的武将设置权重 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `ckQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10618` | <span style=（默认='cf'）：开启后，针对没有设置权重的武将，〔评级〕会根据武将评级为这些武将分配正相关的权重[0.8,1.95… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `qzCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10628` | <span style=（默认=true//false）：开启后，ai在计算卡牌效益时会将权重（若无则为1）与结果作积返回，以此鼓励ai优先“待遇”权重大的… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `chooseQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10633` | 武将权重选项加载失败（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `deleteQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10638` | 删除此权重（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `editQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10651` | 编辑武将权重（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10742` | 一键复制武将权重（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `clearQz` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10763` | 清空设置的武将权重（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `tip3` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10777` | 注意！通过以下功能修改的技能威胁度会覆盖技能原有的威胁度&nbsp&nbsp&…（默认=false）：出牌阶段可以修改场上武将当前拥有的技能的威胁度，一定程度上为AI提供集火优先级 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `fixECf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10781` | <span style=（默认=false）：出牌阶段可以修改场上武将当前拥有的技能的威胁度，一定程度上为AI提供集火优先级 | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `applyCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10786` | 威胁度补充（默认='off'）：〔自动补充〕会在进入游戏时根据武将评级对没有添加威胁度的武将技能增加一定威胁度，单机时可通过〈千幻… | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `chooseCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10796` | 技能威胁度选项加载失败（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `deleteCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10801` | 删除此修改项（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `editCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10814` | 编辑修改的技能威胁度（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `copyCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10905` | 一键复制修改的技能威胁度（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |
| `clearCf` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:10926` | 清空修改的技能威胁度（默认=—） | 原生规则（`crates/content-native-loader/src/ai_tuning.rs`） | 忠实实现 |

### 7.7 界面钩子（UI Hooks）（0 项）

- （上游无此类别实体）

### 7.8 渲染面（Rendering Surfaces）（0 项）

- （上游无此类别实体）

