//! Command operations for `handle_card_with_policies_and_flags` and related helpers.

use super::*;

/// Proactive ordinary tricks that can currently be used source-free by `actor`.
/// `used_mask` stores permanent CardKind-tag bits for skills such as 3D精选【奋计】.
pub(crate) fn source_free_ordinary_trick_kinds(
    state: &crate::state::GameState,
    actor: PlayerId,
    used_mask: u64,
) -> Vec<CardKind> {
    let mut probe = Transaction::begin(state.clone());
    let Some(processing) = probe
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
    else {
        return Vec::new();
    };
    let mut candidates: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.def.is_some())
        .map(|card| card.kind)
        .filter(|kind| {
            kind.is_trick()
                && !kind.is_delayed_trick()
                && !matches!(kind, CardKind::TrickWuxie | CardKind::TrickYoudishenru)
                && used_mask & (1u64 << kind.tag()) == 0
        })
        .collect();
    candidates.sort_by_key(|kind| kind.tag());
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|kind| {
            let card = probe.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: *kind,
                suit: engine_api::Suit::None,
                rank: 0,
                weapon_range: 0,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            probe
                .state_mut()
                .zones
                .get_mut(processing)
                .expect("validated processing zone")
                .cards
                .push(card);
            skill_card_target_spec(&probe, actor, card).is_ok()
        })
        .collect()
}

/// Determine whether a revealed card has a legal use and which target choice it needs.
pub(crate) fn skill_card_target_spec(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<SkillCardTargetSpec, EffectError> {
    skill_card_target_spec_with_options(tx, actor, card, false, false)
}
