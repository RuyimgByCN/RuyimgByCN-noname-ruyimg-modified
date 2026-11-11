# G0 · L06 wuxing

> 核验日期：2026-11-11 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/wuxing`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/wuxing/extension.js`（约 229 行；game.import=True；skills=2; cards=1; characters=None）

### 清点摘要
- 主入口行数：229
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**全源 3/3 项已复核等价（2026-11-12，`l06-verifier` 独立逐行复核，[复核报告](../../operations/evidence/2026-11-12-l06-wuxing-review/README.md)、[G0/L06](./L06-wuxing.md)）**：真实包 `packages/extensions/wuxing-upstream-liuli` 通过 `host_skill_bindings.rs` 与原生 `HostBehavior::{ShengkeCheck,ChangeCardNature}` 完整覆盖 `wuxingpan`（♠5 宝物栏装备授 `wuxingpan_skill`）、`wuxingpan_skill`（出牌阶段限 1 次两段选择永久改手牌属性）与 `_shengke`（`useCardToBegin` 五组相生摸 1、五组相克发起 `shengke_discard` 强制自选弃 1 张 `he` 牌及弃装备撤技、AI 按 `get.disvalue` 弃最低价值牌），6 维真实对局测试全绿（已复核等价 **3** / 简化近似 **0** / 缺失 **0**）

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
- 台账已核查摘要：五行属性、五行盘卡牌/技能、开局随机插牌；逐项清单见 [G0/L06](./L06-wuxing.md)
- 台账待处理（含规则分支）：**仓内 3/3 项已复核等价（①②③④⑤⑦⑧ 7 维全 `✅`）；仅待第⑥维移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `content`：内容包（武将/技能/卡牌目录合并 + rule-vm/宿主行为）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L06-wuxing.md`
- 台账行：`docs/migration/upstream-extension-reconstruction.md` 对应 ID
- 功能面基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/feature-inventory.json`
- 原生包生命周期/内容锁：`crates/content-runtime` + 对应 packages/extensions（若已有转换包）
- 无头启动探针：`node scripts/legacy-harness/probe-liuli-boot.mjs <scratch/liuli根>`

## 5. 旧行为动态样本引用

- ../operations/evidence/2026-10-07-e01-feature-inventory/README.md（功能面清点基线）
- ../operations/evidence/2026-10-08-u05-wuxing-lifecycle/README.md
- scripts/legacy-harness/（无头旧线采样入口；运行副本 scratch/legacy-harness）

## 6. 阻塞与后续

- G0 本文档完成不等于 G1–G5；整包状态仍以[台账](../upstream-extension-reconstruction.md)为准。
- 若需补跑动态样本：在 `scratch/legacy-harness` 运行副本上使用 `scripts/legacy-harness/`，勿写 `archive/upstream`。

## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-11）

- **统计汇总**：上游实体合计 **3** 项（武将 0 / 技能 2 / 卡牌 1 / 模式与战役节点 0 / 配置项 0 / 界面钩子 0 / 渲染面 0）
- **状态分布**：`已复核等价` **3** 项 · `简化近似` **0** 项 · `缺失` **0** 项 · `上游未定义` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `_wuxing` 全局五行生克技能（`resolve_l06_wuxing_skill`） | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:172-204` | `archive/upstream/noname-liuli/noname/extension/五行生克/extension.js:9-95` | 原生规则 / `wuxing-upstream-liuli` | 简化近似（待复核：已按上游实现 `_wuxing` 初始五行属性与生克结算并接入正式包 `wuxing-upstream-liuli` 产品路径，支持与 `shahai-zhenfa` 任意顺序共存去重） |

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（2 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `_shengke` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62` | 触发='useCardToBegin'；弃牌 | 原生规则 / `rule-vm` 包 | 已复核等价（复核人: l06-verifier, 日期: 2026-11-12, 对照: archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172, 测试: l06_shengke_six_dimension_real_match_equivalence） |
| `wuxingpan_skill` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:173` | 五行：出牌阶段限一次，你可以永久改变一张手牌的五行属性 | 原生规则 / `rule-vm` 包 | 已复核等价（复核人: l06-verifier, 日期: 2026-11-12, 对照: archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:173-193, 测试: l06_wuxingpan_and_skill_six_dimension_real_match_equivalence） |

### 7.4 卡牌（Cards）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `wuxingpan` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:196` | 五行盘：出牌阶段限一次，你可以永久改变一张手牌的五行属性 | 原生规则 / `rule-vm` 包 | 已复核等价（复核人: l06-verifier, 日期: 2026-11-12, 对照: archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:7,196-201, 测试: l06_wuxingpan_and_skill_six_dimension_real_match_equivalence） |

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（0 项）

- （上游无此类别实体）

### 7.6 配置项（Config Items）（0 项）

- （上游无此类别实体）

### 7.7 界面钩子（UI Hooks）（0 项）

- （上游无此类别实体）

### 7.8 渲染面（Rendering Surfaces）（0 项）

- （上游无此类别实体）

