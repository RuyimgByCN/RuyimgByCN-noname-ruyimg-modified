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
        let providers = crate::kernel::judge_rewrite_providers(
            core,
            catalog,
            rewriter,
            &c.used_rewrite_providers,
        );
        let mut options = vec![ChoiceOption::Pass];
        if let Some(provider) = c.rewrite_provider {
            options.extend(
                crate::kernel::judge_provider_cards(core, catalog, rewriter, provider)
                    .into_iter()
                    .map(ChoiceOption::Card),
            );
        } else {
            options.extend(providers.into_iter().map(ChoiceOption::Skill));
        }
        return Some(ask(
            rewriter,
            if c.rewrite_provider.is_none() {
                "judge_rewrite_provider"
            } else {
                "gui_dao_rewrite"
            },
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
