//! Golden coverage for KEY 观铃「浓音」and 白羽「孤影」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MISUZU_NONGYIN: u32 = 2_333_084_601;
const SHIROHA_GUYING: u32 = 2_209_884_612;
const NONGYIN_SRC: &str =
    include_str!("../../../../content/original/ported/key/misuzu_nongyin.skill");
const GUYING_SRC: &str =
    include_str!("../../../../content/original/ported/key/shiroha_guying.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
        rank: 4,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn nongyin_source_and_npack_publish_red_tao() {
    let compiled = content_compiler::compile_source_named("misuzu_nongyin", NONGYIN_SRC).unwrap();
    assert!(compiled.view_as_index.iter().any(|view| {
        view.skill_def_id == MISUZU_NONGYIN && view.as_kind == "tao" && view.filter == "red"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| { view.skill_def_id == MISUZU_NONGYIN && view.as_kind == "tao" }));
}

#[test]
fn nongyin_red_card_converts_to_tao_when_wounded() {
    let (mut tx, owner, _) = bootstrap(NONGYIN_SRC, "nongyin-test", MISUZU_NONGYIN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let card = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Heart, 181_201);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == MISUZU_NONGYIN)
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
                targets: vec![owner],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn guying_source_and_npack_publish_both_damage_windows() {
    let compiled = content_compiler::compile_source_named("shiroha_guying", GUYING_SRC).unwrap();
    let mut events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SHIROHA_GUYING)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    events.sort();
    assert_eq!(
        events,
        vec!["owner_is_target", "owner_source_other_damage_before"]
    );
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SHIROHA_GUYING)
            .count(),
        2
    );
}

#[test]
fn guying_reduces_incoming_and_increases_outgoing_damage() {
    let (tx, owner, other) = bootstrap(GUYING_SRC, "guying-test", SHIROHA_GUYING);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
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
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
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
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}
