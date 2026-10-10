//! Golden coverage for KEY「武威」(tomoyo_wuwei): hand cards view-as Sha.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TOMOYO_WUWEI: u32 = 2_212_519_004;
const SOURCE: &str = include_str!("../../../content/original/ported/key/tomoyo_wuwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("tomoyo-wuwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_77, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(TOMOYO_WUWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_805),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 5,
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TOMOYO_WUWEI)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_sha() {
    let compiled = content_compiler::compile_source_named("tomoyo_wuwei", SOURCE).unwrap();
    let mut views: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == TOMOYO_WUWEI)
        .map(|view| view.as_kind.as_str())
        .collect();
    views.sort();
    views.dedup();
    assert_eq!(views, vec!["sha"]);

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == TOMOYO_WUWEI && view.as_kind == "sha"));
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
