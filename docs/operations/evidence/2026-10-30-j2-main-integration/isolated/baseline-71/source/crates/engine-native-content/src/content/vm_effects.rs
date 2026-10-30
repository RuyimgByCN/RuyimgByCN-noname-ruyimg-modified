//! Content-owned VM commit machines. The kernel only stores and pumps their continuations.
mod danshou;
mod fenghe;
mod pingkou;

use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;
use rule_api::EffectRequest;

pub(crate) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    match effect {
        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(fenghe::step(
                ctx,
                owner,
                *require_skill_def,
                *grant_skill_def,
                *grant_mark_def,
                state,
            ))
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player, amount, ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(pingkou::step(ctx, owner, *amount, state))
        }
        EffectRequest::ResolveOlJieDanshou { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(danshou::step(ctx, skill, owner, state))
        }
        _ => None,
    }
}

pub(crate) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match effect {
        EffectRequest::ChooseGiveOrRecastOneHand { player, .. } => {
            fenghe::active_ask(core, catalog, seat(*player, core)?, state)
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player, .. } => {
            pingkou::active_ask(core, seat(*player, core)?, state)
        }
        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            let owner = seat(*player, core)?;
            let name = catalog.skills.get(skill.raw() as usize)?.name;
            danshou::active_ask(core, owner, name, state)
        }
        _ => None,
    }
}

fn seat(player: rule_api::PlayerId, core: &GameStateCore) -> Option<u8> {
    let seat = player.raw().checked_sub(1)?;
    (seat < core.players.len() as u32).then_some(seat as u8)
}

fn cards(core: &GameStateCore, player: u8) -> Vec<ChoiceOption> {
    core.player(player)
        .hand
        .iter()
        .chain(&core.player(player).equips)
        .map(|c| ChoiceOption::Card(*c))
        .collect()
}

fn wait(ask: ExpectedAsk) -> Flow {
    Flow::WaitChoice {
        observer: ask.observer,
        spec: ChoiceSpec {
            kind: ask.kind,
            prompt: ask.prompt,
            options: ask.options,
            min: ask.min,
            max: ask.max,
            allow_cancel: ask.allow_cancel,
        },
    }
}

fn discard(ctx: &mut Ctx<'_>, owner: u8, picked: &[u32]) {
    for card in picked {
        if ctx.core.player(owner).hand.contains(card) {
            ctx.take_hand_card(owner, *card);
        } else {
            ctx.take_owned_card(owner, *card);
        }
    }
    ctx.core.discard.extend_from_slice(picked);
    ctx.out.push(crate::machine::LogEvent::CardDiscarded {
        player: owner,
        cards: picked.to_vec(),
    });
    ctx.emit(crate::event::EventData::CardsDiscarded {
        player: owner,
        cards: picked.to_vec(),
    });
}

/// Validate content continuation shapes before any machine can resume them.
pub(crate) fn validate(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    card_count: usize,
) -> bool {
    if state.is_empty() {
        return true;
    }
    let seats = core.players.len();
    match effect {
        EffectRequest::ResolveOlJieDanshou { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 3
                && (1..=7).contains(&state[0])
                && (state[0] == 1 || (1..=4).contains(&state[1]))
                && (state[0] <= 2 || (state[2] as usize) < seats)
        }
        EffectRequest::ChooseGiveOrRecastOneHand { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 3
                && (1..=5).contains(&state[0])
                && (state[0] == 1 || (state[1] as usize) < card_count)
                && state[2] as usize <= seats
                && (state[0] != 2
                    || core
                        .player((player.raw() - 1) as u8)
                        .hand
                        .contains(&state[1]))
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() >= 4
                && state.len() <= seats + 4
                && (1..=3).contains(&state[0])
                && (state[1] as usize) <= state.len() - 4
                && state[3] < (1 << seats)
                && state[4..]
                    .iter()
                    .all(|target| (*target as usize) < seats && *target + 1 != player.raw())
                && state[4..]
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == state.len() - 4
        }
        _ => false,
    }
}

pub(crate) fn active_eligible(
    module: &rule_vm::BytecodeModule,
    skill: &str,
    owner: u8,
    core: &GameStateCore,
) -> bool {
    use rule_vm::bytecode::{effect_tag, op};
    let tag = effect_tag::RESOLVE_OL_JIE_DANSHOU.to_le_bytes();
    // Only a verified single-resolver body has the resolver's intrinsic activation precondition.
    let single = [
        op::HOST_SELF,
        op::HOST_SKILL,
        op::HOST_EFFECT_EXT,
        tag[0],
        tag[1],
        op::HALT,
    ];
    if module.entry_pc == 0 && module.code == single {
        return !danshou::options(core, owner, skill).is_empty();
    }
    true
}
