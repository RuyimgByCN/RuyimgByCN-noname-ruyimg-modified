//! U05-5b Batch D 询问 `ChoiceSpec` 构建与快照恢复共享模块。

use super::*;

pub(super) const ASK_NUHOU_COST: u32 = 57;
pub(super) const ASK_SHEWU_COST: u32 = 58;
pub(super) const ASK_GWZHANJIANG_COST: u32 = 59;
pub(super) const ASK_GWZHANJIANG_SHA: u32 = 60;
pub(super) const ASK_GWCHUANXIN_CONFIRM: u32 = 61;
pub(super) const ASK_FENGJIAN_TARGET: u32 = 62;
pub(super) const ASK_HUANDIE_TARGETS: u32 = 63;
pub(super) const ASK_HUANDIE_DISCARD_PICK: u32 = 64;
pub(super) const ASK_XUEZZHOU_MODE: u32 = 65;
pub(super) const ASK_FAYIN_COST: u32 = 66;
pub(super) const ASK_GWBAQUAN_TARGET: u32 = 67;
pub(super) const ASK_GWBAQUAN_RETURN: u32 = 68;
pub(super) const ASK_LANQUAN_PICK: u32 = 69;
pub(super) const ASK_LANQUAN_TARGET: u32 = 70;
pub(super) const ASK_HUANDIE_CONFIRM: u32 = 71;

pub(super) fn lanquan_playable_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.deck
        .iter()
        .rev()
        .take(6)
        .copied()
        .filter(|&c| b5b_card_can_use(core, catalog, owner, c, true))
        .map(ChoiceOption::Card)
        .collect()
}

pub(super) fn hand_sha_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| is_sha_or_wuyao(catalog, c))
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_group_d(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match ask_id {
        ASK_NUHOU_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "nuhou_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_SHEWU_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len().min(3);
            Some(ask_spec(
                owner,
                "shewu_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                max,
                true,
            ))
        }
        ASK_GWZHANJIANG_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwzhanjiang_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWZHANJIANG_SHA => {
            let shooter = *targets.first()?;
            let opts: Vec<ChoiceOption> = hand_sha_cards(core, catalog, shooter)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                shooter,
                "gwzhanjiang_sha",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWCHUANXIN_CONFIRM => Some(ask_spec(
            owner,
            "gwchuanxin_confirm",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0)],
            1,
            1,
            true,
        )),
        ASK_FENGJIAN_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && core.player(p).alive
                        && !targets.contains(&p)
                        && !crate::kernel::sha_user_or_target_blocked(core, catalog, owner, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "fengjian_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_HUANDIE_CONFIRM => Some(ask_spec(
            owner,
            "trigger_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        ASK_HUANDIE_TARGETS => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len();
            Some(ask_spec(
                owner,
                "huandie_targets",
                ChoiceKind::SelectPlayers,
                opts,
                0,
                max,
                false,
            ))
        }
        ASK_HUANDIE_DISCARD_PICK => {
            let victim = *targets.first()?;
            let opts: Vec<ChoiceOption> = core
                .player(victim)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, victim, c)
                })
                .map(ChoiceOption::Card)
                .collect();
            let need = opts.len().min(2);
            Some(ask_spec(
                victim,
                "huandie_discard_pick",
                ChoiceKind::SelectCards,
                opts,
                need,
                need,
                false,
            ))
        }
        ASK_XUEZZHOU_MODE => Some(ask_spec(
            owner,
            "xuezhou_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        )),
        ASK_FAYIN_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "fayin_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWBAQUAN_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwbaquan_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWBAQUAN_RETURN => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let need = cards.len().min(opts.len());
            Some(ask_spec(
                owner,
                "gwbaquan_return",
                ChoiceKind::SelectCards,
                opts,
                need,
                need,
                false,
            ))
        }
        ASK_LANQUAN_PICK => Some(ask_spec(
            owner,
            "lanquan_pick",
            ChoiceKind::SelectCards,
            lanquan_playable_options(core, catalog, owner),
            1,
            1,
            true,
        )),
        ASK_LANQUAN_TARGET => {
            let c = card?;
            if let Some(holder) = target {
                if catalog.card_kind(c) == CardKind::Jiedao {
                    let victims = b5b_jiedao_victim_options(core, catalog, owner, holder);
                    return Some(ask_spec(
                        owner,
                        "lanquan_target",
                        ChoiceKind::SelectPlayers,
                        victims,
                        1,
                        1,
                        true,
                    ));
                }
            }
            let (opts, min_t, max_t) = b5b_card_targets(core, catalog, owner, c)?;
            Some(ask_spec(
                owner,
                "lanquan_target",
                ChoiceKind::SelectPlayers,
                opts,
                min_t,
                max_t,
                true,
            ))
        }
        _ => None,
    }
}
