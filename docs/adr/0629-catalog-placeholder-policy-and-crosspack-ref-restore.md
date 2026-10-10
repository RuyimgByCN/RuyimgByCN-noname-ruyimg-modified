# ADR 0629：catalog 占位符策略对齐 + 跨包技能引用回滚修复

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0597](0597-content-fidelity-ledger.md)（保真度清单）、上游迁移收尾
> （[upstream-noname-inventory.md](../migration/upstream-noname-inventory.md)）

## 背景

两件事在本次审计中暴露并修复：

### 1. Aug 7 重打包丢失跨包技能引用（回归）

`content/original/ported/*/<pkg>.npack.json` 与 `extensions/*/<pkg>.npack.json` 在
2026-08-07 01:22–06:06 被重打包（无 git，推断为并发批次或工具重跑）。重打包后，
**130 名武将**的 `character_index[].skills` 数组丢失了跨包技能引用（变空或只剩本包技能）。

例：`diy:diy_wenyang`（文鸯）上游 `skills: ["lvli", "choujue"]`（lvli/choujue 定义在
xianding 包），正确应发布 `skills: [3598772231, 2479638237]`（xianding:lvli/choujue 的
稳定 def_id）；重打包后变成 `skills: []`。受影响 12 包：diy/huicui/key/mobile/offline/
old/onlyOL/refresh/sp/sp2/tw/xianding。

`node scripts/sync-original-content.mjs --verify-only` 因此在
`diy:diy_wenyang: character migration status mismatch` 处失败（catalog 8-04 快照记录为
`ported`，但校验器按 npack 实际发布的空 skills 计算出 `source_synced`）。

### 2. catalog-build 对占位符技能硬抛（与 ADR 0597 冲突）

`buildCatalog` 有一条硬检查：旧 JS 有规则语义（`profileHasRuleSurface` 或非
`metadataOnly`）却被移植为「无行为模块的 DSL 技能」时，直接 `throw`。这在
Aug 4 catalog 之后开始命中（`clan:clanshixi` 等）——因为 legacy-analyzer 重新生成的
capability matrix 给这些技能标记了 `required_surfaces`/`triggers`/`view_as`。

但 ADR 0597 已系统登记 **1109+ 个 `.skill` 占位**（`// 近似：` / `// TODO:` / 空壳
`skill name(id) {}`）为已知保真度缺口，而非构建错误。硬抛导致 catalog 无法重新生成，
sync 全流程阻塞。

## 决策

### D1：跨包技能引用回滚（数据修复，无策略变更）

从 catalog 的 `resolved_skills`（确定性来自上游 JS + `stableDefId`）回填 130 名武将的
正确 `skills` 数组到三处工件：

1. `content/original/ported/<pkg>/characters.json`（roster 源）
2. `content/original/ported/<pkg>/<pkg>.npack.json`（ported npack）
3. `extensions/<pkg>/<pkg>.npack.json` + `extensions/<pkg>/package.npack.json`（deployed）

`character_index.skills` 是 `compute_logic_hash` 的输入（`content-schema/src/package.rs:1041`），
故 36 个 npack 文件（12 包 × 3 副本）的 `manifest.logic_hash` 用 content-schema 的
`compute_logic_hash` 原地重算（`.scratch/rehash_npacks/` 临时 workspace 工具）。
运行时安装时会自行重算 `logic_hash`（`content-runtime/src/store.rs:262`），故不依赖
manifest 内嵌值；重算仅为工件自洽。

### D2：catalog-build 占位符策略对齐 ADR 0597（脚本变更）

`scripts/sync-original-content.mjs` 的 `buildCatalog` 与 `verifyCatalog` 改为：

- 新增 `isEmptyBodySkill(pack, name)`：扫描包内全部 `.skill` 文件（含批处理文件如
  `diy_meta.skill`），用正则 `skill <name>(<id>) [locked|lord_only|conversion|non_gainable]* {}`
  识别空壳（注释先行剥离，多行 `{}` 支持）。
- 旧 JS 有规则语义、npack 中无行为模块、**且** `.skill` 为空壳时 → 记为
  `migration_status: "source_synced"` + `placeholder: true` 标记，**不抛**。
- 旧 JS 有规则语义、npack 中无行为模块、**但** `.skill` 非空壳（真有内容却没产出
  behavior 模块——才 migration 流水管线的真 bug）→ 仍 `throw`。

这把 catalog 从「按 def_id 存在即记 ported」（高估完成度）改为「空壳占位记
source_synced」（如实反映保真度）。`verifyCatalog` 同步对齐，否则会与 buildCatalog
互相矛盾。

## 验证

| 检查 | 结果 |
|---|---|
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅（"synced and verified"） |
| `node scripts/sync-original-content.mjs --verify-only` | **通过** ✅ |
| `cargo test -p engine-core`（全量） | **全绿** ✅ |
| `cargo test -p engine-core --test real_extensions_lock` | **1/1 通过** ✅（31+ 包冷导入 + 开局） |
| `cargo test -p conformance` | **4/4 通过** ✅ |
| `cargo test -p content-runtime` | **全绿** ✅ |
| `cargo test -p engine-core --test original_diy_batch1` | **1/1 通过** ✅ |
| `cargo test -p engine-core --test original_huicui_cross_package_character_closure` | **1/1 通过** ✅ |
| 36 npack 文件 logic_hash 重算 | 36/36 重算 ✅ |
| 130 武将跨包引用回填（三工件一致） | 130/130 ✅（抽样 diy_wenyang=[3598772231,2479638237]） |
| `.npack-store` 12 包重装（meta.json logic_hash 同步） | 12/12 ✅ |

### Golden hash 重钉（logic_hash 变更的预期后果）

`character_index.skills` 进入 `compute_logic_hash`（`content-schema/src/package.rs:1041`），
故 sp 包 logic_hash 因回填 4 名损坏武将（fuwan/jin_guohuai/jin_yanghu/sp_zhangliao）的
跨包引用而变化。3 个 sp 包 golden 测试的硬编码 `PACKAGE_LOGIC_HASH` 常量重钉为新值：

| 测试文件 | 常量 |
|---|---|
| `crates/engine-core/tests/original_sp_dingpan_buzhi.rs` | `PACKAGE_LOGIC_HASH` |
| `crates/engine-core/tests/original_sp_hengzheng_dongzhuo.rs` | `SP_PACKAGE_LOGIC_HASH` |
| `crates/engine-core/tests/original_sp_ziyuan_mizhu.rs` | `PACKAGE_LOGIC_HASH` |

新值 `cd a8 69 b0 …`（sp 包当前 logic_hash）。MODULE_LOGIC_HASH（单技能模块哈希）未变——
本次只改 character_index，不改技能 bytecode。其余 11 包无 golden 测试断言包级 logic_hash。

## 影响：catalog 完成度数字的变化

重建后 catalog 如实反映占位符（此前 8-04 catalog 高估）：

| 指标 | 8-04 catalog（高估） | 8-07 重建（如实） |
|---|---|---|
| migrated_characters | 2245 / 2484（90%） | **903 / 2484（36%）** |
| migrated_skills | 5881 / 5881（100%） | **2873 / 5881（49%）** |
| placeholder skills | 未登记 | **3008**（`placeholder: true`） |

下降不是因为内容丢失，而是因为「def_id 存在但行为为空壳」的技能从 `ported` 重分类为
`source_synced`。这是更诚实的保真度边界，与 ADR 0597 的 1109 项登记互补（0597 按
注释识别近似/TODO；本 ADR 按空壳体识别未实现 behavior）。

## 不动什么

- 不改任何 `.skill` 内容、不改上游 JS、不动 ABI、不动 conformance 向量。
- 不改 `foldedIntoParentSkills`/`reviewedClientOnlySkills`/`reviewedCompositeSkillContainers`
  等已评审例外集。
- 硬抛对「非空壳却缺 behavior 模块」仍保留（migration 流水线真 bug 的守门）。

## 后续

- **Phase 3 已开工（2026-08-07）**：按上游 JS 逐一为占位符补写真实 DSL behavior。首批落地 2 个 sp 技能：
  - `sp:reluanzhan`（乱战，id 4251489811）：`trigger damage owner_is_source` + `owner_is_target` → `add_mark(self, 4251489811, 1)`。上游 `addMark("reluanzhan",1)` on damageEnd/damageSource；近似：标记消费的 `reluanzhan_add/reluanzhan_remove` 子技能（useCard2 增伤/增目标的复杂过滤）另册登记，本定义实现主技能唯一 content 副作用。
  - `sp:resilv`（恃势，id 4240519728）：`trigger damage owner_is_target` → `draw_cards(self, 1)`。上游 `player.draw()` + gaintag on damageEnd；近似：gaintag 标记与 `_restore` 弃置回血子技能需 expansion 支持，本定义实现摸牌核心行为。
  - 落地后 sp 包 modules 105→108、triggers 94→97、logic_hash 重算；3 个 sp golden 测试的 `PACKAGE_LOGIC_HASH` 与 `modules.len()` 同步重钉（108）；catalog migrated_skills 2873→**2875**。`cargo test --workspace` 全绿、sync 全量通过。
  - 新增 `scripts/rebuild-pack.sh <pkg>`：从 `.skill` 源 + characters.json + metadata.json（含既有 asset_manifest/asset_paths）+ assets 目录重建 ported npack，供后续逐批 Phase 3 复用。

- **Phase 3 元数据分类修复（2026-08-07）**：审计占位符时发现 `sync-original-content.mjs` 的 `legacySkillIsMetadataOnly` 有一个**致命 bug**——它调用 `propertyName(item)` 但 `item` 是 `PropertyAssignment`（非 name 节点），sync 的 `propertyName(node)` 访问 `node.name`，所以对每个 item 都返回 `null` → 函数对所有技能恒返回 `false`。修复为 `propertyName(item.name)`（并修正 `intro` 遍历的同类 bug）。
  - 同时新增 `legacySkillHasAuthoritativeRuleSurface(property)`：检测旧 JS 是否含 `trigger/content/filter/mod/viewAs/enable/filterTarget/filterCard/selectCard/selectTarget/cost/onUse/onLose/usable/group/global` 等**权威规则键**。无任何权威键的技能（纯 `ai:`/`init:`-无副作用/`clickableFilter`/`sync`/UI 互动）按项目保真度策略（AI/UI 非权威）DSL 空壳是**正确**的，应记 `ported` 而非占位符。
  - `buildCatalog` 与 `verifyCatalog` 的占位符判定条件加 `&& legacySkillHasAuthoritativeRuleSurface(...)`（buildCatalog 直接用 property；verifyCatalog 用 `authoritativeSurfaceLegacySkills(packs)` 预算集）。
  - 影响：catalog migrated_skills **2875 → 2904**（+29 个纯 AI/UI/元数据技能正确归为 ported，如 `collab:oldianzan` 纯表情投掷、`jsrg:jsrghezhi` 纯 `{zhuSkill,locked}` 元数据）。真正的占位符（旧 JS 有权威规则键但 `.skill` 空壳，如 `clan:clanbaichu`/`clan:clanshixi`）仍记 `source_synced`+`placeholder`。
  - `cargo test --workspace` 全绿、sync 全量通过。

- **Phase 3 的结构性约束（决定可补全的速度）**：审计 sp/全部包的占位符发现，**绝大多数占位符的阻塞不在 DSL 编写，而在引擎原语缺口**。典型例：
  - `sp:mansi` 需「南蛮入侵造成的伤害」过滤（card-name-scoped damage filter）——无原语。
  - `tw:twyanzuo` 需「仁贤额外阶段检测 + 蜀势力限定」过滤——无原语。
  - `sp:olpingzhong` 需「手牌为空的角色数」查询——无原语。
  - `xianding:dckuichi`/`tw:twlihuo3` 需回合内摸牌/伤害历史计数查询——无原语。
  - 仅当 effect 是固定值（draw/recover/loseHp/addMark 的常量）且 trigger 落在既有命名过滤（`owner_is_target`/`owner_is_source`/`owner_is_source_or_target` 等）上时，才能纯 DSL 补全（如 reluanzhan/resilv）。这类「纯 DSL 可补」的占位符是少数；多数需先在 `crates/engine-core` 扩 filter/query 原语、对应 trigger 过滤名、DSL 词法，再补 DSL。每补一个原语可解锁一批占位符。
  - 结论：Phase 3 的完整闭环（3008 个占位符全部升回 `ported`）是按原语族分批的长期工作，非单次可完成。本 ADR 记录的 catalog placeholder 机制 + `rebuild-pack.sh` + 逐技能 ADR 为后续批次提供可复现管线。

- `.npack-store` 预构建快照需用 content-cli 对修复/重建包重新安装以同步 `meta.json.logic_hash`（运行时安装会自动重算，此步仅为工件自洽；sp 包已重装）。
