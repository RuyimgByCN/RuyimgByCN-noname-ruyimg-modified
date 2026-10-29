//! Effect primitives for `suit_code` and related helpers.

use super::*;

pub(crate) fn suit_code(suit: Suit) -> u8 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
        Suit::None => 0,
    }
}

pub(crate) fn mbzhuji_state(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> Option<(u8, u32, u32, CardId)> {
    let blob = &tx.state().skills.get(skill)?.state_blob;
    if blob.len() != 17 || blob.get(..4) != Some(b"MZJ1") {
        return None;
    }
    Some((
        blob[4],
        u32::from_le_bytes(blob[5..9].try_into().unwrap()),
        u32::from_le_bytes(blob[9..13].try_into().unwrap()),
        CardId::new(u32::from_le_bytes(blob[13..17].try_into().unwrap())),
    ))
}

pub(crate) fn set_mbzhuji_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    suit: u8,
    equip_count: u32,
    discarded: u32,
    card: CardId,
) {
    let mut blob = b"MZJ1".to_vec();
    blob.push(suit);
    blob.extend_from_slice(&equip_count.to_le_bytes());
    blob.extend_from_slice(&discarded.to_le_bytes());
    blob.extend_from_slice(&card.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn mbzhuji_legal_suits(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    (1..=4)
        .filter(|suit| {
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        u32::from(suit_code(crate::query::effective_card_suit(
                            tx.state(),
                            player,
                            card,
                        ))) == *suit
                    })
                })
                .collect();
            !cards.is_empty() && cards.iter().all(|card| discardable.contains(card))
        })
        .collect()
}

pub(crate) fn first_shared_equip_of_suit(
    tx: &Transaction,
    player: PlayerId,
    suit: u8,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx
            .state()
            .zones
            .find(kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(zone).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), player, card).is_equip()
                        && suit_code(crate::query::effective_card_suit(tx.state(), player, card))
                            == suit
                })
            })
        }) {
            return Ok(Some((zone, card)));
        }
    }
    Ok(None)
}

pub(crate) fn resolve_mbzhuji(
    tx: &mut Transaction,
    player: PlayerId,
    choice: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = mbzhuji_legal_suits(tx, player);
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![MBZHUJI_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !mbzhuji_legal_suits(tx, player).contains(&u32::from(choice)) {
                return Err(EffectError::Illegal("筑墼花色不再合法".into()));
            }
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        suit_code(crate::query::effective_card_suit(tx.state(), player, card))
                            == choice
                    })
                })
                .collect();
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len().min(u32::MAX as usize) as u32);
            set_mbzhuji_state(
                tx,
                skill,
                choice,
                equip_count,
                cards.len().min(u32::MAX as usize) as u32,
                CardId::NONE,
            );
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbzhuji {
                    player,
                    choice: 0,
                    stage: 2,
                    skill,
                });
            discard_cards(tx, player, cards)
        }
        2 => {
            let Some((suit, equip_count, discarded, _)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            let Some((from, card)) = first_shared_equip_of_suit(tx, player, suit)? else {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
                return Ok(Vec::new());
            };
            set_mbzhuji_state(tx, skill, suit, equip_count, discarded, card);
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbzhuji {
                    player,
                    choice: 0,
                    stage: 3,
                    skill,
                });
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("筑墼手牌区不存在".into()))?;
            move_cards(tx, vec![card], hand, Some(from))
        }
        3 => {
            let Some((_, _, _, card)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tx
                .state()
                .cards
                .get(card)
                .is_some_and(|card| Some(card.zone) == hand)
            {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbzhuji {
                        player,
                        choice: 0,
                        stage: 4,
                        skill,
                    },
                );
                return match crate::command::play::use_equipment_from_hand(tx, player, card) {
                    Ok(()) => Ok(Vec::new()),
                    Err(EffectError::Illegal(_)) => {
                        tx.state_mut().resolution.deferred_tail_effects.pop();
                        resolve_mbzhuji(tx, player, 0, 4, skill)
                    }
                    Err(error) => Err(error),
                };
            }
            resolve_mbzhuji(tx, player, 0, 4, skill)
        }
        4 => {
            let Some((_, equip_count, discarded, _)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            if discarded < equip_count {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
                return Ok(Vec::new());
            }
            let participant = tx.state().players.get(player).unwrap();
            let mut options = vec![1];
            if participant.hp < participant.max_hp as i32 {
                options.push(2);
            }
            let hujia = participant
                .marks
                .iter()
                .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                .map_or(0, |(_, count)| (*count).max(0));
            if hujia < 5 {
                options.push(3);
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
                vec![MBZHUJI_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        5 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            match choice {
                1 => draw_cards(tx, player, 2),
                2 if tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|player| player.hp < player.max_hp as i32) =>
                {
                    recover_hp(tx, player, player, 1)
                }
                3 => {
                    let hujia = tx
                        .state()
                        .players
                        .get(player)
                        .and_then(|player| {
                            player
                                .marks
                                .iter()
                                .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                        })
                        .map_or(0, |(_, count)| (*count).max(0));
                    if hujia >= 5 {
                        return Err(EffectError::Illegal("护甲已达上限".into()));
                    }
                    add_mark(tx, player, crate::content::MARK_HUJIA, 1)
                }
                _ => Err(EffectError::Illegal("筑墼收益不再合法".into())),
            }
        }
        _ => Err(EffectError::ContentFault("bad 兵势孙韶筑墼 stage".into())),
    }
}

pub(crate) fn potlibing_state(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> (u32, u32, u32, u32) {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return (0, 0, 0, 0);
    };
    if blob.len() != 20 || &blob[4..8] != b"LIB1" {
        return (0, 0, 0, 0);
    }
    (
        u32::from_le_bytes(blob[0..4].try_into().unwrap()),
        u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        u32::from_le_bytes(blob[12..16].try_into().unwrap()),
        u32::from_le_bytes(blob[16..20].try_into().unwrap()),
    )
}

pub(crate) fn set_potlibing_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    attack: u32,
    step: u32,
    draws: u32,
    damage: u32,
) {
    let mut blob = attack.to_le_bytes().to_vec();
    blob.extend_from_slice(b"LIB1");
    blob.extend_from_slice(&step.to_le_bytes());
    blob.extend_from_slice(&draws.to_le_bytes());
    blob.extend_from_slice(&damage.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_potlibing(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return Ok(Vec::new());
    };
    let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
    let damage_card = kind.is_sha() || kind.is_damage_trick();
    let (attack, step, draws, damage) = potlibing_state(tx, skill);
    match stage {
        0 if damage_card => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        0 => match step {
            0 => {
                set_potlibing_state(tx, skill, attack.saturating_add(1), 1, draws, damage);
                Ok(Vec::new())
            }
            1 => {
                let count = draws.saturating_add(1);
                set_potlibing_state(tx, skill, attack, 2, count, damage);
                draw_cards(tx, player, count)
            }
            _ => {
                set_potlibing_state(tx, skill, attack, 0, draws, damage.saturating_add(1));
                Ok(Vec::new())
            }
        },
        1 if damage_card && damage > 0 => {
            set_potlibing_state(tx, skill, attack, step, draws, 0);
            if !matches!(kind, CardKind::TrickShandian) {
                crate::command::play::increase_saved_card_base_damage(tx, player, card, damage)?;
            }
            Ok(Vec::new())
        }
        1 => Ok(Vec::new()),
        _ => Err(EffectError::ContentFault("bad 兵势夏侯霸砺兵 stage".into())),
    }
}

pub(crate) fn resolve_potpoxi(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|player| player.alive && !player.dying)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    let mut cards: Vec<_> = cards
        .into_iter()
        .filter(|card| {
            seen.insert(card.raw())
                && tx.state().cards.get(*card).is_some_and(|state| {
                    state.zone == discard
                        && crate::query::effective_card_kind(tx.state(), player, state).is_sha()
                })
        })
        .collect();
    while let Some(card) = cards.first().copied() {
        cards.remove(0);
        let legal = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
        if legal.is_empty() {
            continue;
        }
        let (min, max) = if crate::query::sha_targets_all_for(tx.state(), player) {
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
        let mut payload = vec![
            POTPOXI_CUSTOM_CHOICE,
            player.raw(),
            card.raw(),
            skill.raw(),
            cards.len() as u32,
        ];
        payload.extend(cards.iter().map(|card| card.raw()));
        payload.push(legal.len() as u32);
        payload.extend(legal.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            min,
            max,
            false,
            legal.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn potfuan_state(tx: &Transaction, skill: engine_api::SkillId) -> PotfuanState {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return PotfuanState::default();
    };
    if blob.len() != 32 || &blob[..4] != b"FUA1" {
        return PotfuanState::default();
    }
    let read = |range: std::ops::Range<usize>| u32::from_le_bytes(blob[range].try_into().unwrap());
    PotfuanState {
        recorded: read(4..8),
        used_round: read(8..12),
        used: read(12..16),
        only: read(16..20),
        pending_card: CardId::new(read(20..24)),
        pending_target: PlayerId::new(read(24..28)),
        pending_first: read(28..32),
    }
}

pub(crate) fn set_potfuan_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    state: PotfuanState,
) {
    let mut blob = b"FUA1".to_vec();
    for value in [
        state.recorded,
        state.used_round,
        state.used,
        state.only,
        state.pending_card.raw(),
        state.pending_target.raw(),
        state.pending_first,
    ] {
        blob.extend_from_slice(&value.to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn suit_bit(suit: engine_api::Suit) -> u32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 4,
        engine_api::Suit::Diamond => 8,
        engine_api::Suit::None => 0,
    }
}

pub(crate) fn suit_tag_from_bit(bit: u32) -> Option<u32> {
    match bit {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
        _ => None,
    }
}

pub(crate) fn suit_bit_from_tag(tag: u32) -> Option<u32> {
    match tag {
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        4 => Some(8),
        _ => None,
    }
}

pub(crate) fn attack_range_value(tx: &Transaction, player: PlayerId) -> i32 {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::AttackRange { player },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(value) => value.max(0),
        _ => 0,
    }
}

pub(crate) fn find_skill_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Option<engine_api::SkillId> {
    tx.state()
        .players
        .get(player)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == def)
        })
}

pub(crate) fn potfuan_hand_suits(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state()
        .hand_cards(player)
        .into_iter()
        .filter_map(|card| tx.state().cards.get(card))
        .fold(0, |mask, card| {
            mask | suit_bit(crate::query::effective_card_suit(tx.state(), player, card))
        })
}

pub(crate) fn allocate_virtual_sha(tx: &mut Transaction) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    Ok(card)
}

pub(crate) fn remove_unused_virtual_card(
    tx: &mut Transaction,
    card: CardId,
) -> Result<(), EffectError> {
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::EngineFault("virtual card missing".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
    move_one_card(tx, card, from, removed, PlayerId::NONE, true)
}

pub(crate) fn potfuan_sha_targets(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    if crate::query::card_use_cancelled_for_card(tx.state(), player, card) {
        return Vec::new();
    }
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            crate::command::play::validate_sha_target_with_options(tx, player, card, *target, true)
                .is_ok()
        })
        .collect()
}
