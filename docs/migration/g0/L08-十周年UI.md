# G0 · L08 十周年UI

> 核验日期：2026-10-16 · 来源仓：`noname-liuli` · 入口：`archive/upstream/noname-liuli/noname/extension/十周年UI`  
> 本页为 E01/G0 深度清单（功能 / 菜单与规则分支语义 / 目标接口 / 验证入口 / 旧行为样本引用）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/十周年UI/extension.js`（约 12724 行；game.import=True；skills=0; cards=1; characters=2）

### 清点摘要
- 主入口行数：12724
- game.import：True
- 子插件数：0
- 资源：image:4文件/3890829B
- 混淆/动态特征：extension/十周年UI/extension.js: packed=False, eval=1, long_lines=0
- 台账原生重构状态（摘录）：**待重构；未验收**

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
- 台账已核查摘要：牌桌/选择部件、动画、Worker、卡牌/玩家函数覆盖和技能钩子
- 台账待处理（含规则分支）：主牌桌主题 + 共享选择/动画能力；逐个剥离规则补丁。所有选择可操作、取消/恢复正常，皮肤/标记/模式组合及原生帧指标
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
