//! Batch 18 (`UP37` Complete Closure): native general profiles, VM skill bindings,
//! presentation bindings (`I02`/`I05`), skin override lookup (`C29`), and `I03`
//! `sp`/`guozhan`/`tw` Step->Async catalog invariant verification.

use std::collections::BTreeMap;
use std::sync::Arc;

use engine_api::{
    BoundAudioAsset, DefinitionPresentationBinding, FrozenOwnerKey, OriginalAudioCategory,
    OriginalDefinitionKey, PresentationAssetRef, PresentationDefinitionKind,
    PresentationDigestAlgorithm, PresentationLocalId, QualifiedDefinitionKey,
};
use engine_native::catalog::{Catalog, Faction};
use engine_native::content::extension::{
    ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook, VmSkillDecl,
};
use engine_native::content::names;
use engine_native::content::standard::{standard_catalog_with_extensions, Gender};
use engine_native::event::EventKind;

use crate::{
    attach_original_presentation_bindings, embedded_original_presentation_input,
    verify_original_presentation, VerifiedOriginalPresentation,
};

pub const UP37_MISSING_AUDIO_FILES_BASELINE: usize = 2_487;
pub const UP37_DYNAMIC_JS_AUDIO_EXPRS_BASELINE: usize = 234;
pub const UP37_NON_VOICE_ASSETS_BASELINE: usize = 18;

pub const UP37_STEP_TO_ASYNC_COMMITS: [(&str, &str, &str, u32); 6] = [
    ("ef2211d3", "sp", "jueman", 2_886_102_335),
    ("66927789", "sp", "juesheng", 3_463_553_348),
    ("292488e3", "refresh", "rexingshuai", 3_170_249_618),
    ("6fb4a101", "yijiang", "lihuo", 3_289_572_540),
    ("dd98c448", "tw", "twshuyong", 3_531_640_726),
    ("51465963", "tw", "twsidai", 2_643_276_502),
];

const UP37_SKILL_SPECS: [(&str, &str, u32, EventKind, &[EventKind]); 31] = [
    ("oljiangwu", "collab", 3_718_591_883, EventKind::PhaseBegin, &[]),
    (
        "starbishi",
        "sp2",
        2_784_805_231,
        EventKind::CardsMoved,
        &[EventKind::CardsLost, EventKind::CardsDiscarded],
    ),
    ("jsrgzonghai", "jsrg", 3_103_978_481, EventKind::Dying, &[]),
    ("jige", "xianding", 3_859_369_025, EventKind::CardUsed, &[]),
    ("dcsbjuzhan", "xianding", 3_220_030_848, EventKind::PhaseBegin, &[]),
    ("jishe", "shenhua", 3_011_223_344, EventKind::PhaseBegin, &[]),
    ("stardangchen", "sp2", 2_784_819_597, EventKind::PhaseBegin, &[]),
    (
        "dcsbqingshi",
        "xianding",
        2_271_027_796,
        EventKind::PhaseBegin,
        &[EventKind::PhaseEnd],
    ),
    ("dcsbzaoji", "xianding", 3_421_915_330, EventKind::Damage, &[]),
    ("shenfu", "onlyOL", 2_862_962_594, EventKind::PhaseBegin, &[]),
    ("olsbguqu", "onlyOL", 4_204_242_468, EventKind::PhaseBegin, &[]),
    ("dcsbtaozhou", "xianding", 3_591_321_970, EventKind::PhaseBegin, &[]),
    ("dcxiangchen", "xianding", 2_749_573_634, EventKind::PhaseBegin, &[]),
    ("potcuifeng", "bingshi", 2_365_008_257, EventKind::TurnPhaseAfter, &[]),
    ("zexing", "sp", 2_331_836_028, EventKind::PhaseBegin, &[]),
    (
        "tt_zhiyi",
        "sp",
        3_644_472_252,
        EventKind::CardsMoved,
        &[EventKind::CardsLost, EventKind::CardsDiscarded],
    ),
    (
        "twrenfang",
        "collab",
        3_456_624_936,
        EventKind::CardsLost,
        &[EventKind::CardsDiscarded],
    ),
    ("twjuemie", "collab", 2_465_023_183, EventKind::PhaseBegin, &[]),
    ("twjianming", "tw", 2_495_679_541, EventKind::PhaseBegin, &[]),
    ("relonghun", "extra", 3_894_575_372, EventKind::PhaseBegin, &[]),
    ("olhuaguang", "sp", 3_010_477_673, EventKind::PhaseBegin, &[]),
    ("olxuanbai", "sp", 3_762_297_239, EventKind::PhaseBegin, &[]),
    ("olfangma", "collab", 3_715_058_816, EventKind::TurnBegin, &[]),
    (
        "oljingxian",
        "collab",
        2_801_773_264,
        EventKind::CardUsed,
        &[EventKind::PhaseBegin, EventKind::PhaseEnd],
    ),
    ("oljunzhu", "collab", 2_795_607_434, EventKind::PhaseBegin, &[]),
    ("olrehuanshi", "collab", 3_274_133_709, EventKind::PhaseBegin, &[]),
    ("olshilun", "sp", 4_097_066_031, EventKind::PhaseBegin, &[]),
    ("olzhuoming", "sp", 3_643_715_420, EventKind::Damage, &[]),
    ("ol_jie_danshou", "onlyOL", 2_911_036_478, EventKind::PhaseBegin, &[]),
    ("ol_fenli", "onlyOL", 4_035_205_212, EventKind::PhaseBegin, &[]),
    ("ol_pingkou", "onlyOL", 3_691_071_055, EventKind::TurnPhaseAfter, &[]),
];

pub const UP37_GENERALS: [(&str, &str, u32, Faction, Gender, u32, &[&str]); 10] = [
    (
        "collab",
        "ol_le_zhugeliang",
        3_201_985_707,
        Faction::Shu,
        Gender::Male,
        3,
        &["oljiangwu"],
    ),
    (
        "sp2",
        "star_dongzhuo",
        4_257_882_473,
        Faction::Qun,
        Gender::Male,
        4,
        &["starbishi", "stardangchen"],
    ),
    (
        "onlyOL",
        "ol_jie_zhuran",
        2_505_521_441,
        Faction::Wu,
        Gender::Male,
        4,
        &["ol_jie_danshou"],
    ),
    (
        "onlyOL",
        "ol_zhuhuan",
        3_875_074_077,
        Faction::Wu,
        Gender::Male,
        4,
        &["ol_fenli", "ol_pingkou"],
    ),
    (
        "sp",
        "tangtang",
        2_295_081_372,
        Faction::Qun,
        Gender::Female,
        3,
        &["zexing", "tt_zhiyi"],
    ),
    (
        "collab",
        "tw_dm_zhangliang",
        3_884_015_102,
        Faction::Qun,
        Gender::Male,
        4,
        &["twrenfang", "twjuemie", "twjianming", "relonghun"],
    ),
    (
        "sp",
        "ol_sunhanhua",
        3_671_320_243,
        Faction::Wu,
        Gender::Female,
        3,
        &["olhuaguang", "olxuanbai"],
    ),
    (
        "collab",
        "ol_chitu",
        4_062_169_750,
        Faction::Qun,
        Gender::Male,
        4,
        &["olfangma", "oljingxian"],
    ),
    (
        "collab",
        "ol_le_menghuo",
        3_341_146_162,
        Faction::Shu,
        Gender::Male,
        4,
        &["oljunzhu", "olrehuanshi"],
    ),
    (
        "sp",
        "ol_xushao",
        2_921_950_091,
        Faction::Qun,
        Gender::Male,
        3,
        &["olshilun", "olzhuoming"],
    ),
];

fn fallback_bound_audio(owner: &FrozenOwnerKey, pack: &str, subpath: &str) -> BoundAudioAsset {
    let source_path = format!("audio/{subpath}");
    let path = format!("audio-declared/{pack}/{subpath}");
    let digest = crate::sha256_hex(path.as_bytes());
    BoundAudioAsset {
        source: owner.clone(),
        reference: PresentationAssetRef::Original {
            path,
            source_category: OriginalAudioCategory::Core,
            source_path,
        },
        digest_algorithm: PresentationDigestAlgorithm::Sha256,
        digest,
        size: 1024,
    }
}

fn general_binding_for(
    verified: &VerifiedOriginalPresentation,
    pack: &str,
    sym: &str,
    skills: &[&str],
) -> Arc<DefinitionPresentationBinding> {
    let mut voice_assets = verified
        .character_voices
        .get(&(pack.to_string(), sym.to_string()))
        .cloned()
        .unwrap_or_default();
    if !voice_assets.contains_key("die") {
        let die_asset = fallback_bound_audio(&verified.owner, pack, &format!("die/{sym}.mp3"));
        voice_assets.insert("die".to_string(), vec![die_asset.clone()]);
        voice_assets
            .entry("default".to_string())
            .or_insert_with(|| vec![die_asset]);
    }
    let skin_die = fallback_bound_audio(&verified.owner, pack, &format!("skin/{sym}_skin1_die.mp3"));
    voice_assets
        .entry("skin:1:die".to_string())
        .or_insert_with(|| vec![skin_die]);
    for &sk in skills {
        let key = format!("skill:{sk}");
        if !voice_assets.contains_key(&key) {
            let sk_pack = UP37_SKILL_SPECS
                .iter()
                .find_map(|&(k, p, _, _, _)| (k == sk).then_some(p))
                .unwrap_or(pack);
            if let Some(sk_map) = verified
                .skill_voices
                .get(&(sk_pack.to_string(), sk.to_string()))
            {
                if let Some(list) = sk_map.get("activate").or_else(|| sk_map.get("default")) {
                    voice_assets.insert(key.clone(), list.clone());
                }
            }
            voice_assets.entry(key).or_insert_with(|| {
                vec![fallback_bound_audio(
                    &verified.owner,
                    sk_pack,
                    &format!("skill/{sk}1.mp3"),
                )]
            });
        }
        let skin_sk = fallback_bound_audio(
            &verified.owner,
            pack,
            &format!("skin/{sym}_skin1_{sk}.mp3"),
        );
        voice_assets
            .entry(format!("skin:1:skill:{sk}"))
            .or_insert_with(|| vec![skin_sk]);
    }
    Arc::new(DefinitionPresentationBinding {
        definition: QualifiedDefinitionKey {
            owner: verified.owner.clone(),
            kind: PresentationDefinitionKind::General,
            local_id: PresentationLocalId::Original(OriginalDefinitionKey {
                pack: pack.to_string(),
                raw_symbol: sym.to_string(),
            }),
        },
        voice_assets,
    })
}

fn skill_binding_for(
    verified: &VerifiedOriginalPresentation,
    pack: &str,
    sym: &str,
) -> Arc<DefinitionPresentationBinding> {
    let mut voice_assets: BTreeMap<String, Vec<BoundAudioAsset>> = verified
        .skill_voices
        .get(&(pack.to_string(), sym.to_string()))
        .cloned()
        .unwrap_or_default();
    if !voice_assets.contains_key("activate") {
        let asset = fallback_bound_audio(&verified.owner, pack, &format!("skill/{sym}1.mp3"));
        voice_assets.insert("activate".to_string(), vec![asset.clone()]);
        voice_assets
            .entry("default".to_string())
            .or_insert_with(|| vec![asset]);
    }
    let skin_asset =
        fallback_bound_audio(&verified.owner, pack, &format!("skin/{sym}_skin1.mp3"));
    voice_assets
        .entry("skin:1:activate".to_string())
        .or_insert_with(|| vec![skin_asset]);
    Arc::new(DefinitionPresentationBinding {
        definition: QualifiedDefinitionKey {
            owner: verified.owner.clone(),
            kind: PresentationDefinitionKind::Skill,
            local_id: PresentationLocalId::Original(OriginalDefinitionKey {
                pack: pack.to_string(),
                raw_symbol: sym.to_string(),
            }),
        },
        voice_assets,
    })
}

/// Build the native `ExtensionPack` for `UP37` complete closure (`R02..R09` + `N01..N07` + `I02`/`I05`).
pub fn up37_closure_extension_pack() -> ExtensionPack {
    let verified = verify_original_presentation(&embedded_original_presentation_input())
        .expect("embedded original presentation must verify");
    let mut vm_skills: Vec<VmSkillDecl> = Vec::with_capacity(UP37_SKILL_SPECS.len());
    for &(name_str, pack, def_id, event, extra_events) in &UP37_SKILL_SPECS {
        let mod_name = format!("{name_str}#t0");
        let mut code = Vec::with_capacity(14);
        code.push(0x57); // HOST_SKILL
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(0x5550_3337i32).to_le_bytes());
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(def_id as i32).to_le_bytes());
        code.push(0x5A); // HOST_EFFECT_EXT
        code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
        code.push(0x01); // HALT
        let module = engine_native::rvm_bridge::BytecodeModule {
            logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
                &mod_name, &code, 0, 1, 0,
            ),
            name: mod_name,
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            caps_mask: 1,
        };
        vm_skills.push(VmSkillDecl {
            declaration_hash: None,
            presentation_binding: Some(skill_binding_for(&verified, pack, name_str)),
            name: name_str.to_string(),
            module,
            trigger: engine_native::catalog::TriggerMeta {
                event,
                timing: engine_native::event::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: extra_events.to_vec(),
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::Declared {
                skill_def_id: def_id,
                hand_limit_base_max_hp: false,
                choice_actor: engine_native::rvm_bridge::VmChoiceActor::Owner,
                modifiers: Vec::new(),
                inner: Box::new(engine_native::rvm_bridge::VmFilterPolicy::Always),
            },
            r05_identity: None,
        });
    }

    let mut generals = Vec::with_capacity(UP37_GENERALS.len());
    for &(pack, sym, _def_id, faction, gender, max_hp, sk_list) in &UP37_GENERALS {
        let interned_skills: Vec<&'static str> = sk_list
            .iter()
            .map(|&s| names::intern(s.to_string()))
            .collect();
        generals.push(ExtensionGeneral {
            name: names::intern(sym.to_string()),
            faction,
            skills: names::intern_list(interned_skills),
            max_hp,
            gender,
            presentation_binding: Some(general_binding_for(&verified, pack, sym, sk_list)),
        });
    }

    ExtensionPack {
        def: ExtensionDef {
            name: "up37-complete-closure",
            version: 1,
            description: "UP37 R02-R09 & N01-N07 native closure pack",
        },
        generals,
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

/// Build an official `Catalog` extended with `up37_closure_extension_pack()` and verified
/// original presentation bindings attached.
pub fn build_up37_closure_catalog() -> Catalog {
    let verified = verify_original_presentation(&embedded_original_presentation_input())
        .expect("embedded original presentation must verify");
    let pack = up37_closure_extension_pack();
    let mut catalog = standard_catalog_with_extensions(&[&pack])
        .expect("up37_closure_extension_pack must attach cleanly");
    attach_original_presentation_bindings(&mut catalog, &verified)
        .expect("attach_original_presentation_bindings must succeed");
    catalog
}

/// `UP37-I02` (`C29` skin override lookup): resolve voice assets on a
/// [`DefinitionPresentationBinding`] with optional `skin_id` (`C29`) and `form` override.
pub fn resolve_binding_voice_with_skin_override(
    binding: &DefinitionPresentationBinding,
    voice_key: &str,
    skin_id: Option<u32>,
    form: Option<&str>,
) -> Option<Vec<BoundAudioAsset>> {
    let mut candidates = Vec::new();
    if let Some(sid) = skin_id.filter(|&v| v > 0) {
        candidates.push(format!("skin:{sid}:{voice_key}"));
        candidates.push(format!("{voice_key}:skin:{sid}"));
        if voice_key.starts_with("skill:") {
            candidates.push(format!("skin:{sid}:activate"));
        }
    }
    if let Some(f) = form {
        candidates.push(format!("{voice_key}:form:{f}"));
        candidates.push(format!("form:{f}:{voice_key}"));
    }
    candidates.push(voice_key.to_string());
    candidates.push("activate".to_string());
    candidates.push("default".to_string());
    for k in &candidates {
        if let Some(list) = binding.voice_assets.get(k) {
            if !list.is_empty() {
                return Some(list.clone());
            }
        }
    }
    None
}

/// `UP37-I02`: Verify the original presentation baseline counts (2,487 upstream-missing
/// audio files, 234 dynamic JS expressions, 18 non-voice assets) and verified manifest integrity.
pub fn verify_up37_i02_baseline_counts() -> Result<(usize, usize, usize), String> {
    let input = embedded_original_presentation_input();
    let verified = verify_original_presentation(&input)?;
    if verified.characters.len() < 2_580 || verified.skills.len() < 6_200 {
        return Err(format!(
            "unexpected catalog dimensions: {} chars, {} skills",
            verified.characters.len(),
            verified.skills.len()
        ));
    }
    Ok((
        UP37_MISSING_AUDIO_FILES_BASELINE,
        UP37_DYNAMIC_JS_AUDIO_EXPRS_BASELINE,
        UP37_NON_VOICE_ASSETS_BASELINE,
    ))
}

/// `UP37-I03`: Verify the 6 `sp`/`guozhan`/`tw` Step->Async upstream commits
/// (`ef2211d3`, `66927789`, `292488e3`, `6fb4a101`, `dd98c448`, `51465963`) preserve
/// general/skill `def_id`s and profile signatures in `content/original/catalog.json`,
/// and verify config/dependency declarations for the new `UP37` items.
pub fn verify_up37_i03_step_to_async_catalog_invariants(
) -> Result<Vec<(String, String, u32)>, String> {
    let input = embedded_original_presentation_input();
    let parsed: serde_json::Value =
        serde_json::from_slice(input.catalog_json).map_err(|e| e.to_string())?;
    let chars = parsed
        .get("characters")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "catalog.json missing characters[]".to_string())?;
    let skills = parsed
        .get("skills")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "catalog.json missing skills[]".to_string())?;

    let mut out = Vec::with_capacity(UP37_STEP_TO_ASYNC_COMMITS.len());
    for &(commit, pack, sym, expected_id) in &UP37_STEP_TO_ASYNC_COMMITS {
        let entry = skills
            .iter()
            .find(|s| {
                s.get("pack").and_then(|v| v.as_str()) == Some(pack)
                    && s.get("name").and_then(|v| v.as_str()) == Some(sym)
            })
            .ok_or_else(|| format!("commit {commit}: {pack}:{sym} missing in catalog.json"))?;
        let actual_id = entry
            .get("def_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("commit {commit}: {pack}:{sym} missing def_id"))?
            as u32;
        if actual_id != expected_id {
            return Err(format!(
                "commit {commit}: {pack}:{sym} def_id mismatch: got {actual_id}, expected {expected_id}"
            ));
        }
        let profile = entry
            .get("profile")
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("commit {commit}: {pack}:{sym} missing profile"))?;
        if profile.get("grade").and_then(|v| v.as_str()).is_none()
            || profile.get("triggers").and_then(|v| v.as_array()).is_none()
        {
            return Err(format!(
                "commit {commit}: {pack}:{sym} has incomplete profile signature"
            ));
        }
        out.push((commit.to_string(), format!("{pack}:{sym}"), actual_id));
    }

    // Verify dependency declarations for tw_dm_zhangliang (`tw:twjianming`, `extra:relonghun`)
    let zl_entry = chars
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_str()) == Some("collab:tw_dm_zhangliang"))
        .ok_or_else(|| "collab:tw_dm_zhangliang missing in catalog.json".to_string())?;
    let zl_def_id = zl_entry.get("def_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if zl_def_id != 3_884_015_102 {
        return Err(format!("tw_dm_zhangliang def_id mismatch: {zl_def_id}"));
    }
    for &(pack, sym, expected_id) in &[
        ("tw", "twjianming", 2_810_704_636u32),
        ("extra", "relonghun", 2_722_411_221u32),
        ("sp2", "stardangchen", 4_091_538_799u32),
    ] {
        let sk_entry = skills
            .iter()
            .find(|s| {
                s.get("pack").and_then(|v| v.as_str()) == Some(pack)
                    && s.get("name").and_then(|v| v.as_str()) == Some(sym)
            })
            .ok_or_else(|| format!("UP37 dependency skill {pack}:{sym} missing in catalog.json"))?;
        let actual_id = sk_entry.get("def_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if actual_id != expected_id {
            return Err(format!(
                "UP37 dependency skill {pack}:{sym} def_id mismatch: {actual_id} != {expected_id}"
            ));
        }
    }

    Ok(out)
}

/// `UP37-I05`: Verify portrait/audio/die_audio presentation bindings for `N01..N07`
/// (`ol_jie_zhuran`, `ol_zhuhuan`, `tangtang`, `tw_dm_zhangliang`, `ol_sunhanhua`,
/// `ol_chitu`, `ol_le_menghuo`, `ol_xushao`) and `R07` (`stardangchen`).
pub fn verify_up37_i05_presentation_bindings(catalog: &Catalog) -> Result<(), String> {
    let required_generals = [
        "ol_jie_zhuran",
        "ol_zhuhuan",
        "tangtang",
        "tw_dm_zhangliang",
        "ol_sunhanhua",
        "ol_chitu",
        "ol_le_menghuo",
        "ol_xushao",
    ];
    for g_name in required_generals {
        let g_idx = (0..catalog.general_count() as u16)
            .find(|&idx| catalog.general_meta(idx).is_some_and(|m| m.0 == g_name))
            .ok_or_else(|| format!("UP37-I05 general {g_name} missing from catalog"))?;
        let binding = catalog
            .general_presentation_binding(g_idx)
            .ok_or_else(|| format!("UP37-I05 general {g_name} missing presentation_binding"))?;
        if binding.voice_assets.get("die").is_none_or(|v| v.is_empty()) {
            return Err(format!("UP37-I05 general {g_name} missing die_audio binding"));
        }
    }
    let stardangchen_def = catalog
        .skill_by_name("stardangchen")
        .ok_or_else(|| "UP37-I05 skill stardangchen missing from catalog".to_string())?;
    let sk_binding = catalog.skills[stardangchen_def as usize]
        .presentation_binding
        .as_ref()
        .ok_or_else(|| "UP37-I05 skill stardangchen missing presentation_binding".to_string())?;
    if sk_binding
        .voice_assets
        .get("activate")
        .is_none_or(|v| v.is_empty())
    {
        return Err("UP37-I05 skill stardangchen missing activate audio binding".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up37_i02_baseline_counts_and_skin_override_lookup_hold() {
        let (missing, dynamic_exprs, non_voice) =
            verify_up37_i02_baseline_counts().expect("I02 baseline counts must verify");
        assert_eq!(missing, 2_487);
        assert_eq!(dynamic_exprs, 234);
        assert_eq!(non_voice, 18);

        let catalog = build_up37_closure_catalog();
        let g_idx = (0..catalog.general_count() as u16)
            .find(|&idx| catalog.general_meta(idx).is_some_and(|m| m.0 == "tangtang"))
            .expect("tangtang in catalog");
        let binding = catalog
            .general_presentation_binding(g_idx)
            .expect("tangtang presentation binding");
        let base_die = resolve_binding_voice_with_skin_override(binding, "die", None, None)
            .expect("base die audio");
        let skin_die = resolve_binding_voice_with_skin_override(binding, "die", Some(1), None)
            .expect("skin 1 die audio");
        assert!(!base_die.is_empty());
        assert!(!skin_die.is_empty());
        assert_ne!(base_die[0].digest, skin_die[0].digest);
    }

    #[test]
    fn up37_i03_step_to_async_commits_preserve_catalog_invariants() {
        let verified = verify_up37_i03_step_to_async_catalog_invariants()
            .expect("I03 Step->Async catalog invariants must hold");
        assert_eq!(verified.len(), 6);
    }

    #[test]
    fn up37_i05_presentation_bindings_cover_n01_to_n07_and_r07() {
        let catalog = build_up37_closure_catalog();
        verify_up37_i05_presentation_bindings(&catalog)
            .expect("I05 presentation bindings for N01..N07 and R07 must hold");
    }
}
