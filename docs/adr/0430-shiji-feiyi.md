# ADR 0430：SHIJI 费祎闭环（生息 / 谏喻，ABI 425）

本批发布 SHIJI 费祎（`feiyi`，3307160204，男/蜀/3 血）。技能「生息」
（`mjshengxi`）与「谏喻」（`fyjianyu`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **mjshengxi（生息）**：纯 DSL 复用 TW 生息管线——`phase_end
  owner_alive_phase_end_used_card_no_damage` 可选触发，`resolve_twfeiyi(self, 1)`
  （智囊选择：无中生有/过河拆桥/无懈可击，或摸一张）。
- **ABI 425 `resolve_fyjianyu(player, a, b)`** + 新过滤器
  `owner_alive_card_targeted_marked_peer_<mark>`（前缀参数化，喻标记 def =
  fyjianyu 技能 ID）：
  - mode 0（active，选 2 名其他角色）：授予喻标记（`AddMark`）。
  - mode 1（`card_targeted` 触发）：出牌阶段角色（有喻标记）用牌指定另一名
    喻标记角色为目标时，目标摸 1（`DrawCards`）。
  - 每轮一次/每角色至多一次的旧 JS 约束以 DSL `uses_per_turn 1` 近似，文档化。

## 验证

- 新增 `original_shiji_feiyi.rs` 2 个行为测试：谏喻授予 2 目标标记、
  喻标记来源指定喻目标时目标摸 1。
- 打包：69 技能 / 20 武将，`rule_vm_abi_version` 502。
- 顺带清理并行会话（Codex）pottaishici 的 mod.rs 重复 re-export。
