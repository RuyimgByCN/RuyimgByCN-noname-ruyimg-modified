use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_sha_ask(
    target: PlayerId,
    source: PlayerId,
    asked: bool,
    tieji_red: Option<bool>,
    liuli_stage: u8,
    hujia_holder: Option<PlayerId>,
    asking_qinglong: bool,
    asking_fuchou: bool,
    fuchou_card: Option<u32>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if asking_fuchou {
        let mut options: Vec<ChoiceOption> = core
            .player(target)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        options.extend(
            core.player(target)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card),
        );
        return Some(ask(
            target,
            "wudian_fuchou_cancel",
            ChoiceKind::SelectCards,
            options,
            1,
            1,
            true,
        ));
    }
    if asking_qinglong {
        if fuchou_card.filter(|c| *c != u32::MAX).is_some() {
            let hand_shas: Vec<ChoiceOption> = core
                .player(source)
                .hand
                .iter()
                .filter(|c| catalog.card_kind(**c).is_sha_like())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            return Some(ask(
                source,
                crate::vocabulary::host::WUDIAN_YOUDISHENRU_SHA,
                ChoiceKind::SelectCards,
                hand_shas,
                1,
                1,
                true,
            ));
        }
        let hand_shas: Vec<ChoiceOption> = core
            .player(source)
            .hand
            .iter()
            .filter(|c| {
                matches!(
                    catalog.card_kind(**c),
                    CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                )
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        return Some(ask(
            source,
            "qinglong_replay_sha",
            ChoiceKind::SelectCards,
            hand_shas,
            0,
            1,
            true,
        ));
    }
    if liuli_stage == 1 {
        let mut options: Vec<ChoiceOption> = core
            .player(target)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        options.extend(
            core.player(target)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card),
        );
        return Some(ask(
            target,
            "liuli_discard_choose",
            ChoiceKind::SelectCards,
            options,
            0,
            1,
            true,
        ));
    }
    if liuli_stage == 2 {
        let candidates: Vec<ChoiceOption> = core
            .alive_players()
            .filter(|p| {
                *p != target
                    && *p != source
                    && {
                        let kc = catalog
                            .skill_by_name(crate::vocabulary::names::KONG_CHENG)
                            .map(|d| core.has_skill(*p, d))
                            .unwrap_or(false);
                        !(kc && core.player(*p).hand.is_empty())
                    }
                    && core.computed_distance(catalog, target, *p)
                        <= core.attack_range(catalog, target)
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some(ask(
            target,
            "liuli_transfer",
            ChoiceKind::SelectPlayers,
            candidates,
            1,
            1,
            false,
        ));
    }
    if let Some(h) = hujia_holder {
        let qingguo_h = catalog
            .skill_by_name(QING_GUO)
            .map(|d| core.has_skill(h, d))
            .unwrap_or(false);
        let options: Vec<ChoiceOption> = core
            .player(h)
            .hand
            .iter()
            .copied()
            .filter(|c| {
                let kind = catalog.card_kind(*c);
                kind == CardKind::Shan
                    || (qingguo_h
                        && matches!(
                            catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        ))
            })
            .map(ChoiceOption::Card)
            .collect();
        return Some(ask(
            h,
            "hujia_respond",
            ChoiceKind::SelectCards,
            options,
            0,
            1,
            false,
        ));
    }
    if asked {
        let tieji_blocks = tieji_red == Some(true)
            || core.vm_aux.flags.get(&(target, "mbcmfangzhu_ban".to_string())).copied().unwrap_or(0) > 0;
        let longdan = catalog
            .skill_by_name(LONG_DAN)
            .map(|d| core.has_skill(target, d))
            .unwrap_or(false);
        let qingguo = catalog
            .skill_by_name(QING_GUO)
            .map(|d| core.has_skill(target, d))
            .unwrap_or(false);
        let mut options: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
        for c in core.player(target).hand.iter().copied() {
            let kind = catalog.card_kind(c);
            if !tieji_blocks && (kind == CardKind::Shan || (longdan && kind == CardKind::Sha)) {
                options.push(ChoiceOption::Card(c));
            }
        }
        for c in core.player(target).hand.iter().copied() {
            if catalog.card_kind(c) == CardKind::Youdishenru {
                options.push(ChoiceOption::Card(c));
                break;
            }
        }
        if qingguo && !tieji_blocks {
            for c in core.player(target).hand.iter().copied() {
                let kind = catalog.card_kind(c);
                let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                let black = matches!(
                    catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if black && !already {
                    options.push(ChoiceOption::Card(c));
                }
            }
        }
        if !tieji_blocks {
            append_vm_shan_options(core, catalog, target, &mut options);
        }
        return Some(ask(
            target,
            "respond_shan",
            ChoiceKind::RespondCard,
            options,
            1,
            1,
            false,
        ));
    }
    None
}
