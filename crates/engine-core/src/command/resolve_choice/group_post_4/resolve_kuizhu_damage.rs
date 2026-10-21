//! Command operations for `resolve_kuizhu_damage` and related helpers.

use super::*;

/// 溃诛（伤害分支）：体力值之和为 X 的角色各受 1 点伤害。
pub(crate) fn resolve_kuizhu_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 damage choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let sum: usize = selections
        .iter()
        .map(|&raw| {
            tx.state()
                .players
                .get(PlayerId::new(raw))
                .map(|p| p.hp.max(0) as usize)
                .unwrap_or(0)
        })
        .sum();
    if sum != x || selections.len() > 8 {
        return Err(EffectError::Illegal(
            "溃诛 damage target hp sum must equal the discard count".into(),
        ));
    }
    let requests: Vec<_> = selections
        .iter()
        .map(|&raw| EffectRequest::DealDamage {
            source: player,
            target: PlayerId::new(raw),
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        })
        .collect();
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 掣政：弃置目标的一张牌。
pub(crate) fn resolve_zhizheng_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "掣政 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "掣政 target is no longer legal".into(),
        ));
    }
    let mut options = tx.state().hand_cards(target);
    options.extend(
        tx.state()
            .zones
            .find(crate::state::ZoneKind::Equip, target)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
    );
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_ZHIZHENG_CARD,
            2,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}
