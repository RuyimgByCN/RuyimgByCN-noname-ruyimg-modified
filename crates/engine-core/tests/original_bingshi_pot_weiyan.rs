//! Golden coverage for BINGSHI 势魏延「忠傲/壮誓/饮战/狂骨」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTZHONGAO: u32 = 4_200_000_559;
const POTZHUANGSHI: u32 = 4_200_000_560;
const POTYINZHAN: u32 = 4_200_000_561;
const DIRECT: u32 = 4_200_000_563;
const LIMIT: u32 = 4_200_000_564;
const KUNFEN: u32 = 3_536_332_483;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId) {
    let source = format!(
        "{}\n{}",
        include_str!("../../../content/original/ported/bingshi/pot_weiyan.skill"),
        include_str!("../../../content/original/ported/bingshi/kunfen_pot_weiyan.skill")
    );
    let package = content_compiler::compile_source(&source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 558);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
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
    effects.extend(
        [POTZHONGAO, POTZHUANGSHI, POTYINZHAN].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(558_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_complete_pot_weiyan_at_abi558() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_weiyan")
            .unwrap()
            .skills,
        [POTZHONGAO, POTZHUANGSHI, POTYINZHAN]
    );
}

#[test]
fn mission_failure_replaces_zhuangshi_with_kunfen() {
    let (state, owner) = setup(5581);
    let mut tx = Transaction::begin(state);
    let mission = skill(tx.state(), owner, POTZHONGAO).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotweiyan {
            player: owner,
            target: owner,
            card: CardId::NONE,
            action: 3,
            skill: mission,
        }],
    )
    .unwrap();
    assert!(skill(tx.state(), owner, POTZHUANGSHI).is_none());
    assert!(skill(tx.state(), owner, KUNFEN).is_some());
}

#[test]
fn zhuangshi_discards_and_loses_hp_into_first_n_helpers() {
    let (state, owner) = setup(5582);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let mission = skill(tx.state(), owner, POTZHONGAO).unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotweiyan {
                player: owner,
                target: owner,
                card: CardId::NONE,
                action: 2,
                skill: mission,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![card.raw()], false, 5583);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (state, output) = resolve(state, vec![2], false, 5584);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 2);
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, DIRECT).unwrap())
            .unwrap()
            .mark,
        1
    );
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, LIMIT).unwrap())
            .unwrap()
            .mark,
        2
    );
}
