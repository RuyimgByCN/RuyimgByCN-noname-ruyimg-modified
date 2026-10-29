//! Command operations for `resolve_jiebing` and related helpers.

use super::*;

pub(crate) fn resolve_mbduanyang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5
        || payload[0] != crate::effect::MBDUANYANG_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 断鞅 continuation".into()));
    }
    if cancelled {
        return if payload[1] == 2 {
            Ok(())
        } else {
            Err(EffectError::Illegal("断鞅 cannot cancel".into()))
        };
    }
    if selections.is_empty() || (payload[1] == 1 && selections.len() != 1) || selections.len() > 2 {
        return Err(EffectError::Illegal("invalid 断鞅 selection".into()));
    }
    let player = PlayerId::new(payload[2]);
    let (target, card, mode) = match payload[1] {
        1 => (PlayerId::new(selections[0]), CardId::new(payload[3]), 1),
        2 => (PlayerId::new(payload[3]), CardId::NONE, 2),
        _ => return Err(EffectError::EngineFault("unknown 断鞅 stage".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage: 1,
            cards: if mode == 2 {
                selections.iter().copied().map(CardId::new).collect()
            } else {
                Vec::new()
            },
            skill: SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbrunwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload[0] != crate::effect::MBRUNWEI_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 润微 continuation".into()));
    }
    let stage = payload[1] as u8;
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if cancelled {
        if stage == 0 && selections.is_empty() {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
            }
            return Ok(());
        }
        return Err(EffectError::Illegal("润微 cannot cancel now".into()));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("润微 requires one selection".into()));
    }
    let (target, value, cards, next_stage) = match stage {
        0 if (1..=5).contains(&selections[0]) => (PlayerId::NONE, selections[0], Vec::new(), 1),
        1 if matches!(selections[0], 1 | 2) => (
            PlayerId::NONE,
            selections[0],
            payload[4..].iter().copied().map(CardId::new).collect(),
            2,
        ),
        2 => (
            PlayerId::new(selections[0]),
            *payload.get(4).unwrap_or(&0),
            payload[5..].iter().copied().map(CardId::new).collect(),
            3,
        ),
        _ => return Err(EffectError::Illegal("invalid 润微 branch".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage: next_stage,
            value,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbshuanghuai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBSHUANGHUAI_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad 霜怀 continuation".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbshuanghuai {
            player: PlayerId::new(payload[1]),
            target: PlayerId::new(payload[2]),
            stage: 1,
            choice: selections[0] as u8,
            skill: SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbxiongtu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::MBXIONGTU_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 凶图 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let (stage, card, cards, skill) = match payload[1] {
        0 if !cancelled && selections.len() == 1 => (
            1,
            CardId::new(selections[0]),
            Vec::new(),
            SkillId::new(payload[4]),
        ),
        1 if cancelled && selections.is_empty() => (
            3,
            CardId::new(payload[4]),
            Vec::new(),
            SkillId::new(payload[5]),
        ),
        1 if !cancelled && !selections.is_empty() => (
            2,
            CardId::new(payload[4]),
            selections.iter().copied().map(CardId::new).collect(),
            SkillId::new(payload[5]),
        ),
        _ => return Err(EffectError::Illegal("invalid 凶图 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbsijian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::MBSIJIAN_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 死谏 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    if payload[1] == 0 && cancelled && selections.is_empty() {
        let skill = SkillId::new(payload[4]);
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.turn_fire_count = instance.turn_fire_count.saturating_sub(1);
        }
        return Ok(());
    }
    let (stage, target, choice, helper_def, skill) = match payload[1] {
        0 if !cancelled && selections.len() == 1 && matches!(selections[0], 1 | 2 | 3) => (
            1,
            PlayerId::NONE,
            selections[0] as u8,
            SkillDefId::new(payload[3]),
            SkillId::new(payload[4]),
        ),
        1 if !cancelled && selections.len() == 1 => (
            2,
            PlayerId::new(selections[0]),
            payload[3] as u8,
            SkillDefId::new(payload[4]),
            SkillId::new(payload[5]),
        ),
        _ => return Err(EffectError::Illegal("invalid 死谏 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbfeijing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::MBFEIJING_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 飞径 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 if observer == player && !cancelled && selections.len() == 1 => {
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let skill = SkillId::new(payload[5]);
            let mut order = tx.state().players.alive_by_seat();
            let start = order.iter().position(|role| *role == player);
            let end = order.iter().position(|role| *role == target);
            let (Some(start), Some(end)) = (start, end) else {
                return Err(EffectError::Illegal("飞径 endpoints changed".into()));
            };
            let step = match selections[0] {
                1 => 1,
                2 => -1,
                _ => return Err(EffectError::Illegal("invalid 飞径 path".into())),
            };
            let mut path = Vec::new();
            let mut index = start;
            loop {
                index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
                if index == end {
                    break;
                }
                path.push(order[index]);
            }
            order.clear();
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveMbfeijing {
                    player,
                    target,
                    card,
                    stage: 1,
                    targets: path,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 if !cancelled && selections.len() == 1 && payload.len() >= 7 => {
            let count = payload[6] as usize;
            if payload.len() < 7 + count {
                return Err(EffectError::EngineFault("truncated 飞径 path".into()));
            }
            let targets: Vec<_> = payload[7..7 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut cards: Vec<_> = payload[7 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let chooser = targets
                .get(cards.len())
                .copied()
                .ok_or_else(|| EffectError::Illegal("飞径 choice index overflow".into()))?;
            let selected = CardId::new(selections[0]);
            if observer != chooser
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    chooser,
                    chooser,
                    rule_api::TargetCardScope::Hand,
                )
                .contains(&selected)
            {
                return Err(EffectError::Illegal("invalid 飞径 hand card".into()));
            }
            cards.push(selected);
            crate::effect::continue_mbfeijing_card_choices(
                tx,
                player,
                PlayerId::new(payload[3]),
                CardId::new(payload[4]),
                SkillId::new(payload[5]),
                targets,
                cards,
            )
            .map(|_| ())
        }
        2 if observer == player && payload.len() >= 6 => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("invalid 飞径 color".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count * 2 {
                return Err(EffectError::EngineFault(
                    "truncated 飞径 color context".into(),
                ));
            }
            let targets = payload[6..6 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let cards = payload[6 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            crate::effect::commit_mbfeijing_color(
                tx,
                player,
                CardId::new(payload[3]),
                SkillId::new(payload[4]),
                selections[0] as u8,
                targets,
                cards,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 飞径 selection".into())),
    }
}

pub(crate) fn resolve_mbxuye(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBXUYE_CUSTOM_CHOICE
        || observer.raw() != payload[1]
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 蓄业 selection".into()));
    }
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(selections[0]);
    let from = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .find(|zone| {
            tx.state()
                .zones
                .get(*zone)
                .is_some_and(|zone| zone.cards.contains(&card))
        })
        .ok_or_else(|| EffectError::Illegal("蓄业 card left target region".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: draw,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_spmiaolve_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || observer.raw() != payload[1] {
        return Err(EffectError::Illegal("invalid 妙略 chooser".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [choice] = selections else {
        return Err(EffectError::Illegal("妙略 requires one branch".into()));
    };
    if ![
        0,
        CardKind::TrickWuzhong.tag() as u32,
        CardKind::TrickGuohe.tag() as u32,
        CardKind::TrickWuxie.tag() as u32,
    ]
    .contains(choice)
    {
        return Err(EffectError::Illegal("invalid 妙略 branch".into()));
    }
    let player = PlayerId::new(payload[1]);
    run_effects(
        tx,
        vec![if *choice == 0 {
            EffectRequest::DrawCards { player, count: 2 }
        } else {
            EffectRequest::GainCardByKind {
                player,
                kind: *choice as u8,
            }
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_spyingjia_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || observer.raw() != payload[2] {
        return Err(EffectError::Illegal("invalid 迎驾 chooser".into()));
    }
    let player = PlayerId::new(payload[2]);
    let helper_def = engine_api::SkillDefId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    match payload[1] {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("迎驾 requires one hand card".into()));
            };
            let card = CardId::new(*card_raw);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tx
                .state()
                .cards
                .get(card)
                .is_none_or(|card| card.owner != player || Some(card.zone) != hand)
            {
                return Err(EffectError::Illegal("invalid 迎驾 discard card".into()));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    crate::effect::SPYINGJIA_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    helper_def.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if payload.len() != 6 || cancelled {
                return Err(EffectError::Illegal("invalid 迎驾 target choice".into()));
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("迎驾 requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("迎驾 target is not alive".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveSpyingjia {
                    player,
                    action: 4,
                    helper_def,
                    card: CardId::new(payload[5]),
                    card_kind: 0,
                    target,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 迎驾 choice stage".into())),
    }
}

pub(crate) fn resolve_pottuntian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || observer.raw() != payload[1] || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 势屯田蓄力选择".into()));
    }
    let count = payload[4] as usize;
    if count == 0 || payload.len() != 5 + count {
        return Err(EffectError::EngineFault("malformed 势屯田目标列表".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let zaoxian_def = engine_api::SkillDefId::new(payload[3]);
    let amount = u8::try_from(selections[0])
        .map_err(|_| EffectError::Illegal("势屯田蓄力数量过大".into()))?;
    if usize::from(amount) < count {
        return Err(EffectError::Illegal("势屯田蓄力少于目标数".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolvePottuntian {
            player,
            action: 3,
            amount,
            targets: payload[5..].iter().copied().map(PlayerId::new).collect(),
            zaoxian_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbzhonghui_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || observer.raw() != payload[1] || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 俟资蓄力选择".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhonghui {
            player: PlayerId::new(payload[1]),
            target: PlayerId::NONE,
            action: 8,
            amount: selections[0],
            mbsizi_def: SkillDefId::new(payload[3]),
            mbkechang_def: SkillDefId::new(payload[4]),
            skill: SkillId::new(payload[2]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potjiejie_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 10 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 诫节花色选择".into()));
    }
    let player = PlayerId::new(payload[1]);
    let actor = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|state| state.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("诫节上下文已变化".into()));
    }
    let suit_index = usize::try_from(selections[0])
        .ok()
        .filter(|index| *index < 4)
        .ok_or_else(|| EffectError::Illegal("invalid 诫节花色".into()))?;
    let suit = [
        engine_api::Suit::Spade,
        engine_api::Suit::Heart,
        engine_api::Suit::Club,
        engine_api::Suit::Diamond,
    ][suit_index];
    let hand = tx.state().hand_cards(actor);
    let has_suit = hand.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| crate::query::effective_card_suit(tx.state(), actor, card) == suit)
    });
    let mut effects = Vec::new();
    if has_suit {
        effects.push(EffectRequest::AddSkillUntilTurnEnd {
            player: actor,
            skill_def: SkillDefId::new(payload[5 + suit_index]),
        });
        let discardable: std::collections::BTreeSet<_> =
            crate::query::discardable_hand_equip_cards(tx.state(), actor)
                .into_iter()
                .collect();
        let cards: Vec<_> = hand
            .into_iter()
            .filter(|card| {
                discardable.contains(card)
                    && tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::effective_card_suit(tx.state(), actor, card) != suit
                    })
            })
            .collect();
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: actor,
                cards,
            });
        }
    } else {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        let card = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), actor, card) == suit
                })
            })
        });
        if let Some(card) = card {
            let hand_zone = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::EngineFault("hand zone missing".into()))?;
            effects.push(EffectRequest::MoveCards {
                cards: vec![card],
                to: hand_zone,
                from: Some(draw),
            });
        }
    }
    if payload[9] != 0 {
        let qingshi_def = SkillDefId::new(payload[4]);
        if let Some(qingshi) = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|instance| {
                instance.owner == player
                    && instance.def == qingshi_def
                    && tx.state().skill_effectively_enabled(instance)
            })
            .map(|instance| instance.id)
        {
            effects.push(EffectRequest::ResolvePotqingshi {
                player,
                skill: qingshi,
            });
        }
    }
    run_effects(tx, effects).map(|_| ())
}
