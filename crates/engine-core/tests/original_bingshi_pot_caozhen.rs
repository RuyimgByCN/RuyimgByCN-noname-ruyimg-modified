//! Golden coverage for BINGSHI 势曹真「伺锋」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTSIFENG: u32 = 3_809_208_030;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(517),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 517,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|role| *role != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(POTSIFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == POTSIFENG)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
}

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn distribute_all(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotsifeng {
                player: owner,
                target: owner,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let cards = tx.state().resolution.continuation_payload[4..].to_vec();
    let state = choose(tx.commit().state, cards, 1);
    choose(state, vec![target.raw()], 2)
}

#[test]
fn package_publishes_complete_pot_caozhen_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_caozhen")
            .unwrap()
            .skills,
        [POTSIFENG]
    );
}

#[test]
fn sifeng_distributes_in_selection_order_consumes_first_and_forces_mismatched_discard() {
    let (state, owner, target, skill) = setup();
    let state = distribute_all(state, owner, target, skill);
    let pile = state.skill_pile_cards(target, POTSIFENG);
    assert_eq!(pile.len(), 3);
    let first = pile[0];
    let event_card = state.hand_cards(target)[0];
    let mut tx = Transaction::begin(state);
    let first_red = matches!(
        tx.state().cards.get(first).unwrap().suit,
        Suit::Heart | Suit::Diamond
    );
    tx.state_mut().cards.get_mut(event_card).unwrap().suit =
        if first_red { Suit::Spade } else { Suit::Heart };
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotsifeng {
                player: owner,
                target,
                card: event_card,
                stage: 3,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().skill_pile_cards(target, POTSIFENG).len(), 2);
    let discard = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .find(|card| *card != event_card)
        .unwrap_or(event_card);
    let state = choose(tx.commit().state, vec![discard.raw()], 3);
    assert_eq!(state.hand_count(target), 3);
}

#[test]
fn sifeng_phase_end_can_gain_all_or_discard_all_then_damage() {
    for (branch, gains) in [(1, true), (0, false)] {
        let (state, owner, target, skill) = setup();
        let state = distribute_all(state, owner, target, skill);
        let owner_hand = state.hand_count(owner);
        let hp = state.players.get(target).unwrap().hp;
        let mut tx = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolvePotsifeng {
                    player: owner,
                    target,
                    card: CardId::NONE,
                    stage: 4,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let state = choose(tx.commit().state, vec![branch], 4 + branch as u64);
        assert!(state.skill_pile_cards(target, POTSIFENG).is_empty());
        if gains {
            assert_eq!(state.hand_count(owner), owner_hand + 3);
            assert_eq!(state.players.get(target).unwrap().hp, hp);
        } else {
            assert_eq!(state.hand_count(owner), owner_hand);
            assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        }
    }
}
