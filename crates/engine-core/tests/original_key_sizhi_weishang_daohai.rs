//! Golden coverage for KEY 丝织标记 / 伪伤 / 伪伤破杀 / 蹈海 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHIZUKU_SIZHI2: u32 = 3_470_603_700;
const KENGO_WEISHANG: u32 = 2_671_352_658;
const KENGO_WEISHANG_SHA: u32 = 3_863_413_686;
const SATOMI_DAOHAI: u32 = 2_215_481_549;

const SIZHI_SRC: &str = include_str!("../../../content/original/ported/key/shizuku_sizhi2.skill");
const WEISHANG_SRC: &str =
    include_str!("../../../content/original/ported/key/kengo_weishang.skill");
const WEISHANG_SHA_SRC: &str =
    include_str!("../../../content/original/ported/key/kengo_weishang_sha.skill");
const DAOHAI_SRC: &str = include_str!("../../../content/original/ported/key/satomi_daohai.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn sizhi2_source_and_npack_publish_black_red_modifiers() {
    let compiled = content_compiler::compile_source_named("shizuku_sizhi2", SIZHI_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_ignored_hand_limit"
            && modifier.selector.as_deref() == Some("red")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_distance_unlimited"
            && modifier.selector.as_deref() == Some("black")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_use_unlimited"
            && modifier.selector.as_deref() == Some("black")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2 && modifier.target == "card_use_unlimited"
    }));
}

#[test]
fn weishang_source_and_npack_publish_active_and_combat_mods() {
    let compiled = content_compiler::compile_source_named("kengo_weishang", WEISHANG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KENGO_WEISHANG));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KENGO_WEISHANG && modifier.target == "sha_use_limit"
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KENGO_WEISHANG && modifier.target == "distance_from"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KENGO_WEISHANG));
}

#[test]
fn weishang_play_phase_disables_equip_slots() {
    let (tx, owner, _) = bootstrap(WEISHANG_SRC, "weishang-test", KENGO_WEISHANG);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == KENGO_WEISHANG)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .disabled_equip_slots
        .iter()
        .any(|slot| *slot != 0));
}

#[test]
fn weishang_sha_source_and_npack_publish_sha_target_discard() {
    let compiled =
        content_compiler::compile_source_named("kengo_weishang_sha", WEISHANG_SHA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KENGO_WEISHANG_SHA)
        .expect("weishang_sha trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KENGO_WEISHANG_SHA && trigger.event == "card_targeted"
    }));
}

#[test]
fn weishang_sha_forces_target_to_discard_one() {
    let (mut tx, owner, other) =
        bootstrap(WEISHANG_SHA_SRC, "weishang-sha-test", KENGO_WEISHANG_SHA);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade, 386_301);
    inject_hand(&mut tx, other, CardKind::Shan, Suit::Heart, 386_302);
    let other_hand = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("card used dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(other), other_hand - 1);
}

#[test]
fn daohai_source_and_npack_publish_le() {
    let compiled = content_compiler::compile_source_named("satomi_daohai", DAOHAI_SRC).unwrap();
    assert!(compiled.view_as_index.iter().any(|view| {
        view.skill_def_id == SATOMI_DAOHAI && view.as_kind == "le"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == SATOMI_DAOHAI && view.as_kind == "le"));
}
