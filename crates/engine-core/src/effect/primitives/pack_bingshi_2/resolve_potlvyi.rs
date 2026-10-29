//! Effect primitives for `resolve_potlvyi` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_potlvyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    event_cards: &[CardId],
    _targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        // 隳律：其他角色出牌阶段开始，选择本轮尚未使用的颜色。
        0 => {
            if target == player
                || tx.state().turn.current != target
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
            {
                return Ok(Vec::new());
            }
            let colors = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.pothuilv_state(tx.state().turn.round).0)
                .unwrap_or(0);
            let options: Vec<_> = [1u32, 2]
                .into_iter()
                .filter(|color| colors & (*color as u8) == 0)
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTLVYI_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        // 隳律：记录该实体牌在绑定的出牌阶段内被使用。
        1 => {
            let round = tx.state().turn.round;
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let (colors, actor, tracked, used) = instance.pothuilv_state(round);
            if !used
                && actor == target
                && tracked == card
                && tx.state().turn.current == actor
                && tx.state().turn.phase == Phase::Play
            {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.set_pothuilv_state(round, colors, actor, tracked, true);
                }
            }
            Ok(Vec::new())
        }
        // 隳律：阶段末按“使用过 / 不在弃牌堆”分别结算。
        2 => {
            let round = tx.state().turn.round;
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let (colors, actor, tracked, used) = instance.pothuilv_state(round);
            if actor != target || tracked.is_none() {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.set_pothuilv_state(round, colors, PlayerId::NONE, CardId::NONE, false);
            }
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let outside_discard = tx
                .state()
                .cards
                .get(tracked)
                .is_some_and(|value| Some(value.zone) != discard);
            let mut effects = Vec::new();
            if used {
                effects.push(EffectRequest::DealDamage {
                    source: player,
                    target: actor,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                });
            }
            if outside_discard {
                effects.push(EffectRequest::DealDamage {
                    source: actor,
                    target: player,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                });
                effects.push(EffectRequest::ResolvePotlvyi {
                    player,
                    target: actor,
                    card: tracked,
                    action: 10,
                    event_cards: Vec::new(),
                    targets: Vec::new(),
                    skill,
                });
            }
            run_effects(tx, effects)
        }
        // 颂谳：记录本轮因弃置进入事件批次的实体牌颜色。
        3 => {
            let colors = event_cards.iter().fold(0, |mask, card| {
                mask | tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|value| crate::query::card_color_mask_for_suit(value.suit))
                    .unwrap_or(0)
            });
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.record_potsongyan_discard(round, target, colors);
            }
            Ok(Vec::new())
        }
        // 颂谳：视为使用无中生有，并把这张虚拟牌绑定到本技能的颜色历史。
        4 => {
            if !crate::command::play::source_free_wuzhong_has_legal_use(tx.state(), player) {
                return Ok(Vec::new());
            }
            let virtual_card = materialize_virtual_card(tx, player, CardKind::TrickWuzhong)?;
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.add_potsongyan_card(round, virtual_card);
            }
            crate::command::play::play_as_kind(
                tx,
                player,
                virtual_card,
                CardKind::TrickWuzhong,
                &[player],
                false,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        // 当前牌完整结算后清理颂谳授权和势施逐牌快照。
        5 | 9 => {
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.remove_potsongyan_card(round, card);
                instance.clear_potshishi_card(card);
            }
            Ok(Vec::new())
        }
        // 势施：首次响应发生前累计伤害来源。
        6 => {
            if target != player
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
            {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.record_potshishi_source(target);
                }
            }
            Ok(Vec::new())
        }
        // 势施：使用牌时冻结这一实体牌的不可响应角色集合。
        7 => {
            if card.is_some() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.snapshot_potshishi_card(card);
                }
            }
            Ok(Vec::new())
        }
        // 势施：第一名其他角色响应拥有者的牌后永久停止累计。
        8 => {
            if target != player {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.disable_potshishi();
                }
            }
            Ok(Vec::new())
        }
        // 隳律伤害完成后，可把仍未进入弃牌堆的跟踪牌交给阶段角色以外的人。
        10 => {
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            if tx
                .state()
                .cards
                .get(card)
                .is_none_or(|value| Some(value.zone) == discard)
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != target)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    POTLVYI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault("unknown 势吕壹 action".into())),
    }
}

pub(crate) fn resolve_potlvyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != POTLVYI_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势吕壹 payload".into()));
    }
    let player = PlayerId::new(payload[2]);
    let actor = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("势吕壹技能或选择者已变化".into()));
    }
    match payload[1] {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("隳律颜色选择无效".into()));
            };
            let color = *choice as u8;
            if !matches!(color, 1 | 2) {
                return Err(EffectError::Illegal("隳律颜色无效".into()));
            }
            let round = tx.state().turn.round;
            let used = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.pothuilv_state(round).0)
                .unwrap_or(0);
            if used & color != 0
                || tx.state().turn.current != actor
                || tx.state().turn.phase != Phase::Play
            {
                return Err(EffectError::Illegal("隳律颜色或阶段已变化".into()));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let card = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .copied()
                .find(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        crate::query::card_color_mask_for_suit(value.suit) & color != 0
                    })
                })
                .unwrap_or(CardId::NONE);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.set_pothuilv_state(round, used | color, actor, card, false);
            }
            if card.is_none() {
                return Ok(());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::EngineFault("actor hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to: hand,
                        from: Some(draw),
                    },
                    EffectRequest::RevealCards {
                        cards: vec![card],
                        to: Vec::new(),
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Ok(());
            }
            if payload.len() != 6 || selections.len() != 1 {
                return Err(EffectError::Illegal("隳律交牌目标无效".into()));
            }
            let recipient = PlayerId::new(selections[0]);
            let card = CardId::new(payload[5]);
            if recipient == actor
                || !tx
                    .state()
                    .players
                    .get(recipient)
                    .is_some_and(|role| role.alive)
            {
                return Err(EffectError::Illegal("隳律交牌目标已变化".into()));
            }
            let value = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("隳律跟踪牌已消失".into()))?;
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            if Some(value.zone) == discard {
                return Err(EffectError::Illegal("隳律跟踪牌已进入弃牌堆".into()));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, recipient)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![card],
                    to: hand,
                    from: Some(value.zone),
                    reason: CardMoveReason::Give,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势吕壹 choice stage".into(),
        )),
    }
}

pub(crate) fn potcaoshuang_assignment_union(tx: &Transaction, player: PlayerId) -> u8 {
    [crate::state::POTDIANYI_DEF, crate::state::POTSHEQUAN_DEF]
        .into_iter()
        .filter_map(|def| skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def)))
        .filter_map(|skill| tx.state().skills.get(skill))
        .fold(0, |mask, skill| {
            mask | skill.potcaoshuang_state(tx.state().turn.round).0
        })
}

pub(crate) fn potcaoshuang_skill_options(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    [crate::state::POTDIANYI_DEF, crate::state::POTSHEQUAN_DEF]
        .into_iter()
        .filter(|def| {
            skill_instance_by_def(tx, player, engine_api::SkillDefId::new(*def)).is_some()
        })
        .collect()
}
