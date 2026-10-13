//! O2 全量合并第二批（2026-10-14）：家族 armor 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_armor_effects {
//! Golden for legacy armor effects and their shared bypass channels.

use engine_api::{
    CardDefId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const GUOFENGYUPAO: u32 = 3_538_236_867;
const RW_TENGJIA1: u32 = 3_160_129_602;
const RW_TENGJIA2: u32 = 2_825_664_767;
const RW_TENGJIA4: u32 = 4_041_276_943;
const RW_RENWANG_SKILL: u32 = 4_083_737_417;
const IGNORE_ARMOR: u32 = 4_290_000_001;
const ARMOR_DISABLED: u32 = 4_290_000_002;
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const BYPASS_SOURCE: &str = r#"
skill test_ignore_armor(4290000001) {
  modifier ignore_armor override 1;
}
skill test_armor_disabled(4290000002) {
  modifier armor_effect_disabled override 1;
}
"#;

fn isolated(seed: u64, target_skills: &[u32]) -> (Transaction, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    for bytes in [SP, MOBILE] {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&package).unwrap();
    }
    let bypass = content_compiler::compile_source_named("armor_test", BYPASS_SOURCE).unwrap();
    engine.load_package(&bypass).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let user = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != user)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(target_skills.iter().map(|skill| EffectRequest::AddSkill {
        player: target,
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = user;
    tx.state_mut().turn.phase = Phase::Play;
    for player in [user, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    (tx, user, target)
}

fn target_legal(
    tx: &Transaction,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

fn deal_fire(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
}

fn equip_tengjia(tx: &mut Transaction, player: PlayerId) {
    let card = tx.state().hand_cards(player)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let state = tx.state_mut().cards.get_mut(card).unwrap();
    state.def = CardDefId::new(engine_core::content::DEF_TENGJIA);
    state.kind = CardKind::Armor;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn guofengyupao_blocks_only_incoming_ordinary_tricks_while_armor_is_active() {
    let (mut tx, user, target) = isolated(20_260_901, &[GUOFENGYUPAO]);
    let cards = tx.state().hand_cards(user);
    for (card, kind) in [
        (cards[0], CardKind::TrickGuohe),
        (cards[1], CardKind::TrickLe),
        (cards[2], CardKind::Sha),
        (cards[3], CardKind::TrickWuzhong),
    ] {
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    }

    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(target_legal(&tx, user, cards[1], target));
    assert!(target_legal(&tx, user, cards[2], target));
    assert!(target_legal(&tx, user, cards[3], user));

    add_skill(&mut tx, user, IGNORE_ARMOR);
    assert!(target_legal(&tx, user, cards[0], target));

    let (mut tx, user, target) = isolated(20_260_902, &[GUOFENGYUPAO]);
    let card = tx.state().hand_cards(user)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickGuohe;
    add_skill(&mut tx, target, ARMOR_DISABLED);
    assert!(target_legal(&tx, user, card, target));
}

#[test]
fn rewritten_tengjia_parts_share_target_and_damage_bypass_semantics() {
    let skills = [RW_TENGJIA1, RW_TENGJIA2, RW_TENGJIA4];
    let (mut tx, user, target) = isolated(20_260_903, &skills);
    let cards = tx.state().hand_cards(user);
    for (card, kind) in [
        (cards[0], CardKind::TrickNanman),
        (cards[1], CardKind::TrickWanjian),
        (cards[2], CardKind::Sha),
        (cards[3], CardKind::ShaFire),
    ] {
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    }
    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(!target_legal(&tx, user, cards[1], target));
    assert!(!target_legal(&tx, user, cards[2], target));
    assert!(target_legal(&tx, user, cards[3], target));
    deal_fire(&mut tx, user, target);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 8);

    let (mut tx, user, target) = isolated(20_260_904, &skills);
    add_skill(&mut tx, user, IGNORE_ARMOR);
    deal_fire(&mut tx, user, target);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 9);

    let (mut tx, user, target) = isolated(20_260_905, &skills);
    add_skill(&mut tx, target, ARMOR_DISABLED);
    deal_fire(&mut tx, user, target);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 9);

    let (mut tx, user, target) = isolated(20_260_906, &[]);
    equip_tengjia(&mut tx, target);
    add_skill(&mut tx, target, ARMOR_DISABLED);
    deal_fire(&mut tx, user, target);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 9);
}

#[test]
fn rewritten_renwang_blocks_black_and_heart_sha_without_blocking_other_heart_cards() {
    let (mut tx, user, target) = isolated(20_260_907, &[RW_RENWANG_SKILL]);
    let cards = tx.state().hand_cards(user);
    for (card, kind, suit) in [
        (cards[0], CardKind::Sha, Suit::Spade),
        (cards[1], CardKind::ShaFire, Suit::Heart),
        (cards[2], CardKind::Sha, Suit::Diamond),
        (cards[3], CardKind::TrickGuohe, Suit::Heart),
    ] {
        let state = tx.state_mut().cards.get_mut(card).unwrap();
        state.kind = kind;
        state.suit = suit;
    }
    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(!target_legal(&tx, user, cards[1], target));
    assert!(target_legal(&tx, user, cards[2], target));
    assert!(target_legal(&tx, user, cards[3], target));

    add_skill(&mut tx, user, IGNORE_ARMOR);
    assert!(target_legal(&tx, user, cards[0], target));
}
}

mod original_armor_family_zero_primitive {
//! Golden coverage for ADR 0643: armor family dissolves to zero primitives via
//! MARK_HUJIA (def 3) + `owner_mark_<def>` value source. 5 skills across
//! mobile / key / shahai, no ABI change (500 / 651 / 645).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUXIANG: u32 = 2_271_957_513;
const ERIKA_SHISONG: u32 = 2_366_255_007;
const TOMOYO_CHANGSHI: u32 = 3_488_868_404;
const ZHANHOU: u32 = 2_287_156_849;
const OLDZHANHOU: u32 = 3_706_511_387;

const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SHAHAI: &[u8] = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");
const CHANGSHI_SRC: &str =
    include_str!("../../../content/original/ported/key/tomoyo_changshi.skill");

fn hujia(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(def, _)| *def == 3)
        .map_or(0, |(_, n)| *n)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
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
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn armor_family_npack_contracts() {
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    // 玉箱：出向 -护甲、入向 +护甲（无护甲时值为 0 自动失效）。
    assert!(mobile.modifier_index.iter().any(|m| {
        m.skill_def_id == YUXIANG
            && m.target == "distance_from"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    assert!(mobile.modifier_index.iter().any(|m| {
        m.skill_def_id == YUXIANG && m.target == "distance_to"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);

    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    // 诗诵：手牌上限 + 护甲值。
    assert!(key.modifier_index.iter().any(|m| {
        m.skill_def_id == ERIKA_SHISONG
            && m.target == "hand_limit"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    // 长视：任意回复触发授予护甲标记。
    assert!(key.trigger_index.iter().any(|t| {
        t.skill_def_id == TOMOYO_CHANGSHI && t.event == "hp_recovered" && !t.optional
    }));
    assert_eq!(key.manifest.rule_vm_abi_version, 691);

    let shahai = content_schema::ContentPackage::from_json_bytes(SHAHAI).unwrap();
    // 战吼双形态：限一次 / 不限。
    assert!(shahai
        .active_index
        .iter()
        .any(|a| a.skill_def_id == ZHANHOU));
    assert!(shahai
        .active_index
        .iter()
        .any(|a| a.skill_def_id == OLDZHANHOU));
    assert_eq!(shahai.manifest.rule_vm_abi_version, 657);
}

#[test]
fn changshi_recovery_grants_armor_mark() {
    let compiled = content_compiler::compile_source_named("changshi", CHANGSHI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(TOMOYO_CHANGSHI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TOMOYO_CHANGSHI),
        }],
    )
    .unwrap();
    assert_eq!(hujia(tx.state(), owner), 0);
    // 其他角色回复体力 → 长视获得 1 点护甲（forced，无需确认）。
    let planned = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            player: other,
            source: PlayerId::NONE,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, planned).unwrap();
    let state = tx.commit().state;
    assert_eq!(hujia(&state, owner), 1, "recovery should grant one armor");
}
}
