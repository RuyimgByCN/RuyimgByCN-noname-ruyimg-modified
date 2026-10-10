# ADR 0393：SHIJI SP 张温闭环（送枢）

本批发布 SHIJI SP 张温（`sp_zhangwen`，3787046384）。武将由「割卜」（`gebo`，已有）
与「送枢」（`spsongshu`）组成，送枢由 ABI 404 `resolve_spsongshu` 权威结算；旧
JavaScript 仅作为角色数据语义来源，不进入运行时。

## 决策

- 复用 ABI 404 既有原语 `resolve_spsongshu(player, phase_player, renku_skill_def,
  block_skill_def)`：其他角色摸牌阶段开始时，若其体力值大于拥有者且仁库非空，
  将其摸牌数置零，令其从仁库（割卜技能牌堆）选至多 `min(5, 拥有者体力, 仁库牌数)`
  张获得，并临时授予「送枢·禁」（`spsongshu_block`，不能指定其他角色为目标）。
- 辅助技能 `spsongshu_block`（2373945789）以普通技能定义随包发布，作为 effect
  参数传递，不挂武将。
- 武将数据对齐旧 JS：男、吴、3 点体力上限。
- 将角色加入 SHIJI 包并同步 `extensions/shiji/`。

## 验证

- `content-cli pack` 全量 34 个 .skill 编译通过，npack 含 44 技能 / 9 武将，
  `rule_vm_abi_version` 502 不变。
- 武将-技能引用完整性校验：`sp_zhangwen` → `gebo` + `spsongshu`，无悬空 ID。
- 全工作区 `cargo test` 门禁（golden 向量不受影响：引擎侧无新 DSL 语义
  变更，仅有内容新增；若有漂移则按 golden re-bless 流程重钉）。
