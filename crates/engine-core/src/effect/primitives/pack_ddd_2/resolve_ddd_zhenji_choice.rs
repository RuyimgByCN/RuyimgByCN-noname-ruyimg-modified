//! Effect primitives for `resolve_ddd_jijian_kind_choice` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_zhenji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_DDD_ZHENJI_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated 3D精选甄宓 choice".into(),
        ));
    }
    let choice = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player && choice != 5 {
        return Err(EffectError::Illegal("甄宓选择观察者不匹配".into()));
    }
    match choice {
        0 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || selections.len() != 1 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形必须保留一组起始手牌".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 0,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: payload[5..].iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        1 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let current_hand = tx.state().hand_count(player) as usize;
            let legal: std::collections::BTreeSet<_> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    ddd_zhenji_pile_def(*index).is_ok_and(|pile| {
                        tx.state().skill_pile_cards(player, pile).len() != current_hand
                    })
                })
                .map(u32::from)
                .collect();
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if cancelled {
                if !selections.is_empty() || legal.len() != 1 {
                    return Err(EffectError::Illegal("淼形不能取消强制平衡".into()));
                }
                return Ok(());
            }
            if selections.is_empty()
                || selections.len() > 2
                || selected.len() != selections.len()
                || !selected.is_subset(&legal)
            {
                return Err(EffectError::Illegal("淼形平衡组选择无效".into()));
            }
            let mut piles: Vec<_> = selections.iter().map(|value| *value as u8).collect();
            piles.sort_unstable();
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 1,
                    piles,
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        2 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || count == 0 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形移除水相选择无效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 2,
                    piles: payload[5..].iter().map(|value| *value as u8).collect(),
                    cards: selections.iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        3 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 2,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        4 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世目标时不得选人".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一名男性角色".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::new(selections[0]),
                    action: 2,
                    stage: 3,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        5 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("truncated 浮世目标选择".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal("浮世男性角色观察者不匹配".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世交换时不得选水相".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世男性角色必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target,
                    action: 2,
                    stage: 4,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        _ => {
            return Err(EffectError::EngineFault("unknown 3D精选甄宓 choice".into()));
        }
    }
    Ok(())
}

pub(crate) fn ddd_fenji_total_damage(state: &crate::state::GameState) -> u32 {
    state
        .damage_dealt_totals
        .values()
        .copied()
        .fold(0u32, u32::saturating_add)
}

pub(crate) fn ddd_fenji_phase_valid(tx: &Transaction, player: PlayerId, action: u8) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && match action {
            0 => {
                tx.state().turn.phase == Phase::Draw
                    && tx.state().turn.draw_phase_fixed_count().is_none()
            }
            1 => tx.state().turn.phase == Phase::Discard,
            _ => false,
        }
}

pub(crate) fn ddd_fenji_play(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let count = if action == 0 {
        ddd_fenji_total_damage(tx.state())
    } else {
        targets.len().min(u32::MAX as usize) as u32
    };
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage: 2,
            card,
            kind: kind.tag(),
            count,
            skill,
        });
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, player, card, targets);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}
