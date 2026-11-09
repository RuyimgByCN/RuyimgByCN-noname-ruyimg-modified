# G0 · L02 SJ Settings

> 核验日期：2026-11-10 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/SJ Settings`  
> 本页为 E01/G0 深度清单与**裁决 34 逐项上游清单**（武将 / 技能 / 卡牌 / 模式与战役节点 / 配置项 / 界面钩子 / 渲染面 / 现有 `decision34*` 归类）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/SJ Settings/extension.js`（约 737 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：737
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：extension/SJ Settings/extension.js: packed=False, eval=1, long_lines=0
- 台账原生重构状态（摘录）：**配置与装载面忠实实现（2026-10-17，[证据](../../operations/evidence/2026-10-17-t12-l02-eight/README.md)）**：真实包 `packages/extensions/sj-settings-upstream-liuli`。安卓键为空时启用名单是 SJ Settings，缺省 `asset_version` 写成 `无`。`{"a":1}` 编码为 `eyJhIjoxfQ==`。数据库导入保留/不保留旧名单、版本号比较（`1.2` vs `1.10`）、不兼容目录项拒绝均已实测；不请求网络，不 eval update.js。**2026-11-09 裁决 34 清点**：4 配置项 + 1 工作台渲染面共 5 项全部忠实实现（忠实实现 **5** / 示意或自造 **0** / 缺失 **0**）

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

### 菜单 / 配置分支
- 入口配置项计数（清点近似）：extension/SJ Settings/extension.js: config_items≈5
- settings/config 引用次数（清点近似）：0
- 子插件/子目录：无
### 规则 / 依赖分支
- 外部扩展引用：SJ Settings
- 台账已核查摘要：Android 外部打开扩展、配置导入、下载入口和调试台；逐项清单见 [G0/L02](./L02-SJ-Settings.md)
- 台账待处理（含规则分支）：**仓内 5/5 项已忠实实现；待移动端真机验收后签署整包**
- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

- `platform`：平台桥（导入/分享/更新/拖放）
- `config`：配置 schema + 内容锁敏感项
- `lifecycle`：包生命周期（安装/启停/升级/卸载）

## 4. 验证入口

- G0 本文档：`docs/migration/g0/L02-SJ-Settings.md`
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

- **统计汇总**：上游实体合计 **5** 项（武将 0 / 技能 0 / 卡牌 0 / 模式与战役节点 0 / 配置项 4 / 界面钩子 0 / 渲染面 1）
- **状态分布**：`忠实实现` **5** 项 · `示意或自造` **0** 项 · `缺失` **0** 项

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

### 7.6 配置项（Config Items）（4 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `tutorialapk` | `archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.js:338` | 查看apk使用教程（默认=false）：查看apk使用教程 | 原生规则（`crates/content-native-loader/src/sj_settings.rs`） | 忠实实现 |
| `openTools` | `archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.js:346` | 启用控制台（默认=false）：启用eruda控制台 | 原生规则（`crates/content-native-loader/src/sj_settings.rs`） | 忠实实现 |
| `getExtensions` | `archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.js:359` | 点击获取扩展（默认=—）：点击获取扩展 | 原生规则（`crates/content-native-loader/src/sj_settings.rs`） | 忠实实现 |
| `tutorial` | `archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.js:710` | 查看无名杀教程（默认=—）：查看无名杀教程 | 原生规则（`crates/content-native-loader/src/sj_settings.rs`） | 忠实实现 |

### 7.7 界面钩子（UI Hooks）（0 项）

- （上游无此类别实体）

### 7.8 渲染面（Rendering Surfaces）（1 项）

| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|
| `extension.css` | `archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.css:1-3` | CSS 样式表 `extension.css`（3 行） | 原生前端组件 | 忠实实现 |

