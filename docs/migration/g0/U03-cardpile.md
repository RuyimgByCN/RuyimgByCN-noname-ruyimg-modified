# G0 · U03 cardpile

> 核验日期：2026-11-10 · 来源仓：`noname` · 入口：`archive/upstream/noname/apps/core/extension/cardpile`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/cardpile/extension.js`（约 100 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：100
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**规则与正式包忠实实现（2026-10-17，[证据](../../operations/evidence/2026-10-17-t12-u03-eight/README.md)）**：真实包 `packages/extensions/cardpile-upstream-noname` 可发现/导入/激活，比例表桥接为 113 张唯一实体牌入目录，扩展酒/杀在局中按引擎规则结算，存档按集合绑定恢复。十二项倍率全为 1、牌堆长度 207 时追加 113 张；杀倍率 2 时杀方片 33、火杀红桃 8；牌堆 600 时杀方片 53、铁索梅花 35。缺任一倍率整段不追加；未知键和负倍率拒绝。**2026-11-09 裁决 34 清点**：上游 29 类牌堆条目与 1 个模式节点共 30 项全部忠实实现（忠实实现 **30** / 示意或自造 **0** / 缺失 **0**）

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
- 台账已核查摘要：牌堆比例、花色和随机点数；来源含 `jiu`、`leisha`；逐项清单见 [G0/U03](./U03-cardpile.md)
- 台账待处理（含规则分支）：**仓内规则与正式包已忠实实现（随机点数由确定性序列注入仍为登记差异）；待移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `content`：内容包（武将/技能/卡牌目录合并 + rule-vm/宿主行为）
- `config`：配置 schema + 内容锁敏感项
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/U03-cardpile.md`
- 台账行：`docs/migration/upstream-extension-reconstruction.md` 对应 ID
- 功能面基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/feature-inventory.json`
- 原生包生命周期/内容锁：`crates/content-runtime` + 对应 packages/extensions（若已有转换包）
- 无头启动探针：`node scripts/legacy-harness/probe-liuli-boot.mjs <scratch/liuli根>`

## 5. 旧行为动态样本引用

- ../operations/evidence/2026-10-07-e01-feature-inventory/README.md（功能面清点基线）
- ../operations/evidence/2026-10-08-u03-cardpile-lifecycle/README.md
- scripts/legacy-harness/（无头旧线采样入口；运行副本 scratch/legacy-harness）

## 6. 阻塞与后续

- G0 本文档完成不等于 G1–G5；整包状态仍以[台账](../upstream-extension-reconstruction.md)为准。
- 若需补跑动态样本：在 `scratch/legacy-harness` 运行副本上使用 `scripts/legacy-harness/`，勿写 `archive/upstream`。

## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-10）

- **统计汇总**：上游实体合计 **30** 项（武将 0 / 技能 0 / 卡牌 29 / 模式与战役节点 1 / 配置项 0 / 界面钩子 0 / 渲染面 0）
- **状态分布**：`忠实实现` **30** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

- 本来源未被 `decision34_closure.rs` / `decision34_pack9.rs` 注入自造武将或默认摸 1 技能。

### 7.2 武将（Generals）（0 项）

- （上游无此类别实体）

### 7.3 技能（Skills）（0 项）

- （上游无此类别实体）

### 7.4 卡牌（Cards）（29 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `pile_diamond_10` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:10` | 牌堆花色基数权重 `diamond: 6`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_11` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:11` | 牌堆花色基数权重 `club: 14`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_12` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:12` | 牌堆花色基数权重 `heart: 3`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_13` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:13` | 牌堆花色基数权重 `spade: 7`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_16` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:16` | 牌堆花色基数权重 `diamond: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_17` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:17` | 牌堆花色基数权重 `heart: 3`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_20` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:20` | 牌堆花色基数权重 `spade: 5`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_21` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:21` | 牌堆花色基数权重 `club: 4`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_24` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:24` | 牌堆花色基数权重 `heart: 6`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_25` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:25` | 牌堆花色基数权重 `diamond: 18`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_28` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:28` | 牌堆花色基数权重 `diamond: 1`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_29` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:29` | 牌堆花色基数权重 `spade: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_30` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:30` | 牌堆花色基数权重 `club: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_33` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:33` | 牌堆花色基数权重 `heart: 9`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_34` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:34` | 牌堆花色基数权重 `diamond: 3`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_37` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:37` | 牌堆花色基数权重 `heart: 1`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_40` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:40` | 牌堆花色基数权重 `spade: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_41` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:41` | 牌堆花色基数权重 `club: 1`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_44` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:44` | 牌堆花色基数权重 `spade: 3`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_45` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:45` | 牌堆花色基数权重 `club: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_46` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:46` | 牌堆花色基数权重 `heart: 1`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_49` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:49` | 牌堆花色基数权重 `spade: 3`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_50` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:50` | 牌堆花色基数权重 `diamond: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_heart_53` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:53` | 牌堆花色基数权重 `heart: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_diamond_54` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:54` | 牌堆花色基数权重 `diamond: 1`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_55` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:55` | 牌堆花色基数权重 `spade: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_56` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:56` | 牌堆花色基数权重 `club: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_spade_59` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:59` | 牌堆花色基数权重 `spade: 2`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |
| `pile_club_60` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:60` | 牌堆花色基数权重 `club: 4`（按 `cardpile_*_playpackconfig` 缩放） | 原生规则（`pile_bridge.rs`） | 忠实实现 |

### 7.5 模式 / 战役节点（Modes & Campaign Nodes）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `cardpile_arenaReady` | `archive/upstream/noname/apps/core/extension/cardpile/extension.js:6` | 按 `data` 160 张基数与比例配置向 `lib.card.list` 追加牌堆条目（上限 1000） | 原生规则（`pile_bridge.rs`） | 忠实实现 |

### 7.6 配置项（Config Items）（0 项）

- （上游无此类别实体）

### 7.7 界面钩子（UI Hooks）（0 项）

- （上游无此类别实体）

### 7.8 渲染面（Rendering Surfaces）（0 项）

- （上游无此类别实体）

