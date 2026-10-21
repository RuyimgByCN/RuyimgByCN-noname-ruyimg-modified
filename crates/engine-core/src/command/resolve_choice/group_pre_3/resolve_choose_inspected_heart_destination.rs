//! Command operations for `resolve_choose_inspected_heart_destination` and related helpers.

use super::*;

pub(crate) fn resolve_choose_inspected_heart_destination(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, card_raw, _skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad inspected-heart destination payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let card = CardId::new(*card_raw);
    let [destination] = selections else {
        return Err(EffectError::Illegal(
            "inspected-heart destination requires one option".into(),
        ));
    };
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    if cancelled
        || observer != player
        || *destination > 1
        || !tx.state().cards.get(card).is_some_and(|state| {
            hand == Some(state.zone)
                && crate::query::effective_card_suit(tx.state(), target, state)
                    == engine_api::Suit::Heart
        })
    {
        return Err(EffectError::Illegal(
            "inspected-heart destination is no longer legal".into(),
        ));
    }
    if *destination == 0 {
        return run_effects(
            tx,
            vec![EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards: vec![card],
                scope: rule_api::TargetCardScope::Hand,
            }],
        )
        .map(|_| ());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
            EffectRequest::MoveCards {
                cards: vec![card],
                to: draw,
                from: hand,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_mode(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Yeyan mode payload".into()));
    };
    let [mode] = selections else {
        return Err(EffectError::Illegal(
            "Yeyan mode requires exactly one option".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || *mode > 1 {
        return Err(EffectError::Illegal("invalid Yeyan mode choice".into()));
    }
    if *mode == 1 {
        let suits: std::collections::BTreeSet<_> =
            crate::query::yeyan_hand_cards(tx.state(), player)
                .into_iter()
                .filter_map(|card| tx.state().cards.get(card))
                .map(|card| {
                    crate::command::judge::suit_tag(crate::query::effective_card_suit(
                        tx.state(),
                        player,
                        card,
                    ))
                })
                .collect();
        if suits.len() < 4 {
            return Err(EffectError::Illegal(
                "big Yeyan no longer has four distinct suits".into(),
            ));
        }
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: *mode as u8,
            stage: 1,
            index: 0,
            targets: Vec::new(),
            data: 0,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_big_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal("bad big-Yeyan card payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let count = payload[2] as usize;
    if cancelled || observer != player || payload.len() != 3 + count || selections.len() != 4 {
        return Err(EffectError::Illegal(
            "big Yeyan requires exactly four cards".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[3..].iter().copied().collect();
    let current: std::collections::BTreeSet<_> = crate::query::yeyan_hand_cards(tx.state(), player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let mut cards = std::collections::BTreeSet::new();
    let suits: std::collections::BTreeSet<_> = selections
        .iter()
        .filter_map(|raw| {
            let card = tx.state().cards.get(CardId::new(*raw))?;
            (original.contains(raw) && current.contains(raw) && cards.insert(*raw)).then(|| {
                crate::command::judge::suit_tag(crate::query::effective_card_suit(
                    tx.state(),
                    player,
                    card,
                ))
            })
        })
        .collect();
    if cards.len() != 4 || suits.len() != 4 {
        return Err(EffectError::Illegal(
            "big Yeyan cards must have four distinct effective suits".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 2,
            index: 0,
            targets: Vec::new(),
            data: 0,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: selections.iter().copied().map(CardId::new).collect(),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mode, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Yeyan target payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let max = if *mode == 0 { 3 } else { 2 };
    let mut unique = std::collections::BTreeSet::new();
    let targets: Vec<_> = selections
        .iter()
        .copied()
        .map(PlayerId::new)
        .filter(|target| unique.insert(target.raw()))
        .collect();
    if cancelled
        || observer != player
        || !matches!(*mode, 0 | 1)
        || targets.len() != selections.len()
        || !(1..=max).contains(&targets.len())
        || targets.iter().any(|target| {
            !tx.state()
                .players
                .get(*target)
                .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Yeyan targets are no longer legal".into(),
        ));
    }
    let skill = engine_api::SkillId::new(*skill_raw);
    if *mode == 0 {
        return run_effects(
            tx,
            vec![EffectRequest::ResolveYeyan {
                player,
                mode: 0,
                stage: 4,
                index: 0,
                targets,
                data: 0,
                skill,
            }],
        )
        .map(|_| ());
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 3,
            index: 0,
            targets,
            data: 0,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 3,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_single_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, target_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad single-target Yeyan damage payload".into(),
        ));
    };
    let [amount] = selections else {
        return Err(EffectError::Illegal(
            "single-target Yeyan requires one amount".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || !matches!(*amount, 2 | 3) {
        return Err(EffectError::Illegal(
            "invalid single-target Yeyan damage".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 4,
            index: 0,
            targets: vec![PlayerId::new(*target_raw)],
            data: *amount,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_double_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, first, second] = payload else {
        return Err(EffectError::Illegal(
            "bad two-target Yeyan allocation payload".into(),
        ));
    };
    let [boosted] = selections else {
        return Err(EffectError::Illegal(
            "two-target Yeyan requires one boosted target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || ![*first, *second].contains(boosted) {
        return Err(EffectError::Illegal(
            "invalid two-target Yeyan allocation".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 4,
            index: 0,
            targets: vec![PlayerId::new(*first), PlayerId::new(*second)],
            data: *boosted,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_all_lose_hp_or_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Qinyin choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal("Qinyin observer mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qinyin choice must be empty".into(),
            ))
        };
    }
    let [choice] = selections else {
        return Err(EffectError::Illegal(
            "Qinyin requires one lose-or-recover option".into(),
        ));
    };
    let skill = engine_api::SkillId::new(*skill_raw);
    if *choice > 1
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || tx
            .state()
            .turn
            .flag(crate::state::turn_flag::DISCARD_PHASE_DISCARDED)
            < 2
    {
        return Err(EffectError::Illegal("Qinyin context changed".into()));
    }
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|target| {
            if *choice == 0 {
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }
            } else {
                EffectRequest::RecoverHp {
                    source: player,
                    player: target,
                    amount: 1,
                }
            }
        })
        .collect();
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_discard_weapon_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, stage] = payload else {
        return Err(EffectError::Illegal(
            "bad discard-weapon-or-lose-HP payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "discard-weapon-or-lose-HP context changed".into(),
        ));
    }
    match (*stage, selections) {
        (0, [0])
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive && participant.hp >= 1) =>
        {
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        (0, [1]) => {
            let weapons = crate::effect::discardable_weapon_cards(tx, player);
            if weapons.is_empty() {
                return Err(EffectError::Illegal("no discardable weapon remains".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                weapons.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDiscardWeaponOrLoseHp,
                vec![player.raw(), skill.raw(), 1],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::effect::discardable_weapon_cards(tx, player).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is not a discardable weapon".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid discard-weapon-or-lose-HP selection".into(),
        )),
    }
}

/// 弃一张手牌/装备牌或失去体力（庸肆 xinyongsi2）的选择恢复。
/// payload: `[player, skill, lose_amount, stage]`；
/// stage 0 = 二选一（0=失去体力, 1=选牌），stage 1 = 确认弃置的牌。
pub(crate) fn resolve_choose_discard_one_he_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, lose_amount, stage] = payload else {
        return Err(EffectError::Illegal(
            "bad discard-one-or-lose-HP payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "discard-one-or-lose-HP context changed".into(),
        ));
    }
    match (*stage, selections) {
        (0, [0])
            if tx.state().players.get(player).is_some_and(|participant| {
                participant.alive && participant.hp >= *lose_amount as i32
            }) =>
        {
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: *lose_amount,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        (0, [1]) => {
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if discardable.is_empty() {
                return Err(EffectError::Illegal(
                    "no discardable hand/equip card remains".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                discardable.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDiscardOneHeOrLoseHp,
                vec![player.raw(), skill.raw(), *lose_amount, 1],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is not discardable hand/equip".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid discard-one-or-lose-HP selection".into(),
        )),
    }
}

/// 随势（suishi2）来源确认恢复：payload `[owner, source, skill, amount]`。
/// 来源确认（selections=[1]）后 owner 失去 amount 点体力；取消则无事发生。
pub(crate) fn resolve_choose_source_confirm_owner_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [owner_raw, source_raw, skill_raw, amount] = payload else {
        return Err(EffectError::Illegal(
            "bad source-confirm-owner-lose-HP payload".into(),
        ));
    };
    let owner = PlayerId::new(*owner_raw);
    let source = PlayerId::new(*source_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != source
        || cancelled
        || selections != [1]
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == owner && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "source-confirm-owner-lose-HP context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: *amount,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_target_card_alternate_zone_then_target_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::Illegal(
            "bad alternate-zone target gain payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let option_count = payload[3] as usize;
    if payload.len() != 4 + option_count
        || observer != player
        || cancelled
        || selections.len() != 1
        || !payload[4..].contains(&selections[0])
    {
        return Err(EffectError::Illegal(
            "invalid alternate-zone target gain choice".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                selected: CardId::new(selections[0]),
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad Qixing hand choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("Qixing observer mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qixing choice must be empty".into(),
            ))
        };
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("Qixing skill is no longer enabled".into()))?;
    let current_hand: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let pile = tx.state().skill_pile_cards(player, skill_def);
    if selections.is_empty()
        || selections.len() > u8::MAX as usize
        || selected.len() != selections.len()
        || !selected.is_subset(&current_hand)
        || selections.len() > pile.len()
    {
        return Err(EffectError::Illegal(
            "Qixing hand cards are no longer legal".into(),
        ));
    }
    let mut next_payload = vec![player.raw(), skill.raw(), selections.len() as u32];
    next_payload.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        pile.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseQixingPileCards,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_pile_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal(
            "bad Qixing pile choice payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let count = payload[2] as usize;
    if observer != player
        || cancelled
        || count == 0
        || payload.len() != 3 + count
        || selections.len() != count
    {
        return Err(EffectError::Illegal(
            "Qixing requires equal hand and pile card counts".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveQixing {
            player,
            initialize: false,
            stage: 2,
            hand_cards: payload[3..].iter().copied().map(CardId::new).collect(),
            pile_cards: selections.iter().copied().map(CardId::new).collect(),
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_weather_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, _skill_raw, qixing_raw, _helper_raw, max_targets] = payload else {
        return Err(EffectError::Illegal(
            "bad Qixing weather card payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "Qixing weather observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qixing weather choice must be empty".into(),
            ))
        };
    }
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, *qixing_raw)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selections.is_empty()
        || selections.len() > *max_targets as usize
        || selected.len() != selections.len()
        || !selected.is_subset(&current)
    {
        return Err(EffectError::Illegal(
            "Qixing weather cards are no longer legal".into(),
        ));
    }
    let mut next_payload = payload.to_vec();
    next_payload.push(selections.len() as u32);
    next_payload.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectTargets,
        ContinuationKind::ChooseQixingWeatherTargets,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_weather_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::Illegal(
            "bad Qixing weather target payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[5] as usize;
    if observer != player
        || cancelled
        || count == 0
        || payload.len() != 6 + count
        || selections.len() != count
    {
        return Err(EffectError::Illegal(
            "Qixing weather requires equal card and target counts".into(),
        ));
    }
    let qixing_skill_def = engine_api::SkillDefId::new(payload[2]);
    let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, qixing_skill_def.raw())
        .into_iter()
        .collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    let mut unique_targets = std::collections::BTreeSet::new();
    if selected.len() != cards.len()
        || !selected.is_subset(&current)
        || !selections.iter().all(|target| {
            let target = PlayerId::new(*target);
            unique_targets.insert(target)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Qixing weather selection is no longer legal".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == qixing_skill_def.raw())
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("Qixing pile is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def: engine_api::SkillDefId::new(payload[3]),
            max_targets: payload[4] as u8,
            stage: 1,
            cards: cards.clone(),
            targets: selections.iter().copied().map(PlayerId::new).collect(),
            skill: engine_api::SkillId::new(payload[1]),
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(pile),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_link_other(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad link-other choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "link-other choice requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let target_unlinked = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if observer != player
        || cancelled
        || player == target
        || !target_unlinked
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "link-other choice is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_poxi_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal("bad Poxi choice payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let hand_limit_skill_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    let options = payload
        .get(5..5 + option_count)
        .ok_or_else(|| EffectError::Illegal("truncated Poxi choice payload".into()))?;
    if observer != player {
        return Err(EffectError::Illegal("Poxi chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Poxi choice must be empty".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 4 {
        return Err(EffectError::Illegal(
            "Poxi requires exactly four cards".into(),
        ));
    }
    let mut selected = selections.to_vec();
    selected.sort_unstable();
    selected.dedup();
    if selected.len() != 4 || selected.iter().any(|card| !options.contains(card)) {
        return Err(EffectError::Illegal(
            "Poxi selected cards are not distinct offered options".into(),
        ));
    }
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing Poxi owner hand".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing Poxi target hand".into()))?;
    if player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Poxi context is no longer legal".into(),
        ));
    }
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let mut suits = Vec::with_capacity(4);
    let mut own_cards = Vec::new();
    let mut target_cards = Vec::new();
    for card in cards {
        let state = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal("Poxi card no longer exists".into()))?;
        if state.owner == player && state.zone == player_hand {
            own_cards.push(card);
        } else if state.owner == target && state.zone == target_hand {
            target_cards.push(card);
        } else {
            return Err(EffectError::Illegal(
                "Poxi card left the selected hands".into(),
            ));
        }
        let suit = crate::query::effective_card_suit(tx.state(), state.owner, state);
        if suit == engine_api::Suit::None || suits.contains(&suit) {
            return Err(EffectError::Illegal(
                "Poxi cards must have four distinct suits".into(),
            ));
        }
        suits.push(suit);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage: 1,
            own_count: own_cards.len() as u8,
            skill,
        });
    let mut effects = Vec::new();
    if !own_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: own_cards,
            to: discard,
            from: Some(player_hand),
        });
    }
    if !target_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: target_cards,
            to: discard,
            from: Some(target_hand),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_drlt_jieying_transfer(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jieying transfer payload".into()));
    };
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Jieying transfer requires one target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let mark_count = |target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .and_then(|participant| {
                participant
                    .marks
                    .iter()
                    .find(|(found, _)| *found == mark.raw())
                    .map(|(_, count)| *count)
            })
            .unwrap_or(0)
            .max(0) as u32
    };
    let count = mark_count(player);
    if observer != player
        || cancelled
        || player == target
        || count == 0
        || mark_count(target) != 0
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Jieying transfer is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveMark {
                player,
                mark,
                amount: 0,
            },
            EffectRequest::AddMark {
                player: target,
                mark,
                delta: count.min(i32::MAX as u32) as i32,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_disable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Duorui requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Duorui equipment slot".into()))?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
    {
        return Err(EffectError::Illegal(
            "Duorui equipment slot is no longer enabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DisableEquipSlots {
                player,
                slots: vec![slot as u8],
            },
            EffectRequest::ResolveDrltDuorui {
                player,
                target: PlayerId::new(*target_raw),
                helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
                stage: 1,
                selected_skill_def: engine_api::SkillDefId::NONE,
                skill: engine_api::SkillId::new(*skill_raw),
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui skill payload".into()));
    };
    let [selected] = selections else {
        return Err(EffectError::Illegal("Duorui requires one skill".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("invalid Duorui skill choice".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveDrltDuorui {
            player,
            target: PlayerId::new(*target_raw),
            helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
            stage: 2,
            selected_skill_def: engine_api::SkillDefId::new(*selected),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_zhiti_enable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Zhiti slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Zhiti requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Zhiti equipment slot".into()))?;
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] > 0)
    {
        return Err(EffectError::Illegal(
            "Zhiti equipment slot is no longer disabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::EnableEquipSlot {
            player,
            slot: slot as u8,
            source: player,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_jilue_jizhi_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, card_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jilue Jizhi payload".into()));
    };
    let [discard] = selections else {
        return Err(EffectError::Illegal(
            "Jilue Jizhi requires one confirmation".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || *discard > 1 {
        return Err(EffectError::Illegal(
            "invalid Jilue Jizhi confirmation".into(),
        ));
    }
    if *discard == 0 {
        return Ok(());
    }
    let card = CardId::new(*card_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def)
        .ok_or_else(|| EffectError::Illegal("Jilue Jizhi skill context changed".into()))?;
    if !tx.state().hand_cards(player).contains(&card)
        || !tx.state().cards.get(card).is_some_and(|card_state| {
            crate::query::effective_card_kind(tx.state(), player, card_state).is_basic()
        })
    {
        return Err(EffectError::Illegal(
            "Jilue Jizhi drawn card is no longer discardable".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![card],
            },
            EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(skill_def.raw()),
                delta: 1,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, _skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad max-mark death judgment payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "max-mark death judgment requires one target".into(),
        ));
    }
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let target = PlayerId::new(selections[0]);
    let mark_count = |candidate: &crate::state::Player| {
        candidate
            .marks
            .iter()
            .find(|(definition, _)| *definition == mark.raw())
            .map_or(0, |(_, count)| *count)
    };
    let max = tx
        .state()
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != player)
        .map(mark_count)
        .max()
        .unwrap_or(0);
    let valid = max > 0
        && tx.state().players.get(target).is_some_and(|candidate| {
            candidate.alive && candidate.id != player && mark_count(candidate) == max
        });
    if !valid {
        return Err(EffectError::Illegal(
            "max-mark death judgment target is stale".into(),
        ));
    }

    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let kind = tx
        .state()
        .cards
        .get(result.card)
        .map(|card| card.kind)
        .ok_or_else(|| EffectError::EngineFault("judgment result card is missing".into()))?;
    let mut effects = vec![EffectRequest::EmitJudgmentResolved {
        player: target,
        result_card: result.card,
        suit: result.suit,
        rank: result.rank,
        cause_card: CardId::NONE,
    }];
    if !matches!(kind, CardKind::Tao | CardKind::TrickTaoyuan) {
        effects.push(EffectRequest::KillPlayer {
            player: target,
            killer: PlayerId::NONE,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [removable_skill_def, restore_skill_def] = payload else {
        return Err(EffectError::Illegal(
            "bad recover-to-max skill exchange payload".into(),
        ));
    };
    if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
        return Err(EffectError::Illegal(
            "recover-to-max skill exchange requires one confirm choice".into(),
        ));
    }
    if selections[0] == 0 {
        return Ok(());
    }
    let removable_skill_def = engine_api::SkillDefId::new(*removable_skill_def);
    let restore_skill_def = engine_api::SkillDefId::new(*restore_skill_def);
    let valid = tx.state().players.get(observer).is_some_and(|participant| {
        participant.alive
            && participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
    });
    if !valid {
        return Ok(());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveSkillByDef {
                player: observer,
                skill_def: removable_skill_def,
            },
            EffectRequest::AddSkill {
                player: observer,
                skill_def: restore_skill_def,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn response_card_zone(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<engine_api::ZoneId, EffectError> {
    let card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad response card".into()))?;
    let in_hand = tx.state().zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = tx.state().zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = tx.state().zones.find(ZoneKind::Expansion, player) == Some(card.zone);
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card.id).is_some();
    let fusi = crate::command::skill::dddfusi_approved_source(tx.state(), player, card.id)
        .is_some()
        && card.face_up
        && tx
            .state()
            .zones
            .get(card.zone)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    if (!borrowed && !fusi && card.owner != player)
        || (!in_hand && !in_equip && !in_expansion && !borrowed && !fusi)
    {
        return Err(EffectError::Illegal(
            "response card not owned in hand, equip, or expansion".into(),
        ));
    }
    Ok(card.zone)
}
