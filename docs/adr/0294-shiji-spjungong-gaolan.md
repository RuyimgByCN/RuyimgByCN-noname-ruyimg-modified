# ADR 0294：SHIJI 高览「峻攻」的弃牌/失体力与命中禁用（ABI 400）

ABI 400 增加
`spjungong_discard_or_lose_then_sha(player, target, self_skill)` 与配套触发过滤器
`owner_source_sha_damage_recorded_by_skill` 及 `ban_active_until_turn_end(player, self_skill)`。

效果在发动时动态读取 `use_count + 1` 作为成本点数 N；弹出 SelectCards 屏障（选 0..N 张牌），
选择 0 张牌则通过普通伤害管线对自己造成 N 点无来源普通伤害，否则强制弃置所选 N 张手牌/装备牌。
卡牌移动或扣体力完成后对目标使用无距离限制的虚拟【杀】。

若该次【杀】造成伤害，配套触发读取权威 Sha 结算记录中关联的技能实例，把 `active_turn_banned`
置位，使本阶段内不能再发动【峻攻】；未造成伤害（被【闪】抵消或伤害被防止）则不触发禁用。

高览（`sp_gaolan`）与已移植的「登历」（`spdengli`）组成完整 4 体力群雄武将闭包。
旧 JS 的 UI 估值提示与历史数组折叠进权威引擎状态；旧 JavaScript 不进入 npack 或运行时。
