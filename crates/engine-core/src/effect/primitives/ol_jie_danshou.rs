//! OL界朱然「胆守」：四选项每阶段各一次；选择目标执行效果，然后弃置序号张牌。
//!
//! 阶段 0 选项、1 目标、2 目标牌（选项 1/2）、3 弃置自身序号张牌。
//! 使用技能私有 storage 键 1..4 记录本阶段已用选项；`phase_play` 触发清零。

use super::*;

fn used(tx: &Transaction, skill: engine_api::SkillId, option: u32) -> bool {
    tx.state().skills.storage_get(skill, option) > 0
}

fn has_cards(tx: &Transaction, player: PlayerId) -> bool {
    [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state()
            .zones
            .find(kind, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| !zone.cards.is_empty())
    })
}

fn he_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        if let Some(zone) = tx
            .state()
            .zones
            .find(kind, player)
            .and_then(|zone| tx.state().zones.get(zone))
        {
            cards.extend(zone.cards.iter().copied());
        }
    }
    cards
}

fn option_cards(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    option: u32,
) -> Vec<CardId> {
    match option {
        1 => crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        ),
        2 => crate::query::gainable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        ),
        _ => he_cards(tx, target),
    }
}

fn targets_for(tx: &Transaction, player: PlayerId, option: u32) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| match option {
            1 | 2 => {
                (option != 2 || *candidate != player)
                    && !option_cards(tx, player, *candidate, option).is_empty()
            }
            _ => true,
        })
        .collect()
}

fn available_options(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Vec<u32> {
    if !has_cards(tx, player) {
        return Vec::new();
    }
    (1..=4)
        .filter(|option| {
            !used(tx, skill, *option) && !targets_for(tx, player, *option).is_empty()
        })
        .collect()
}

fn open_self_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    option: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = he_cards(tx, player);
    let max = option.min(cards.len() as u32) as u8;
    if max == 0 {
        return Ok(Vec::new());
    }
    crate::effect::primitives::open_choice(
        tx,
        player,
        0,
        max,
        true,
        cards.into_iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveOlJieDanshou,
        vec![3, player.raw(), skill.raw(), option],
    )
}

pub(crate) fn resolve_ol_jie_danshou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Ok(Vec::new());
    }
    let options = available_options(tx, player, skill);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    crate::effect::primitives::open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ResolveOlJieDanshou,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_ol_jie_danshou_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player_raw, skill_raw, rest @ ..] = payload else {
        return Err(EffectError::EngineFault("bad danshou continuation".into()));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("danshou observer mismatch".into()));
    }
    if *stage == 0 {
        if cancelled || selections.len() != 1 {
            return Err(EffectError::Illegal("choose one danshou option".into()));
        }
        let option = selections[0];
        let options = available_options(tx, player, skill);
        if !options.contains(&option) {
            return Err(EffectError::Illegal("danshou option unavailable".into()));
        }
        let targets = targets_for(tx, player, option);
        return crate::effect::primitives::open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.into_iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveOlJieDanshou,
            vec![1, player.raw(), skill.raw(), option],
        )
        .map(|_| ());
    }
    if *stage == 1 {
        let option = *rest
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing danshou option".into()))?;
        if cancelled || selections.len() != 1 {
            return Err(EffectError::Illegal("choose one danshou target".into()));
        }
        let target = PlayerId::new(selections[0]);
        if !targets_for(tx, player, option).contains(&target) {
            return Err(EffectError::Illegal("danshou target invalid".into()));
        }
        tx.state_mut().skills.storage_set(skill, option, 1);
        return match option {
            1 | 2 => {
                let cards = option_cards(tx, player, target, option);
                if cards.is_empty() {
                    Err(EffectError::Illegal("danshou target has no card".into()))
                } else {
                    crate::effect::primitives::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(|card| card.raw()).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::ResolveOlJieDanshou,
                        vec![2, player.raw(), skill.raw(), option, target.raw()],
                    )
                    .map(|_| ())
                }
            }
            3 => {
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?;
                let _ = open_self_discard(tx, player, skill, option)?;
                Ok(())
            }
            _ => {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards { player, count: 2 }],
                )?;
                if tx.state().players.get(target).is_some_and(|p| p.alive) {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player: target,
                            count: 2,
                        }],
                    )?;
                }
                let _ = open_self_discard(tx, player, skill, option)?;
                Ok(())
            }
        };
    }
    if *stage == 2 {
        let option = *rest
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing danshou option".into()))?;
        let target = PlayerId::new(
            *rest
                .get(1)
                .ok_or_else(|| EffectError::EngineFault("missing danshou target".into()))?,
        );
        if cancelled || selections.len() != 1 {
            return Err(EffectError::Illegal("choose one danshou card".into()));
        }
        let card = CardId::new(selections[0]);
        let owner = tx.state().cards.get(card).map(|c| c.owner);
        if owner != Some(target)
            || !option_cards(tx, player, target, option).contains(&card)
        {
            return Err(EffectError::Illegal("danshou card moved".into()));
        }
        if option == 1 {
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![card],
                }],
            )?;
        } else {
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no danshou hand zone".into()))?;
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|c| c.zone)
                .ok_or_else(|| EffectError::Illegal("danshou card zone".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )?;
        }
        let _ = open_self_discard(tx, player, skill, option)?;
        return Ok(());
    }
    if *stage == 3 {
        let option = *rest
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing danshou option".into()))?;
        if selections.len() > option as usize {
            return Err(EffectError::Illegal("too many danshou discards".into()));
        }
        let own = he_cards(tx, player);
        let cards: Vec<CardId> = selections
            .iter()
            .map(|raw| CardId::new(*raw))
            .collect();
        if cards.iter().any(|card| !own.contains(card)) {
            return Err(EffectError::Illegal("danshou discard moved".into()));
        }
        if !cards.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards,
                }],
            )?;
        }
        return Ok(());
    }
    Err(EffectError::EngineFault("unknown danshou stage".into()))
}
