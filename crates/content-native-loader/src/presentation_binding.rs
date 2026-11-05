//! Verified presentation and audio bindings (`I02` Decision 30, `proposal-v3 §2`).
//!
//! Attaches immutable [`DefinitionPresentationBinding`] records to catalog definitions
//! from either:
//! 1. Frozen original presentation inputs (`content/original/catalog.json` +
//!    `content/original/asset-manifest.json`) verified against build SHA-256 pins and
//!    six-field alias/backing invariants (see [`original`]).
//! 2. Verified [`FrozenPackage`] copies and [`ContentPackage`] indices, resolving
//!    explicit `definition_id` rule linkages and BLAKE3 audio payload digests.

pub mod original;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use content_runtime::{FrozenPackage, LockedPackage};
use content_schema::{ContentPackage, PackageSkinPresentation};
use engine_api::{
    BoundAudioAsset, DefinitionPresentationBinding, FrozenOwnerKey, PresentationAssetRef,
    PresentationDefinitionKind, PresentationDigestAlgorithm, PresentationLocalId,
    QualifiedDefinitionKey,
};
use engine_native::catalog::CardKind;
use engine_native::content::extension::ExtensionPack;
use engine_native::rvm_bridge::VmFilterPolicy;

use crate::{faction_of, gender_of, invalid, BridgeError};

pub use original::{
    attach_embedded_original_presentation, attach_original_presentation_bindings,
    embedded_original_presentation_input, sha256_hex, verify_original_presentation,
    verify_original_presentation_with_pins, OriginalPresentationInput,
    VerifiedOriginalPresentation, EXPECTED_ORIGINAL_ADOPTED_REVISION,
    EXPECTED_ORIGINAL_ASSET_MANIFEST_SHA256, EXPECTED_ORIGINAL_CATALOG_SHA256,
    EXPECTED_ORIGINAL_REPOSITORY,
};

/// Explicit SDK `definition_id` linkages parsed from `cards.json` and `characters.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BridgedDefinitionIds {
    /// Optional `character_index[].id` per entry in `pack.generals`.
    pub general_definition_ids: Vec<Option<u32>>,
    /// Optional `card_index[].card_def_id` per entry in `pack.cards`.
    pub card_definition_ids: Vec<Option<u32>>,
    /// Optional `skill_index[].skill_def_id` keyed by interned host skill name.
    pub host_skill_definition_ids: BTreeMap<&'static str, Option<u32>>,
}

impl BridgedDefinitionIds {
    /// Merge `other` into `self`, rejecting conflicting `definition_id`s on shared host skills.
    pub fn merge(&mut self, other: BridgedDefinitionIds) -> Result<(), BridgeError> {
        self.general_definition_ids
            .extend(other.general_definition_ids);
        self.card_definition_ids.extend(other.card_definition_ids);
        for (skill_name, def_id) in other.host_skill_definition_ids {
            match (
                self.host_skill_definition_ids.get(skill_name).copied().flatten(),
                def_id,
            ) {
                (Some(existing), Some(incoming)) if existing != incoming => {
                    return Err(invalid(
                        "skills[].definition_id",
                        format!("宿主技能 {skill_name} 的 definition_id 冲突: {existing} vs {incoming}"),
                    ));
                }
                (None, Some(incoming)) => {
                    self.host_skill_definition_ids
                        .insert(skill_name, Some(incoming));
                }
                (None, None) => {
                    self.host_skill_definition_ids.entry(skill_name).or_insert(None);
                }
                (Some(_), _) => {}
            }
        }
        Ok(())
    }
}

pub(crate) fn push_unique_source(dest: &mut Vec<BoundAudioAsset>, candidate: BoundAudioAsset) {
    let is_dup = dest.iter().any(|existing| match (&existing.reference, &candidate.reference) {
        (
            PresentationAssetRef::Original {
                source_category: c1,
                source_path: p1,
                ..
            },
            PresentationAssetRef::Original {
                source_category: c2,
                source_path: p2,
                ..
            },
        ) => c1 == c2 && p1 == p2,
        _ => existing == &candidate,
    });
    if !is_dup {
        dest.push(candidate);
    }
}

/// Convert a [`LockedPackage`] binding to its [`FrozenOwnerKey::Package`] representation.
pub fn frozen_package_owner_key(binding: &LockedPackage) -> FrozenOwnerKey {
    FrozenOwnerKey::Package {
        namespace: binding.namespace.clone(),
        package_id: binding.package_id.clone(),
        version: binding.version.clone(),
        logic_hash: binding.logic_hash.clone(),
        content_hash: binding.content_hash.clone(),
    }
}

/// Resolve and verify one package-local audio `asset_id` against `copy` and `parsed`.
pub fn resolve_package_audio_asset(
    copy: &FrozenPackage,
    parsed: &ContentPackage,
    asset_id: &str,
) -> Result<BoundAudioAsset, String> {
    let relative = parsed
        .asset_paths
        .get(asset_id)
        .ok_or_else(|| format!("missing asset_path for audio asset {asset_id}"))?;
    let manifest = parsed
        .asset_manifest
        .as_ref()
        .ok_or_else(|| format!("missing asset_manifest for audio asset {asset_id}"))?;
    let entry = manifest
        .entries
        .iter()
        .find(|e| e.asset_id == asset_id)
        .ok_or_else(|| format!("missing asset_manifest entry for audio asset {asset_id}"))?;
    if entry.kind != resource_schema::AssetKind::Audio {
        return Err(format!(
            "asset {asset_id} has kind {:?}, expected Audio",
            entry.kind
        ));
    }
    let bytes = copy
        .assets
        .get(relative)
        .ok_or_else(|| format!("missing frozen asset bytes for {asset_id} at {relative}"))?;
    if bytes.is_empty() {
        return Err(format!("empty audio asset payload for {asset_id}"));
    }
    let actual_hash = blake3::hash(bytes);
    if actual_hash.as_bytes() != &entry.source_hash {
        return Err(format!("audio asset {asset_id} source_hash mismatch"));
    }
    Ok(BoundAudioAsset {
        source: frozen_package_owner_key(&copy.binding),
        reference: PresentationAssetRef::Package {
            namespace: copy.binding.namespace.clone(),
            package_id: copy.binding.package_id.clone(),
            asset_id: asset_id.to_string(),
        },
        digest_algorithm: PresentationDigestAlgorithm::Blake3,
        digest: actual_hash.to_hex().to_string(),
        size: bytes.len() as u64,
    })
}

fn resolve_voice_map(
    copy: &FrozenPackage,
    parsed: &ContentPackage,
    voices: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<BoundAudioAsset>>, String> {
    let mut out: BTreeMap<String, Vec<BoundAudioAsset>> = BTreeMap::new();
    for (voice_key, asset_id) in voices {
        let bound = resolve_package_audio_asset(copy, parsed, asset_id)?;
        out.entry(voice_key.clone()).or_default().push(bound);
    }
    if !out.contains_key("default") {
        if let Some(act) = out.get("activate").cloned() {
            out.insert("default".to_string(), act);
        } else if let Some(die) = out.get("die").cloned() {
            out.insert("default".to_string(), die);
        }
    }
    Ok(out)
}

/// Validate and attach [`DefinitionPresentationBinding`]s for a single [`FrozenPackage`]
/// within an optional multi-package [`VerifiedLockedContent`] set.
pub fn attach_frozen_package_presentation_bindings(
    pack: &mut ExtensionPack,
    definition_ids: &BridgedDefinitionIds,
    copy: &FrozenPackage,
    parsed: &ContentPackage,
    all_packages: &[(&FrozenPackage, &ContentPackage)],
) -> Result<(), String> {
    // 1. Validate all declared voice_assets in `parsed` up front so corrupt/empty audio rejects.
    for ch in &parsed.character_index {
        let _ = resolve_voice_map(copy, parsed, &ch.voice_assets)?;
    }
    for sk in &parsed.skill_index {
        let _ = resolve_voice_map(copy, parsed, &sk.voice_assets)?;
    }
    for cd in &parsed.card_index {
        let _ = resolve_voice_map(copy, parsed, &cd.voice_assets)?;
    }
    for skin in &parsed.skin_index.skins {
        let _ = resolve_voice_map(copy, parsed, &skin.voice_assets)?;
    }

    let owner = frozen_package_owner_key(&copy.binding);

    // Build skill lookup across `all_packages` (defaulting to current package).
    let mut all_skills: BTreeMap<u32, &str> = BTreeMap::new();
    for (_, pkg) in all_packages {
        for sk in &pkg.skill_index {
            all_skills.insert(sk.skill_def_id, sk.name.as_str());
        }
    }
    for sk in &parsed.skill_index {
        all_skills.insert(sk.skill_def_id, sk.name.as_str());
    }

    // 2. Attach VM skill bindings and track claimed skill_def_ids.
    let mut claimed_skill_ids = BTreeSet::new();
    for vm_decl in &mut pack.vm_skills {
        if vm_decl.r05_identity.is_some() {
            continue;
        }
        let skill_def_id = match &vm_decl.filter {
            VmFilterPolicy::Declared { skill_def_id, .. } => Some(*skill_def_id),
            VmFilterPolicy::Handlers(set) => Some(set.skill_def_id),
            _ => None,
        };
        if let Some(id) = skill_def_id {
            let entry = parsed
                .skill_index
                .iter()
                .find(|s| s.skill_def_id == id)
                .ok_or_else(|| format!("VM skill {} missing skill_index entry {id}", vm_decl.name))?;
            if entry.name != vm_decl.name {
                return Err(format!(
                    "VM skill {} name mismatch with skill_index {}: {}",
                    vm_decl.name, id, entry.name
                ));
            }
            if id != 0 {
                claimed_skill_ids.insert(id);
                let voice_assets = resolve_voice_map(copy, parsed, &entry.voice_assets)?;
                vm_decl.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
                    definition: QualifiedDefinitionKey {
                        owner: owner.clone(),
                        kind: PresentationDefinitionKind::Skill,
                        local_id: PresentationLocalId::Package(id),
                    },
                    voice_assets,
                }));
            }
        }
    }

    // 3. Attach host skill bindings when `definition_id` is provided.
    for host_decl in &mut pack.host_skills {
        if let Some(Some(id)) = definition_ids.host_skill_definition_ids.get(host_decl.name) {
            let id = *id;
            if !claimed_skill_ids.insert(id) {
                return Err(format!("duplicate skill definition_id {id} in package"));
            }
            let entry = parsed
                .skill_index
                .iter()
                .find(|s| s.skill_def_id == id)
                .ok_or_else(|| {
                    format!(
                        "host skill {} references missing skill_index definition_id {id}",
                        host_decl.name
                    )
                })?;
            if entry.name != host_decl.name {
                return Err(format!(
                    "host skill {} definition_id {id} linkage mismatch: skill_index name is {}",
                    host_decl.name, entry.name
                ));
            }
            let voice_assets = resolve_voice_map(copy, parsed, &entry.voice_assets)?;
            host_decl.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
                definition: QualifiedDefinitionKey {
                    owner: owner.clone(),
                    kind: PresentationDefinitionKind::Skill,
                    local_id: PresentationLocalId::Package(id),
                },
                voice_assets,
            }));
        }
    }

    // 4. Attach general bindings when `definition_id` is provided.
    if definition_ids.general_definition_ids.len() > pack.generals.len() {
        return Err("general_definition_ids length exceeds pack.generals".into());
    }
    let mut claimed_general_ids = BTreeSet::new();
    for (idx, general) in pack.generals.iter_mut().enumerate() {
        let Some(char_id) = definition_ids
            .general_definition_ids
            .get(idx)
            .copied()
            .flatten()
        else {
            continue;
        };
        if !claimed_general_ids.insert(char_id) {
            return Err(format!("duplicate general definition_id {char_id}"));
        }
        let entry = parsed
            .character_index
            .iter()
            .find(|c| c.id == char_id)
            .ok_or_else(|| {
                format!(
                    "general {} references missing character_index definition_id {char_id}",
                    general.name
                )
            })?;
        if entry.max_hp != general.max_hp {
            return Err(format!(
                "general {} definition_id {char_id} max_hp mismatch: {} vs {}",
                general.name, entry.max_hp, general.max_hp
            ));
        }
        if !entry.sex.is_empty() {
            let expected_gender = gender_of(&entry.sex).map_err(|e| e.to_string())?;
            if expected_gender != general.gender {
                return Err(format!(
                    "general {} definition_id {char_id} sex mismatch",
                    general.name
                ));
            }
        }
        if !entry.group.is_empty() {
            let expected_faction = faction_of(&entry.group).map_err(|e| e.to_string())?;
            if expected_faction != general.faction {
                return Err(format!(
                    "general {} definition_id {char_id} group mismatch",
                    general.name
                ));
            }
        }
        for &sk_id in &entry.skills {
            let sk_name = all_skills.get(&sk_id).ok_or_else(|| {
                format!(
                    "character_index {char_id} references unknown skill_def_id {sk_id}"
                )
            })?;
            if !general.skills.contains(sk_name) {
                return Err(format!(
                    "general {} missing skill {sk_name} declared on character_index {char_id}",
                    general.name
                ));
            }
        }

        let mut voice_assets = resolve_voice_map(copy, parsed, &entry.voice_assets)?;
        attach_character_skin_voices(
            char_id,
            &mut voice_assets,
            copy,
            parsed,
            all_packages,
        )?;
        general.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
            definition: QualifiedDefinitionKey {
                owner: owner.clone(),
                kind: PresentationDefinitionKind::General,
                local_id: PresentationLocalId::Package(char_id),
            },
            voice_assets,
        }));
    }

    // 5. Attach card bindings when `definition_id` is provided.
    if definition_ids.card_definition_ids.len() > pack.cards.len() {
        return Err("card_definition_ids length exceeds pack.cards".into());
    }
    let mut claimed_card_ids = BTreeSet::new();
    for (idx, card_def) in pack.cards.iter_mut().enumerate() {
        let Some(card_id) = definition_ids
            .card_definition_ids
            .get(idx)
            .copied()
            .flatten()
        else {
            continue;
        };
        if card_id == 0 || !claimed_card_ids.insert(card_id) {
            return Err(format!("invalid or duplicate card definition_id {card_id}"));
        }
        let entry = parsed
            .card_index
            .iter()
            .find(|c| c.card_def_id == card_id)
            .ok_or_else(|| {
                format!(
                    "card {} references missing card_index definition_id {card_id}",
                    card_def.name
                )
            })?;
        let expected_kind = CardKind::from_key(&entry.kind)
            .ok_or_else(|| format!("card_index {card_id} has unknown kind {}", entry.kind))?;
        if expected_kind != card_def.kind {
            return Err(format!(
                "card {} definition_id {card_id} kind mismatch: {:?} vs {:?}",
                card_def.name, expected_kind, card_def.kind
            ));
        }
        if card_def.kind == CardKind::Weapon
            && entry.weapon_range > 0
            && u32::from(entry.weapon_range) != card_def.range
        {
            return Err(format!(
                "card {} definition_id {card_id} weapon_range mismatch: {} vs {}",
                card_def.name, entry.weapon_range, card_def.range
            ));
        }
        for &eq_sk_id in &entry.equip_skills {
            let sk_name = all_skills.get(&eq_sk_id).ok_or_else(|| {
                format!("card_index {card_id} references unknown equip_skill {eq_sk_id}")
            })?;
            if !card_def.equip_skills.contains(sk_name) {
                return Err(format!(
                    "card {} missing equip_skill {sk_name} declared on card_index {card_id}",
                    card_def.name
                ));
            }
        }
        let voice_assets = resolve_voice_map(copy, parsed, &entry.voice_assets)?;
        card_def.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
            definition: QualifiedDefinitionKey {
                owner: owner.clone(),
                kind: PresentationDefinitionKind::Card,
                local_id: PresentationLocalId::Package(card_id),
            },
            voice_assets,
        }));
    }

    Ok(())
}

fn attach_character_skin_voices(
    char_id: u32,
    voice_assets: &mut BTreeMap<String, Vec<BoundAudioAsset>>,
    own_copy: &FrozenPackage,
    own_parsed: &ContentPackage,
    all_packages: &[(&FrozenPackage, &ContentPackage)],
) -> Result<(), String> {
    let single = [(own_copy, own_parsed)];
    let packages = if all_packages.is_empty() {
        &single[..]
    } else {
        all_packages
    };

    // Direct skins targeting `char_id`
    for (skin_copy, skin_pkg) in packages {
        for skin in skin_pkg
            .skin_index
            .skins
            .iter()
            .filter(|s| s.character_id == char_id)
        {
            for (vkey, asset_id) in &skin.voice_assets {
                let bound = resolve_package_audio_asset(skin_copy, skin_pkg, asset_id)?;
                push_unique_source(
                    voice_assets
                        .entry(format!("{vkey}:form:{}", skin.name_key))
                        .or_default(),
                    bound.clone(),
                );
                push_unique_source(
                    voice_assets
                        .entry(format!("form:{}:{vkey}", skin.name_key))
                        .or_default(),
                    bound.clone(),
                );
                push_unique_source(
                    voice_assets
                        .entry(format!("skin:{}:{vkey}", skin.id))
                        .or_default(),
                    bound.clone(),
                );
                if let PackageSkinPresentation::Substitute { rule, .. } = &skin.presentation {
                    push_unique_source(
                        voice_assets
                            .entry(format!("{vkey}:form:{rule}"))
                            .or_default(),
                        bound.clone(),
                    );
                }
                if !voice_assets.contains_key(vkey) {
                    push_unique_source(voice_assets.entry(vkey.clone()).or_default(), bound);
                }
            }
        }
        // Alias entries targeting `char_id`
        for alias in skin_pkg
            .skin_index
            .aliases
            .iter()
            .filter(|a| a.character_id == char_id)
        {
            let (src_copy, src_pkg, src_char) = packages
                .iter()
                .find_map(|(c, p)| {
                    p.character_index
                        .iter()
                        .find(|ch| ch.id == alias.source_character_id)
                        .map(|ch| (*c, *p, ch))
                })
                .ok_or_else(|| {
                    format!(
                        "skin alias for character {char_id} references missing source character {}",
                        alias.source_character_id
                    )
                })?;
            for (target_vkey, source_vkey) in &alias.voice_mappings {
                if let Some(asset_id) = src_char.voice_assets.get(source_vkey) {
                    let bound = resolve_package_audio_asset(src_copy, src_pkg, asset_id)?;
                    push_unique_source(
                        voice_assets.entry(target_vkey.clone()).or_default(),
                        bound,
                    );
                }
                for (sc, sp) in packages {
                    for skin in sp
                        .skin_index
                        .skins
                        .iter()
                        .filter(|s| s.character_id == alias.source_character_id)
                    {
                        if let Some(asset_id) = skin.voice_assets.get(source_vkey) {
                            let bound = resolve_package_audio_asset(sc, sp, asset_id)?;
                            push_unique_source(
                                voice_assets
                                    .entry(format!("{target_vkey}:form:{}", skin.name_key))
                                    .or_default(),
                                bound.clone(),
                            );
                            if !voice_assets.contains_key(target_vkey) {
                                push_unique_source(
                                    voice_assets.entry(target_vkey.clone()).or_default(),
                                    bound,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    if !voice_assets.contains_key("default") {
        if let Some(act) = voice_assets.get("activate").cloned() {
            voice_assets.insert("default".to_string(), act);
        } else if let Some(die) = voice_assets.get("die").cloned() {
            voice_assets.insert("default".to_string(), die);
        }
    }
    Ok(())
}
