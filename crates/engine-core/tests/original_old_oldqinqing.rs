//! Golden coverage for old「寝情」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDQINQING: u32 = 2_296_564_419;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldqinqing.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    lord: PlayerId,
    skill: SkillId,
    discard_card: engine_api::CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oldqinqing-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_807, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let lord = players
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != lord)
        .unwrap();
    let target = players
        .iter()
        .copied()
        .find(|player| *player != lord && *player != owner)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().zones.get(hand).unwrap().cards.clone();
        effects.push(EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDQINQING),
    });
    run_effects(&mut tx, effects).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards[..3].to_vec();
    let lord_hand = tx.state().zones.find(ZoneKind::Hand, lord).unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: cards[..1].to_vec(),
                to: lord_hand,
                from: Some(draw),
            },
            EffectRequest::MoveCards {
                cards: cards[1..].to_vec(),
                to: target_hand,
                from: Some(draw),
            },
        ],
    )
    .unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        target,
        lord,
        skill,
        discard_card: cards[1],
    }
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn discards_then_target_draws_then_owner_draws_on_hand_advantage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldQinqing {
                player: setup.owner,
                target: PlayerId::NONE,
                lord: PlayerId::NONE,
                stage: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = choose(tx.commit().state, setup.owner, setup.target.raw(), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = choose(state, setup.owner, setup.discard_card.raw(), 2);
    assert_eq!(state.hand_count(setup.lord), 1);
    assert_eq!(state.hand_count(setup.target), 2);
    assert_eq!(state.hand_count(setup.owner), 1);
}

#[test]
fn package_publishes_oldqinqing_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDQINQING && skill.name == "oldqinqing"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
