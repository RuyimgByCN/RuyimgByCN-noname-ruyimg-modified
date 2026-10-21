//! Golden coverage for 标甘夫人「淑慎」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDSHUSHENX: u32 = 3_633_500_345;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdshushenx.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_412, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHUSHENX),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = engine_core::state::Phase::End;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    (tx, owner, target, source)
}

fn phase_end(current: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current,
        phase: TurnPhaseTag::End,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi140_and_complete_ganfuren() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDSHUSHENX && skill.name == "stdshushenx" && !skill.locked
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "std_ganfuren"));

    let compiled = content_compiler::compile_source_named("stdshushenx", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 139);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn draw_and_gift_share_turn_gain_count_then_recover_other() {
    let (mut tx, owner, target, source) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.card_gain_count(owner), 2);
    assert!(
        dispatch_for_events(&mut tx, &[phase_end(source)], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );

    let card = tx.state().hand_cards(source)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let moved = run_effects(
        &mut tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .unwrap();
    assert!(moved.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
    assert_eq!(tx.state().turn.card_gain_count(owner), 3);

    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(source)], &mut TriggerFireRecord::new(),)
            .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (selecting, selected) = resolve(committed.state, owner, vec![1], 1);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::SelectTargets)
    );
    let before = selecting.players.get(target).unwrap().hp;
    let (done, recovered) = resolve(selecting, owner, vec![target.raw()], 2);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before + 1);
}
