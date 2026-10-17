# G0 · L02 SJ Settings

> 核验日期：2026-10-16 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/SJ Settings`  
> 本页为 E01/G0 深度清单（功能 / 菜单与规则分支语义 / 目标接口 / 验证入口 / 旧行为样本引用）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/SJ Settings/extension.js`（约 737 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：737
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：extension/SJ Settings/extension.js: packed=False, eval=1, long_lines=0
- 台账原生重构状态（摘录）：**八项整包已验收（2026-10-17）；不请求网络、不 eval update.js、不打开 PDF、不启动 eruda、不调用 game.reload 或 game.loop。Cordova、进度条和通知仍为登记差异**。真实包 `packages/extensions/sj-settings-upstream-liuli`。G0 本页不是该次上调的证明，证据见 [L02 八项](../../operations/evidence/2026-10-17-t12-l02-eight/README.md)。L21 计入后总账为 15/37

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
- 台账已核查摘要：Android 外部打开扩展、配置导入、下载入口和调试台
- 台账待处理（含规则分支）：八项公式已从真实安装验收（[L02 八项](../../operations/evidence/2026-10-17-t12-l02-eight/README.md)）。不请求网络，不 eval update.js，不打开 PDF，不启动 eruda，不调用 game.reload 或 game.loop。Cordova、进度条和通知没有迁入。数据库导入要求 config 与 data 都是对象，比上游数组为真更严
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
