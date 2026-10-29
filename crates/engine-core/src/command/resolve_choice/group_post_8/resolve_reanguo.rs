//! Command operations for `resolve_choose_other_give_one_hand_equip` and related helpers.

use super::*;

pub(crate) fn resolve_reanguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, option_count, options @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "truncated Reanguo recast payload".into(),
        ));
    };
    if options.len() != *option_count as usize {
        return Err(EffectError::EngineFault(
            "bad Reanguo recast payload".into(),
        ));
    }
    let player = PlayerId::new(*player_raw);
    let skill = SkillId::new(*skill_raw);
    if observer != player || (cancelled && !selections.is_empty()) {
        return Err(EffectError::Illegal(
            "invalid Reanguo recast cancellation".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let max = options.len().min(u8::MAX as usize);
    let mut unique = std::collections::BTreeSet::new();
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || cards.is_empty()
        || cards.len() > max
        || cards.iter().any(|card| {
            !unique.insert(card.raw())
                || !options.contains(&card.raw())
                || tx.state().cards.get(*card).is_none_or(|state| {
                    state.owner != player || (Some(state.zone) != hand && Some(state.zone) != equip)
                })
        })
    {
        return Err(EffectError::Illegal(
            "Reanguo recast choice context changed".into(),
        ));
    }
    run_effects(tx, vec![EffectRequest::RecastCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_scoped_discard_target_card_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
    scope: rule_api::TargetCardScope,
    gain_mount: bool,
) -> Result<(), EffectError> {
    if cancelled {
        if !gain_mount && payload.get(3).copied().unwrap_or(0) == 1 {
            return Ok(());
        }
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    let chooser = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if observer != chooser {
        return Err(EffectError::Illegal(
            "only the target-card discard chooser may resolve this choice".into(),
        ));
    }
    if ![chooser, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "target-card discard participants are no longer alive".into(),
        ));
    }
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(tx.state(), chooser, target, scope)
        .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "target card selection is no longer legal".into(),
        ));
    }
    let is_mount = tx
        .state()
        .cards
        .get(selected)
        .is_some_and(|card| matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef));
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: chooser,
        target,
        cards: vec![selected],
        scope,
    }];
    if gain_mount && is_mount {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, chooser)
            .ok_or_else(|| EffectError::EngineFault("missing chooser hand".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: vec![selected],
            to: hand,
            from: Some(discard),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_beizhu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Beizhu discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let option_count = payload[4] as usize;
            if cancelled
                || observer != player
                || selections.len() != 1
                || payload.len() != 5 + option_count
            {
                return Err(EffectError::Illegal(
                    "Beizhu requires one target card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[5..].contains(&card.raw())
                || !tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.owner == player)
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Beizhu target-card choice context changed".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveBeizhu {
                    player,
                    target,
                    stage: 1,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards: vec![card],
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, target_raw, card_raw, draw_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Beizhu draw-pile Sha payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player {
                return Err(EffectError::Illegal(
                    "only the Beizhu owner may resolve this choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal(
                    "invalid Beizhu draw-pile Sha choice".into(),
                ));
            }
            let target = PlayerId::new(*target_raw);
            let card = CardId::new(*card_raw);
            let draw = engine_api::ZoneId::new(*draw_raw);
            let skill = engine_api::SkillId::new(*skill_raw);
            if ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            }) || !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
                || !tx.state().zones.get(draw).is_some_and(|zone| {
                    zone.kind == ZoneKind::DrawPile
                        && zone.owner.is_none()
                        && zone.cards.contains(&card)
                })
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.zone == draw && state.kind.is_sha())
            {
                return Err(EffectError::Illegal(
                    "Beizhu draw-pile Sha choice context changed".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Beizhu target hand is missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to: hand,
                    from: draw,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Beizhu choice stage".into(),
        )),
    }
}

pub(crate) fn vm_trigger_marker(marker: [u32; 3]) -> EffectRequest {
    EffectRequest::EmitRuleEvent {
        code: 100,
        payload: marker.to_vec(),
    }
}

/// Store a VM pause into resolution state and open the corresponding choice.
pub(crate) fn open_vm_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    module_name: &str,
    skill_id: u32,
    ctx: [u32; 4],
    trigger_marker: [u32; 3],
    pause: crate::vm_host::VmPause,
) -> Result<(), EffectError> {
    {
        let r = &mut tx.state_mut().resolution;
        r.vm_cont_frames = pause.frames;
        r.vm_cont_module = module_name.to_string();
        r.vm_cont_pc = pause.pc;
        r.vm_cont_fuel = pause.fuel_remaining;
        r.vm_cont_ctx = ctx;
        r.vm_cont_skill = skill_id;
        r.vm_cont_trigger_marker = trigger_marker;
    }
    crate::effect::open_choice(
        tx,
        observer,
        1,
        1,
        false,
        pause.options,
        engine_api::ChoiceKind::Custom,
        ContinuationKind::VmResume,
        Vec::new(),
    )?;
    Ok(())
}

pub(crate) fn jiedao_give_weapon(
    tx: &mut Transaction,
    actor: PlayerId,
    holder: PlayerId,
) -> Result<(), EffectError> {
    use crate::state::CardKind;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, holder)
        .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
    let weapon = tx
        .state()
        .zones
        .get(equip)
        .and_then(|z| {
            z.cards.iter().copied().find(|cid| {
                tx.state()
                    .cards
                    .get(*cid)
                    .map(|c| c.kind == CardKind::Weapon)
                    .unwrap_or(false)
            })
        })
        .ok_or_else(|| EffectError::Illegal("no weapon".into()))?;
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: to_hand,
            from: Some(equip),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_multi_card_view_as_response(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad multi-card response kind".into()))?;
    if cancelled || observer != player || selections.is_empty() {
        return Err(EffectError::Illegal(
            "multi-card response source choice cannot cancel".into(),
        ));
    }
    let rule = crate::command::skill::multi_card_response_rule(tx.state(), player, skill, kind)
        .ok_or_else(|| EffectError::Illegal("multi-card response conversion unavailable".into()))?;
    let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
    if !crate::command::skill::view_as_sources_match(tx.state(), player, &rule, &cards)
        || cards
            .iter()
            .any(|card| crate::query::card_color_locked_for_card(tx.state(), player, *card))
    {
        return Err(EffectError::Illegal(
            "invalid multi-card response sources".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown response conversion skill".into()))?;
    if rule
        .uses_per_turn
        .is_some_and(|limit| instance.use_count >= u32::from(limit))
    {
        return Err(EffectError::Illegal(
            "view_as skill use limit reached".into(),
        ));
    }
    if rule.cost.as_deref() == Some("potqingyan") {
        run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            }],
        )?;
        let round = tx.state().turn.round;
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.view_as_use_round != round {
                instance.view_as_use_round = round;
                instance.view_as_used_branches = 0;
            }
            instance.view_as_used_branches = instance.view_as_used_branches.saturating_add(1);
            instance.state_blob = cards
                .iter()
                .flat_map(|card| card.raw().to_le_bytes())
                .collect();
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.use_count = instance.use_count.saturating_add(1);
    }
    let bonus = (cards.len() > 1)
        .then_some(rule.multi_card_bonus.as_deref())
        .flatten();
    if rule.cost.as_deref() == Some("potqingyan") {
        complete_view_as_response(tx, payload, &[], bonus)
    } else {
        complete_view_as_response(tx, payload, &cards, bonus)
    }
}

/// Complete a source-free conversion after its cost has resolved.
pub(crate) fn resume_source_free_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let source = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|skill| skill.take_source_free_physical_card());
    match source {
        Some(source) => complete_view_as_response(tx, payload, &[source], None),
        None => complete_view_as_response(tx, payload, &[], None),
    }
}


pub(crate) fn resolve_choose_give_or_recast_one_hand(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || selections.len() != 1 || cancelled || observer != PlayerId::new(payload[0])
    {
        return Err(EffectError::Illegal(
            "choose one card/target for give-or-recast".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let stage = payload[2];
    let enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !enabled || !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "give-or-recast skill context changed".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing hand zone".into()))?;
    match stage {
        0 => {
            let card = CardId::new(selections[0]);
            if !tx
                .state()
                .zones
                .get(hand)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Err(EffectError::Illegal("chosen card left hand".into()));
            }
            let def = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.def.raw())
                .unwrap_or(0);
            let recastable = def != 0
                && (crate::content::is_recastable_card_def(def)
                    || tx
                        .state()
                        .content
                        .card_metadata(engine_api::CardDefId::new(def))
                        .is_some_and(|metadata| metadata.name == "tiesuo"));
            let mut options = Vec::new();
            if recastable {
                options.push(0);
            }
            options.extend(
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|target| *target != player)
                    .map(PlayerId::raw),
            );
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ChooseGiveOrRecastOneHand,
                vec![
                    player.raw(),
                    skill.raw(),
                    1,
                    card.raw(),
                    payload[3],
                    payload[4],
                    payload[5],
                ],
            )?;
            Ok(())
        }
        1 => {
            if payload.len() != 7 {
                return Err(EffectError::EngineFault(
                    "bad give-or-recast card payload".into(),
                ));
            }
            let card = CardId::new(payload[3]);
            let require_skill_def = payload[4];
            let grant_skill_def = payload[5];
            let grant_mark_def = payload[6];
            if !tx
                .state()
                .zones
                .get(hand)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Err(EffectError::Illegal("chosen card left hand".into()));
            }
            if selections[0] == 0 {
                let recastable = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|state| state.def.raw())
                    .is_some_and(|def| {
                        crate::content::is_recastable_card_def(def)
                            || tx
                                .state()
                                .content
                                .card_metadata(engine_api::CardDefId::new(def))
                                .is_some_and(|metadata| metadata.name == "tiesuo")
                    });
                if !recastable {
                    return Err(EffectError::Illegal(
                        "card is not recastable".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::RecastCards {
                        player,
                        cards: vec![card],
                    }],
                )?;
                return Ok(());
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx.state().players.get(target).is_some_and(|p| p.alive)
            {
                return Err(EffectError::Illegal("invalid give target".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("missing target hand zone".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from: hand,
                }],
            )?;
            let owns_required = require_skill_def != 0
                && tx.state().skills.sorted_live().iter().any(|instance| {
                    instance.owner == player && instance.def.raw() == require_skill_def
                });
            if owns_required
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && !participant.dying)
            {
                let mut rewards = Vec::new();
                if grant_skill_def != 0 {
                    rewards.push(EffectRequest::AddSkill {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(grant_skill_def),
                    });
                }
                if grant_mark_def != 0 {
                    rewards.push(EffectRequest::AddMark {
                        player: target,
                        mark: engine_api::MarkDefId::new(grant_mark_def),
                        delta: 1,
                    });
                }
                if !rewards.is_empty() {
                    run_effects(tx, rewards)?;
                }
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "bad give-or-recast stage".into(),
        )),
    }
}
