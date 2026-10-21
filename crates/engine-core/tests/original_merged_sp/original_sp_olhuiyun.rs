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
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/olhuiyun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("olhuiyun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_31, "identity-standard").unwrap();
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
