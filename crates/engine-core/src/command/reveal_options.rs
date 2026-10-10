//! Resumable reveal → discard-by-suit gain and/or use-one-card effect.

use crate::command::play::{self, SkillCardTargetSpec};
use crate::effect::{
    add_mark, draw_cards, move_cards, move_one_card, open_choice, reshuffle_discard_into_draw,
    run_effects, EffectError,
};
use crate::state::{ContinuationKind, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardId, ChoiceKind, DomainEvent, MarkDefId, PlayerId};
use rule_api::EffectRequest;

const DISCARD_DONE: u8 = 1;
const USE_DONE: u8 = 2;
const STEP_CHOOSE: u8 = 0;
const STEP_DISCARD_SUIT: u8 = 1;
const STEP_CLEANUP: u8 = 2;

const STAGE_ORDER: u32 = 1;
const STAGE_DISCARD: u32 = 2;
const STAGE_RECIPIENT: u32 = 3;
const STAGE_USE_CARD: u32 = 4;
const STAGE_TARGETS: u32 = 5;
const STAGE_TARGET_PAIR: u32 = 6;

#[derive(Debug, Clone)]
struct Progress {
    player: PlayerId,
    mark: MarkDefId,
    base_count: u8,
    max_mark: u8,
    completed: u8,
    revealed: Vec<CardId>,
}

impl Progress {
    fn effect(&self, completed: u8, step: u8, data: u32) -> EffectRequest {
        EffectRequest::RevealDiscardSuitGainOrUse {
            player: self.player,
            mark: self.mark,
            base_count: self.base_count,
            max_mark: self.max_mark,
            revealed: self.revealed.clone(),
            completed,
            step,
            data,
        }
    }

    fn payload(&self, stage: u32, extra: &[u32]) -> Vec<u32> {
        let mut payload = vec![
            stage,
            self.player.raw(),
            self.mark.raw(),
            u32::from(self.base_count),
            u32::from(self.max_mark),
            u32::from(self.completed),
            self.revealed.len() as u32,
        ];
        payload.extend(self.revealed.iter().map(|card| card.raw()));
        payload.extend_from_slice(extra);
        payload
    }
}

fn parse_payload(payload: &[u32]) -> Result<(u32, Progress, &[u32]), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "short reveal-option continuation".into(),
        ));
    }
    let len = payload[6] as usize;
    if payload.len() < 7 + len {
        return Err(EffectError::EngineFault(
            "truncated reveal-option continuation".into(),
        ));
    }
    let progress = Progress {
        player: PlayerId::new(payload[1]),
        mark: MarkDefId::new(payload[2]),
        base_count: u8::try_from(payload[3])
            .map_err(|_| EffectError::EngineFault("bad reveal-option base count".into()))?,
        max_mark: u8::try_from(payload[4])
            .map_err(|_| EffectError::EngineFault("bad reveal-option mark cap".into()))?,
        completed: u8::try_from(payload[5])
            .map_err(|_| EffectError::EngineFault("bad reveal-option bitset".into()))?,
        revealed: payload[7..7 + len]
            .iter()
            .copied()
            .map(CardId::new)
            .collect(),
    };
    Ok((payload[0], progress, &payload[7 + len..]))
}

fn mark_count(tx: &Transaction, player: PlayerId, mark: MarkDefId) -> u8 {
    tx.state()
        .players
        .get(player)
        .and_then(|player| {
            player
                .marks
                .iter()
                .find(|(id, _)| *id == mark.raw())
                .map(|(_, count)| *count)
        })
        .unwrap_or(0)
        .clamp(0, i32::from(u8::MAX)) as u8
}

fn processing_cards(tx: &Transaction, cards: &[CardId]) -> Vec<CardId> {
    let Some(processing) = tx.state().zones.find(ZoneKind::Processing, PlayerId::NONE) else {
        return Vec::new();
    };
    cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.zone == processing && state.owner.is_none())
        })
        .collect()
}

fn usable_cards(tx: &Transaction, progress: &Progress) -> Vec<CardId> {
    processing_cards(tx, &progress.revealed)
        .into_iter()
        .filter(|card| play::skill_card_target_spec(tx, progress.player, *card).is_ok())
        .collect()
}

/// Apply or resume the typed effect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    tx: &mut Transaction,
    player: PlayerId,
    mark: MarkDefId,
    base_count: u8,
    max_mark: u8,
    mut revealed: Vec<CardId>,
    completed: u8,
    step: u8,
    data: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let player_state = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad reveal-option player".into()))?;
    if !player_state.alive {
        return discard_remaining(
            tx,
            &Progress {
                player,
                mark,
                base_count,
                max_mark,
                completed,
                revealed,
            },
        );
    }

    let mut events = Vec::new();
    if revealed.is_empty() && completed == 0 && step == STEP_CHOOSE {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
        let count = base_count.saturating_add(mark_count(tx, player, mark).min(max_mark));
        for _ in 0..count {
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let Some(card) = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last().copied())
            else {
                break;
            };
            move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
            revealed.push(card);
        }
        if !revealed.is_empty() {
            events.push(DomainEvent::CardsMoved {
                cards: revealed.clone(),
                from: draw,
                to: processing,
                reason: engine_api::CardMoveReason::Unspecified,
            });
            events.push(DomainEvent::CardsRevealed {
                cards: revealed.clone(),
                to: Vec::new(),
            });
        }
    }

    let progress = Progress {
        player,
        mark,
        base_count,
        max_mark,
        completed,
        revealed,
    };
    let mut next = match step {
        STEP_CHOOSE => advance(tx, &progress)?,
        STEP_DISCARD_SUIT => after_discard(tx, &progress, data)?,
        STEP_CLEANUP => cleanup(tx, &progress)?,
        _ => {
            return Err(EffectError::EngineFault(format!(
                "unknown reveal-option step {step}"
            )))
        }
    };
    events.append(&mut next);
    Ok(events)
}

fn advance(tx: &mut Transaction, progress: &Progress) -> Result<Vec<DomainEvent>, EffectError> {
    if progress.completed == DISCARD_DONE | USE_DONE {
        return cleanup(tx, progress);
    }
    let can_discard =
        !crate::query::discardable_hand_equip_cards(tx.state(), progress.player).is_empty();
    let can_use = !usable_cards(tx, progress).is_empty();
    match progress.completed {
        0 if can_discard && can_use => open_choice(
            tx,
            progress.player,
            1,
            1,
            false,
            vec![DISCARD_DONE as u32, USE_DONE as u32],
            ChoiceKind::Custom,
            ContinuationKind::RevealDiscardSuitGainOrUse,
            progress.payload(STAGE_ORDER, &[]),
        ),
        0 if can_discard => open_discard(tx, progress, false),
        0 if can_use => open_use_card(tx, progress, false),
        DISCARD_DONE if can_use => open_use_card(tx, progress, true),
        USE_DONE if can_discard => open_discard(tx, progress, true),
        _ => cleanup(tx, progress),
    }
}

fn open_discard(
    tx: &mut Transaction,
    progress: &Progress,
    optional: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), progress.player);
    if options.is_empty() {
        return Err(EffectError::Illegal("no discardable card".into()));
    }
    open_choice(
        tx,
        progress.player,
        1,
        1,
        optional,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RevealDiscardSuitGainOrUse,
        progress.payload(STAGE_DISCARD, &[]),
    )
}

fn open_use_card(
    tx: &mut Transaction,
    progress: &Progress,
    optional: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = usable_cards(tx, progress);
    if options.is_empty() {
        return Err(EffectError::Illegal("no usable revealed card".into()));
    }
    open_choice(
        tx,
        progress.player,
        1,
        1,
        optional,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RevealDiscardSuitGainOrUse,
        progress.payload(STAGE_USE_CARD, &[]),
    )
}

fn after_discard(
    tx: &mut Transaction,
    progress: &Progress,
    suit_tag: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !(1..=4).contains(&suit_tag) {
        return Err(EffectError::EngineFault("bad discarded suit tag".into()));
    }
    let suit = crate::command::judge::suit_from_tag(suit_tag);
    let matches: Vec<_> = processing_cards(tx, &progress.revealed)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.suit == suit)
        })
        .collect();
    if matches.is_empty() {
        return advance(
            tx,
            &Progress {
                completed: progress.completed | DISCARD_DONE,
                ..progress.clone()
            },
        );
    }
    let options = tx.state().players.alive_by_seat();
    open_choice(
        tx,
        progress.player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::RevealDiscardSuitGainOrUse,
        progress.payload(STAGE_RECIPIENT, &[suit_tag]),
    )
}

fn cleanup(tx: &mut Transaction, progress: &Progress) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    if progress.completed == DISCARD_DONE | USE_DONE {
        if mark_count(tx, progress.player, progress.mark) < progress.max_mark {
            events.extend(add_mark(tx, progress.player, progress.mark.raw(), 1)?);
        }
        let hand_cards = tx.state().hand_cards(progress.player);
        if !hand_cards.is_empty() {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, progress.player)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let count = hand_cards.len() as u32;
            events.extend(move_cards(tx, hand_cards, discard, Some(hand))?);
            events.extend(draw_cards(tx, progress.player, count)?);
        }
    }

    events.extend(discard_remaining(tx, progress)?);
    Ok(events)
}

fn discard_remaining(
    tx: &mut Transaction,
    progress: &Progress,
) -> Result<Vec<DomainEvent>, EffectError> {
    let remaining = processing_cards(tx, &progress.revealed);
    if !remaining.is_empty() {
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
        return move_cards(tx, remaining, discard, Some(processing));
    }
    Ok(Vec::new())
}

/// Resolve one state-machine choice. The command pipeline has already authenticated the observer.
pub(crate) fn resolve_choice(
    tx: &mut Transaction,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (stage, progress, extra) = parse_payload(payload)?;
    match stage {
        STAGE_ORDER => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("must choose the first option".into()));
            }
            match selections[0] as u8 {
                DISCARD_DONE => {
                    open_discard(tx, &progress, false)?;
                    Err(EffectError::ChoiceRequested)
                }
                USE_DONE => {
                    open_use_card(tx, &progress, false)?;
                    Err(EffectError::ChoiceRequested)
                }
                _ => Err(EffectError::Illegal("bad reveal-option order".into())),
            }
        }
        STAGE_DISCARD => resolve_discard(tx, &progress, selections, cancelled),
        STAGE_RECIPIENT => resolve_recipient(tx, &progress, extra, selections, cancelled),
        STAGE_USE_CARD => resolve_use_card(tx, &progress, selections, cancelled),
        STAGE_TARGETS => resolve_targets(tx, &progress, extra, selections, cancelled),
        STAGE_TARGET_PAIR => resolve_target_pair(tx, &progress, extra, selections, cancelled),
        _ => Err(EffectError::EngineFault(format!(
            "unknown reveal-option stage {stage}"
        ))),
    }
}

fn resolve_discard(
    tx: &mut Transaction,
    progress: &Progress,
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        if progress.completed == 0 {
            return Err(EffectError::Illegal("first option is mandatory".into()));
        }
        return run_effects(
            tx,
            vec![progress.effect(progress.completed, STEP_CLEANUP, 0)],
        )
        .map(|_| ());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("discard exactly one card".into()));
    }
    let card = CardId::new(selections[0]);
    if !crate::query::discardable_hand_equip_cards(tx.state(), progress.player).contains(&card) {
        return Err(EffectError::Illegal("card is not discardable".into()));
    }
    let state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
    let suit = crate::query::effective_card_suit(tx.state(), progress.player, state);
    let suit_tag = crate::command::judge::suit_tag(suit);
    let from = state.zone;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(from),
            },
            progress.effect(progress.completed, STEP_DISCARD_SUIT, suit_tag),
        ],
    )
    .map(|_| ())
}

fn resolve_recipient(
    tx: &mut Transaction,
    progress: &Progress,
    extra: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("must choose one recipient".into()));
    }
    let target = PlayerId::new(selections[0]);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal("bad recipient".into()));
    }
    let suit_tag = *extra
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing discarded suit".into()))?;
    if !(1..=4).contains(&suit_tag) {
        return Err(EffectError::EngineFault("bad discarded suit".into()));
    }
    let suit = crate::command::judge::suit_from_tag(suit_tag);
    let cards: Vec<_> = processing_cards(tx, &progress.revealed)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.suit == suit)
        })
        .collect();
    let next = progress.effect(progress.completed | DISCARD_DONE, STEP_CHOOSE, 0);
    if cards.is_empty() {
        return run_effects(tx, vec![next]).map(|_| ());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards,
                to: hand,
                from: Some(processing),
            },
            next,
        ],
    )
    .map(|_| ())
}

fn resolve_use_card(
    tx: &mut Transaction,
    progress: &Progress,
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        if progress.completed == 0 {
            return Err(EffectError::Illegal("first option is mandatory".into()));
        }
        return run_effects(
            tx,
            vec![progress.effect(progress.completed, STEP_CLEANUP, 0)],
        )
        .map(|_| ());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "select exactly one revealed card".into(),
        ));
    }
    let card = CardId::new(selections[0]);
    if !usable_cards(tx, progress).contains(&card) {
        return Err(EffectError::Illegal("revealed card is not usable".into()));
    }
    match play::skill_card_target_spec(tx, progress.player, card)? {
        SkillCardTargetSpec::Immediate(targets) => use_card(tx, progress, card, &targets),
        SkillCardTargetSpec::Select { min, max, options } => {
            open_choice(
                tx,
                progress.player,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::RevealDiscardSuitGainOrUse,
                progress.payload(STAGE_TARGETS, &[card.raw()]),
            )?;
            Err(EffectError::ChoiceRequested)
        }
        SkillCardTargetSpec::Pairs(pairs) => {
            open_choice(
                tx,
                progress.player,
                1,
                1,
                false,
                (1..=pairs.len() as u32).collect(),
                ChoiceKind::Custom,
                ContinuationKind::RevealDiscardSuitGainOrUse,
                progress.payload(STAGE_TARGET_PAIR, &[card.raw()]),
            )?;
            Err(EffectError::ChoiceRequested)
        }
    }
}

fn resolve_targets(
    tx: &mut Transaction,
    progress: &Progress,
    extra: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "card target choice is mandatory".into(),
        ));
    }
    let card = CardId::new(
        *extra
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing revealed card".into()))?,
    );
    let SkillCardTargetSpec::Select { min, max, options } =
        play::skill_card_target_spec(tx, progress.player, card)?
    else {
        return Err(EffectError::Illegal("card target shape changed".into()));
    };
    if selections.len() < min as usize || selections.len() > max as usize {
        return Err(EffectError::Illegal("wrong card target count".into()));
    }
    let mut seen = std::collections::BTreeSet::new();
    let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if targets
        .iter()
        .any(|target| !seen.insert(*target) || !options.contains(target))
    {
        return Err(EffectError::Illegal("illegal card target".into()));
    }
    use_card(tx, progress, card, &targets)
}

fn resolve_target_pair(
    tx: &mut Transaction,
    progress: &Progress,
    extra: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("must choose one target pair".into()));
    }
    let card = CardId::new(
        *extra
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing revealed card".into()))?,
    );
    let SkillCardTargetSpec::Pairs(pairs) =
        play::skill_card_target_spec(tx, progress.player, card)?
    else {
        return Err(EffectError::Illegal("card target shape changed".into()));
    };
    let index = selections[0]
        .checked_sub(1)
        .map(|value| value as usize)
        .ok_or_else(|| EffectError::Illegal("bad target pair".into()))?;
    let (first, second) = pairs
        .get(index)
        .copied()
        .ok_or_else(|| EffectError::Illegal("bad target pair".into()))?;
    use_card(tx, progress, card, &[first, second])
}

fn use_card(
    tx: &mut Transaction,
    progress: &Progress,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        progress.effect(progress.completed | USE_DONE, STEP_CHOOSE, 0),
    );
    play::handle_skill_card(tx, progress.player, card, targets)
}
