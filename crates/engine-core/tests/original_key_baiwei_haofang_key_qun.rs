//! Golden coverage for KEY 摆尾 / 豪放 / Key势 / 群势 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const INARI_BAIWEI: u32 = 3_124_826_192;
const HARUKO_HAOFANG: u32 = 2_831_053_102;
const KOTORI_SKILL_KEY: u32 = 4_028_878_707;
const KOTORI_SKILL_QUN: u32 = 2_366_750_902;

const BAIWEI_SRC: &str = include_str!("../../../content/original/ported/key/inari_baiwei.skill");
const HAOFANG_SRC: &str = include_str!("../../../content/original/ported/key/haruko_haofang.skill");
const KEY_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_key.skill");
const QUN_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_qun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
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
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn baiwei_source_and_npack_publish_diamond_basics() {
    let compiled = content_compiler::compile_source_named("inari_baiwei", BAIWEI_SRC).unwrap();
    for kind in ["sha", "shan", "tao", "jiu"] {
        assert!(
            compiled.view_as_index.iter().any(|view| {
                view.skill_def_id == INARI_BAIWEI && view.as_kind == kind && view.filter == "diamond"
            }),
            "missing view_as {kind}"
        );
    }
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.view_as_index.iter().any(|view| {
        view.skill_def_id == INARI_BAIWEI && view.as_kind == "sha" && view.filter == "diamond"
    }));
}

#[test]
fn haofang_source_and_npack_publish_delayed_as_wuzhong() {
    let compiled = content_compiler::compile_source_named("haruko_haofang", HAOFANG_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HARUKO_HAOFANG
            && modifier.target == "card_name"
            && modifier.selector.as_deref() == Some("delayed_trick")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HARUKO_HAOFANG && modifier.target == "card_name"
    }));
}

#[test]
fn haofang_delayed_trick_is_effective_wuzhong() {
    let (mut tx, owner) = bootstrap(HAOFANG_SRC, "haofang-test", HARUKO_HAOFANG);
    let lebu = inject_hand(&mut tx, owner, CardKind::TrickLe, Suit::Heart, 283_101);
    let physical = tx.state().cards.get(lebu).unwrap();
    assert_eq!(physical.kind, CardKind::TrickLe);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::TrickWuzhong
    );
}

#[test]
fn key_source_and_npack_publish_play_phase_active() {
    let compiled = content_compiler::compile_source_named("kotori_skill_key", KEY_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KOTORI_SKILL_KEY));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KOTORI_SKILL_KEY));
}

#[test]
fn key_play_phase_draws_one_and_loses_one_hp() {
    let (mut tx, owner) = bootstrap(KEY_SRC, "kotori-key-test", KOTORI_SKILL_KEY);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 3;
    }
    let before_hand = tx.state().hand_count(owner);
    let before_hp = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == KOTORI_SKILL_KEY)
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
    assert_eq!(state.hand_count(owner), before_hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp - 1);
}

#[test]
fn qun_source_and_npack_publish_hand_limit_lost_hp() {
    let compiled = content_compiler::compile_source_named("kotori_skill_qun", QUN_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KOTORI_SKILL_QUN
            && modifier.target == "hand_limit"
            && modifier.value_source.as_deref() == Some("owner_lost_hp")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KOTORI_SKILL_QUN && modifier.target == "hand_limit"
    }));
}

#[test]
fn qun_hand_limit_grows_with_lost_hp() {
    let (mut tx, owner) = bootstrap(QUN_SRC, "kotori-qun-test", KOTORI_SKILL_QUN);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 4;
    }
    assert_eq!(hand_limit(tx.state(), owner), 4);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 2;
    }
    // Base hand limit is current HP (2); owner_lost_hp adds 2.
    assert_eq!(hand_limit(tx.state(), owner), 4);
}
