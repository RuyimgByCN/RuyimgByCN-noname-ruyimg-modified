# G0 · L03 boss

> 核验日期：2026-11-09 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/boss`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/boss/extension.js`（约 138 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：138
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**规则与装载面忠实实现（2026-10-17，[证据](../../operations/evidence/2026-10-17-t12-u02-l03-boss-play-eight/README.md)）**：真实包 `packages/extensions/boss-play-upstream-liuli`（与 U02 共享 `boss_playpack` 模块，liuli 变体；另含 `boss-zhugeliang-upstream-liuli` 共 2 包）。包键 `mode_extension_boss`/`mode_extension_jiange`，翻译键与 `mode:boss`/`mode:versus` 标签、`bossallowed` 解析、8 条 intro 别名。**2026-11-09 裁决 34 清点勘正**：`decision34::jiange_siege_step` 在上游 `L03/extension.js` 中不存在（属自造步骤，归入待删清退）；上游 8 项 intro 角色条目均已忠实实现（忠实实现 **8** / 示意或自造 **0** / 缺失 **0**，另 `decision34` 自造 1 项待删）

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
- 台账已核查摘要：mode_extension_boss/jiange 注册、旧数组式角色与缓存；逐项清单见 [G0/L03](./L03-boss.md)
- 台账待处理（含规则分支）：**待删除 `decision34::jiange_siege_step` 自造代码，并待移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `content`：内容包（武将/技能/卡牌目录合并 + rule-vm/宿主行为）
- `mode`：模式扩展（建局/胜负/关卡流程钩子）
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L03-boss.md`
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

- **统计汇总**：上游实体合计 **8** 项（武将 8 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 0 / 界面钩子 0 / 渲染面 0）
- **状态分布**：`忠实实现` **8** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

### 7.1 现有 `decision34*` / 前端示意代码逐条归类

| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |
|---|---|---|---|---|
| `jiange_siege_step` / `boss_playpack.rs` | `crates/content-native-loader/src/boss_playpack.rs:1-120` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:5-136` | 原生规则（`boss_playpack.rs`） | 忠实实现（挑战/剑阁武将池缓存与体力归一化已在 `boss_playpack.rs` 忠实实现） |

### 7.2 武将（Generals）（8 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `boss_liedixuande` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:41` | 剑阁/挑战武将简介映射 `boss_liedixuande` → `liubei`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_gongshenyueying` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:42` | 剑阁/挑战武将简介映射 `boss_gongshenyueying` → `huangyueying`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_tianhoukongming` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:43` | 剑阁/挑战武将简介映射 `boss_tianhoukongming` → `shen_zhugeliang`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_yuhuoshiyuan` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:44` | 剑阁/挑战武将简介映射 `boss_yuhuoshiyuan` → `pangtong`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_qiaokuijunyi` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:45` | 剑阁/挑战武将简介映射 `boss_qiaokuijunyi` → `zhanghe`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_jiarenzidan` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:46` | 剑阁/挑战武将简介映射 `boss_jiarenzidan` → `caozhen`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_duanyuzhongda` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:47` | 剑阁/挑战武将简介映射 `boss_duanyuzhongda` → `simayi`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |
| `boss_juechenmiaocai` | `archive/upstream/noname-liuli/noname/extension/boss/extension.js:48` | 剑阁/挑战武将简介映射 `boss_juechenmiaocai` → `xiahouyuan`（`boss_storage_playpackconfig`） | 原生规则（`boss_playpack.rs`） | 忠实实现 |

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

### 7.8 渲染面（Rendering Surfaces）（0 项）

- （上游无此类别实体）

