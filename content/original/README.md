# 原仓库内容同步

`catalog.json` 是原仓库武将、技能和皮肤的机器可读总目录；`legacy-source/` 保留迁移所需
的原始武将包源码，`skill-capability-matrix.*` 由现有 `legacy-analyzer` 生成。

## 状态口径（2026-09-16 整理）

此目录服务于 `engine-core` 旧参考线。当前目录、技能画像及资源缺口读取 `catalog.json`；
技能注释分类与空实现标记读取 [保真度台账](../../docs/content-fidelity-ledger.md)。
不在本页重复维护计数，`ported` 与无近似注释均不表示可玩语义完成。

历史同步与逐批差异保留在 [旧线迁移记录](../../docs/migration/upstream-1.11.5-delta.md)；
新原生线的正式规则来源、覆盖与行为差异见 [原生覆盖清单](../../docs/native-coverage-manifest.md)。
旧清单不会自动计入新线的迁移完成度。

资源体积超过 10 GiB，不进入 Git。同步器在同一文件系统上使用硬链接，跨文件系统才复制：

```bash
node scripts/sync-original-content.mjs
node scripts/sync-original-content.mjs --verify-only
node scripts/sync-original-content.mjs --skin-source /path/to/reviewed/apps/core
```

默认核心源目录是 `../archive/upstream/noname`；也接受直接传入 `apps/core`，可用
`--core-source PATH`（兼容 `--source PATH`）或 `NONAME_ORIGINAL_ROOT` 覆盖。外部皮肤源必须
显式传 `--skin-source`，其许可状态会与核心来源分开记录；未审来源不会被标记为可再分发。

同步只迁移数据、源码和资源，不会在运行时执行旧 JavaScript。武将替身、皮肤共享和动态皮肤
配置仅通过 TypeScript AST 白名单读取安全字面量，拒绝调用、动态属性和任意代码求值。技能的
`migration_status: source_synced` 表示源码已同步但尚未完成 DSL/VM 语义重写；只有
`ported` 才能进入确定性对局。旧 JS 的 group/global helper 若已由父技能完整吸收，也标为
`ported`，并以 `covered_by` 指向实际承载规则的父技能，不生成重复的运行时技能。

空 DSL 技能仅允许对应旧 JS 确实只有 `charlotte`、客户端 AI 提示（含其客户端助手生命周期清理）、
音频元数据，或已在显式审计清单中折叠进父技能的旧全局助手；同步校验同时检查 TypeScript AST
属性白名单、显式审计清单和能力画像，拒绝把继承技、权威触发器、主动技、转换技、修正器或效果
伪装成空实现。

武将闭包除技能引用外还必须审计旧角色元数据：`isUnseen`、0 体力隐藏形态、`dualSideCharacter`
等切换关系都不是可独立发布的普通武将。在对应元规则完成前，同步器或候选脚本即使发现其技能
均已移植，也不得把这些形态加入选将池。

`ported` 武将还必须满足精确技能闭包：旧武将声明的每个技能都已移植，npack 中的技能 ID
数量与顺序完全一致。仅因角色 ID 已进入某个 npack，不再视为迁移完成；无依赖包在装入前校验，
跨包依赖则在内容锁完整载入后统一拒绝仍不可用的技能引用。

闭包审计还必须递归检查旧技能里的动态授予、派生和分组引用，不能只看武将声明的第一层
`skills`。例如 `key:sasami_miaobian` 运行时还会授予尚未移植的 `sasami_funan` 与
`sasami_baoqiu`，所以 `key_sasami` 不能因表面只差一个技能就提前发布。

`ported/newjiang/` 已建立独立 npack 与扩展目录；首个技能「轶祖」不会因同包其余技能尚未重写
而被合并进其他包。

旧魏延位于 `old`，其「狂骨」原始定义位于 `shenhua`；旧徐盛同样位于 `old`，其「破军」定义
位于 `yijiang`。武将与技能均按上游归属分别打包，运行时通过稳定技能 ID 组合，不复制技能定义。
旧朱然同样只在 OLD 保存武将定义，通过 YIJIANG 依赖获得 ABI 286 重写的「胆守」；发动后清空
共享处理区、终止当前结算并结束当前阶段，但不会把该阶段记为“跳过”。
标吕范位于 `sixiang`，其「调度」原始定义位于 `shiji`；SIXIANG npack 以包依赖引用 SHIJI 中的
稳定技能 ID，验证了直接投放扩展目录时跨包武将闭包无需复制技能实现。

## 迁移流水日志（已拆分）

逐武将/逐批次的迁移完成记录（93 条「第 N 名完整武将」、逐 ABI 落地细节）已拆分至
[`ported-character-log.md`](./ported-character-log.md)；**新条目追加到该文件末尾**，
本 README 只保留同步状态汇总与规则。
