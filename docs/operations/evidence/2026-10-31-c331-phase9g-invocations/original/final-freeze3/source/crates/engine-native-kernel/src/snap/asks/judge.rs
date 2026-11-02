use super::*;
pub(super) fn active_judge_ask(
    owner: &PlayerId,
    current: &Option<Box<JudgementProgressSnap>>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let c = current.as_deref()?;
    if !c.finalized && c.judge_held {
        if let Some(cards) = &c.gaiming_cards {
            return Some(ask(
                *owner,
                "gaiming_pick",
                ChoiceKind::SelectCards,
                hand_card_options(cards),
                1,
                1,
                true,
            ));
        }
        if let (Some(holder), Some(other)) = (c.haotian_holder, c.haotian_other) {
            return Some(ask(
                holder,
                "haotianta_pick",
                ChoiceKind::SelectCards,
                vec![ChoiceOption::Card(c.judge_card), ChoiceOption::Card(other)],
                1,
                1,
                false,
            ));
        }
    }
    if c.in_rewrite {
        if !c.asked {
            return None;
        }
        let rewriter = *c.rewriters.first()?;
        // 黑色手牌 + 黑色装备；鬼才持有者全部手牌可打
        let gui_cai = catalog
            .skill_by_name(GUI_CAI)
            .map(|d| core.has_skill(rewriter, d))
            .unwrap_or(false);
        let any_he = [crate::vocabulary::names::BOSS_TIANDAO, "gtiandao"]
            .iter()
            .any(|name| {
                catalog
                    .skill_by_name(name)
                    .is_some_and(|d| core.has_skill(rewriter, d))
            });
        let is_black = |id: CardId| {
            matches!(
                catalog.cards[id as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        };
        let mut options = vec![ChoiceOption::Pass];
        for c in core.player(rewriter).hand.iter().copied() {
            if any_he || gui_cai || is_black(c) {
                options.push(ChoiceOption::Card(c));
            }
        }
        for c in core.player(rewriter).equips.iter().copied() {
            if any_he || is_black(c) {
                options.push(ChoiceOption::Card(c));
            }
        }
        return Some(ask(
            rewriter,
            "gui_dao_rewrite",
            ChoiceKind::RespondCard,
            options,
            1,
            1,
            false,
        ));
    }
    // 天妒询问（判定确定后、获得前、判定牌仍由机器持有）
    if c.finalized && !c.tian_du.is_empty() && c.asked && !c.gained && c.judge_held {
        return Some(confirm_ask(c.tian_du[0], "tian_du_gain"));
    }
    None
}
