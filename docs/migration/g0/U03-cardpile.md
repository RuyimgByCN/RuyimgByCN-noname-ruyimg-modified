# G0 · U03 cardpile

> 核验日期：2026-10-16 · 来源仓：`noname` · 入口：`archive/upstream/noname/apps/core/extension/cardpile`  
> 本页为 E01/G0 深度清单（功能 / 菜单与规则分支语义 / 目标接口 / 验证入口 / 旧行为样本引用）。非整包验收证明。

## 1. 功能清单

### 入口与形态
- `extension/cardpile/extension.js`（约 100 行；game.import=True；skills=None; cards=None; characters=None）

### 清点摘要
- 主入口行数：100
- game.import：True
- 子插件数：0
- 资源：无统计
- 混淆/动态特征：无
- 台账原生重构状态（摘录）：**部分重构；纵向样本已验收（第 18 轮）**：真实包 `packages/extensions/cardpile-upstream-noname` 可发现/导入/激活，比例表桥接为 113 张唯一实体牌入目录，扩展酒/杀在局中按引擎规则结算，存档按集合绑定恢复

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
- 台账已核查摘要：牌堆比例、花色和随机点数；来源含 `jiu`、`leisha`
- 台账待处理（含规则分支）：倍率缩放（160 总量补足）与上游随机点数未映射（点数为确定性流水号，随机性由建局洗牌承接）——登记为与上游牌堆逐张等价的剩余差距；非本体倍率配置分支未逐项对照。**整包验收完成（2026-10-08，[cardpile 生命周期](../operations/evidence/2026-10-08-u03-cardpile-lifecycle/README.md)）**：双仓同装→noname 仓 113 张建局→对局中 revoke_slot 仅撤 noname 仓→在局锁定副本续玩→重装 Revoked 守卫→重新 import 被拒（liuli 仓激活不受影响）
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
