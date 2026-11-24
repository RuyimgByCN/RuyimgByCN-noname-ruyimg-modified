//! Runtime bridges connecting Q4 parsed configs/assets (`U02`, `L03`, `U03`,
//! `L04`, `L15`, `L21`, `U04`, `L17`, `L26`) directly to live `Engine` / `ExtensionPack`.

use std::collections::{BTreeMap, BTreeSet};

use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::pile::{extension_pack_from_scale, scale_plan, PileEntry, PileSpec, ScaleStatus};
use engine_native::content::standard::Gender;
use engine_native::rng::RuleRng;
use engine_native::{Engine, FoolLiegongStyle, MartialConfigSpec, MartialHpRule, U04CoinMode};
use serde_json::Value;

use crate::{
    apply_shousha, layout_numbers, parse_boss_play_config, parse_coin_config,
    parse_douzhuan_config, parse_fool_config, parse_martial_config, parse_pile_factors,
    parse_shousha_config, push_host_skill_if_unseen, rank_score, BridgeError, CharacterIn,
    CoinError, CoinGate, DouzhuanError, FoolError, InitOutcome, PlayError, PlayStorage,
    ShoushaError, ShoushaReport, ShoushaRequest, U02_BOSS_GENERALS,
};

/// Execute `boss_play_init` + `boss_play_arenaReady` gating directly onto a live `Engine`:
/// enables `isBossAllowed` pool filtering and sets `forbidai` / `isBossAllowed` flags per mode.
pub fn boss_play_init_and_arena_ready(
    config_json: &str,
    engine: &mut Engine,
    mode: &str,
    versus_mode: &str,
    status_mode: &str,
    storage: &PlayStorage,
    enableai: bool,
) -> Result<InitOutcome, PlayError> {
    let cfg = parse_boss_play_config(config_json)?;
    let out = crate::init_outcome(&cfg, mode, versus_mode, status_mode, storage, enableai);
    if mode != "tafang" {
        boss_play_arena_ready(engine, &cfg.source, mode, enableai);
    } else {
        engine.set_boss_pool_filtering(false);
    }
    Ok(out)
}

/// Apply `boss_play_arenaReady` pool filtering (`isBossAllowed` in boss/versus modes,
/// `forbidai` controlled by `enableai` across eligible modes) directly onto `Engine`.
pub fn boss_play_arena_ready(engine: &mut Engine, source: &str, mode: &str, enableai: bool) {
    if mode == "tafang" {
        engine.set_boss_pool_filtering(false);
        return;
    }
    let boss_mode = matches!(mode, "boss" | "versus");
    engine.set_boss_pool_filtering(boss_mode);
    let liuli = source == "noname-liuli" || source == "liuli";
    for (idx, &(name, _, _, _, _)) in U02_BOSS_GENERALS.iter().enumerate() {
        let in_source = !liuli || idx < 8;
        if in_source {
            engine.set_general_unseen_named(name, false);
            engine.set_boss_general_allowed(name, boss_mode);
            engine.set_general_forbidai_named(name, !enableai);
        } else {
            engine.set_general_unseen_named(name, true);
            engine.set_boss_general_allowed(name, false);
            engine.set_general_forbidai_named(name, true);
        }
    }
}

/// Parse `pile.json` into a typed `PileSpec` plus flattened `(card, suit, count)` buckets.
pub fn verify_pile_json_buckets(
    pile_json: &str,
) -> Result<(PileSpec, Vec<(String, String, u32)>), BridgeError> {
    let val: Value = serde_json::from_str(pile_json).map_err(|e| BridgeError::BadJson(e.to_string()))?;
    let obj = val
        .as_object()
        .ok_or(BridgeError::MissingField("pile_object"))?;
    let source = obj
        .get("source")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(BridgeError::MissingField("source"))?
        .to_string();
    let total = obj
        .get("total")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or(BridgeError::MissingField("total"))?;
    let raw_entries = obj
        .get("entries")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingField("entries"))?;
    let mut entries = Vec::with_capacity(raw_entries.len());
    let mut buckets = Vec::new();
    for item in raw_entries {
        let row = item
            .as_object()
            .ok_or(BridgeError::MissingField("entries[].object"))?;
        let card = row
            .get("card")
            .and_then(Value::as_str)
            .ok_or(BridgeError::MissingField("entries[].card"))?
            .to_string();
        let counts_obj = row
            .get("counts")
            .and_then(Value::as_object)
            .ok_or(BridgeError::MissingField("entries[].counts"))?;
        let mut counts = Vec::with_capacity(counts_obj.len());
        for (suit, v) in counts_obj {
            let count = v
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| BridgeError::InvalidField {
                    field: "entries[].counts",
                    reason: format!("bad count for {card}.{suit}"),
                })?;
            buckets.push((card.clone(), suit.clone(), count));
            counts.push((suit.clone(), count));
        }
        entries.push(PileEntry { card, counts });
    }
    Ok((PileSpec { source, total, entries }, buckets))
}

/// Execute `cardpile_arenaReady`: run `scale_plan` against `base_deck_len`, generate
/// deterministic RNG `1..=13` ranks using `RuleRng::from_seed(rng_seed)`, and build a ready
/// `ExtensionPack` for `Engine::new_with_extensions`. Returns `Ok(None)` when unconfigured or no additions.
pub fn cardpile_arena_ready_pack(
    pile_json: &str,
    factors_json: &str,
    base_deck_len: u32,
    rng_seed: u64,
    def: ExtensionDef,
    name_prefix: &str,
) -> Result<Option<ExtensionPack>, BridgeError> {
    let (spec, _) = verify_pile_json_buckets(pile_json)?;
    let factors = parse_pile_factors(factors_json).map_err(|_| BridgeError::InvalidField {
        field: "factors",
        reason: "bad pile factors JSON".to_string(),
    })?;
    let status = scale_plan(&spec, &factors, base_deck_len).map_err(|e| BridgeError::InvalidField {
        field: "scale_plan",
        reason: format!("{e:?}"),
    })?;
    match status {
        ScaleStatus::Unconfigured | ScaleStatus::NoAddition => Ok(None),
        ScaleStatus::Added(buckets) => {
            let total: usize = buckets.iter().map(|b| b.count as usize).sum();
            let mut rng = RuleRng::from_seed(rng_seed);
            let ranks: Vec<u8> = (0..total).map(|_| (rng.gen_range(13) + 1) as u8).collect();
            let pack = extension_pack_from_scale(def, name_prefix, &buckets, &ranks).map_err(|e| {
                BridgeError::InvalidField {
                    field: "extension_pack_from_scale",
                    reason: format!("{e:?}"),
                }
            })?;
            Ok(Some(pack))
        }
    }
}

/// Apply `L15 手杀全扩` (`shousha.rs`) pools and ban configs onto live `Engine` `Catalog` generals.
pub fn apply_shousha_to_engine(
    config_json: &str,
    mode: &str,
    ban_bt: bool,
    packs: &[String],
    pack_members: &BTreeMap<String, Vec<String>>,
    character_pack_map: &BTreeMap<String, String>,
    engine: &mut Engine,
) -> Result<ShoushaReport, ShoushaError> {
    let config = parse_shousha_config(config_json)?;
    let cat = engine.catalog().clone();
    let mut characters = Vec::with_capacity(cat.general_count());
    for idx in 0..cat.general_count() as u16 {
        let Some((id, _, _, _, gender)) = cat.general_meta(idx) else {
            continue;
        };
        let g_str = match gender {
            Gender::Male => "male",
            Gender::Female => "female",
        };
        let pack = character_pack_map
            .get(id)
            .cloned()
            .unwrap_or_else(|| "standard".to_string());
        characters.push(CharacterIn {
            id: id.to_string(),
            gender: g_str.to_string(),
            pack,
            name: id.to_string(),
        });
    }
    let request = ShoushaRequest {
        mode: mode.to_string(),
        ban_bt,
        characters: characters.clone(),
        packs: packs.to_vec(),
        pack_members: pack_members.clone(),
        skills: Vec::new(),
    };
    let report = apply_shousha(&config, &request)?;
    for (ch, input) in report.characters.iter().zip(characters.iter()) {
        let unseen = ch.tags.iter().any(|t| t == "unseen");
        let forbidai = ch.tags.iter().any(|t| t == "forbidai") || report.forbidai.contains(&ch.id);
        let ban_txhj = config.txhj && mode != "taixuhuanjing" && ch.id.contains("txhj_");
        let pack_filtered =
            (config.jinjiang && config.packs) || config.huanyuan || config.yinjian;
        let ban_packs = pack_filtered && !packs.is_empty() && !report.packs.contains(&input.pack);
        engine.set_general_unseen_named(&ch.id, unseen);
        engine.set_general_forbidai_named(&ch.id, forbidai);
        engine.set_general_ban_category(&ch.id, "sex", false);
        engine.set_general_ban_category(&ch.id, "txhj", ban_txhj);
        engine.set_general_ban_category(&ch.id, "packs", ban_packs);
        let mut bits = 0u32;
        if config.huanyuan && config.contains_huanyuan(&ch.id) {
            bits |= 1;
        }
        if config.yinjian && config.contains_yinjian(&ch.id) {
            bits |= 2;
        }
        if config.contains_doudizhu(&ch.id) {
            bits |= 4;
        }
        engine.set_l15_pool_bits(&ch.id, bits);
    }
    Ok(report)
}

/// Construct a real `L21 武皇争霸` 2-player KOF `Engine` directly from `config.json`.
pub fn martial_engine_from_config(
    seed: u64,
    config_json: &str,
    pools: [Vec<u16>; 2],
    auto_join: bool,
    packs: &[&ExtensionPack],
) -> Result<Engine, String> {
    let cfg = parse_martial_config(config_json).map_err(|e| format!("{e:?}"))?;
    let layout = layout_numbers(cfg.show, cfg.battle, cfg.hide);
    let hp_rule = MartialHpRule::from_config_str(&cfg.double_hp).map_err(str::to_string)?;
    let spec = MartialConfigSpec {
        show_number: layout.show as u8,
        battle_number: layout.battle as u8,
        hide_number: layout.hidden as u8,
        double_character: cfg.double_character,
        double_hp: hp_rule,
        free_choose: cfg.free_choose,
        change_choice: cfg.change_choice,
        auto_join,
    };
    let mut engine = Engine::new_with_extensions_mode(
        seed,
        2,
        packs,
        engine_native::state::GameMode::Kof {
            pools,
            wins: [0, 0],
            games: 0,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    engine.configure_martial_match(&spec)?;
    Ok(engine)
}

/// Apply `U04` `coin` configuration and gates directly onto a live `Engine`.
pub fn apply_coin_to_engine(
    config_json: &str,
    gate: CoinGate,
    engine: &mut Engine,
) -> Result<(), CoinError> {
    let cfg = parse_coin_config(config_json)?;
    let mode = match gate.mode {
        crate::CoinMode::Identity => U04CoinMode::Identity,
        crate::CoinMode::Guozhan => U04CoinMode::Guozhan,
        crate::CoinMode::Versus => U04CoinMode::Versus,
        crate::CoinMode::Chess => U04CoinMode::Chess,
        crate::CoinMode::Other => U04CoinMode::Other,
    };
    engine.u04_coin_init(cfg.balance, mode, gate.chess_leader, gate.video, gate.connect);
    Ok(())
}

fn apply_douzhuan_report_and_ranks(
    cfg: &crate::DouzhuanConfig,
    report: &crate::DouzhuanReport,
    online: bool,
    rarities: &BTreeMap<String, String>,
    engine: &mut Engine,
) {
    engine.set_doudizhu_ai_config(cfg.ai_qianghua, online);
    let mut all_ids = BTreeSet::new();
    all_ids.extend(cfg.stars.keys().cloned());
    all_ids.extend(rarities.keys().cloned());
    let cat = engine.catalog().clone();
    for idx in 0..cat.general_count() as u16 {
        if let Some((name, _, _, _, _)) = cat.general_meta(idx) {
            if cfg.stars.contains_key(name) || rarities.contains_key(name) {
                all_ids.insert(name.to_string());
            }
        }
    }
    for id in all_ids {
        let saved = cfg.stars.get(&id).map(Vec::as_slice);
        let rarity = rarities.get(&id).map(String::as_str);
        let lord = rank_score(saved, "地主", rarity).unwrap_or(4) as i32;
        let farmer = rank_score(saved, "农民", rarity).unwrap_or(4) as i32;
        engine.set_doudizhu_ai_ranks(&id, lord, farmer);
    }
    engine.set_douzhuan_miaoshu_enabled(cfg.miaoshu);
    engine.set_douzhuan_reload_guard(cfg.forbid_reload);
    engine.set_douzhuan_mode(&report.mode);
    engine.clear_douzhuan_l17_bans();
    let forbid_unseen = report.forbid_type == "unseen";
    engine.set_douzhuan_jinjiang_state(report.gate, forbid_unseen, report.plan_index);
    if report.gate {
        for id in &report.banned {
            engine.set_douzhuan_ban_named(id, forbid_unseen);
        }
    }
}

/// Apply all `L17` (`JinJiang`/`JinJiangType`/`JinJiangRuZhen`, `AIqianghuaXuanjiang`,
/// `miaoshuXiuGai`, `forbidReload`) rules directly onto a live `Engine`.
pub fn apply_douzhuan_to_engine(
    config_json: &str,
    online: bool,
    rarities: &BTreeMap<String, String>,
    engine: &mut Engine,
) -> Result<(), DouzhuanError> {
    let cfg = parse_douzhuan_config(config_json)?;
    let cat = engine.catalog().clone();
    let characters = (0..cat.general_count() as u16)
        .filter_map(|idx| {
            cat.general_meta(idx).map(|(name, _, _, _, _)| crate::DouzhuanCharacterIn {
                id: name.to_string(),
                tags: None,
            })
        })
        .collect();
    let request = crate::DouzhuanRequest {
        mode: engine.default_douzhuan_mode_key().to_string(),
        brawl: false,
        story: None,
        network: Some(if online { "online" } else { "offline" }.to_string()),
        previous_tempmode: None,
        characters,
        skills: Vec::new(),
        dynamic: Vec::new(),
        plans: None,
        mode_plans: None,
        import_text: None,
        import_index: None,
        rename: None,
        rename_index: None,
        edit_index: None,
        ban_ids: Vec::new(),
        open_ids: Vec::new(),
        keep_ids: Vec::new(),
        pack_base: None,
        pack_views: Vec::new(),
        pick: None,
    };
    let report = crate::apply_douzhuan(&cfg, &request)?;
    apply_douzhuan_report_and_ranks(&cfg, &report, online, rarities, engine);
    Ok(())
}

/// Apply `L17` interactive `banCharF` / mode-switch request rules directly onto a live `Engine`.
pub fn apply_douzhuan_request_to_engine(
    config_json: &str,
    request_json: &str,
    rarities: &BTreeMap<String, String>,
    engine: &mut Engine,
) -> Result<crate::DouzhuanReport, DouzhuanError> {
    let cfg = parse_douzhuan_config(config_json)?;
    let request = crate::request_from_json(request_json)?;
    let online = request
        .network
        .as_deref()
        .filter(|s| !s.is_empty())
        != Some("offline");
    let report = crate::apply_douzhuan(&cfg, &request)?;
    apply_douzhuan_report_and_ranks(&cfg, &report, online, rarities, engine);
    Ok(report)
}

/// Build an authoritative `ExtensionPack` exposing `L26 Fool_skill` (`ziyuan`/`minsi`/`xinfu_fangtong`)
/// and attach helper `HostSkillDecl`s cleanly without colliding with `prior_packs`.
pub fn fool_extension_pack(prior_packs: &[&ExtensionPack]) -> ExtensionPack {
    let mut seen_skills = BTreeSet::new();
    for p in prior_packs {
        for s in &p.host_skills {
            seen_skills.insert(s.name.to_string());
        }
        for s in &p.vm_skills {
            seen_skills.insert(s.name.clone());
        }
    }
    let mut host_skills = Vec::new();
    append_fool_host_skills(&mut seen_skills, &mut host_skills);
    ExtensionPack {
        def: ExtensionDef {
            name: "fool-plugin-upstream-liuli",
            version: 1,
            description: "Upstream L26 fool plugin point-sum and liegong hint extension",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    }
}

pub(crate) fn append_fool_host_skills(
    seen_skills: &mut BTreeSet<String>,
    out_host_skills: &mut Vec<HostSkillDecl>,
) {
    for sk in ["ziyuan", "minsi", "xinfu_fangtong"] {
        push_host_skill_if_unseen(sk, HostBehavior::Q4FoolSkill, out_host_skills, seen_skills);
    }
}

pub(crate) fn append_q4_formal_rules(
    namespace: &str,
    package_id: &str,
    seen_generals: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
    extra_generals: &mut Vec<ExtensionGeneral>,
    extra_host_skills: &mut Vec<HostSkillDecl>,
) {
    match (namespace, package_id) {
        ("upstream-noname", "boss-play") => {
            crate::boss_play_data::append_boss_play_content(
                "noname",
                seen_generals,
                seen_skills,
                extra_generals,
                extra_host_skills,
            );
        }
        ("upstream-liuli", "boss-play") => {
            crate::boss_play_data::append_boss_play_content(
                "noname-liuli",
                seen_generals,
                seen_skills,
                extra_generals,
                extra_host_skills,
            );
        }
        ("upstream-liuli", "fool-plugin") => {
            append_fool_host_skills(seen_skills, extra_host_skills);
        }
        _ => {}
    }
}

/// Apply `L26` `Fool_liegong` and `Fool_skill` configuration onto a live `Engine`.
pub fn apply_fool_to_engine(config_json: &str, engine: &mut Engine) -> Result<(), FoolError> {
    let cfg = parse_fool_config(config_json)?;
    let style = FoolLiegongStyle::from_config_str(&cfg.liegong)
        .ok_or_else(|| FoolError::BadConfig(format!("liegong {}", cfg.liegong)))?;
    engine.set_fool_liegong_style(style);
    engine.set_fool_skill_enabled(cfg.skill_calc);
    Ok(())
}
