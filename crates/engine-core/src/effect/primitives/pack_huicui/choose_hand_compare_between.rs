//! Effect primitives for `choose_hand_compare_between` and related helpers.

use super::*;

/// ABI 680 (batch 174): third-party comparison — `first` picks an opponent (excluding
/// `player`), both stash one hand card, and the commit (flag=2 payload tail) makes the
/// winner use a distance-free virtual Sha against the loser on a non-tie. Payload reuses
/// continuations 75/76/49 with participant bits `[player, first]` and trailing `flag=2`.
pub(crate) fn choose_hand_compare_between(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "hand comparison requires its skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(first)
            .is_some_and(|participant| participant.alive)
        || state.hand_cards(first).is_empty()
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != first && candidate != player)
        .filter(|&candidate| !state.hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        first,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::BetweenOpponent,
        vec![player.raw(), first.raw(), skill.raw()],
    )
}

pub(crate) fn choose_hand_compare(
    tx: &mut Transaction,
    player: PlayerId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "hand comparison requires its skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || state.hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !state.hand_cards(target).is_empty())
        .map(PlayerId::raw)
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
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseHandCompareTarget,
        vec![
            player.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
        ],
    )
}
