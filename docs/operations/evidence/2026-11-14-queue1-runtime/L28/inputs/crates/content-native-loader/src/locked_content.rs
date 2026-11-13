//! Controlled locked content factory (`R05 v4/v5`).
//!
//! Bridges [`VerifiedLockedContent`] deterministically into `Engine` instances without
//! accepting or returning bare `ExtensionPack` slices on the locked product path (`v5 §1/§3`).

use crate::{
    attach_frozen_package_presentation_bindings, attach_original_presentation_bindings,
    bridge_generals, bridge_generals_with_vm_package, bridge_pack, bridge_pile,
    bridge_vm_skill_decls, embedded_original_presentation_input, verify_original_presentation,
    BridgedDefinitionIds, BridgedPack, OriginalPresentationInput,
};
pub use content_runtime::{
    prepare_locked_content, validate_frozen_package, verify_locked_content, ContentLock,
    FrozenPackage, LockedContentEnvelope, LockedPackage, VerifiedLockedContent,
};
use content_schema::ContentPackage;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::content::names;
use engine_native::content::pile::extension_pack_from_pile;
use engine_native::{Engine, SnapshotError, StateSnapshot};
use std::collections::BTreeMap;

/// Match opening mode for the controlled locked engine factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockedStartMode {
    /// Prototype mode (`Engine::new` / `Engine::new_with_extensions`).
    Prototype,
    /// Identity mode (`Engine::try_new_identity_with_extensions`).
    Identity,
}

fn bridge_single_frozen(
    copy: &FrozenPackage,
    parsed: &ContentPackage,
    all_packages: &[(&FrozenPackage, &ContentPackage)],
    prior_packs: &[ExtensionPack],
) -> Result<(ExtensionPack, Vec<String>), String> {
    let pid = parsed.manifest.package_id.clone();
    let ns = parsed.manifest.namespace.clone();
    let cards = copy.text_asset("assets/cards.json")?;
    let characters = match copy.text_asset("assets/character.json")? {
        Some(c) => Some(c),
        None => copy.text_asset("assets/characters.json")?,
    };
    let pile = copy.text_asset("assets/pile.json")?;

    let outcome = bridge_vm_skill_decls(parsed).map_err(|e| e.to_string())?;
    let vm_skills = outcome.decls;
    let mut definition_ids = BridgedDefinitionIds::default();
    let (mut pack, deferred) = if let Some(json) = cards.as_ref() {
        let b: BridgedPack = bridge_pack(json).map_err(|e| e.to_string())?;
        definition_ids
            .merge(b.definition_ids)
            .map_err(|e| e.to_string())?;
        (b.pack, b.deferred)
    } else {
        (
            ExtensionPack {
                def: ExtensionDef {
                    name: names::intern(pid),
                    version: parsed
                        .manifest
                        .version
                        .split('.')
                        .next()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0),
                    description: names::intern(ns),
                },
                generals: Vec::new(),
                cards: Vec::new(),
                host_skills: Vec::new(),
                vm_skills: Vec::new(),
                init_hook: InitHook::None,
                enabled: true,
            },
            Vec::new(),
        )
    };
    pack.vm_skills.extend(vm_skills);
    if let Some(json) = characters {
        let generals = if parsed.modules.is_empty() {
            bridge_generals(&json)
        } else {
            bridge_generals_with_vm_package(&json, parsed)
        }
        .map_err(|e| e.to_string())?;
        definition_ids
            .merge(generals.definition_ids)
            .map_err(|e| e.to_string())?;
        pack.generals.extend(generals.generals);
        for decl in generals.host_skills {
            match pack.host_skills.iter().find(|d| d.name == decl.name) {
                Some(d) if d.behavior != decl.behavior => {
                    return Err(format!("重复技能 {}", decl.name));
                }
                Some(_) => {}
                None => pack.host_skills.push(decl),
            }
        }
    }
    if let Some(json) = pile {
        let spec = bridge_pile(&json).map_err(|e| e.to_string())?;
        let addition = extension_pack_from_pile(pack.def.clone(), pack.def.name, &spec)
            .map_err(|e| format!("pile: {e:?}"))?;
        pack.cards.extend(addition.cards);
    }
    attach_frozen_package_presentation_bindings(
        &mut pack,
        &definition_ids,
        copy,
        parsed,
        all_packages,
    )?;
    crate::decision34_pack9::attach_formal_package_upstream_rules(
        &parsed.manifest.namespace,
        &parsed.manifest.package_id,
        &mut pack,
        prior_packs,
    );
    pack.def.name = names::intern(format!(
        "installed:{}",
        serde_json::to_string(&copy.binding).map_err(|e| e.to_string())?
    ));
    Ok((pack, deferred))
}

fn bridge_verified_packs(
    verified: &VerifiedLockedContent,
) -> Result<(Vec<ExtensionPack>, Vec<String>), String> {
    let pairs: Vec<(&FrozenPackage, &ContentPackage)> = verified
        .copies()
        .iter()
        .zip(verified.packages().iter())
        .collect();
    let mut packs = Vec::with_capacity(pairs.len());
    let mut deferred_all = Vec::new();
    for &(copy, parsed) in &pairs {
        let (pack, deferred) = bridge_single_frozen(copy, parsed, &pairs, &packs)?;
        packs.push(pack);
        deferred_all.extend(deferred);
    }
    Ok((packs, deferred_all))
}

/// Deep-validate a single raw `package.json` text and its raw declared asset bytes at install/import time (`R05 v4 §1`).
pub fn validate_and_freeze_raw_package(
    package_json: String,
    assets: BTreeMap<String, Vec<u8>>,
) -> Result<FrozenPackage, String> {
    let (frozen, parsed) = validate_frozen_package(package_json, assets)?;
    let _ = bridge_single_frozen(&frozen, &parsed, &[(&frozen, &parsed)], &[])?;
    Ok(frozen)
}

/// Verify and bridge a full set of [`FrozenPackage`] copies into [`VerifiedLockedContent`] (`R05 v4 §2`).
pub fn prepare_locked_content_with_bridge(
    copies: Vec<FrozenPackage>,
) -> Result<VerifiedLockedContent, String> {
    let verified = prepare_locked_content(copies)?;
    let (packs, _) = bridge_verified_packs(&verified)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    if !refs.is_empty() {
        engine_native::content::standard::standard_catalog_with_extensions(&refs)
            .map_err(|e| format!("{e:?}"))?;
    }
    Ok(verified)
}

/// Verify an existing `(ContentLock, copies)` pair and ensure its catalog bridges cleanly (`R05 v4 §2`).
pub fn verify_locked_content_with_bridge(
    lock: &ContentLock,
    copies: &[FrozenPackage],
) -> Result<VerifiedLockedContent, String> {
    let verified = verify_locked_content(lock, copies)?;
    let (packs, _) = bridge_verified_packs(&verified)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    if !refs.is_empty() {
        engine_native::content::standard::standard_catalog_with_extensions(&refs)
            .map_err(|e| format!("{e:?}"))?;
    }
    Ok(verified)
}

/// Return deferred card names for a verified locked content set without exposing `ExtensionPack`.
pub fn deferred_cards_for_locked_content(
    verified: &VerifiedLockedContent,
) -> Result<Vec<String>, String> {
    let (_, deferred) = bridge_verified_packs(verified)?;
    Ok(deferred)
}

/// Controlled locked engine creation factory (`R05 v5 §1/§3`): accepts only [`VerifiedLockedContent`], never bare `packs`.
pub fn start_locked_engine(
    seed: u64,
    seats: usize,
    mode: LockedStartMode,
    verified: &VerifiedLockedContent,
) -> Result<Engine, String> {
    start_locked_engine_with_original_presentation(
        seed,
        seats,
        mode,
        verified,
        Some(&embedded_original_presentation_input()),
    )
}

/// Controlled locked engine creation factory with optional verified original presentation inputs (`I02 §2`).
pub fn start_locked_engine_with_original_presentation(
    seed: u64,
    seats: usize,
    mode: LockedStartMode,
    verified: &VerifiedLockedContent,
    original_presentation: Option<&OriginalPresentationInput<'_>>,
) -> Result<Engine, String> {
    let verified_orig = match original_presentation {
        Some(input) => Some(verify_original_presentation(input)?),
        None => None,
    };
    let (packs, _) = bridge_verified_packs(verified)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    let mut engine = match (mode, refs.is_empty()) {
        (LockedStartMode::Prototype, true) => Engine::new(seed, seats),
        (LockedStartMode::Prototype, false) => {
            Engine::new_with_extensions(seed, seats, &refs).map_err(|e| format!("{e:?}"))?
        }
        (LockedStartMode::Identity, _) => {
            Engine::try_new_identity_with_extensions(seed, seats, &refs)?
        }
    };
    if let Some(orig) = &verified_orig {
        engine.try_with_catalog_mutation(|cat| attach_original_presentation_bindings(cat, orig))?;
    }
    Ok(engine)
}

/// Controlled locked engine restore factory (`R05 v5 §1/§3`): accepts only [`VerifiedLockedContent`], never bare `packs`.
pub fn restore_locked_engine(
    snapshot: &StateSnapshot,
    verified: &VerifiedLockedContent,
) -> Result<Engine, SnapshotError> {
    restore_locked_engine_with_original_presentation(
        snapshot,
        verified,
        Some(&embedded_original_presentation_input()),
    )
}

/// Controlled locked engine restore factory with optional verified original presentation inputs (`I02 §2`).
pub fn restore_locked_engine_with_original_presentation(
    snapshot: &StateSnapshot,
    verified: &VerifiedLockedContent,
    original_presentation: Option<&OriginalPresentationInput<'_>>,
) -> Result<Engine, SnapshotError> {
    let verified_orig = match original_presentation {
        Some(input) => {
            Some(verify_original_presentation(input).map_err(|_| SnapshotError::UnknownMachine)?)
        }
        None => None,
    };
    let (packs, _) =
        bridge_verified_packs(verified).map_err(|_| SnapshotError::UnknownMachine)?;
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    let mut engine = if refs.is_empty() {
        Engine::restore(snapshot)?
    } else {
        Engine::restore_with_extensions(snapshot, &refs)?
    };
    if let Some(orig) = &verified_orig {
        engine
            .try_with_catalog_mutation(|cat| attach_original_presentation_bindings(cat, orig))
            .map_err(|_| SnapshotError::UnknownMachine)?;
    }
    Ok(engine)
}
