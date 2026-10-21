//! Effect primitives for `resolve_oldxuanfeng` and related helpers.

use super::*;

pub(crate) fn resolve_oldxuanfeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: CardKind::Sha,
                suit: Suit::None,
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
            let options = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && (oldxuanfeng_distance_one(tx, player, target)
                            || crate::command::play::validate_sha_target_with_options(
                                tx, player, card, target, false,
                            )
                            .is_ok())
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                remove_oldxuanfeng_virtual_sha(tx, card)?;
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
                ContinuationKind::ResolveOldXuanfeng,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let direct = oldxuanfeng_distance_one(tx, player, target);
            let sha = crate::command::play::validate_sha_target_with_options(
                tx, player, card, target, false,
            )
            .is_ok();
            match (sha, direct) {
                (true, true) => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![1, 2],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveOldXuanfeng,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                ),
                (true, false) => {
                    crate::command::play::handle_skill_card_with_policies(
                        tx,
                        player,
                        card,
                        &[target],
                        false,
                        true,
                    )?;
                    Ok(Vec::new())
                }
                (false, true) => {
                    remove_oldxuanfeng_virtual_sha(tx, card)?;
                    run_effects(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: player,
                            target,
                            amount: 1,
                            is_lose_hp: false,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                    )
                }
                (false, false) => Err(EffectError::Illegal(
                    "old Xuanfeng target became illegal".into(),
                )),
            }
        }
        2 => match branch {
            1 => {
                crate::command::play::handle_skill_card_with_policies(
                    tx,
                    player,
                    card,
                    &[target],
                    false,
                    true,
                )?;
                Ok(Vec::new())
            }
            2 if oldxuanfeng_distance_one(tx, player, target) => {
                remove_oldxuanfeng_virtual_sha(tx, card)?;
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
            }
            _ => Err(EffectError::Illegal("invalid old Xuanfeng branch".into())),
        },
        _ => Err(EffectError::ContentFault(
            "invalid old Xuanfeng stage".into(),
        )),
    }
}

pub(crate) fn choose_target_gain_max_hp(
    tx: &mut Transaction,
    player: PlayerId,
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
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetGainMaxHp,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn gain_target_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
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
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !tx.state().hand_cards(target).is_empty())
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::GainTargetHandCard,
        vec![0, player.raw(), skill.raw()],
    )
}

/// 穆荫 (clan/clanmuyin, ABI 648): the owner selects a living player who is the
/// owner themself or shares a clan with the owner and whose hand limit is not
/// the field maximum; that player permanently gains +1 hand limit.
pub(crate) fn choose_clan_not_max_hand_limit_gain(
    tx: &mut Transaction,
    player: PlayerId,
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
    let owner_clans: std::collections::BTreeSet<String> = tx
        .state()
        .players
        .get(player)
        .map(|p| p.character_def)
        .and_then(|def| tx.state().content.character(def))
        .map(|c| c.clans.iter().cloned().collect())
        .unwrap_or_default();
    let hand_limit = |state: &crate::state::GameState, target: PlayerId| -> i32 {
        match crate::query::query(
            state,
            &rule_api::QueryRequest::HandLimit { player: target },
            &mut crate::query::QueryContext::new(),
        ) {
            rule_api::QueryResponse::Int(n) => n,
            _ => 0,
        }
    };
    let state = tx.state();
    let is_field_max = |target: PlayerId| -> bool {
        let own = hand_limit(state, target);
        state
            .players
            .alive_by_seat()
            .into_iter()
            .any(|other| other != target && hand_limit(state, other) > own)
    };
    let targets: Vec<u32> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            if target == player {
                return !is_field_max(target);
            }
            let shares_clan = state
                .players
                .get(target)
                .map(|p| p.character_def)
                .and_then(|def| state.content.character(def))
                .is_some_and(|c| c.clans.iter().any(|clan| owner_clans.contains(clan)));
            shares_clan && !is_field_max(target)
        })
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseClanNotMaxHandLimitGain,
        vec![player.raw(), skill.raw()],
    )
}

/// 额外结算 (clanfenshi family, ABI 649): queue one extra effect pass for the
/// currently suspended trick. Upstream `trigger.effectCount++` runs inside the
/// CardUsed trigger window, i.e. while the trick sits in its WuxieWindow
/// continuation; the extra passes are applied by `apply_deferred_trick` via
/// `extra_effect_count` once the window clears.
pub(crate) fn extra_resolve_current_trick(
    tx: &mut Transaction,
    player: PlayerId,
    card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::ContentFault(
            "extra resolution requires a suspended trick (WuxieWindow)".into(),
        ));
    }
    // Resolve the suspended source card from the wuxie payload marker
    // (same extraction as `increase_saved_card_base_damage`: rposition over the
    // whole ask-phase resume payload).
    let payload = &tx.state().resolution.resume_payload;
    let suspended_card = payload
        .iter()
        .rposition(|value| *value == crate::command::play::SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .map(|index| engine_api::CardId::new(*payload.get(index + 1).unwrap_or(&0)))
        .unwrap_or(engine_api::CardId::NONE);
    if card.is_none() || card != suspended_card {
        return Err(EffectError::Illegal(
            "extra resolution card does not match the suspended trick".into(),
        ));
    }
    let entry = tx
        .state_mut()
        .extra_effect_count
        .entry(card.raw())
        .or_insert(0);
    if *entry >= 2 {
        return Err(EffectError::Illegal(
            "extra resolution pass limit reached".into(),
        ));
    }
    *entry += 1;
    Ok(Vec::new())
}
