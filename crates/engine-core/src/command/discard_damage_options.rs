//! Resumable discard-target and/or fire-damage option effect.

use crate::effect::{open_choice, run_effects, EffectError};
use crate::state::{turn_flag, ContinuationKind, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardId, ChoiceKind, DomainEvent, PlayerId};
use rule_api::{damage_nature, EffectRequest};
use std::collections::BTreeSet;

const DISCARD: u8 = 1;
const DAMAGE: u8 = 2;

const STEP_CHOOSE: u8 = 0;
const STEP_AFTER_DISCARD: u8 = 1;

const STAGE_OPTIONS: u32 = 1;
const STAGE_DISCARD_TARGET: u32 = 2;
const STAGE_DISCARD_CARDS: u32 = 3;
const STAGE_DAMAGE_TARGET: u32 = 4;

#[derive(Debug, Clone, Copy)]
struct Progress {
    player: PlayerId,
    max_discard: u8,
    damage: u8,
    selected: u8,
    target: PlayerId,
}

impl Progress {
    fn payload(self, stage: u32) -> Vec<u32> {
        vec![
            stage,
            self.player.raw(),
            u32::from(self.max_discard),
            u32::from(self.damage),
            u32::from(self.selected),
            self.target.raw(),
        ]
    }

    fn after_discard(self) -> EffectRequest {
        EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player: self.player,
            max_discard: self.max_discard,
            damage: self.damage,
            selected: self.selected,
            step: STEP_AFTER_DISCARD,
            target: self.target,
        }
    }
}

fn parse_payload(payload: &[u32]) -> Result<(u32, Progress), EffectError> {
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad discard/damage continuation".into(),
        ));
    }
    Ok((
        payload[0],
        Progress {
            player: PlayerId::new(payload[1]),
            max_discard: u8::try_from(payload[2])
                .map_err(|_| EffectError::EngineFault("bad max discard".into()))?,
            damage: u8::try_from(payload[3])
                .map_err(|_| EffectError::EngineFault("bad damage".into()))?,
            selected: u8::try_from(payload[4])
                .map_err(|_| EffectError::EngineFault("bad branch bitset".into()))?,
            target: PlayerId::new(payload[5]),
        },
    ))
}

fn discardable_cards(tx: &Transaction, actor: PlayerId, target: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(target);
    if let Some(equip) = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| tx.state().zones.get(zone))
    {
        cards.extend(equip.cards.iter().copied());
    }
    cards.retain(|card| {
        !crate::query::card_discard_protected_from(tx.state(), actor, target, *card)
    });
    cards
}

fn discard_targets(tx: &Transaction, player: PlayerId) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| !discardable_cards(tx, player, *target).is_empty())
        .collect()
}

fn branch_options(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    let mut options = Vec::with_capacity(2);
    if !discard_targets(tx, player).is_empty() {
        options.push(u32::from(DISCARD));
    }
    if !tx.state().players.alive_by_seat().is_empty() {
        options.push(u32::from(DAMAGE));
    }
    options
}

/// Apply or resume the typed effect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    tx: &mut Transaction,
    player: PlayerId,
    max_discard: u8,
    damage: u8,
    selected: u8,
    step: u8,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if max_discard == 0 || damage == 0 {
        return Err(EffectError::Illegal(
            "discard count and damage must be positive".into(),
        ));
    }
    let owner = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad discard/damage player".into()))?;
    if !owner.alive {
        return Ok(Vec::new());
    }
    if tx.state().turn.current != player {
        return Err(EffectError::Illegal(
            "phase-skip effect requires the current player".into(),
        ));
    }

    let progress = Progress {
        player,
        max_discard,
        damage,
        selected,
        target,
    };
    match step {
        STEP_CHOOSE if selected == 0 => {
            let options = branch_options(tx, player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                options.len() as u8,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::DiscardOrFireDamageSkipPhases,
                progress.payload(STAGE_OPTIONS),
            )
        }
        STEP_AFTER_DISCARD => {
            if tx.state().hand_count(target) <= tx.state().hand_count(player) {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
            }
            if selected & DAMAGE != 0 {
                open_damage_target(tx, progress)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::EngineFault(format!(
            "bad discard/damage step {step}"
        ))),
    }
}

fn open_discard_target(tx: &mut Transaction, progress: Progress) -> Result<(), EffectError> {
    let targets = discard_targets(tx, progress.player);
    if targets.is_empty() {
        return Err(EffectError::Illegal("no discard target".into()));
    }
    open_choice(
        tx,
        progress.player,
        1,
        1,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::DiscardOrFireDamageSkipPhases,
        progress.payload(STAGE_DISCARD_TARGET),
    )?;
    Err(EffectError::ChoiceRequested)
}

fn open_discard_cards(tx: &mut Transaction, progress: Progress) -> Result<(), EffectError> {
    let cards = discardable_cards(tx, progress.player, progress.target);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "discard target has no legal card".into(),
        ));
    }
    open_choice(
        tx,
        progress.player,
        1,
        progress.max_discard.min(cards.len() as u8),
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardOrFireDamageSkipPhases,
        progress.payload(STAGE_DISCARD_CARDS),
    )?;
    Err(EffectError::ChoiceRequested)
}

fn open_damage_target(
    tx: &mut Transaction,
    progress: Progress,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = tx.state().players.alive_by_seat();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        progress.player,
        1,
        1,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::DiscardOrFireDamageSkipPhases,
        progress.payload(STAGE_DAMAGE_TARGET),
    )
}

/// Resolve one state-machine choice.
pub(crate) fn resolve_choice(
    tx: &mut Transaction,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (stage, mut progress) = parse_payload(payload)?;
    match stage {
        STAGE_OPTIONS => {
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() || selections.len() > 2 {
                return Err(EffectError::Illegal("choose one or two branches".into()));
            }
            let legal: BTreeSet<_> = branch_options(tx, progress.player).into_iter().collect();
            let mut seen = BTreeSet::new();
            if selections
                .iter()
                .any(|selection| !legal.contains(selection) || !seen.insert(*selection))
            {
                return Err(EffectError::Illegal("illegal branch selection".into()));
            }
            progress.selected = selections
                .iter()
                .fold(0_u8, |bits, selection| bits | *selection as u8);
            if progress.selected & DISCARD != 0 {
                open_discard_target(tx, progress)
            } else {
                open_damage_target(tx, progress)?;
                Err(EffectError::ChoiceRequested)
            }
        }
        STAGE_DISCARD_TARGET => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("choose one discard target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !discard_targets(tx, progress.player).contains(&target) {
                return Err(EffectError::Illegal("illegal discard target".into()));
            }
            progress.target = target;
            open_discard_cards(tx, progress)
        }
        STAGE_DISCARD_CARDS => resolve_discard_cards(tx, progress, selections, cancelled),
        STAGE_DAMAGE_TARGET => resolve_damage_target(tx, progress, selections, cancelled),
        _ => Err(EffectError::EngineFault(format!(
            "bad discard/damage stage {stage}"
        ))),
    }
}

fn resolve_discard_cards(
    tx: &mut Transaction,
    progress: Progress,
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let legal: BTreeSet<_> = discardable_cards(tx, progress.player, progress.target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let mut seen = BTreeSet::new();
    if cancelled
        || selections.is_empty()
        || selections.len() > usize::from(progress.max_discard)
        || selections
            .iter()
            .any(|selection| !legal.contains(selection) || !seen.insert(*selection))
    {
        return Err(EffectError::Illegal("illegal discard cards".into()));
    }

    let cards = selections.iter().copied().map(CardId::new).collect();
    tx.state_mut()
        .resolution
        .deferred_effects
        .insert(0, progress.after_discard());
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: progress.target,
            cards,
        }],
    )
    .map(|_| ())
}

fn resolve_damage_target(
    tx: &mut Transaction,
    progress: Progress,
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("choose one damage target".into()));
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.alive_by_seat().contains(&target) {
        return Err(EffectError::Illegal("illegal damage target".into()));
    }
    let result = run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: progress.player,
            target,
            amount: u32::from(progress.damage),
            is_lose_hp: false,
            nature: damage_nature::FIRE,
        }],
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let player_hp = tx.state().players.get(progress.player).unwrap().hp;
    if target_hp <= player_hp {
        tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 1);
    }
    result.map(|_| ())
}
