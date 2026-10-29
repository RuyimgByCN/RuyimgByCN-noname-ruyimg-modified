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
