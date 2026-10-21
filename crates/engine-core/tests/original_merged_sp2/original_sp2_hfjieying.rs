//! Golden coverage for 界营（sp2 韩馥）选人授技 + 杀加目标（ABI 465）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HFJIEYING: u32 = 2_801_808_452;
const HFJIEYING2: u32 = 4_111_610_111;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/hfjieying.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_end_phase(
    tx: Transaction,
    player: PlayerId,
) -> Result<Transaction, engine_core::effect::EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn grants_temp_skill_and_adds_sha_target() {
    let package = content_compiler::compile_source_named("hfjieying-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 465);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, recipient, other) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HFJIEYING),
    });
    run_effects(&mut tx, effects).unwrap();

    // Owner's end phase: confirm + choose the recipient.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    let tx = fire_end_phase(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, recipient.raw()); // choose the recipient
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == recipient && skill.def.raw() == HFJIEYING2),
        "recipient gains hfjieying2"
    );

    // Recipient's play phase: uses a Sha; the temp skill offers an extra target.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = recipient;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, recipient)
        .expect("recipient hand");
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: recipient,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(sha);
    let committed = tx.commit();
    let revision = committed.state.revision;
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: recipient,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    let choice = result.choice.expect("extra-target choice pending");
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, recipient);
}
