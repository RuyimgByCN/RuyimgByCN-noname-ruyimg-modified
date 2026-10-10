//! Golden coverage for MOBILE 手杀界朱然「胆守」(ABI 500).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MOBILEDANSHOU: u32 = 2_790_027_559;
const XIN_ZHURAN: u32 = 4_142_079_595;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mobiledanshou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("mobiledanshou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 500);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOBILEDANSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, target)
}

fn dispatch(tx: &mut Transaction, events: &[DomainEvent]) -> Result<(), EffectError> {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new())?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(state: GameState, owner: PlayerId, cards: Vec<u32>) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: cards,
                cancelled: false,
            },
        },
    )
}

#[test]
fn zero_targetings_draws_and_two_targetings_discard_then_damage() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(packaged.manifest.rule_vm_abi_version, 734);
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == XIN_ZHURAN)
            .unwrap()
            .skills,
        [MOBILEDANSHOU]
    );

    let (mut tx, owner, target) = setup(20_260_802);
    let hand_before = tx.state().hand_count(owner);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);

    let (mut tx, owner, target) = setup(20_260_803);
    let used = tx.state().hand_cards(target);
    for card in used.into_iter().take(2) {
        dispatch(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: target,
                card,
                targets: vec![owner],
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().turn.card_target_count(target, owner), 2);
    let payment: Vec<_> = tx.state().hand_cards(owner).into_iter().take(2).collect();
    let hp = tx.state().players.get(target).unwrap().hp;
    let result = dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let (state, result) = resolve(
        tx.commit().state,
        owner,
        payment.iter().map(|card| card.raw()).collect(),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(payment
        .iter()
        .all(|card| !state.hand_cards(owner).contains(card)));
}
