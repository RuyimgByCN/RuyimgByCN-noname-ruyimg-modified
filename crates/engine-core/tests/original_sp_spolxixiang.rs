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
