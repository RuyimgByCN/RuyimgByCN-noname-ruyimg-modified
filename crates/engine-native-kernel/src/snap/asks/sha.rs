use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_sha_ask(
    target: PlayerId,
    source: PlayerId,
    asked: bool,
    tieji_red: Option<bool>,
    liuli_stage: u8,
    hujia_holder: Option<PlayerId>,
    asking_qilin: bool,
    asking_qinglong: bool,
    asking_fuchou: bool,
    asking_fengxueren: bool,
    _fuchou_card: Option<u32>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if asking_fengxueren {
        return Some(ask(
            source,
            "fengxueren_flip_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ));
    }
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
    if liuli_stage == 3 {
        let opts = crate::kernel::ShaResolveMachine::youdishenru_use_options(core, catalog, target);
        if opts.is_empty() {
            return None;
        }
        return Some(ask(
            target,
            crate::vocabulary::host::WUDIAN_YOUDISHENRU_USE,
            ChoiceKind::SelectCards,
            opts,
            1,
            1,
            true,
        ));
    }
    if asking_qinglong {
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
                    && core.in_attack_range(catalog, target, *p)
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
                !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(core, catalog, h, *c)
                    && (kind == CardKind::Shan
                        || (qingguo_h
                            && matches!(
                                catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )))
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
    if asking_qilin {
        let options: Vec<ChoiceOption> = core
            .player(target)
            .equips
            .iter()
            .filter(|c| {
                matches!(
                    catalog.card_kind(**c),
                    CardKind::HorsePlus | CardKind::HorseMinus
                )
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        return Some(ask(
            source,
            "qilin_discard_horse",
            ChoiceKind::SelectCards,
            options,
            0,
            1,
            true,
        ));
    }
    if asked {
        let tieji_blocks = tieji_red == Some(true)
            || core.vm_aux.flags.get(&(target, "mbcmfangzhu_ban".to_string())).copied().unwrap_or(0) > 0;
        let sealed = catalog.is_tieji_sealed(core, target);
        let hand_locked =
            crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, target);
        let longdan = !sealed
            && !catalog.is_rechanyuan_sealed_name(core, target, LONG_DAN)
            && catalog
                .skill_by_name(LONG_DAN)
                .map(|d| core.has_skill(target, d))
                .unwrap_or(false);
        let qingguo = !sealed
            && !catalog.is_rechanyuan_sealed_name(core, target, QING_GUO)
            && catalog
                .skill_by_name(QING_GUO)
                .map(|d| core.has_skill(target, d))
                .unwrap_or(false);
        let target_sentinel = catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
            .is_some_and(|def| core.has_skill(target, def) && core.player(target).baolei_mode == 1);
        let target_can_play = !tieji_blocks
            && !target_sentinel
            && !catalog.blocks_all_cards(core, target);
        let mut options: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
        if !hand_locked {
            for c in core.player(target).hand.iter().copied() {
                let kind = catalog.card_kind(c);
                if target_can_play && (kind == CardKind::Shan || (longdan && kind == CardKind::Sha))
                {
                    options.push(ChoiceOption::Card(c));
                }
            }
        }
        if qingguo && target_can_play && !hand_locked {
            for c in core.player(target).hand.iter().copied() {
                let kind = catalog.card_kind(c);
                let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                let black = matches!(
                    catalog.effective_card_suit(core, target, c),
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if black && !already {
                    options.push(ChoiceOption::Card(c));
                }
            }
        }
        if target_can_play {
            append_vm_shan_options(core, catalog, target, &mut options);
        }
        options.retain(|o| {
            !matches!(o, ChoiceOption::Card(c)
                if crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
                    core, catalog, target, *c
                ))
        });
        let chihe_block_mask = core
            .vm_aux
            .flags
            .get(&(target, "l10_scschihe_block_suits".to_string()))
            .copied()
            .unwrap_or(0);
        if chihe_block_mask != 0 {
            options.retain(|o| match o {
                ChoiceOption::Card(c) => {
                    let bit = match catalog.effective_card_suit(core, target, *c) {
                        crate::catalog::Suit::Spade => 1 << 0,
                        crate::catalog::Suit::Heart => 1 << 1,
                        crate::catalog::Suit::Club => 1 << 2,
                        crate::catalog::Suit::Diamond => 1 << 3,
                    };
                    (chihe_block_mask & bit) == 0
                }
                _ => true,
            });
        }
        if target_can_play {
            for rdef in crate::kernel::host_response_skills(core, catalog, target, 1) {
                options.push(ChoiceOption::Skill(rdef));
            }
            for opt in (catalog.hooks.respond_skill_options)(
                core,
                catalog,
                target,
                CardKind::Shan,
                true,
            ) {
                if !options.contains(&opt) {
                    options.push(opt);
                }
            }
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
