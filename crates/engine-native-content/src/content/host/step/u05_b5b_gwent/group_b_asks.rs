//! U05-5b Batch B 询问 `ChoiceSpec` 构建与快照恢复共享模块。

use super::*;

pub(super) const ASK_SHUANGXI_TARGETS: u32 = 25;
pub(super) const ASK_GWFENGSHI_CHOICE: u32 = 26;
pub(super) const ASK_GWQINWU_TARGET: u32 = 27;
pub(super) const ASK_HUANSHU_CARD: u32 = 28;
pub(super) const ASK_GWJIEYIN_PICK: u32 = 29;
pub(super) const ASK_GWJIEYIN_TARGET: u32 = 30;
pub(super) const ASK_ZHENGJUN_PICK: u32 = 31;
pub(super) const ASK_BOLANG_PICK: u32 = 32;
pub(super) const ASK_BOLANG_GAIN: u32 = 33;
pub(super) const ASK_GWJUSHI_TARGET: u32 = 34;
pub(super) const ASK_GWFENGCHI_PICK: u32 = 35;

pub(super) fn jieyin_mask_options(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    let used = flag(core, owner, "gw_jieyin_mask");
    (0u8..3)
        .filter(|i| (used & (1u32 << i)) == 0)
        .map(ChoiceOption::Tag)
        .collect()
}

pub(super) fn bolang_reclaimable(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    core.discard
        .iter()
        .copied()
        .filter(|c| flag(core, owner, &format!("gw_bolang_tag:{c}")) > 0)
        .map(ChoiceOption::Card)
        .collect()
}

pub(super) fn jushi_target_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core.computed_distance(catalog, owner, p) <= 1
                && !hand_and_equips(core, p).is_empty()
        })
        .map(ChoiceOption::Player)
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_group_b(
    ask_id: u32,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match ask_id {
        ASK_SHUANGXI_TARGETS => {
            let damaged = flag(core, owner, "gw_shuangxi_turn_damaged") > 0;
            let opts = eligible_bronze_targets(core, catalog, owner, CardKind::Ciguhanshuang);
            let max = if damaged { opts.len().min(3) } else { 1 };
            Some(ask_spec(
                owner,
                "shuangxi_targets",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max,
                true,
            ))
        }
        ASK_GWFENGSHI_CHOICE => Some(ask_spec(
            owner,
            "gwfengshi_choice",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        )),
        ASK_GWQINWU_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwqinwu_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_HUANSHU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "huanshu_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWJIEYIN_PICK => Some(ask_spec(
            owner,
            "gwjieyin_pick",
            ChoiceKind::PickTag,
            jieyin_mask_options(core, owner),
            1,
            1,
            true,
        )),
        ASK_GWJIEYIN_TARGET => {
            let kind = match card {
                Some(1) => CardKind::Yanziyaoshui,
                Some(2) => CardKind::Kunenfayin,
                _ => CardKind::Wenyi,
            };
            let opts = eligible_bronze_targets(core, catalog, owner, kind);
            Some(ask_spec(
                owner,
                "gwjieyin_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_ZHENGJUN_PICK => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            Some(ask_spec(
                owner,
                "zhengjun_pick",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_BOLANG_PICK => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            let max = opts.len().min(3);
            Some(ask_spec(
                owner,
                "bolang_pick",
                ChoiceKind::SelectCards,
                opts,
                1,
                max,
                true,
            ))
        }
        ASK_BOLANG_GAIN => {
            let opts = bolang_reclaimable(core, owner);
            Some(ask_spec(
                owner,
                "bolang_gain",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWJUSHI_TARGET => {
            let opts = jushi_target_options(core, catalog, owner);
            Some(ask_spec(
                owner,
                "gwjushi_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWFENGCHI_PICK => {
            let n = flag(core, owner, "gw_fengchi_n");
            let mut opts = Vec::new();
            for i in 0..n {
                let dp1 = flag(core, owner, &format!("gw_fengchi_opt:{i}"));
                if dp1 > 0 {
                    opts.push(ChoiceOption::Skill((dp1 - 1) as SkillDefId));
                }
            }
            Some(ask_spec(
                owner,
                "gwfengchi_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        _ => None,
    }
}
