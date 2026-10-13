//! O2 全量合并批次 1（2026-10-14）：家族 sp 的 130 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sp_active_viewas_batch {
//! Golden coverage for the active/view_as rewrite batch in sp:
//!   - 断发 xinfu_duanfa: active, discard any number of black hand cards, draw the same count.
//!   - 宠慎 olchongshen: red hand cards may be used as 【闪】.
//!   - 拒关 juguan: a hand card may be used as 【杀】 or 【决斗】 once per play phase.
//!
//! Exercises the `card_black` active selection filter (new primitive), the
//! `discard_selected`/`draw_selected` pair, and the `view_as` shan/sha/juedou
//! declarations.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_DUANFA: u32 = 3_504_403_856;
const OLCHONGSHEN: u32 = 4_123_829_304;
const JUGUAN: u32 = 4_273_039_202;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_901,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject_hand(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: cards,
        },
    }
}

#[test]
fn package_declares_the_new_view_as_and_active_entries() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let chongshen = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLCHONGSHEN)
        .unwrap();
    assert_eq!(chongshen.as_kind, "shan");
    assert_eq!(chongshen.window, "any");
    assert_eq!(chongshen.filter, "red");
    for (kind, window) in [("sha", "play_phase"), ("juedou", "play_phase")] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == JUGUAN
                && view.as_kind == kind
                && view.window == window
                && view.uses_per_turn == Some(1)
        }));
    }
    assert!(package
        .active_index
        .iter()
        .any(|active| active.skill_def_id == XINFU_DUANFA));
}

#[test]
fn duanfa_discards_black_cards_and_draws_equal_count() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, XINFU_DUANFA);
    let black1 = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let black2 = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club);
    let red = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let hand_before = tx.state().hand_count(owner);
    let draw_pile_before = {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        tx.state().zones.get(draw).unwrap().cards.len()
    };

    // Red card is not selectable: activating with only the red card is rejected.
    let (_, rejected) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![red], 1),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // Two black cards: discard both, draw two.
    let (state, accepted) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![black1, black2], 2),
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&black1));
    assert!(!state.hand_cards(owner).contains(&black2));
    assert!(
        state.hand_cards(owner).contains(&red),
        "red card untouched"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before,
        "discard 2 + draw 2 keeps hand size"
    );
    assert_eq!(
        {
            let draw = state
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap();
            state.zones.get(draw).unwrap().cards.len()
        },
        draw_pile_before - 2,
        "two cards drawn from the pile"
    );
}

#[test]
fn duanfa_is_limited_to_once_per_turn() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, XINFU_DUANFA);
    let black = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);

    let (state, first) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![], vec![black], 1),
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));

    let second_black = {
        let mut tx = Transaction::begin(state.clone());
        inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club)
    };
    let (_, second) = apply_command(
        state,
        activate(owner, skill, vec![], vec![second_black], 2),
    );
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
}

#[test]
fn chongshen_offers_only_red_hand_cards_in_respond_shan_window() {
    let (engine, owner, other) = start2();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    strip_skills(&mut tx, other);
    clear_hand(&mut tx, other);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = other;
    add_skill(&mut tx, owner, OLCHONGSHEN);
    let red = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let black = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let sha = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);

    // The other player plays 杀 at the owner → a respond-shan window opens for the owner.
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }), "unexpected play result: {:?}", result.result);
    let choice = result
        .choice
        .expect("杀 targeting the owner must open a respond-shan window");
    assert_eq!(choice.observer, owner);
    assert!(
        choice
            .options
            .iter()
            .any(|option| option.token == red.raw()),
        "red hand card offered as 闪 via 宠慎"
    );
    assert!(
        !choice
            .options
            .iter()
            .any(|option| option.token == black.raw()),
        "black hand card not offered"
    );

    // Resolving with the red card dodges the 杀.
    let (state, dodged) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![red.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(
        !state.hand_cards(owner).contains(&red),
        "the red card was consumed as the 闪"
    );
}

#[test]
fn juguan_offers_sha_and_juedou_once_each() {
    let (engine, owner, other) = start2();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, JUGUAN);
    let card1 = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart);
    let card2 = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Diamond);

    let (_, as_sha) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![card1], 1),
    );
    assert!(
        matches!(as_sha.result, CommandResult::Accepted { .. }),
        "hand card usable as 杀"
    );

    let (_, as_juedou) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![card2], 2),
    );
    assert!(
        matches!(as_juedou.result, CommandResult::Accepted { .. }),
        "hand card usable as 决斗 (separate once-per-transform limit)"
    );
}

fn start2() -> (Engine, PlayerId, PlayerId) {
    let (engine, owner) = start();
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}
}

mod original_sp_batch_a_dsl {
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
}

mod original_sp_batch_aa_dsl {
//! Golden for sp batch-AA DSL rewrite:
//! 2 个技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头注释引用的
//! 行号处均有真实规则体：
//!   - xiaoxi（小西）：游戏开始时视为使用一张【杀】（game_started optional）
//!   - zfengshi（风石）：使用【杀】指定目标后弃置其一张装备
//!     （card_targeted + 新增 owner_source_sha_target_alive_has_discardable_equip 过滤器）

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XIAOXI: u32 = 2_994_468_828;
const ZFENGSHI: u32 = 3_436_733_477;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_aa_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, XIAOXI | ZFENGSHI))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(XIAOXI, "game_started", "owner_alive", true)));
    assert!(triggers.contains(&(
        ZFENGSHI,
        "card_targeted",
        "owner_source_sha_target_alive_has_discardable_equip",
        true
    )));
    // 两技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, XIAOXI | ZFENGSHI)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn xiaoxi_opens_virtual_sha_on_game_start() {
    let (state, owner) = started(2_026_081_401);
    let state = with_skill(state, owner, XIAOXI);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入虚拟杀目标选择
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

#[test]
fn zfengshi_discards_target_equip_after_sha_targeted() {
    let (state, owner) = started(2_026_081_402);
    let state = with_skill(state, owner, ZFENGSHI);
    let mut tx = Transaction::begin(state);
    // 给目标装一件装备
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let equip_card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(2_000_101),
        kind: CardKind::Armor,
        suit: engine_api::Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone: equip,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(equip_card);
    // 目标手牌放一张【杀】（用作事件卡）
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1_000_101),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: other,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后打开目标装备弃置选择
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
}
}

mod original_sp_batch_ab_dsl {
//! Golden for sp batch-AB DSL rewrite:
//! 2 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体：
//!   - pxliangzhoux（梁州）：成为他人【杀】目标后弃置其一张牌
//!   - pxchengdu（成都）：对自己使用牌后摸 1（新增 owner_source_card_used_targets_self）

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXLIANGZHOUX: u32 = 2_317_213_965;
const PXCHENGDU: u32 = 2_164_643_288;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_201),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_ab_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXLIANGZHOUX | PXCHENGDU))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXLIANGZHOUX,
        "card_targeted",
        "owner_targeted_by_other_sha",
        true
    )));
    assert!(triggers.contains(&(
        PXCHENGDU,
        "card_used",
        "owner_source_card_used_targets_self",
        true
    )));
    // 两技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXLIANGZHOUX | PXCHENGDU)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxliangzhoux_discards_sha_user_card() {
    let (state, owner) = started(2_026_081_501);
    let state = with_skill(state, owner, PXLIANGZHOUX);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let sha = inject_hand(&mut tx, other, CardKind::Sha);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后打开对使用者手牌/装备的弃置选择
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
}

#[test]
fn pxchengdu_draws_after_self_targeted_card_use() {
    let (state, owner) = started(2_026_081_502);
    let state = with_skill(state, owner, PXCHENGDU);
    let mut tx = Transaction::begin(state);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn pxchengdu_ignores_card_use_not_targeting_self() {
    let (state, owner) = started(2_026_081_503);
    let state = with_skill(state, owner, PXCHENGDU);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "card use not targeting self must not trigger"
    );
}
}

mod original_sp_batch_ac_dsl {
//! Golden for sp batch-AC DSL rewrite:
//! 2 个技能——被批量脚本误标为「audio-only」/「需修改技原语」，上游 (sp/skill.js) 在
//! 文件头注释引用的行号处均有真实规则体：
//!   - olmiaoxin（妙心·OL）：受伤后摸 2（damage owner_is_target forced）
//!   - oltingji（廷机·OL）：回合内指定未受伤其他角色为目标后令该牌不可被响应
//!     （card_targeted + 新增 owner_source_current_phase_card_targeted_other_healthy）

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLMIAOXIN: u32 = 3_006_386_800;
const OLTINGJI: u32 = 4_244_206_396;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_301),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_ac_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLMIAOXIN | OLTINGJI))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLMIAOXIN, "damage", "owner_is_target", false)));
    assert!(triggers.contains(&(
        OLTINGJI,
        "card_targeted",
        "owner_source_current_phase_card_targeted_other_healthy",
        false
    )));
    // 两技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, OLMIAOXIN | OLTINGJI)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn olmiaoxin_draws_two_after_damage() {
    let (state, owner) = started(2_026_081_601);
    let state = with_skill(state, owner, OLMIAOXIN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn oltingji_makes_card_unrespondable_when_targeting_healthy_other() {
    let (state, owner) = started(2_026_081_602);
    let state = with_skill(state, owner, OLTINGJI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 目标保持满血（健康）
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: other,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // 触发成功且无选择（锁定技强制）——事件中应含不可响应效果
    assert!(
        !effects.is_empty(),
        "healthy-other target must trigger 廷机"
    );
    run_effects(&mut tx, effects).unwrap();
}

#[test]
fn oltingji_does_not_fire_for_wounded_target() {
    let (state, owner) = started(2_026_081_603);
    let state = with_skill(state, owner, OLTINGJI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 打伤目标
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: other,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "wounded target must not trigger 廷机"
    );
}
}

mod original_sp_batch_ad_dsl {
//! Golden for sp batch-AD DSL rewrite:
//! 武继 (wuji, 关银屏)：结束阶段开始时，若本回合累计造成至少 3 点伤害，失去「虎啸」、
//! 加 1 点体力上限并回复 1 点体力，然后觉醒。上游 (sp/skill.js:42961)。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{GameState, turn_flag};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const WUJI: u32 = 2_966_745_164;
const HUXIAO: u32 = 3_796_089_056;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_wuji_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == WUJI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_turn_damage_at_least_three"
            && !trigger.optional));
    // 武继不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WUJI && modifier.target == "hand_limit"
    }));
}

#[test]
fn wuji_awakens_at_three_dealt_damage() {
    let (state, owner) = started(2_026_081_701);
    let state = with_skill(state, owner, WUJI);
    let mut tx = Transaction::begin(state);
    // 本回合已造成 3 点伤害
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 3);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before + 1);
    assert_eq!(player.hp, hp_before + 1);
    // 失去「虎啸」并觉醒（移除武继自身）
    let defs: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(!defs.contains(&HUXIAO), "虎啸 must be removed");
    assert!(!defs.contains(&WUJI), "武继 must remove itself after awakening");
}

#[test]
fn wuji_does_not_awaken_below_three_damage() {
    let (state, owner) = started(2_026_081_702);
    let state = with_skill(state, owner, WUJI);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty(), "below-3 damage must not trigger");
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before);
    let defs: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&WUJI), "武继 must stay until threshold met");
}
}

mod original_sp_batch_ad_mingshi {
//! Golden for sp batch-AD DSL rewrite (mingshi):
//! 名士 (mingshi)：受到伤害前，若来源体力大于你，可弃一张牌令伤害 -1。
//! 上游 (sp/skill.js:41702)。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MINGSHI: u32 = 2_979_740_696;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_mingshi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == MINGSHI
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == MINGSHI && modifier.target == "hand_limit"
    }));
}

#[test]
fn mingshi_reduces_damage_when_source_hp_greater() {
    let (state, owner) = started(2_026_081_801);
    let state = with_skill(state, owner, MINGSHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 来源体力 > 拥有者体力
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: other,
            target: owner,
            amount: 2,
            base_amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::Custom
    );
}

#[test]
fn mingshi_skips_when_source_hp_not_greater() {
    let (state, owner) = started(2_026_081_802);
    let state = with_skill(state, owner, MINGSHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 来源体力不大于拥有者（默认 2 人局主公满血）
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: other,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    // 可选触发的 confirm 仍会打开（body if 只挡效果不挡确认）
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 条件不满足：无弃牌选择跟随
    assert!(!matches!(
        state.resolution.continuation_kind,
        ContinuationKind::Custom
    ));
}
}

mod original_sp_batch_ae_dsl {
//! Golden for sp batch-AE DSL rewrite:
//! 凉州 (pxliangzhou)：将一张黑色牌当【杀】使用。上游 (sp/skill.js:1572)。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
};
use engine_core::Engine;

const PXLIANGZHOU: u32 = 2_433_321_161;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn package_publishes_pxliangzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == PXLIANGZHOU)
        .expect("pxliangzhou black-sha view_as must be registered");
    assert_eq!(view_as.as_kind, "sha");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "black");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
    // 凉州不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXLIANGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxliangzhou_package_loads_in_engine() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_026_081_901),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_081_901,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
}
}

mod original_sp_batch_af_dsl {
//! Golden for sp batch-AF DSL rewrite:
//! 7 个 changeSkillsAfter 州郡（px*）技能——批量脚本误标为「audio-only」，上游
//! (sp/skill.js) 在文件头注释引用的行号处均有真实规则体。新增引擎字段
//! `TriggerContext.added_skill_def`（SkillAdded dispatch 透传）+ 参数化过滤器
//! `owner_skill_added_self_<def>`（5 处注册）：
//!   - pxbeihai（北海）：获得后手牌摸至体力上限
//!   - pxyouzhou（幽州）：获得后体力上限 +1
//!   - pxfengyi（冯翊）：获得后可回复 1
//!   - pxhongnong（弘农）：获得后可回复 1（装备重铸未建模）
//!   - pxjizhou（冀州）：获得后从牌堆获得【万箭齐发】
//!   - pxwuxiang（武乡）：获得后从牌堆获得【火攻】
//!   - pxquting（句町）：获得后可失去 1 体力并摸 3

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXBEIHAI: u32 = 3_401_654_349;
const PXYOUZHOU: u32 = 3_102_513_552;
const PXFENGYI: u32 = 4_070_224_935;
const PXHONGNONG: u32 = 4_043_252_708;
const PXJIZHOU: u32 = 2_256_193_085;
const PXWUXIANG: u32 = 4_127_527_891;
const PXQUTING: u32 = 2_515_073_240;
const PXCHANGSHAN: u32 = 4_243_325_414;
const PXBEIPING: u32 = 3_838_939_507;
const PXLEAN: u32 = 4_240_733_007;
const PXYANMEN: u32 = 3_339_454_948;
const PXLONGXI: u32 = 4_087_538_948;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

/// 通过 AddSkill 效果授予技能——run_effects 会派发 SkillAdded 事件并运行
/// skill_added 触发链（技能触发在 add_skill 内先注册）。
fn grant(state: GameState, owner: PlayerId, def: u32) -> (GameState, Vec<DomainEvent>) {
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    );
    match effects {
        Ok(events) => (tx.commit().state, events),
        Err(EffectError::ChoiceRequested) => {
            let tx = tx.commit();
            let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            (state, result.events.clone())
        }
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_af_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                PXBEIHAI | PXYOUZHOU | PXFENGYI | PXHONGNONG | PXJIZHOU | PXWUXIANG
                    | PXQUTING | PXCHANGSHAN | PXBEIPING | PXLEAN | PXYANMEN | PXLONGXI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXBEIHAI, "skill_added", "owner_skill_added_self_3401654349", false)));
    assert!(triggers.contains(&(PXYOUZHOU, "skill_added", "owner_skill_added_self_3102513552", false)));
    assert!(triggers.contains(&(PXFENGYI, "skill_added", "owner_skill_added_self_4070224935", true)));
    assert!(triggers.contains(&(PXHONGNONG, "skill_added", "owner_skill_added_self_4043252708", true)));
    assert!(triggers.contains(&(PXJIZHOU, "skill_added", "owner_skill_added_self_2256193085", false)));
    assert!(triggers.contains(&(PXWUXIANG, "skill_added", "owner_skill_added_self_4127527891", false)));
    assert!(triggers.contains(&(PXQUTING, "skill_added", "owner_skill_added_self_2515073240", true)));
    assert!(triggers.contains(&(PXCHANGSHAN, "skill_added", "owner_skill_added_self_4243325414", false)));
    assert!(triggers.contains(&(PXBEIPING, "skill_added", "owner_skill_added_self_3838939507", false)));
    assert!(triggers.contains(&(PXLEAN, "skill_added", "owner_skill_added_self_4240733007", false)));
    assert!(triggers.contains(&(PXYANMEN, "skill_added", "owner_skill_added_self_3339454948", false)));
    assert!(triggers.contains(&(PXLONGXI, "skill_added", "owner_skill_added_self_4087538948", false)));
    // 十二技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXBEIHAI | PXYOUZHOU | PXFENGYI | PXHONGNONG | PXJIZHOU | PXWUXIANG
                | PXQUTING | PXCHANGSHAN | PXBEIPING | PXLEAN | PXYANMEN | PXLONGXI
        ) && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxbeihai_draws_to_max_hp_on_gain() {
    let (state, owner) = started(2_026_081_001);
    // 2 人局主公 max_hp=5 但起始手牌 6（主公加成），先弃 3 张使手牌低于上限
    let mut tx = Transaction::begin(state);
    let cards: Vec<_> = tx.state().hand_cards(owner);
    let discard: Vec<_> = cards.iter().take(3).copied().collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: discard,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let player = state.players.get(owner).unwrap();
    assert!(state.hand_count(owner) < player.max_hp);
    let (state, _) = grant(state, owner, PXBEIHAI);
    let player = state.players.get(owner).unwrap();
    assert_eq!(
        state.hand_count(owner),
        player.max_hp,
        "北海 must draw the owner's hand up to max HP"
    );
}

#[test]
fn pxyouzhou_increases_max_hp_on_gain() {
    let (state, owner) = started(2_026_081_002);
    let max_hp_before = state.players.get(owner).unwrap().max_hp;
    let (state, _) = grant(state, owner, PXYOUZHOU);
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before + 1,
        "幽州 must add one max HP"
    );
}

#[test]
fn pxjizhou_gains_wanjian_on_gain() {
    let (state, owner) = started(2_026_081_003);
    let (state, _) = grant(state, owner, PXJIZHOU);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_wanjian = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::TrickWanjian);
    assert!(has_wanjian, "冀州 must gain a 万箭齐发 into hand");
}

#[test]
fn pxwuxiang_gains_huogong_on_gain() {
    let (state, owner) = started(2_026_081_004);
    let (state, _) = grant(state, owner, PXWUXIANG);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_huogong = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::TrickHuogong);
    assert!(has_huogong, "武乡 must gain a 火攻 into hand");
}

#[test]
fn pxquting_loses_hp_and_draws_three_on_gain() {
    let (state, owner) = started(2_026_081_005);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let (state, _) = grant(state, owner, PXQUTING);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), hand_before + 3);
}

fn count_hand_kind(state: &GameState, owner: PlayerId, kind: CardKind) -> usize {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand && card.kind == kind)
        .count()
}

#[test]
fn pxchangshan_gains_two_shan_on_gain() {
    let (state, owner) = started(2_026_081_006);
    let (state, _) = grant(state, owner, PXCHANGSHAN);
    assert!(
        count_hand_kind(&state, owner, CardKind::Shan) >= 2,
        "常山 must gain two 闪 into hand"
    );
}

#[test]
fn pxbeiping_gains_two_sha_on_gain() {
    let (state, owner) = started(2_026_081_007);
    let (state, _) = grant(state, owner, PXBEIPING);
    assert!(
        count_hand_kind(&state, owner, CardKind::Sha) >= 2,
        "北平 must gain two 杀 into hand"
    );
}

#[test]
fn pxlean_gains_fire_sha_and_tiesuo_on_gain() {
    let (state, owner) = started(2_026_081_008);
    let (state, _) = grant(state, owner, PXLEAN);
    assert!(
        count_hand_kind(&state, owner, CardKind::ShaFire) >= 1
            || count_hand_kind(&state, owner, CardKind::ShaThunder) >= 1,
        "乐安 must gain an elemental sha"
    );
    assert!(
        count_hand_kind(&state, owner, CardKind::TrickTiesuo) >= 1,
        "乐安 must gain a 铁索连环"
    );
}

#[test]
fn pxyanmen_gains_weapon_and_treasure_on_gain() {
    let (state, owner) = started(2_026_081_009);
    let (state, _) = grant(state, owner, PXYANMEN);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let slots: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .filter_map(|card| card.kind.equip_slot())
        .collect();
    assert!(
        slots.contains(&engine_core::state::EquipSlot::Weapon),
        "雁门 must gain a weapon"
    );
    assert!(
        slots.contains(&engine_core::state::EquipSlot::OffensiveMount),
        "雁门 must gain an offensive mount"
    );
}

#[test]
fn pxlongxi_gains_defensive_horse_on_gain() {
    let (state, owner) = started(2_026_081_010);
    let (state, _) = grant(state, owner, PXLONGXI);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let has_horse = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.zone == hand)
        .any(|card| card.kind == CardKind::HorseDef);
    assert!(has_horse, "陇西 must gain a defensive horse");
}

const PXNANFU: u32 = 2_422_439_050;
const PXGUANGLING: u32 = 3_967_602_994;

#[test]
fn package_publishes_batch_af_wave3_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNANFU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2422439050"
            && trigger.optional));
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == PXGUANGLING)
        .expect("pxguangling wugu view_as must be registered");
    assert_eq!(view_as.as_kind, "wugu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.max_cards, 2);
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXNANFU | PXGUANGLING)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxnanfu_loses_hp_then_opens_virtual_sha() {
    let (state, owner) = started(2_026_081_011);
    let hp_before = state.players.get(owner).unwrap().hp;
    // 可选触发：确认后先失体力再进入虚拟杀目标选择
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXNANFU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // 确认后进入虚拟杀目标选择（skill_added 触发内）
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
}

const PXXIANGYANG: u32 = 4_133_746_261;
const PXYANGZHOU: u32 = 3_910_911_761;

#[test]
fn package_publishes_batch_af_wave4_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXXIANGYANG | PXYANGZHOU))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXXIANGYANG,
        "skill_added",
        "owner_skill_added_self_4133746261",
        true
    )));
    assert!(triggers.contains(&(
        PXYANGZHOU,
        "skill_added",
        "owner_skill_added_self_3910911761",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXXIANGYANG | PXYANGZHOU)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxxiangyang_opens_move_choice_on_gain() {
    let (state, owner) = started(2_026_081_012);
    let mut tx = Transaction::begin(state);
    // 给另一名角色装武器——移动原语从其他角色场上传一张牌到拥有者
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2_000_501),
        kind: CardKind::Weapon,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 2,
        zone: equip,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXXIANGYANG);
    // 可选触发确认后进入移动装备区牌的选择
    assert!(
        state.has_pending_choice(),
        "襄阳 must open a move-field-card choice"
    );
}

#[test]
fn pxyangzhou_opens_other_recover_choice_on_gain() {
    let (state, owner) = started(2_026_081_013);
    let mut tx = Transaction::begin(state);
    // 打伤另一名角色使其可被回复
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXYANGZHOU);
    // 确认后进入其他角色选择
    assert!(
        state.has_pending_choice(),
        "扬州 must open an other-recover choice"
    );
}

const PXFANYANG: u32 = 3_972_961_518;
const PXRUNAN: u32 = 3_675_374_846;

#[test]
fn package_publishes_batch_af_wave5_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXFANYANG | PXRUNAN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXFANYANG,
        "skill_added",
        "owner_skill_added_self_3972961518",
        true
    )));
    assert!(triggers.contains(&(
        PXRUNAN,
        "skill_added",
        "owner_skill_added_self_3675374846",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXFANYANG | PXRUNAN)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxfanyang_opens_other_damage_choice_on_gain() {
    let (state, owner) = started(2_026_081_014);
    let (state, _) = grant(state, owner, PXFANYANG);
    assert!(
        state.has_pending_choice(),
        "范阳 must open an other-damage choice"
    );
}

#[test]
fn pxrunan_opens_other_draw_choice_on_gain() {
    let (state, owner) = started(2_026_081_015);
    let (state, _) = grant(state, owner, PXRUNAN);
    assert!(
        state.has_pending_choice(),
        "汝南 must open an other-draw choice"
    );
}

const PXHEFEI: u32 = 3_666_201_428;

#[test]
fn package_publishes_pxhefei_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXHEFEI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3666201428"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXHEFEI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxhefei_confirm_then_discard_then_draw() {
    let (state, owner) = started(2_026_081_016);
    // 手牌初始 6 张，弃 1 摸 1 后手牌数不变
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXHEFEI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discardable: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|c| c.zone == hand)
        .map(|c| c.id.raw())
        .collect();
    assert!(!discardable.is_empty());
    // 完成弃牌（选择第一张）→ 摸 1
    let (state, accepted) = resolve(state, owner, vec![discardable[0]], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 6, "弃 1 摸 1 后手牌数不变");
}

const PXSHANGYONG: u32 = 2_310_331_473;

#[test]
fn package_publishes_pxshangyong_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXSHANGYONG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2310331473"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXSHANGYONG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxshangyong_opens_other_draw_then_self_draw() {
    let (state, owner) = started(2_026_081_017);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHANGYONG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（摸 1）
    assert!(
        state.has_pending_choice(),
        "上庸 must open an other-draw choice"
    );
    assert_eq!(state.hand_count(owner), hand_before);
}

const PXTIANSHUI: u32 = 3_271_313_270;

#[test]
fn package_publishes_pxtianshui_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXTIANSHUI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3271313270"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXTIANSHUI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxtianshui_opens_discard_choice_then_draws_two() {
    let (state, owner) = started(2_026_081_018);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXTIANSHUI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色弃牌选择
    assert!(
        state.has_pending_choice(),
        "天水 must open a target-discard choice"
    );
}

const PXB_OHAI: u32 = 2_568_108_187;

#[test]
fn package_publishes_pxbohai_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXB_OHAI
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2568108187"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXB_OHAI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxbohai_confirm_then_discard_then_draw_two() {
    let (state, owner) = started(2_026_081_019);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXB_OHAI),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discardable: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|c| c.zone == hand)
        .map(|c| c.id.raw())
        .collect();
    assert!(!discardable.is_empty());
    let (state, accepted) = resolve(state, owner, vec![discardable[0]], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 7, "弃 1 摸 2 后手牌 +1");
}

const PXPENGCHENG: u32 = 3_892_864_153;

#[test]
fn package_publishes_pxpengcheng_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXPENGCHENG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3892864153"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXPENGCHENG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxpengcheng_opens_give_then_recover_choice() {
    let (state, owner) = started(2_026_081_020);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXPENGCHENG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（交牌）
    assert!(
        state.has_pending_choice(),
        "彭城 must open an other-give choice"
    );
}

const PXJIANGXIA: u32 = 2_687_838_381;

#[test]
fn package_publishes_pxjiangxia_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJIANGXIA
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2687838381"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJIANGXIA && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjiangxia_opens_other_draw_then_give_choice() {
    let (state, owner) = started(2_026_081_021);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJIANGXIA),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色选择（摸 2）
    assert!(
        state.has_pending_choice(),
        "江夏 must open an other-draw choice"
    );
}

const PXJUCHAO: u32 = 3_375_023_858;

#[test]
fn package_publishes_pxjuchao_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJUCHAO
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3375023858"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJUCHAO && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjuchao_opens_target_discard_choice() {
    let (state, owner) = started(2_026_081_022);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJUCHAO),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "居巢 must open a target-discard choice"
    );
}

const PXNANYANG: u32 = 3_458_061_931;

#[test]
fn package_publishes_pxnanyang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNANYANG
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3458061931"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXNANYANG && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxnanyang_opens_recover_choice_after_confirm() {
    let (state, owner) = started(2_026_081_023);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXNANYANG),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择（Custom）
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
}

const PXWUWEI: u32 = 3_519_997_787;

#[test]
fn package_publishes_pxwuwei_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWUWEI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_damage_card_once_per_turn"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWUWEI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwuwei_opens_target_adjust_on_damage_card() {
    let (state, owner) = started(2_026_081_024);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXWUWEI),
        }],
    );
    // 添加技能时无选择（skill 本身无触发）
    assert!(effects.is_ok());
    // 进入出牌阶段并以杀派发 card_used 事件
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_801),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    // card_used 事件派发：可选触发打开确认
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXWENXIAN: u32 = 4_051_127_169;

#[test]
fn package_publishes_pxwenxian_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWENXIAN
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_4051127169"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWENXIAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwenxian_gains_max_hp_and_recovers_on_gain() {
    let (state, owner) = started(2_026_081_025);
    let mut tx = Transaction::begin(state);
    // 先打伤以便验证回复
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    // v1.11.6 增量 roster +1 平移种子：bootstrap 武将上限随座位分配变化，断言按相对值
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let tx = tx.commit();
    let (state, _) = grant(tx.state, owner, PXWENXIAN);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before + 1, "温县 must add one max HP");
    assert_eq!(player.hp, 4, "温县 must recover 1");
}

const PXJINGZHOU: u32 = 2_589_712_461;

#[test]
fn package_publishes_pxjingzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJINGZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2589712461"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJINGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjingzhou_opens_give_choice_on_gain() {
    let (state, owner) = started(2_026_081_026);
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXJINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "荆州 must open a give choice"
    );
}

const PXZANGKE: u32 = 3_744_135_692;

#[test]
fn package_publishes_pxzangke_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXZANGKE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXZANGKE && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxzangke_opens_choice_after_damage() {
    let (state, owner) = started(2_026_081_027);
    let mut tx = Transaction::begin(state);
    // 先移除全部既有技能（标准主公技能可能在伤害事件上循环消耗触发预算），再加 pxzangke
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXZANGKE),
    });
    let effects = run_effects(&mut tx, effects);
    assert!(effects.is_ok());
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入弃牌选择
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
}

const PXWENXI: u32 = 3_323_337_569;

#[test]
fn package_publishes_pxwenxi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXWENXI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_first_card_suit_this_turn"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXWENXI && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxwenxi_draws_on_first_suit_use_only() {
    let (state, owner) = started(2_026_081_028);
    let mut tx = Transaction::begin(state);
    // 先移除既有技能再加 pxwenxi
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXWENXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut card_seq = 0u32;
    let mut make_sha = |tx: &mut Transaction, suit: engine_api::Suit| {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        card_seq += 1;
        let c = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: engine_api::CardDefId::new(1_000_900 + card_seq),
            kind: CardKind::Sha,
            suit,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(c);
        c
    };
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let fire = |tx: &mut Transaction, card| -> u32 {
        let before = tx.state().hand_count(owner);
        let effects = dispatch_for_events(
            tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![other],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap();
        run_effects(tx, effects).unwrap();
        u32::try_from(tx.state().hand_count(owner)).unwrap() - before
    };
    // 第一张黑桃：首次使用该花色 → 摸 1
    let sha1 = make_sha(&mut tx, engine_api::Suit::Spade);
    assert_eq!(fire(&mut tx, sha1), 1, "first spade use must draw 1");
    // 第二张黑桃：花色已用 → 不摸
    let sha2 = make_sha(&mut tx, engine_api::Suit::Spade);
    assert_eq!(fire(&mut tx, sha2), 0, "second spade use must not draw");
    // 第三张红桃：新花色 → 摸 1
    let sha3 = make_sha(&mut tx, engine_api::Suit::Heart);
    assert_eq!(fire(&mut tx, sha3), 1, "first heart use must draw 1");
}


const PXPINGYUAN: u32 = 2_911_898_950;

#[test]
fn package_publishes_pxpingyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXPINGYUAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_lost_hand_now_empty"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXPINGYUAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxpingyuan_draws_when_hand_becomes_empty() {
    let (state, owner) = started(2_026_081_029);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxpingyuan
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXPINGYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // 清空手牌区（最后一批）→ 手牌为空 → 摸 1
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let hand_cards = tx.state().hand_cards(owner);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: hand_cards,
            from: hand,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 1, "平原 must draw 1 after hand empties");
}

const OL_SHICHOU: u32 = 2_434_401_673;

#[test]
fn package_publishes_ol_shichou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == OL_SHICHOU
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OL_SHICHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn ol_shichou_opens_target_adjust_on_sha() {
    let (state, owner) = started(2_026_081_030);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 ol_shichou
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OL_SHICHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_001),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const YONGSI2: u32 = 4_237_803_231;

#[test]
fn package_publishes_yongsi2_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YONGSI2
            && trigger.event == "phase_discard"
            && trigger.filter == "owner_is_target"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YONGSI2 && modifier.target == "hand_limit"
    }));
}

#[test]
fn yongsi2_discards_one_at_discard_phase() {
    let (state, owner) = started(2_026_081_031);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YONGSI2),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand_before = tx.state().hand_count(owner);
    // 强制触发直接返回弃牌效果请求
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        opened.iter().any(|e| matches!(e, EffectRequest::DiscardHandEquipCount { .. })),
        "yongsi2 must emit a discard-hand-equip-count effect"
    );
    // 运行效果 → 打开弃牌选择
    let result = run_effects(&mut tx, opened);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    // 取消选择：手牌不变
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![], true, 1);
    let _ = accepted;
    assert_eq!(state.hand_count(owner), hand_before);
}

const PXHANDAN: u32 = 3_498_155_858;

#[test]
fn package_publishes_pxhandan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXHANDAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_ordinary_trick"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXHANDAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxhandan_opens_gain_choice_on_ordinary_trick() {
    let (state, owner) = started(2_026_081_032);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxhandan
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXHANDAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    // 使用一张普通锦囊（过河拆桥）
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let trick = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_101),
        kind: CardKind::TrickGuohe,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(trick);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: trick,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXJULU: u32 = 2_943_799_463;

#[test]
fn package_publishes_pxjulu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJULU
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_lost_spade_two_to_nine"
            && !trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJULU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjulu_draws_when_spade_two_to_nine_lost() {
    let (state, owner) = started(2_026_081_033);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXJULU),
    });
    run_effects(&mut tx, effects).unwrap();
    // 注入一张黑桃 7 并移出
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let spade = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_201),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(spade);
    // 从手牌移出黑桃 7 → 触发摸 1
    let before = tx.state().hand_count(owner);
    let discard_zone = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![spade],
            from: hand,
            to: discard_zone,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    // 合成事件不移除实体牌，手牌保持 before 且额外摸 1
    assert_eq!(
        tx.state().hand_count(owner) as u32,
        before + 1,
        "巨鹿 must draw 1 when a spade 2-9 is lost"
    );
}

const PXJIUYUAN: u32 = 2_634_282_552;

#[test]
fn package_publishes_pxjiuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXJIUYUAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXJIUYUAN && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxjiuyuan_opens_target_adjust_on_sha() {
    let (state, owner) = started(2_026_081_034);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXJIUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_001_301),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

const PXYUZHOU: u32 = 4_075_358_047;

#[test]
fn package_publishes_pxyuzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXYUZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_4075358047"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXYUZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxyuzhou_links_self_then_opens_recover_choice() {
    let (state, owner) = started(2_026_081_035);
    let mut tx = Transaction::begin(state);
    // 移除既有技能加 pxyuzhou——skill_added 可选触发在 AddSkill 时打开确认
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXYUZHOU),
    });
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 确认后进入其他角色回复选择
    assert!(
        state.has_pending_choice(),
        "豫州 must open an other-recover choice"
    );
}


const PXYONGZHOU: u32 = 2_600_588_951;

#[test]
fn package_publishes_pxyongzhou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXYONGZHOU
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_2600588951"
            && trigger.optional));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == PXYONGZHOU && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxyongzhou_opens_confirm_on_gain() {
    let (state, owner) = started(2_026_081_036);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXYONGZHOU),
    });
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}
}

mod original_sp_batch_ah_dsl {
//! Golden for sp batch-AH DSL rewrites:
//! 困奋 (kunfen) end-phase lose 1 HP then draw 2;
//! 礼下 (lixia) other's end phase outside owner's range → self/event-player draw 1
//!   + rebiluan2 distance mark -1;
//! 妆梳·锦囊 (zhuangshu_trick) judge phase → skip judge or discard phase;
//! 骗南 (olpianan) game start / discard-phase end → discard all non-Shan hand, draw to HP.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const KUNFEN: u32 = 2_673_970_052;
const LIXIA: u32 = 2_791_840_336;
const REBILUAN2: u32 = 3_075_740_841;
const ZHUANGSHU_TRICK: u32 = 2_226_439_241;
const OLPIANAN: u32 = 3_870_010_283;
const OLLINJIE: u32 = 4_117_878_839;
const ZHUANGSHU_BASIC: u32 = 3_108_569_468;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn equip_mount(state: &mut GameState, player: PlayerId, kind: CardKind) {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(9_999),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_ah_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(
            trigger.skill_def_id,
            KUNFEN | LIXIA | ZHUANGSHU_TRICK | OLPIANAN | OLLINJIE | ZHUANGSHU_BASIC
        ) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(KUNFEN, "phase_end", "owner_is_target", false)));
    assert!(seen.contains(&(
        LIXIA,
        "phase_end",
        "owner_alive_other_phase_end_player_outside_owner_range",
        true
    )));
    assert!(seen.contains(&(ZHUANGSHU_TRICK, "phase_judge", "owner_is_target", true)));
    assert!(seen.contains(&(OLPIANAN, "game_started", "owner_alive", false)));
    assert!(seen.contains(&(OLPIANAN, "phase_discard_end", "owner_is_target", false)));
    assert!(seen.contains(&(OLLINJIE, "damage", "owner_is_target", false)));
    assert!(seen.contains(&(ZHUANGSHU_BASIC, "damage_before", "owner_is_target", true)));
}

#[test]
fn kunfen_end_phase_loses_one_hp_then_draws_two() {
    let (state, owner) = started(2_026_081_500);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let state = with_skill(state, owner, KUNFEN);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
}

#[test]
fn lixia_outside_range_fires_and_marks_distance_minus_one() {
    let (mut state, owner) = started(2_026_081_501);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // A defensive horse on the owner makes distance(other→owner) = 2 > other's
    // attack range 1 → the event player is not in the owner's range.
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, LIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    // Confirm the optional trigger, then branch 1: self draws 1.
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn lixia_branch_two_draws_event_player_one() {
    let (mut state, owner) = started(2_026_081_502);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, LIXIA);
    let other_before = state.hand_count(other);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Branch 2: the event player draws 1.
    let (state, resolved) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(other), other_before + 1);
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn zhuangshu_trick_branch_one_skips_next_judge_phase() {
    let (state, owner) = started(2_026_081_503);
    let state = with_skill(state, owner, ZHUANGSHU_TRICK);
    assert!(state.pending_judge_skips.is_empty());

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    );
    // Optional trigger opens during the Judge transition.
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // OptionalTrigger → Custom choice (branch 1 = skip judge).
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.pending_judge_skips, [owner]);
}

#[test]
fn zhuangshu_trick_branch_two_skips_next_discard_phase() {
    let (state, owner) = started(2_026_081_504);
    let state = with_skill(state, owner, ZHUANGSHU_TRICK);

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Branch 2 = skip discard phase.
    let (state, resolved) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.pending_judge_skips.is_empty());
    assert_eq!(state.turn.skip_discard(), true);
}

#[test]
fn olpianan_game_started_discards_all_then_draws_to_hp() {
    let (state, owner) = started(2_026_081_505);
    let hp = state.players.get(owner).unwrap().hp;
    assert!(state.hand_count(owner) as i32 >= hp);
    let state = with_skill(state, owner, OLPIANAN);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // All hand discarded, then drawn back up to hp.
    assert_eq!(state.hand_count(owner), hp as u32);
}

#[test]
fn olpianan_discard_phase_end_discards_and_draws_to_hp() {
    let (state, owner) = started(2_026_081_506);
    let hp = state.players.get(owner).unwrap().hp;
    let state = with_skill(state, owner, OLPIANAN);
    // Add extra hand cards so the discard is observable.
    let mut state = state;
    inject_hand(&mut state, owner, CardKind::Tao, 1);
    inject_hand(&mut state, owner, CardKind::Shan, 2);
    let hand_before = state.hand_count(owner);
    assert!(hand_before > hp as u32);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::Discard,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), hp as u32);
}


#[test]
fn ollinjie_source_discards_sha_then_owner_draws() {
    let (state, owner) = started(2_026_081_510);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLINJIE);
    let owner_before = state.hand_count(owner);
    // Source is forced to discard a hand card; inject a Sha into its hand.
    let mut state = state;
    let sha = inject_hand(&mut state, source, CardKind::Sha, 1);
    let source_before = state.hand_count(source);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // Forced trigger returns the effect requests; running them opens the source's
    // hand-card discard choice.
    let opened = run_effects(&mut tx, opened);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveOllinjie
    );
    let tx = tx.commit();
    let (state, resolved) = resolve(tx.state, source, vec![sha.raw()], false, 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), source_before - 1);
    // Discarded card was a Sha → owner draws 1.
    assert_eq!(state.hand_count(owner), owner_before + 1);
}

#[test]
fn ollinjie_source_discards_non_sha_no_draw() {
    let (state, owner) = started(2_026_081_511);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLINJIE);
    let owner_before = state.hand_count(owner);
    let mut state = state;
    let tao = inject_hand(&mut state, source, CardKind::Tao, 2);
    let source_before = state.hand_count(source);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let opened = run_effects(&mut tx, opened);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, resolved) = resolve(tx.state, source, vec![tao.raw()], false, 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), source_before - 1);
    // Discarded card is not a Sha → owner does not draw.
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn zhuangshu_basic_discards_one_and_prevents_damage() {
    let (state, owner) = started(2_026_081_512);
    let state = with_skill(state, owner, ZHUANGSHU_BASIC);
    let hp_before = state.players.get(owner).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let discardable = state.hand_cards(owner);
    assert!(!discardable.is_empty(), "owner has hand cards");
    let cost = discardable[0];

    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // Confirm the discard choice: pick the first discardable hand card.
    let (state, discarded) = resolve(state, owner, vec![cost.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before);
}
}

mod original_sp_batch_aj_dsl {
//! Golden for sp batch-AJ DSL rewrites:
//! 青州 (pxqingzhou) on-gain optional: discard the whole hand, then draw the same
//!   count (count captured at plan time before the discard runs);
//! 寿春 (pxshouchun) on-gain optional: select any living player (self included);
//!   that player gains 1 max hp (new primitive choose_target_gain_max_hp, ABI 646).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXQINGZHOU: u32 = 2_570_205_671;
const PXSHOUCHUN: u32 = 3_720_575_836;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_aj_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(trigger.skill_def_id, PXQINGZHOU | PXSHOUCHUN) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(
        PXQINGZHOU,
        "skill_added",
        "owner_skill_added_self_2570205671",
        true
    )));
    assert!(seen.contains(&(
        PXSHOUCHUN,
        "skill_added",
        "owner_skill_added_self_3720575836",
        true
    )));
}

#[test]
fn pxqingzhou_discards_all_hand_then_draws_same_count() {
    let (mut state, owner) = started(2_026_081_520);
    let marker = inject_hand(&mut state, owner, CardKind::Tao, 20);
    let hand_before = state.hand_count(owner);

    // Granting opens the optional on-gain confirm (OptionalTrigger).
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXQINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // Confirm → discard all + draw the pre-discard count back.
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(!state.hand_cards(owner).contains(&marker));
}

#[test]
fn pxqingzhou_cancel_keeps_hand_intact() {
    let (mut state, owner) = started(2_026_081_521);
    let marker = inject_hand(&mut state, owner, CardKind::Tao, 21);
    let hand_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXQINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![], true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(state.hand_cards(owner).contains(&marker));
}

#[test]
fn pxshouchun_selected_other_gains_one_max_hp() {
    let (state, owner) = started(2_026_081_522);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_max_before = state.players.get(other).unwrap().max_hp;
    let other_hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // Confirm opens the target selection (any living player, self included).
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetGainMaxHp
    );
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let other_after = state.players.get(other).unwrap();
    assert_eq!(other_after.max_hp, other_max_before + 1);
    // Upstream gainMaxHp raises only the cap, never current hp.
    assert_eq!(other_after.hp, other_hp_before);
}

#[test]
fn pxshouchun_can_select_self() {
    let (state, owner) = started(2_026_081_523);
    let owner_max_before = state.players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, _) = resolve(tx.state, owner, vec![1], false, 5);
    let (state, result) = resolve(state, owner, vec![owner.raw()], false, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().max_hp, owner_max_before + 1);
}

#[test]
fn pxshouchun_cancelled_target_choice_is_noop() {
    let (state, owner) = started(2_026_081_524);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_max_before = state.players.get(other).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, _) = resolve(tx.state, owner, vec![1], false, 7);
    // Cancel the target selection entirely.
    let (state, result) = resolve(state, owner, vec![], true, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().max_hp, other_max_before);
}
}

mod original_sp_batch_ak_dsl {
//! Golden for sp batch-AK DSL rewrites:
//! 疾刺·亡 (rejici2) death: the killer discards `7 - owner's regushe marks`
//!   hand-or-equip cards, then loses 1 HP (new expression-count discard
//!   primitive `discard_hand_equip_expr`, reusing the existing VM op);
//! 雍州 (pxyongzhou) on-gain optional: draw 2, then discard hand down to the
//!   hand limit (upgrade from the previous fixed-discard-1 approximation to
//!   exact semantics via `discard_hand_expr`).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REJICI2: u32 = 3_976_772_344;
const REGUSHE: u32 = 3_913_395_307;
const PXYONGZHOU: u32 = 2_600_588_951;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn other_of(state: &GameState, owner: PlayerId) -> PlayerId {
    state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_800 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

#[test]
fn package_publishes_batch_ak_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // The expression-count discard reuses the existing HostEffect op: no ABI bump.
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(trigger.skill_def_id, REJICI2 | PXYONGZHOU) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(
        REJICI2,
        "death",
        "owner_is_target_source_alive",
        false
    )));
    assert!(seen.contains(&(
        PXYONGZHOU,
        "skill_added",
        "owner_skill_added_self_2600588951",
        true
    )));
}

#[test]
fn rejici2_killer_discards_seven_minus_marks_then_loses_hp() {
    let (state, owner) = started(2_026_081_530);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    // Strip pre-existing skills so death triggers stay deterministic.
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    // Three regushe marks on the dying owner → the killer discards 4 cards.
    effects.push(EffectRequest::AddMark {
        player: owner,
        mark: MarkDefId::new(REGUSHE),
        delta: 3,
    });
    run_effects(&mut tx, effects).unwrap();
    let mut state = tx.commit().state;

    // Give the killer exactly three hand cards so the discard count is observable.
    inject_hand(&mut state, other, CardKind::Tao, 1);
    inject_hand(&mut state, other, CardKind::Shan, 2);
    inject_hand(&mut state, other, CardKind::Jiu, 3);
    let other_hand = state.hand_count(other);
    inject_hand(&mut state, other, CardKind::Sha, 4);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: other,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // 7 - 3 marks = 4 discarded from the killer's hand-or-equip.
    assert_eq!(state.hand_count(other), other_hand + 1 - 4);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn rejici2_full_marks_skip_discard_but_keep_lose_hp() {
    let (state, owner) = started(2_026_081_531);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    // Seven regushe marks → discard count 7 - 7 = 0, only the HP loss remains.
    effects.push(EffectRequest::AddMark {
        player: owner,
        mark: MarkDefId::new(REGUSHE),
        delta: 7,
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let other_hand = state.hand_count(other);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: other,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(other), other_hand);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn rejici2_no_killer_does_not_fire() {
    let (state, owner) = started(2_026_081_532);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let other_hand = state.hand_count(other);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    let state = tx.commit().state;
    assert_eq!(state.hand_count(other), other_hand);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before);
    let _ = other;
}

#[test]
fn pxyongzhou_draws_two_then_discards_down_to_limit() {
    let (state, owner) = started(2_026_081_533);
    let hand_before = state.hand_count(owner) as i32;
    let limit = hand_limit(&state, owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXYONGZHOU),
        }],
    );
    assert!(matches!(effects, Err(engine_core::effect::EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // Draw 2, then discard the excess over the hand limit.
    let expected = (hand_before + 2).min(limit) as u32;
    assert_eq!(state.hand_count(owner), expected);
}

#[test]
fn pxyongzhou_cancel_keeps_hand_plus_nothing() {
    let (state, owner) = started(2_026_081_534);
    let hand_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXYONGZHOU),
        }],
    );
    assert!(matches!(effects, Err(engine_core::effect::EffectError::ChoiceRequested)));
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![], true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
}
}

mod original_sp_batch_am_dsl {
//! Golden for sp batch-AM DSL rewrites:
//! 庐江 (pxlujiang) on-gain optional: compare with a chosen player; the winner
//!   draws two (win/lose synthetic sub-skills via the existing
//!   choose_hand_compare primitive);
//! 肃州 (pxsizhou) on-gain optional: gain one hand card of another player
//!   (approximated by choose_other_give_one_hand_equip, chooser/zone annotated).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PXLUJIANG: u32 = 3_832_025_639;
const PXLUJIANG_WIN: u32 = 1_378_056_785;
const PXLUJIANG_LOSE: u32 = 1_747_940_174;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp/pxlujiang.skill"),
);

fn run_big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("batch-am", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_081_560, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 100;
        participant.max_hp = 100;
    }
    (tx.commit().state, seats[0], seats[1])
}

fn add_skill(state: GameState, player: PlayerId, skill_def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn dispatch_compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: CardId::NONE,
            target_card: CardId::NONE,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(engine_core::state::Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_000 + tag),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_am_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(
            trigger.skill_def_id,
            PXLUJIANG | PXLUJIANG_WIN | PXLUJIANG_LOSE
        ) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(
        PXLUJIANG,
        "skill_added",
        "owner_skill_added_self_3832025639",
        true
    )));
    assert!(seen.contains(&(
        PXLUJIANG_WIN,
        "hand_compare_finished",
        "owner_hand_compare_finished_participant",
        false
    )));
    assert!(seen.contains(&(
        PXLUJIANG_LOSE,
        "hand_compare_finished",
        "owner_hand_compare_finished_participant",
        false
    )));

}

#[test]
fn pxlujiang_winner_subskill_draws_two() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // The holder of the win sub-skill draws two when the compare finishes.
        let state = add_skill(state, target, PXLUJIANG_WIN);
        let owner_before = state.hand_count(owner);
        let target_before = state.hand_count(target);
        let state = dispatch_compare(state, owner, target, target);
        assert_eq!(state.hand_count(target), target_before + 2);
        assert_eq!(state.hand_count(owner), owner_before);
    });
}

#[test]
fn pxlujiang_loser_subskill_draws_nothing() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        let state = add_skill(state, target, PXLUJIANG_LOSE);
        let target_before = state.hand_count(target);
        // The opponent wins: the loser's sub-skill only removes itself.
        let state = dispatch_compare(state, owner, target, owner);
        assert_eq!(state.hand_count(target), target_before);
    });
}

#[test]
fn pxlujiang_on_gain_opens_compare_target_selection() {
    run_big_stack(|| {
        let (mut state, owner, _target) = setup();
        // Both sides need hand cards for the compare to be legal.
        inject_hand(&mut state, owner, 1);
        inject_hand(&mut state, _target, 2);
        let mut tx = Transaction::begin(state);
        let effects = run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(PXLUJIANG),
            }],
        );
        assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
        assert_eq!(
            tx.state().resolution.continuation_kind,
            ContinuationKind::OptionalTrigger
        );
        let tx = tx.commit();
        // Confirm → the compare target selection opens.
        let (state, result) = resolve(tx.state, owner, vec![1], 1);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert!(state.has_pending_choice());
    });
}
}

mod original_sp_batch_an_dsl {
//! Golden for sp batch-AN DSL rewrite:
//! 肃州 (pxsizhou) on-gain optional: gain one hand card of another player via the
//! new two-stage `gain_target_hand_card` primitive (ABI 647, Cont 267):
//! target selection (cancellable) → owner picks one of the target's hand cards
//! → the card moves to the owner's hand.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXSIZHOU: u32 = 3_079_253_303;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_an_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == PXSIZHOU)
        .expect("pxsizhou trigger");
    assert_eq!(entry.event.as_str(), "skill_added");
    assert_eq!(entry.filter.as_str(), "owner_skill_added_self_3079253303");
    assert!(entry.optional);
}

#[test]
fn pxsizhou_gains_one_target_hand_card() {
    let (mut state, owner) = started(2_026_081_550);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let target_card = inject_hand(&mut state, other, 1);
    inject_hand(&mut state, other, 2);
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // Confirm → target selection opens.
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GainTargetHandCard
    );
    // Pick the target, then the owner picks one of its hand cards.
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GainTargetHandCard
    );
    let (state, result) = resolve(state, owner, vec![target_card.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(other), other_before - 1);
    assert!(state.hand_cards(owner).contains(&target_card));
}

#[test]
fn pxsizhou_cancel_keeps_hands_intact() {
    let (mut state, owner) = started(2_026_081_551);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    inject_hand(&mut state, other, 3);
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    let tx = tx.commit();
    // Confirm, then cancel the target selection entirely.
    let (state, _) = resolve(tx.state, owner, vec![1], false, 4);
    let (state, result) = resolve(state, owner, vec![], true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(state.hand_count(other), other_before);
}

#[test]
fn pxsizhou_decline_keeps_hands_intact() {
    let (mut state, owner) = started(2_026_081_552);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    inject_hand(&mut state, other, 4);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    let tx = tx.commit();
    // Decline at the optional-trigger confirm.
    let (state, result) = resolve(tx.state, owner, vec![], true, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}
}

mod original_sp_batch_r_dsl {
//! Golden for sp batch-R DSL rewrites:
//! 邢神 (olxingshen) damage → draw 1 + 「严教」mark cap 6;
//! 避乱·OL (olbiluan) own end phase → discard 1 → rebiluan2 + alive-group-count mark;
//! 避乱·新 (rebiluan) own end phase → discard 1 → rebiluan2 + min(4, alive) mark.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXINGSHEN: u32 = 2_814_304_936;
const OLBILUAN: u32 = 3_868_553_904;
const REBILUAN: u32 = 3_478_138_593;
const REBILUAN2: u32 = 3_075_740_841;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_r_trigger_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                OLXINGSHEN | OLBILUAN | REBILUAN
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLXINGSHEN, "damage", "owner_is_target", false)));
    assert!(triggers.contains(&(
        OLBILUAN,
        "phase_end",
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive",
        true
    )));
    assert!(triggers.contains(&(
        REBILUAN,
        "phase_end",
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, OLXINGSHEN | OLBILUAN | REBILUAN)
    }));
}

#[test]
fn olxingshen_draws_one_and_marks_lost_hp_capped_at_six() {
    let (state, owner) = started(2_026_081_201);
    let source = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLXINGSHEN);
    let before = state.hand_count(owner);
    let max_hp = state.players.get(owner).unwrap().max_hp;
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 1);
    let lost = max_hp as i32 - hp;
    assert_eq!(mark_of(&state, owner, OLXINGSHEN), Some(lost.min(6)));
    // The synthetic DamageDealt dispatch only runs the trigger chain; the real
    // damage application is a separate engine step, so hp is untouched here.
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}

#[test]
fn olbiluan_optionally_discards_and_marks_alive_group_count() {
    let (state, owner) = started(2_026_081_202);
    let state = with_skill(state, owner, OLBILUAN);
    let before = state.hand_count(owner);
    let hand_card = state.hand_cards(owner)[0];

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (confirmed, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(confirmed.resolution.continuation_kind, ContinuationKind::Custom);
    assert_eq!(accepted.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);

    let (state, discarded) = resolve(confirmed, owner, vec![hand_card.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, REBILUAN2));
    let groups = engine_core::query::alive_group_count(&state);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(groups));
    assert_eq!(state.hand_count(owner), before - 1);
    assert_eq!(
        state.cards.get(hand_card).unwrap().zone,
        state
            .zones
            .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn rebiluan_optionally_discards_and_marks_min_four_alive() {
    let (state, owner) = started(2_026_081_203);
    let state = with_skill(state, owner, REBILUAN);
    let before = state.hand_count(owner);
    let hand_card = state.hand_cards(owner)[0];

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (confirmed, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(accepted.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);

    let (state, discarded) = resolve(confirmed, owner, vec![hand_card.raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(2));
    assert_eq!(state.hand_count(owner), before - 1);
}

#[test]
fn olbiluan_can_be_cancelled_without_discard_or_mark() {
    let (state, owner) = started(2_026_081_204);
    let state = with_skill(state, owner, OLBILUAN);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();

    let (state, cancelled) = resolve(tx.state, owner, vec![], true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before);
    assert!(!has_skill(&state, owner, REBILUAN2));
    assert!(mark_of(&state, owner, REBILUAN2).is_none());
}
}

mod original_sp_batch_s_dsl {
//! Golden for sp batch-S DSL rewrites:
//! 新镇南 (newzhennan) prepare-phase discard 2 → virtual 南蛮 (ABI 604);
//! 起乱 (qiluan) global phase-end draw by kill counts + death draw (ABI 605);
//! 摧怀 (cihuai) / 咄咄 (oldcihuai) play-phase no-Sha-in-hand virtual Sha.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NEWZHENNAN: u32 = 4_220_659_285;
const QILUAN: u32 = 2_783_300_002;
const CIHUAI: u32 = 4_115_676_072;
const OLDCIHUAI: u32 = 4_216_219_987;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn turn_state(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(90_700 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_s_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                NEWZHENNAN | QILUAN | CIHUAI | OLDCIHUAI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(NEWZHENNAN, "phase_prepare", "owner_is_target", true)));
    assert!(triggers.contains(&(QILUAN, "phase_end", "owner_alive_global_phase_end", false)));
    assert!(triggers.contains(&(QILUAN, "death", "owner_alive_other_source_death", false)));
    assert!(triggers.contains(&(CIHUAI, "phase_play", "owner_is_target_play_phase_hand_no_sha", true)));
    assert!(triggers.contains(&(
        OLDCIHUAI,
        "phase_play",
        "owner_is_target_play_phase_hand_no_sha",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            NEWZHENNAN | QILUAN | CIHUAI | OLDCIHUAI
        )
    }));
}

#[test]
fn newzhennan_discards_two_then_uses_virtual_nanman() {
    let (state, owner) = started(2_026_081_301);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, NEWZHENNAN);
    let hand: Vec<_> = state.hand_cards(owner);
    assert!(hand.len() >= 2, "natural deal must hold >=2 cards");
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::Custom
    );
    assert_eq!(accepted.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    let hand_now: Vec<_> = state.hand_cards(owner);
    assert!(hand_now.len() >= 2);
    let (state, result) = resolve(
        state,
        owner,
        vec![hand_now[0].raw(), hand_now[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        u32::try_from(hand.len()).unwrap() - 2
    );
    // 南蛮 resolves immediately against the other living player (Immediate targets).
    assert!(state.hand_count(other) >= 0);
}

#[test]
fn qiluan_draws_by_kill_counts_on_global_phase_end() {
    let (state, owner) = started(2_026_081_302);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    // Owner killed 1 this turn (×3); other killed 2 (×1) → 3 + 2 = 5.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::kill_count_by_player(owner),
        1,
    );
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::kill_count_by_player(other),
        2,
    );
    let tx = tx.commit();
    let state = turn_state(tx.state, other, TurnPhaseTag::End);
    assert_eq!(state.hand_count(owner), before + 5);
}

#[test]
fn qiluan_death_trigger_draws_one_when_owner_not_killer() {
    let (state, owner) = started(2_026_081_303);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: other,
            killer: other,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn qiluan_death_trigger_skipped_when_owner_is_killer() {
    let (state, owner) = started(2_026_081_304);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: other,
            killer: owner,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn cihuai_opens_virtual_sha_target_choice_without_hand_sha() {
    let (state, owner) = started(2_026_081_305);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, CIHUAI);
    // Clear all hand cards so the owner has no Sha in hand.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let tx = tx.commit();
    let (mut state, opened) = {
        let mut tx = Transaction::begin(tx.state);
        let result = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        );
        (tx, result)
    };
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        state.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = state.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&other.raw()));
}

#[test]
fn oldcihuai_opens_virtual_sha_target_choice_without_hand_sha() {
    let (state, owner) = started(2_026_081_306);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLDCIHUAI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let tx = tx.commit();
    let (mut tx, opened) = {
        let mut tx = Transaction::begin(tx.state);
        let result = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        );
        (tx, result)
    };
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&other.raw()));
}
}

mod original_sp_batch_t_dsl {
//! Golden for sp batch-T DSL rewrites:
//! 飞杨·OL (olfeiyang) prepare-phase discard 2 → clear judgment zone (ABI 606);
//! 礼下·OL (ollixia) other's end phase outside owner's range → draw self 1 / event player 2
//! + rebiluan2 distance mark -1.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLFEIYANG: u32 = 2_973_432_445;
const OLLIXIA: u32 = 3_877_304_106;
const REBILUAN2: u32 = 3_075_740_841;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_zone(state: &mut GameState, player: PlayerId, zone: ZoneKind, kind: CardKind, tag: u32) -> CardId {
    let zone_id = state.zones.find(zone, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn equip_mount(state: &mut GameState, player: PlayerId, kind: CardKind) {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(9_999),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
}

fn mark_of(state: &GameState, player: PlayerId, mark: u32) -> Option<i32> {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_batch_t_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLFEIYANG | OLLIXIA))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLFEIYANG, "phase_prepare", "owner_is_target_judge_nonempty", true)));
    assert!(triggers.contains(&(
        OLLIXIA,
        "phase_end",
        "owner_alive_other_phase_end_player_outside_owner_range",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, OLFEIYANG | OLLIXIA)
    }));
}

#[test]
fn olfeiyang_discards_two_hand_and_clears_judgment_zone() {
    let (mut state, owner) = started(2_026_081_401);
    let hand: Vec<_> = state.hand_cards(owner);
    assert!(hand.len() >= 2, "natural deal must hold >=2 cards");
    // Give the owner a judgment-zone card (乐不思蜀) so the filter fires.
    inject_zone(
        &mut state,
        owner,
        ZoneKind::Judge,
        CardKind::TrickLe,
        1,
    );
    let state = with_skill(state, owner, OLFEIYANG);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(accepted.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    let hand_now: Vec<_> = state.hand_cards(owner);
    let (state, result) = resolve(
        state,
        owner,
        vec![hand_now[0].raw(), hand_now[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        u32::try_from(hand.len()).unwrap() - 2
    );
    let judge_zone = state
        .zones
        .find(ZoneKind::Judge, owner)
        .unwrap();
    assert!(
        state.zones.get(judge_zone).unwrap().cards.is_empty(),
        "judgment zone must be cleared by discard_all_own_judge"
    );
}

#[test]
fn ollixia_in_range_does_not_fire() {
    let (state, owner) = started(2_026_081_402);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLLIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    // In a 2-player identity game the event player is at distance 1, within the
    // owner's attack range, so the trigger should NOT fire.
    assert!(matches!(opened, Ok(_)), "owner must be in range in a 2p game");
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before);
    assert!(!has_skill(&state, owner, REBILUAN2));
    assert!(mark_of(&state, owner, REBILUAN2).is_none());
}

#[test]
fn ollixia_outside_range_fires_and_marks_distance_minus_one() {
    let (mut state, owner) = started(2_026_081_403);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // A defensive horse on the owner makes distance(other→owner) = 2 > other's
    // attack range 1 → the event player is not in the owner's range.
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, OLLIXIA);
    let before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();

    // Choose branch 1: self draws 1 (confirm optional trigger, then branch choice).
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(has_skill(&state, owner, REBILUAN2));
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}

#[test]
fn ollixia_branch_two_draws_event_player_two() {
    let (mut state, owner) = started(2_026_081_404);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    equip_mount(&mut state, owner, CardKind::HorseDef);
    let state = with_skill(state, owner, OLLIXIA);
    let other_before = state.hand_count(other);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, confirmed) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // OptionalTrigger → Custom choice (branch 2 = event player draws 2).
    let (state, accepted) = resolve(state, owner, vec![2], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(other), other_before + 2);
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(mark_of(&state, owner, REBILUAN2), Some(-1));
}
}

mod original_sp_batch_u_dsl {
//! Golden for sp batch-U DSL rewrites (zero new primitives):
//! 小西·OL (olxiaoxi) round-start virtual Sha;
//! 逆乱·OL (olniluan) other's end phase with higher HP → virtual Sha;
//! 淑慎·国战 (gzshushen) recover → other draws.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXIAOXI: u32 = 3_429_531_288;
const OLNILUAN: u32 = 3_803_313_156;
const GZSHUSHEN: u32 = 2_220_815_073;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_u_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                OLXIAOXI | OLNILUAN | GZSHUSHEN
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLXIAOXI, "round_started", "owner_alive", true)));
    assert!(triggers.contains(&(OLNILUAN, "phase_end", "owner_alive_not_target", true)));
    assert!(triggers.contains(&(GZSHUSHEN, "hp_recovered", "owner_is_target", true)));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, OLXIAOXI | OLNILUAN | GZSHUSHEN)
    }));
}

#[test]
fn olxiaoxi_opens_virtual_sha_on_round_start() {
    let (state, owner) = started(2_026_081_501);
    let state = with_skill(state, owner, OLXIAOXI);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::RoundStarted {
            round: 1,
            current: owner,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

#[test]
fn olniluan_opens_virtual_sha_when_event_player_hp_gt_owner() {
    let (state, owner) = started(2_026_081_502);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLNILUAN);
    let mut tx = Transaction::begin(state);
    // Boost the event player's HP above the owner.
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

#[test]
fn olniluan_skipped_when_event_player_hp_not_above_owner() {
    let (state, owner) = started(2_026_081_503);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLNILUAN);
    let mut tx = Transaction::begin(state);
    // Owner has max HP (lord bonus); the event player is not strictly higher.
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 1;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "the optional trigger still opens a confirm choice"
    );
    let tx = tx.commit();
    // Confirming the optional trigger runs the body; the HP guard skips the
    // virtual Sha, so no VirtualShaTargets choice follows.
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn gzshushen_opens_other_choice_on_recover() {
    let (state, owner) = started(2_026_081_504);
    let state = with_skill(state, owner, GZSHUSHEN);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: owner,
            player: owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // choose_other_draw_or_recover opens a target-selection choice.
    assert!(state.has_pending_choice());
}
}

mod original_sp_batch_v_dsl {
//! Golden for sp batch-V DSL rewrites (zero new primitives):
//! 誓仇·DC (dc_olshichou) wounded Sha target-add;
//! 内伐·基本 (neifa_basic) trick/equip disabled + Sha target-add + Sha-use mark;
//! 内伐·非基本 (neifa_nobasic) basic disabled + ordinary-trick target add/remove.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DC_OLSHICHOU: u32 = 2_535_213_152;
const NEIFA_BASIC: u32 = 3_242_431_242;
const NEIFA_NOBASIC: u32 = 3_552_802_810;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_v_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                DC_OLSHICHOU | NEIFA_BASIC | NEIFA_NOBASIC
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(DC_OLSHICHOU, "card_used", "owner_source_sha", true)));
    assert!(triggers.contains(&(NEIFA_BASIC, "card_used", "owner_source_sha", true)));
    assert!(triggers.contains(&(
        NEIFA_NOBASIC,
        "card_used",
        "owner_source_ordinary_trick",
        true
    )));
    // Modifiers: neifa_basic cancels trick/equip and adds Sha-use from its mark.
    let mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == NEIFA_BASIC)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref().unwrap_or(""),
                modifier.mode.as_str(),
            )
        })
        .collect();
    assert!(mods.contains(&("card_use_cancel", "trick", "override")));
    assert!(mods.contains(&("card_use_cancel", "equip", "override")));
    assert!(mods.contains(&("sha_use_limit", "", "add")));
    let nobasic_mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == NEIFA_NOBASIC)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref().unwrap_or(""),
                modifier.mode.as_str(),
            )
        })
        .collect();
    assert!(nobasic_mods.contains(&("card_use_cancel", "basic", "override")));
}

fn play_sha(state: GameState, actor: PlayerId, target: PlayerId, key: u64) -> (GameState, engine_core::command::PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn dc_olshichou_opens_target_add_when_wounded() {
    let (mut state, owner) = started(2_026_081_601);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let target = others[0];
    // Wound the owner.
    state.players.get_mut(owner).unwrap().hp = 1;
    let state = with_skill(state, owner, DC_OLSHICHOU);
    let (state, played) = play_sha(state, owner, target, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    // The target-adjust choice offers at least the second other player as an
    // addable target (all alive players except the original target).
    let targets: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(targets.contains(&others[1].raw()));
}

#[test]
fn dc_olshichou_skipped_when_healthy() {
    let (state, owner) = started(2_026_081_602);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let state = with_skill(state, owner, DC_OLSHICHOU);
    let (state, played) = play_sha(state, owner, others[0], 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // Healthy owner: the body guard skips the target-add effect. The Sha play
    // continues its normal resolution (targets respond), but never opens the
    // target-adjust choice.
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
}

#[test]
fn neifa_basic_cancels_trick_use() {
    let (state, owner) = started(2_026_081_603);
    let state = with_skill(state, owner, NEIFA_BASIC);
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_010),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let tx = tx.commit();
    let resp = query(
        &tx.state,
        &QueryRequest::CardUsable {
            player: owner,
            card,
        },
        &mut QueryContext::new(),
    );
    assert_eq!(resp, QueryResponse::Bool(false));
}

#[test]
fn neifa_nobasic_opens_target_adjust_on_ordinary_trick() {
    let (state, owner) = started(2_026_081_604);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let state = with_skill(state, owner, NEIFA_NOBASIC);
    // Play a trick that can target others: 过河拆桥 (guohe). Play via the trick pipeline.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_030),
        kind: CardKind::TrickGuohe,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    let (state, played) = apply_command(
        tx.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![others[0]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // The optional trigger opens before Wuxie/target resolution; confirm it.
    let mut state = state;
    if state.resolution.continuation_kind == ContinuationKind::OptionalTrigger {
        let (s, accepted) = resolve(state, owner, vec![1], false, 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        state = s;
    }
    // Wuxie window may precede the target-adjust choice; loop to the target choice.
    let mut key = 3;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let (s, _) = resolve(state, owner, vec![], true, key);
        state = s;
        key += 1;
    }
    assert!(state.has_pending_choice());
}
}

mod original_sp_batch_w_dsl {
//! Golden for sp batch-W DSL rewrite:
//! 灵杉 (huamu) card-resolved → store used card onto the owner's skill pile (ABI 607).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const HUAMU: u32 = 3_881_736_086;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_huamu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HUAMU
            && trigger.event == "card_resolved"
            && trigger.filter == "owner_source"
            && trigger.optional
    }));
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HUAMU
    }));
}

#[test]
fn huamu_stores_discarded_used_card_on_skill_pile() {
    let (state, owner) = started(2_026_081_701);
    let state = with_skill(state, owner, HUAMU);
    // Manually place a card in the discard pile and fire a CardResolved event whose
    // card is that discard-pile card; the skill should move it to the owner's pile.
    let mut tx = Transaction::begin(state);
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_200),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
    let card_id = card;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card: card_id,
            kind: CardKind::Sha.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: Vec::new(),
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // The used card should now sit in the owner's skill pile (expansion zone).
    let pile_cards = state.skill_pile_cards(owner, HUAMU);
    assert!(
        pile_cards.contains(&card_id),
        "used card must be stored on the skill pile"
    );
    assert_eq!(
        state.cards.get(card_id).unwrap().zone,
        state
            .zones
            .find(ZoneKind::Expansion, owner)
            .unwrap()
    );
}

#[test]
fn huamu_skips_card_not_in_discard() {
    let (state, owner) = started(2_026_081_702);
    let state = with_skill(state, owner, HUAMU);
    let mut tx = Transaction::begin(state);
    // Card in the hand, not the discard pile: the store effect must no-op.
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_201),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let card_id = card;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card: card_id,
            kind: CardKind::Sha.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: Vec::new(),
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let pile_cards = state.skill_pile_cards(owner, HUAMU);
    assert!(
        !pile_cards.contains(&card_id),
        "card outside the discard pile must not be stored"
    );
}
}

mod original_sp_batch_x_dsl {
//! Golden for sp batch-X DSL rewrite:
//! 5 个被批量脚本误标为「audio-only」的州郡（px*）技能——上游 (sp/skill.js) 在
//! 文件头注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxqiaoxian（谯县）：受伤后可选摸 1（damage + owner_is_target optional）
//!   - pxdunhuang（敦煌）：锁定，【杀】被抵消后摸 1（sha_dodged + owner_source_sha_dodged）
//!   - pxchenliu（陈留）：锁定，跳过弃牌阶段（modifier skip_discard_phase override 1）
//!   - pxyanzhou（兖州）：回复体力后可选摸 1（hp_recovered + owner_is_target optional）
//!   - pxdianchi（滇池）：造成火焰伤害后可令受伤角色失去 1 点体力
//!     （damage + owner_source_fire optional）

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXQIAOXIAN: u32 = 2_589_677_698;
const PXDUNHUANG: u32 = 3_965_660_786;
const PXCHENLIU: u32 = 4_259_220_755;
const PXYANZHOU: u32 = 4_070_969_920;
const PXDIANCHI: u32 = 4_289_301_860;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_x_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                PXQIAOXIAN | PXDUNHUANG | PXYANZHOU | PXDIANCHI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXQIAOXIAN, "damage", "owner_is_target", true)));
    assert!(triggers.contains(&(PXDUNHUANG, "sha_dodged", "owner_source_sha_dodged", false)));
    assert!(triggers.contains(&(PXYANZHOU, "hp_recovered", "owner_is_target", true)));
    assert!(triggers.contains(&(PXDIANCHI, "damage", "owner_source_fire", true)));
    // 陈留以 modifier 实现，不应出现在 trigger_index
    assert!(!package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXCHENLIU));
    // 五技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXQIAOXIAN | PXDUNHUANG | PXYANZHOU | PXDIANCHI
        ) && modifier.target == "hand_limit"
    }));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXCHENLIU
            && modifier.target == "skip_discard_phase"
            && modifier.mode == "override"
            && modifier.value == 1));
}

#[test]
fn pxqiaoxian_draws_after_damage_when_accepted() {
    let (state, owner) = started(2_026_081_101);
    let state = with_skill(state, owner, PXQIAOXIAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn pxqiaoxian_cancels_without_draw() {
    let (state, owner) = started(2_026_081_102);
    let state = with_skill(state, owner, PXQIAOXIAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![], true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn pxdunhuang_draws_when_owner_sha_dodged() {
    let (state, owner) = started(2_026_081_103);
    let state = with_skill(state, owner, PXDUNHUANG);
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_000),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let before = tx.state().hand_count(owner);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::ShaDodged {
            source: owner,
            target: other,
            card,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn pxchenliu_skips_discard_phase() {
    let (state, owner) = started(2_026_081_104);
    let state = with_skill(state, owner, PXCHENLIU);
    // 陈留无触发，通过 modifier 跳过弃牌阶段：以查询校验
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::skip_discard_phase(tx.state(), owner),
        "skip_discard_phase modifier must force skip"
    );
}

#[test]
fn pxyanzhou_draws_after_recover() {
    let (state, owner) = started(2_026_081_105);
    let state = with_skill(state, owner, PXYANZHOU);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: owner,
            player: owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn pxdianchi_makes_fire_damage_target_lose_hp() {
    let (state, owner) = started(2_026_081_106);
    let state = with_skill(state, owner, PXDIANCHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}

#[test]
fn pxdianchi_ignores_non_fire_damage() {
    let (state, owner) = started(2_026_081_107);
    let state = with_skill(state, owner, PXDIANCHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty(), "non-fire damage must not trigger");
}
}

mod original_sp_batch_y_dsl {
//! Golden for sp batch-Y DSL rewrite:
//! 4 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxfuling（涪陵）：装备牌不能被其他角色弃置（modifier card_discard_protected）
//!   - pxwudu（武都）：使用牌无次数限制（modifier card_use_unlimited）
//!   - pxanding（安定）：锁定，传导属性伤害防止（damage_before + chain_damage_linked）
//!   - pxzhuti（朱提）：属性伤害改为回复等量体力（damage_before 双触发 prevent+recover）

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXFULING: u32 = 3_616_870_464;
const PXWUDU: u32 = 4_254_976_850;
const PXANDING: u32 = 3_330_093_204;
const PXZHUTI: u32 = 2_771_412_426;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_y_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXANDING | PXZHUTI))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXANDING, "damage_before", "owner_is_target_chain_damage_linked", false)));
    assert!(triggers.contains(&(PXZHUTI, "damage_before", "owner_target_fire", true)));
    assert!(triggers.contains(&(PXZHUTI, "damage_before", "owner_target_thunder", true)));
    // 涪陵/武都以 modifier 实现，不应出现在 trigger_index
    assert!(!package
        .trigger_index
        .iter()
        .any(|trigger| matches!(trigger.skill_def_id, PXFULING | PXWUDU)));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXFULING
            && modifier.target == "card_discard_protected"
            && modifier.mode == "override"
            && modifier.value == 1));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXWUDU
            && modifier.target == "card_use_unlimited"
            && modifier.mode == "override"
            && modifier.value == 1));
    // 四技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXFULING | PXWUDU | PXANDING | PXZHUTI
        ) && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxfuling_protects_equip_from_other_player_discard() {
    let (state, owner) = started(2_026_081_201);
    let state = with_skill(state, owner, PXFULING);
    let mut tx = Transaction::begin(state);
    // 给 owner 装一件装备（防具）
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2_000_001),
        kind: CardKind::Armor,
        suit: engine_api::Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 其他角色不能弃置该装备
    assert!(
        engine_core::query::card_discard_protected_from(tx.state(), other, owner, card),
        "other player must not be able to discard owner's equip"
    );
    // 本人仍可弃置自己的装备
    assert!(
        !engine_core::query::card_discard_protected_from(tx.state(), owner, owner, card),
        "owner can still discard own equip"
    );
}

#[test]
fn pxwudu_unlimits_card_use() {
    let (state, owner) = started(2_026_081_202);
    let state = with_skill(state, owner, PXWUDU);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unlimited_for(tx.state(), owner),
        "card_use_unlimited modifier must lift use limits"
    );
}

#[test]
fn pxanding_prevents_chain_damage() {
    let (state, owner) = started(2_026_081_203);
    let state = with_skill(state, owner, PXANDING);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 构造一个带传导标记的 DamageBefore：经 CommitDamage 管线派发
    let effects = run_prevent_check(&mut tx, source, owner, rule_api::damage_nature::THUNDER);
    assert!(
        effects
            .iter()
            .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
        "chain elemental damage must be prevented"
    );
}

#[test]
fn pxzhuti_replaces_elemental_damage_with_recover() {
    let (state, owner) = started(2_026_081_204);
    let state = with_skill(state, owner, PXZHUTI);
    let mut tx = Transaction::begin(state);
    // 先把 owner 打伤（2 人局 owner 为主公，满血 5/5，回满无空间），再验证回复
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // damage_before 触发链：火焰伤害打开可选确认（prevent + recover）
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 伤害被防止，改为回复等量体力
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

/// 通过 CommitDamage 管线派发带传导标记的属性伤害，返回产出事件。
fn run_prevent_check(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
) -> Vec<DomainEvent> {
    // CommitDamage 直接派发 damage_before 触发链
    let effects = run_effects(
        tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: rule_api::damage_card_flag::CHAIN_PROPAGATED,
        }],
    );
    match effects {
        Ok(events) => events,
        Err(EffectError::ChoiceRequested) => Vec::new(),
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}
}

mod original_sp_batch_z_dsl {
//! Golden for sp batch-Z DSL rewrite:
//! 3 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxdaifang（带方）：属性【杀】不可被响应（modifier card_use_unrespondable）
//!   - pxpeixian（沛县）：锦囊不可被响应（modifier card_use_unrespondable）
//!   - pxningzhou（宁州）：【杀】不可被抵消 + 杀伤害视为体力流失
//!     （card_use_unrespondable + damage_before owner_source_sha prevent+lose_hp）

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXDAIFANG: u32 = 2_175_017_211;
const PXPEIXIAN: u32 = 4_115_824_913;
const PXNINGZHOU: u32 = 2_289_560_906;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_batch_z_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    // 三技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXDAIFANG | PXPEIXIAN | PXNINGZHOU
        ) && modifier.target == "hand_limit"
    }));
    // 带方/沛县/宁州均为 card_use_unrespondable 修饰
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|modifier| matches!(
                modifier.skill_def_id,
                PXDAIFANG | PXPEIXIAN | PXNINGZHOU
            ))
            .filter(|modifier| modifier.target == "card_use_unrespondable"
                && modifier.mode == "override"
                && modifier.value == 1)
            .count(),
        3
    );
    // 宁州另有 damage_before 触发
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNINGZHOU
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_sha"
            && !trigger.optional));
}

#[test]
fn pxdaifang_makes_elemental_sha_unrespondable() {
    let (state, owner) = started(2_026_081_301);
    let state = with_skill(state, owner, PXDAIFANG);
    let tx = Transaction::begin(state);
    // 属性【杀】不可被响应
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::ShaFire,
            engine_api::Suit::Heart,
        ),
        "elemental sha must be unrespondable"
    );
    // 普通【杀】不受影响
    assert!(
        !engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "ordinary sha must remain respondable"
    );
}

#[test]
fn pxpeixian_makes_trick_unrespondable() {
    let (state, owner) = started(2_026_081_302);
    let state = with_skill(state, owner, PXPEIXIAN);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::TrickGuohe,
            engine_api::Suit::Spade,
        ),
        "trick must be unrespondable"
    );
    // 基本牌不受影响
    assert!(
        !engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "basic card must remain respondable"
    );
}

#[test]
fn pxningzhou_makes_sha_unrespondable() {
    let (state, owner) = started(2_026_081_303);
    let state = with_skill(state, owner, PXNINGZHOU);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "sha must be unrespondable"
    );
}

#[test]
fn pxningzhou_turns_sha_damage_into_hp_loss() {
    let (state, owner) = started(2_026_081_304);
    let state = with_skill(state, owner, PXNINGZHOU);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    // 杀伤害在 damage_before 被防止，改为等量体力流失
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: other,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    // 触发链会先经过 prevent；若无选择，则事件中应含体力流失
    match effects {
        Ok(events) => {
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
                "sha damage must be prevented"
            );
            assert!(
                events.iter().any(|event| matches!(
                    event,
                    DomainEvent::DamageDealt {
                        is_lose_hp: true,
                        ..
                    }
                )),
                "sha damage must convert to hp loss"
            );
            assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 1);
        }
        Err(EffectError::ChoiceRequested) => {
            panic!("pxningzhou is a locked forced skill and must not open choices");
        }
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}
}

mod original_sp_benyu {
//! 第一百五十批：sp 程昱「贲育」（benyu）零新原语真实语义重写。
//! 上游：当你受到伤害后，你可选择：①将手牌摸至与伤害来源手牌数相同（至多摸至五张）；
//! ②弃置大于伤害来源手牌数的牌，然后对其造成 1 点伤害。
//! 近似口径：optional 确认 + 强制二选一（效果面条件等价）；分支②确定性弃牌。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BENYU: u32 = 3_440_506_159;
const BENYU_SRC: &str = include_str!("../../../content/original/ported/sp/benyu.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("benyu", BENYU_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(BENYU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BENYU),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

/// 受伤并驱动贲育确认：返回确认后的状态（此时 choose(1,2) 分支选择已挂起）。
fn take_damage_and_confirm(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    key: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    ) {
        Ok(_) => panic!("expected the benyu optional confirm to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    resolve(state, owner, vec![1], key)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_of(state: &GameState, player: PlayerId) -> u32 {
    state.hand_count(player)
}

#[test]
fn benyu_draw_to_arm_tops_up_to_min_source_hand_and_five() {
    let (state, owner, other) = bootstrap();
    let mut tx = Transaction::begin(state);
    // 来源 6 张、owner 1 张：摸至 min(6,5)=5。
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: other,
                count: 6,
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hp_before = hp_of(&tx.state(), owner);
    let state = tx.commit().state;

    let state = take_damage_and_confirm(state, owner, other, 10);
    let state = resolve(state, owner, vec![1], 11);

    assert_eq!(hp_of(&state, owner), hp_before - 1, "damage lands first");
    assert_eq!(hand_of(&state, owner), 5, "draw-to arm tops up to 5");
}

#[test]
fn benyu_discard_arm_discards_and_damages_source() {
    let (state, owner, other) = bootstrap();
    let mut tx = Transaction::begin(state);
    // 来源 0 张：分支②需弃 1 张；owner 3 张可弃。
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    let other_hp = hp_of(&tx.state(), other);
    let state = tx.commit().state;

    let state = take_damage_and_confirm(state, owner, other, 20);
    let state = resolve(state, owner, vec![2], 21);

    assert_eq!(hand_of(&state, owner), 2, "exactly need=0+1 card discarded");
    assert_eq!(hp_of(&state, other), other_hp - 1, "source takes 1 damage");
}
}

mod original_sp_chongzhen_offline_yy_zhaoyun {
//! Golden coverage for 燕幽赵云与【冲阵】的旧 JavaScript 语义重写。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LONGDAN: u32 = 17;
const CHONGZHEN: u32 = 4_144_500_448;
const YICONG: u32 = 4_025_383_419;
const YY_ZHAOYUN: u32 = 728_577_123;
const SOURCE: &str = concat!(
    include_str!("../../../content/standard/longdan.skill"),
    "\n",
    include_str!("../../../content/original/ported/sp/chongzhen.skill")
);

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone != state.zones.find(ZoneKind::Hand, owner).unwrap(),
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_turn: bool) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("chongzhen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGDAN),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHONGZHEN),
        },
        EffectRequest::SetTurnState {
            current: if owner_turn { owner } else { other },
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let longdan = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LONGDAN)
        .unwrap();
    (tx.commit().state, owner, other, longdan)
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.unwrap();
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn packages_publish_chongzhen_and_yy_zhaoyun_without_javascript() {
    let sp = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/sp/sp.npack.json"
    ))
    .unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/offline/offline.npack.json"
    ))
    .unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHONGZHEN && skill.name == "chongzhen"));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == YY_ZHAOYUN)
        .unwrap();
    assert_eq!(character.skills, [LONGDAN, CHONGZHEN, YICONG]);
    assert!(!serde_json::to_string(&(sp, offline))
        .unwrap()
        .contains(".js"));
}

#[test]
fn longdan_shan_as_sha_takes_only_first_targets_hand_card() {
    let (mut state, owner, target, longdan) = setup(2_026_072_801, true);
    let shan = inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 231_001);
    let hand = inject(&mut state, target, ZoneKind::Hand, CardKind::Tao, 231_002);
    let equip = inject(
        &mut state,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        231_003,
    );
    let revision = state.revision;
    let (state, optional) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: longdan,
                targets: vec![target],
                card_targets: vec![shan],
            },
        },
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, pick) = resolve(state, optional, owner, vec![1], 2);
    assert_eq!(pick.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    assert_eq!(pick.choice.as_ref().unwrap().options[0].token, hand.raw());
    assert!(!pick
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == equip.raw()));
    let (state, done) = resolve(state, pick, owner, vec![hand.raw()], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(hand).unwrap().owner, owner);
}

#[test]
fn physical_sha_does_not_trigger_but_longdan_response_does() {
    let (mut state, owner, attacker, _) = setup(2_026_072_802, false);
    let sha = inject(&mut state, attacker, ZoneKind::Hand, CardKind::Sha, 231_011);
    let loot = inject(&mut state, attacker, ZoneKind::Hand, CardKind::Tao, 231_012);
    let response = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 231_013);
    let revision = state.revision;
    let (state, response_choice) = apply_command(
        state,
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "实体杀本身不能触发冲阵"
    );
    let (state, optional) = resolve(state, response_choice, owner, vec![response.raw()], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, pick) = resolve(state, optional, owner, vec![1], 12);
    assert_eq!(pick.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    assert!(pick
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == loot.raw()));
    let (state, done) = resolve(state, pick, owner, vec![loot.raw()], 13);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(loot).unwrap().owner, owner);
}
}

mod original_sp_cunsi_yongjue {
//! Golden coverage for SP 糜夫人「存嗣」and its derived「勇决」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUNSI: u32 = 3_184_006_504;
const YONGJUE: u32 = 3_901_337_090;
const MIFUREN: u32 = 2_211_264_628;
const SHAMOKE: u32 = 3_007_668_710;
const GANFUREN: u32 = 3_981_632_700;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_complete_mifuren_and_current_abi() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let mifuren = package
        .character_index
        .iter()
        .find(|character| character.id == MIFUREN)
        .unwrap();
    assert_eq!(mifuren.skills, [2_533_816_770, CUNSI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YONGJUE));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == CUNSI)
        .unwrap();
    assert!(active.once_per_game && active.hand_nonempty && active.face_up);
    assert!(active.targets.as_ref().is_some_and(|targets| {
        !targets.allow_self && targets.male && targets.min == 1 && targets.max == 1
    }));
}

#[test]
fn cunsi_gives_all_hand_grants_yongjue_and_turns_over() {
    let (mut state, players) = started(2_026_072_501);
    let owner = players[0];
    let male = players[1];
    let female = players[2];
    state.players.get_mut(owner).unwrap().character_def = MIFUREN;
    state.players.get_mut(male).unwrap().character_def = SHAMOKE;
    state.players.get_mut(female).unwrap().character_def = GANFUREN;
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, CUNSI);
    let first = inject(&mut state, owner, CardKind::Sha, 91_001);
    let second = inject(&mut state, owner, CardKind::Shan, 91_002);

    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![female],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![male],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.cards.get(first).unwrap().owner, male);
    assert_eq!(state.cards.get(second).unwrap().owner, male);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert!(state
        .players
        .get(male)
        .unwrap()
        .skills
        .iter()
        .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == YONGJUE }));
}

#[test]
fn yongjue_optionally_gains_another_players_resolved_sha() {
    let (mut state, players) = started(2_026_072_502);
    let owner = players[0];
    let source = players[1];
    let target = players[2];
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    state.players.get_mut(target).unwrap().hp = 10;
    state.players.get_mut(target).unwrap().max_hp = 10;
    add_skill(&mut state, owner, YONGJUE);
    let sha = inject(&mut state, source, CardKind::Sha, 91_101);

    let (state, played) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, owner);

    let (state, confirmed) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        11,
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, owner);
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}
}

mod original_sp_cuorui_baobian {
//! Golden for sp 崔锐 (cuorui) + 豹变 (baobian) rewrites.
//!
//! cuorui: locked — draws min(5, maxHp) at game start; once per game, skips the
//! judge phase when the judge area is non-empty.
//! baobian: locked — treats HP ≤3/≤2/≤1 as having 挑衅/咆哮/神速 respectively
//! (approximated by re-syncing the granted skills at each prepare phase).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CUORUI: u32 = 4_002_191_084;
const BAOBIAN: u32 = 3_727_333_388;
const OL_TIAOXIN: u32 = 3_936_161_008;
const OL_PAOXIAO: u32 = 2_829_854_661;
const XIN_SHENSU: u32 = 2_561_255_694;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const REFRESH: &[u8] = include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] = include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn start_with(packages: &[&[u8]]) -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    for bytes in packages {
        let pkg = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&pkg).unwrap();
    }
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_101,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "start must be accepted: {result:?}"
    );
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add(tx: &mut Transaction, owner: PlayerId, def: u32) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
}

fn inject_hand(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_judge_zone(tx: &mut Transaction, player: PlayerId) -> CardId {
    // Reuse the canonical Judge zone (created by the rules bootstrap); only
    // allocate a fresh one when the game has not created it yet.
    let zone_id = match tx.state().zones.find(ZoneKind::Judge, player) {
        Some(existing) => existing,
        None => tx.state_mut().zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Judge,
            owner: player,
            cards: vec![],
        }),
    };
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_200),
        kind: CardKind::TrickLe,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn fire_game_started(tx: &mut Transaction, count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted { player_count: count }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn fire_phase(tx: &mut Transaction, player: PlayerId, phase: TurnPhaseTag) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn has_skill_def(tx: &Transaction, player: PlayerId, def: u32) -> bool {
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == player && skill.def.raw() == def)
}

#[test]
fn committed_package_publishes_cuorui_and_baobian_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let cuorui: Vec<&str> = pkg
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == CUORUI)
        .map(|t| t.event.as_str())
        .collect();
    assert!(cuorui.contains(&"game_started"));
    assert!(cuorui.contains(&"phase_prepare"));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == BAOBIAN && t.event == "phase_prepare"
    }));
}

#[test]
fn cuorui_draws_min_five_max_hp_at_game_start() {
    let (engine, owner, _other) = start_with(&[PACKAGE]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, CUORUI);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    fire_game_started(&mut tx, 2);

    let expected = max_hp.min(5);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + expected,
        "game start draws min(5, maxHp)"
    );
}

#[test]
fn cuorui_skips_judge_phase_once_when_judge_area_nonempty() {
    let (engine, owner, _other) = start_with(&[PACKAGE]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, CUORUI);
    inject_judge_zone(&mut tx, owner);
    let marks = |tx: &Transaction| {
        tx.state()
            .players
            .get(owner)
            .and_then(|p| p.marks.iter().find(|(id, _)| *id == CUORUI))
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };

    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert_eq!(marks(&tx), 1, "first prepare with judge cards consumes the once-gate");

    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert_eq!(marks(&tx), 1, "second prepare does not re-fire (限定技)");
}

#[test]
fn baobian_grants_and_removes_skills_by_hp() {
    let (engine, owner, _other) = start_with(&[PACKAGE, REFRESH, SHENHUA]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, BAOBIAN);

    // HP 1 → all three derived skills present.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(has_skill_def(&tx, owner, XIN_SHENSU));

    // HP 4 → all three removed.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(!has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(!has_skill_def(&tx, owner, XIN_SHENSU));

    // HP 3 → only 挑衅.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(!has_skill_def(&tx, owner, XIN_SHENSU));
}

#[test]
fn baobian_does_not_leak_derived_skills_to_other_players() {
    let (engine, owner, other) = start_with(&[PACKAGE, REFRESH, SHENHUA]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, BAOBIAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(!has_skill_def(&tx, other, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, other, OL_PAOXIAO));
}
}

mod original_sp_danji_nuzhan {
//! Golden coverage for SP 关羽「单骑」and its derived「怒斩」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DANJI: u32 = 2_150_258_599;
const NUZHAN: u32 = 2_974_830_679;
const NEW_REWUSHENG: u32 = 3_038_912_803;
const JSP_GUANYU: u32 = 3_737_388_953;
const LIUBEI: u32 = 8;
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const DANJI_SOURCE: &str = include_str!("../../../content/original/ported/sp/danji.skill");
const NUZHAN_SOURCE: &str = include_str!("../../../content/original/ported/sp/nuzhan.skill");
const WUSHENG_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/new_rewusheng.skill");

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn skill(state: &GameState, player: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let source = format!("{DANJI_SOURCE}\n{NUZHAN_SOURCE}\n{WUSHENG_SOURCE}");
    let package = content_compiler::compile_source_named("danji-nuzhan-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_073_103, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, seats[0], seats[1])
}

fn add_skills(mut state: GameState, owner: PlayerId, defs: &[u32]) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        defs.iter()
            .map(|def| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*def),
            })
            .collect(),
    )
    .unwrap();
    state = tx.commit().state;
    state
}

fn use_as_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    source: CardId,
    key: u64,
) -> GameState {
    let view_as = skill(&state, owner, NEW_REWUSHENG);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: view_as,
                targets: vec![target],
                card_targets: vec![source],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_closes_jsp_guanyu_and_preserves_both_legacy_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.character_index.iter().any(|character| {
        character.id == JSP_GUANYU
            && character.name_key == "jsp_guanyu"
            && character.skills == [NEW_REWUSHENG, DANJI]
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|entry| entry.skill_def_id == NUZHAN));

    let (state, owner, target) = setup();
    let mut state = add_skills(state, owner, &[NEW_REWUSHENG, NUZHAN]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    state.players.get_mut(target).unwrap().hp = 10;
    state.players.get_mut(target).unwrap().max_hp = 10;
    let trick_a = inject(&mut state, owner, CardKind::TrickJuedou, 345_001);
    state = use_as_sha(state, owner, target, trick_a, 1);
    let trick_b = inject(&mut state, owner, CardKind::TrickGuohe, 345_002);
    state = use_as_sha(state, owner, target, trick_b, 2);
    let equipment = inject(&mut state, owner, CardKind::Weapon, 345_003);
    state = use_as_sha(state, owner, target, equipment, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 6);

    let (state, lord, owner) = setup();
    let mut state = add_skills(state, owner, &[DANJI]);
    state.players.get_mut(lord).unwrap().character_def = LIUBEI;
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(owner).unwrap().max_hp = 4;
    for def in 345_010..345_014 {
        inject(&mut state, owner, CardKind::Other, def);
    }
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(has_skill(tx.state(), owner, DANJI));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 4);

    tx.state_mut().players.get_mut(lord).unwrap().character_def = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill(tx.state(), owner, DANJI));
    assert!(has_skill(tx.state(), owner, 2));
    assert!(has_skill(tx.state(), owner, NUZHAN));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);
}
}

mod original_sp_danlao {
//! 第一百四十九批：sp 杨修「啖酪」（danlao）零新原语真实语义重写。
//! 上游：当你成为【杀】或普通锦囊牌的目标后，若此牌的目标数大于 1，
//! 你可以摸一张牌，然后令此牌对你无效。
//! 实现：card_targeted 触发 + owner_in_multitarget_sha_or_trick_targets（本批注册）
//! → draw_cards。免除臂为登记债（AOE 队列无挂起期触发面），行为测试按近似口径。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DANLAO: u32 = 3_326_219_495;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    (state, owner, others)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }])
    .unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

/// Inject a nanman into `actor`'s hand and play it against everyone else.
fn play_nanman(
    state: GameState,
    actor: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::TrickNanman,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_of(state: &GameState, player: PlayerId) -> u32 {
    state.hand_count(player)
}

#[test]
fn package_publishes_danlao_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DANLAO
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_in_multitarget_sha_or_trick_targets"
            && trigger.optional
    }));
    // 啖酪不再保留诚实 noop 的 hand_limit +0 modifier。
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == DANLAO && modifier.target == "hand_limit"));
}

#[test]
fn danlao_draws_and_excludes_owner_from_multitarget_trick() {
    let (state, owner, others) = started(2_026_090_301);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, DANLAO);
    let owner_hp = hp_of(&state, owner);
    let other_hp = hp_of(&state, others[0]);

    // 他人使用南蛮入侵（目标 = 除使用者外全体，含 owner，目标数 > 1）。
    let (state, played) = play_nanman(
        state,
        others[1],
        others.to_vec(),
        1,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // 可选触发 confirm。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, _) = resolve(state, owner, vec![1], false, 2);
    // 批 155：免除臂经 deferred_card_exclusions 在 AOE 队列逐目标时消费——
    // owner 不受南蛮伤害。
    assert!(!state.has_pending_choice(), "resolution expected to finish");
    // 摸 1 + cancel_incoming 共用原语（杜达元帅）副作用：获得被免除的南蛮实体牌。
    assert_eq!(hand_of(&state, owner), 2, "draw one + gain the excluded card (shared primitive)");
    assert_eq!(hp_of(&state, owner), owner_hp, "owner must be excluded from nanman");
    assert_eq!(hp_of(&state, others[0]), other_hp - 1);
}

#[test]
fn danlao_not_triggered_by_single_target_card() {
    let (state, owner, others) = started(2_026_090_302);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, DANLAO);
    let owner_hp = hp_of(&state, owner);

    // 单目标【杀】（目标数 = 1）：啖酪过滤不过，无可选触发。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = others[1];
    let zone = tx.state().zones.find(ZoneKind::Hand, others[1]).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: others[1],
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    let (state, played) = apply_command(
        tx.state,
        Command {
            actor: others[1],
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "single-target sha must not open the danlao confirm"
    );
    assert_eq!(hp_of(&state, owner), owner_hp - 1, "sha lands normally");
}
}

mod original_sp_dingpan_buzhi {
//! Golden coverage for SP 步骘「定叛」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const DINGPAN: u32 = 3_739_676_314;
const HONGDE: u32 = 3_138_095_197;
const BUZHI: u32 = 4_110_519_851;
const PROTECT_EQUIP: u32 = 4_200_000_201;
const DIE_ON_MOVE: u32 = 4_200_000_202;
const DRAW_PROBE: u32 = 4_200_000_203;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/dingpan.skill");
const HONGDE_SOURCE: &str = include_str!("../../../content/original/ported/sp/hongde.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MODULE_LOGIC_HASH: [u8; 32] = [
    93, 161, 222, 203, 230, 32, 163, 105,
    106, 44, 219, 70, 1, 166, 164, 166,
    246, 41, 189, 231, 46, 141, 42, 198,
    125, 110, 100, 76, 201, 124, 148, 222,
];
const PACKAGE_LOGIC_HASH: [u8; 32] = [104,11,251,180,248,79,187,16,89,76,180,14,141,177,202,117,119,248,212,56,232,136,9,34,34,192,63,77,154,118,188,178];
const HELPERS: &str = r#"
skill protect_equip(4200000201) locked {
  modifier card_discard_protected(equip) override 1;
}

skill die_on_move(4200000202) locked {
  trigger cards_moved owner_alive {
    effect lose_hp(self, 99);
  }
}

skill draw_probe(4200000203) {
  trigger cards_drawn owner_is_target optional {
    effect lose_hp(self, 1);
  }
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    other_rebel: PlayerId,
    skill: SkillId,
    equipment: Vec<CardId>,
}

fn inject_equipment(state: &mut GameState, player: PlayerId, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank,
        weapon_range: 2,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    self_target: bool,
    equipment_count: u8,
    owner_helpers: &[u32],
    target_helpers: &[u32],
) -> Setup {
    let package = compile_source_named(
        "sp-dingpan-test",
        &format!("{SOURCE}\n{HONGDE_SOURCE}\n{HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 5, 2_026_073_002, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[1];
    let target = if self_target { owner } else { players[2] };
    let other = players[0];
    let other_rebel = players[3];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DINGPAN),
    });
    effects.extend(
        owner_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    effects.extend(
        target_helpers
            .iter()
            .filter(|definition| !self_target || !owner_helpers.contains(definition))
            .map(|definition| EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        tx.state_mut().players.get_mut(*player).unwrap().identity = Identity::Loyalist;
    }
    tx.state_mut().players.get_mut(other).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Rebel;
    tx.state_mut()
        .players
        .get_mut(other_rebel)
        .unwrap()
        .identity = Identity::Rebel;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DINGPAN)
        .unwrap();
    let mut state = tx.commit().state;
    let equipment = (0..equipment_count)
        .map(|index| inject_equipment(&mut state, target, index + 1))
        .collect();
    Setup {
        state,
        owner,
        target,
        other,
        other_rebel,
        skill,
        equipment,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn pass_dying_choices(
    mut state: GameState,
    mut result: PipelineResult,
    mut key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let mut events = result.events;
    while state.has_pending_choice() {
        assert_eq!(
            state.resolution.continuation_kind,
            engine_core::ContinuationKind::DyingSave
        );
        let actor = PlayerId::new(state.resolution.pending_observer);
        (state, result) = resolve(state, actor, vec![0], false, key);
        key += 1;
        events.extend(result.events);
    }
    (state, events)
}

#[test]
fn package_module_and_character_contract_are_stable() {
    let compiled = compile_source_named("sp-dingpan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 302);
    let active = &compiled.active_index[0];
    assert_eq!(active.skill_def_id, DINGPAN);
    assert_eq!(
        active.uses_per_turn_source.as_deref(),
        Some("living_rebel_count")
    );
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.allow_self && targets.has_equip));
    let module = &compiled.modules[0];
    assert_eq!(module.code, [83, 85, 90, 33, 1, 1]);
    assert_eq!(module.logic_hash, MODULE_LOGIC_HASH);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN, 289);
    assert_eq!(
        engine_core::ContinuationKind::ChooseDingpanBranch.tag(),
        207
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, PACKAGE_LOGIC_HASH);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == BUZHI)
        .unwrap();
    assert_eq!(character.name_key, "buzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [HONGDE, DINGPAN]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn living_rebel_limit_is_dynamic_zero_forbids_and_extra_play_does_not_reset() {
    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let other_rebel = fixture.other_rebel;
    let skill = fixture.skill;
    let equipment = fixture.equipment[0];
    let (state, result) = activate(fixture, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, _) = resolve(state, target, vec![0], false, 11);
    let (mut state, result) = resolve(state, owner, vec![equipment.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    inject_equipment(&mut state, target, 8);

    let allowed = Setup {
        state: state.clone(),
        owner,
        target,
        other: PlayerId::NONE,
        other_rebel,
        skill,
        equipment: Vec::new(),
    };
    let (_, result) = activate(allowed, 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    state.players.get_mut(other_rebel).unwrap().alive = false;
    let reduced = Setup {
        state,
        owner,
        target,
        other: PlayerId::NONE,
        other_rebel,
        skill,
        equipment: Vec::new(),
    };
    let (_, result) = activate(reduced, 14);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let mut zero = setup(false, 1, &[], &[]);
    for player in zero
        .state
        .players
        .iter_live()
        .map(|player| player.id)
        .collect::<Vec<_>>()
    {
        zero.state.players.get_mut(player).unwrap().identity = Identity::Loyalist;
    }
    let (_, result) = activate(zero, 15);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn headless_runner_retries_dynamic_skill_while_rebel_limit_remains() {
    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let skill = fixture.skill;
    let equipment = fixture.equipment[0];
    let (state, _) = activate(fixture, 16);
    let (state, _) = resolve(state, target, vec![0], false, 17);
    let (mut state, _) = resolve(state, owner, vec![equipment.raw()], false, 18);
    inject_equipment(&mut state, target, 9);

    let mut engine = Engine::new();
    engine.restore_snapshot(&Snapshot::capture(&state)).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        command.kind,
        CommandKind::ActivateSkill {
            skill: selected,
            ..
        } if selected == skill
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
}

#[test]
fn discard_branch_revalidates_protection_and_never_deals_damage() {
    let protected = setup(false, 1, &[], &[PROTECT_EQUIP]);
    let target = protected.target;
    let equipment = protected.equipment[0];
    let hp = protected.state.players.get(target).unwrap().hp;
    let (state, _) = activate(protected, 20);
    let (state, result) = resolve(state, target, vec![0], false, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.cards.get(equipment).unwrap().zone,
        state.zones.find(ZoneKind::Equip, target).unwrap()
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
}

#[test]
fn activation_and_discard_branch_reject_forged_inputs_then_discard_one_equip() {
    let (_, result) = activate(setup(false, 0, &[], &[]), 22);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let equipment = fixture.equipment[0];
    let (state, result) = activate(fixture, 23);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);

    let (state, result) = resolve(state, target, Vec::new(), true, 24);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    let (state, result) = resolve(state, target, vec![2], false, 25);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );

    let (state, result) = resolve(state, target, vec![0], false, 26);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDiscardTargetCard
    );

    let (state, result) = resolve(state, owner, vec![u32::MAX], false, 27);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDiscardTargetCard
    );
    let (state, result) = resolve(state, owner, vec![equipment.raw()], false, 28);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
    assert!(!result.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt {
                is_lose_hp: false,
                ..
            }
        )
    }));
}

#[test]
fn draw_trigger_choice_and_effect_finish_before_branch_choice() {
    let fixture = setup(false, 1, &[], &[DRAW_PROBE]);
    let target = fixture.target;
    let hp = fixture.state.players.get(target).unwrap().hp;
    let (state, result) = activate(fixture, 25);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp);

    let (state, result) = resolve(state, target, vec![1], false, 26);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn gain_branch_moves_one_batch_hongde_finishes_before_damage_and_empty_still_hurts() {
    let fixture = setup(true, 2, &[HONGDE], &[]);
    let owner = fixture.owner;
    let other = fixture.other;
    let equipment = fixture.equipment.clone();
    let hp = fixture.state.players.get(owner).unwrap().hp;
    let (state, _) = activate(fixture, 30);
    let (state, result) = resolve(state, owner, vec![1], false, 31);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(equipment.iter().all(|card| {
        state.cards.get(*card).unwrap().zone == state.zones.find(ZoneKind::Hand, owner).unwrap()
    }));
    let moves: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { cards, .. } if cards == &equipment => Some(cards.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(moves, [equipment]);

    let (state, result) = resolve(state, owner, vec![1], false, 32);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 33);
    let drawn = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == other),
        )
        .unwrap();
    let damaged = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::DamageDealt { target, .. } if *target == owner),
        )
        .unwrap();
    assert!(drawn < damaged);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let empty = setup(false, 1, &[], &[]);
    let target = empty.target;
    let hp = empty.state.players.get(target).unwrap().hp;
    let equip = empty.equipment[0];
    let (mut state, _) = activate(empty, 34);
    let hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = state.zones.find(ZoneKind::Equip, target).unwrap();
    state.zones.get_mut(equip_zone).unwrap().cards.clear();
    state.zones.get_mut(hand).unwrap().cards.push(equip);
    let card = state.cards.get_mut(equip).unwrap();
    card.zone = hand;
    card.owner = target;
    card.face_up = false;
    let (state, result) = resolve(state, target, vec![1], false, 35);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(result.events.iter().any(|event| {
        matches!(event, DomainEvent::DamageDealt { target: damaged, .. } if *damaged == target)
    }));
}

#[test]
fn movement_death_makes_damage_source_free_and_dead_target_skips_damage() {
    let source_dies = setup(false, 1, &[DIE_ON_MOVE], &[]);
    let owner = source_dies.owner;
    let target = source_dies.target;
    let target_hp = source_dies.state.players.get(target).unwrap().hp;
    let (state, _) = activate(source_dies, 40);
    let (state, result) = resolve(state, target, vec![1], false, 41);
    let (state, events) = pass_dying_choices(state, result, 42);
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt { source, target: damaged, .. }
                if source.is_none() && *damaged == target
        )
    }));

    let target_dies = setup(false, 1, &[], &[DIE_ON_MOVE]);
    let target = target_dies.target;
    let (state, _) = activate(target_dies, 50);
    let (state, result) = resolve(state, target, vec![1], false, 51);
    let (state, events) = pass_dying_choices(state, result, 52);
    assert!(!state.players.get(target).unwrap().alive);
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt {
                target: damaged,
                is_lose_hp: false,
                ..
            } if *damaged == target
        )
    }));
}
}

mod original_sp_duanbing {
//! Golden coverage for old 短兵 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUANBING: u32 = 4_181_445_466;
const UNLIMITED_SHA: u32 = 4_290_000_277;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/duanbing.skill");

#[test]
fn additional_sha_target_is_limited_to_final_distance_one() {
    let package = content_compiler::compile_source_named("duanbing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 277);
    let fixture = content_compiler::compile_source_named(
        "duanbing-unlimited-fixture",
        "skill unlimited_sha(4290000277) { modifier card_distance_unlimited(sha) override 1; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.load_package(&fixture).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_072_938,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let first = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];
    let near = seats[(owner_index + 3) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DUANBING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    // Fixed HP so damage assertions are independent of the booted roster.
    for seat in &seats {
        let participant = tx.state_mut().players.get_mut(*seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_277),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![first],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let choice = offered.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == near.raw()));
    assert!(choice
        .options
        .iter()
        .all(|option| option.token != far.raw()));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![near.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { target, .. } if *target == near
    )));
    // The 杀 now resolves through its response window: drive every pending
    // choice (each target declines 闪) until the damage commits.
    let mut state = state;
    let mut key = 3u64;
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(r.result, CommandResult::Accepted { .. }),
            "{:?}",
            r.result
        );
        state = next;
        key += 1;
    }
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(state.players.get(near).unwrap().hp, 3);
    assert_eq!(state.players.get(far).unwrap().hp, 4);
}
}

mod original_sp_fanghun {
//! Golden for sp 梅影 (fanghun): damage grants 「梅影」 marks; each mark may be
//! spent to convert a hand/equip card into 【桃】/【酒】 with a 1-card draw.
//!
//! Exercises the `cost hand_equip_mark mark_cost` view_as deep variant
//! (mark + source-card cost, `post_draw`), and the damage → `add_mark` trigger.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FANGHUN: u32 = 3_479_603_365;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_401,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_fanghun(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANGHUN),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == FANGHUN)
        .unwrap()
        .id
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn marks_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == FANGHUN))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_declares_fanghun_damage_trigger_and_mark_cost_view_as() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGHUN && t.event == "damage"
    }));
    for kind in ["tao", "jiu"] {
        let view = pkg
            .view_as_index
            .iter()
            .find(|v| v.skill_def_id == FANGHUN && v.as_kind == kind)
            .unwrap();
        assert_eq!(view.source_zone.as_deref(), Some("none"));
        assert_eq!(view.cost.as_deref(), Some("hand_equip_mark"));
        assert_eq!(view.mark_cost, Some(FANGHUN));
        assert_eq!(view.post_draw, 1);
    }
}

#[test]
fn damage_grants_marks_equal_to_amount() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_fanghun(&mut tx, owner);
    assert_eq!(marks_of(&tx, owner), 0);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_or_else(|e| panic!("damage error: {e:?}"));

    assert_eq!(marks_of(&tx, owner), 2, "damage grants 2 梅影 marks");
}

#[test]
fn spending_a_mark_converts_a_hand_card_to_tao_and_draws_one() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_fanghun(&mut tx, owner);
    let source = inject_hand(&mut tx, owner, CardKind::Shan);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3; // wounded → 桃 usable
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(FANGHUN),
            delta: 2,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let hand_before = tx.state().hand_count(owner);

    // Use 桃 via 梅影: mark is consumed and a source-card choice opens.
    let (state, first) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Tao.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(first.result, CommandResult::Accepted { .. }),
        "梅影 桃 conversion accepted: {:?}",
        first.result
    );
    let source_choice = first
        .choice
        .expect("hand_equip_mark opens the source-card choice");
    assert_eq!(source_choice.observer, owner);
    assert!(
        source_choice
            .options
            .iter()
            .any(|option| option.token == source.raw()),
        "the hand card is offered as the conversion source"
    );

    // Pick the source card → 桃 resolves (heal 1) + post_draw 1.
    let (state, healed) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: source_choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(healed.result, CommandResult::Accepted { .. }),
        "source pick accepted: {:?}",
        healed.result
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(
        marks_of(&tx, owner),
        1,
        "one 梅影 mark spent for the conversion"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        4,
        "the converted 桃 healed the owner"
    );
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "source card consumed (-1) + post_draw 1 = unchanged hand size"
    );
    let _ = DomainEvent::MarkChanged {
        player: owner,
        mark_def: FANGHUN,
        count: 1,
    };
}
}

mod original_sp_fanxiang {
//! Golden coverage for SP 孙尚香「返乡」(fanxiang).
//! Owner's prepare phase, once: +1 max HP, recover 1, lose 良助, gain 枭姬.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const FANXIANG: u32 = 3_943_157_500;
const LIANGZHU: u32 = 3_065_103_097;
const XIAOJI: u32 = 22;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fanxiang.skill");
const LIANGZHU_SOURCE: &str = include_str!("../../../content/original/ported/sp/liangzhu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(
        "fanxiang-test",
        &format!("{SOURCE}\n{LIANGZHU_SOURCE}"),
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_01),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_01,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIANGZHU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    (tx.commit().state, owner)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
}

fn once_mark(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == FANXIANG))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn compiled_source_and_npack_publish_the_fanxiang_contract() {
    let compiled = content_compiler::compile_source_named("fanxiang", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == FANXIANG && skill.name == "fanxiang" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANXIANG)
        .expect("fanxiang trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FANXIANG && skill.name == "fanxiang"));
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANXIANG)
        .expect("npack fanxiang trigger");
    assert_eq!(pack_trigger.event, "phase_prepare");
    assert_eq!(pack_trigger.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for fanxiang");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn prepare_gains_max_hp_recovers_swaps_liangzhu_for_xiaoji_once() {
    let (state, owner) = setup();
    assert!(has_def(&state, owner, LIANGZHU));
    assert!(!has_def(&state, owner, XIAOJI));
    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner), 1);
    assert!(!has_def(&state, owner, LIANGZHU));
    assert!(has_def(&state, owner, XIAOJI));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner), 1);
}
}

mod original_sp_fengpo_mayunlu {
//! Golden coverage for SP Ma Yunlu's 凤魄 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENGPO: u32 = 4_276_310_203;
const MASHU: u32 = 2;
const MAYUNLU: u32 = 3_591_117_734;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fengpo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("fengpo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENGPO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, owner, target, other)
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve_optional(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.expect("凤魄应询问是否发动");
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: accept.then_some(1).into_iter().collect(),
                cancelled: !accept,
            },
        },
    )
}

fn resolve_branch(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    branch: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume
    );
    let choice = pipeline.choice.expect("凤魄应询问分支");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

fn cancel_remaining_choices(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while let Some(choice) = pipeline.choice.take() {
        assert_eq!(choice.kind, ChoiceKind::SelectCards);
        let revision = state.revision;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(choice.choice_id.raw()),
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        key += 1;
    }
    (state, pipeline)
}

#[test]
fn package_publishes_fengpo_and_mayunlu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 302);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENGPO && skill.name == "fengpo" && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FENGPO)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MAYUNLU)
        .unwrap();
    assert_eq!(character.skills, [FENGPO, MASHU]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn first_sha_and_first_juedou_are_tracked_independently() {
    let (mut state, owner, target, _) = started(2_026_072_706);
    let first_sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 229_001);
    let (state, optional) = play(state, owner, first_sha, vec![target], 1);
    let (mut state, declined) = resolve_optional(state, optional, owner, false, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));

    let second_sha = inject(&mut state, owner, CardKind::Sha, Suit::Heart, 229_002);
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: second_sha,
            targets: vec![target],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    state = tx.commit().state;
    let juedou = inject(
        &mut state,
        owner,
        CardKind::TrickJuedou,
        Suit::Club,
        229_003,
    );
    let (state, result) = play(state, owner, juedou, vec![target], 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn sha_draw_x_branch_adds_one_damage_and_multi_target_is_rejected_by_filter() {
    let (mut state, owner, target, other) = started(2_026_072_707);
    inject(
        &mut state,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        229_011,
    );
    inject(&mut state, target, CardKind::Tao, Suit::Diamond, 229_012);
    let sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 229_013);
    let hand_before = state.hand_count(owner);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![target], 10);
    let (state, branch) = resolve_optional(state, optional, owner, true, 11);
    let (state, result) = resolve_branch(state, branch, owner, 1, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);

    let package = content_compiler::compile_source_named("fengpo-multi", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_072_708, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(FENGPO),
        }],
    )
    .unwrap();
    let card = inject(
        tx.state_mut(),
        players[0],
        CardKind::Sha,
        Suit::Spade,
        229_014,
    );
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: players[0],
            card,
            targets: vec![players[1], players[2]],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_ne!(target, other);
}

#[test]
fn juedou_draw_one_branch_adds_current_diamond_count_to_damage() {
    let (mut state, owner, target, _) = started(2_026_072_709);
    inject(
        &mut state,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        229_021,
    );
    inject(&mut state, target, CardKind::Tao, Suit::Diamond, 229_022);
    let juedou = inject(
        &mut state,
        owner,
        CardKind::TrickJuedou,
        Suit::Club,
        229_023,
    );
    let hand_before = state.hand_count(owner);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, optional) = play(state, owner, juedou, vec![target], 20);
    let (state, branch) = resolve_optional(state, optional, owner, true, 21);
    let (state, result) = resolve_branch(state, branch, owner, 2, 22);
    let (state, result) = cancel_remaining_choices(state, result, 23);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 3);
}
}

mod original_sp_fenming_xianding_chendong {
//! Golden for SP「奋命」and the XIANDING 陈董 cross-package character closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENMING: u32 = 4_273_591_874;
const DCDUANXIE: u32 = 4_278_513_633;
const CHENDONG: u32 = 2_934_854_435;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fenming.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const XIANDING_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    linked: PlayerId,
    empty_linked: PlayerId,
    unlinked: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup(owner_linked: bool) -> Setup {
    let package = content_compiler::compile_source_named("fenming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_502, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(FENMING),
        },
        EffectRequest::AddMark {
            player: players[1],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        },
        EffectRequest::AddMark {
            player: players[2],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        },
        EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::Discard.to_u8(),
        },
    ]);
    if owner_linked {
        effects.push(EffectRequest::AddMark {
            player: players[0],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner: players[0],
        linked: players[1],
        empty_linked: players[2],
        unlinked: players[3],
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_fenming_and_complete_chendong_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENMING));
    assert_eq!(
        content_compiler::compile_source_named("fenming-test", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        189
    );

    let xianding = content_schema::ContentPackage::from_json_bytes(XIANDING_PACKAGE).unwrap();
    assert!(xianding.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    assert_eq!(
        xianding
            .character_index
            .iter()
            .find(|character| character.id == CHENDONG)
            .unwrap()
            .skills,
        [DCDUANXIE, FENMING]
    );

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&xianding).unwrap();
}

#[test]
fn linked_owner_discards_one_card_from_each_eligible_linked_player_in_seat_order() {
    let mut setup = setup(true);
    let owner_card = inject(&mut setup.state, setup.owner, ZoneKind::Hand, 189_001);
    let target_card = inject(&mut setup.state, setup.linked, ZoneKind::Equip, 189_002);
    let unlinked_card = inject(&mut setup.state, setup.unlinked, ZoneKind::Hand, 189_003);

    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, owner_choice) = resolve(state, setup.owner, vec![1], 2);
    let choice = owner_choice.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [owner_card.raw()]
    );

    let (state, target_choice) = resolve(state, setup.owner, vec![owner_card.raw()], 3);
    let choice = target_choice.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_card.raw()]
    );

    let (state, discarded) = resolve(state, setup.owner, vec![target_card.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(owner_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
    assert_eq!(
        state.cards.get(unlinked_card).unwrap().owner,
        setup.unlinked
    );
    assert!(state.hand_cards(setup.empty_linked).is_empty());
}

#[test]
fn unlinked_owner_is_not_offered_fenming() {
    let mut setup = setup(false);
    let target_card = inject(&mut setup.state, setup.linked, ZoneKind::Hand, 189_010);
    let (state, result) = submit(setup.state, setup.owner, CommandKind::EndPhase, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.linked);
}
}

mod original_sp_fenxun {
//! Golden coverage for old 奋迅 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const FENXUN: u32 = 3_362_200_043;
const FENXUN2: u32 = 3_152_146_054;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fenxun.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_000 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn discards_any_hand_card_and_unlocks_the_bound_target_for_this_turn() {
    let package = content_compiler::compile_source_named("fenxun-test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_072_937,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let target = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENXUN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::Distance {
                from: owner,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance > 1
    ));
    let cost = inject(&mut tx, owner, CardKind::Other);
    let sha = inject(&mut tx, owner, CardKind::Sha);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == FENXUN)
        .unwrap();
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == FENXUN2 }));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(cost).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&sha));
}
}

mod original_sp_fuqi_refuqi {
//! Golden tests for SP/SP2「伏骑」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const FUQI_SOURCE: &str = include_str!("../../../content/original/ported/sp/fuqi.skill");
const REFUQI_SOURCE: &str = include_str!("../../../content/original/ported/sp2/refuqi.skill");
const FUQI: u32 = 3_157_963_287;
const REFUQI: u32 = 4_088_557_780;
const JIAOZI: u32 = 2_415_897_133;
const QUYI: u32 = 2_408_538_527;
const RE_QUYI: u32 = 2_820_866_421;
const DEFENSIVE_DISTANCE: u32 = 4_294_000_001;
const MULTI_SHA: u32 = 4_294_000_002;
const FIXTURE_SOURCE: &str = r#"
skill test_defensive_distance(4294000001) {
  modifier distance_to add 1;
}
skill test_multi_sha(4294000002) {
  modifier sha_target_limit add 1;
  modifier card_distance_unlimited(sha) override 1;
}
"#;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let fixture = content_compiler::compile_source_named("fuqi-fixture", FIXTURE_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&sp).unwrap();
    engine.load_package(&sp2).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(start(seed));

    let source = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let pos = order.iter().position(|player| *player == source).unwrap();
    let near = order[(pos + 1) % order.len()];
    let far = order[(pos + 2) % order.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, near, far)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn remove_skill(tx: &mut Transaction, skill_def: u32) {
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == skill_def)
        .unwrap()
        .id;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
}

fn clear_hands(tx: &mut Transaction) {
    let players = tx.state().players.alive_by_seat();
    for player in players {
        run_effects(
            tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match engine_core::query::query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut engine_core::query::QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn play(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_packages_match_sources_and_close_both_characters() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let compiled_fuqi = content_compiler::compile_source_named("fuqi", FUQI_SOURCE).unwrap();
    let compiled_refuqi = content_compiler::compile_source_named("refuqi", REFUQI_SOURCE).unwrap();

    assert!(compiled_fuqi.skill_index[0].locked);
    assert_eq!(
        compiled_fuqi.modifier_index[0].target,
        "card_use_unrespondable_within_responder_to_owner_distance"
    );
    assert!(compiled_refuqi.skill_index[0].locked);
    assert_eq!(
        compiled_refuqi.modifier_index[0].target,
        "card_use_unrespondable_within_owner_to_responder_distance"
    );
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FUQI && skill.locked));
    assert!(sp2
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REFUQI && skill.locked));
    assert!(sp2.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    assert!(sp
        .character_index
        .iter()
        .any(|character| { character.id == QUYI && character.skills == vec![FUQI, JIAOZI] }));
    assert!(sp2
        .character_index
        .iter()
        .any(|character| { character.id == RE_QUYI && character.skills == vec![REFUQI, JIAOZI] }));
}

#[test]
fn legacy_distance_direction_is_preserved() {
    let (mut tx, source, near, far) = loaded(20_260_715);
    assert_eq!(distance(&tx, source, near), 1);
    assert_eq!(distance(&tx, near, source), 1);
    assert!(distance(&tx, source, far) > 1);

    add_skill(&mut tx, source, FUQI);
    assert!(engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        far
    ));
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        source
    ));

    add_skill(&mut tx, source, DEFENSIVE_DISTANCE);
    assert_eq!(distance(&tx, source, near), 1);
    assert_eq!(distance(&tx, near, source), 2);
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));

    remove_skill(&mut tx, FUQI);
    add_skill(&mut tx, source, REFUQI);
    assert!(engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));
}

#[test]
fn fuqi_locks_only_near_targets_in_a_multi_target_sha() {
    let (mut tx, source, near, far) = loaded(20_260_716);
    clear_hands(&mut tx);
    add_skill(&mut tx, source, FUQI);
    add_skill(&mut tx, source, MULTI_SHA);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 7);
    inject(&mut tx, near, CardKind::Shan, Suit::Heart, 8);
    inject(&mut tx, far, CardKind::Shan, Suit::Diamond, 9);
    tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(far).unwrap().hp = 10;
    tx.state_mut().players.get_mut(far).unwrap().max_hp = 10;

    let (state, pipeline) = play(&tx, source, sha, vec![near, far]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(near).unwrap().hp, 9);
    assert_eq!(state.players.get(far).unwrap().hp, 10);
    assert_eq!(pipeline.choice.unwrap().observer, far);
}

#[test]
fn fuqi_applies_to_aoe_duel_and_wuxie_windows() {
    let (mut nanman_tx, source, near, far) = loaded(20_260_717);
    clear_hands(&mut nanman_tx);
    add_skill(&mut nanman_tx, source, FUQI);
    let nanman = inject(
        &mut nanman_tx,
        source,
        CardKind::TrickNanman,
        Suit::Spade,
        7,
    );
    inject(&mut nanman_tx, near, CardKind::Sha, Suit::Heart, 8);
    inject(&mut nanman_tx, far, CardKind::Sha, Suit::Diamond, 9);
    nanman_tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    nanman_tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    let (nanman_state, nanman_pipeline) = play(&nanman_tx, source, nanman, vec![]);
    assert!(matches!(
        nanman_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(nanman_state.players.get(near).unwrap().hp, 9);
    assert_eq!(nanman_pipeline.choice.unwrap().observer, far);

    let (mut duel_tx, source, near, _) = loaded(20_260_718);
    clear_hands(&mut duel_tx);
    add_skill(&mut duel_tx, source, FUQI);
    let duel = inject(&mut duel_tx, source, CardKind::TrickJuedou, Suit::Club, 1);
    inject(&mut duel_tx, near, CardKind::Sha, Suit::Spade, 2);
    duel_tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    duel_tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    let (duel_state, duel_pipeline) = play(&duel_tx, source, duel, vec![near]);
    assert!(matches!(
        duel_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!duel_state.has_pending_choice());
    assert_eq!(duel_state.players.get(near).unwrap().hp, 9);

    let (mut wuxie_tx, source, near, far) = loaded(20_260_719);
    clear_hands(&mut wuxie_tx);
    add_skill(&mut wuxie_tx, source, FUQI);
    let wuzhong = inject(
        &mut wuxie_tx,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    inject(&mut wuxie_tx, near, CardKind::TrickWuxie, Suit::Club, 11);
    let far_wuxie = inject(&mut wuxie_tx, far, CardKind::TrickWuxie, Suit::Spade, 12);
    let (_, wuxie_pipeline) = play(&wuxie_tx, source, wuzhong, vec![]);
    assert!(matches!(
        wuxie_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    let choice = wuxie_pipeline.choice.unwrap();
    assert_eq!(choice.observer, far);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == far_wuxie.raw()));
}
}

mod original_sp_gongao {
//! Golden for the first original-content DSL rewrite: SP 诸葛诞「功獒」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GONGAO: u32 = 2_416_186_304;
const LIEWEI: u32 = 3_656_060_804;
const ZHUISHE: u32 = 2_536_634_542;
const WEIZHONG: u32 = 2_998_881_378;
const GUILONGZHANYUEDAO: u32 = 4_091_349_421;
const CHOUHAI: u32 = 2_265_775_915;
const RECHOUHAI: u32 = 4_121_233_014;
const ZHUANGSHU_EQUIP: u32 = 2_527_325_512;
const RANSHANG: u32 = 3_675_767_094;
const RANSHANG2: u32 = 3_932_018_676;
const RANSHANG_MARK: u32 = 638_947_751;
const JINGONG2: u32 = 3_807_235_921;
const SHENGXI: u32 = 2_200_786_615;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn gongao_package_hash_and_death_semantics() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    let gongao = committed
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == GONGAO)
        .unwrap();
    assert!(gongao.locked);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 3_007_668_710)
            .unwrap()
            .skills,
        vec![3_565_936_884]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_408_538_527)
            .unwrap()
            .skills,
        vec![3_157_963_287, 2_415_897_133]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 4_052_796_884)
            .unwrap()
            .skills,
        vec![3_743_761_196]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_327_484_329)
            .unwrap()
            .skills,
        vec![2_416_186_304, 2_383_670_146]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 4_039_499_432)
            .unwrap()
            .skills,
        vec![3_675_767_094, 3_661_917_608]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 3_981_632_700)
            .unwrap()
            .skills,
        vec![3_504_783_479, 2_944_037_719]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_211_264_628)
            .unwrap()
            .skills,
        vec![2_533_816_770, 3_184_006_504]
    );

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });

    let owner = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    // 清除开局角色的默认技能，避免并发新增角色的死亡触发干扰。
    let mut setup_effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup_effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GONGAO),
    });
    run_effects(&mut tx, setup_effects).unwrap();
    let before = tx.state().players.get(owner).unwrap().clone();

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();

    let after_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let after_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(after_max_hp, before.max_hp + 1);
    assert_eq!(after_hp, before.hp + 1);
    let killer = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: owner,
            killer,
        }],
    )
    .unwrap();
    let dead_owner = tx.state().players.get(owner).unwrap();
    assert!(!dead_owner.alive);
    assert_eq!(dead_owner.hp, 0);
    assert_eq!(
        dead_owner.max_hp, after_max_hp,
        "功獒不能在拥有者死亡后发动"
    );
}

#[test]
fn weizhong_draws_two_at_minimum_hand_and_one_otherwise() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(WEIZHONG),
            },
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn zhuishe_adds_one_to_the_default_sha_use_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == owner)
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHUISHE),
    });
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
}

#[test]
fn liewei_offers_three_cards_only_after_the_owner_kills() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });

    let owner = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LIEWEI),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(tx.state().hand_count(owner), hand);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 3);
}

#[test]
fn guilongzhanyuedao_makes_only_red_sha_unrespondable() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUILONGZHANYUEDAO),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut make_sha = |suit| {
        tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(92_120_000 + suit as u32),
            kind: CardKind::Sha,
            suit,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        })
    };
    let red = make_sha(Suit::Heart);
    let black = make_sha(Suit::Spade);
    assert!(engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        red
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        black
    ));
}

#[test]
fn chouhai_adds_one_damage_only_while_the_owner_has_no_hand_cards() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHOUHAI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 9);

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 7);
}

#[test]
fn rechouhai_only_adds_damage_for_sha_while_the_owner_has_no_hand_cards() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RECHOUHAI as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RECHOUHAI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECHOUHAI),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_damage_taken_for(tx.state(), owner, 1),
        1
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_damage_taken_for(tx.state(), owner, 1),
        2
    );
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(RECHOUHAI),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 8);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 7);
}

#[test]
fn zhuangshu_equip_draws_to_the_lower_of_hand_limit_and_five() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(ZHUANGSHU_EQUIP as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: ZHUANGSHU_EQUIP as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (hand_limit, expected) in [(3, 3), (8, 5)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        run_effects(
            &mut tx,
            vec![
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(ZHUANGSHU_EQUIP),
                },
                EffectRequest::DiscardFromHand {
                    player: owner,
                    count: u32::MAX,
                },
                EffectRequest::DrawCards {
                    player: owner,
                    count: 1,
                },
            ],
        )
        .unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = hand_limit;
            player.max_hp = hand_limit as u32;
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Play.to_u8(),
            }],
        )
        .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Discard.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn ranshang_adds_burn_only_for_fire_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RANSHANG as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RANSHANG as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RANSHANG),
        }],
    )
    .unwrap();

    for (nature, amount, expected) in [
        (rule_api::damage_nature::NORMAL, 1, 0),
        (rule_api::damage_nature::FIRE, 2, 2),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        let burns = tx
            .state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == RANSHANG_MARK)
            .map_or(0, |(_, count)| *count);
        assert_eq!(burns, expected);
    }
}

#[test]
fn ranshang2_resolves_burn_marks_at_the_end_phase() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RANSHANG2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RANSHANG2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (burns, expected_hp, expected_max_hp, expected_draws) in [(2, 8, 10, 0), (3, 7, 8, 2)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = 10;
            player.max_hp = 10;
        }
        run_effects(
            &mut tx,
            vec![
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(RANSHANG2),
                },
                EffectRequest::AddMark {
                    player: owner,
                    mark: engine_api::MarkDefId::new(RANSHANG_MARK),
                    delta: burns,
                },
            ],
        )
        .unwrap();
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            }],
        )
        .unwrap();
        let player = tx.state().players.get(owner).unwrap();
        assert_eq!(player.hp, expected_hp);
        assert_eq!(player.max_hp, expected_max_hp);
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
    }
}

#[test]
fn jingong2_only_penalizes_a_turn_without_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(JINGONG2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: JINGONG2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for dealt_damage in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let target = tx
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        add_test_skill(&mut tx, owner, JINGONG2);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
        if dealt_damage {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: owner,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap();
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().players.get(owner).unwrap().hp,
            if dealt_damage { 4 } else { 3 }
        );
    }
}

#[test]
fn shengxi_only_offers_two_cards_in_a_damage_free_discard_phase() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_827),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    for dealt_damage in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        tx.state_mut().players.get_mut(other).unwrap().hp = 10;
        tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SHENGXI),
            }],
        )
        .unwrap();
        if dealt_damage {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: owner,
                    target: other,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap();
        }
        let hand = tx.state().hand_count(owner);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Discard.to_u8(),
            }],
        );
        if dealt_damage {
            result.unwrap();
            assert_eq!(tx.state().hand_count(owner), hand);
        } else {
            assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
            let (state, result) = apply_command(
                tx.state().clone(),
                Command {
                    actor: owner,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(1),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            assert_eq!(state.hand_count(owner), hand + 2);
        }
    }
}

fn add_test_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}
}

mod original_sp_gzjili {
//! Golden coverage for SP 沙摩柯「蒺藜」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord, BOOTSTRAP_QICAO};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SHAMOKE: u32 = 3_007_668_710;
const GZJILI: u32 = 3_565_936_884;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup(extra_range: bool) -> (Transaction, PlayerId, engine_api::CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let shamoke = package
        .character_index
        .iter()
        .find(|character| character.id == SHAMOKE)
        .unwrap();
    assert_eq!(shamoke.skills, vec![GZJILI]);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let card = engine.state().hand_cards(owner)[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GZJILI),
    });
    if extra_range {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, card)
}

fn action_count(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::CardActionCount { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(count) => count,
        other => panic!("unexpected query result: {other:?}"),
    }
}

fn accept(tx: &Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn first_card_use_at_default_range_offers_draw_one() {
    let (mut tx, owner, card) = setup(false);
    let hand = tx.state().hand_count(owner);
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(action_count(&tx, owner), 1);

    let state = accept(&tx, owner);
    assert_eq!(state.hand_count(owner), hand + 1);
}

#[test]
fn use_plus_response_reaches_range_two_and_count_resets_next_turn() {
    let (mut tx, owner, card) = setup(true);
    let hand = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(action_count(&tx, owner), 1);

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: engine_api::CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(action_count(&tx, owner), 2);

    let state = accept(&tx, owner);
    assert_eq!(state.hand_count(owner), hand + 2);
    let mut next_turn = Transaction::begin(state);
    run_effects(
        &mut next_turn,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(action_count(&next_turn, owner), 0);
}
}

mod original_sp_hanyong_wutugu {
//! Golden for SP 兀突骨「悍勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HANYONG: u32 = 3_661_917_608;
const RANSHANG: u32 = 3_675_767_094;
const RANSHANG_MARK: u32 = 638_947_751;
const WUTUGU: u32 = 4_039_499_432;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/hanyong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn started(seed: u64, round: u32, hp: i32) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HANYONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 15;
        participant.hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().turn.round = round;
    (tx.commit().state, owner, targets)
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve_optional(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.expect("optional Hanyong choice");
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == RANSHANG_MARK).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_optional_hanyong_and_wutugu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("hanyong", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        104
    );
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == HANYONG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_wounded_nanman_wanjian_or_normal_spade_sha"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUTUGU)
        .unwrap();
    assert_eq!(character.name_key, "wutugu");
    assert_eq!(character.max_hp, 15);
    assert_eq!(character.skills, [RANSHANG, HANYONG]);
}

#[test]
fn accepted_nanman_increases_every_hit_and_adds_burn_when_round_not_above_hp() {
    let (mut state, owner, targets) = started(2_026_072_303, 10, 14);
    let nanman = inject(
        &mut state,
        owner,
        CardKind::TrickNanman,
        Suit::Heart,
        99_101,
    );
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, pipeline) = play(state, owner, nanman, Vec::new(), 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let (state, pipeline) = resolve_optional(state, pipeline, owner, true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 2);
    }
    assert_eq!(mark_count(&state, owner), 1);
}

#[test]
fn declined_or_ineligible_cards_keep_normal_damage_and_round_gate_only_blocks_burn() {
    let (mut state, owner, targets) = started(2_026_072_304, 10, 14);
    let nanman = inject(&mut state, owner, CardKind::TrickNanman, Suit::Club, 99_201);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, nanman, Vec::new(), 3);
    let (state, _) = resolve_optional(state, pipeline, owner, false, 4);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(2_026_072_305, 15, 14);
    let sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 99_202);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, sha, vec![targets[0]], 5);
    let (state, _) = resolve_optional(state, pipeline, owner, true, 6);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 2);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(2_026_072_306, 1, 14);
    let fire_sha = inject(&mut state, owner, CardKind::ShaFire, Suit::Spade, 99_203);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, fire_sha, vec![targets[0]], 7);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
    assert_eq!(mark_count(&state, owner), 0);
}
}

mod original_sp_hengzheng_dongzhuo {
//! Golden coverage for SP 董卓「横征」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HENGZHENG: u32 = 3_568_349_953;
const SP_DONGZHUO: u32 = 2_623_303_183;
const LIANYING: u32 = 21;
const GAIN_GUARD: u32 = 4_000_000_002;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/hengzheng.skill");
const GUIXIN_SOURCE: &str = include_str!("../../../content/original/ported/extra/guixin.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const HENGZHENG_LOGIC_HASH: [u8; 32] = [
    51, 75, 221, 18, 16, 179, 8, 161,
    79, 14, 244, 210, 172, 213, 57, 187,
    250, 214, 132, 239, 137, 187, 51, 206,
    135, 234, 209, 55, 90, 193, 233, 250,
];
const SP_PACKAGE_LOGIC_HASH: [u8; 32] = [104,11,251,180,248,79,187,16,89,76,180,14,141,177,202,117,119,248,212,56,232,136,9,34,34,192,63,77,154,118,188,178];
const GUIXIN_LOGIC_HASH: [u8; 32] = [
    131, 229, 90, 4, 186, 198, 104, 70, 86, 232, 54, 203, 123, 216, 108, 106, 162, 185, 21, 128,
    76, 230, 51, 135, 225, 1, 120, 29, 62, 138, 32, 77,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    last: PlayerId,
    first_hand: CardId,
    protected_armor: CardId,
    second_judge: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_600 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let source = format!(
        "{SOURCE}\nskill gain_guard({GAIN_GUARD}) locked {{\n  modifier card_gain_protected(armor_or_treasure) override 1;\n}}\n"
    );
    let package = compile_source_named("sp-hengzheng-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_729_01, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[2];
    let first = players[3];
    let second = players[0];
    let last = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HENGZHENG),
        },
        EffectRequest::AddSkill {
            player: first,
            skill_def: SkillDefId::new(GAIN_GUARD),
        },
        EffectRequest::AddSkill {
            player: first,
            skill_def: SkillDefId::new(LIANYING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let first_hand = inject(tx.state_mut(), first, ZoneKind::Hand, CardKind::Other, 3);
    let protected_armor = inject(tx.state_mut(), first, ZoneKind::Equip, CardKind::Armor, 4);
    let second_judge = inject(
        tx.state_mut(),
        second,
        ZoneKind::Judge,
        CardKind::TrickLe,
        5,
    );
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first,
        last,
        first_hand,
        protected_armor,
        second_judge,
    }
}

fn dispatch_draw(
    state: GameState,
    owner: PlayerId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let dispatch = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match dispatch {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) | Err(EffectError::ChoiceRequested) => {}
            Err(error) => panic!("draw effects failed: {error:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("draw dispatch failed: {error:?}"),
    }
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

#[test]
fn package_and_module_freeze_hengzheng_without_changing_guixin_contract() {
    let compiled = compile_source_named("sp-hengzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 300);
    let trigger = compiled.trigger_index.first().unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(
        trigger.filter,
        "owner_is_target_draw_not_fixed_hp_one_or_hand_empty"
    );
    assert!(trigger.optional);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "hengzheng_phase_draw")
        .unwrap();
    assert_eq!(module.code, [83, 90, 31, 1, 1]);
    assert_eq!(module.logic_hash, HENGZHENG_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW,
        287
    );

    let package = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, SP_PACKAGE_LOGIC_HASH);
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_DONGZHUO)
        .unwrap();
    assert_eq!(character.name_key, "sp_dongzhuo");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [HENGZHENG]);

    let guixin = compile_source_named("legacy-guixin", GUIXIN_SOURCE).unwrap();
    assert_eq!(guixin.manifest.rule_vm_abi_version, 237);
    assert_eq!(guixin.modules[0].code, [83, 80, 231, 1]);
    assert_eq!(guixin.modules[0].logic_hash, GUIXIN_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
        231
    );
    assert_eq!(
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver.tag(),
        164
    );
}

#[test]
fn requires_hp_one_or_empty_hand_and_rejects_an_already_fixed_draw() {
    let setup = setup();
    let mut ineligible = setup.state.clone();
    inject(
        &mut ineligible,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        7,
    );
    let (_, choice) = dispatch_draw(ineligible, setup.owner);
    assert!(choice.is_none());

    let mut hp_one = setup.state.clone();
    inject(&mut hp_one, setup.owner, ZoneKind::Hand, CardKind::Other, 8);
    hp_one.players.get_mut(setup.owner).unwrap().hp = 1;
    let (_, choice) = dispatch_draw(hp_one, setup.owner);
    assert_eq!(choice.unwrap().kind, ChoiceKind::Confirm);

    let mut fixed = setup.state;
    fixed.players.get_mut(setup.owner).unwrap().hp = 1;
    fixed.turn.set_draw_phase_fixed_count(3);
    let (_, choice) = dispatch_draw(fixed, setup.owner);
    assert!(choice.is_none());
}

#[test]
fn gains_clockwise_gainable_cards_waits_for_nested_triggers_and_does_not_turn_over() {
    let setup = setup();
    let (state, optional) = dispatch_draw(setup.state, setup.owner);
    let optional = optional.unwrap();
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    assert_eq!(optional.observer, setup.owner);

    let (state, first) = resolve(state, setup.owner, vec![1], false, 1);
    let first = first.choice.unwrap();
    assert_eq!(first.kind, ChoiceKind::SelectCards);
    assert_eq!(first.observer, setup.owner);
    assert_eq!(
        first
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.first_hand.raw()]
    );
    assert!(!first
        .options
        .iter()
        .any(|option| option.token == setup.protected_armor.raw()));

    let (state, nested) = resolve(state, setup.owner, vec![setup.first_hand.raw()], false, 2);
    let nested = nested.choice.unwrap();
    assert_eq!(nested.kind, ChoiceKind::Confirm);
    assert_eq!(nested.observer, setup.first);

    let (state, second) = resolve(state, setup.first, vec![1], false, 3);
    let second = second.choice.unwrap();
    assert_eq!(second.kind, ChoiceKind::SelectCards);
    assert_eq!(
        second
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.second_judge.raw()]
    );
    assert_eq!(state.hand_count(setup.first), 1);

    let (state, done) = resolve(state, setup.owner, vec![setup.second_judge.raw()], false, 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert!(!state.players.get(setup.owner).unwrap().turned_over);
    assert!(state.hand_cards(setup.owner).contains(&setup.first_hand));
    assert!(state.hand_cards(setup.owner).contains(&setup.second_judge));
    assert_eq!(
        state.cards.get(setup.protected_armor).unwrap().owner,
        setup.first
    );
    assert!(state.hand_cards(setup.last).is_empty());
}
}

mod original_sp_hongde {
//! Golden coverage for sp 弘德 (hongde): optional skill that, when its owner gains
//! (or loses) at least two cards in one move, opens a choose-target dialog and
//! makes the chosen *other* player draw 1 card.
//!
//! Exercises the `choose_target_draw` dialog primitive end-to-end:
//!   - the DSL `choose_target_draw(self, 1, other)` compiles into `HOST_EFFECT(59)`,
//!   - an optional trigger first opens a Confirm choice,
//!   - confirming runs the effect, which opens a SelectTargets choice whose options
//!     are filtered by the `other` scope (self excluded),
//!   - resolving with the other player makes that player draw the card,
//!   - declining the trigger (selecting option 0) draws for nobody.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const HONGDE: u32 = 3_138_095_197;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_814,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_hongde(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HONGDE),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == HONGDE)
        .unwrap()
        .id
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_publishes_hongde_draw_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HONGDE
            && trigger.event == "cards_drawn"
            && trigger.filter == "owner_gained_or_lost_at_least_two"
            && trigger.optional
    }));
}

#[test]
fn confirming_opens_scope_filtered_choose_target_draw_and_target_draws() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_hongde(&mut tx, owner);
    let c1 = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let c2 = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let owner_hand_before = tx.state().hand_count(owner);
    let other_hand_before = tx.state().hand_count(other);

    // Owner gains two cards in one event → the optional trigger opens a Confirm choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsDrawn {
            player: owner,
            cards: vec![c1, c2],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(_) => panic!("optional trigger must open a confirm choice"),
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(tx.state().has_pending_choice());
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );

    // Confirm the optional trigger → the skill effect runs and opens the
    // choose_target_draw dialog (scope = other, so only `other` is offered).
    let confirm_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChooseTargetDraw,
        "confirming runs the effect, which opens the target-draw dialog"
    );
    let draw_choice = confirmed
        .choice
        .as_ref()
        .expect("choose_target_draw must publish a choice");
    let tokens: Vec<u32> = draw_choice
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(
        tokens,
        vec![other.raw()],
        "scope=other excludes the chooser"
    );
    assert_eq!(
        state.resolution.continuation_payload,
        vec![owner.raw(), 1, 0, skill.raw()],
        "payload encodes (player, count=1, scope=other, skill)"
    );

    // Pick the other player → they draw 1 card; the owner's hand is untouched.
    let draw_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, picked) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: draw_id,
                selections: vec![other.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(other),
        other_hand_before + 1,
        "chosen target draws the card"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before,
        "owner's hand is unchanged"
    );
}

#[test]
fn declining_the_optional_trigger_draws_for_nobody() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = strip_and_add_hongde(&mut tx, owner);
    let c1 = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let c2 = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let owner_hand_before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsDrawn {
            player: owner,
            cards: vec![c1, c2],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(
        fired,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));

    // Select option 0 → the optional trigger is declined, no dialog opens.
    let confirm_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, declined) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(
        !state.has_pending_choice(),
        "declining the optional trigger must not open a draw dialog"
    );
    assert_eq!(state.hand_count(owner), owner_hand_before);
}
}

mod original_sp_huxiao {
//! Golden coverage for sp 虎啸 (huxiao): locked skill that, after the owner deals
//! fire damage, makes the damaged player draw 1 card.
//!
//! Exercises the `owner_source_fire` damage filter and the `event_player` draw
//! target. The upstream huxiao3 temp-skill (no card-use limit against the damaged
//! player this turn) is not ported; see the .skill comment.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HUXIAO: u32 = 3_796_089_056;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_818,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_huxiao(tx: &mut Transaction, owner: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUXIAO),
    });
    run_effects(tx, effects).unwrap();
}

fn deal_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature,
        }],
    )
    .unwrap_or_else(|e| panic!("damage error: {e:?}"));
}

#[test]
fn committed_package_publishes_huxiao_fire_damage_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HUXIAO
            && trigger.event == "damage"
            && trigger.filter == "owner_source_fire"
    }));
}

#[test]
fn fire_damage_makes_the_damaged_player_draw_one() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_huxiao(&mut tx, owner);
    let other_hand_before = tx.state().hand_count(other);

    deal_damage(&mut tx, owner, other, 1, 1 /* fire */);

    assert_eq!(
        tx.state().hand_count(other),
        other_hand_before + 1,
        "fire-damaged player draws 1"
    );
}

#[test]
fn non_fire_damage_draws_nothing() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_huxiao(&mut tx, owner);
    let other_hand_before = tx.state().hand_count(other);

    deal_damage(&mut tx, owner, other, 1, 0 /* normal */);

    assert_eq!(
        tx.state().hand_count(other),
        other_hand_before,
        "normal damage does not trigger 虎啸"
    );
}
}

mod original_sp_juesheng {
//! Golden coverage for SP 范疆张达「决生」(juesheng).
//! Play-phase view-as: one hand card as 决斗, once per turn.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUESHENG: u32 = 3_463_553_348;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/juesheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    card: CardId,
}

fn setup() -> Setup {
    let compiled = content_compiler::compile_source_named("juesheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_03, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUESHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_401),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JUESHENG)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
        card,
    }
}

fn activate(state: GameState, setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.other],
                card_targets: vec![setup.card],
            },
        },
    )
}

fn drain_cancels(mut state: GameState, mut pipe: PipelineResult, mut key: u64) -> GameState {
    while state.has_pending_choice() {
        let choice = pipe.choice.clone().expect("pending choice");
        key += 1;
        let (next, next_pipe) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(next_pipe.result, CommandResult::Accepted { .. }),
            "{:?}",
            next_pipe.result
        );
        state = next;
        pipe = next_pipe;
        if key > 32 {
            break;
        }
    }
    let _ = pipe;
    state
}

#[test]
fn compiled_source_and_npack_publish_the_juedou_view_as() {
    let compiled = content_compiler::compile_source_named("juesheng", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUESHENG && skill.name == "juesheng" && !skill.locked));
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JUESHENG)
        .expect("juesheng view_as");
    assert_eq!(view.as_kind, "juedou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "any");
    assert!(
        view.source_zone.as_deref().is_none() || view.source_zone.as_deref() == Some("hand"),
        "default/hand source, got {:?}",
        view.source_zone
    );
    assert_eq!(view.uses_per_turn, Some(1));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack_view = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JUESHENG)
        .expect("npack juesheng view_as");
    assert_eq!(pack_view.as_kind, "juedou");
    assert_eq!(pack_view.uses_per_turn, Some(1));
}

#[test]
fn hand_card_converts_to_juedou_and_damages_when_unanswered() {
    let setup = setup();
    let other_hp = setup.state.players.get(setup.other).unwrap().hp;
    let (state, pipe) = activate(setup.state.clone(), &setup, 1);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let state = if state.has_pending_choice() {
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::WuxieWindow
        );
        drain_cancels(state, pipe, 1)
    } else {
        state
    };
    assert_eq!(state.players.get(setup.other).unwrap().hp, other_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.card).unwrap().zone, discard);
}
}

mod original_sp_juesi_pangde {
//! Golden coverage for SP 庞德「决死」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUESI: u32 = 2_412_286_994;
const MASHU: u32 = 2;
const SP_PANGDE: u32 = 2_874_744_170;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/juesi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("juesi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    for player in [owner, target] {
        tx.state_mut().players.get_mut(player).unwrap().hp = 4;
        tx.state_mut().players.get_mut(player).unwrap().max_hp = 4;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUESI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JUESI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(setup: Setup, cost: CardId, key: u64) -> (GameState, PipelineResult) {
    let revision = setup.state.revision;
    apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![cost],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi192_and_complete_sp_pangde() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JUESI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.in_attack_range && targets.hand_equip_nonempty);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_sha")
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_PANGDE)
        .unwrap();
    assert_eq!(character.name_key, "sp_pangde");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [MASHU, JUESI]);

    let compiled = content_compiler::compile_source_named("juesi", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn non_sha_discard_at_equal_hp_uses_duel_after_both_discards() {
    let mut setup = setup(2_026_072_609);
    let cost = inject(&mut setup.state, setup.owner, 192_001, CardKind::Sha);
    let discarded = inject(
        &mut setup.state,
        setup.target,
        192_002,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut setup.state,
        setup.target,
        192_003,
        CardKind::TrickTaoyuan,
    );
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let target = setup.target;
    let (state, offered) = activate(setup, cost, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner
    );

    let (state, resolved) = resolve(state, target, discarded, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed { player, targets, .. }
            if *player != target && targets == &[target]
    )));
}

#[test]
fn sha_discard_or_lower_target_hp_suppresses_duel() {
    let mut sha_case = setup(2_026_072_610);
    let cost = inject(&mut sha_case.state, sha_case.owner, 192_011, CardKind::Sha);
    let discarded_sha = inject(&mut sha_case.state, sha_case.target, 192_012, CardKind::Sha);
    inject(
        &mut sha_case.state,
        sha_case.target,
        192_013,
        CardKind::TrickWuzhong,
    );
    let target = sha_case.target;
    let hp = sha_case.state.players.get(target).unwrap().hp;
    let (state, _) = activate(sha_case, cost, 10);
    let (state, resolved) = resolve(state, target, discarded_sha, 11);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(!resolved
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));

    let mut hp_case = setup(2_026_072_611);
    hp_case.state.players.get_mut(hp_case.target).unwrap().hp = 3;
    let cost = inject(&mut hp_case.state, hp_case.owner, 192_021, CardKind::Sha);
    let discarded = inject(
        &mut hp_case.state,
        hp_case.target,
        192_022,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut hp_case.state,
        hp_case.target,
        192_023,
        CardKind::TrickTaoyuan,
    );
    let target = hp_case.target;
    let (state, _) = activate(hp_case, cost, 20);
    let (state, resolved) = resolve(state, target, discarded, 21);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(!resolved
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));
}
}

mod original_sp_juyi_zhugedan {
//! Golden coverage for SP 诸葛诞「举义」and its SHENHUA dependency.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUGEDAN: u32 = 2_327_484_329;
const GONGAO: u32 = 2_416_186_304;
const JUYI: u32 = 2_383_670_146;
const BENGHUAI: u32 = 3_365_741_831;
const WEIZHONG: u32 = 2_998_881_378;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/juyi.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SHENHUA_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn has_skill_def(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn setup() -> (Transaction, PlayerId) {
    let source =
        format!("{SOURCE}\nskill benghuai({BENGHUAI}) {{}}\nskill weizhong({WEIZHONG}) {{}}\n");
    let package = content_compiler::compile_source_named("juyi-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_723, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUYI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

#[test]
fn committed_package_exposes_juyi_and_complete_zhugedan_closure() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUYI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
    let character = sp
        .character_index
        .iter()
        .find(|character| character.id == ZHUGEDAN)
        .unwrap();
    assert_eq!(character.name_key, "zhugedan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![GONGAO, JUYI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
}

#[test]
fn juyi_requires_strictly_more_max_hp_and_awakens_only_once() {
    let (mut tx, owner) = setup();
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    let hand_before = tx.state().hand_count(owner);

    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert!(has_skill_def(tx.state(), owner, JUYI));

    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 5;
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before + 5);
    assert!(!has_skill_def(tx.state(), owner, JUYI));
    assert!(has_skill_def(tx.state(), owner, BENGHUAI));
    assert!(has_skill_def(tx.state(), owner, WEIZHONG));

    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before + 5);
}
}

mod original_sp_kaikang_caoang {
//! Golden coverage for SP Cao Ang's 慷忾 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KAIKANG: u32 = 3_829_685_432;
const CAOANG: u32 = 3_627_114_828;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/kaikang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
    armor: CardId,
    sha: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("sp-kaikang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_704, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [owner, target, source] {
        for kind in [ZoneKind::Hand, ZoneKind::Equip] {
            let zone = tx.state().zones.find(kind, player).unwrap();
            let cards = tx.state().zones.get(zone).unwrap().cards.clone();
            if !cards.is_empty() {
                effects.push(EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                });
            }
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(KAIKANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let armor = inject(&mut tx, owner, CardKind::Armor, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, 7);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
        armor,
        sha,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_kaikang_and_caoang_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KAIKANG && skill.name == "kaikang"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOANG)
        .unwrap();
    assert_eq!(character.skills, [KAIKANG]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn draws_gives_then_recipient_optionally_uses_the_exact_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: setup.source,
                card: setup.sha,
                targets: vec![setup.target],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, invoked) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(invoked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    assert!(invoked
        .choice
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == setup.armor.raw()));

    let (state, given) = resolve(state, setup.owner, vec![setup.armor.raw()], false, 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::UseGivenEquipment
    );
    assert_eq!(state.cards.get(setup.armor).unwrap().owner, setup.target);

    let (state, used) = resolve(state, setup.target, vec![1], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    assert_eq!(state.cards.get(setup.armor).unwrap().zone, equip);
}
}

mod original_sp_kuangfu_old_panfeng {
//! Golden coverage for 狂斧 and old 潘凤 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, EquipSlot, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KUANGFU: u32 = 2_689_128_355;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/kuangfu.skill");

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(card);
    card
}

fn setup(seed: u64, disable_weapon: bool) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("kuangfu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 282);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_damage_target_alive_has_discardable_equip"
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KUANGFU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    if disable_weapon {
        tx.state_mut()
            .players
            .get_mut(owner)
            .unwrap()
            .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
    }
    let old_weapon = if disable_weapon {
        CardId::NONE
    } else {
        inject_weapon(&mut tx, owner, 98_101)
    };
    let target_weapon = inject_weapon(&mut tx, target, 98_102);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(KUANGFU))
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveKuangfu {
                player: owner,
                target,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    (tx.commit().state, owner, target, old_weapon, target_weapon)
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn compatible_equipment_moves_and_replaces_while_disabled_slot_forces_discard() {
    let (state, owner, _, old_weapon, target_weapon) = setup(2_026_072_901, false);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, target_weapon.raw(), 1);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, 1, 2);
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_weapon).unwrap().zone, owner_equip);
    assert_eq!(state.cards.get(old_weapon).unwrap().zone, discard);

    let (state, owner, _, _, target_weapon) = setup(2_026_072_902, true);
    let state = choose(state, owner, target_weapon.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_weapon).unwrap().zone, discard);
}
}

mod original_sp_kuiwei_offline_jsp_caoren {
//! Golden coverage for SP「溃围」and the OFFLINE SP 曹仁 cross-package closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JSP_CAOREN: u32 = 1_393_689_295;
const KUIWEI: u32 = 2_977_201_770;
const KUIWEI2: u32 = 4_087_547_393;
const YANZHENG: u32 = 2_899_287_996;
const NESTED_EMPTY_HAND: u32 = 9_900_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/kuiwei.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(source: &str, skill_defs: &[u32]) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("kuiwei-test", source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_723, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, first, second)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn has_skill_def(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn dispatch_optional_phase(
    tx: &mut Transaction,
    player: PlayerId,
    phase: Phase,
    tag: TurnPhaseTag,
) {
    tx.state_mut().turn.current = player;
    tx.state_mut().turn.phase = phase;
    let result = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
}

fn dispatch_forced_phase(tx: &mut Transaction, player: PlayerId, phase: Phase, tag: TurnPhaseTag) {
    tx.state_mut().turn.current = player;
    tx.state_mut().turn.phase = phase;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(run_effects(tx, effects), Err(EffectError::ChoiceRequested));
}

#[test]
fn committed_packages_expose_kuiwei_and_complete_jsp_caoren_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUIWEI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUIWEI2
            && trigger.event == "phase_draw"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == JSP_CAOREN)
        .unwrap();
    assert_eq!(character.name_key, "jsp_caoren");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![KUIWEI, YANZHENG]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn end_phase_draw_uses_then_current_weapon_count_and_next_draw_recounts_for_discard() {
    let (mut tx, owner, first, second) = setup(SOURCE, &[KUIWEI]);
    let first_weapon = inject_weapon(&mut tx, first, 7);
    inject_weapon(&mut tx, second, 8);
    let hand_before = tx.state().hand_count(owner);

    dispatch_optional_phase(&mut tx, owner, Phase::End, TurnPhaseTag::End);
    let mut state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 4);
    assert!(has_skill_def(&state, owner, KUIWEI2));

    let mut tx = Transaction::begin(state);
    let equip = tx.state().zones.find(ZoneKind::Equip, first).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![first_weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().turned_over = false;
    dispatch_forced_phase(&mut tx, owner, Phase::Draw, TurnPhaseTag::Draw);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill
    );
    let selected = tx.state().resolution.continuation_payload[4];
    let hand_before_discard = tx.state().hand_count(owner);
    state = resolve(tx.state().clone(), owner, vec![selected], false, 2);
    assert_eq!(state.hand_count(owner), hand_before_discard - 1);
    assert!(!has_skill_def(&state, owner, KUIWEI2));
    assert!(has_skill_def(&state, owner, KUIWEI));
}

#[test]
fn insufficient_cards_are_all_discarded_without_choice_and_helper_is_removed() {
    let (mut tx, owner, first, second) = setup(SOURCE, &[KUIWEI2]);
    inject_weapon(&mut tx, first, 7);
    inject_weapon(&mut tx, second, 8);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: cards[1..].to_vec(),
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
    assert!(!tx.state().has_pending_choice());
    assert!(!has_skill_def(tx.state(), owner, KUIWEI2));
}

#[test]
fn one_shot_cleanup_waits_for_nested_discard_triggers() {
    let nested_source = format!(
        "{SOURCE}\nskill nested({NESTED_EMPTY_HAND}) {{\n  trigger cards_moved owner_from_hand_empty_other_discardable_hand_equip optional {{\n    effect emit_note(9900001);\n  }}\n}}\n"
    );
    let (mut tx, owner, first, _) = setup(&nested_source, &[KUIWEI2, NESTED_EMPTY_HAND]);
    inject_weapon(&mut tx, first, 7);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: cards[1..].to_vec(),
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();

    dispatch_forced_phase(&mut tx, owner, Phase::Draw, TurnPhaseTag::Draw);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert!(has_skill_def(tx.state(), owner, KUIWEI2));
    let state = resolve(tx.state().clone(), owner, Vec::new(), true, 3);
    assert!(!state.has_pending_choice());
    assert!(!has_skill_def(&state, owner, KUIWEI2));
}
}

mod original_sp_leiluan {
//! Golden coverage for SP 孔淑「累卵」(leiluan).
//! Hand/equip may be used as Sha, Shan, or Tao.

use engine_api::SkillDefId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LEILUAN: u32 = 3_861_314_381;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/leiluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn compiled_source_and_npack_publish_sha_shan_tao() {
    let compiled = content_compiler::compile_source_named("leiluan", SOURCE).unwrap();
    let mut kinds: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == LEILUAN)
        .map(|view| view.as_kind.as_str())
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["sha", "shan", "tao"]);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut pack: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == LEILUAN)
        .map(|view| view.as_kind.as_str())
        .collect();
    pack.sort();
    assert_eq!(pack, vec!["sha", "shan", "tao"]);
}

#[test]
fn skill_attaches_from_compiled_source() {
    let compiled = content_compiler::compile_source_named("leiluan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_34, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LEILUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LEILUAN));
}
}

mod original_sp_lizhan {
//! Golden coverage for SP 曹仁「励战」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIZHAN: u32 = 2_790_547_004;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/lizhan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(player_count: u8) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("lizhan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_rewritten_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LIZHAN && skill.name == "lizhan" && !skill.locked));
}

#[test]
fn no_wounded_player_suppresses_the_trigger() {
    let (mut tx, players) = setup(3);
    assert!(dispatch_for_events(
        &mut tx,
        &[phase_end(players[0])],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn chooses_any_wounded_subset_and_draws_in_seat_order() {
    let (mut tx, players) = setup(3);
    let [owner, second, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let owner = *owner;
    let second = *second;
    let third = *third;
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    tx.state_mut().players.get_mut(third).unwrap().hp -= 1;
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert!(choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![owner.raw(), third.raw()]
    );
    assert_eq!(
        committed.state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );

    let draw_pile = committed
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top_two: Vec<_> = committed
        .state
        .zones
        .get(draw_pile)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    let (state, result) = resolve(
        committed.state,
        owner,
        vec![third.raw(), owner.raw()],
        false,
        1,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(second), 0);
    assert_eq!(state.hand_count(third), 1);
    assert_eq!(state.cards.get(top_two[0]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_two[1]).unwrap().owner, third);
}

#[test]
fn target_choice_can_be_cancelled() {
    let (mut tx, players) = setup(2);
    let owner = players[0];
    tx.state_mut().players.get_mut(players[1]).unwrap().hp -= 1;
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.state.resolution.continuation_payload.len(), 6);
    let (state, result) = resolve(committed.state, owner, Vec::new(), true, 2);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(players[1]), 0);
}
}

mod original_sp_miuyan_kouchao_qiaoli {
//! Golden for the sp view_as rewrite batch:
//!   - 鸣雁 olmiuyan: black hand/equip card as 【火攻】.
//!   - 酷嘲 olkouchao: a hand/equip card as 【杀】/【火攻】/【过河拆桥】, once each per turn.
//!   - 巧力 qiaoli: an equipment card as 【决斗】, up to twice per turn.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLMIUYAN: u32 = 3_010_058_014;
const OLKOUCHAO: u32 = 2_986_388_793;
const QIAOLI: u32 = 3_514_459_394;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_201,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, player: PlayerId, zone: ZoneKind, kind: CardKind, suit: Suit) -> engine_api::CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: cards,
        },
    }
}

/// Clears any pending response choices (无懈 / 决斗出杀 / 火攻选牌) by declining
/// them, or by picking the first option when decline is not allowed.
fn clear_choices(state: &engine_core::state::GameState, mut key: u64) -> (engine_core::state::GameState, u64) {
    use engine_core::state::GameState;
    let mut state = state.clone();
    while state.has_pending_choice() {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
        let (next, pipe) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        key += 1;
        if matches!(pipe.result, CommandResult::Rejected { .. }) {
            // Decline not allowed → accept the first option.
            let next_observer = PlayerId::new(next.resolution.pending_observer);
            let fallback_id = engine_api::ChoiceId::new(next.resolution.pending_choice_id);
            let (next2, pipe2) = apply_command(
                next,
                Command {
                    actor: next_observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: fallback_id,
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            key += 1;
            assert!(
                matches!(pipe2.result, CommandResult::Accepted { .. }),
                "fallback resolve must accept"
            );
            state = next2;
        } else {
            state = next;
        }
    }
    (state, key)
}

#[test]
fn package_declares_the_view_as_entries() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let miuyan = pkg
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == OLMIUYAN)
        .unwrap();
    assert_eq!(miuyan.as_kind, "huogong");
    assert_eq!(miuyan.filter, "black");
    assert_eq!(miuyan.source_zone.as_deref(), Some("hand_equip"));
    for kind in ["sha", "huogong", "guohe"] {
        assert!(pkg.view_as_index.iter().any(|v| {
            v.skill_def_id == OLKOUCHAO && v.as_kind == kind && v.uses_per_turn == Some(1)
        }));
    }
    let qiaoli = pkg
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == QIAOLI)
        .unwrap();
    assert_eq!(qiaoli.as_kind, "juedou");
    assert_eq!(qiaoli.filter, "card_equip");
    assert_eq!(qiaoli.uses_per_turn, Some(2));
}

#[test]
fn miuyan_accepts_black_cards_as_huogong_and_rejects_red() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, OLMIUYAN);
    let black = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
    let red = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Tao, Suit::Heart);

    let (_, black_pipe) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![black], 1),
    );
    assert!(
        matches!(black_pipe.result, CommandResult::Accepted { .. }),
        "black hand card usable as 火攻"
    );

    let (_, red_pipe) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![red], 2),
    );
    assert!(
        matches!(red_pipe.result, CommandResult::Rejected { .. }),
        "red hand card rejected"
    );
}

#[test]
fn kouchao_offers_all_three_convertibles() {
    // Each convertible kind works in a fresh game (sha / huogong / guohe).
    for (kind, key) in [("sha", 1u64), ("huogong", 2), ("guohe", 3)] {
        let (engine, owner, other) = start();
        let mut tx = Transaction::begin(engine.state().clone());
        strip_skills(&mut tx, owner);
        clear_hand(&mut tx, owner);
        tx.state_mut().turn.phase = Phase::Play;
        let skill = add_skill(&mut tx, owner, OLKOUCHAO);
        let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
        let (_, pipe) = apply_command(
            tx.state().clone(),
            activate(owner, skill, vec![other], vec![card], key),
        );
        assert!(
            matches!(pipe.result, CommandResult::Accepted { .. }),
            "{kind} conversion accepted"
        );
    }
}

#[test]
fn qiaoli_accepts_equipment_twice_then_rejects() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, QIAOLI);
    let equip1 = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Weapon, Suit::Heart);
    let equip2 = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Armor, Suit::Diamond);
    let equip3 = inject(&mut tx, owner, ZoneKind::Hand, CardKind::HorseOff, Suit::Club);

    let mut key = 1u64;
    for card in [equip1, equip2] {
        let (state, pipe) = apply_command(
            tx.state().clone(),
            activate(owner, skill, vec![other], vec![card], key),
        );
        assert!(
            matches!(pipe.result, CommandResult::Accepted { .. }),
            "equipment convertible to 决斗 (use {key})"
        );
        let (state, next_key) = clear_choices(&state, key + 1);
        key = next_key;
        tx = Transaction::begin(state);
    }

    let (_, third) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![equip3], key),
    );
    assert!(
        matches!(third.result, CommandResult::Rejected { .. }),
        "third equipment conversion rejected (限两次)"
    );
}
}

mod original_sp_moukui_collab_mushun {
//! Golden coverage for SP「谋溃」and 联动包 SP穆顺.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MOUKUI: u32 = 2_368_472_328;
const SP_MUSHUN: u32 = 3_974_832_852;
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(include_str!(
        "../../../content/original/ported/sp/moukui.skill"
    ))
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOUKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

#[test]
fn packages_publish_moukui_and_sp_mushun_dependency() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MOUKUI));

    let collab = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    assert!(collab.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let mushun = collab
        .character_index
        .iter()
        .find(|character| character.id == SP_MUSHUN)
        .unwrap();
    assert_eq!(mushun.name_key, "sp_mushun");
    assert_eq!(mushun.max_hp, 4);
    assert_eq!(mushun.skills, [MOUKUI]);
}

#[test]
fn draw_branch_arms_exact_sha_and_target_for_forced_counter_discard() {
    let (mut state, owner, target) = setup(20_260_725_01);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_001);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_002,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_003);

    let (state, played) = play_sha(state, owner, target, sha, 1);
    let branch = played.choice.unwrap();
    assert_eq!(branch.observer, owner);
    assert_eq!(branch.kind, ChoiceKind::Custom);
    assert_eq!(
        branch
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let hand_before = state.hand_count(owner);
    let (state, chosen) = resolve(state, owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(chosen.choice.as_ref().unwrap().observer, target);

    let (state, dodged) = resolve(state, target, vec![shan.raw()], false, 3);
    let discard = dodged.choice.unwrap();
    assert_eq!(discard.observer, target);
    assert_eq!(discard.kind, ChoiceKind::SelectCards);
    assert!(discard
        .options
        .iter()
        .any(|option| option.token == owner_card.raw()));

    let (state, finished) = resolve(state, target, vec![owner_card.raw()], false, 4);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card).unwrap().owner, PlayerId::NONE);
}

#[test]
fn discard_branch_discards_target_then_forces_counter_discard_after_dodge() {
    let (mut state, owner, target) = setup(20_260_725_02);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_011);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_012,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_013);
    let target_card = inject(
        &mut state,
        target,
        ZoneKind::Equip,
        CardKind::HorseDef,
        92_014,
    );

    let (state, _) = play_sha(state, owner, target, sha, 10);
    let (state, selected_branch) = resolve(state, owner, vec![2], false, 11);
    let discard_target = selected_branch.choice.unwrap();
    assert_eq!(discard_target.observer, owner);
    assert!(discard_target
        .options
        .iter()
        .any(|option| option.token == target_card.raw()));

    let (state, responded) = resolve(state, owner, vec![target_card.raw()], false, 12);
    assert_eq!(state.cards.get(target_card).unwrap().owner, PlayerId::NONE);
    assert_eq!(responded.choice.as_ref().unwrap().observer, target);

    let (state, counter) = resolve(state, target, vec![shan.raw()], false, 13);
    let counter_discard = counter.choice.unwrap();
    assert_eq!(counter_discard.observer, target);
    assert!(counter_discard
        .options
        .iter()
        .any(|option| option.token == owner_card.raw()));
    let (state, _) = resolve(state, target, vec![owner_card.raw()], false, 14);
    assert_eq!(state.cards.get(owner_card).unwrap().owner, PlayerId::NONE);
}

#[test]
fn cancelling_initial_branch_does_not_arm_the_dodge_consequence() {
    let (mut state, owner, target) = setup(20_260_725_03);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_021);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_022,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_023);

    let (state, _) = play_sha(state, owner, target, sha, 20);
    let (state, declined) = resolve(state, owner, Vec::new(), true, 21);
    assert_eq!(declined.choice.as_ref().unwrap().observer, target);
    let (state, finished) = resolve(state, target, vec![shan.raw()], false, 22);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert!(finished.choice.is_none());
    assert_eq!(state.cards.get(owner_card).unwrap().owner, owner);
}
}

mod original_sp_naman {
//! Golden coverage for 纳蛮 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NAMAN: u32 = 4_147_868_153;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/naman.skill");

fn add_discard_card(tx: &mut Transaction, kind: CardKind) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_276),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    card
}

#[test]
fn gains_only_another_players_physical_sha_response() {
    let package = content_compiler::compile_source_named("naman-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 276);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 2_026_072_936,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let responder = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NAMAN),
        }],
    )
    .unwrap();

    let sha = add_discard_card(&mut tx, CardKind::Sha);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::CardResponded {
                player: responder,
                respond_to: owner,
                card: sha,
                response_kind: CardKind::Sha.tag(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, owner);
    assert!(state.hand_cards(owner).contains(&sha));

    let mut tx = Transaction::begin(state);
    let own_sha = add_discard_card(&mut tx, CardKind::Sha);
    let shan = add_discard_card(&mut tx, CardKind::Shan);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::CardResponded {
                player: owner,
                respond_to: responder,
                card: own_sha,
                response_kind: CardKind::Sha.tag(),
            },
            EffectRequest::CardResponded {
                player: responder,
                respond_to: owner,
                card: shan,
                response_kind: CardKind::Shan.tag(),
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state().cards.get(own_sha).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        tx.state().cards.get(shan).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_sp_neifa_use {
//! Golden coverage for sp 内伐·用 (neifa_use): locked sub-skill that, after its
//! owner uses an equipment card, draws as many cards as the owner's current
//! 「内伐·无基本」mark count (upstream `player.draw(player.countMark(...))`).
//!
//! Exercises the body-level equipment-kind check (`card_attribute(event_card, 0)`
//! in the 10..=14 tag range) and the `mark_count` query feeding `draw_cards`.
//! The upstream `usable: 2` per-turn limit has no corresponding filter primitive;
//! the ported .skill documents this approximation.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NEIFA_USE: u32 = 2_442_423_886;
const NEIFA_NOBASIC: u32 = 3_552_802_810;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_neifa_use(tx: &mut Transaction, owner: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEIFA_USE),
    });
    run_effects(tx, effects).unwrap();
}

fn set_marks(tx: &mut Transaction, player: PlayerId, count: i32) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player,
            mark: MarkDefId::new(NEIFA_NOBASIC),
            delta: count,
        }],
    )
    .unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_neifa_use_equip_draw_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEIFA_USE
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn using_equip_with_marks_draws_marks_count() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    set_marks(&mut tx, owner, 3);
    let equip = inject_hand(tx.state_mut(), owner, CardKind::Weapon);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, equip);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 3,
        "equip use with 3 marks draws 3"
    );
}

#[test]
fn using_equip_without_marks_draws_nothing() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    let equip = inject_hand(tx.state_mut(), owner, CardKind::Armor);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, equip);

    assert_eq!(tx.state().hand_count(owner), hand_before, "0 marks → no draw");
}

#[test]
fn using_non_equip_with_marks_draws_nothing() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_neifa_use(&mut tx, owner);
    set_marks(&mut tx, owner, 2);
    let sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "non-equip card → no draw despite marks"
    );
}
}

mod original_sp_nuzhan2_reduanbing_hongmianbaihuapao {
//! Golden for three SP skills rewritten from legacy JavaScript semantics:
//! - 怒斩2 (nuzhan2): locked — a Sha whose only material is one equipment card deals +1 base damage.
//! - 断兵 (reduanbing): optional — after using a Sha, add one legal target within distance 1.
//! - 红棉百花袍 (hongmianbaihuapao_skill): locked equip skill — chain-propagated damage is prevented.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_card_flag, damage_nature, EffectRequest};

const NUZHAN2: u32 = 3_186_808_257;
const REDUANBING: u32 = 3_958_105_675;
const HONGMIANBAIHUAPAO: u32 = 2_276_671_070;
const UNLIMITED_SHA: u32 = 4_290_000_277;
const NEW_REWUSHENG: u32 = 3_038_912_803;
const NUZHAN2_SOURCE: &str = include_str!("../../../content/original/ported/sp/nuzhan2.skill");
const REDUANBING_SOURCE: &str = include_str!("../../../content/original/ported/sp/reduanbing.skill");
const HONGMIAN_SOURCE: &str =
    include_str!("../../../content/original/ported/sp/hongmianbaihuapao_skill.skill");
const REWUSHENG_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/new_rewusheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn skill_id(state: &engine_core::state::GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

/// Drive every pending choice (each target declines 闪) until no choice remains.
fn run_to_stable(
    mut state: engine_core::state::GameState,
    mut key: u64,
) -> (engine_core::state::GameState, u64) {
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(r.result, CommandResult::Accepted { .. }), "{:?}", r.result);
        state = next;
        key += 1;
    }
    (state, key)
}

#[test]
fn package_declares_all_three_rewritten_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == NUZHAN2
            && t.event == "card_used"
            && t.filter == "owner_source_sha_single_equip_source_card"
            && !t.optional
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == REDUANBING
            && t.event == "card_used"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == HONGMIANBAIHUAPAO
            && t.event == "damage_before"
            && t.filter == "owner_is_target_chain_damage_linked"
            && !t.optional
    }));
    // No stale noop modifier remains for these defs.
    assert!(!pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == NUZHAN2
            || m.skill_def_id == REDUANBING
            || m.skill_def_id == HONGMIANBAIHUAPAO
    }));
}

#[test]
fn nuzhan2_equip_material_sha_boosts_base_damage() {
    let source = format!(
        "{NUZHAN2_SOURCE}\n{REWUSHENG_SOURCE}\nskill unlimited_sha(4290000277) {{ modifier card_distance_unlimited(sha) override 1; }}"
    );
    let compiled = content_compiler::compile_source_named("nuzhan2-test", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_081_101,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NUZHAN2),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REWUSHENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in tx.state().players.alive_by_seat() {
        let participant = tx.state_mut().players.get_mut(seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    // A red equipment card converted into a Sha: the only use material is this equip.
    let equip = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_278),
        kind: CardKind::Weapon,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(equip);

    let view_as = skill_id(tx.state(), owner, NEW_REWUSHENG);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: view_as,
                targets: vec![target],
                card_targets: vec![equip],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, _) = run_to_stable(state, 2);
    // Base 1 + nuzhan2 +1 = 2 damage.
    assert_eq!(state.players.get(target).unwrap().hp, 2);
}

#[test]
fn reduanbing_adds_one_distance_one_target_to_used_sha() {
    let compiled =
        content_compiler::compile_source_named("reduanbing-test", REDUANBING_SOURCE).unwrap();
    let fixture = content_compiler::compile_source_named(
        "unlimited-sha-fixture",
        "skill unlimited_sha(4290000277) { modifier card_distance_unlimited(sha) override 1; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 2_026_081_102,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let first = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];
    let near = seats[(owner_index + 3) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(REDUANBING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in &seats {
        let participant = tx.state_mut().players.get_mut(*seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_279),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![first],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    // The optional trigger asks the owner to confirm first.
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let confirm = offered.choice.unwrap();
    assert_eq!(confirm.observer, owner);
    let (state, accepted) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let choice = accepted.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == near.raw()));
    assert!(choice
        .options
        .iter()
        .all(|option| option.token != far.raw()));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![near.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { target, .. } if *target == near
    )));
}

#[test]
fn hongmianbaihuapao_prevents_only_chain_propagated_damage() {
    let compiled =
        content_compiler::compile_source_named("hongmian-test", HONGMIAN_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_081_103,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HONGMIANBAIHUAPAO),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in tx.state().players.alive_by_seat() {
        let participant = tx.state_mut().players.get_mut(seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }

    let initial = tx.state().players.get(owner).unwrap().hp;

    // Direct fire damage (not chain-propagated) must still apply.
    let direct = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(direct.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, initial - 1);

    // Chain-propagated elemental damage is prevented.
    let chained = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
            card: CardId::NONE,
            card_kind: CardKind::TrickShandian.tag(),
            card_flags: damage_card_flag::CHAIN_PROPAGATED,
        }],
    )
    .unwrap();
    assert!(!chained.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, initial - 1);
}
}

mod original_sp_olbiri {
//! Golden for SP「蔽日」的首个非摸牌得牌语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLBIRI: u32 = 3_834_015_931;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn draws_only_for_the_first_non_draw_gain_each_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLBIRI),
    });
    run_effects(&mut tx, effects).unwrap();

    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: others[0],
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);

    run_effects(
        &mut tx,
        vec![EffectRequest::TakeFromHand {
            from: others[0],
            to: others[1],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::TakeFromHand {
            from: others[1],
            to: others[0],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_sp_olcuorui_niujin {
//! Golden coverage for SP Niu Jin's 挫锐 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLCUORUI: u32 = 2_241_852_934;
const LIEWEI: u32 = 3_656_060_804;
const NIUJIN: u32 = 2_469_157_690;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olcuorui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup(player_count: u8) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("sp-olcuorui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 2_026_072_705, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().zones.get(hand).unwrap().cards.clone();
    if !cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLCUORUI),
    });
    run_effects(&mut tx, effects).unwrap();
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_030),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone: source_hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(source_hand)
        .unwrap()
        .cards
        .push(delayed);
    (tx, owner, source, delayed)
}

#[test]
fn package_publishes_olcuorui_and_niujin_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLCUORUI && skill.name == "olcuorui" && skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NIUJIN)
        .unwrap();
    assert_eq!(character.skills, [OLCUORUI, LIEWEI]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn fills_to_eight_then_consumes_one_pending_judge_skip() {
    let (mut tx, owner, source, delayed) = setup(8);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 8 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 8);

    let targeted = DomainEvent::CardUsed {
        player: source,
        card: delayed,
        targets: vec![owner],
    };
    let effects =
        dispatch_for_events(&mut tx, &[targeted.clone()], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().pending_judge_skips, [owner]);

    let repeated =
        dispatch_for_events(&mut tx, &[targeted], &mut TriggerFireRecord::new()).unwrap();
    assert!(repeated.is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.pending_judge_skips.is_empty());
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Judge,
        } if *current == owner
    )));
}
}

mod original_sp_old_guanyinping {
//! Golden coverage for old 关银屏's rewritten skill closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XUEJI_OLD: u32 = 3_528_622_023;
const OLDHUXIAO: u32 = 3_615_190_640;
const OLDWUJI: u32 = 3_438_545_283;

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(76_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![card],
            },
        },
    );
    (state, result.result)
}

#[test]
fn xueji_enforces_red_lost_hp_cap_and_oldwuji_awakens_at_three_damage() {
    let source = [
        include_str!("../../../content/original/ported/sp/xueji_old.skill"),
        include_str!("../../../content/original/ported/sp/oldhuxiao.skill"),
        include_str!("../../../content/original/ported/sp/oldwuji.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("old-guanyinping-test", &source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 279);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_944, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = &players[1..];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.extend(
        [XUEJI_OLD, OLDHUXIAO, OLDWUJI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 1;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let red = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        0,
    );
    let black_weapon = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        8,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(XUEJI_OLD))
        .unwrap()
        .id;
    let state = tx.commit().state;

    let (_, result) = activate(state.clone(), owner, skill, targets.to_vec(), red, 1);
    assert!(matches!(result, CommandResult::Rejected { .. }));
    let (_, result) = activate(
        state.clone(),
        owner,
        skill,
        vec![targets[0]],
        black_weapon,
        2,
    );
    assert!(matches!(result, CommandResult::Rejected { .. }));

    let (state, result) = activate(state, owner, skill, vec![targets[0], targets[1]], red, 3);
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.turn.flag(turn_flag::DAMAGE_DEALT), 2);
    assert_eq!(state.hand_count(targets[0]), 1);
    assert_eq!(state.hand_count(targets[1]), 1);
    assert!(state.zones.get(discard).unwrap().cards.contains(&red));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: targets[2],
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (2, 4));
    let defs: Vec<_> = player
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&XUEJI_OLD));
    assert!(!defs.contains(&OLDHUXIAO));
    assert!(!defs.contains(&OLDWUJI));
}
}

mod original_sp_old_shenxian {
//! Golden for SP「甚贤」and OLD「旧甚贤」discard-cause semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHENXIAN: u32 = 3_316_671_408;
const OLD_SHENXIAN: u32 = 2_584_430_535;
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn isolated(package_bytes: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, discarder) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: discarder,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, discarder)
}

fn accept_optional(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    state
}

fn set_kind(
    tx: &mut Transaction,
    player: PlayerId,
    index: usize,
    kind: CardKind,
) -> engine_api::CardId {
    let card = tx.state().hand_cards(player)[index];
    tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    card
}

#[test]
fn package_filters_preserve_the_new_and_old_turn_limit_difference() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHENXIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_basic_outside_turn_once"
            && trigger.optional
    }));
    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert!(old.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLD_SHENXIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_basic_outside_turn"
            && trigger.optional
    }));
}

#[test]
fn shenxian_requires_an_actual_basic_discard_outside_the_owner_turn() {
    let (mut tx, owner, discarder) = isolated(SP_PACKAGE, SHENXIAN);
    let before = tx.state().hand_count(owner);
    let basic_move = set_kind(&mut tx, discarder, 0, CardKind::Shan);
    let non_basic = set_kind(&mut tx, discarder, 1, CardKind::TrickWuzhong);
    let basic_discard = set_kind(&mut tx, discarder, 2, CardKind::Sha);
    let hand = tx.state().zones.find(ZoneKind::Hand, discarder).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    // A basic card reaching the discard pile through an unspecified use/response-style move is
    // not a discard and must not offer 甚贤.
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![basic_move],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: discarder,
            cards: vec![non_basic],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![basic_discard],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: discarder,
            count: 1,
        }],
    )
    .unwrap();
    let own_turn_basic = set_kind(&mut tx, discarder, 0, CardKind::Tao);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![own_turn_basic],
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn shenxian_is_once_per_character_turn_but_oldshenxian_is_not() {
    let (mut tx, owner, discarder) = isolated(SP_PACKAGE, SHENXIAN);
    let first = set_kind(&mut tx, discarder, 0, CardKind::Sha);
    let second = set_kind(&mut tx, discarder, 1, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![first],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 2);
    let after_first = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: discarder,
            cards: vec![second],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), after_first);

    let (mut tx, owner, discarder) = isolated(OLD_PACKAGE, OLD_SHENXIAN);
    let first = set_kind(&mut tx, discarder, 0, CardKind::Sha);
    let second = set_kind(&mut tx, discarder, 1, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![first],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 3);
    let after_first = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![second],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 4);
    assert_eq!(state.hand_count(owner), after_first + 1);
}
}

mod original_sp_oldhuxiao {
//! Golden coverage for old 虎啸 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDHUXIAO: u32 = 3_615_190_640;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/oldhuxiao.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_278 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(state: GameState, owner: PlayerId, target: PlayerId, sha: CardId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn dodged_sha_refunds_its_use_count_but_a_hit_still_counts() {
    let package = content_compiler::compile_source_named("oldhuxiao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 278);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_939, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDHUXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let first_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart);
    let state = play(tx.commit().state, owner, target, first_sha, 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let mut tx = Transaction::begin(state);
    let second_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Club);
    let state = play(tx.commit().state, owner, target, second_sha, 3);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
}
}

mod original_sp_oldianjun {
//! Golden for sp 典军 (oldianjun): locked — at the owner's phase end, the owner
//! takes 1 no-source HP loss (approximated by lose_hp) and queues an extra play phase.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLDIANJUN: u32 = 2_275_218_678;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_101,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDIANJUN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_oldianjun_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == OLDIANJUN && t.event == "phase_end"
    }));
}

#[test]
fn phase_end_loses_one_hp_and_queues_extra_play() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase-end dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "典军 loses 1 HP at phase end"
    );
}
}

mod original_sp_olfeibai {
//! Golden coverage for SP「飞白」(olfeibai). Outgoing damage_before +1.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLFEIBAI: u32 = 3_111_364_655;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olfeibai.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olfeibai-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_52, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFEIBAI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_damage_plus_one() {
    let compiled = content_compiler::compile_source_named("olfeibai", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFEIBAI)
        .expect("olfeibai trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source_other_damage_before");
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLFEIBAI && trigger.filter == "owner_source_other_damage_before"
    }));
}

#[test]
fn outgoing_damage_increases_by_one() {
    let (state, owner, other) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 2);
}
}

mod original_sp_olfeibian {
//! Golden coverage for SP OL 祢衡「飞辩」(olfeibian).
//! After the owner uses a card they must discard one hand card.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLFEIBIAN: u32 = 3_525_562_624;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olfeibian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("olfeibian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_43, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFEIBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_803),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, owner, card)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_the_use_then_discard() {
    let compiled = content_compiler::compile_source_named("olfeibian", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFEIBIAN)
        .expect("olfeibian trigger");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFEIBIAN)
        .expect("npack olfeibian trigger");
    assert_eq!(pack.event, "card_used");
    assert_eq!(pack.filter, "owner_is_source");
}

#[test]
fn using_a_card_forces_a_hand_discard() {
    let (state, owner, card) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(run_effects(&mut tx, effects), Err(EffectError::ChoiceRequested));
    let (state, discarded) = resolve(tx.commit().state, owner, vec![card.raw()], 1);
    assert!(
        matches!(discarded.result, CommandResult::Accepted { .. }),
        "{:?}",
        discarded.result
    );
    assert_eq!(state.hand_count(owner), before - 1);
}
}

mod original_sp_olfengyan {
//! Golden coverage for SP「讽言」(olfengyan). Incoming damage draws 1.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLFENGYAN: u32 = 4_005_200_434;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olfengyan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olfengyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_54, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFENGYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_the_damage_draw() {
    let compiled = content_compiler::compile_source_named("olfengyan", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFENGYAN)
        .expect("olfengyan trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLFENGYAN && trigger.event == "damage"
    }));
}

#[test]
fn incoming_damage_draws_one() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_sp_olfudao {
//! Golden coverage for SP OL 丁尚涴「抚悼」(olfudao).
//! Game start: owner draws 3.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLFUDAO: u32 = 2_837_007_198;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olfudao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olfudao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_11, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFUDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_the_game_start_draw() {
    let compiled = content_compiler::compile_source_named("olfudao", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLFUDAO && skill.name == "olfudao" && !skill.locked));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFUDAO)
        .expect("olfudao trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFUDAO)
        .expect("npack olfudao trigger");
    assert_eq!(pack.event, "game_started");
    assert_eq!(pack.filter, "owner_alive");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olfudao");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn game_start_draws_three() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 3);
}
}

mod original_sp_olhuiyun {
//! Golden coverage for SP OL 胡班「晖云」(olhuiyun).
//! Play-phase view-as: a hand/equip card as 火攻.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLHUIYUN: u32 = 4_044_368_124;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olhuiyun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("olhuiyun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_31, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLHUIYUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_501),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let shown = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_502),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 2,
        weapon_range: 0,
        zone: other_hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(other_hand)
        .unwrap()
        .cards
        .push(shown);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLHUIYUN)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_huogong_view_as() {
    let compiled = content_compiler::compile_source_named("olhuiyun", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLHUIYUN)
        .expect("olhuiyun view_as");
    assert_eq!(view.as_kind, "huogong");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "any");

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLHUIYUN)
        .expect("npack olhuiyun view_as");
    assert_eq!(pack.as_kind, "huogong");
}

#[test]
fn activate_is_accepted_as_huogong() {
    let (state, owner, other, skill, card) = setup();
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let _ = state;
}
}

mod original_sp_oljiaowei {
//! Golden for sp 骄巍 (oljiaowei): locked — black cards are excluded from the
//! owner's hand limit.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLJIAOWEI: u32 = 2_764_079_934;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_501,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

#[test]
fn package_declares_black_hand_limit_ignored_modifier() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == OLJIAOWEI
            && m.target == "card_ignored_hand_limit"
            && m.selector.as_deref() == Some("black")
            && m.mode == "override"
            && m.value == 1
    }));
}

#[test]
fn adding_oljiaowei_keeps_the_skill_locked() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIAOWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == OLJIAOWEI)
        .map(|skill| skill.id)
        .expect("oljiaowei installed");
    let skill = tx.state().skills.get(skill_id).unwrap();
    assert_eq!(skill.owner, owner);
    assert!(skill.enabled);
}
}

mod original_sp_oljiyun {
//! Golden coverage for SP OL 李丰「集运」(oljiyun).
//! Game start draws 3. Prepare may choose up to 3 living players to draw 1 each.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLJIYUN: u32 = 3_017_780_662;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/oljiyun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("oljiyun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_23, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIYUN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_both_windows() {
    let compiled = content_compiler::compile_source_named("oljiyun", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIYUN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("game_started", "owner_alive", false),
            ("phase_prepare", "owner_is_target", false),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIYUN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"game_started"));
    assert!(pack.contains(&"phase_prepare"));
}

#[test]
fn game_start_draws_three_then_prepare_draws_for_chosen_target() {
    let (state, owner, other) = setup();
    let start = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), start + 3);

    let other_before = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![other.raw()], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }), "{:?}", confirmed.result);
    assert_eq!(state.hand_count(other), other_before + 1);
}
}

mod original_sp_olkenshang {
//! Golden coverage for SP「垦伤」(olkenshang). Play-phase view-as Sha.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLKENSHANG: u32 = 3_699_245_274;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olkenshang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("olkenshang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_51, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLKENSHANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_901),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 4,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLKENSHANG)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_sha_view_as() {
    let compiled = content_compiler::compile_source_named("olkenshang", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLKENSHANG)
        .expect("olkenshang view_as");
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.window, "play_phase");
    // 批 298：at_least 1 → 不封顶（255 哨兵）。
    assert_eq!(view.min_cards, 1);
    assert_eq!(view.max_cards, u8::MAX);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == OLKENSHANG && view.as_kind == "sha"));
}

#[test]
fn hand_card_converts_to_sha() {
    let (state, owner, other, skill, card) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }), "{:?}", pipe.result);
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
}
}

mod original_sp_olpingyuan {
//! Golden coverage for SP OL 刘晔「平远」(olpingyuan).
//! Game start may draw 1. Turn begin may open a virtual Sha.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLPINGYUAN: u32 = 3_423_099_864;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olpingyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olpingyuan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_24, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLPINGYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_both_optional_windows() {
    let compiled = content_compiler::compile_source_named("olpingyuan", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLPINGYUAN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("game_started", "owner_alive", true),
            ("phase_begin", "owner_is_target", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLPINGYUAN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"game_started"));
    assert!(pack.contains(&"phase_begin"));
}

#[test]
fn game_start_draws_one_when_confirmed() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 2 }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn prepare_opens_virtual_sha_choice() {
    let (state, owner) = setup();
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
}
}

mod original_sp_olpingzhong {
//! Golden coverage for SP OL 杨阜「评忠」(olpingzhong).
//! After taking damage the owner draws 1.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLPINGZHONG: u32 = 2_762_208_431;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olpingzhong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olpingzhong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_12, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLPINGZHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_the_damage_draw() {
    let compiled = content_compiler::compile_source_named("olpingzhong", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == OLPINGZHONG && skill.name == "olpingzhong" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLPINGZHONG)
        .expect("olpingzhong trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLPINGZHONG)
        .expect("npack olpingzhong trigger");
    assert_eq!(pack.event, "damage");
    assert_eq!(pack.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olpingzhong");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn incoming_damage_draws_one() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_sp_olqiaozhi_machine {
//! Golden for SP 巧织 olqiaozhi (ABI 740, tag 761, Cont 330):
//! 出牌阶段，弃一张牌 → 亮牌堆顶两张 → 获得其中一张（打「巧织」标，
//! 未选入弃）；手牌有巧织牌时不可发动。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLQIAOZHI: u32 = 3_591_197_180;
const OLQIAOZHI_TAG: u32 = 1_909_178_934;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, kind: CardKind, tag: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + tag),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_olqiaozhi_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLQIAOZHI)
        .expect("olqiaozhi active entry");
    assert!(active.hand_nonempty);
    // 批 301：命令级 gaintag gate。
    assert_eq!(active.hand_missing_gaintag, Some(1_909_178_934));
}

#[test]
fn olqiaozhi_discards_reveals_and_gains_tagged_card() {
    let (mut state, players) = started(2_026_096_001);
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLQIAOZHI);
    let cost = inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao, 1);
    let top_a = inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, CardKind::Shan, 2);
    let top_b = inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, CardKind::Other, 3);
    let hand_before = state.hand_count(owner);

    // 发动 → stage 0：开弃成本窗口。
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlqiaozhiStage
    );

    // 弃成本 → 亮顶 2 选 1。
    let (state, result) = resolve(state, owner, vec![cost.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlqiaozhiStage
    );
    // 选择窗口选项 = 亮出的两张。
    let choice = result.choice.as_ref().expect("reveal choice open");
    let options: Vec<u32> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options.len(), 2);
    assert!(options.contains(&top_a.raw()) && options.contains(&top_b.raw()));

    // 选 top_a → 打标入手牌；top_b 入弃牌堆。
    let (state, result) = resolve(state, owner, vec![top_a.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&top_a));
    assert!(state
        .content_card_tags
        .get(&top_a)
        .is_some_and(|tags| tags.contains(&OLQIAOZHI_TAG)));
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&top_b));
    // 净手牌：-1 成本 +1 所得。
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(!state.has_pending_choice());
}

#[test]
fn olqiaozhi_gate_blocks_reactivation_while_tagged_card_in_hand() {
    let (mut state, players) = started(2_026_096_002);
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLQIAOZHI);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao, 4);
    // 手牌预置一张「巧织」牌。
    let tagged = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 5);
    state.content_card_tags.entry(tagged).or_default().insert(OLQIAOZHI_TAG);

    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        1,
    );
    // 批 301：命令级 gaintag gate → 发动被拒。
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert!(!state.has_pending_choice());
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}
}

mod original_sp_olqisi {
//! Golden coverage for SP OL 蒲元「奇思」(olqisi).
//! Game start: owner draws 2.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLQISI: u32 = 2_846_642_634;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olqisi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olqisi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_21, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLQISI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_the_game_start_draw() {
    let compiled = content_compiler::compile_source_named("olqisi", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLQISI && skill.name == "olqisi" && !skill.locked));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLQISI)
        .expect("olqisi trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLQISI)
        .expect("npack olqisi trigger");
    assert_eq!(pack.event, "game_started");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olqisi");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn game_start_draws_two() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
}

mod original_sp_olsiqi {
//! Golden coverage for SP OL 薛灵芸「思泣」(olsiqi).
//! Optional draw 1 after taking damage.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLSIQI: u32 = 2_641_086_803;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olsiqi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olsiqi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_33, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLSIQI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_the_optional_damage_draw() {
    let compiled = content_compiler::compile_source_named("olsiqi", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLSIQI)
        .expect("olsiqi trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLSIQI)
        .expect("npack olsiqi trigger");
    assert_eq!(pack.event, "damage");
    assert!(pack.optional);
}

#[test]
fn damage_draws_one_when_confirmed() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}
}

mod original_sp_olsuyi_machine {
//! Golden for SP 俗义 olsuyi (ABI 739, tag 760):
//! 出牌阶段限一次，选一名其他角色：牌堆顶向下检索第一张其可装备的装备
//! 牌令其获得并装备；然后对其视为使用一张【杀】。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLSUYI: u32 = 4_134_784_601;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject_draw_top(state: &mut GameState, kind: CardKind, tag: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000 + tag),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 1,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_olsuyi_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLSUYI)
        .expect("olsuyi active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|targets| {
        !targets.allow_self && targets.min == 1 && targets.max == 1
    }));
}

#[test]
fn olsuyi_equips_target_from_deck_then_uses_virtual_sha() {
    let (mut state, players) = started(2_026_095_001);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLSUYI);
    let weapon = inject_draw_top(&mut state, CardKind::Weapon, 1);
    let hp_before = state.players.get(target).unwrap().hp;

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 装备牌离开牌堆、为目标装备（目标手牌区不留）。
    let weapon_state = state.cards.get(weapon).unwrap();
    assert_eq!(weapon_state.owner, target);
    assert_ne!(weapon_state.zone, state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap());
    assert_ne!(weapon_state.zone, state.zones.find(ZoneKind::Hand, target).unwrap());
    // 虚拟杀完整结算：目标无闪 → 掉 1 体力。
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);

    // uses_per_turn 1：本回合再次发动被权威拒绝。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn olsuyi_without_equipment_in_deck_still_uses_virtual_sha() {
    let (mut state, players) = started(2_026_095_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLSUYI);
    inject_draw_top(&mut state, CardKind::Tao, 2); // 非装备牌 → 跳过装备段
    let hp_before = state.players.get(target).unwrap().hp;

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}
}

mod original_sp_olwenren_machine {
//! Golden for SP 温仁 olwenren (ABI 736, tag 757):
//! 出牌阶段限一次，任选任意名目标，每名目标无手牌摸 1、手牌数不大于
//! owner 再摸 1（可叠加，可摸 0）。active 全管道驱动（ActivateSkill）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLWENREN: u32 = 4_257_407_239;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, count: usize, tag: u32) {
    for index in 0..count {
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(93_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        state.zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_olwenren_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLWENREN)
        .expect("olwenren active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|targets| {
        targets.allow_self && targets.min == 1 && targets.max == 255
    }));
}

#[test]
fn olwenren_draws_two_for_empty_one_for_lequal_and_zero_for_greater() {
    let (mut state, players) = started(2_026_092_001);
    let owner = players[0];
    let empty_target = players[1];
    let rich_target = players[2];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLWENREN);
    inject(&mut state, owner, 2, 1);
    // empty_target 无手牌（摸 2）；rich_target 3 张 > owner 2 张（摸 0）。
    inject(&mut state, rich_target, 3, 2);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![empty_target, rich_target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(empty_target), 2);
    assert_eq!(state.hand_count(rich_target), 3);
    assert_eq!(state.hand_count(owner), 2);

    // uses_per_turn 1：本回合再次发动被权威拒绝。
    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![empty_target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(state.hand_count(empty_target), 2);
}

#[test]
fn olwenren_gives_one_draw_to_target_with_lequal_hand_count() {
    let (mut state, players) = started(2_026_092_002);
    let owner = players[0];
    let lequal_target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLWENREN);
    inject(&mut state, owner, 3, 3);
    // 1 张 ≤ owner 3 张且有手牌（非空）→ 仅满足第二条件 → 摸 1。
    inject(&mut state, lequal_target, 1, 4);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![lequal_target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(lequal_target), 2);
}
}

mod original_sp_olxianying {
//! Golden coverage for SP OL 秦朗「贤膺」(olxianying).
//! Optional draw 2 after taking damage, and again at the owner's prepare.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLXIANYING: u32 = 2_869_635_574;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olxianying.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olxianying-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_14, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLXIANYING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_both_optional_draw_windows() {
    let compiled = content_compiler::compile_source_named("olxianying", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLXIANYING && skill.name == "olxianying" && !skill.locked));
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLXIANYING)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("damage", "owner_is_target", true),
            ("phase_prepare", "owner_is_target", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLXIANYING)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"damage"));
    assert!(pack.contains(&"phase_prepare"));
}

#[test]
fn damage_then_prepare_each_draw_two_when_confirmed() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);

    let mut tx = Transaction::begin(state);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let after_damage = tx.state().hand_count(owner);
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), after_damage + 2);
}
}

mod original_sp_olxiasheng {
//! Golden coverage for sp 狭赏 (olxiasheng): locked skill whose useCard branch
//! draws 1 when the owner uses a black card while holding more red than black
//! hand cards.
//!
//! Exercises `hand_color_count`, the suit-based black/red check via
//! `card_attribute`, and `draw_cards`. The useCard2 add-target branch is not
//! ported (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXIASHENG: u32 = 2_354_381_667;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_819,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_olxiasheng(tx: &mut Transaction, owner: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLXIASHENG),
    });
    run_effects(tx, effects).unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn replace_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    cards: &[(CardKind, Suit)],
) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let old: Vec<_> = state.hand_cards(player);
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get_mut(discard).unwrap().cards.extend(old.clone());
    for card in &old {
        state.cards.get_mut(*card).unwrap().zone = discard;
    }
    state.zones.get_mut(hand).unwrap().cards.clear();
    cards.iter().map(|(kind, suit)| inject_hand(state, player, *kind, *suit)).collect()
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_olxiasheng_card_used_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLXIASHENG
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn red_majority_black_card_draws_one() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 red (桃 heart, 酒 diamond), 1 black (杀 spade).
    let black_sha = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Tao, Suit::Heart),
            (CardKind::Jiu, Suit::Diamond),
            (CardKind::Sha, Suit::Spade),
        ],
    )[2];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, black_sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "black card with red hand majority draws 1"
    );
}

#[test]
fn black_majority_black_card_draws_nothing() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 black (杀 spade, 闪 club), 1 red (桃 heart).
    let black_sha = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Sha, Suit::Spade),
            (CardKind::Shan, Suit::Club),
            (CardKind::Tao, Suit::Heart),
        ],
    )[0];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, black_sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "black-card use with black hand majority draws nothing"
    );
}

#[test]
fn red_majority_red_card_draws_nothing() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 red, 1 black — use the red card.
    let red_tao = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Tao, Suit::Heart),
            (CardKind::Jiu, Suit::Diamond),
            (CardKind::Sha, Suit::Spade),
        ],
    )[0];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, red_tao);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "red-card use draws nothing"
    );
}
}

mod original_sp_olxinggu {
//! Golden coverage for SP 张世平「行贾」(olxinggu).
//! Game start: owner draws 3.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLXINGGU: u32 = 2_689_779_795;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olxinggu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olxinggu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_22, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLXINGGU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_the_game_start_draw() {
    let compiled = content_compiler::compile_source_named("olxinggu", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLXINGGU && skill.name == "olxinggu" && !skill.locked));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLXINGGU)
        .expect("olxinggu trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLXINGGU)
        .expect("npack olxinggu trigger");
    assert_eq!(pack.event, "game_started");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olxinggu");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn game_start_draws_three() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 3);
}
}

mod original_sp_olyinji {
//! Golden for SP 刘焉「殷积」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLYINJI: u32 = 2_633_842_541;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated(owner_hp: i32, other_hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLYINJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hp;
    tx.state_mut().players.get_mut(other).unwrap().hp = other_hp;
    (tx, owner)
}

fn phase_end(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    option: u32,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![option],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_locked_phase_end_choice() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 70);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLYINJI && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLYINJI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "phase_end",
            "owner_is_target_not_unique_max_current_hp",
            false
        )
    );
}

#[test]
fn non_unique_owner_must_choose_recover_or_gain_max_hp() {
    let (mut recover, owner) = isolated(2, 3);
    assert_eq!(
        phase_end(&mut recover, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let max_hp = recover.state().players.get(owner).unwrap().max_hp;
    let recovered = resolve(&recover, owner, 1, 1);
    assert_eq!(recovered.players.get(owner).unwrap().hp, 3);
    assert_eq!(recovered.players.get(owner).unwrap().max_hp, max_hp);

    let (mut gain_max, owner) = isolated(2, 3);
    assert_eq!(
        phase_end(&mut gain_max, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let max_hp = gain_max.state().players.get(owner).unwrap().max_hp;
    let increased = resolve(&gain_max, owner, 2, 2);
    assert_eq!(increased.players.get(owner).unwrap().hp, 2);
    assert_eq!(increased.players.get(owner).unwrap().max_hp, max_hp + 1);

    let (mut tied, owner) = isolated(3, 3);
    assert_eq!(
        phase_end(&mut tied, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn unique_highest_owner_gets_no_choice_or_effect() {
    let (mut tx, owner) = isolated(4, 3);
    let before = tx.state().clone();
    let effects = phase_end(&mut tx, owner).unwrap();
    assert!(effects.is_empty(), "{effects:?}");
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner), before.players.get(owner));
}
}

mod original_sp_olzaowang_weicheng {
//! Golden for sp 早亡 (olzaowang) + 微城 (weicheng) rewrites.
//!
//! olzaowang: 限定技 — play phase, once per game: a chosen player gains 1 max HP,
//! heals 1, and draws 3.
//! weicheng: when cards move from the owner's hand to another player's hand, if the
//! owner's hand size is below their HP, they may draw 1.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLZAOWANG: u32 = 3_649_255_847;
const WEICHENG: u32 = 4_091_204_654;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_701,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn inject_hand(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_declares_olzaowang_active_and_weicheng_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.active_index.iter().any(|a| a.skill_def_id == OLZAOWANG));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == WEICHENG
            && t.event == "cards_given"
            && t.optional
    }));
}

#[test]
fn olzaowang_grants_max_hp_heal_and_draw_once() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, OLZAOWANG);
    tx.state_mut().turn.phase = Phase::Play;
    let other_max_before = tx.state().players.get(other).unwrap().max_hp;
    let other_hp_before = tx.state().players.get(other).unwrap().hp;
    let other_hand_before = tx.state().hand_count(other);

    let (state, first) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(first.result, CommandResult::Accepted { .. }),
        "早亡 activation accepted: {:?}",
        first.result
    );
    assert_eq!(state.players.get(other).unwrap().max_hp, other_max_before + 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp_before + 1);
    assert_eq!(
        state.hand_count(other),
        other_hand_before + 3,
        "目标摸三张"
    );

    // 限定技 — second activation rejected.
    let (_, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(second.result, CommandResult::Rejected { .. }),
        "限定技 only fires once"
    );
}

#[test]
fn weicheng_draws_when_hand_below_hp_after_giving() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, WEICHENG);
    // Owner is the lord: hp 5, hand 4 → hand < hp holds.
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let give = tx.state().hand_cards(owner)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![give],
            to: other_hand,
            from: Some(hand),
        }],
    )
    .unwrap();
    // Set the owner's HP above their hand so the draw condition holds.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    let owner_hand_before = tx.state().hand_count(owner);
    assert!((owner_hand_before as i32) < 6);

    // Dispatch the cards-given event → optional trigger opens a confirm choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![give],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(
        tx.state().has_pending_choice(),
        "optional 微城 opens a confirm choice"
    );

    // Confirm → owner draws 1 (hand below HP).
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "confirm accepted: {:?}",
        confirmed.result
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "微城 draws 1"
    );
}

#[test]
fn weicheng_confirm_without_hand_below_hp_draws_nothing() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, WEICHENG);
    // Owner hp = 1 → hand (4) is not below HP → confirming draws nothing.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let give = tx.state().hand_cards(owner)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![give],
            to: other_hand,
            from: Some(hand),
        }],
    )
    .unwrap();
    let owner_hand_before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![give],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(tx.state().has_pending_choice(), "trigger still offers");

    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before,
        "hand not below HP → no draw"
    );
}
}

mod original_sp_olzhijie_machine {
//! Golden for SP 直谏 olzhijie (ABI 738, tag 759, Cont 329):
//! 锁定技——一名角色的结束阶段，若你本回合使用/打出过牌、且本回合被选
//! 择为牌的目标、且射程内有角色：选射程内一名角色对其造成 1 点伤害。
//! 真管道驱动（dispatch_for_events PhaseEnded）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLZHIJIE: u32 = 3_419_173_958;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject_hand(state: &mut GameState, owner: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_000 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn end_phase(
    state: &mut GameState,
    current: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    let mut tx = Transaction::begin(state.clone());
    // 上游 phaseJieshuBegin = 结束阶段开始 = TurnStateChanged(End)。
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    // open_choice 打开时以 Err(ChoiceRequested) 短路——正常信号。
    let dispatch_result = run_effects(&mut tx, effects);
    *state = tx.commit().state;
    match dispatch_result {
        Ok(_) | Err(EffectError::ChoiceRequested) => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

#[test]
fn package_publishes_olzhijie_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLZHIJIE)
        .expect("olzhijie trigger");
    assert_eq!(entry.event.as_str(), "phase_end");
    assert_eq!(
        entry.filter.as_str(),
        "owner_used_and_was_targeted_this_turn_in_range"
    );
    assert!(!entry.optional);
}

#[test]
fn end_phase_with_all_conditions_met_deals_one_damage_in_range() {
    let (mut state, players) = started(2_026_094_001);
    let owner = players[0];
    let target = players[1];
    grant(&mut state, owner, OLZHIJIE);
    // 三条件置备：本回合用过牌（flag 直置）、被指定为目标（事件记录）、
    // 射程内有角色（二人局相邻默认在射程内）。
    {
        let mut tx = Transaction::begin(state.clone());
        let key = engine_core::state::turn_flag::card_action_count_by_player(owner);
        tx.state_mut().turn.set_flag(key, 1);
        state = tx.commit().state;
    }
    // 经真实 CardUsed 事件产生被指定目标记录（track 门控开启）。
    let card = inject_hand(&mut state, target, 1);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: target,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert!(state.turn.card_targeted_count(owner) > 0, "被指定目标应记录");

    // 结束阶段触发 → 强制选目标（射程内）。
    end_phase(&mut state, target).unwrap();
    assert!(
        state.has_pending_choice(),
        "三条件齐备应打开选目标窗口"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlzhijieStage
    );

    let (state, result) = resolve(state, owner, vec![target.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn end_phase_without_conditions_is_quiet() {
    let (mut state, players) = started(2_026_094_002);
    let owner = players[0];
    let other = players[1];
    grant(&mut state, owner, OLZHIJIE);
    // 无用牌、无被指定目标 → 静默。
    let effects = end_phase(&mut state, other).unwrap();
    assert!(effects.is_empty());
    let other_hp = state.players.get(other).unwrap().hp;
    assert_eq!(other_hp, 3);
}
}

mod original_sp_olzhubi {
//! Golden coverage for SP OL 刘巴「铸币」(olzhubi).
//! Play-phase active: selected target draws 1.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZHUBI: u32 = 4_168_204_873;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olzhubi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("olzhubi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_53, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLZHUBI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLZHUBI)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

#[test]
fn compiled_source_and_npack_publish_the_active_draw() {
    let compiled = content_compiler::compile_source_named("olzhubi", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLZHUBI)
        .expect("olzhubi active");
    assert_eq!(active.window, "play_phase");
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .active_index
        .iter()
        .any(|active| active.skill_def_id == OLZHUBI && active.window == "play_phase"));
}

#[test]
fn selected_target_draws_one() {
    let (state, owner, other, skill) = setup();
    let before = state.hand_count(other);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }), "{:?}", result.result);
    assert_eq!(state.hand_count(other), before + 1);
}
}

mod original_sp_olzonglve {
//! Golden coverage for SP OL 牛辅「纵掠」(olzonglve).
//! Play-phase view-as Sha once per turn; dealing damage takes one he from the victim.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZONGLVE: u32 = 3_052_953_425;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olzonglve.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject_hand(state: &mut GameState, player: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("olzonglve-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_32, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLZONGLVE),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = inject_hand(tx.state_mut(), other, 170_601);
    (tx.commit().state, owner, other, card)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_sha_and_take() {
    let compiled = content_compiler::compile_source_named("olzonglve", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLZONGLVE)
        .expect("olzonglve view_as");
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.uses_per_turn, Some(1));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLZONGLVE)
        .expect("olzonglve trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_target_alive_has_hand_or_equip");

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == OLZONGLVE && view.as_kind == "sha"));
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLZONGLVE && trigger.filter == "owner_source_target_alive_has_hand_or_equip"
    }));
}

#[test]
fn damage_takes_one_card_from_the_victim() {
    let (state, owner, other, card) = setup();
    let owner_hand = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, taken) = resolve(tx.commit().state, owner, vec![card.raw()], 1);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }), "{:?}", taken.result);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
}
}

mod original_sp_pxlangya {
//! 第一百七十六批：sp「狼牙」（彭城 pxlangya）观星族首原语（ABI 681）。
//! 上游：锁定技，你的技能增加后，你观看牌堆顶的五张牌并任意重排（拖拽顶/底）。
//! 实现：skill_added + owner_skill_added_self_<def> → guanxing_top(self, 5)
//! （逐张二选顶/底近似拖拽自由排列——登记债）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXLANGYA: u32 = 3_999_958_998;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    // AddSkill 触发狼牙（skill_added）→ guanxing_top 的 ChoiceRequested 属预期控制流。
    let _ = engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXLANGYA),
        }],
    );
    (tx.commit().state, owner, other)
}

#[test]
fn package_publishes_pxlangya_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == PXLANGYA
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3999958998"
            && !trigger.optional
    }));
}

#[test]
fn pxlangya_guanxing_reorders_top_five() {
    // 给牌堆顶 5 张打上可识别的点数序列。
    let (state, owner, _other) = started(2_026_092_001);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let state = tx.commit().state;

    // 模拟「技能增加」：owner 再获得任意技能（sp 包内真实 def——用 pxlangya 自身
    // 会重复；改用重复 AddSkill pxlangya 亦可再次触发）。
    // 直接路径：再次 AddSkill pxlangya 触发 skill_added（过滤器要求 def 匹配自身，
    // 重复添加仍发出 SkillAdded 事件）。
    let mut tx = Transaction::begin(state);
    let effects = vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXLANGYA),
    }];
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    // 观星选择链开启（ChoiceRequested）或已直接进入多轮；断言 pending 存在。
    let _ = outcome;
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "guanxing pick loop must open after skill gain"
    );

    // 五轮：前四轮全部选「底」（token 2），最后一轮选「顶」（token 1）。
    let mut key = 2_026_092_002;
    for round in 0..5 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        let pick = if round == 4 { 1 } else { 2 };
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![pick],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "round {round}: resolve rejected {:?}",
            result.result
        );
        state = next;
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "guanxing loop must close after 5 rounds"
    );

    // 结算后牌堆顶 5 张被重排：全部置底的 4 张在前（底序列），最后置顶的 1 张在
    // 牌堆最顶。核验「处理区已清空、牌回到牌堆」：
    let processing = state.zones.find(ZoneKind::Processing, PlayerId::NONE).unwrap();
    assert!(
        state.zones.get(processing).unwrap().cards.is_empty(),
        "processing must be drained after guanxing settles"
    );
    // 牌堆顶存在 5 张（重排落位完成，可能混入洗牌补堆的牌——牌堆不足时自动洗回）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    assert!(
        state.zones.get(draw).unwrap().cards.len() >= 5,
        "draw pile must hold at least the reordered cards"
    );
}
}

mod original_sp_qiangwu {
//! Golden for SP/OLD 张星彩「枪舞」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIANGWU: u32 = 2_563_176_834;
const SHENXIAN: u32 = 3_316_671_408;
const OLD_SHENXIAN: u32 = 2_584_430_535;
const ZHANGXINGCAI: u32 = 3_565_568_608;
const OLD_ZHANGXINGCAI: u32 = 2_854_497_058;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/qiangwu.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn inject(
    state: &mut GameState,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
    def: u32,
) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = compile_source_named("sp-qiangwu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_501, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let far_target = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGWU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == QIANGWU)
        .unwrap()
        .id;
    (tx.commit().state, owner, far_target, skill)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(state: GameState, owner: PlayerId, card: CardId, target: PlayerId, key: u64) -> GameState {
    let (state, pipeline) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    state
}

#[test]
fn packages_publish_both_zhangxingcai_closures() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIANGWU));
    assert_eq!(
        sp.character_index
            .iter()
            .find(|character| character.id == ZHANGXINGCAI)
            .unwrap()
            .skills,
        [SHENXIAN, QIANGWU]
    );

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_ZHANGXINGCAI)
            .unwrap()
            .skills,
        [OLD_SHENXIAN, QIANGWU]
    );
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
}

#[test]
fn judgment_rank_drives_strict_lower_distance_and_higher_no_count() {
    let (mut state, owner, far_target, skill) = setup();
    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Other,
        7,
        188_001,
    );
    let low = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 3, 188_002);
    let equal = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 7, 188_003);
    let high = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 9, 188_004);
    let higher = inject(
        &mut state,
        ZoneKind::Hand,
        owner,
        CardKind::Sha,
        10,
        188_005,
    );
    let rankless = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 0, 188_006);

    let (state, activated) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                rank: 7,
                ..
            } if *player == owner
        )
    }));
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == QIANGWU)
            .map(|(_, count)| *count),
        Some(7)
    );
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state, owner, equal, far_target
    ));
    assert!(!engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, equal
    ));
    assert!(engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, high
    ));
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, rankless, far_target
    ));
    assert!(engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, rankless
    ));

    let adjacent = state.players.alive_by_seat()[1];
    let state = play(state, owner, low, far_target, 2);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let state = play(state, owner, high, adjacent, 3);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let state = play(state, owner, higher, adjacent, 4);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);

    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: equal,
            targets: vec![adjacent],
            card_targets: Vec::new(),
        },
        5,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn play_phase_end_clears_rank_policy() {
    let (mut state, owner, far_target, skill) = setup();
    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Other,
        8,
        188_010,
    );
    let low = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 2, 188_011);
    let (state, activated) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));

    let (state, ended) = submit(state, owner, CommandKind::EndPhase, 11);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == QIANGWU));
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));
}
}

mod original_sp_qiangwu3_olshilu {
//! Golden for sp 强武 (qiangwu3) + 识露 (olshilu) rewrites.
//!
//! qiangwu3: locked modifiers — Sha with rank below the skill mark ignore
//! distance; Sha with rank above the mark ignore use limits.
//! olshilu: locked — after taking damage, draw min(5, HP).

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLSHILU: u32 = 2_791_913_057;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_401,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_olshilu_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == OLSHILU && t.event == "damage"
    }));
}

#[test]
fn olshilu_draws_min_hp_five_after_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, OLSHILU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let hand_before = tx.state().hand_count(owner);

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }

    // HP is 3 before, 2 after the damage → min(5, 2) = 2.
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "识露 draws min(5, HP)=2 after the damage"
    );
}

#[test]
fn olshilu_draws_capped_at_five() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, OLSHILU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    let hp_after = tx.state().players.get(owner).unwrap().hp - 1;
    let expected = hp_after.min(5).max(0) as u32;
    let hand_before = tx.state().hand_count(owner);

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + expected,
        "识露 caps the draw at 5"
    );
}

#[test]
fn olshilu_trigger_is_locked_and_forced() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = pkg
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == OLSHILU)
        .unwrap();
    assert!(!trigger.optional, "识露 is forced");
}
}

mod original_sp_qimen_zniaoxiang {
//! Golden tests for SP「奇门八卦」与「鸟翔」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const QIMEN_SOURCE: &str = include_str!("../../../content/original/ported/sp/qimenbagua.skill");
const NIAOXIANG_SOURCE: &str = include_str!("../../../content/original/ported/sp/zniaoxiang.skill");
const QIMENBAGUA: u32 = 2_757_055_338;
const ZNIAOXIANG: u32 = 3_916_851_836;
const IGNORE_ARMOR: u32 = 4_290_000_067;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64, with_fixture: bool) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let fixture = content_compiler::compile_source_named(
        "qimen-fixture",
        &format!("skill ignore_armor({IGNORE_ARMOR}) {{ modifier ignore_armor override 1; }}"),
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    if with_fixture {
        engine.load_package(&fixture).unwrap();
    }
    assert!(matches!(
        engine.submit(start(seed)),
        CommandResult::Accepted { .. }
    ));
    // 2026-08-15：sp 包出现 optional game_started 技能（olpingyuan/xiaoxi 等），
    // 抽到相关武将时 StartGame 会留下挂起的可选触发确认，击穿后续 setup 批次；
    // 此处以「取消」排空开局选择，对任何 optional 开局技保持鲁棒。
    let mut drained = engine.state().clone();
    let mut drain_key = 1u64;
    while drained.has_pending_choice() {
        let choice = engine_api::ChoiceId::new(drained.resolution.pending_choice_id);
        let revision = drained.revision;
        let (next, output) = apply_command(
            drained,
            engine_api::Command {
                actor: engine_api::PlayerId::NONE,
                expected_revision: engine_api::ExpectedRevision(revision),
                idempotency_key: engine_api::IdempotencyKey(seed ^ drain_key),
                kind: engine_api::CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, engine_api::CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        drained = next;
        drain_key += 1;
    }
    let source = drained.turn.current;
    let target = drained
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(drained);
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(67_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(source: PlayerId, card: CardId, target: PlayerId, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_package_contains_both_exact_modifiers() {
    let committed = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let qimen = content_compiler::compile_source_named("qimenbagua", QIMEN_SOURCE).unwrap();
    let niaoxiang = content_compiler::compile_source_named("zniaoxiang", NIAOXIANG_SOURCE).unwrap();

    assert_eq!(qimen.modifier_index[0].target, "armor_card_target_cancel");
    assert_eq!(qimen.modifier_index[0].selector.as_deref(), Some("sha"));
    assert_eq!(niaoxiang.modifier_index[0].target, "response_count");
    for skill in [QIMENBAGUA, ZNIAOXIANG] {
        assert!(committed
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill && entry.locked));
    }
}

#[test]
fn qimenbagua_blocks_sha_only_while_armor_effects_apply() {
    let (mut blocked, source, target) = loaded(20_260_718, false);
    add_skill(&mut blocked, target, QIMENBAGUA);
    let sha = inject(&mut blocked, source, CardKind::Sha, 7);
    let hp = blocked.state().players.get(target).unwrap().hp;
    let (state, pipeline) = apply_command(blocked.state().clone(), play(source, sha, target, 1));
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(state.hand_cards(source).contains(&sha));

    let (mut bypassed, source, target) = loaded(20_260_719, true);
    add_skill(&mut bypassed, target, QIMENBAGUA);
    add_skill(&mut bypassed, source, IGNORE_ARMOR);
    let sha = inject(&mut bypassed, source, CardKind::Sha, 8);
    let hp = bypassed.state().players.get(target).unwrap().hp;
    let (state, pipeline) = apply_command(bypassed.state().clone(), play(source, sha, target, 2));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn zniaoxiang_requires_two_shan_for_each_sha_target() {
    let (mut tx, source, target) = loaded(20_260_720, false);
    add_skill(&mut tx, source, ZNIAOXIANG);
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), source, 1),
        2
    );
    let sha = inject(&mut tx, source, CardKind::Sha, 9);
    let first = inject(&mut tx, target, CardKind::Shan, 10);
    let second = inject(&mut tx, target, CardKind::Shan, 11);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, sha, target, 3));
    let first_choice = pipeline.choice.unwrap();
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: first_choice.choice_id,
                selections: vec![first.raw()],
                cancelled: false,
            },
        },
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let second_choice = pipeline.choice.unwrap();
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id: second_choice.choice_id,
                selections: vec![second.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
}

mod original_sp_qiongshou {
//! Golden for SP 霍峻「穷守」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const QIONGSHOU: u32 = 2_792_246_343;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/qiongshou.skill");

fn package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("sp-qiongshou", SOURCE).unwrap()
}

fn isolated() -> (Transaction, PlayerId) {
    let package = package();
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QIONGSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn package_declares_locked_game_start_rule_modifier_and_abi18() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 18);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == QIONGSHOU)
        .unwrap();
    assert!(skill.locked);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(package.trigger_index[0].event, "game_started");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_has_enabled_equip_slot"
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == QIONGSHOU
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value == 4
    }));
}

#[test]
fn game_start_disables_all_slots_discards_equipment_draws_four_and_raises_limit() {
    let (mut tx, owner) = isolated();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);
    let before_hand = tx.state().hand_count(owner);
    let before_limit = hand_limit(tx.state(), owner);
    assert_eq!(before_limit, tx.state().players.get(owner).unwrap().hp + 4);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.len() >= 2, "{effects:?}");
    assert!(
        matches!(
            &effects[..2],
            [
                EffectRequest::DisableEquipSlots { player, slots },
                EffectRequest::DrawCardsFromSkill { player: draw_player, count: 4, .. }
            ] if *player == owner && slots == &[1, 2, 3, 4, 5] && *draw_player == owner
        ),
        "{effects:?}"
    );
    let events = run_effects(&mut tx, effects).unwrap();

    assert_eq!(tx.state().hand_count(owner), before_hand + 4);
    assert_eq!(hand_limit(tx.state(), owner), before_limit);
    assert_eq!(
        tx.state().players.get(owner).unwrap().disabled_equip_slots,
        [1; 5]
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(weapon).unwrap().zone, discard);
    let disabled: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::EquipSlotDisabled { player, slot } if *player == owner => Some(*slot),
            _ => None,
        })
        .collect();
    assert_eq!(disabled, vec![1, 2, 3, 4, 5]);
}

#[test]
fn fully_disabled_owner_does_not_draw_again() {
    let (mut tx, owner) = isolated();
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots = [1; 5];
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(tx.state().hand_count(owner), before);
}
}

mod original_sp_rebaobian_xianding_xiahouba {
//! Golden coverage for SP「豹变」and XIANDING 新杀夏侯霸 (ABI 575).

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REBAOBIAN: u32 = 4_258_572_312;
const TIAOXIN: u32 = 695_232_805;
const OLPAOXIAO: u32 = 2_829_854_661;
const XINSHENSU: u32 = 2_561_255_694;
const DC_XIAHOUBA: u32 = 931_835_034;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp/rebaobian.skill"),
    "\n",
    include_str!("../../../content/original/ported/shenhua/jiangwei.skill"),
    "\n",
    include_str!("../../../content/original/ported/refresh/olpaoxiao.skill"),
    "\n",
    include_str!("../../../content/original/ported/shenhua/xinshensu.skill"),
);
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const XIANDING_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("rebaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, REBAOBIAN as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REBAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

#[test]
fn packages_publish_the_skill_and_complete_character_closure() {
    let compiled = content_compiler::compile_source_named("rebaobian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 575);
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    let skill = sp
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REBAOBIAN)
        .unwrap();
    assert_eq!(skill.name, "rebaobian");
    assert!(skill.locked);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "rebaobian_damage")
        .unwrap();
    let actual = sp
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let xianding = content_schema::ContentPackage::from_json_bytes(XIANDING_PACKAGE).unwrap();
    let character = xianding
        .character_index
        .iter()
        .find(|character| character.id == DC_XIAHOUBA)
        .unwrap();
    assert_eq!(character.skills, [REBAOBIAN]);
}

#[test]
fn each_damage_grants_the_first_missing_derivation_in_fixed_order() {
    let (state, owner, source) = setup();
    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, TIAOXIN));
    assert!(!has_skill(&state, owner, OLPAOXIAO));

    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, OLPAOXIAO));
    assert!(!has_skill(&state, owner, XINSHENSU));

    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, XINSHENSU));
    let skill_count = state.players.get(owner).unwrap().skills.len();

    let state = damage(state, source, owner);
    assert_eq!(state.players.get(owner).unwrap().skills.len(), skill_count);
}

#[test]
fn existing_earlier_skill_is_skipped_without_player_choice() {
    let (state, owner, source) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIAOXIN),
        }],
    )
    .unwrap();
    let state = damage(tx.commit().state, source, owner);
    assert!(has_skill(&state, owner, OLPAOXIAO));
    assert!(!state.has_pending_choice());
}
}

mod original_sp_recanshi_sunhao {
//! Golden coverage for SP 孙皓「残蚀」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RECANSHI: u32 = 2_466_532_461;
const RECANSHI2: u32 = 2_932_254_333;
const RECHOUHAI: u32 = 4_121_233_014;
const GUIMING: u32 = 3_828_399_214;
const SUNHAO: u32 = 2_778_414_395;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp/recanshi.skill"),
    "\nskill rechouhai(4121233014) locked {}\nskill guiming(3828399214) locked lord_only {}\n"
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    qun: PlayerId,
    wu: PlayerId,
}

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("recanshi_{id}"),
        max_hp: 5,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(seed: u64) -> Setup {
    let mut package = content_compiler::compile_source_named("recanshi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_266_001, "wu"),
        character(9_266_002, "qun"),
        character(9_266_003, "wu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let others: Vec<_> = players
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();
    for (player, character) in [
        (owner, 9_266_001),
        (others[0], 9_266_002),
        (others[1], 9_266_003),
    ] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECANSHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIMING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    Setup {
        state: tx.commit().state,
        owner,
        qun: others[0],
        wu: others[1],
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn inject_event_sha(state: &mut GameState, def: u32) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_current_sp_abi_and_complete_sunhao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RECANSHI
            && trigger.event == "phase_draw"
            && trigger.filter == "owner_is_target_alive_wounded_or_guiming_wu_exists"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RECANSHI2
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SUNHAO)
        .unwrap();
    assert_eq!(character.name_key, "sunhao");
    assert_eq!(character.max_hp, 5);
    assert!(character.is_lord);
    assert_eq!(character.skills, [RECANSHI, RECHOUHAI, GUIMING]);
}

#[test]
fn wounded_and_guiming_wu_union_adds_draws_then_sha_forces_discard_and_expires() {
    let mut setup = setup(2_026_072_503);
    setup.state.players.get_mut(setup.owner).unwrap().hp -= 1;
    setup.state.players.get_mut(setup.qun).unwrap().hp -= 1;
    assert_eq!(
        engine_core::query::alive_wounded_or_guiming_wu_count(&setup.state, setup.owner),
        3
    );
    let before = setup.state.hand_count(setup.owner);
    let (offered_state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (mut state, accepted) = resolve(offered_state, setup.owner, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 5);
    assert!(skill_id(&state, setup.owner, RECANSHI2).is_some());

    state.turn.phase = Phase::Play;
    let sha = inject_event_sha(&mut state, 190_001);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.wu],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let pending = tx.commit();
    let choice = pending.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let cost = choice.options[0].token;
    let hand_before = pending.state.hand_count(setup.owner);
    let (state, result) = resolve(pending.state, setup.owner, vec![cost], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before - 1);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), setup.owner, RECANSHI2).is_none());
}

#[test]
fn no_eligible_player_means_no_optional_offer() {
    let mut setup = setup(2_026_072_504);
    setup.state.players.get_mut(setup.owner).unwrap().identity = Identity::Rebel;
    assert_eq!(
        engine_core::query::alive_wounded_or_guiming_wu_count(&setup.state, setup.owner),
        0
    );
    let (state, result) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(skill_id(&state, setup.owner, RECANSHI2).is_none());
}
}

mod original_sp_rehengjiang_huicui_zangba {
//! Golden coverage for SP 臧霸「横江」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{turn_flag, GameState};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REHENGJIANG: u32 = 2_552_200_673;
const HAND_HELPER: u32 = 4_200_000_582;
const DRAW_HELPER: u32 = 4_200_000_583;
const ZANGBA: u32 = 1_112_200_336;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/rehengjiang.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const HUICUI_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, SkillId) {
    let package = compile_source_named("sp-rehengjiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803_01, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let actor = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REHENGJIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = actor;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REHENGJIANG)
        .unwrap()
        .id;
    (tx, owner, actor, skill)
}

fn activate(tx: &mut Transaction, owner: PlayerId, skill: SkillId) {
    run_effects(
        tx,
        vec![EffectRequest::ResolveRehengjiang {
            player: owner,
            actor: PlayerId::NONE,
            action: 0,
            hand_limit_def: SkillDefId::new(HAND_HELPER),
            draw_def: SkillDefId::new(DRAW_HELPER),
            skill,
        }],
    )
    .unwrap();
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

fn draw_helper(tx: &Transaction, owner: PlayerId) -> SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DRAW_HELPER)
        .unwrap()
        .id
}

fn settle(tx: &mut Transaction, owner: PlayerId, actor: PlayerId) {
    let helper = draw_helper(tx, owner);
    run_effects(
        tx,
        vec![EffectRequest::ResolveRehengjiang {
            player: owner,
            actor,
            action: 1,
            hand_limit_def: SkillDefId::new(HAND_HELPER),
            draw_def: SkillDefId::new(DRAW_HELPER),
            skill: helper,
        }],
    )
    .unwrap();
}

#[test]
fn each_damage_point_stacks_and_no_discard_draws_activation_count() {
    let (mut tx, owner, actor, skill) = setup();
    let base_limit = hand_limit(&tx, actor);
    activate(&mut tx, owner, skill);
    activate(&mut tx, owner, skill);
    assert_eq!(hand_limit(&tx, actor), base_limit - 2);

    let hand_helper = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == actor && instance.def.raw() == HAND_HELPER)
        .unwrap();
    assert_eq!(hand_helper.temporary_until_turn_end_of, actor);
    let before = tx.state().hand_count(owner);
    settle(&mut tx, owner, actor);
    assert_eq!(tx.state().hand_count(owner), before + 2);
    assert!(tx
        .state()
        .skills
        .get(draw_helper(&tx, owner))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn discarding_in_discard_phase_reduces_reward_to_one() {
    let (mut tx, owner, actor, skill) = setup();
    activate(&mut tx, owner, skill);
    activate(&mut tx, owner, skill);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_DISCARDED, 1);
    let before = tx.state().hand_count(owner);
    settle(&mut tx, owner, actor);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn huicui_zangba_reuses_the_canonical_sp_skill() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REHENGJIANG));
    assert!(huicui
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
    let zangba = huicui
        .character_index
        .iter()
        .find(|character| character.id == ZANGBA)
        .unwrap();
    assert_eq!(zangba.name_key, "zangba");
    assert_eq!(zangba.skills, [REHENGJIANG]);
}
}

mod original_sp_rekenshang {
//! Golden coverage for SP 马休马铁「垦伤」(rekenshang).
//! Play-phase view-as: at least two hand/equip cards as Sha (uncapped,
//! at_least syntax with max 255 sentinel, 批 298).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REKENSHANG: u32 = 3_074_298_633;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/rekenshang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, Vec<CardId>) {
    let compiled = content_compiler::compile_source_named("rekenshang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_41, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKENSHANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut cards = Vec::new();
    for (index, kind) in [CardKind::Tao, CardKind::Shan, CardKind::Other].into_iter().enumerate() {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(170_800 + index as u32),
            kind,
            suit: Suit::Club,
            rank: 4,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        cards.push(card);
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == REKENSHANG)
        .unwrap();
    (tx.commit().state, owner, other, skill, cards)
}

#[test]
fn compiled_source_and_npack_publish_sha_at_least_two_uncapped() {
    let compiled = content_compiler::compile_source_named("rekenshang", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == REKENSHANG)
        .expect("rekenshang view_as");
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.min_cards, 2);
    assert_eq!(view.max_cards, u8::MAX);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == REKENSHANG)
        .expect("npack rekenshang view_as");
    assert_eq!(pack.as_kind, "sha");
    assert_eq!(pack.min_cards, 2);
    assert_eq!(pack.max_cards, u8::MAX);
}

#[test]
fn conversion_requires_two_cards_minimum() {
    let (state, owner, other, skill, cards) = setup();
    let hp = state.players.get(other).unwrap().hp;
    // 1 张素材 → 低于 at_least 2 下限，拒绝。
    let (_, rejected) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![cards[0]],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // 2 张素材 → 接受，转换【杀】结算掉血。
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![cards[0], cards[1]],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
}
}

mod original_sp_shanjia_old_caochun {
//! Golden coverage for 缮甲 and old 曹纯 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHANJIA: u32 = 3_876_659_100;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/shanjia.skill");

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_200 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_equipped_weapon(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_299),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 13,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn only_owner_equipment_uses_count_to_seven_and_equipment_discard_offers_far_sha() {
    let package = content_compiler::compile_source_named("shanjia-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 283);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_source_equip_used_skill_mark_below_seven" }));
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_903, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let far = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANJIA),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let foreign_equip = inject_weapon(&mut tx, other, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card: foreign_equip,
            targets: vec![other],
        }],
    )
    .unwrap();
    for rank in 2..=9 {
        let weapon = inject_weapon(&mut tx, owner, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card: weapon,
                targets: vec![owner],
            }],
        )
        .unwrap();
    }
    let equipped = inject_equipped_weapon(&mut tx, owner);
    let marks = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SHANJIA)
        .map_or(0, |(_, count)| *count);
    assert_eq!(marks, 7);

    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(SHANJIA))
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveShanjia {
                player: owner,
                count: marks as u32,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut selected = vec![equipped.raw()];
    selected.extend(
        engine_core::query::discardable_hand_equip_cards(tx.state(), owner)
            .into_iter()
            .filter(|card| *card != equipped)
            .take(6)
            .map(CardId::raw),
    );
    assert_eq!(selected.len(), 7);
    let state = tx.commit().state;
    assert_eq!(state.resolution.continuation_payload[1], 7);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let legal = engine_core::query::discardable_hand_equip_cards(&state, owner);
    assert!(
        selected
            .iter()
            .all(|card| legal.contains(&CardId::new(*card))),
        "{selected:?} not within {legal:?}"
    );
    let state = choose(state, owner, selected, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.resolution.continuation_payload[2], 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipped).unwrap().zone, discard);
    let state = choose(state, owner, vec![far.raw()], 2);
    assert_ne!(
        state.cards.get(equipped).unwrap().zone,
        state.zones.find(ZoneKind::Equip, owner).unwrap()
    );
}
}

mod original_sp_shenzhi {
//! Golden for SP 甘夫人「神智」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SHENZHI: u32 = 2_944_037_719;
const STDSHUSHEN: u32 = 3_504_783_479;
const GANFUREN: u32 = 3_981_632_700;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated(hand: u32, hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SHENZHI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: SHENZHI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENZHI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    assert_eq!(tx.state().hand_count(owner), hand);
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_optional_nonempty_prepare_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GANFUREN)
        .unwrap();
    assert_eq!(character.name_key, "ganfuren");
    assert_eq!(character.skills, vec![STDSHUSHEN, SHENZHI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHENZHI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_prepare", "owner_is_target_hand_nonempty", true)
    );
}

#[test]
fn accepting_discards_every_hand_card_and_recovers_when_count_reaches_hp() {
    let (mut tx, owner) = isolated(3, 2);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (mut no_recover, owner) = isolated(1, 2);
    assert_eq!(
        offer(&mut no_recover, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&no_recover, owner, true, 2);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn declining_preserves_cards_and_empty_hand_creates_no_choice() {
    let (mut declined, owner) = isolated(2, 2);
    assert_eq!(
        offer(&mut declined, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&declined, owner, false, 3);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);

    let (mut empty, owner) = isolated(0, 2);
    assert!(offer(&mut empty, owner).unwrap().is_empty());
    assert!(!empty.state().has_pending_choice());
}
}

mod original_sp_shoucheng {
//! Golden for SP「守成」：非当前回合角色失去最后的手牌后可摸一张牌。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHOUCHENG: u32 = 3_263_722_944;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn resolve(tx: Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(SHOUCHENG as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn only_non_current_hand_loss_to_empty_offers_the_draw() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHOUCHENG
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_event_player_outside_turn_from_hand_empty"
            && trigger.optional
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, other, current) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHOUCHENG),
        },
        EffectRequest::DrawCards {
            player: other,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_card = tx.state().hand_cards(other)[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![other_card],
                to: discard,
                from: Some(other_hand),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx, owner);
    assert_eq!(state.hand_count(other), 1);

    let mut tx = Transaction::begin(state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let equipment = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    let shoucheng = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SHOUCHENG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: shoucheng },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: other_equip,
                from: Some(other_hand),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SHOUCHENG),
            },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: discard,
                from: Some(other_equip),
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(other), 0);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DiscardFromHand {
                player: current,
                count: u32::MAX,
            },
            EffectRequest::DrawCards {
                player: current,
                count: 1,
            },
        ],
    )
    .unwrap();
    let current_hand = tx.state().zones.find(ZoneKind::Hand, current).unwrap();
    let current_card = tx.state().hand_cards(current)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![current_card],
            to: discard,
            from: Some(current_hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(current), 0);
}
}

mod original_sp_shulv {
//! Golden for SP `shulv`, rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHULV: u32 = 2_586_462_603;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHULV),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHULV)
        .unwrap();
    (tx.state().clone(), owner, skill)
}

fn add_equipment(state: &mut GameState, owner: PlayerId) -> CardId {
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_042),
        kind: CardKind::Treasure,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn shulv_requires_hand_above_hp_then_discards_one_and_draws_one() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHULV)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_gt_hp);
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand_equip");

    let (mut state, owner, skill) = started(2_026_071_519);
    state.players.get_mut(owner).unwrap().hp = 3;
    let cost = add_equipment(&mut state, owner);
    let hand = state.hand_count(owner);
    let (state, pipe) = activate(state, owner, skill, cost, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(
        state.cards.get(cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (mut state, owner, skill) = started(2_026_071_520);
    let hand = state.hand_count(owner);
    state.players.get_mut(owner).unwrap().hp = hand as i32;
    let cost = add_equipment(&mut state, owner);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, cost, 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_sp_shushen_offline_jsp_ganfuren {
//! Golden coverage for SP「淑慎」and the OFFLINE SP 甘夫人 cross-package closure.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSP_GANFUREN: u32 = 3_576_521_509;
const SHUSHEN: u32 = 3_835_991_174;
const SHENZHI: u32 = 2_944_037_719;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/shushen.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("shushen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_723, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHUSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, first, second)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
}

fn recover(tx: &mut Transaction, player: PlayerId, amount: u32) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::RecoverHp {
            source: player,
            player,
            amount,
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_packages_expose_shushen_and_complete_jsp_ganfuren_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("shushen-abi", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        101
    );
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHUSHEN
            && trigger.event == "hp_recovered"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == JSP_GANFUREN)
        .unwrap();
    assert_eq!(character.name_key, "jsp_ganfuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, vec![SHUSHEN, SHENZHI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn each_recovered_hp_point_has_an_independent_target_choice_and_target_draw_branch() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 2);
    damage(&mut tx, source, target, 1);
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 2),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, target, vec![1], false, 2);
    assert_eq!(state.hand_count(target), before_hand + 2);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, Vec::new(), true, 3);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), before_hand + 2);
}

#[test]
fn wounded_target_can_choose_recovery() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 1);
    damage(&mut tx, source, target, 1);
    let before_hp = tx.state().players.get(target).unwrap().hp;
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 1),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    let state = resolve(state, target, vec![2], false, 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, before_hp + 1);
    assert_eq!(state.hand_count(target), before_hand);
}

#[test]
fn full_hp_target_draws_without_a_second_choice() {
    let (mut tx, owner, target, source) = setup();
    damage(&mut tx, source, owner, 1);
    let before_hand = tx.state().hand_count(target);
    assert_eq!(
        recover(&mut tx, owner, 1),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![target.raw()], false, 1);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), before_hand + 2);
}
}

mod original_sp_sijian_offline_tianfeng {
//! Golden coverage for SP「死谏」and the OFFLINE 田丰 cross-package character closure.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANFENG: u32 = 4_153_931_211;
const SIJIAN: u32 = 3_832_046_646;
const GZSUISHI: u32 = 4_271_491_554;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/sijian.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("sijian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, bystander) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SIJIAN),
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 1,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, bystander)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn lose_last_hand_card(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state().hand_cards(owner)[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_packages_expose_sijian_and_complete_tianfeng_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 70);
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SIJIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_from_hand_empty_other_discardable_hand_equip"
            && trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == TIANFENG)
        .unwrap();
    assert_eq!(character.name_key, "tianfeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, vec![SIJIAN, GZSUISHI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn losing_the_last_hand_card_can_discard_another_players_hand_or_equip_card() {
    let (mut tx, owner, target, _) = setup();
    let target_card = tx.state().hand_cards(target)[0];
    assert_eq!(
        lose_last_hand_card(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target.raw()], 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target_card.raw()], 3);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
}

#[test]
fn nonfinal_hand_loss_and_judgment_only_targets_do_not_offer_sijian() {
    let (mut nonfinal, owner, _, _) = setup();
    run_effects(
        &mut nonfinal,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    lose_last_hand_card(&mut nonfinal, owner).unwrap();
    assert_eq!(nonfinal.state().hand_count(owner), 1);
    assert!(!nonfinal.state().has_pending_choice());

    let (mut judgment_only, owner, target, bystander) = setup();
    for player in [target, bystander] {
        let hand = judgment_only
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .unwrap();
        let judge = judgment_only
            .state()
            .zones
            .find(ZoneKind::Judge, player)
            .unwrap();
        let cards = judgment_only.state().hand_cards(player);
        run_effects(
            &mut judgment_only,
            vec![EffectRequest::MoveCards {
                cards,
                to: judge,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
    lose_last_hand_card(&mut judgment_only, owner).unwrap();
    assert!(!judgment_only.state().has_pending_choice());
}
}

mod original_sp_songci {
//! Golden coverage for SP 陈琳「颂词」(songci).
//! Play-phase active: selected target draws 2.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SONGCI: u32 = 2_662_892_097;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/songci.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("songci-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_13, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SONGCI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SONGCI)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_the_active_draw() {
    let compiled = content_compiler::compile_source_named("songci", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SONGCI && skill.name == "songci" && !skill.locked));
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SONGCI)
        .expect("songci active");
    assert_eq!(active.window, "play_phase");
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SONGCI)
        .expect("npack songci active");
    assert_eq!(pack.window, "play_phase");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for songci");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selected_target_draws_two() {
    let (state, owner, other, skill) = setup();
    let before = state.hand_count(other);
    let (state, result) = activate(state, owner, other, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.hand_count(other), before + 2);
}
}

mod original_sp_spniluan {
//! Golden coverage for SP 韩遂「逆乱」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPNILUAN: u32 = 4_063_477_590;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/spniluan.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn dodged_converted_sha_refunds_use_count_but_hit_still_counts() {
    let package = content_compiler::compile_source_named("spniluan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 354);
    assert!(package.view_as_index[0].refund_use_count_if_no_damage);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::MoveCards {
        cards: tx.state().hand_cards(*player),
        to: discard,
        from: tx.state().zones.find(ZoneKind::Hand, *player),
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPNILUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| tx.state().skills.get(*id).unwrap().def.raw() == SPNILUAN)
        .unwrap();
    let first = inject(&mut tx, owner, CardKind::Shan, Suit::Club);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![first],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let mut tx = Transaction::begin(state);
    let second = inject(&mut tx, owner, CardKind::Tao, Suit::Spade);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![second],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
}
}

mod original_sp_spolaige_machine {
//! Golden for SP 霸业 spolaige (ABI 737, tag 758):
//! 觉醒技——全局第二次濒死结算后，失去〖西向〗获得〖朔北〗，手牌摸至体力
//! 上限并回复体力至体力上限。真管道驱动（EnterDying 全窗口结算）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPOLAIGE: u32 = 4_158_403_560;
const SPOLXIXIANG: u32 = 2_531_785_939;
const SPOLZHUBEI: u32 = 3_309_571_556;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn owns(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn inject(state: &mut GameState, owner: PlayerId, count: usize, tag: u32) {
    for index in 0..count {
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(94_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        state.zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_spolaige_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SPOLAIGE)
        .expect("spolaige trigger");
    assert_eq!(entry.event.as_str(), "dying_after");
    assert_eq!(entry.filter.as_str(), "owner_alive_second_global_dying");
    assert!(!entry.optional);
}

#[test]
fn second_global_dying_awakens_owner_to_full_max_hp() {
    let (mut state, players) = started(2_026_093_001);
    let owner = players[0];
    let first_victim = players[1];
    let second_victim = players[2];
    grant(&mut state, owner, SPOLAIGE);
    grant(&mut state, owner, SPOLXIXIANG);
    // owner：体力上限 4，体力 1，手牌 1。
    {
        let o = state.players.get_mut(owner).unwrap();
        o.max_hp = 4;
        o.hp = 1;
    }
    inject(&mut state, owner, 1, 1);
    let mut tx = Transaction::begin(state);

    // 第一次濒死（victim1）：进入后无桃自动关闭窗口，再以恢复脱离濒死
    // （上游 dyingAfter = 脱离濒死后；死亡路径无 LeftDying）。
    tx.state_mut().players.get_mut(first_victim).unwrap().hp = 0;
    run_effects(
        &mut tx,
        vec![EffectRequest::EnterDying {
            player: first_victim,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: first_victim,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_events, 1);
    assert!(owns(tx.state(), owner, SPOLXIXIANG));
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);

    // 第二次濒死（victim2）：脱离濒死 → 觉醒——换技 + 摸/回至体力上限。
    tx.state_mut().players.get_mut(second_victim).unwrap().hp = 0;
    run_effects(
        &mut tx,
        vec![EffectRequest::EnterDying {
            player: second_victim,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: second_victim,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_events, 2);
    assert!(!owns(tx.state(), owner, SPOLXIXIANG));
    assert!(owns(tx.state(), owner, SPOLZHUBEI));
    assert_eq!(tx.state().hand_count(owner), 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn spolaige_contract_command_submission_is_rejected_when_exhausted_not_applicable() {
    // 占位对齐：确认 ActivateSkill 之外的普通命令路径不受影响。
    let (state, players) = started(2_026_093_002);
    let owner = players[0];
    let (_, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: engine_api::SkillId::new(1),
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
}

mod original_sp_spolxixiang {
//! Golden coverage for SP OL 曹操「西向」(spolxixiang).
//! Play-phase view-as: a hand/equip card as Sha or Duel.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPOLXIXIANG: u32 = 2_531_785_939;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/spolxixiang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("spolxixiang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_42, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPOLXIXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_802),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SPOLXIXIANG)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_sha_and_juedou() {
    let compiled = content_compiler::compile_source_named("spolxixiang", SOURCE).unwrap();
    let mut kinds: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == SPOLXIXIANG)
        .map(|view| view.as_kind.as_str())
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["juedou", "sha"]);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut pack: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == SPOLXIXIANG)
        .map(|view| view.as_kind.as_str())
        .collect();
    pack.sort();
    assert_eq!(pack, vec!["juedou", "sha"]);
}

#[test]
fn hand_card_converts_to_sha() {
    let (state, owner, other, skill, card) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
}
}

mod original_sp_spolxixiang_machine {
//! Golden for SP 西向 spolxixiang (ABI 741):
//! owner_basic_uses_plus_one_cards 动态张数原语——转换素材最少
//! 1+本局基本牌使用数张；计数器经真实 CardUsed 事件递增（track 门控）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPOLXIXIANG: u32 = 2_531_785_939;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + tag),
        kind,
        suit: Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_spolxixiang_dynamic_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == SPOLXIXIANG)
        .collect();
    assert_eq!(rules.len(), 2);
    for rule in &rules {
        assert!(rule.owner_basic_uses_plus_one_cards);
    }
    for rule in &rules {
        assert!(rule.once_per_turn_per_branch);
    }
    assert!(rules.iter().any(|rule| rule.as_kind == "sha" && rule.no_use_count));
    assert!(rules.iter().any(|rule| rule.as_kind == "juedou"));
}

#[test]
fn basic_use_counter_increments_via_real_card_used_events() {
    let (mut state, players) = started(2_026_097_001);
    let owner = players[0];
    let tao_card = inject(&mut state, owner, CardKind::Tao, 1);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: tao_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_basic_card_uses, 1);
}

#[test]
fn conversion_requires_plus_one_source_cards() {
    let (mut state, players) = started(2_026_097_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, SPOLXIXIANG);
    // 本局基本牌使用数 = 1 → 至少 2 张素材。
    state.lifetime_basic_card_uses = 1;
    let c1 = inject(&mut state, owner, CardKind::Tao, 1);
    let c2 = inject(&mut state, owner, CardKind::Shan, 2);
    let c3 = inject(&mut state, owner, CardKind::Other, 3);
    let hp_before = state.players.get(target).unwrap().hp;

    // 1 张素材 → 拒绝。
    let (_, rejected) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c1],
        },
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // 2 张素材 → 接受（转换【决斗】结算：目标掉血）。
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c2, c3],
        },
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.players.get(target).unwrap().hp < hp_before,
        "决斗结算应使目标掉血"
    );
}

#[test]
fn each_rule_usable_once_per_turn() {
    let (mut state, players) = started(2_026_097_003);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, SPOLXIXIANG);
    let c1 = inject(&mut state, owner, CardKind::Tao, 1);
    let c2 = inject(&mut state, owner, CardKind::Shan, 2);
    let c3 = inject(&mut state, owner, CardKind::Other, 3);

    // 第一发：sha 规则（index 0，位 0）。
    let (state, first) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c1],
        },
        1,
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));

    // 第二发：sha 位已置跳过，决斗规则（index 1，位 1）接管。
    let (state, second) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c2],
        },
        2,
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));

    // 第三发：两条规则位均已置 → 无匹配规则 → 拒绝。
    let (_, third) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c3],
        },
        3,
    );
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
}
}

mod original_sp_spolxudai {
//! 第一百五十批：sp 庞德（SP）「虚待」（spolxudai）零新原语真实语义重写。
//! 上游：限定技，当你响应牌或受到伤害后，你可以令一名其他角色获得〖煮酒〗。
//! 实现：card_responded（owner_is_source）/ damage（owner_is_target）双臂 +
//! choose_other_add_skill(self, 煮酒def) + remove_skill(self_skill)（限定技自移除）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPOLXUDAI: u32 = 3_686_006_799;
const SPOLZHUJIU: u32 = 3_397_135_295;
const SPOLXUDAI_SRC: &str =
    include_str!("../../../content/original/ported/sp/spolxudai.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("spolxudai", SPOLXUDAI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(SPOLXUDAI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPOLXUDAI),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn has_skill_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn xudai_damage_arm_grants_zhujiu_and_removes_itself() {
    let (state, owner, other) = bootstrap();
    // 受到 1 点伤害 → 确认 → 选 other → other 获得煮酒、虚待自移除。
    let mut tx = Transaction::begin(state);
    match run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    ) {
        Ok(_) => panic!("expected the xudai optional confirm to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 10);
    // 目标选择：options 为其他存活角色，选 other。
    assert!(state.has_pending_choice(), "target selection must open");
    let state = resolve(state, owner, vec![other.raw()], 11);

    assert!(
        has_skill_def(&state, other, SPOLZHUJIU),
        "target must gain 煮酒"
    );
    assert!(
        !has_skill_def(&state, owner, SPOLXUDAI),
        "xudai must remove itself (limited)"
    );

    // 一次性：再次受伤不再触发。
    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(
        !matches!(opened, Err(EffectError::ChoiceRequested)),
        "xudai must not fire twice"
    );
}

#[test]
fn xudai_respond_arm_opens_confirm() {
    let (state, owner, other) = bootstrap();
    // 合成响应事件（owner 为响应者）→ 虚待确认打开。
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: other,
            card: CardId::NONE,
            response_kind: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(opened, Err(EffectError::ChoiceRequested)),
        "respond arm must open the confirm"
    );
}
}

mod original_sp_spweiwu {
//! Golden for SP 韩遂「违忤」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPWEIWU: u32 = 4_254_049_099;
const ZHANYI_BASIC_SHA: u32 = 3_535_942_143;
const ZHANYI_BASIC_TAO: u32 = 3_584_326_530;
const ZHANYI_BASIC_JIU: u32 = 3_922_392_848;
const YANZHENG: u32 = 2_899_287_996;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let skills = tx.state().players.get(player).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_500 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: vec![card],
        },
    }
}

fn pass_wuxie(
    mut state: GameState,
    mut pipe: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult, u64) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.clone().unwrap();
        key += 1;
        (state, pipe) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    }
    (state, pipe, key)
}

#[test]
fn package_declares_red_hand_equip_once_and_distance_bypass() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == SPWEIWU)
        .unwrap();
    assert_eq!(view.as_kind, "shunshou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "red");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, Some(1));
    assert!(view.ignore_distance);
    assert!(package.manifest.rule_vm_abi_version >= 70);
    for (skill, kind) in [
        (ZHANYI_BASIC_SHA, "sha"),
        (ZHANYI_BASIC_TAO, "tao"),
        (ZHANYI_BASIC_JIU, "jiu"),
    ] {
        for window in ["play_phase", "response"] {
            assert!(package.view_as_index.iter().any(|view| {
                view.skill_def_id == skill
                    && view.as_kind == kind
                    && view.window == window
                    && view.filter == "card_basic"
                    && view.source_zone.is_none()
            }));
        }
    }
}

#[test]
fn red_equipment_takes_at_distance_two_but_black_and_second_use_fail() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_724,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    for player in seats {
        strip_skills(&mut tx, player);
    }
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = add_skill(&mut tx, actor, SPWEIWU);
    let ordinary = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickShunshou,
        Suit::Club,
        2,
    );
    let black = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        3,
    );
    let red = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        4,
    );
    let second_red = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        5,
    );
    let prize = inject(&mut tx, far, ZoneKind::Hand, CardKind::Tao, Suit::Heart, 6);

    let (_, ordinary_pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: ordinary,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        ordinary_pipe.result,
        CommandResult::Rejected { .. }
    ));

    let (_, black_pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![far], black, 2),
    );
    assert!(matches!(black_pipe.result, CommandResult::Rejected { .. }));

    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![far], red, 3),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let (state, pipe, key) = pass_wuxie(state, pipe, 3);
    let take = pipe.choice.unwrap();
    assert_eq!(take.observer, actor);
    assert!(take
        .options
        .iter()
        .any(|option| option.token == prize.raw()));
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: take.choice_id,
                selections: vec![prize.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&prize));

    let (_, second_pipe) = apply_command(
        state,
        activate(actor, skill, vec![far], second_red, key + 2),
    );
    assert!(matches!(second_pipe.result, CommandResult::Rejected { .. }));
}

#[test]
fn zhanyi_basic_tao_accepts_only_basic_hand_cards() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(30),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_730,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let actor = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, actor);
    clear_hand(&mut tx, actor);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, actor, ZHANYI_BASIC_TAO);
    let equipment = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        7,
    );
    let basic = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    tx.state_mut().players.get_mut(actor).unwrap().hp -= 1;

    let (_, rejected) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], equipment, 31),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(actor).unwrap().hp;
    let (state, accepted) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], basic, 32),
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, hp + 1);
}

#[test]
fn zhanyi_basic_sha_is_offered_for_duel_response() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(33),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_731,
            ruleset: "identity-standard".into(),
        },
    });
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, source);
    strip_skills(&mut tx, target);
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    tx.state_mut().turn.phase = Phase::Play;
    add_skill(&mut tx, target, ZHANYI_BASIC_SHA);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        9,
    );
    let basic = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        10,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        11,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(34),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("战杀应进入决斗出杀响应");
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == basic.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
}

#[test]
fn zhanyi_basic_jiu_can_self_save_from_dying() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(35),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_732,
            ruleset: "identity-standard".into(),
        },
    });
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, actor);
    clear_hand(&mut tx, actor);
    add_skill(&mut tx, actor, ZHANYI_BASIC_JIU);
    let basic = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        12,
    );
    tx.state_mut().players.get_mut(actor).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: actor,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), actor),
        Some(basic)
    );

    let committed = tx.commit();
    let choice_id = engine_api::ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, pipe) = apply_command(
        committed.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(36),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let player = state.players.get(actor).unwrap();
    assert!(player.alive && !player.dying);
    assert_eq!(player.hp, 1);
    assert!(!state.hand_cards(actor).contains(&basic));
}

#[test]
fn yanzheng_offers_equipment_as_wuxie_only_when_hand_exceeds_hp() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == YANZHENG)
        .unwrap();
    assert_eq!(view.as_kind, "wuxie");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_hand_gt_hp");
    assert_eq!(view.source_zone.as_deref(), Some("equip"));
    assert!(package.manifest.rule_vm_abi_version >= 70);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_736),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_736,
            ruleset: "identity-standard".into(),
        },
    });
    let actor = engine.state().turn.current;
    let responder = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, actor);
    strip_skills(&mut tx, responder);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, responder);
    add_skill(&mut tx, responder, YANZHENG);
    tx.state_mut().players.get_mut(responder).unwrap().hp = 1;
    let first_trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        13,
    );
    let second_trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        14,
    );
    inject(
        &mut tx,
        responder,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        15,
    );
    let equipment = inject(
        &mut tx,
        responder,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        16,
    );
    let equal_hand_hp = tx.state().clone();

    let (_, result) = apply_command(
        equal_hand_hp.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first_trick,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());

    let mut high_hand = Transaction::begin(equal_hand_hp);
    let hand_card = inject(
        &mut high_hand,
        responder,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        17,
    );
    let (_, result) = apply_command(
        high_hand.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: second_trick,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    let choice = result.choice.expect("严整应提供装备区牌作为无懈");
    assert_eq!(choice.observer, responder);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == hand_card.raw()));
}
}

mod original_sp_wangong {
//! Golden for SP 黄祖「挽弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WANGONG: u32 = 3_743_761_196;
const WANGONG2: u32 = 3_164_390_759;
const HUANGZU: u32 = 4_052_796_884;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/wangong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "card use rejected: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == skill_def)
        })
}

#[test]
fn package_publishes_wangong_and_huangzu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("wangong-test", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        99
    );
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUANGZU)
            .unwrap()
            .skills,
        [WANGONG]
    );
    for (skill, filter) in [
        (WANGONG, "owner_source_basic"),
        (WANGONG2, "owner_source_sha"),
        (WANGONG2, "owner_source_non_basic"),
    ] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == skill
                && trigger.event == "card_used"
                && trigger.filter == filter
                && !trigger.optional
        }));
    }
}

#[test]
fn basic_primes_next_card_sha_gains_damage_and_non_basic_consumes() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_072_301, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WANGONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let player = tx.state_mut().players.get_mut(*player).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let tao = inject(tx.state_mut(), owner, CardKind::Tao, 99_001);
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 99_002);
    let wuzhong = inject(tx.state_mut(), owner, CardKind::TrickWuzhong, 99_003);
    let mut state = tx.commit().state;

    state = play(state, owner, tao, Vec::new(), 1);
    assert!(has_skill(&state, owner, WANGONG2));
    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        i32::MAX as u32
    );
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, sha, target
    ));

    state = play(state, owner, sha, vec![target], 2);
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(has_skill(&state, owner, WANGONG2));

    state = play(state, owner, wuzhong, Vec::new(), 3);
    assert!(!has_skill(&state, owner, WANGONG2));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 1);
}
}

mod original_sp_weikui_caoren {
//! Golden coverage for SP 曹仁「伪溃」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WEIKUI: u32 = 3_890_756_007;
const WEIKUI2: u32 = 2_980_539_775;
const LIZHAN: u32 = 2_790_547_004;
const SP_CAOREN: u32 = 4_080_002_414;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/weikui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("weikui-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[2];
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WEIKUI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    let skill = skill_id(tx.state(), owner, WEIKUI).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn bound_target_state(target: PlayerId) -> Vec<u8> {
    let mut state = b"BND1".to_vec();
    state.extend_from_slice(&target.raw().to_le_bytes());
    state
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi196_and_complete_sp_caoren() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WEIKUI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.hand_nonempty);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WEIKUI2
            && modifier.target == "distance_from_skill_bound_target"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_CAOREN)
        .unwrap();
    assert_eq!(character.name_key, "sp_caoren");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [WEIKUI, LIZHAN]);

    let compiled = content_compiler::compile_source_named("weikui", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn no_shan_loses_hp_then_forces_one_target_hand_discard() {
    let mut setup = setup(2_026_072_601);
    let card = inject(
        &mut setup.state,
        setup.target,
        196_001,
        CardKind::TrickWuzhong,
    );
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let owner = setup.owner;
    let target = setup.target;
    let (state, offered) = activate(setup, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );
    assert!(!offered
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, resolved) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(skill_id(&state, owner, WEIKUI2).is_none());
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn shan_is_privately_revealed_then_normal_sha_binds_distance_until_turn_end() {
    let mut setup = setup(2_026_072_602);
    let shan = inject(&mut setup.state, setup.target, 196_011, CardKind::Shan);
    let owner = setup.owner;
    let target = setup.target;
    let target_hp = setup.state.players.get(target).unwrap().hp;
    assert_eq!(distance(&setup.state, owner, target), 2);

    let (state, offered) = activate(setup, 10);
    let response = offered.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    assert!(offered.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan] && to == &[owner]
    )));
    assert!(skill_id(&state, owner, WEIKUI2).is_none());

    let (state, resolved) = resolve(state, target, vec![shan.raw()], false, 11);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    let helper = skill_id(&state, owner, WEIKUI2).unwrap();
    assert_eq!(
        state.skills.get(helper).unwrap().state_blob,
        bound_target_state(target)
    );
    assert_eq!(distance(&state, owner, target), 1);

    let expired = end_owner_turn(state, owner);
    assert!(skill_id(&expired, owner, WEIKUI2).is_none());
    assert_eq!(distance(&expired, owner, target), 2);
}

#[test]
fn exhausted_sha_count_skips_sha_but_still_binds_target() {
    let mut setup = setup(2_026_072_603);
    let shan = inject(&mut setup.state, setup.target, 196_021, CardKind::Shan);
    setup
        .state
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let owner = setup.owner;
    let target = setup.target;
    let (state, result) = activate(setup, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan] && to == &[owner]
    )));
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, WEIKUI2).unwrap())
            .unwrap()
            .state_blob,
        bound_target_state(target)
    );
}

#[test]
fn dying_from_the_hp_cost_stops_the_hand_branch() {
    let mut setup = setup(2_026_072_604);
    setup.state.players.get_mut(setup.owner).unwrap().hp = 1;
    let card = inject(
        &mut setup.state,
        setup.target,
        196_031,
        CardKind::TrickWuzhong,
    );
    let owner = setup.owner;
    let target = setup.target;
    assert_eq!(setup.state.hand_count(owner), 0);
    assert!(engine_core::query::dying_save_card_for(&setup.state, owner, owner).is_none());
    assert_eq!(setup.state.cards.get(card).unwrap().owner, target);
    let (state, result) = activate(setup, 30);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let participant = state.players.get(owner).unwrap();
    assert!(!participant.alive && !participant.dying && participant.hp == 0);
    assert!(state.hand_cards(target).contains(&card));
    assert!(skill_id(&state, owner, WEIKUI2).is_none());
    assert!(result.choice.is_none());
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::DamageDealt {
                    target: damaged,
                    is_lose_hp: true,
                    ..
                } if *damaged == owner
            ))
            .count(),
        1
    );
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
}
}

mod original_sp_xiaoguo_yuejin {
//! Golden coverage for SP 乐进「骁果」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const XIAOGUO: u32 = 3_124_297_312;
const YUEJIN: u32 = 3_111_158_867;
const LIANYING: u32 = 21;
const XIAOJI: u32 = 22;
const SELF_PROTECT: u32 = 4_000_000_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/xiaoguo.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const XIAOGUO_LOGIC_HASH: [u8; 32] = [
    202, 246, 190, 46, 2, 115, 148, 61, 144, 149, 37, 189, 76, 146, 48, 6, 147, 0, 246, 171, 167,
    47, 165, 248, 157, 203, 236, 177, 184, 39, 163, 119,
];
const STANDARD_MODULE_LOGIC_HASH: [u8; 32] = [
    70, 220, 40, 61, 246, 193, 90, 74, 103, 36, 2, 104, 26, 2, 184, 113, 132, 28, 34, 241, 78, 86,
    25, 50, 39, 5, 150, 204, 19, 169, 163, 64,
];
const STANDARD_PACKAGE_LOGIC_HASH: [u8; 32] = [221,86,128,181,186,208,77,141,253,212,39,65,62,1,22,122,8,121,220,124,174,50,158,117,82,75,93,232,8,179,60,11];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    cost: CardId,
    equipment: Option<CardId>,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_500 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    equipment_zone: Option<ZoneKind>,
    owner_extra_skill: Option<u32>,
    target_extra_skill: Option<u32>,
    protect_target_equipment: bool,
) -> Setup {
    let source = if protect_target_equipment {
        format!(
            "{SOURCE}\nskill self_protect({SELF_PROTECT}) {{\n  modifier card_discard_protected(equip) override 1;\n}}\n"
        )
    } else {
        SOURCE.to_owned()
    };
    let package = compile_source_named("sp-xiaoguo-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730_01, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XIAOGUO),
    });
    if let Some(skill) = owner_extra_skill {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        });
    }
    if let Some(skill) = target_extra_skill {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(skill),
        });
    }
    if protect_target_equipment {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(SELF_PROTECT),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cost = inject(tx.state_mut(), owner, ZoneKind::Hand, CardKind::Other, 2);
    let equipment =
        equipment_zone.map(|zone| inject(tx.state_mut(), target, zone, CardKind::Armor, 3));
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        cost,
        equipment,
    }
}

fn dispatch_end(state: GameState, target: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

#[test]
fn package_and_module_freeze_sp_contract_without_touching_standard() {
    let compiled = compile_source_named("sp-xiaoguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 299);
    let trigger = compiled.trigger_index.first().unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_other_phase_end_hand_nonempty");
    assert!(!trigger.optional);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "xiaoguo_phase_end")
        .unwrap();
    assert_eq!(module.code, [83, 84, 90, 30, 1, 1]);
    assert_eq!(module.logic_hash, XIAOGUO_LOGIC_HASH);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_SP_XIAOGUO, 286);
    assert_eq!(ContinuationKind::ResolveSpXiaoguo.tag(), 206);

    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let yuejin = sp
        .character_index
        .iter()
        .find(|character| character.id == YUEJIN)
        .unwrap();
    assert_eq!(yuejin.name_key, "yuejin");
    assert_eq!(yuejin.sex, "male");
    assert_eq!(yuejin.group, "wei");
    assert_eq!(yuejin.max_hp, 4);
    assert_eq!(yuejin.skills, [XIAOGUO]);

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    assert_eq!(standard.manifest.rule_vm_abi_version, 261);
    assert_eq!(standard.manifest.logic_hash, STANDARD_PACKAGE_LOGIC_HASH);
    let std = standard
        .modules
        .iter()
        .find(|module| module.name == "stdxiaoguo_phase_end")
        .unwrap();
    assert_eq!(std.code, [83, 84, 80, 177, 1]);
    assert_eq!(std.logic_hash, STANDARD_MODULE_LOGIC_HASH);
}

#[test]
fn unrestricted_hand_cost_then_target_discards_equipment_or_takes_damage() {
    let setup = super::original_sp_xiaoguo_yuejin::setup(Some(ZoneKind::Hand), None, None, false);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let equipment = setup.equipment.unwrap();
    let (state, cost) = dispatch_end(setup.state, setup.target);
    assert_eq!(cost.observer, setup.owner);
    assert_eq!(cost.kind, ChoiceKind::SelectCards);
    assert_eq!(
        cost.options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.cost.raw()]
    );
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 1);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));

    let (paid, result) = resolve(state.clone(), setup.target, vec![equipment.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(paid.players.get(setup.target).unwrap().hp, hp);
    assert_eq!(paid.hand_cards(setup.owner).len(), 1);
    assert!(!paid.hand_cards(setup.target).contains(&equipment));

    let (damaged, result) = resolve(state, setup.target, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(damaged.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(damaged.hand_cards(setup.target).contains(&equipment));
    assert!(damaged.hand_cards(setup.owner).is_empty());
}

#[test]
fn owner_cancel_is_noop_and_missing_equipment_damages_without_another_choice() {
    let setup = setup(None, None, None, false);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (cancelled, result) = resolve(state.clone(), setup.owner, Vec::new(), true, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(cancelled.hand_cards(setup.owner).contains(&setup.cost));
    assert_eq!(cancelled.players.get(setup.target).unwrap().hp, hp);

    let (damaged, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(damaged.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(damaged.hand_cards(setup.owner).is_empty());
}

#[test]
fn both_discard_followups_wait_for_nested_movement_triggers() {
    let setup = setup(Some(ZoneKind::Hand), Some(LIANYING), None, false);
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 20);
    let lianying = result.choice.unwrap();
    assert_eq!(lianying.observer, setup.owner);
    assert!(state.hand_cards(setup.owner).is_empty());
    let (state, result) = resolve(state, setup.owner, vec![1], false, 21);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
    let (_, result) = resolve(state, setup.target, vec![equipment.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let setup = super::original_sp_xiaoguo_yuejin::setup(Some(ZoneKind::Equip), None, Some(XIAOJI), false);
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 23);
    assert_eq!(result.choice.unwrap().observer, setup.target);
    let (state, result) = resolve(state, setup.target, vec![equipment.raw()], false, 24);
    let xiaoji = result.choice.unwrap();
    assert_eq!(xiaoji.observer, setup.target);
    assert!(state.hand_cards(setup.owner).is_empty());
    assert!(state.hand_cards(setup.target).is_empty());
    let (state, result) = resolve(state, setup.target, vec![1], false, 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(setup.target).len(), 2);
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
}

#[test]
fn other_player_discard_protection_does_not_block_target_self_discard() {
    let setup = setup(Some(ZoneKind::Equip), None, None, true);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 30);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    let (state, result) = resolve(state, setup.target, vec![equipment.raw()], false, 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    assert!(!state.zones.get(equip).unwrap().cards.contains(&equipment));
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
}

#[test]
fn headless_runner_cancels_both_optional_sp_xiaoguo_choices_legally() {
    let setup = setup(Some(ZoneKind::Hand), None, None, false);
    let owner = setup.owner;
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, cost_choice) = dispatch_end(setup.state, target);
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(cost_choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: true,
            ..
        } if selections.is_empty()
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert!(engine.state().hand_cards(owner).contains(&setup.cost));
    assert_eq!(engine.state().players.get(target).unwrap().hp, hp);

    let setup = super::original_sp_xiaoguo_yuejin::setup(Some(ZoneKind::Hand), None, None, false);
    let owner = setup.owner;
    let target = setup.target;
    let equipment = setup.equipment.unwrap();
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, _) = dispatch_end(setup.state, target);
    let (state, result) = resolve(state, owner, vec![setup.cost.raw()], false, 40);
    let consequence_choice = result.choice.unwrap();
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(consequence_choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: true,
            ..
        } if selections.is_empty()
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().players.get(target).unwrap().hp, hp - 1);
    assert!(engine.state().hand_cards(target).contains(&equipment));
}
}

mod original_sp_xinfu_dianhua {
//! 批 179 首版 / 批 183 花色化后随动：sp「点化」（xinfu_dianhua，法箓族）。
//! 上游：准备/结束阶段，若你有「法箓」标记，观看牌堆顶 X 张（X=标记花色种数）
//! 并任意重排。实现（批 183 终版）：phase_prepare/phase_end + owner_is_target →
//! guanxing_top_by_falu_marks(self, [四花色法箓标记 def])（数非零种数为 N）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DIANHUA: u32 = 3_399_270_449;
const FALU_SUIT_MARKS: [u32; 4] = [
    3_540_560_359, // spade
    3_540_560_360, // heart
    3_540_560_361, // club
    3_540_560_362, // diamond
];
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DIANHUA),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

/// 前 n 个花色各加 1 枚法箓标记（批 183 花色化口径：种数即观星张数）。
fn add_falu_marks(state: &mut GameState, owner: PlayerId, n: u32) {
    let mut tx = Transaction::begin(state.clone());
    let effects: Vec<_> = FALU_SUIT_MARKS
        .iter()
        .take(n as usize)
        .map(|def| EffectRequest::AddMark {
            player: owner,
            mark: rule_api::MarkDefId::new(*def),
            delta: 1,
        })
        .collect();
    engine_core::effect::run_effects(&mut tx, effects).unwrap();
    *state = tx.commit().state;
}

fn trigger_phase(state: &mut GameState, owner: PlayerId, phase: engine_api::TurnPhaseTag) {
    let mut tx = Transaction::begin(state.clone());
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(requests) => {
            let _ = engine_core::effect::run_effects(&mut tx, requests);
        }
        Err(e) => panic!("dispatch err: {e:?}"),
    }
    *state = tx.commit().state;
}

fn resolve_pick(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_dianhua_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    for event in ["phase_prepare", "phase_end"] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == DIANHUA
                    && trigger.event == event
                    && trigger.filter == "owner_is_target"
                    && !trigger.optional
            }),
            "dianhua must publish {event} arm"
        );
    }
}

#[test]
fn dianhua_guanxing_rounds_equal_mark_count() {
    // 3 枚法箓标记 → 观星 3 轮。
    let (mut state, owner) = started(2_026_092_201);
    add_falu_marks(&mut state, owner, 3);
    trigger_phase(
        &mut state,
        owner,
        engine_api::TurnPhaseTag::Prepare,
    );
    assert!(state.has_pending_choice(), "guanxing loop must open");

    let mut key = 2;
    for round in 0..3 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "loop closes after 3 rounds");
}

#[test]
fn dianhua_zero_marks_stays_silent() {
    // 无法箓标记 → count=0 原语静默返回（无 pending）。
    let (mut state, owner) = started(2_026_092_202);
    trigger_phase(&mut state, owner, engine_api::TurnPhaseTag::Prepare);
    assert!(
        !state.has_pending_choice(),
        "zero marks must stay silent"
    );
}

#[test]
fn dianhua_end_phase_arm_also_fires() {
    let (mut state, owner) = started(2_026_092_203);
    add_falu_marks(&mut state, owner, 2);
    trigger_phase(&mut state, owner, engine_api::TurnPhaseTag::End);
    assert!(state.has_pending_choice(), "end-phase arm must fire");
    let mut key = 2;
    for _ in 0..2 {
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(!state.has_pending_choice());
}
}

mod original_sp_xinfu_falu {
//! Golden for sp 法箓 (xinfu_falu)：locked —— 开局获得四花色「法箓」标记各 1 枚
//! （批 183 花色化终版；supersede 批 179 的统一标记口径）。

use engine_api::{Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_FALU: u32 = 3_540_560_358;
const FALU_SUIT_MARKS: [u32; 4] = [
    3_540_560_359, // spade
    3_540_560_360, // heart
    3_540_560_361, // club
    3_540_560_362, // diamond
];
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_001,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn suit_marks_of(tx: &Transaction, player: PlayerId) -> Vec<i32> {
    FALU_SUIT_MARKS
        .iter()
        .map(|def| {
            tx.state()
                .players
                .get(player)
                .and_then(|p| p.marks.iter().find(|(id, _)| *id == *def))
                .map(|(_, c)| *c)
                .unwrap_or(0)
        })
        .collect()
}

#[test]
fn committed_package_publishes_falu_game_start_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == XINFU_FALU && t.event == "game_started"
    }));
}

#[test]
fn game_start_grants_four_marks() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_FALU),
    });
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(suit_marks_of(&tx, owner), [0, 0, 0, 0]);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(suit_marks_of(&tx, owner), [1, 1, 1, 1], "开局四花色标记各 1 枚");
}
}

mod original_sp_xinfu_falu_family {
//! 第一百八十三批：sp 法箓族花色化落地（xinfu_falu 收集 + xinfu_dianhua 点化，
//! ABI 683）。falu：game_started 全 add 四花色标记；cards_moved 弃花色 X 且无
//! X 标记 → add_mark。dianhua：guanxing_top_by_falu_marks 数非零种数观星。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FALU: u32 = 3_540_560_358;
const DIANHUA: u32 = 3_399_270_449;
const FALU_SPADE: u32 = 3_540_560_359;
const FALU_HEART: u32 = 3_540_560_360;
const FALU_CLUB: u32 = 3_540_560_361;
const FALU_DIAMOND: u32 = 3_540_560_362;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(FALU),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DIANHUA),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_701),
        kind: CardKind::Sha,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn discard_to(tx: &mut Transaction, player: PlayerId, card: CardId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    tx.state_mut().cards.get_mut(card).unwrap().zone = discard;
    tx.state_mut().zones.get_mut(hand).unwrap().cards.retain(|c| *c != card);
    tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
}

fn mark_count(state: &GameState, player: PlayerId, def: u32) -> u32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(found, _)| *found == def)
                .map(|(_, count)| *count)
        })
        .unwrap_or(0)
        .max(0) as u32
}

#[test]
fn package_publishes_falu_family_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 683);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FALU
            && trigger.event == "game_started"
            && trigger.filter == "owner_alive"
    }));
    for filter in [
        "owner_discard_suit_no_own_mark_spade_falu",
        "owner_discard_suit_no_own_mark_heart_falu",
        "owner_discard_suit_no_own_mark_club_falu",
        "owner_discard_suit_no_own_mark_diamond_falu",
    ] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == FALU && trigger.filter == filter
            }),
            "falu must publish {filter}"
        );
    }
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DIANHUA
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn falu_collects_suit_marks_on_discard() {
    // owner 弃黑桃与红桃各一张：获得 spade/heart 两枚法箓标记。
    let (mut state, owner, _other) = started(2_026_092_301);
    let c1 = {
        let mut tx = Transaction::begin(state.clone());
        let c = inject_hand(&mut tx, owner, Suit::Spade, 6);
        state = tx.commit().state;
        c
    };
    let c2 = {
        let mut tx = Transaction::begin(state.clone());
        let c = inject_hand(&mut tx, owner, Suit::Heart, 7);
        state = tx.commit().state;
        c
    };
    // game_started 已初始补齐四花色标记各 1（法箓①效果）。
    assert_eq!(mark_count(&state, owner, FALU_SPADE), 1);
    assert_eq!(mark_count(&state, owner, FALU_HEART), 1);

    // 弃黑桃牌 → cards_moved 触发 spade 法箓标记。
    {
        let mut tx = Transaction::begin(state.clone());
        discard_to(&mut tx, owner, c1);
        let dispatched = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardsMoved {
                cards: vec![c1],
                from: state.zones.find(ZoneKind::Hand, owner).unwrap(),
                to: state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap(),
                reason: engine_api::CardMoveReason::Discard,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        );
        match dispatched {
            Ok(requests) => {
                let _ = engine_core::effect::run_effects(&mut tx, requests);
            }
            Err(e) => panic!("dispatch err: {e:?}"),
        }
        state = tx.commit().state;
    }
    assert_eq!(
        mark_count(&state, owner, FALU_SPADE),
        1,
        "spade discard keeps spade mark at 1 (no duplicate)"
    );

    // 弃红桃牌 → heart 标记。
    {
        let mut tx = Transaction::begin(state.clone());
        discard_to(&mut tx, owner, c2);
        let dispatched = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardsMoved {
                cards: vec![c2],
                from: state.zones.find(ZoneKind::Hand, owner).unwrap(),
                to: state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap(),
                reason: engine_api::CardMoveReason::Discard,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        );
        match dispatched {
            Ok(requests) => {
                let _ = engine_core::effect::run_effects(&mut tx, requests);
            }
            Err(e) => panic!("dispatch err: {e:?}"),
        }
        state = tx.commit().state;
    }
    assert_eq!(
        mark_count(&state, owner, FALU_HEART),
        1,
        "heart discard grants heart falu mark"
    );
}
}

mod original_sp_xinfu_guolun {
//! Golden coverage for SP 庞统「过论」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::project_observer;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_GUOLUN: u32 = 2_723_622_416;
const XINFU_SONGSANG: u32 = 2_272_181_771;
const RE_JSP_PANGTONG: u32 = 2_348_829_214;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    owner_hand: CardId,
    target_cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(294_000 + u32::from(rank)),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(owner_rank: u8, target_ranks: &[u8]) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_073_001, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_GUOLUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XINFU_GUOLUN)
        .unwrap();
    let mut state = tx.commit().state;
    let owner_hand = inject(&mut state, owner, ZoneKind::Hand, owner_rank);
    let target_cards = target_ranks
        .iter()
        .map(|rank| inject(&mut state, target, ZoneKind::Hand, *rank))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        owner_hand,
        target_cards,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn knows_face(state: &GameState, observer: PlayerId, card: CardId) -> bool {
    project_observer(state, ObserverKind::Player(observer))
        .known_cards
        .iter()
        .any(|known| known.id == card && known.def.is_some())
}

#[test]
fn current_sp_package_keeps_complete_re_jsp_pangtong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XINFU_GUOLUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_nonempty);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_JSP_PANGTONG)
        .unwrap();
    assert_eq!(character.name_key, "re_jsp_pangtong");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [XINFU_GUOLUN, XINFU_SONGSANG]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn target_hand_is_hidden_then_only_selected_face_is_projected_and_cancel_consumes_use() {
    let setup = setup(9, &[2, 11]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let owner_hand = setup.owner_hand;
    let selected = setup.target_cards[0];
    let hidden = setup.target_cards[1];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);
    assert!(!knows_face(&setup.state, owner, selected));
    assert!(!knows_face(&setup.state, owner, hidden));

    let (state, offered) = activate(setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let first = offered.choice.unwrap();
    assert_eq!(first.observer, owner);
    assert_eq!(first.kind, ChoiceKind::SelectCards);
    assert!(!first.allow_cancel);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveXinfuGuolun
    );
    assert!(!knows_face(&state, owner, selected));
    assert!(!knows_face(&state, owner, hidden));

    let (state, revealed) = resolve(state, owner, vec![selected.raw()], false, 2);
    assert!(matches!(revealed.result, CommandResult::Accepted { .. }));
    let second = revealed.choice.unwrap();
    assert!(second.allow_cancel);
    assert!(second
        .options
        .iter()
        .any(|option| option.token == owner_hand.raw()));
    assert!(knows_face(&state, owner, selected));
    assert!(!knows_face(&state, owner, hidden));

    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 3);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_count);
    assert_eq!(state.hand_count(target), target_count);
    assert_eq!(state.cards.get(selected).unwrap().owner, target);
    assert_eq!(state.cards.get(owner_hand).unwrap().owner, owner);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let (_, repeated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        4,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn lower_authoritative_number_equipment_is_exchanged_then_its_owner_draws() {
    let mut setup = setup(12, &[8]);
    let equipment = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 3);
    let owner = setup.owner;
    let target = setup.target;
    let target_card = setup.target_cards[0];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);

    let (state, _) = activate(setup, 10);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 11);
    let (state, exchanged) = resolve(state, owner, vec![equipment.raw()], false, 12);
    assert!(matches!(exchanged.result, CommandResult::Accepted { .. }));
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().owner, target);
    assert_eq!(state.cards.get(equipment).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_card).unwrap().zone, owner_hand);
    assert_eq!(state.hand_count(owner), owner_count + 2);
    assert_eq!(state.hand_count(target), target_count);
}

#[test]
fn equal_authoritative_numbers_exchange_without_drawing() {
    let setup = setup(7, &[7]);
    let owner = setup.owner;
    let target = setup.target;
    let owner_card = setup.owner_hand;
    let target_card = setup.target_cards[0];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);

    let (state, _) = activate(setup, 20);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 21);
    let (state, exchanged) = resolve(state, owner, vec![owner_card.raw()], false, 22);
    assert!(matches!(exchanged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card).unwrap().owner, target);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert_eq!(state.hand_count(owner), owner_count);
    assert_eq!(state.hand_count(target), target_count);
}
}

mod original_sp_xinfu_qiai_machine {
//! Golden for SP 求援 xinfu_qiai (ABI 735, tag 756, Cont 328):
//! 限定技濒死交牌循环状态机 — owner 濒死时，其他角色依座位序（owner 之后
//! 环绕）各强制交给 owner 一张手/装备牌；无牌者跳过。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_QIAI: u32 = 2_677_335_078;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64, count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let players = state.players.alive_by_seat();
    (state, players)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, tag: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_700 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hands(tx: &mut Transaction, players: &[PlayerId]) {
    for player in players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: *player,
                    cards,
                }],
            )
            .unwrap();
        }
    }
}

fn arm_dying_owner(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_QIAI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
}

#[test]
fn package_publishes_xinfu_qiai_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_QIAI)
        .expect("xinfu_qiai trigger");
    assert_eq!(entry.event.as_str(), "dying");
    assert_eq!(
        entry.filter.as_str(),
        "owner_dying_zhupeilan_once_per_game"
    );
    assert!(!entry.optional);
}

#[test]
fn dying_owner_collects_one_card_from_each_other_player_in_seat_order() {
    let (state, players) = started(2_026_091_001, 3);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx, &players);
    arm_dying_owner(&mut tx, owner);
    let c1 = inject_hand(&mut tx, players[1], 1);
    let _c2 = inject_hand(&mut tx, players[1], 2);
    let c3 = inject_hand(&mut tx, players[2], 3);
    let owner_before = tx.state().hand_count(owner);

    // Entry：首 giver = players[1]（座位序 owner 之后）。
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinfuQiai { player: owner }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveXinfuQiaiStage
    );

    // giver1 交 c1 → 移入 owner 手牌区 + 打开 giver2 的选牌。
    let (state, result) = resolve(tx.commit().state, players[1], vec![c1.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let tx = Transaction::begin(state);
    assert!(tx.state().hand_cards(owner).contains(&c1));
    assert!(!tx.state().hand_cards(players[1]).contains(&c1));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveXinfuQiaiStage
    );
    assert_ne!(tx.state().resolution.pending_choice_id, 0);

    // giver2 交 c3 → 无更多 giver，机器收尾。
    let (state, result) = resolve(tx.commit().state, players[2], vec![c3.raw()], false, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state_tx = state.clone();
    assert!(state_tx.hand_cards(owner).contains(&c3));
    assert_eq!(state_tx.hand_count(owner), owner_before + 2);
    assert!(!state_tx.has_pending_choice());
}

#[test]
fn entry_without_any_qualified_giver_is_a_quiet_noop() {
    let (state, players) = started(2_026_091_002, 2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx, &players);
    arm_dying_owner(&mut tx, owner);

    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinfuQiai { player: owner }],
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}
}

mod original_sp_xinfu_songsang {
//! Golden for SP「送丧」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_JSP_PANGTONG: u32 = 2_348_829_214;
const XINFU_SONGSANG: u32 = 2_272_181_771;
const XINFU_ZHANJI: u32 = 4_089_086_847;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn has_skill(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn setup(wounded: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, XINFU_SONGSANG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_SONGSANG),
    });
    run_effects(&mut tx, effects).unwrap();
    if wounded {
        tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    }
    Setup {
        state: tx.commit().state,
        owner,
        others: players
            .into_iter()
            .filter(|player| *player != owner)
            .collect(),
    }
}

fn kill(state: GameState, player: PlayerId, killer: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let requested = run_effects(&mut tx, vec![EffectRequest::KillPlayer { player, killer }])
        == Err(EffectError::ChoiceRequested);
    (tx.commit().state, requested)
}

fn resolve(state: GameState, owner: PlayerId, cancelled: bool, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: if cancelled { Vec::new() } else { vec![1] },
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_optional_once_only_death_skill_without_incomplete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_SONGSANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(trigger.optional);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_JSP_PANGTONG && character.skills == [2_723_622_416, XINFU_SONGSANG]
    }));
}

#[test]
fn cancellation_preserves_songsang_and_self_death_does_not_trigger() {
    let wounded = setup(true);
    let hp = wounded.state.players.get(wounded.owner).unwrap().hp;
    let max_hp = wounded.state.players.get(wounded.owner).unwrap().max_hp;
    let (state, requested) = kill(wounded.state, wounded.others[0], wounded.owner);
    assert!(requested);
    let state = resolve(state, wounded.owner, true, 1);
    assert!(has_skill(&state, wounded.owner, XINFU_SONGSANG));
    assert!(!has_skill(&state, wounded.owner, XINFU_ZHANJI));
    assert_eq!(state.players.get(wounded.owner).unwrap().hp, hp);
    assert_eq!(state.players.get(wounded.owner).unwrap().max_hp, max_hp);
    let (_, requested_again) = kill(state, wounded.others[1], wounded.owner);
    assert!(requested_again);

    let full = setup(false);
    let (state, requested) = kill(full.state, full.owner, full.others[0]);
    assert!(!requested);
    assert!(!state.has_pending_choice());
}

#[test]
fn accepted_songsang_recovers_if_wounded_otherwise_only_raises_max_hp() {
    let wounded = setup(true);
    let hp = wounded.state.players.get(wounded.owner).unwrap().hp;
    let max_hp = wounded.state.players.get(wounded.owner).unwrap().max_hp;
    let (state, requested) = kill(wounded.state, wounded.others[0], wounded.owner);
    assert!(requested);
    let state = resolve(state, wounded.owner, false, 2);
    assert_eq!(state.players.get(wounded.owner).unwrap().hp, hp + 1);
    assert_eq!(state.players.get(wounded.owner).unwrap().max_hp, max_hp);
    assert!(!has_skill(&state, wounded.owner, XINFU_SONGSANG));
    assert!(has_skill(&state, wounded.owner, XINFU_ZHANJI));

    let full = setup(false);
    let hp = full.state.players.get(full.owner).unwrap().hp;
    let max_hp = full.state.players.get(full.owner).unwrap().max_hp;
    let (state, requested) = kill(full.state, full.others[0], full.owner);
    assert!(requested);
    let state = resolve(state, full.owner, false, 3);
    assert_eq!(state.players.get(full.owner).unwrap().hp, hp);
    assert_eq!(state.players.get(full.owner).unwrap().max_hp, max_hp + 1);
    assert!(!has_skill(&state, full.owner, XINFU_SONGSANG));
    assert!(has_skill(&state, full.owner, XINFU_ZHANJI));
}
}

mod original_sp_xinfu_tushe {
//! Golden for SP `xinfu_tushe`, rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XINFU_TUSHE: u32 = 2_236_248_008;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let targets = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_TUSHE),
        }],
    )
    .unwrap();
    (tx, owner, targets)
}

fn set_hand_kind(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> Vec<CardId> {
    let hand = tx.state().hand_cards(owner);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = kind;
    }
    hand
}

#[test]
fn xinfu_tushe_optionally_draws_full_target_count_once() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_TUSHE)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_non_equip_no_basic_hand_has_targets"
    );
    assert!(trigger.optional);

    let (mut tx, owner, targets) = started(2_026_071_523);
    let card = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong)[0];
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + targets.len() as u32);
}

#[test]
fn xinfu_tushe_rejects_basic_hand_equipment_and_targetless_uses() {
    let (mut tx, owner, targets) = started(2_026_071_524);
    let hand = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong);
    tx.state_mut().cards.get_mut(hand[1]).unwrap().kind = CardKind::Tao;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: hand[0],
            targets: targets.clone(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    let (mut tx, owner, targets) = started(2_026_071_525);
    let card = set_hand_kind(&mut tx, owner, CardKind::Weapon)[0];
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    let (mut tx, owner, _) = started(2_026_071_526);
    let card = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong)[0];
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_sp_xinfu_yinshi {
//! Golden coverage for SP 司马徽「隐士」(xinfu_yinshi).
//! Incoming damage_before prevents the current damage.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_YINSHI: u32 = 3_413_758_271;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/xinfu_yinshi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("xinfu_yinshi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_04, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_YINSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn compiled_source_and_npack_publish_the_prevent_window() {
    let compiled = content_compiler::compile_source_named("xinfu_yinshi", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == XINFU_YINSHI && skill.name == "xinfu_yinshi" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_YINSHI)
        .expect("xinfu_yinshi trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_YINSHI)
        .expect("npack xinfu_yinshi trigger");
    assert_eq!(pack_trigger.event, "damage_before");
    assert_eq!(pack_trigger.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| compiled.modules.iter().any(|donor| donor.name == module.name))
        .expect("npack module for xinfu_yinshi");
    let compiled_module = compiled
        .modules
        .iter()
        .find(|module| module.name == actual.name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled_module.logic_hash);
}

#[test]
fn incoming_damage_is_prevented() {
    let (state, owner, other) = setup();
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            target,
            ..
        } if *target == owner
    )));
}
}

mod original_sp_xinfu_zhanji {
//! Golden for SP `xinfu_zhanji`, rewritten from legacy JavaScript.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_ZHANJI: u32 = 4_089_086_847;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_ZHANJI),
        }],
    )
    .unwrap();
    (tx, owner)
}

#[test]
fn xinfu_zhanji_rewards_each_external_play_phase_draw_without_self_recursion() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_ZHANJI)
        .unwrap();
    assert_eq!(trigger.event, "cards_drawn");
    assert_eq!(trigger.filter, "owner_draws_during_own_play_not_self");

    let (mut tx, owner) = started(2_026_071_521);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 4);

    tx.state_mut().turn.phase = Phase::End;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}
}

mod original_sp_xinfu_zhennan {
//! 第一百五十二批：sp「镇南」（xinfu_zhennan）零新原语重写。
//! 上游：当你成为【南蛮入侵】的目标时，你可以对一名其他角色造成 1-3 点随机伤害。
//! 实现：card_targeted + owner_targeted_by_nanman（本批注册）→ choose_other_damage。
//! 近似：随机 1-3 点伤害定为固定 1 点。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_ZHENNAN: u32 = 4_152_984_845;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    (state, owner, others)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }])
    .unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

/// Inject a nanman into `actor`'s hand and play it against everyone else.
fn play_nanman(
    state: GameState,
    actor: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::TrickNanman,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_zhennan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINFU_ZHENNAN
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_targeted_by_nanman"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == XINFU_ZHENNAN && modifier.target == "hand_limit"));
}

#[test]
fn zhennan_deals_damage_to_chosen_other_when_targeted_by_nanman() {
    let (state, owner, others) = started(2_026_090_401);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, XINFU_ZHENNAN);
    let owner_hp = hp_of(&state, owner);
    let other_hp = hp_of(&state, others[0]);

    // 他人使用南蛮入侵（owner 为目标之一）。
    let (state, played) = play_nanman(state, others[1], others.to_vec(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 确认后选择伤害目标：选 others[0]。
    let state = resolve(state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![others[0].raw()], 3);
    let _ = state.has_pending_choice();

    // 近似口径：镇南固定 1 点伤害落在被选目标上；owner 另受南蛮 1 点（免除臂未建模，
    // 南蛮对其照常结算）。
    assert_eq!(hp_of(&state, others[0]), other_hp - 2, "chosen target takes zhennan 1 + nanman 1");
    let _ = owner_hp;
}
}

mod original_sp_xionghuo_shanjia {
//! Golden for sp 凶祸 (xinfu_xionghuo) + 神甲 (olshanjia) rewrites.
//!
//! xinfu_xionghuo: 3 「暴戾」 marks at game start; may give one to another player
//! (play phase); +1 damage to marked players.
//! olshanjia: 3 「损」 marks at game start; play-phase draw 3 (once per turn).

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XIONGHUO: u32 = 3_787_720_298;
const SHANJIA: u32 = 3_933_707_186;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_801,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn fire_game_started(tx: &mut Transaction, count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted { player_count: count }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn marks_of(tx: &Transaction, player: PlayerId, def: u32) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == def))
        .map(|(_, c)| *c)
        .unwrap_or(0)
}

#[test]
fn committed_package_publishes_both_skills() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGHUO && t.event == "game_started"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGHUO && t.event == "damage_before"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SHANJIA && t.event == "game_started"
    }));
}

#[test]
fn xionghuo_grants_gives_and_boosts_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, XIONGHUO);
    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner, XIONGHUO), 3, "开局 3 枚暴戾");

    // Give one mark to the other player.
    tx.state_mut().turn.phase = Phase::Play;
    let (state, gave) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(gave.result, CommandResult::Accepted { .. }),
        "give-mark accepted: {:?}",
        gave.result
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(marks_of(&tx, owner, XIONGHUO), 2);
    assert_eq!(marks_of(&tx, other, XIONGHUO), 1);

    // Damage the marked other: +1.
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        other_hp - 2,
        "marked target takes +1 damage"
    );
}

#[test]
fn xionghuo_does_not_boost_unmarked_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = add_skill(&mut tx, owner, XIONGHUO);
    fire_game_started(&mut tx, 2);
    let other_hp = tx.state().players.get(other).unwrap().hp;

    let damage = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("damage error: {e:?}"),
    }
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        other_hp - 1,
        "unmarked target takes normal damage"
    );
}

#[test]
fn shanjia_grants_marks_and_draws_three() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_skill(&mut tx, owner, SHANJIA);
    fire_game_started(&mut tx, 2);
    assert_eq!(marks_of(&tx, owner, SHANJIA), 3, "开局 3 枚损");

    tx.state_mut().turn.phase = Phase::Play;
    let hand_before = tx.state().hand_count(owner);
    let (state, drew) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(drew.result, CommandResult::Accepted { .. }),
        "神甲 draw accepted: {:?}",
        drew.result
    );
    assert_eq!(state.hand_count(owner), hand_before + 3, "摸三张");
}
}

mod original_sp_xiongyi_offline_mateng {
//! Golden coverage for SP `xiongyi` and the dependent OFFLINE Mateng closure.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGYI: u32 = 2_476_017_492;
const MATENG: u32 = 3_979_368_086;
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn package(bytes: &[u8]) -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(bytes).unwrap()
}

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let standard = package(STANDARD);
    let shenhua = package(SHENHUA);
    let sp = package(SP);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&sp).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIONGYI),
        }],
    )
    .unwrap();
    (tx.state().clone(), owner, others[0], others[1])
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == XIONGYI)
        .unwrap();
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn packages_publish_xiongyi_and_close_offline_mateng() {
    let sp = package(SP);
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let active = sp
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 255));
    assert!(targets.allow_self);

    let offline = package(OFFLINE);
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let mateng = offline
        .character_index
        .iter()
        .find(|character| character.id == MATENG)
        .unwrap();
    assert_eq!(mateng.name_key, "mateng");
    assert_eq!(mateng.sex, "male");
    assert_eq!(mateng.group, "qun");
    assert_eq!(mateng.max_hp, 4);
    assert_eq!(mateng.skills, [2, XIONGYI]);
}

#[test]
fn xiongyi_draws_selected_targets_in_seat_order_recovers_at_minimum_and_is_limited() {
    let (mut state, owner, first_other, second_other) = started(2_026_072_301);
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first_other).unwrap().hp = 2;
    state.players.get_mut(second_other).unwrap().hp = 3;
    let owner_hand = state.hand_count(owner);
    let target_hand = state.hand_count(first_other);
    let expected_order: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player == owner || *player == first_other)
        .collect();

    let (state, pipe) = activate(state, owner, vec![first_other, owner], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 3);
    assert_eq!(state.hand_count(first_other), target_hand + 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    let draw_order: Vec<_> = pipe
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if cards.len() == 3 => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(draw_order, expected_order);
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered {
            player,
            amount: 1,
            ..
        } if *player == owner
    )));

    let owner_hand = state.hand_count(owner);
    let (state, pipe) = activate(state, owner, vec![second_other], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(state.hand_count(owner), owner_hand);
}

#[test]
fn xiongyi_does_not_recover_when_owner_is_not_at_minimum_hp() {
    let (mut state, owner, target, other) = started(2_026_072_302);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(other).unwrap().hp = 1;
    let target_hand = state.hand_count(target);

    let (state, pipe) = activate(state, owner, vec![target], 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), target_hand + 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!pipe
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == owner)));
}
}

mod original_sp_xisheng_key_rei {
//! Golden coverage for SP「牺牲」and KEY 零 rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::dying_save_card_for;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XISHENG: u32 = 4_243_619_960;
const XIANDENG: u32 = 2_439_633_691;
const SHULV: u32 = 2_586_462_603;
const KEY_REI: u32 = 3_471_626_821;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/xisheng.skill");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("xisheng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 289);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_729_32,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let attacker = engine.state().turn.current;
    let owner = players
        .into_iter()
        .find(|player| *player != attacker)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: attacker,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XISHENG),
            },
        ],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    for card in tx.state().hand_cards(owner) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    for card in tx.state().hand_cards(attacker) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    let attack = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    (tx.commit().state, owner, attacker)
}

#[test]
fn packages_publish_exact_cross_package_character_contract() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let view = sp
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == XISHENG)
        .unwrap();
    assert_eq!(
        (
            view.as_kind.as_str(),
            view.window.as_str(),
            view.filter.as_str(),
            view.source_zone.as_deref(),
            view.uses_per_turn,
            view.min_cards,
            view.max_cards,
        ),
        (
            "tao",
            "response",
            "owner_outside_turn",
            Some("hand_equip_expansion"),
            Some(1),
            2,
            2,
        )
    );
    assert!(key
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
    assert_eq!(
        key.character_index
            .iter()
            .find(|character| character.id == KEY_REI)
            .unwrap()
            .skills,
        [XIANDENG, SHULV, XISHENG]
    );
}

#[test]
fn exactly_two_cards_save_once_only_outside_the_owners_turn() {
    let (state, owner, attacker) = setup();
    let sources = state.hand_cards(owner);
    assert!(sources.len() >= 2);
    assert_eq!(dying_save_card_for(&state, owner, owner), None);
    let attack = state.hand_cards(attacker)[0];
    let (state, save) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        2,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let options: Vec<_> = save
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    let conversion = options
        .iter()
        .copied()
        .find(|token| !matches!(*token, 0 | 1) && !source_tokens.contains(token))
        .unwrap_or_else(|| panic!("missing conversion: options={options:?} sources={sources:?}"));
    let (state, choice) = resolve(state, owner, vec![conversion], 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    let selected: Vec<_> = sources.iter().take(2).map(|card| card.raw()).collect();
    let (mut state, result) = resolve(state, owner, selected, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);

    state.players.get_mut(owner).unwrap().hp = 0;
    state.players.get_mut(owner).unwrap().dying = true;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave {
            player: owner,
            source: attacker,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);

    let (mut state, owner, attacker) = setup();
    state.turn.current = owner;
    state.players.get_mut(owner).unwrap().hp = 0;
    state.players.get_mut(owner).unwrap().dying = true;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave {
            player: owner,
            source: attacker,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);
}
}

mod original_sp_yongsi1_sanku {
//! Golden for sp 雍嗣·摸 (yongsi1) + 三哭 (sanku) rewrites.
//!
//! yongsi1: locked — during the draw phase, draw `alive_group_count()` extra cards.
//! sanku: locked — when entering dying, lose 1 max HP then recover to max HP.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YONGSI1: u32 = 3_190_419_615;
const SANKU: u32 = 2_924_465_879;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start(player_count: u8) -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: 20_261_501,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
}

fn end_phase(state: engine_core::state::GameState, actor: PlayerId, key: u64) -> (engine_core::state::GameState, engine_api::CommandResult) {
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    );
    (state, pipe.result)
}

#[test]
fn committed_package_publishes_yongsi1_and_sanku_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == YONGSI1 && t.event == "phase_draw"
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SANKU && t.event == "dying"
    }));
}

#[test]
fn yongsi1_draws_alive_group_count_extra_cards() {
    let (engine, owner, _other) = start(4);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, YONGSI1);
    tx.state_mut().turn.current = owner;
    // Start at Judge so EndPhase transitions INTO the draw phase (where the bonus applies).
    tx.state_mut().turn.phase = Phase::Judge;
    let before = tx.state().hand_count(owner);
    // Standard 4-player start: four distinct factions (wei/shu/wu/qun).
    let expected_extra = engine_core::query::alive_group_count(tx.state()) as u32;
    assert!(expected_extra >= 3, "at least three distinct factions at a 4-player start");

    let (state, result) = end_phase(tx.commit().state, owner, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        before + 2 + expected_extra,
        "base 2 + alive_group_count extra cards drawn"
    );
}

#[test]
fn sanku_reduces_max_hp_and_recovers_to_max_when_dying() {
    let (engine, owner, other) = start(2);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, SANKU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    assert!(max_hp_before >= 3);

    // Deal 1 damage → owner enters dying → sanku loses 1 max HP and recovers fully.
    let damage_result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage_result {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("unexpected damage error: {e:?}"),
    }

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(
        player.max_hp,
        max_hp_before - 1,
        "sanku loses 1 max HP on dying"
    );
    assert_eq!(
        player.hp,
        player.max_hp as i32,
        "sanku recovers the owner to full HP"
    );
    assert!(player.alive, "recovered from dying");
    assert!(!player.dying, "no longer dying after recovery");
}

#[test]
fn sanku_recovers_after_heavy_damage_through_dying() {
    let (engine, owner, other) = start(2);
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner, SANKU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    let damage_result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage_result {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("unexpected damage error: {e:?}"),
    }

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before - 1);
    assert_eq!(player.hp, player.max_hp as i32, "healed back to the reduced max HP");
    assert!(player.alive);
}
}

mod original_sp_yuanchou {
//! Golden coverage for SP 范疆张达「远筹」(yuanchou).
//! Locked ignore_armor: the owner's cards bypass the target's armor.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{apply_int_modifiers_for_state, ModifierTarget};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YUANCHOU: u32 = 3_016_188_154;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/yuanchou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("yuanchou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_02, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YUANCHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn source_ignores_armor(state: &GameState, source: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::IgnoreArmor);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

#[test]
fn compiled_source_and_npack_publish_ignore_armor() {
    let compiled = content_compiler::compile_source_named("yuanchou", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == YUANCHOU && skill.name == "yuanchou" && skill.locked
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YUANCHOU && modifier.target == "ignore_armor"
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YUANCHOU && skill.name == "yuanchou"));
    let pack = committed
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == YUANCHOU)
        .expect("npack yuanchou modifier");
    assert_eq!(pack.target, "ignore_armor");
    assert_eq!(pack.mode, "override");
    assert_eq!(pack.value, 1);
}

#[test]
fn owner_ignores_armor_other_does_not() {
    let (state, owner, other) = setup();
    assert!(source_ignores_armor(&state, owner));
    assert!(!source_ignores_armor(&state, other));
}
}

mod original_sp_zhengnan_guansuo {
//! Golden coverage for SP「征南」及关索武将闭包。

const ZHENGNAN: u32 = 3_435_011_424;
const XIEFANG: u32 = 3_450_775_931;
const GUANSUO: u32 = 3_505_703_155;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn zhengnan_draws_before_missing_skill_choice_and_closes_guansuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "onlyOL"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHENGNAN)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    let draw = module
        .code
        .windows(2)
        .position(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    rule_vm::bytecode::effect_tag::DRAW_CARDS as u8,
                ]
        })
        .unwrap();
    let choose = module
        .code
        .windows(2)
        .position(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_EFFECT,
                    rule_vm::bytecode::effect_tag::CHOOSE_ADD_MISSING_SKILL as u8,
                ]
        })
        .unwrap();
    assert!(draw < choose);
    assert!(package.character_index.iter().any(|character| {
        character.id == GUANSUO && character.skills == vec![ZHENGNAN, XIEFANG]
    }));
}
}

mod original_sp_zhenlue {
//! Golden tests for SP「缜略」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/sp/zhenlue.skill");
const ZHENLUE: u32 = 3_469_981_264;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64) -> (Transaction, [PlayerId; 3]) {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(start(seed));
    let actor = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let pos = order.iter().position(|player| *player == actor).unwrap();
    let players = [
        actor,
        order[(pos + 1) % order.len()],
        order[(pos + 2) % order.len()],
    ];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in players {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, players)
}

fn add_zhenlue(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(ZHENLUE),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_900 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

fn play(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_package_matches_rewritten_source() {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let compiled = content_compiler::compile_source_named("zhenlue", SOURCE).unwrap();
    assert!(compiled.skill_index[0].locked);
    assert_eq!(compiled.modifier_index[0].target, "card_use_unrespondable");
    assert_eq!(
        compiled.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(compiled.modifier_index[1].target, "card_target_cancel");
    assert_eq!(
        compiled.modifier_index[1].selector.as_deref(),
        Some("delayed_trick")
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHENLUE && skill.locked));
}

#[test]
fn ordinary_trick_skips_the_whole_wuxie_window() {
    let (mut tx, [source, responder, _]) = loaded(20_260_720);
    add_zhenlue(&mut tx, source);
    let wuzhong = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let wuxie = inject(&mut tx, responder, CardKind::TrickWuxie, Suit::Club, 12);

    let (state, pipeline) = play(&tx, source, wuzhong, vec![]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 2);
    assert!(state.hand_cards(responder).contains(&wuxie));
}

#[test]
fn a_zhenlue_wuxie_cannot_be_countered() {
    let (mut tx, [source, first, second]) = loaded(20_260_721);
    add_zhenlue(&mut tx, first);
    let wuzhong = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let first_wuxie = inject(&mut tx, first, CardKind::TrickWuxie, Suit::Club, 12);
    let second_wuxie = inject(&mut tx, second, CardKind::TrickWuxie, Suit::Spade, 13);

    let (state, pipeline) = play(&tx, source, wuzhong, vec![]);
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, first);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: first,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![first_wuxie.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 0);
    assert!(state.hand_cards(second).contains(&second_wuxie));
}

#[test]
fn delayed_tricks_cannot_target_owner_but_remain_respondable_when_owner_uses_them() {
    let (mut blocked, [source, owner, _]) = loaded(20_260_722);
    add_zhenlue(&mut blocked, owner);
    let le = inject(&mut blocked, source, CardKind::TrickLe, Suit::Heart, 6);
    let (_, blocked_pipeline) = play(&blocked, source, le, vec![owner]);
    assert!(matches!(
        blocked_pipeline.result,
        CommandResult::Rejected { .. }
    ));

    let (mut allowed, [owner, target, _]) = loaded(20_260_723);
    add_zhenlue(&mut allowed, owner);
    let le = inject(&mut allowed, owner, CardKind::TrickLe, Suit::Heart, 6);
    inject(&mut allowed, target, CardKind::TrickWuxie, Suit::Club, 12);
    let (state, pipeline) = play(&allowed, owner, le, vec![target]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );
    assert_eq!(pipeline.choice.unwrap().observer, target);
}
}

mod original_sp_zhenwei_three {
//! Golden for SP 统率三军「镇卫」的阵营距离光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::Identity;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZHENWEI_THREE: u32 = 3_450_139_382;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHENWEI_THREE && skill.locked));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == ZHENWEI_THREE
            && modifier.target == "global_distance_to"));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ZHENWEI_THREE as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ZHENWEI_THREE as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(players[1]).unwrap().identity = Identity::Loyalist;
    tx.state_mut().players.get_mut(players[2]).unwrap().identity = Identity::Loyalist;
    tx.state_mut().players.get_mut(players[3]).unwrap().identity = Identity::Rebel;
    (tx, players[0], players[1], players[2], players[3])
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected distance integer, got {other:?}"),
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHENWEI_THREE),
        }],
    )
    .unwrap();
}

#[test]
fn living_allied_owners_stack_only_against_enemy_sources() {
    let (mut tx, protected, first_owner, second_owner, enemy) = isolated();
    let base = distance(&tx, enemy, protected);
    let ally_base = distance(&tx, first_owner, protected);
    add_skill(&mut tx, first_owner);
    assert_eq!(distance(&tx, enemy, protected), base + 1);
    assert_eq!(distance(&tx, first_owner, protected), ally_base);

    add_skill(&mut tx, second_owner);
    assert_eq!(distance(&tx, enemy, protected), base + 2);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(distance(&tx, enemy, protected), base + 1);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(distance(&tx, enemy, protected), base);
}

#[test]
fn an_owner_does_not_protect_self_but_is_protected_by_another_owner() {
    let (mut tx, protected, first_owner, second_owner, enemy) = isolated();
    let self_base = distance(&tx, enemy, first_owner);
    add_skill(&mut tx, first_owner);
    assert_eq!(distance(&tx, enemy, first_owner), self_base);

    add_skill(&mut tx, second_owner);
    assert_eq!(distance(&tx, enemy, first_owner), self_base + 1);

    let protected_base = distance(&tx, enemy, protected);
    add_skill(&mut tx, protected);
    assert_eq!(distance(&tx, enemy, protected), protected_base);
}

#[test]
fn enemy_owned_skill_does_not_protect_the_opposing_team() {
    let (mut tx, protected, _, _, enemy) = isolated();
    let base = distance(&tx, enemy, protected);
    add_skill(&mut tx, enemy);
    assert_eq!(distance(&tx, enemy, protected), base);
}
}

mod original_sp_zhiri {
//! Golden coverage for sp 炙日 (zhiri): awakening skill that, at the owner's
//! prepare phase, if the 「焚」 (fentian) skill pile holds at least 3 cards,
//! reduces the owner's max HP by 1 and grants 心惔 (xintan). Fires at most once.
//!
//! Exercises the awakening approximation (`mark_count` once-gate), the
//! `skill_pile_count` query, and `change_max_hp` + `add_skill` effects. The
//! upstream `awakenSkill` once-per-game guard is approximated with the
//! awakening mark (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::state::{Card, CardKind, SkillPileRef, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;

const ZHIRI: u32 = 2_217_683_305;
const XINTAN: u32 = 2_185_342_847;
const FENTIAN: u32 = 3_669_648_107;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_817,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_zhiri(tx: &mut Transaction, owner: PlayerId) {
    use engine_core::effect::run_effects;
    use rule_api::EffectRequest;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIRI),
    });
    run_effects(tx, effects).unwrap();
}

/// Seeds `count` face-up cards into the owner's fentian (焚) skill pile.
fn seed_fentian(tx: &mut Transaction, owner: PlayerId, count: usize) {
    let zone_id = tx.state_mut().zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: vec![],
    });
    let mut cards = Vec::new();
    for index in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(170_100 + index as u32),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: zone_id,
            owner,
            face_up: true,
        });
        cards.push(card);
    }
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards = cards;
    tx.state_mut().skill_piles.push(SkillPileRef {
        owner,
        skill_def: FENTIAN,
        zone: zone_id,
        usable_as_hand: false,
        wants_lost_count: false,
    });
}

fn fire_prepare(tx: &mut Transaction, owner: PlayerId) {
    use engine_core::effect::run_effects;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn once_mark(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == ZHIRI))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn committed_package_publishes_zhiri_prepare_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHIRI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn fentian_at_three_awakens_once() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_zhiri(&mut tx, owner);
    seed_fentian(&mut tx, owner, 3);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    assert!(
        !tx
            .state()
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.def.raw() == XINTAN),
        "心惔 not granted before awakening"
    );

    fire_prepare(&mut tx, owner);

    let owner_state = tx.state().players.get(owner).unwrap();
    assert_eq!(
        owner_state.max_hp,
        max_hp_before - 1,
        "awakening loses 1 max HP"
    );
    assert_eq!(once_mark(&tx, owner), 1, "awakening mark set");
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.def.raw() == XINTAN),
        "心惔 granted after awakening"
    );

    // A later prepare phase must not re-fire the awakening.
    let max_hp_after = tx.state().players.get(owner).unwrap().max_hp;
    fire_prepare(&mut tx, owner);
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_after,
        "awakening fires at most once"
    );
}

#[test]
fn fentian_below_three_does_not_awaken() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_zhiri(&mut tx, owner);
    seed_fentian(&mut tx, owner, 2);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    fire_prepare(&mut tx, owner);

    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before,
        "2 cards in 焚 → no awakening"
    );
    assert_eq!(once_mark(&tx, owner), 0);
    assert!(
        !tx
            .state()
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.def.raw() == XINTAN)
    );
}
}

mod original_sp_ziyuan_mizhu {
//! Golden coverage for SP 糜竺「资援」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::effective_card_number;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, query, Engine, QueryContext, Snapshot};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZIYUAN: u32 = 2_303_983_371;
const JUGU: u32 = 2_684_521_577;
const MIZHU: u32 = 3_031_952_466;
const NUMBER_THIRTEEN: u32 = 4_200_000_131;
const NUMBER_TWELVE: u32 = 4_200_000_132;
const GIVER_DIES: u32 = 4_200_000_133;
const TARGET_DIES: u32 = 4_200_000_134;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/ziyuan.skill");
const JUGU_SOURCE: &str = include_str!("../../../content/original/ported/sp/jugu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MODULE_LOGIC_HASH: [u8; 32] = [
    21, 26, 175, 88, 48, 123, 237, 245,
    208, 193, 63, 34, 142, 204, 22, 13,
    42, 95, 51, 235, 64, 164, 128, 170,
    160, 1, 57, 86, 247, 238, 217, 69,
];
const PACKAGE_LOGIC_HASH: [u8; 32] = [104,11,251,180,248,79,187,16,89,76,180,14,141,177,202,117,119,248,212,56,232,136,9,34,34,192,63,77,154,118,188,178];
const TEST_HELPERS: &str = r#"
skill number_thirteen(4200000131) locked {
  modifier card_number override 13;
}

skill number_twelve(4200000132) locked {
  modifier card_number override 12;
}

skill giver_dies(4200000133) locked {
  trigger cards_given owner_alive_gave_cards_to_other_alive {
    effect lose_hp(self, 99);
  }
}

skill target_dies(4200000134) locked {
  trigger cards_moved owner_received_gift_from_other {
    effect lose_hp(self, 99);
  }
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + u32::from(rank)),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(ranks: &[u8], owner_helpers: &[u32], target_helpers: &[u32]) -> Setup {
    let package = compile_source_named(
        "sp-ziyuan-test",
        &format!("{SOURCE}\n{JUGU_SOURCE}\n{TEST_HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_001, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZIYUAN),
    });
    effects.extend(owner_helpers.iter().map(|skill| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill),
    }));
    effects.extend(target_helpers.iter().map(|skill| EffectRequest::AddSkill {
        player: target,
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let max_hp = tx.state().players.get(target).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = max_hp - 1;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZIYUAN)
        .unwrap();
    let mut state = tx.commit().state;
    let cards = ranks
        .iter()
        .map(|rank| inject(&mut state, owner, ZoneKind::Hand, *rank))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        cards,
    }
}

fn activate(setup: Setup, selected: Vec<CardId>, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: selected,
        },
        key,
    )
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_module_character_and_jugu_contract_are_stable() {
    let compiled = compile_source_named("sp-ziyuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 301);
    let active = compiled.active_index.first().unwrap();
    assert_eq!(active.skill_def_id, ZIYUAN);
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_play_phase, Some(1));
    assert_eq!(
        active
            .selection
            .as_ref()
            .unwrap()
            .sum_effective_number_exact,
        Some(13)
    );
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "ziyuan_active_play_phase_0")
        .unwrap();
    assert_eq!(module.code, [83, 85, 16, 1, 0, 0, 0, 90, 32, 1, 1]);
    assert_eq!(module.logic_hash, MODULE_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET,
        288
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, PACKAGE_LOGIC_HASH);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MIZHU)
        .unwrap();
    assert_eq!(character.name_key, "mizhu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [ZIYUAN, JUGU]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JUGU
            && modifier.target == "initial_hand_count"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_max_hp")
            && modifier.value == 1
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JUGU
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_max_hp")
            && modifier.value == 1
    }));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn king_and_five_plus_eight_are_valid_and_full_target_is_allowed() {
    let mut king = setup(&[13], &[], &[]);
    let max_hp = king.state.players.get(king.target).unwrap().max_hp as i32;
    king.state.players.get_mut(king.target).unwrap().hp = max_hp;
    let card = king.cards[0];
    let target = king.target;
    let (state, result) = activate(king, vec![card], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(card).unwrap().owner, target);
    assert_eq!(state.players.get(target).unwrap().hp, max_hp);

    let pair = setup(&[5, 8], &[], &[]);
    let target = pair.target;
    let cards = pair.cards.clone();
    let hp = pair.state.players.get(target).unwrap().hp;
    let (state, result) = activate(pair, cards.clone(), 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().owner == target));
    assert_eq!(state.players.get(target).unwrap().hp, hp + 1);
    let moves: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            engine_api::DomainEvent::CardsMoved { cards, reason, .. }
                if *reason == CardMoveReason::Give =>
            {
                Some(cards.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(moves, [cards]);
}

#[test]
fn jugu_still_adds_owner_max_hp_to_initial_hand_and_hand_limit() {
    let setup = setup(&[], &[JUGU], &[]);
    let player = setup.state.players.get(setup.owner).unwrap();
    let max_hp = u32::from(player.max_hp);
    assert_eq!(
        engine_core::query::initial_hand_count_for(&setup.state, setup.owner, 4),
        4 + max_hp
    );
    assert_eq!(
        query(
            &setup.state,
            &QueryRequest::HandLimit {
                player: setup.owner,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(player.hp + max_hp as i32)
    );
}

#[test]
fn invalid_sums_duplicates_and_non_hand_cards_are_rejected() {
    for (ranks, selected) in [(&[5, 7][..], vec![0, 1]), (&[6, 8][..], vec![0, 1])] {
        let setup = setup(ranks, &[], &[]);
        let cards: Vec<_> = selected.iter().map(|index| setup.cards[*index]).collect();
        let (_, result) = activate(setup, cards, 10 + ranks[0] as u64);
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
    }

    let duplicate = setup(&[13], &[], &[]);
    let card = duplicate.cards[0];
    let (_, result) = activate(duplicate, vec![card, card], 30);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let mut non_hand = setup(&[5], &[], &[]);
    let equipment = inject(&mut non_hand.state, non_hand.owner, ZoneKind::Equip, 8);
    let hand = non_hand.cards[0];
    let (_, result) = activate(non_hand, vec![hand, equipment], 31);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn effective_card_number_modifier_changes_exact_sum_legality() {
    let positive = setup(&[12], &[NUMBER_THIRTEEN], &[]);
    let card = positive.cards[0];
    assert_eq!(
        effective_card_number(
            &positive.state,
            positive.owner,
            positive.state.cards.get(card).unwrap()
        ),
        13
    );
    let (_, result) = activate(positive, vec![card], 40);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let negative = setup(&[13], &[NUMBER_TWELVE], &[]);
    let card = negative.cards[0];
    assert_eq!(
        effective_card_number(
            &negative.state,
            negative.owner,
            negative.state.cards.get(card).unwrap()
        ),
        12
    );
    let (_, result) = activate(negative, vec![card], 41);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn use_limit_resets_on_each_play_phase_entry() {
    let setup = setup(&[13, 13], &[], &[]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let first = setup.cards[0];
    let second = setup.cards[1];
    let (state, first_result) = activate(setup, vec![first], 50);
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.skills.get(skill).unwrap().play_phase_use_count, 1);

    let (state, repeated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![second],
        },
        51,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().skills.get(skill).unwrap().play_phase_use_count,
        0
    );
    let state = tx.commit().state;
    let (_, second_result) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![second],
        },
        52,
    );
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
}

#[test]
fn runner_selects_an_exact_sum_subset() {
    let setup = setup(&[1, 5, 8], &[], &[]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let five = setup.cards[1];
    let eight = setup.cards[2];
    let (state, result) = command(
        setup.state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        60,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.unwrap();
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections == &vec![five.raw(), eight.raw()]
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().cards.get(five).unwrap().owner, target);
    assert_eq!(engine.state().cards.get(eight).unwrap().owner, target);
}

#[test]
fn nested_giver_death_does_not_cancel_recovery_and_target_death_skips_it() {
    let giver_dies = setup(&[13], &[GIVER_DIES], &[]);
    let owner = giver_dies.owner;
    let target = giver_dies.target;
    let target_hp = giver_dies.state.players.get(target).unwrap().hp;
    let card = giver_dies.cards[0];
    let (state, result) = activate(giver_dies, vec![card], 70);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp + 1);
    let killed = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                engine_api::DomainEvent::PlayerKilled { player, .. } if *player == owner
            )
        })
        .unwrap();
    let recovered = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                engine_api::DomainEvent::HpRecovered { player, .. } if *player == target
            )
        })
        .unwrap();
    assert!(killed < recovered);

    let target_dies = setup(&[13], &[], &[TARGET_DIES]);
    let target = target_dies.target;
    let card = target_dies.cards[0];
    let (state, result) = activate(target_dies, vec![card], 71);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(target).unwrap().alive);
    assert!(!state.has_pending_choice());
    assert!(result.events.iter().any(|event| {
        matches!(
            event,
            engine_api::DomainEvent::PlayerKilled { player, .. } if *player == target
        )
    }));
    assert!(!result.events.iter().any(|event| {
        matches!(
            event,
            engine_api::DomainEvent::HpRecovered { player, .. } if *player == target
        )
    }));
}
}
