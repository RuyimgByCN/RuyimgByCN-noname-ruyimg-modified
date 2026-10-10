//! Golden coverage for BINGSHI 势臧洪「烈志」「拒孤」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POT_LIEZHI: u32 = 3_956_687_454;
const POT_JUGU: u32 = 4_277_593_099;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(defs: &[u32]) -> (engine_core::state::GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(516),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 516,
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
    effects.extend(defs.iter().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner)
}

fn skill(state: &engine_core::state::GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(
    state: engine_core::state::GameState,
    selections: Vec<u32>,
    key: u64,
) -> engine_core::state::GameState {
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

#[test]
fn package_publishes_complete_pot_zanghong_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zanghong")
            .unwrap()
            .skills,
        [POT_LIEZHI, POT_JUGU]
    );
}

#[test]
fn liezhi_uses_source_free_jiu_loses_max_hp_and_shares_one_turn_limit() {
    let (state, owner) = setup(&[POT_LIEZHI]);
    let liezhi = skill(&state, owner, POT_LIEZHI);
    let before = state.players.get(owner).unwrap().max_hp;
    let revision = state.revision;
    let (state, used) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill: liezhi,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().max_hp, before - 1);
    assert_eq!(state.skills.get(liezhi).unwrap().use_count, 1);
    let revision = state.revision;
    let (_, repeated) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::UseSkillCard {
                skill: liezhi,
                card_kind: CardKind::Tao.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn jugu_discards_one_then_two_across_successive_turn_fires() {
    let (state, owner) = setup(&[POT_JUGU]);
    let jugu = skill(&state, owner, POT_JUGU);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjugu {
                player: owner,
                skill: jugu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let first = tx.state().hand_cards(owner)[0];
    let state = choose(tx.commit().state, vec![first.raw()], 3);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjugu {
                player: owner,
                skill: jugu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let cards: Vec<_> = tx.state().hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let state = choose(tx.commit().state, cards, 4);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(state.skills.get(jugu).unwrap().turn_fire_count, 2);
}
