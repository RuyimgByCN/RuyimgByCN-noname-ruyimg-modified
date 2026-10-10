//! Mutable-before-commit rule intents.
//!
//! Commands and VM effects first become these values; declarative DSL modifiers
//! may change or cancel them before any mutation or domain event is produced.

use crate::query::{
    apply_int_modifiers_for_state, apply_int_modifiers_for_state_with_other, ModifierTarget,
};
use crate::state::GameState;
use engine_api::{CardId, PlayerId};

/// Damage proposed by a rule effect, before HP mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageIntent {
    /// Source, or `NONE` for system/lose-HP damage.
    pub source: PlayerId,
    /// Target.
    pub target: PlayerId,
    /// Proposed amount.
    pub amount: u32,
    /// Lose-HP channel.
    pub is_lose_hp: bool,
    /// 0=normal, 1=fire, 2=thunder.
    pub nature: u8,
    /// Vetoed intents produce no mutation/event.
    pub cancelled: bool,
}

/// Card use proposed by a command or VM effect, before moving the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardUseIntent {
    /// User.
    pub player: PlayerId,
    /// Card instance.
    pub card: CardId,
    /// Ordered targets.
    pub targets: Vec<PlayerId>,
    /// Vetoed intents produce no mutation/event.
    pub cancelled: bool,
}

/// Draw proposed for a draw phase, before cards move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawIntent {
    /// Drawing player.
    pub player: PlayerId,
    /// Proposed count.
    pub count: u32,
    /// Vetoed intents draw nothing.
    pub cancelled: bool,
}

/// Apply declarative modifiers to a damage intent.
pub fn resolve_damage_intent(state: &GameState, intent: DamageIntent) -> DamageIntent {
    resolve_damage_intent_with_kind(state, intent, false)
}

/// Apply declarative modifiers to a damage intent while preserving whether the
/// damage came from the logical Sha resolution channel.
pub fn resolve_damage_intent_with_kind(
    state: &GameState,
    mut intent: DamageIntent,
    logical_sha: bool,
) -> DamageIntent {
    if intent.is_lose_hp {
        return intent;
    }
    if intent.source.is_some() {
        if logical_sha {
            let modifiers = state
                .modifiers
                .for_target(ModifierTarget::DamageAsHpLossForSha);
            let (replace, _, vetoed) =
                apply_int_modifiers_for_state(state, 0, &modifiers, Some(intent.source));
            if replace != 0 || vetoed {
                intent.is_lose_hp = true;
                return intent;
            }
        }
        let modifiers = state.modifiers.for_target(ModifierTarget::DamageAsHpLoss);
        let (replace, _, vetoed) =
            apply_int_modifiers_for_state(state, 0, &modifiers, Some(intent.source));
        if replace != 0 || vetoed {
            intent.is_lose_hp = true;
            return intent;
        }

        if intent.source != intent.target {
            let modifiers = state
                .modifiers
                .for_target(ModifierTarget::DamageBetweenOwnerAndOtherAsHpLoss);
            let applies = |owner| {
                let (replace, _, vetoed) =
                    apply_int_modifiers_for_state(state, 0, &modifiers, Some(owner));
                replace != 0 || vetoed
            };
            if applies(intent.source)
                || (state
                    .players
                    .get(intent.source)
                    .is_some_and(|source| source.alive)
                    && applies(intent.target))
            {
                intent.is_lose_hp = true;
                return intent;
            }
        }
    }
    let (amount, amount_veto, nature, nature_veto, cancel, cancel_veto) = if intent.source.is_some()
    {
        let owner = Some(intent.source);
        let amount = state
            .modifiers
            .for_damage_amount(intent.nature, logical_sha);
        let (amount, _, amount_veto) = apply_int_modifiers_for_state_with_other(
            state,
            intent.amount as i32,
            &amount,
            owner,
            Some(intent.target),
        );
        let nature = state.modifiers.for_target(ModifierTarget::DamageNature);
        let (nature, _, nature_veto) =
            apply_int_modifiers_for_state(state, intent.nature as i32, &nature, owner);
        let cancel = state.modifiers.for_target(ModifierTarget::DamageCancel);
        let (cancel, _, cancel_veto) = apply_int_modifiers_for_state(state, 0, &cancel, owner);
        (
            amount,
            amount_veto,
            nature,
            nature_veto,
            cancel,
            cancel_veto,
        )
    } else {
        (
            intent.amount as i32,
            false,
            intent.nature as i32,
            false,
            0,
            false,
        )
    };
    let resolved_nature = crate::query::damage_taken_nature_for(
        state,
        intent.target,
        nature.clamp(0, u8::MAX as i32) as u8,
    );
    let taken_cancel = state.modifiers.for_damage_taken_cancel(resolved_nature);
    let (taken_cancel, _, taken_cancel_veto) =
        apply_int_modifiers_for_state(state, 0, &taken_cancel, Some(intent.target));
    intent.amount = amount.max(0) as u32;
    intent.nature = resolved_nature;
    intent.cancelled = amount_veto
        || nature_veto
        || cancel_veto
        || cancel != 0
        || taken_cancel_veto
        || taken_cancel != 0;
    intent
}

/// Apply source-owned declarative modifiers to a card-use intent.
pub fn resolve_card_use_intent(state: &GameState, mut intent: CardUseIntent) -> CardUseIntent {
    intent.cancelled = crate::query::card_use_cancelled_for_card(state, intent.player, intent.card);
    intent
}

/// Apply draw-phase modifiers to a draw intent.
pub fn resolve_draw_intent(state: &GameState, mut intent: DrawIntent) -> DrawIntent {
    let modifiers = state.modifiers.for_draw_count(intent.player, |owner| {
        state.players.get(owner).is_some_and(|player| player.alive)
    });
    let (count, _, vetoed) = crate::query::apply_int_modifiers_for_state_with_context(
        state,
        intent.count as i32,
        &modifiers,
        None,
        Some(intent.player),
        None,
    );
    intent.count = count.max(0) as u32;
    intent.cancelled = vetoed;
    intent
}
