//! Effect primitives for `resolve_twtanfeng` and related helpers.

use super::*;

pub(crate) fn resolve_twtanfeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    phase: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Prepare
    {
        return Ok(Vec::new());
    }
    let alive = |tx: &Transaction, participant: PlayerId| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    };
    match stage {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .filter(|candidate| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        *candidate,
                        TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwtanfeng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquipJudge,
            );
            if !alive(tx, target) || options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwtanfeng,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::command::play::twtanfeng_sha_source_cards(tx, target, player);
            if options.is_empty() {
                return resolve_twtanfeng(tx, player, target, CardId::NONE, 4, 0, skill);
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwtanfeng,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        3 => {
            let legal = crate::command::play::twtanfeng_sha_targets(tx, target, player, card);
            if legal.is_empty() {
                return Err(EffectError::Illegal(
                    "Tanfeng Sha source became illegal".into(),
                ));
            }
            if crate::query::sha_targets_all_for(tx.state(), target) {
                crate::command::play::play_twtanfeng_sha_source(tx, target, player, card, &legal)?;
                return Ok(Vec::new());
            }
            let max = crate::query::sha_target_limit_for(tx.state(), target) as usize;
            if max <= 1 {
                crate::command::play::play_twtanfeng_sha_source(
                    tx,
                    target,
                    player,
                    card,
                    &[player],
                )?;
                return Ok(Vec::new());
            }
            let extras: Vec<_> = legal
                .into_iter()
                .filter(|candidate| *candidate != player)
                .collect();
            if extras.is_empty() {
                crate::command::play::play_twtanfeng_sha_source(
                    tx,
                    target,
                    player,
                    card,
                    &[player],
                )?;
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                0,
                extras.len().min(max - 1).min(u8::MAX as usize) as u8,
                true,
                extras.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwtanfeng,
                vec![3, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        4 => run_effects(
            tx,
            vec![
                EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                },
                EffectRequest::ResolveTwtanfeng {
                    player,
                    target,
                    card: CardId::NONE,
                    stage: 5,
                    phase: 0,
                    skill,
                },
            ],
        ),
        5 => {
            if !alive(tx, target) {
                return Ok(Vec::new());
            }
            let mut phases = Vec::new();
            if !tx.state().pending_judge_skips.contains(&player) {
                phases.push(Phase::Judge.to_u8() as u32);
            }
            if !tx.state().turn.skip_draw() {
                phases.push(Phase::Draw.to_u8() as u32);
            }
            if !tx.state().turn.skip_play() {
                phases.push(Phase::Play.to_u8() as u32);
            }
            if !tx.state().turn.skip_discard() {
                phases.push(Phase::Discard.to_u8() as u32);
            }
            if phases.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                phases,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwtanfeng,
                vec![4, player.raw(), target.raw(), skill.raw()],
            )
        }
        6 => match Phase::from_u8(phase) {
            Phase::Judge => skip_next_judge_phase(tx, player),
            Phase::Draw => {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
                Ok(Vec::new())
            }
            Phase::Play => skip_next_play_phase(tx, player),
            Phase::Discard => {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
                Ok(Vec::new())
            }
            _ => Err(EffectError::Illegal("invalid Tanfeng skipped phase".into())),
        },
        _ => Err(EffectError::ContentFault("unknown TW Tanfeng stage".into())),
    }
}

pub(crate) fn twzhibian_movable_cards(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        TargetCardScope::HandEquipJudge,
    );
    let player_judge = tx.state().zones.find(ZoneKind::Judge, player);
    gainable
        .into_iter()
        .filter(|card| {
            let Some(card_state) = tx.state().cards.get(*card) else {
                return false;
            };
            match tx.state().zones.get(card_state.zone).map(|zone| zone.kind) {
                Some(ZoneKind::Hand) => true,
                Some(ZoneKind::Equip) => card_state.kind.equip_slot().is_some_and(|slot| {
                    tx.state()
                        .players
                        .get(player)
                        .is_some_and(|owner| owner.equip_slot_enabled(slot))
                }),
                Some(ZoneKind::Judge) => {
                    card_state.kind.is_delayed_trick()
                        && player_judge.is_some_and(|zone| {
                            tx.state().zones.get(zone).is_some_and(|zone| {
                                zone.cards.iter().all(|existing| {
                                    tx.state()
                                        .cards
                                        .get(*existing)
                                        .is_none_or(|existing| existing.kind != card_state.kind)
                                })
                            })
                        })
                }
                _ => false,
            }
        })
        .collect()
}

pub(crate) fn twzhibian_move_effects(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if !twzhibian_movable_cards(tx, player, target).contains(&card) {
        return Err(EffectError::Illegal(
            "TW Zhibian card is no longer movable".into(),
        ));
    }
    let card_state = tx.state().cards.get(card).unwrap();
    let source = card_state.zone;
    let source_kind = tx.state().zones.get(source).unwrap().kind;
    let destination_kind = source_kind;
    let destination = tx
        .state()
        .zones
        .find(destination_kind, player)
        .ok_or_else(|| EffectError::EngineFault("TW Zhibian destination missing".into()))?;
    let mut effects = Vec::new();
    if source_kind == ZoneKind::Equip {
        let slot = card_state.kind.equip_slot().unwrap();
        let replaced: Vec<_> = tx
            .state()
            .zones
            .get(destination)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|existing| {
                tx.state()
                    .cards
                    .get(*existing)
                    .and_then(|card| card.kind.equip_slot())
                    == Some(slot)
            })
            .collect();
        if !replaced.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards: replaced,
                to: tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .unwrap(),
                from: Some(destination),
            });
        }
    }
    effects.push(if source_kind == ZoneKind::Hand {
        EffectRequest::GiveCards {
            cards: vec![card],
            to: destination,
            from: source,
        }
    } else {
        EffectRequest::MoveCards {
            cards: vec![card],
            to: destination,
            from: Some(source),
        }
    });
    Ok(effects)
}

pub(crate) fn resolve_twzhibian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player
                || !matches!(tx.state().turn.phase, Phase::Prepare | Phase::Play)
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player && !tx.state().hand_cards(*target).is_empty())
                .filter(|_| !tx.state().hand_cards(player).is_empty())
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwzhibian,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Zhibian skill")
                .clear_twzhibian_compare();
            if winner != player {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            let can_move = !twzhibian_movable_cards(tx, player, target).is_empty();
            let can_recover = tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.hp < owner.max_hp as i32);
            if !can_move && !can_recover {
                return Ok(Vec::new());
            }
            let can_backwater = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .any(|card| {
                    tx.state().cards.get(card).is_some_and(|card| {
                        !crate::query::effective_card_kind(tx.state(), player, card).is_basic()
                    })
                });
            let mut options = Vec::new();
            if can_move {
                options.push(1);
            }
            if can_recover {
                options.push(2);
            }
            if can_backwater {
                options.push(3);
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwzhibian,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            let options = twzhibian_movable_cards(tx, player, target);
            if options.is_empty() {
                return recover_hp(tx, player, player, 1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzhibian,
                vec![3, player.raw(), target.raw(), skill.raw(), 3],
            )
        }
        _ => Err(EffectError::ContentFault("unknown TW Zhibian stage".into())),
    }
}

/// 劫囚·装备栏编排（ABI 673，tw twjieqiu 多路器）：
/// sub 0：选择一名其他存活角色 → 禁用其全部装备槽（现有装备入弃牌堆）并令其
///        摸 k 张（k=禁用前装备区牌数）→ 打上 BUFF 标记；
/// sub 2：若目标仍有禁用槽，为持有者入队一个纯额外回合（check=0 语义）。
// TWJIEQIU_BUFF_MARK 常量上移至 crate::content（dispatch 恢复钩子共用）。

pub(crate) fn resolve_twjieqiu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    subaction: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    match subaction {
        0 => {
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::TwjieqiuSelect,
                vec![player.raw(), 0, skill.raw()],
            )
        }
        2 => {
            let disabled =
                tx.state().players.get(target).is_some_and(|p| {
                    p.alive && p.disabled_equip_slots.iter().any(|&count| count > 0)
                });
            if !disabled {
                return Ok(Vec::new());
            }
            queue_extra_turn(tx, player, 0, skill)
        }
        _ => Err(EffectError::Illegal("bad twjieqiu subaction".into())),
    }
}

/// Cont 282 resolver（劫囚·选目标段）：禁用目标全部装备槽（装备入弃牌堆），
/// 令其摸 k 张（k=禁用前装备区牌数），并打上 BUFF 标记。
pub(crate) fn resolve_twjieqiu_select(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _sub, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad twjieqiu-select payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("twjieqiu chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "twjieqiu must select exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal("bad twjieqiu target".into()));
    }
    // k = 禁用前装备区牌数。
    let k = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| zone.cards.len() as u32)
        .unwrap_or(0);
    let slots: Vec<u8> = (1u8..=5u8).collect();
    disable_equip_slots(tx, target, slots)?;
    let mut effects = Vec::new();
    if k > 0 {
        effects.push(EffectRequest::DrawCards {
            player: target,
            count: k,
        });
    }
    effects.push(EffectRequest::AddMark {
        player: target,
        mark: engine_api::MarkDefId::new(crate::content::TWJIEQIU_BUFF_MARK),
        delta: 1,
    });
    run_effects(tx, effects)?;
    Ok(())
}
