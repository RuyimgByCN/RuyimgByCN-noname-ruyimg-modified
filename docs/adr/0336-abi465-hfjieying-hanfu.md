# ADR 0336：ABI 465 选人授技原语 + SP2 韩馥

本批为 SP2 包韩馥（`hanfu`）的「界营」（`hfjieying`，2801808452）完整移植，
需要新的复合原语 `choose_other_add_skill(player, skill_def)`（ABI 465）：选择一名
其他角色并授予指定技能。

## 原语设计（ABI 465）

- DSL 表面：`choose_other_add_skill(player, skill_def)`（skill_def 非零）；编译为
  `effect_tag::RESOLVE_HFJIEYING`（tag 382）。
- 引擎：`EffectRequest::ResolveHfjieying { player, target, grant_def, skill }`；
  选择结算走 `ContinuationKind::Custom` + `RESOLVE_HFJIEYING_CUSTOM_CHOICE`
  （0xDCF3_4650）：选择其他存活角色后 `AddSkill` 授予。

## 技能移植

「界营」（`hfjieying`，2801808452）：
- `trigger phase_end owner_is_target optional`：`choose_other_add_skill(self, 4111610111)`。
- 「界营·督战」（`hfjieying2`，4111610111，新派生技能）：
  `card_used owner_source_sha` → `choose_add_current_sha_target(self, event_card)`
  加目标；`phase_end owner_is_target` 移除。

## 文档化近似

旧 JS 的界营临时技能含封牌副作用（storage 从未置位，冗余）与锦囊加目标；DSL 以
`choose_add_current_sha_target` 表达【杀】加目标（引擎加目标管线为【杀】专用，
锦囊部分文档化近似舍弃）。

## 打包与验证

- `hfjieying.skill`（含 `hfjieying2`）加入 `content/original/ported/sp2/`（README
  显式列表补充）；韩馥（3334856927，男/群/4 体力，技能 [hfjieying 2801808452,
  weipo 3483198633]）加入 sp2 `characters.json`；catalog 标记 `hfjieying` 为
  `ported` 并新增 `hfjieying2`。
- sp2 npack 重建（ABI 465，37 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_hfjieying.rs` 全绿：结束阶段授技 + 获授者使用【杀】时
  弹出加目标选择。
- 旧 JavaScript 不进入 npack 或运行时。
