//! Occurrence-time public presentation evaluation (`I02` Decision 30, `proposal-v3 §3`).
//!
//! Evaluates a [`LogEvent`] against the live [`GameStateCore`], [`Resolution`], and
//! [`Catalog`] at the exact moment the log event is recorded, producing a
//! [`PublicPresentationEvent`] whose speaker role, form, effective card identity, and
//! audio asset bindings are frozen at occurrence time.
//!
//! Invariants:
//! - Read-only over `core`, `res`, and `catalog`: never mutates rule state, never
//!   consumes RNG or `EventId`, and never affects hashes or archives.
//! - Uses only publicly visible state at the moment of the occurrence (e.g., unrevealed
//!   general slots in `GameMode::Kingdoms` are never disclosed).
//! - View-as cards (`use_as` / `declared_kinds`) resolve by their committed effective
//!   kind and nature, never by their hidden/physical card face.

use engine_api::{
    BoundAudioAsset, DefinitionPresentationBinding, EffectivePresentationCard, PlayerId as ApiPlayerId,
    PresentationLocalId, PresentationNature, PublicPresentationEvent, PublicPresentationSkill,
    PublicVoice,
};

use crate::catalog::{CardKind, Catalog};
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::LogEvent;
use crate::resolution::Resolution;
use crate::snap::MachineSnapshot;
use crate::state::{GameMode, GameStateCore};
use crate::vocabulary::generals::Gender;

fn api_player(p: PlayerId) -> ApiPlayerId {
    ApiPlayerId::new(u32::from(p) + 1)
}

fn api_targets(targets: &[PlayerId]) -> Vec<ApiPlayerId> {
    let mut out = Vec::with_capacity(targets.len());
    for &t in targets {
        let ap = api_player(t);
        if !out.contains(&ap) {
            out.push(ap);
        }
    }
    out
}

fn map_nature(n: DamageNature) -> PresentationNature {
    match n {
        DamageNature::Normal => PresentationNature::Normal,
        DamageNature::Fire => PresentationNature::Fire,
        DamageNature::Thunder => PresentationNature::Thunder,
    }
}

fn card_kind_default_nature(kind: CardKind) -> PresentationNature {
    match kind {
        CardKind::FireSha => PresentationNature::Fire,
        CardKind::LeiSha | CardKind::ShanDian => PresentationNature::Thunder,
        _ => PresentationNature::Normal,
    }
}

/// Publicly visible general slots `(slot_index, general_def_id)` for `player` at the
/// current moment. In `GameMode::Kingdoms`, unrevealed slots are excluded.
fn public_general_slots(core: &GameStateCore, player: PlayerId) -> Vec<(usize, u16)> {
    let Some(p) = core.players.get(player as usize) else {
        return Vec::new();
    };
    let kingdoms = matches!(core.mode, GameMode::Kingdoms);
    let mut out = Vec::with_capacity(2);
    for (idx, slot) in p.general_slots.iter().enumerate() {
        if let Some(g) = *slot {
            if !kingdoms || p.revealed_generals[idx] {
                out.push((idx, g));
            }
        }
    }
    out
}

/// Returns `true` if `skill` belongs to an unrevealed general slot of `player` in
/// `GameMode::Kingdoms` and is NOT declared on any revealed general slot of `player`.
fn is_hidden_kingdoms_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    skill: &str,
) -> bool {
    if !matches!(core.mode, GameMode::Kingdoms) {
        return false;
    }
    let Some(p) = core.players.get(player as usize) else {
        return false;
    };
    let mut on_revealed = false;
    let mut on_hidden = false;
    for (idx, slot) in p.general_slots.iter().enumerate() {
        let Some(g) = *slot else { continue };
        let Some((_, _, skills, _, _)) = catalog.general_meta(g) else {
            continue;
        };
        if skills.contains(&skill) {
            if p.revealed_generals[idx] {
                on_revealed = true;
            } else {
                on_hidden = true;
            }
        }
    }
    on_hidden && !on_revealed
}

/// Select the speaker's public general `(general_def_id, gender, binding)` at occurrence time.
fn speaker_public_general<'a>(
    core: &GameStateCore,
    catalog: &'a Catalog,
    player: PlayerId,
    preferred_skill: Option<&str>,
) -> Option<(u16, Gender, &'a DefinitionPresentationBinding)> {
    let public_slots = public_general_slots(core, player);
    if public_slots.is_empty() {
        return None;
    }
    let chosen_g = if let Some(sk) = preferred_skill {
        public_slots
            .iter()
            .copied()
            .find(|&(_, g)| {
                catalog
                    .general_meta(g)
                    .is_some_and(|(_, _, skills, _, _)| skills.contains(&sk))
            })
            .map(|(_, g)| g)
            .unwrap_or(public_slots[0].1)
    } else {
        public_slots[0].1
    };
    let (_, _, _, _, gender) = catalog.general_meta(chosen_g)?;
    let binding = catalog.general_presentation_binding(chosen_g)?.as_ref();
    Some((chosen_g, gender, binding))
}

/// Public form token for `player` at occurrence time, or `None` for the base role.
fn public_form_for(core: &GameStateCore, player: PlayerId) -> Option<String> {
    let p = core.players.get(player as usize)?;
    match p.baolei_mode {
        1 => return Some("bshaowei".to_string()),
        2 => return Some("zhencha".to_string()),
        _ => {}
    }
    if let Some(form_id) = core
        .vm_aux
        .flags
        .get(&(player, "olxuanbai_form".to_string()))
        .copied()
    {
        match form_id {
            1 => return Some("dangmo".to_string()),
            2 => return Some("jihui".to_string()),
            3 => return Some("xiaju".to_string()),
            _ => {}
        }
    }
    if let Some(skin_id) = core
        .vm_aux
        .flags
        .get(&(player, "c29_skin_id".to_string()))
        .copied()
        .filter(|&v| v > 0)
    {
        return Some(format!("skin:{skin_id}"));
    }
    None
}

fn dedup_assets(assets: &[BoundAudioAsset]) -> Vec<BoundAudioAsset> {
    let mut out = Vec::with_capacity(assets.len());
    for a in assets {
        if !out.contains(a) {
            out.push(a.clone());
        }
    }
    out
}

fn lookup_first_nonempty(
    map: &std::collections::BTreeMap<String, Vec<BoundAudioAsset>>,
    keys: &[String],
) -> Option<Vec<BoundAudioAsset>> {
    for k in keys {
        if let Some(list) = map.get(k) {
            let d = dedup_assets(list);
            if !d.is_empty() {
                return Some(d);
            }
        }
    }
    None
}

fn resolve_card_voice(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    effective_kind: CardKind,
    nature: PresentationNature,
    physical_card_if_untransformed: Option<CardId>,
) -> Option<PublicVoice> {
    let (_, gender, role_binding) = speaker_public_general(core, catalog, player, None)?;
    let form = public_form_for(core, player);

    let card_binding = physical_card_if_untransformed
        .and_then(|cid| catalog.cards.get(cid as usize))
        .and_then(|c| c.presentation_binding.as_deref())
        .or_else(|| {
            catalog
                .cards
                .iter()
                .find(|c| c.kind == effective_kind && c.presentation_binding.is_some())
                .and_then(|c| c.presentation_binding.as_deref())
        })?;

    let gender_key = match gender {
        Gender::Male => "male",
        Gender::Female => "female",
    };
    let mut candidate_keys = Vec::with_capacity(5);
    match nature {
        PresentationNature::Fire => {
            candidate_keys.push(format!("{gender_key}:fire"));
            candidate_keys.push("fire".to_string());
        }
        PresentationNature::Thunder => {
            candidate_keys.push(format!("{gender_key}:thunder"));
            candidate_keys.push("thunder".to_string());
        }
        PresentationNature::Normal => {}
    }
    candidate_keys.push(gender_key.to_string());
    candidate_keys.push("default".to_string());
    candidate_keys.push("activate".to_string());

    let assets = lookup_first_nonempty(&card_binding.voice_assets, &candidate_keys)?;
    Some(PublicVoice {
        role: role_binding.definition.clone(),
        form,
        assets,
    })
}

fn resolve_skill_voice(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
    skill_binding: &DefinitionPresentationBinding,
) -> Option<PublicVoice> {
    let (_, _, role_binding) = speaker_public_general(core, catalog, owner, Some(skill_name))?;
    let form = public_form_for(core, owner);

    let raw_skill_key = match &skill_binding.definition.local_id {
        PresentationLocalId::Original(orig) => orig.raw_symbol.as_str(),
        PresentationLocalId::Package(_) => skill_name,
    };
    let role_symbol = match &role_binding.definition.local_id {
        PresentationLocalId::Original(orig) => Some(orig.raw_symbol.as_str()),
        PresentationLocalId::Package(_) => None,
    };

    if let Some(ref f) = form {
        let mut role_form_keys = vec![
            format!("skill:{raw_skill_key}:form:{f}"),
            format!("form:{f}:skill:{raw_skill_key}"),
            format!("skill:{skill_name}:form:{f}"),
            format!("form:{f}:skill:{skill_name}"),
            format!("activate:form:{f}"),
            format!("form:{f}:activate"),
        ];
        if let Some(skin_id) = f.strip_prefix("skin:") {
            role_form_keys.insert(0, format!("skin:{skin_id}:skill:{raw_skill_key}"));
            role_form_keys.insert(1, format!("skin:{skin_id}:skill:{skill_name}"));
            role_form_keys.push(format!("skin:{skin_id}:activate"));
        }
        if let Some(assets) = lookup_first_nonempty(&role_binding.voice_assets, &role_form_keys) {
            return Some(PublicVoice {
                role: role_binding.definition.clone(),
                form,
                assets,
            });
        }
        let mut skill_form_keys = vec![
            format!("role:{f}"),
            format!("activate:form:{f}"),
            format!("form:{f}:activate"),
        ];
        if let Some(skin_id) = f.strip_prefix("skin:") {
            skill_form_keys.insert(0, format!("skin:{skin_id}:activate"));
            skill_form_keys.insert(1, format!("skin:{skin_id}"));
        }
        if let Some(assets) = lookup_first_nonempty(&skill_binding.voice_assets, &skill_form_keys) {
            return Some(PublicVoice {
                role: role_binding.definition.clone(),
                form,
                assets,
            });
        }
    }

    let role_skill_keys = [
        format!("skill:{raw_skill_key}"),
        format!("skill:{skill_name}"),
    ];
    if let Some(assets) = lookup_first_nonempty(&role_binding.voice_assets, &role_skill_keys) {
        return Some(PublicVoice {
            role: role_binding.definition.clone(),
            form,
            assets,
        });
    }

    let mut skill_keys = Vec::with_capacity(3);
    if let Some(rs) = role_symbol {
        skill_keys.push(format!("role:{rs}"));
    }
    skill_keys.push("activate".to_string());
    skill_keys.push("default".to_string());
    if let Some(assets) = lookup_first_nonempty(&skill_binding.voice_assets, &skill_keys) {
        return Some(PublicVoice {
            role: role_binding.definition.clone(),
            form,
            assets,
        });
    }

    let fallback_role_keys = ["activate".to_string()];
    if let Some(assets) = lookup_first_nonempty(&role_binding.voice_assets, &fallback_role_keys) {
        return Some(PublicVoice {
            role: role_binding.definition.clone(),
            form,
            assets,
        });
    }

    None
}

fn resolve_die_voice(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> Option<PublicVoice> {
    let (_, _, role_binding) = speaker_public_general(core, catalog, player, None)?;
    let form = public_form_for(core, player);

    let mut keys = Vec::with_capacity(6);
    if let Some(ref f) = form {
        if let Some(skin_id) = f.strip_prefix("skin:") {
            keys.push(format!("skin:{skin_id}:die"));
            keys.push(format!("die:skin:{skin_id}"));
        }
        keys.push(format!("die:form:{f}"));
        keys.push(format!("form:{f}:die"));
    }
    keys.push("die".to_string());

    let assets = lookup_first_nonempty(&role_binding.voice_assets, &keys)?;
    Some(PublicVoice {
        role: role_binding.definition.clone(),
        form,
        assets,
    })
}

fn current_damage_info(
    res: &Resolution,
    target: PlayerId,
    card: Option<CardId>,
    catalog: &Catalog,
) -> (Option<PlayerId>, PresentationNature) {
    for frame in res.stack.iter().rev() {
        match &frame.data {
            EventData::Damage(d) if d.target == target => {
                return (d.source, map_nature(d.nature));
            }
            EventData::DamageWindow { root, .. } => {
                if let Some(rf) = res.frame(*root) {
                    if let EventData::Damage(d) = &rf.data {
                        if d.target == target {
                            return (d.source, map_nature(d.nature));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for frame in res.stack.iter().rev() {
        if let EventData::Damage(d) = &frame.data {
            return (d.source, map_nature(d.nature));
        }
    }
    let nature = card
        .map(|cid| card_kind_default_nature(catalog.card_kind(cid)))
        .unwrap_or(PresentationNature::Normal);
    (None, nature)
}

fn effective_card_info_for_use(
    core: &GameStateCore,
    res: &Resolution,
    _catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    fallback_kind: CardKind,
) -> (EffectivePresentationCard, Option<CardId>) {
    let mut use_as: Option<CardKind> = None;
    let mut frame_nature: Option<DamageNature> = None;
    for frame in res.stack.iter().rev() {
        if let EventData::CardUsed(u) = &frame.data {
            if u.player == player && u.card == card {
                use_as = u.use_as;
                frame_nature = Some(u.nature);
                break;
            }
        }
    }
    if use_as.is_none() {
        if let Some(cid) = card {
            use_as = core.declared_kinds.get(&cid).copied();
        }
    }
    let effective_kind = use_as.unwrap_or(fallback_kind);
    let nature = match (effective_kind, frame_nature) {
        (CardKind::FireSha, _) | (_, Some(DamageNature::Fire)) => PresentationNature::Fire,
        (CardKind::LeiSha, _) | (_, Some(DamageNature::Thunder)) => PresentationNature::Thunder,
        (_, Some(DamageNature::Normal)) => card_kind_default_nature(effective_kind),
        (_, None) => card_kind_default_nature(effective_kind),
    };
    let untransformed_card = if use_as.is_none() { card } else { None };
    let key = match (effective_kind, nature) {
        (CardKind::Sha, PresentationNature::Fire) => "huosha".to_string(),
        (CardKind::Sha, PresentationNature::Thunder) => "leisha".to_string(),
        _ => effective_kind.key().to_string(),
    };
    (
        EffectivePresentationCard {
            kind: effective_kind.key().to_string(),
            key,
            nature,
        },
        untransformed_card,
    )
}

fn infer_response_kind_from_snapshot(
    snap: &MachineSnapshot,
    card: CardId,
    physical_kind: CardKind,
) -> Option<CardKind> {
    match snap {
        MachineSnapshot::NullifyWindow {
            window_open,
            effect,
            ..
        } => {
            if *window_open {
                Some(CardKind::Nullify)
            } else if let Some(inner) = effect.as_deref() {
                infer_response_kind_from_snapshot(inner, card, physical_kind)
            } else {
                None
            }
        }
        MachineSnapshot::Sha {
            card: sha_card,
            asking_qinglong,
            ..
        } => {
            if physical_kind == CardKind::Youdishenru {
                Some(CardKind::Youdishenru)
            } else if *asking_qinglong || *sha_card == Some(card) {
                Some(CardKind::Sha)
            } else {
                Some(CardKind::Shan)
            }
        }
        MachineSnapshot::Duel { .. } => Some(CardKind::Sha),
        MachineSnapshot::AoeTarget { need, .. } => Some(*need),
        MachineSnapshot::DyingSave { .. } => Some(CardKind::Tao),
        _ => None,
    }
}

fn effective_card_info_for_response(
    core: &GameStateCore,
    res: &Resolution,
    catalog: &Catalog,
    _player: PlayerId,
    card: CardId,
) -> (CardKind, EffectivePresentationCard, Option<CardId>) {
    let physical_kind = catalog.card_kind(card);
    let inferred_kind = if let Some(&decl) = core.declared_kinds.get(&card) {
        decl
    } else {
        res.parks
            .iter()
            .rev()
            .find_map(|park| {
                infer_response_kind_from_snapshot(&park.machine.snapshot(), card, physical_kind)
            })
            .unwrap_or(physical_kind)
    };
    let nature = card_kind_default_nature(inferred_kind);
    let untransformed_card = (inferred_kind == physical_kind).then_some(card);
    let key = match (inferred_kind, nature) {
        (CardKind::Sha, PresentationNature::Fire) => "huosha".to_string(),
        (CardKind::Sha, PresentationNature::Thunder) => "leisha".to_string(),
        _ => inferred_kind.key().to_string(),
    };
    (
        inferred_kind,
        EffectivePresentationCard {
            kind: inferred_kind.key().to_string(),
            key,
            nature,
        },
        untransformed_card,
    )
}

fn resolve_skill_def_for_owner(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill: &str,
) -> Option<crate::ids::SkillDefId> {
    let speaker_owner = speaker_public_general(core, catalog, owner, Some(skill))
        .map(|(_, _, b)| b.definition.owner.clone());
    let mut enabled_matches = Vec::new();
    let mut disabled_matches = Vec::new();
    for ((seat, def_id), inst) in core.skills.range((owner, 0)..) {
        if *seat != owner {
            break;
        }
        if let Some(sdef) = catalog.skills.get(*def_id as usize) {
            if sdef.name == skill && !sdef.name.contains("#r05:") {
                if inst.enabled {
                    enabled_matches.push(*def_id);
                } else {
                    disabled_matches.push(*def_id);
                }
            }
        }
    }
    for list in [&enabled_matches, &disabled_matches] {
        if let Some(ref sp_owner) = speaker_owner {
            if let Some(&id) = list.iter().find(|&&id| {
                catalog.skills[id as usize]
                    .presentation_binding
                    .as_ref()
                    .is_some_and(|b| &b.definition.owner == sp_owner)
            }) {
                return Some(id);
            }
        }
        if let Some(&id) = list
            .iter()
            .find(|&&id| catalog.skills[id as usize].presentation_binding.is_some())
        {
            return Some(id);
        }
        if let Some(&id) = list.first() {
            return Some(id);
        }
    }
    catalog.skill_by_name(skill)
}

/// Evaluate and attach the occurrence-time [`PublicPresentationEvent`] to `event` in-place.
pub fn attach_occurrence_presentation(
    core: &GameStateCore,
    res: &Resolution,
    catalog: &Catalog,
    event: &mut LogEvent,
) {
    attach_occurrence_presentation_with_skill_def(core, res, catalog, None, event);
}

/// Evaluate and attach the occurrence-time [`PublicPresentationEvent`] to `event` in-place,
/// using `explicit_skill_def` when known at the scheduler trigger site.
pub fn attach_occurrence_presentation_with_skill_def(
    core: &GameStateCore,
    res: &Resolution,
    catalog: &Catalog,
    explicit_skill_def: Option<crate::ids::SkillDefId>,
    event: &mut LogEvent,
) {
    match event {
        LogEvent::DamageDealt {
            source,
            target,
            amount,
            card,
            presentation,
            ..
        } => {
            if *amount == 0 {
                *presentation = None;
                return;
            }
            let (frame_source, nature) = current_damage_info(res, *target, *card, catalog);
            let final_source = source.or(frame_source).map(api_player);
            *presentation = Some(Box::new(PublicPresentationEvent::DamageDealt {
                source: final_source,
                target: api_player(*target),
                amount: *amount,
                nature,
            }));
        }
        LogEvent::DamagePrevented {
            target,
            presentation,
        } => {
            let (source, nature) = current_damage_info(res, *target, None, catalog);
            *presentation = Some(Box::new(PublicPresentationEvent::DamagePrevented {
                source: source.map(api_player),
                target: api_player(*target),
                nature,
            }));
        }
        LogEvent::HpRecovered {
            player,
            amount,
            presentation,
        } => {
            if *amount == 0 {
                *presentation = None;
                return;
            }
            *presentation = Some(Box::new(PublicPresentationEvent::HpRecovered {
                actor: api_player(*player),
                amount: *amount,
            }));
        }
        LogEvent::CardPlayed {
            card,
            player,
            targets,
            presentation,
        } => {
            let fallback_kind = catalog.card_kind(*card);
            let (effective, untransformed_card) =
                effective_card_info_for_use(core, res, catalog, *player, Some(*card), fallback_kind);
            let effective_kind = CardKind::from_key(&effective.kind).unwrap_or(fallback_kind);
            let voice = resolve_card_voice(
                core,
                catalog,
                *player,
                effective_kind,
                effective.nature,
                untransformed_card,
            );
            *presentation = Some(Box::new(PublicPresentationEvent::CardUsed {
                actor: api_player(*player),
                targets: Some(api_targets(targets)),
                effective,
                voice,
            }));
        }
        LogEvent::VirtualCardPlayed {
            kind,
            player,
            targets,
            presentation,
        } => {
            let (effective, _) =
                effective_card_info_for_use(core, res, catalog, *player, None, *kind);
            let voice = resolve_card_voice(
                core,
                catalog,
                *player,
                *kind,
                effective.nature,
                None,
            );
            *presentation = Some(Box::new(PublicPresentationEvent::CardUsed {
                actor: api_player(*player),
                targets: Some(api_targets(targets)),
                effective,
                voice,
            }));
        }
        LogEvent::CardResponded {
            card,
            player,
            presentation,
        } => {
            let (effective_kind, effective, untransformed_card) =
                effective_card_info_for_response(core, res, catalog, *player, *card);
            let voice = resolve_card_voice(
                core,
                catalog,
                *player,
                effective_kind,
                effective.nature,
                untransformed_card,
            );
            *presentation = Some(Box::new(PublicPresentationEvent::CardResponded {
                actor: api_player(*player),
                effective,
                voice,
            }));
        }
        LogEvent::PlayerDied {
            player,
            killer,
            presentation,
        } => {
            let voice = resolve_die_voice(core, catalog, *player);
            *presentation = Some(Box::new(PublicPresentationEvent::PlayerDied {
                actor: api_player(*player),
                killer: killer.map(api_player),
                voice,
            }));
        }
        LogEvent::SkillTriggered {
            skill,
            owner,
            presentation,
        } => {
            if skill.contains("#r05:") || is_hidden_kingdoms_skill(core, catalog, *owner, skill) {
                *presentation = None;
                return;
            }
            // In Kingdoms mode, if the owner has general slots configured and none are revealed,
            // do not disclose a skill activation tied to their hidden identity.
            if matches!(core.mode, GameMode::Kingdoms) {
                if let Some(p) = core.players.get(*owner as usize) {
                    if p.general_slots.iter().any(Option::is_some)
                        && !p.revealed_generals.iter().any(|&r| r)
                    {
                        *presentation = None;
                        return;
                    }
                }
            }
            let Some(skill_def_id) = explicit_skill_def
                .or_else(|| resolve_skill_def_for_owner(core, catalog, *owner, skill))
            else {
                *presentation = None;
                return;
            };
            let Some(skill_def) = catalog.skills.get(skill_def_id as usize) else {
                *presentation = None;
                return;
            };
            let Some(skill_binding) = skill_def.presentation_binding.as_deref() else {
                *presentation = None;
                return;
            };
            let voice = resolve_skill_voice(core, catalog, *owner, skill, skill_binding);
            *presentation = Some(Box::new(PublicPresentationEvent::SkillActivated {
                actor: api_player(*owner),
                skill: PublicPresentationSkill {
                    definition: skill_binding.definition.clone(),
                },
                targets: None,
                voice,
            }));
        }
        _ => {}
    }
}
