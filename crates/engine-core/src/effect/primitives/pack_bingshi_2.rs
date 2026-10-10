//! Effect primitive implementations — bingshi 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) fn potcaoshuang_timing(action: u8, offset: u8) -> Result<u8, EffectError> {
    match action.checked_sub(offset) {
        Some(0) => Ok(0b001),
        Some(1) => Ok(0b010),
        Some(2) => Ok(0b100),
        _ => Err(EffectError::ContentFault(
            "invalid 势曹爽 timing action".into(),
        )),
    }
}

pub(crate) fn resolve_potcaoshuang(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("势曹爽技能已消失".into()))?;
    match action {
        // 渐专：首次三种时机分配给典易/奢权；全部分配后的下一次时机觉醒。
        0..=2 => {
            if skill_def != crate::state::POTJIANZHUAN_DEF {
                return Err(EffectError::ContentFault("渐专技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 0)?;
            let assigned = potcaoshuang_assignment_union(tx, player);
            let options = potcaoshuang_skill_options(tx, player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            if assigned == 0b111 {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options,
                    ChoiceKind::Custom,
                    ContinuationKind::Custom,
                    vec![POTCAOSHUANG_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
                );
            }
            if assigned & timing != 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTCAOSHUANG_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    skill.raw(),
                    u32::from(timing),
                ],
            )
        }
        // 典易：本轮第 X 次发动，随机获得至多 X 张场上/牌堆/弃牌堆的“奢权”牌。
        3..=5 => {
            if skill_def != crate::state::POTDIANYI_DEF {
                return Err(EffectError::ContentFault("典易技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 3)?;
            let round = tx.state().turn.round;
            let (_, triggered, _) = tx
                .state()
                .skills
                .get(skill)
                .unwrap()
                .potcaoshuang_state(round);
            let count = triggered.count_ones().saturating_add(1) as usize;
            let mut candidates: Vec<_> = tx
                .state()
                .cards
                .sorted_live()
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .card_has_content_tag(card.id, crate::state::SHEQUAN_CARD_TAG)
                })
                .filter(|card| {
                    tx.state()
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| match zone.kind {
                            ZoneKind::DrawPile | ZoneKind::DiscardPile => true,
                            ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge => {
                                zone.owner.is_some() && zone.owner != player
                            }
                            _ => false,
                        })
                })
                .map(|card| card.id)
                .collect();
            let mut rng = RuleRng::from_state(tx.state().rng);
            rng.shuffle(&mut candidates);
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            candidates.truncate(count);
            let total_gained = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .and_then(|instance| {
                    instance.fire_potcaoshuang_timing(round, timing, candidates.len() as u32)
                })
                .map(|(_, gained)| gained)
                .ok_or_else(|| EffectError::Illegal("典易本轮时机已结算".into()))?;
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("owner hand missing".into()))?;
            let mut effects = Vec::with_capacity(candidates.len() + 1);
            for candidate in candidates {
                let from = tx.state().cards.get(candidate).unwrap().zone;
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: vec![candidate],
                    to: hand,
                    from: Some(from),
                    reason: CardMoveReason::Unspecified,
                });
            }
            if total_gained
                > tx.state()
                    .players
                    .get(player)
                    .map_or(0, |owner| owner.max_hp)
            {
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            run_effects(tx, effects)
        }
        // 奢权：所有其他角色先各自私选一张未标记手牌，最后统一写入持久标签。
        6..=8 => {
            if skill_def != crate::state::POTSHEQUAN_DEF {
                return Err(EffectError::ContentFault("奢权技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 6)?;
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .and_then(|instance| instance.fire_potcaoshuang_timing(round, timing, 0))
                .ok_or_else(|| EffectError::Illegal("奢权本轮时机已结算".into()))?;
            let mut participants: Vec<PlayerId> = tx.state().players.alive_by_seat();
            if let Some(index) = participants
                .iter()
                .position(|participant| *participant == player)
            {
                let shift = (index + 1) % participants.len().max(1);
                participants.rotate_left(shift);
            }
            participants.retain(|participant| {
                *participant != player
                    && tx.state().hand_cards(*participant).iter().any(|card| {
                        !tx.state()
                            .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                    })
            });
            let Some(observer) = participants.first().copied() else {
                return Ok(Vec::new());
            };
            let options: Vec<_> = tx
                .state()
                .hand_cards(observer)
                .into_iter()
                .filter(|card| {
                    !tx.state()
                        .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                })
                .collect();
            let mut payload = vec![
                POTCAOSHUANG_CUSTOM_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                0,
                participants.len() as u32,
            ];
            payload.extend(participants.iter().map(|participant| participant.raw()));
            open_choice(
                tx,
                observer,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        // 溺尊：回合结束收集弃牌堆中的全部“奢权”牌。
        9 => {
            if skill_def != crate::state::POTNIZUN_DEF
                || tx.state().turn.flag(turn_flag::card_used_by_player(player)) <= 0
            {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(discard)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .filter(|card| {
                    tx.state()
                        .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                })
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            move_cards_with_reason(tx, cards, pile, Some(discard), CardMoveReason::Unspecified)
        }
        // 溺尊：本轮每使用一张牌堆牌，受到的伤害增加一。
        10 => {
            if skill_def != crate::state::POTNIZUN_DEF {
                return Err(EffectError::ContentFault("溺尊技能来源不匹配".into()));
            }
            let count = tx.state().skills.get(skill).map_or(0, |instance| {
                instance.potnizun_use_count(tx.state().turn.round)
            });
            modify_current_damage(tx, player, i32::try_from(count).unwrap_or(i32::MAX))
        }
        // 溺尊：记录本轮从本技能牌堆使用/打出的精确实体牌。
        11 => {
            if skill_def != crate::state::POTNIZUN_DEF
                || !tx.state().turn.used_playable_skill_pile_card(player, card)
                || !tx.state().turn.used_playable_skill_pile(player, skill_def)
                || !tx
                    .state()
                    .card_has_content_tag(card, crate::state::SHEQUAN_CARD_TAG)
            {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.record_potnizun_card(round, card);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown 势曹爽 action".into())),
    }
}

pub(crate) fn resolve_potcaoshuang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != POTCAOSHUANG_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势曹爽 payload".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer.is_none() || !originating_skill_is_enabled(tx, player, skill) || cancelled {
        return Err(EffectError::Illegal("势曹爽技能或强制选择已变化".into()));
    }
    match payload[1] {
        // 渐专觉醒：失去所选技能、获得溺尊，并按该技能时机数成长。
        0 => {
            if observer != player
                || selections.len() != 1
                || potcaoshuang_assignment_union(tx, player) != 0b111
            {
                return Err(EffectError::Illegal("渐专觉醒选择无效".into()));
            }
            let def = selections[0];
            if !matches!(
                def,
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) {
                return Err(EffectError::Illegal("渐专失去技能无效".into()));
            }
            let removed = skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                .ok_or_else(|| EffectError::Illegal("渐专所选技能已消失".into()))?;
            let assigned = tx
                .state()
                .skills
                .get(removed)
                .unwrap()
                .potcaoshuang_state(tx.state().turn.round)
                .0
                .count_ones()
                .saturating_sub(1);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.lifetime_fire_count = 1;
            }
            let mut effects = vec![
                EffectRequest::RemoveSkill { skill: removed },
                EffectRequest::AddSkill {
                    player,
                    skill_def: engine_api::SkillDefId::new(crate::state::POTNIZUN_DEF),
                },
            ];
            if assigned > 0 {
                effects.push(EffectRequest::ChangeMaxHp {
                    player,
                    delta: i32::try_from(assigned).unwrap_or(i32::MAX),
                });
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: assigned,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        // 渐专首次时机分配。
        1 => {
            if observer != player || payload.len() != 5 || selections.len() != 1 {
                return Err(EffectError::Illegal("渐专分配选择无效".into()));
            }
            let timing = payload[4] as u8;
            if !matches!(timing, 0b001 | 0b010 | 0b100)
                || potcaoshuang_assignment_union(tx, player) & timing != 0
            {
                return Err(EffectError::Illegal("渐专时机已变化".into()));
            }
            let def = selections[0];
            if !matches!(
                def,
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) {
                return Err(EffectError::Illegal("渐专目标技能无效".into()));
            }
            let target_skill = skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                .ok_or_else(|| EffectError::Illegal("渐专目标技能已消失".into()))?;
            let round = tx.state().turn.round;
            if !tx
                .state_mut()
                .skills
                .get_mut(target_skill)
                .is_some_and(|instance| instance.assign_potcaoshuang_timing(round, timing))
            {
                return Err(EffectError::Illegal("渐专时机写入失败".into()));
            }
            Ok(())
        }
        // 奢权的逐角色私选；最后一人完成后统一标记。
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault("奢权选择载荷截断".into()));
            }
            let index = payload[4] as usize;
            let count = payload[5] as usize;
            if count == 0 || index >= count || payload.len() != 6 + count + index {
                return Err(EffectError::EngineFault("奢权选择载荷无效".into()));
            }
            let participants: Vec<_> = payload[6..6 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let chooser = participants[index];
            let [selected] = selections else {
                return Err(EffectError::Illegal("奢权必须选择一张牌".into()));
            };
            let selected = CardId::new(*selected);
            let hand = tx.state().zones.find(ZoneKind::Hand, chooser);
            if observer != chooser
                || tx
                    .state()
                    .cards
                    .get(selected)
                    .is_none_or(|card| card.owner != chooser || Some(card.zone) != hand)
                || tx
                    .state()
                    .card_has_content_tag(selected, crate::state::SHEQUAN_CARD_TAG)
            {
                return Err(EffectError::Illegal("奢权所选手牌已变化".into()));
            }
            let mut cards: Vec<_> = payload[6 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            cards.push(selected);
            if index + 1 < count {
                let next = participants[index + 1];
                let options: Vec<_> = tx
                    .state()
                    .hand_cards(next)
                    .into_iter()
                    .filter(|card| {
                        !tx.state()
                            .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                    })
                    .collect();
                if options.is_empty() {
                    return Err(EffectError::Illegal("奢权下一角色已无未标记手牌".into()));
                }
                let mut next_payload = vec![
                    POTCAOSHUANG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    skill.raw(),
                    (index + 1) as u32,
                    count as u32,
                ];
                next_payload.extend(participants.iter().map(|participant| participant.raw()));
                next_payload.extend(cards.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    next,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next_payload,
                )
                .map(|_| ());
            }
            for card in cards {
                tx.state_mut()
                    .add_content_card_tag(card, crate::state::SHEQUAN_CARD_TAG);
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势曹爽 choice stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_potyuji(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    card: CardId,
    action: u8,
    cards: &[CardId],
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        // 符济：展示并按选择顺序一一赠予，随后检查全场最少手牌奖励。
        0 => {
            let other_count = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .count();
            if cards.is_empty()
                || cards.len() != targets.len()
                || cards.len() > other_count
                || targets
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != targets.len()
                || targets.iter().any(|target| {
                    *target == player
                        || !tx
                            .state()
                            .players
                            .get(*target)
                            .is_some_and(|role| role.alive)
                })
            {
                return Err(EffectError::Illegal("符济牌与目标必须一一对应".into()));
            }
            for selected in cards {
                let Some(state) = tx.state().cards.get(*selected) else {
                    return Err(EffectError::Illegal("符济所选牌不存在".into()));
                };
                let legal = state.owner == player
                    && tx.state().zones.get(state.zone).is_some_and(|zone| {
                        zone.owner == player
                            && matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                    });
                if !legal {
                    return Err(EffectError::Illegal(
                        "符济只能交出自己的手牌或装备牌".into(),
                    ));
                }
            }
            let mut effects = vec![EffectRequest::RevealCards {
                cards: cards.to_vec(),
                to: Vec::new(),
            }];
            for (selected, recipient) in cards.iter().zip(targets) {
                tx.state_mut()
                    .add_content_card_tag(*selected, crate::state::POTFUJI_CARD_TAG);
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, *recipient)
                    .ok_or_else(|| EffectError::EngineFault("符济目标手牌区不存在".into()))?;
                let from = tx.state().cards.get(*selected).unwrap().zone;
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: vec![*selected],
                    to: hand,
                    from: Some(from),
                    reason: CardMoveReason::Give,
                });
                effects.push(EffectRequest::AddSkill {
                    player: *recipient,
                    skill_def: engine_api::SkillDefId::new(crate::state::POTFUJI_EFFECT_DEF),
                });
            }
            let mut events = run_effects(tx, effects)?;
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .min()
                .unwrap_or(0);
            if tx.state().hand_count(player) == min_hand {
                events.extend(run_effects(
                    tx,
                    vec![
                        EffectRequest::DrawCards { player, count: 2 },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: engine_api::SkillDefId::new(crate::state::POTFUJI_SHA_DEF),
                        },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: engine_api::SkillDefId::new(crate::state::POTFUJI_SHAN_DEF),
                        },
                    ],
                )?);
            }
            Ok(events)
        }
        // 使用符济牌：每张物理来源牌按原始花色从牌堆取得一张牌。
        1 => gain_potfuji_matching_suit_cards(tx, player, card, cards, true),
        // 符济【杀】造成的每次伤害 +1。
        2 => {
            if potfuji_event_kind(tx, player, card).is_some_and(CardKind::is_sha)
                && potfuji_event_has_tagged_source(tx, card, cards)
            {
                run_effects(
                    tx,
                    vec![EffectRequest::ModifyCurrentDamage {
                        player,
                        delta: 1,
                        skill,
                    }],
                )
            } else {
                Ok(Vec::new())
            }
        }
        // 符济【闪】结算完毕后摸一张牌。
        3 => {
            let tagged = potfuji_event_has_tagged_source(tx, card, cards);
            let events = if tagged && potfuji_event_kind(tx, player, card) == Some(CardKind::Shan) {
                draw_cards(tx, player, 1)?
            } else {
                Vec::new()
            };
            if tagged {
                let mut sources = cards.to_vec();
                sources.push(card);
                sources.sort_by_key(|source| source.raw());
                sources.dedup();
                for source in sources {
                    tx.state_mut()
                        .remove_content_card_tag(source, crate::state::POTFUJI_CARD_TAG);
                }
            }
            Ok(events)
        }
        // 最少手牌奖励：下一张【杀】获得同花色牌并令基础伤害 +1。
        4 => {
            if !potfuji_event_kind(tx, player, card).is_some_and(CardKind::is_sha) {
                return Ok(Vec::new());
            }
            let mut events = gain_potfuji_matching_suit_cards(tx, player, card, &[], false)?;
            events.extend(run_effects(
                tx,
                vec![EffectRequest::IncreaseCurrentShaBaseDamage {
                    player,
                    card,
                    amount: 1,
                    skill,
                }],
            )?);
            Ok(events)
        }
        // 最少手牌奖励：下一张【闪】先获得同花色牌。
        5 => {
            if potfuji_event_kind(tx, player, card) == Some(CardKind::Shan) {
                gain_potfuji_matching_suit_cards(tx, player, card, &[], false)
            } else {
                Ok(Vec::new())
            }
        }
        // 对应【杀】结算完毕后移除一次性效果。
        6 => {
            if potfuji_event_kind(tx, player, card).is_some_and(CardKind::is_sha) {
                remove_owned_skill_def(tx, player, crate::state::POTFUJI_SHA_DEF)
            } else {
                Ok(Vec::new())
            }
        }
        // 对应【闪】结算完毕后摸一张并移除一次性效果。
        7 => {
            if potfuji_event_kind(tx, player, card) != Some(CardKind::Shan) {
                return Ok(Vec::new());
            }
            let mut events = draw_cards(tx, player, 1)?;
            events.extend(remove_owned_skill_def(
                tx,
                player,
                crate::state::POTFUJI_SHAN_DEF,
            )?);
            Ok(events)
        }
        // 下个自己的回合开始时，两项未消费效果同时失效。
        8 => {
            let mut events = remove_owned_skill_def(tx, player, crate::state::POTFUJI_SHA_DEF)?;
            events.extend(remove_owned_skill_def(
                tx,
                player,
                crate::state::POTFUJI_SHAN_DEF,
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown 势于吉 action".into())),
    }
}

pub(crate) fn potfuji_event_kind(tx: &Transaction, player: PlayerId, card: CardId) -> Option<CardKind> {
    tx.state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
}

pub(crate) fn resolve_rehengjiang(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    action: u8,
    hand_limit_def: engine_api::SkillDefId,
    draw_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|role| role.alive)
            {
                return Ok(Vec::new());
            }
            let actor = tx.state().turn.current;
            if actor.is_none() || !tx.state().players.get(actor).is_some_and(|role| role.alive) {
                return Ok(Vec::new());
            }
            let events = run_effects(
                tx,
                vec![
                    EffectRequest::AddSkillUntilTurnEnd {
                        player: actor,
                        skill_def: hand_limit_def,
                    },
                    EffectRequest::AddSkillUntilTurnEnd {
                        player,
                        skill_def: draw_def,
                    },
                ],
            )?;
            let hand_skill = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .find(|instance| instance.owner == actor && instance.def == hand_limit_def)
                .map(|instance| instance.id)
                .ok_or_else(|| EffectError::EngineFault("横江手牌上限 helper 未添加".into()))?;
            let total = tx
                .state()
                .skills
                .get(hand_skill)
                .and_then(|instance| instance.state_blob.get(..4))
                .map(|raw| u32::from_le_bytes(raw.try_into().unwrap()))
                .unwrap_or(0)
                .saturating_add(1);
            tx.state_mut()
                .skills
                .get_mut(hand_skill)
                .unwrap()
                .state_blob = total.to_le_bytes().to_vec();

            let draw_skill = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .find(|instance| instance.owner == player && instance.def == draw_def)
                .map(|instance| instance.id)
                .ok_or_else(|| EffectError::EngineFault("横江摸牌 helper 未添加".into()))?;
            let activations = tx
                .state()
                .skills
                .get(draw_skill)
                .and_then(|instance| {
                    (instance.state_blob.len() == 12
                        && &instance.state_blob[..4] == b"RHJ1"
                        && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap())
                            == actor.raw())
                    .then(|| u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap()))
                })
                .unwrap_or(0)
                .saturating_add(1);
            tx.state_mut()
                .skills
                .get_mut(draw_skill)
                .unwrap()
                .state_blob = [
                b"RHJ1".as_slice(),
                &actor.raw().to_le_bytes(),
                &activations.to_le_bytes(),
            ]
            .concat();
            Ok(events)
        }
        1 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner != player || instance.def != draw_def || actor.is_none() {
                return Ok(Vec::new());
            }
            let Some((pending_actor, activations)) = (instance.state_blob.len() == 12
                && &instance.state_blob[..4] == b"RHJ1")
                .then(|| {
                    (
                        PlayerId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        )),
                        u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap()),
                    )
                })
            else {
                return Ok(Vec::new());
            };
            if pending_actor != actor || tx.state().turn.current != actor || activations == 0 {
                return Ok(Vec::new());
            }
            let helper_present = tx.state().skills.sorted_live().into_iter().any(|instance| {
                instance.owner == actor
                    && instance.def == hand_limit_def
                    && instance
                        .state_blob
                        .get(..4)
                        .is_some_and(|raw| u32::from_le_bytes(raw.try_into().unwrap()) > 0)
            });
            let discarded = tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) > 0;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            draw_cards(
                tx,
                player,
                if helper_present && !discarded {
                    activations
                } else {
                    1
                },
            )
        }
        _ => Err(EffectError::ContentFault("unknown 横江 action".into())),
    }
}

pub(crate) fn potfuji_event_has_tagged_source(tx: &Transaction, card: CardId, event_cards: &[CardId]) -> bool {
    tx.state()
        .card_has_content_tag(card, crate::state::POTFUJI_CARD_TAG)
        || event_cards.iter().any(|source| {
            tx.state()
                .card_has_content_tag(*source, crate::state::POTFUJI_CARD_TAG)
        })
}

pub(crate) fn gain_potfuji_matching_suit_cards(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    event_cards: &[CardId],
    tagged_only: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut sources = if event_cards.is_empty() {
        vec![card]
    } else {
        event_cards.to_vec()
    };
    sources.sort_by_key(|card| card.raw());
    sources.dedup();
    if tagged_only {
        sources.retain(|source| {
            tx.state()
                .card_has_content_tag(*source, crate::state::POTFUJI_CARD_TAG)
        });
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("hand zone missing".into()))?;
    let mut chosen = Vec::new();
    for source in sources {
        let Some(suit) = tx.state().cards.get(source).map(|card| card.suit) else {
            continue;
        };
        if let Some(gain) = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|candidate| {
                !chosen.contains(candidate)
                    && tx
                        .state()
                        .cards
                        .get(*candidate)
                        .is_some_and(|card| card.suit == suit)
            })
        }) {
            chosen.push(gain);
        }
    }
    if chosen.is_empty() {
        Ok(Vec::new())
    } else {
        move_cards(tx, chosen, hand, Some(draw))
    }
}

pub(crate) fn remove_owned_skill_def(
    tx: &mut Transaction,
    player: PlayerId,
    def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == player && skill.def.raw() == def)
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects)
}

pub(crate) fn refund_current_sha_use_and_draw_if_red(
    tx: &mut Transaction,
    player: PlayerId,
    sha_user: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|value| value.alive)
        || !tx
            .state()
            .players
            .get(sha_user)
            .is_some_and(|value| value.alive)
    {
        return Err(EffectError::ContentFault(
            "Sha use refund requires living skill owner and card user".into(),
        ));
    }
    let is_red_sha = tx.state().cards.get(card).is_some_and(|value| {
        crate::query::effective_card_kind(tx.state(), sha_user, value).is_sha()
            && matches!(
                crate::query::effective_card_suit(tx.state(), sha_user, value),
                Suit::Heart | Suit::Diamond
            )
    });
    crate::command::play::refund_saved_sha_use(tx, sha_user, card)?;
    if is_red_sha {
        draw_cards(tx, player, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_discard_one_for_sha_refund_and_draw_if_red(
    tx: &mut Transaction,
    player: PlayerId,
    sha_user: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|value| value.alive)
    {
        return Ok(Vec::new());
    }
    crate::command::play::validate_saved_sha_use(tx, sha_user, card)?;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
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
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed,
        vec![player.raw(), sha_user.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
    tx: &mut Transaction,
    player: PlayerId,
    sha_user: PlayerId,
    card: CardId,
    restore_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|value| value.alive)
    {
        return Ok(Vec::new());
    }
    crate::command::play::validate_saved_sha_use(tx, sha_user, card)?;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
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
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed,
        vec![
            player.raw(),
            sha_user.raw(),
            card.raw(),
            skill.raw(),
            restore_skill_def.raw(),
        ],
    )
}

pub(crate) fn turn_over(tx: &mut Transaction, player: PlayerId) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !participant.alive {
        return Err(EffectError::Illegal("dead player cannot turn over".into()));
    }
    if crate::query::turn_over_cancelled(tx.state(), player) {
        return Ok(Vec::new());
    }
    if !participant.turned_over && crate::query::turn_face_down_cancelled(tx.state(), player) {
        return Ok(Vec::new());
    }
    let turned_over = {
        let player = tx
            .state_mut()
            .players
            .get_mut(player)
            .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
        player.turned_over = !player.turned_over;
        player.turned_over
    };
    Ok(vec![DomainEvent::PlayerTurnedOver {
        player,
        turned_over,
    }])
}

pub(crate) fn restore_character_state(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !participant.alive {
        return Err(EffectError::Illegal(
            "dead player character state cannot be restored".into(),
        ));
    }
    let linked = participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0);
    let turned_over = participant.turned_over;
    let mut events = Vec::new();
    if linked {
        events.extend(remove_mark(tx, player, crate::content::MARK_CHAIN, 0)?);
    }
    if turned_over {
        events.extend(turn_over(tx, player)?);
    }
    Ok(events)
}

pub(crate) fn gain_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    if let Some((from, card)) = first_shared_pile_card_by_kind(tx, kind)? {
        return move_cards(tx, vec![card], hand, Some(from));
    }
    Ok(Vec::new())
}

pub(crate) fn first_shared_pile_card_by_kind(
    tx: &Transaction,
    kind: CardKind,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == kind)
            })
        }) {
            return Ok(Some((from, card)));
        }
    }
    Ok(None)
}

pub(crate) fn gain_equipment_from_draw_pile(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let draw_pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let equipment = tx.state().zones.get(draw_pile).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_equip())
        })
    });
    let Some(equipment) = equipment else {
        return Ok(Vec::new());
    };
    move_cards(tx, vec![equipment], hand, Some(draw_pile))
}

pub(crate) fn give_all_hand_recover_to_one(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    ready_to_recover: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if player == target || !owner_alive || !target_alive {
        return Ok(Vec::new());
    }
    if ready_to_recover {
        let amount = tx
            .state()
            .players
            .get(target)
            .map(|participant| (1_i32 - participant.hp).max(0) as u32)
            .unwrap_or(0);
        return if amount == 0 {
            Ok(Vec::new())
        } else {
            recover_hp(tx, player, target, amount)
        };
    }
    let events = give_all_hand(tx, player, target)?;
    if events.is_empty() {
        return Ok(events);
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::GiveAllHandRecoverToOne {
            player,
            target,
            ready_to_recover: true,
        });
    Ok(events)
}

pub(crate) fn give_selected_recover_after_phase_total_two(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
    ready_to_recover: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_recover {
        return if tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            recover_hp(tx, player, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || cards.is_empty()
    {
        return Err(EffectError::Illegal("invalid Rende gift context".into()));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Rende owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Rende target has no hand".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && state.zone == from)
    }) {
        return Err(EffectError::Illegal(
            "Rende cards must be distinct current hand cards".into(),
        ));
    }
    let previous = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.play_phase_fire_count)
        .ok_or_else(|| EffectError::Illegal("Rende skill is missing".into()))?;
    let current = previous.saturating_add(cards.len() as u32);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated Rende skill")
        .play_phase_fire_count = current;
    if previous < 2 && current >= 2 {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
                ready_to_recover: true,
            },
        );
    }
    give_cards(tx, cards, to, from)
}

pub(crate) fn choose_other_hand_targets_gain_one_replace_draw(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
    reduce_from_draw_count: Option<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let fixed_draw_count = reduce_from_draw_count
        .map(|count| count.saturating_sub(targets.len() as u32))
        .unwrap_or(0);
    let continuing =
        !targets.is_empty() && tx.state().turn.draw_phase_fixed_count() == Some(fixed_draw_count);
    if (!continuing && !originating_skill_is_enabled(tx, player, skill))
        || (!continuing && tx.state().turn.draw_phase_is_fixed())
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid Tuxi draw context".into()));
    }

    if targets.is_empty() {
        let draw_count = reduce_from_draw_count.map_or(0, |captured| {
            if captured == 0 {
                crate::query::draw_count_for(tx.state(), player).saturating_add(
                    tx.state()
                        .turn
                        .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS)
                        .max(0) as u32,
                )
            } else {
                captured
            }
        });
        if reduce_from_draw_count.is_some() && draw_count == 0 {
            return Ok(Vec::new());
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| target != player && tx.state().hand_count(target) > 0)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let stage = if reduce_from_draw_count.is_some() {
            2
        } else {
            0
        };
        let mut payload = vec![stage, player.raw(), skill.raw()];
        if reduce_from_draw_count.is_some() {
            payload.push(draw_count);
        }
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            options
                .len()
                .min(if reduce_from_draw_count.is_some() {
                    draw_count as usize
                } else {
                    2
                })
                .min(u8::MAX as usize) as u8,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::TuxiGainOtherHandsReplaceDraw,
            payload,
        );
    }

    tx.state_mut()
        .turn
        .set_draw_phase_fixed_count(fixed_draw_count);

    if selected.is_some() {
        let target = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("Tuxi target index changed".into()))?;
        let from = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("Tuxi target hand is missing".into()))?;
        if !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| card.owner == target && card.zone == from)
        {
            return Err(EffectError::Illegal(
                "Tuxi selection is not a current target hand card".into(),
            ));
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Tuxi owner hand is missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            if let Some(draw_count) = reduce_from_draw_count {
                EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                    player,
                    targets,
                    index: index.saturating_add(1),
                    selected: CardId::NONE,
                    draw_count,
                    skill,
                }
            } else {
                EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                    player,
                    targets,
                    index: index.saturating_add(1),
                    selected: CardId::NONE,
                    skill,
                }
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(target) = targets.get(index as usize).copied() {
        let mut options = tx.state().hand_cards(target);
        options.sort_by_key(|card| card.raw());
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let stage = if reduce_from_draw_count.is_some() {
            3
        } else {
            1
        };
        let mut payload = vec![stage, player.raw(), skill.raw()];
        if let Some(draw_count) = reduce_from_draw_count {
            payload.push(draw_count);
        }
        payload.extend([index, targets.len() as u32]);
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::TuxiGainOtherHandsReplaceDraw,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn gain_one_hand_from_selected_targets(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid Cuorui play context".into()));
    }

    if index == 0 && selected.is_none() {
        let hp = tx
            .state()
            .players
            .get(player)
            .map(|participant| participant.hp.max(0) as usize)
            .unwrap_or(0);
        let mut seen = std::collections::BTreeSet::new();
        if targets.is_empty()
            || targets.len() > hp
            || targets.iter().any(|&target| {
                target == player
                    || !seen.insert(target.raw())
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                    || tx.state().hand_count(target) == 0
            })
        {
            return Err(EffectError::Illegal("invalid Cuorui targets".into()));
        }
    }

    if selected.is_some() {
        let target = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("Cuorui target index changed".into()))?;
        let from = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("Cuorui target hand is missing".into()))?;
        if !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| card.owner == target && card.zone == from)
        {
            return Err(EffectError::Illegal(
                "Cuorui selection is not a current target hand card".into(),
            ));
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Cuorui owner hand is missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::GainOneHandFromSelectedTargets {
                player,
                targets,
                index: index.saturating_add(1),
                selected: CardId::NONE,
                skill,
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(target) = targets.get(index as usize).copied() {
        let mut options = tx.state().hand_cards(target);
        options.sort_by_key(|card| card.raw());
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let mut payload = vec![player.raw(), skill.raw(), index, targets.len() as u32];
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::GainOneHandFromSelectedTargets,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn distribute_top_two_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    selected: Vec<CardId>,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !ready_to_choose {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Err(EffectError::Illegal("invalid Yiji skill context".into()));
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("Yiji draw pile is missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < 2
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("Yiji draw pile disappeared".into()))?;
        let take = zone.cards.len().min(2);
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("Yiji processing zone is missing".into()))?;
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::DistributeTopTwoCards {
                player,
                cards: cards.clone(),
                selected: Vec::new(),
                target: PlayerId::NONE,
                skill,
                ready_to_choose: true,
            },
        );
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }

    if cards.is_empty() {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Yiji processing zone is missing".into()))?;
    if cards.iter().any(|card| {
        !tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.zone == processing)
    }) {
        return Err(EffectError::Illegal(
            "Yiji remaining cards left processing".into(),
        ));
    }

    if selected.is_empty() {
        if cards.len() == 1 {
            return distribute_top_two_cards(
                tx,
                player,
                cards.clone(),
                cards,
                PlayerId::NONE,
                skill,
                true,
            );
        }
        let mut payload = vec![0, player.raw(), skill.raw(), cards.len() as u32];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            cards.len() as u8,
            false,
            cards.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::YijiDistributeTopCards,
            payload,
        );
    }

    let mut seen = std::collections::BTreeSet::new();
    if selected.is_empty()
        || selected
            .iter()
            .any(|card| !seen.insert(card.raw()) || !cards.contains(card))
    {
        return Err(EffectError::Illegal(
            "Yiji subset must contain distinct remaining cards".into(),
        ));
    }
    if target.is_none() {
        let options = tx.state().players.alive_by_seat();
        let mut payload = vec![1, player.raw(), skill.raw(), cards.len() as u32];
        payload.extend(cards.iter().map(|card| card.raw()));
        payload.push(selected.len() as u32);
        payload.extend(selected.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::YijiDistributeTopCards,
            payload,
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Yiji recipient is no longer alive".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Yiji recipient hand is missing".into()))?;
    let remaining: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| !selected.contains(card))
        .collect();
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::DistributeTopTwoCards {
            player,
            cards: remaining,
            selected: Vec::new(),
            target: PlayerId::NONE,
            skill,
            ready_to_choose: true,
        },
    );
    move_cards(tx, selected, to, Some(processing))
}

pub(crate) fn give_all_hand(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if player == target || !owner_alive || !target_alive {
        return Ok(Vec::new());
    }
    let Some(from) = tx.state().zones.find(ZoneKind::Hand, player) else {
        return Ok(Vec::new());
    };
    let cards = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let Some(to) = tx.state().zones.find(ZoneKind::Hand, target) else {
        return Ok(Vec::new());
    };
    give_cards(tx, cards, to, from)
}

pub(crate) fn choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_check: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !owner_alive {
        return Ok(Vec::new());
    }
    if ready_to_check {
        let Some(target_state) = tx.state().players.get(target) else {
            return Ok(Vec::new());
        };
        let hand_count = i32::try_from(tx.state().hand_count(target)).unwrap_or(i32::MAX);
        return if hand_count != target_state.hp {
            super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                }],
            )
            .map(|_| Vec::new())
        } else {
            Ok(Vec::new())
        };
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| {
                !crate::query::discardable_target_cards(
                    tx.state(),
                    target,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
            })
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp,
            vec![0, player.raw(), skill.raw()],
        );
    }
    let options = crate::query::discardable_target_cards(
        tx.state(),
        target,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || options.is_empty()
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp,
        vec![1, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_give_one_hand_equip_or_recover(
    tx: &mut Transaction,
    owner: PlayerId,
    giver: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    let target_dying =
        tx.state().players.get(target).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        });
    if !originating_skill_is_enabled(tx, owner, skill)
        || !alive(owner)
        || !alive(giver)
        || !target_dying
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, giver);
    let equip = tx.state().zones.find(ZoneKind::Equip, giver);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.is_empty() {
        return recover_hp(tx, PlayerId::NONE, target, 1);
    }

    let mut payload = vec![
        owner.raw(),
        giver.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        giver,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveOneHandEquipOrRecover,
        payload,
    )
}

pub(crate) fn choose_give_two_different_colors_or_recover(
    tx: &mut Transaction,
    owner: PlayerId,
    giver: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    let target_dying =
        tx.state().players.get(target).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        });
    if !originating_skill_is_enabled(tx, owner, skill)
        || !alive(owner)
        || !alive(giver)
        || !target_dying
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, giver);
    let equip = tx.state().zones.find(ZoneKind::Equip, giver);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    let has_valid_pair = options.iter().enumerate().any(|(index, first)| {
        let color = crate::query::effective_card_color_mask(tx.state(), giver, *first);
        options[index + 1..].iter().any(|second| {
            crate::query::effective_card_color_mask(tx.state(), giver, *second) != color
        })
    });
    if !has_valid_pair {
        return recover_hp(tx, PlayerId::NONE, target, 1);
    }

    let mut payload = vec![
        owner.raw(),
        giver.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        giver,
        2,
        2,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover,
        payload,
    )
}

pub(crate) fn resolve_mbzhuguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |id| {
        tx.state()
            .players
            .get(id)
            .is_some_and(|participant| participant.alive)
    };
    if !originating_skill_is_enabled(tx, player, skill) || !alive(player) || !alive(target) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let limit = tx
                .state()
                .players
                .get(target)
                .map_or(0, |participant| participant.max_hp.min(5));
            let hand_count = tx.state().hand_count(target);
            if hand_count < limit {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbzhuguo {
                        player,
                        target,
                        stage: 1,
                        skill,
                    },
                );
                return draw_cards(tx, target, limit - hand_count);
            }
            if hand_count > limit {
                let hand = tx.state().zones.find(ZoneKind::Hand, target);
                let mut options: Vec<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .filter(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| Some(card.zone) == hand)
                        })
                        .collect();
                options.sort_by_key(|card| card.raw());
                let need = usize::try_from(hand_count - limit).unwrap_or(usize::MAX);
                if options.len() >= need {
                    let mut payload = vec![
                        0,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        need as u32,
                        options.len() as u32,
                    ];
                    payload.extend(options.iter().map(|card| card.raw()));
                    return open_choice(
                        tx,
                        target,
                        u8::try_from(need).unwrap_or(u8::MAX),
                        u8::try_from(need).unwrap_or(u8::MAX),
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::ResolveMbzhuguo,
                        payload,
                    );
                }
            }
            recover_hp(tx, PlayerId::NONE, target, 1)
        }
        1 => {
            let target_hand = tx.state().hand_count(target);
            let maximum = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|id| tx.state().hand_count(id))
                .max()
                .unwrap_or(0);
            if target_hand < maximum {
                return Ok(Vec::new());
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player && *candidate != target)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                candidates.len() as u32,
            ];
            payload.extend(candidates.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbzhuguo,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid mbzhuguo effect stage".into(),
        )),
    }
}

pub(crate) fn resolve_mbanxian(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
            let mut options: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        card_state.zone == discard
                            && crate::query::effective_card_kind(tx.state(), player, card_state)
                                .is_sha()
                    })
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            options.dedup();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbanxian,
                payload,
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx.state().cards.get(card).is_some_and(|card_state| {
                Some(card_state.zone) == hand
                    && crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
            }) {
                return Ok(Vec::new());
            }
            let legal = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
            if legal.is_empty() {
                return Ok(Vec::new());
            }
            let (min_count, max_count) = if crate::query::sha_targets_all_for(tx.state(), player) {
                let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
                (count, count)
            } else {
                (
                    1,
                    u8::try_from(
                        legal
                            .len()
                            .min(crate::query::sha_target_limit_for(tx.state(), player) as usize),
                    )
                    .unwrap_or(u8::MAX),
                )
            };
            let mut payload = vec![1, player.raw(), card.raw(), skill.raw(), legal.len() as u32];
            payload.extend(legal.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                min_count,
                max_count,
                false,
                legal.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbanxian,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid mbanxian effect stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbjinzu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 | 1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive && !participant.dying)
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || helper_skill_def.is_none()
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.def == helper_skill_def)
            {
                return Err(EffectError::Illegal(
                    "invalid mbjinzu active context".into(),
                ));
            }
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || card.is_none()
                || !tx.state().cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
                })
            {
                return Err(EffectError::Illegal(
                    "invalid mbjinzu next-Sha context".into(),
                ));
            }
        }
        3 | 4 => {
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
            {
                return Ok(Vec::new());
            }
        }
        _ => {
            return Err(EffectError::ContentFault(
                "invalid mbjinzu effect stage".into(),
            ));
        }
    }

    match stage {
        0 => {
            let [owner_card] = cards.as_slice() else {
                return Err(EffectError::Illegal(
                    "mbjinzu requires one selected owner hand card".into(),
                ));
            };
            let owner_hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx.state().cards.get(*owner_card).is_some_and(|card_state| {
                Some(card_state.zone) == owner_hand && card_state.owner == player
            }) {
                return Err(EffectError::Illegal("mbjinzu owner card left hand".into()));
            }
            let target_cards = tx.state().hand_cards(target);
            if target_cards.len() <= 2 {
                let mut revealed = vec![*owner_card];
                revealed.extend(target_cards);
                return resolve_mbjinzu(
                    tx,
                    player,
                    target,
                    revealed,
                    CardId::NONE,
                    helper_skill_def,
                    1,
                    skill,
                );
            }
            let mut payload = vec![
                0,
                player.raw(),
                target.raw(),
                owner_card.raw(),
                helper_skill_def.raw(),
                skill.raw(),
                target_cards.len() as u32,
            ];
            payload.extend(target_cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                2,
                2,
                false,
                target_cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbjinzu,
                payload,
            )
        }
        1 => {
            let Some(owner_card) = cards.first().copied() else {
                return Err(EffectError::Illegal(
                    "mbjinzu reveal snapshot is empty".into(),
                ));
            };
            let target_cards = &cards[1..];
            let owner_hand = tx.state().zones.find(ZoneKind::Hand, player);
            let target_hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx.state().cards.get(owner_card).is_some_and(|card_state| {
                Some(card_state.zone) == owner_hand && card_state.owner == player
            }) || target_cards.len() > 2
                || (target_cards.is_empty() && tx.state().hand_count(target) != 0)
                || target_cards.iter().any(|card| {
                    !tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == target_hand && card_state.owner == target
                    })
                })
            {
                return Err(EffectError::Illegal(
                    "mbjinzu reveal context changed".into(),
                ));
            }
            let owner_number = tx
                .state()
                .cards
                .get(owner_card)
                .map(|card| crate::query::effective_card_number(tx.state(), player, card))
                .ok_or_else(|| EffectError::Illegal("mbjinzu owner card missing".into()))?;
            let target_numbers: Vec<_> = target_cards
                .iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .map(|card| crate::query::effective_card_number(tx.state(), player, card))
                })
                .collect();
            let (extreme, grant_helper) = if target_numbers.is_empty() {
                (true, true)
            } else {
                let min = target_numbers
                    .iter()
                    .copied()
                    .chain(std::iter::once(owner_number))
                    .min()
                    .unwrap_or(owner_number);
                let max = target_numbers
                    .iter()
                    .copied()
                    .chain(std::iter::once(owner_number))
                    .max()
                    .unwrap_or(owner_number);
                (
                    owner_number == min || owner_number == max,
                    target_numbers.iter().any(|number| *number <= owner_number)
                        && target_numbers.iter().any(|number| *number >= owner_number),
                )
            };
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            }];
            if !extreme {
                events.extend(add_mbjinzu_helper(tx, player, target, helper_skill_def)?);
                return Ok(events);
            }

            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbjinzu {
                    player,
                    target,
                    cards: Vec::new(),
                    card: CardId::NONE,
                    helper_skill_def,
                    stage: if grant_helper { 4 } else { 3 },
                    skill,
                });
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
            events.extend(move_cards_with_reason(
                tx,
                vec![owner_card],
                discard,
                owner_hand,
                CardMoveReason::Discard,
            )?);
            if !target_cards.is_empty() {
                events.extend(move_cards_with_reason(
                    tx,
                    target_cards.to_vec(),
                    discard,
                    target_hand,
                    CardMoveReason::Discard,
                )?);
            }
            Ok(events)
        }
        2 => {
            let targets = crate::command::play::saved_sha_targets(tx, player, card)?;
            let bonuses = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .map(|instance| instance.take_mbjinzu_targets())
                .unwrap_or_default();
            let mut events = Vec::new();
            for (target, amount) in bonuses {
                if !targets.contains(&target) {
                    continue;
                }
                events.extend(make_current_sha_target_unrespondable(
                    tx, player, card, target,
                )?);
                events.extend(increase_current_sha_target_damage(
                    tx, player, card, target, amount,
                )?);
            }
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        3 | 4 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.use_count = 0;
                instance.play_phase_use_count = 0;
            }
            if stage == 4 {
                add_mbjinzu_helper(tx, player, target, helper_skill_def)
            } else {
                Ok(Vec::new())
            }
        }
        _ => unreachable!(),
    }
}

pub(crate) fn gain_discard_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let card = tx.state().zones.get(discard).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == kind)
        })
    });
    card.map_or_else(
        || Ok(Vec::new()),
        |card| move_cards(tx, vec![card], hand, Some(discard)),
    )
}

pub(crate) fn choose_gain_discard_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let options: Vec<_> = tx
        .state()
        .zones
        .get(discard)
        .into_iter()
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == kind)
        })
        .collect();
    if options.is_empty() {
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
        ContinuationKind::ChooseGainDiscardCardByKind,
        vec![player.raw(), u32::from(kind.tag()), skill.raw()],
    )
}

pub(crate) fn choose_gain_own_lost_discard_card_this_turn(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(owner) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("bad player".into()));
    };
    let loss_count = tx.state().turn.card_loss_count(player);
    if !originating_skill_is_enabled(tx, player, skill)
        || !owner.alive
        || tx.state().turn.current != player
        || loss_count == 0
        || loss_count > owner.hp.max(0) as u32
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let options: Vec<_> = tx
        .state()
        .turn
        .lost_cards_for(player)
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == discard)
        })
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn,
        vec![player.raw(), skill.raw()],
    )
}
