//! Golden coverage for sp batch-A DSL rewrites:
//! 任城 (pxrencheng) hand_limit + sha-count, 阴平 (pxyinping) distance_to +1,
//! 螭纹玉佩 (chiwenyupei) recover/draw branches, 太沉 (taichen) active virtual Sha.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::query;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const PXRENCHENG: u32 = 2_654_089_885;
const PXYINPING: u32 = 2_186_620_771;
const CHIWENYUPEI: u32 = 2_463_243_907;
const TAICHEN: u32 = 3_277_821_116;
const XINYONGSI1: u32 = 3_518_300_522;
const XINFU_QINGUO: u32 = 2_493_371_671;
const YINGSHI_DIE: u32 = 4_161_454_313;
const OLZHENNAN: u32 = 3_516_136_840;
const XINYONGSI2: u32 = 3_077_331_699;
const BINTIESHUANGJI: u32 = 4_139_600_320;
const OLTIELUN: u32 = 2_476_158_656;
const NEWWUNIANG: u32 = 3_700_934_480;
const OL_PILICHE: u32 = 2_743_752_239;
const SHUFAZIJINGUAN: u32 = 3_390_731_241;
const XINYONGSI: u32 = 2_559_297_491;
const OLQIWU: u32 = 2_221_901_766;
const REFENXUN: u32 = 3_762_143_918;
const REFENXUN2: u32 = 3_372_440_036;
const OLHONGYUAN: u32 = 2_470_217_214;
const REFUHAN: u32 = 3_778_064_756;
const FANGHUN: u32 = 3_479_603_365;
const RELEIJIE: u32 = 3_971_138_184;
const WUSHUANGFANGTIANJI: u32 = 3_620_526_961;
const RELUANZHAN_ADD: u32 = 4_002_502_989;
const SUISHI2: u32 = 3_233_154_442;
const OLWUNIANG: u32 = 3_669_301_225;
const XINZHENGNAN: u32 = 2_799_539_959;
const RELUANZHAN_REMOVE: u32 = 2_198_542_863;
const SPMENGJIN: u32 = 4_088_994_109;
const OLXUSHEN: u32 = 4_033_253_376;
const XIEMU2: u32 = 3_238_781_068;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn grant(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn inject_sha_hand(tx: &mut Transaction, player: PlayerId, count: usize) {
    let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    for _ in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: engine_api::CardDefId::new(1),
            kind: CardKind::Sha,
            suit: engine_api::Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone,
            owner: player,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    }
}

#[test]
fn pxrencheng_adds_sha_hand_count_to_hand_limit() {
    let mut engine = started(2_026_081_101);
    let state = engine.state().clone();
    let mut tx = Transaction::begin(state);
    let owner = tx.state().turn.current;
    grant(&mut tx, owner, PXRENCHENG);
    inject_sha_hand(&mut tx, owner, 3);
    let state = tx.commit().state;
    let sha_in_hand = state
        .hand_cards(owner)
        .into_iter()
        .filter(|card| state.cards.get(*card).is_some_and(|card| card.kind.is_sha()))
        .count();
    assert!(
        sha_in_hand >= 3,
        "injected 3 Sha plus natural deal must hold at least 3 Sha, got {sha_in_hand}"
    );
    assert_eq!(
        query(
            &state,
            &QueryRequest::HandLimit { player: owner },
            &mut engine_core::QueryContext::new(),
        ),
        QueryResponse::Int(state.players.get(owner).unwrap().hp + sha_in_hand as i32)
    );
}

#[test]
fn pxyinping_increases_distance_to_owner() {
    let mut engine = started(2_026_081_102);
    let state = engine.state().clone();
    let mut tx = Transaction::begin(state);
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    grant(&mut tx, owner, PXYINPING);
    let state = tx.commit().state;
    let dist = query(
        &state,
        &QueryRequest::Distance {
            from: other,
            to: owner,
        },
        &mut engine_core::QueryContext::new(),
    );
    assert!(
        matches!(dist, QueryResponse::Int(d) if d >= 2),
        "distance other->owner should be >= 2 with 阴平, got {dist:?}"
    );
}

#[test]
fn chiwenyupei_fires_forced_triggers_without_prompt() {
    let mut engine = started(2_026_081_103);
    let state = engine.state().clone();
    let mut tx = Transaction::begin(state);
    let owner = tx.state().turn.current;
    grant(&mut tx, owner, CHIWENYUPEI);
    let state = tx.commit().state;
    // Both triggers must be registered as forced (non-optional).
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigs: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == CHIWENYUPEI)
        .collect();
    assert_eq!(trigs.len(), 2);
    assert!(trigs.iter().all(|t| !t.optional));
    assert!(trigs
        .iter()
        .any(|t| t.filter == "owner_is_target_healthy_draw_not_fixed"));
}

#[test]
fn taichen_registers_play_phase_active_once_per_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == TAICHEN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let _ = TAICHEN;
}

#[test]
fn xinyongsi1_overrides_draw_count_with_alive_group_count() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let modifier = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == XINYONGSI1)
        .expect("xinyongsi1 draw override must be registered");
    assert_eq!(modifier.target, "draw_count");
    assert_eq!(modifier.mode, "override");
    assert_eq!(modifier.value_source.as_deref(), Some("alive_group_count"));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == XINYONGSI1 && s.locked));
}

#[test]
fn xinfu_qinguo_uses_equip_trigger_with_optional_virtual_sha() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == XINFU_QINGUO)
        .expect("xinfu_qinguo equip trigger must be registered");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_alive_equip_used");
    assert!(trigger.optional);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == XINFU_QINGUO && !s.locked));
}

#[test]
fn yingshi_die_registers_locked_death_trigger_with_pile_gain() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == YINGSHI_DIE)
        .expect("yingshi_die death trigger must be registered");
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == YINGSHI_DIE && s.locked));
    assert!(package.manifest.rule_vm_abi_version >= 600);
}

#[test]
fn olzhennan_registers_nanman_view_as_once_per_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == OLZHENNAN)
        .expect("olzhennan nanman view_as must be registered");
    assert_eq!(view_as.as_kind, "nanman");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.uses_per_turn, Some(1));
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
}

#[test]
fn xinyongsi2_registers_locked_discard_phase_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == XINYONGSI2)
        .expect("xinyongsi2 discard trigger must be registered");
    assert_eq!(trigger.event, "phase_discard");
    assert!(!trigger.optional);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == XINYONGSI2 && s.locked));
    assert!(package.manifest.rule_vm_abi_version >= 601);
}

#[test]
fn bintieshuangji_registers_sha_limit_modifier_and_dodge_triggers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == BINTIESHUANGJI
            && m.target == "sha_use_limit"
            && m.mode == "add"
            && m.value_source.as_deref() == Some("owner_skill_mark")
    }));
    let dodged = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == BINTIESHUANGJI && t.event == "sha_dodged")
        .expect("sha_dodged trigger must be registered");
    assert!(dodged.optional);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == BINTIESHUANGJI && t.event == "turn_ended" && !t.optional));
}

#[test]
fn oltielun_registers_distance_mark_and_round_reset() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == OLTIELUN
            && m.target == "distance_to"
            && m.value_source.as_deref() == Some("owner_skill_mark")
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == OLTIELUN && t.event == "card_used"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == OLTIELUN && t.event == "round_started"
    }));
}

#[test]
fn newwuniang_registers_non_basic_sha_view_as_both_windows() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let va: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|v| v.skill_def_id == NEWWUNIANG)
        .collect();
    assert_eq!(va.len(), 2);
    assert!(va.iter().all(|v| v.as_kind == "sha" && v.filter == "non_basic"));
    assert!(va.iter().any(|v| v.window == "play_phase"));
    assert!(va.iter().any(|v| v.window == "response"));
}

#[test]
fn ol_piliche_registers_damage_source_optional_discard_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OL_PILICHE)
        .expect("ol_piliche damage trigger must be registered");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(trigger.optional);
}

#[test]
fn batch_g_skills_register_with_expected_surfaces() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let shufa = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SHUFAZIJINGUAN)
        .expect("shufazijinguan trigger");
    assert_eq!(shufa.event, "phase_prepare");
    assert!(shufa.optional);
    let yongsi = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == XINYONGSI)
        .expect("xinyongsi parent trigger");
    assert!(!yongsi.optional);
    assert!(package
        .view_as_index
        .iter()
        .any(|v| v.skill_def_id == OLQIWU && v.as_kind == "sha" && v.uses_per_turn == Some(1)));
}

#[test]
fn batch_h_skills_register_with_expected_surfaces() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == REFENXUN && a.window == "play_phase" && a.uses_per_turn == Some(1)));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == REFENXUN2 && m.target == "distance_from_skill_bound_target"
    }));
    let hongyuan = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OLHONGYUAN)
        .expect("olhongyuan trigger");
    assert_eq!(hongyuan.event, "cards_moved");
    assert_eq!(hongyuan.filter, "owner_gained_or_lost_at_least_two");
    assert!(hongyuan.optional);
}

#[test]
fn refuhan_registers_mark_conditional_prepare_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == REFUHAN)
        .expect("refuhan trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(
        trigger.filter,
        format!("owner_is_target_mark_positive_{FANGHUN}")
    );
    assert!(trigger.optional);
}

#[test]
fn releijie_registers_play_phase_active_once_per_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == RELEIJIE)
        .expect("releijie active");
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(package.manifest.rule_vm_abi_version >= 602);
}

#[test]
fn wushuangfangtianji_registers_sha_damage_optional_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == WUSHUANGFANGTIANJI)
        .expect("wushuangfangtianji trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
}

#[test]
fn reluanzhan_add_registers_mark_gated_sha_black_trick_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == RELUANZHAN_ADD)
        .expect("reluanzhan_add trigger");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_black_trick_mark_positive_4251489811"
    );
    assert!(trigger.optional);
}

#[test]
fn suishi2_registers_locked_death_source_confirm_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SUISHI2)
        .expect("suishi2 trigger");
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    assert!(package.manifest.rule_vm_abi_version >= 603);
}

#[test]
fn olwuniang_registers_sha_use_optional_counterattack_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OLWUNIANG)
        .expect("olwuniang trigger");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
}

#[test]
fn xinzhengnan_registers_optional_dying_recover_and_skill_gain() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == XINZHENGNAN)
        .expect("xinzhengnan trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(trigger.optional);
}

#[test]
fn reluanzhan_remove_registers_locked_mark_removal_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == RELUANZHAN_REMOVE)
        .expect("reluanzhan_remove trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_black_trick_mark_positive_4251489811"
    );
    assert!(!trigger.optional);
}

#[test]
fn spmengjin_registers_sha_target_optional_discard_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SPMENGJIN)
        .expect("spmengjin trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
}

#[test]
fn olxushen_registers_optional_dying_recover_skill_gain() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OLXUSHEN)
        .expect("olxushen trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(trigger.optional);
}

#[test]
fn xiemu2_registers_locked_black_target_draw_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == XIEMU2)
        .expect("xiemu2 trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_alive_targeted_by_other_black_card");
    assert!(!trigger.optional);
}
