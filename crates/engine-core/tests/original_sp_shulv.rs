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
