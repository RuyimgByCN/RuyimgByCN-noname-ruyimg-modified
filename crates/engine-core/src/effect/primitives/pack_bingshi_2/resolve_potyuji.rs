//! Effect primitives for `resolve_potlvyi` and related helpers.

use super::*;

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

pub(crate) fn potfuji_event_kind(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Option<CardKind> {
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

pub(crate) fn potfuji_event_has_tagged_source(
    tx: &Transaction,
    card: CardId,
    event_cards: &[CardId],
) -> bool {
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
