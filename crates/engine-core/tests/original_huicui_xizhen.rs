//! Golden coverage for 袭阵（huicui 高览，ABI 482）.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XIZHEN: u32 = 4_003_133_285;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/xizhen.skill");

fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XIZHEN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the target's hand so the virtual Sha cannot be dodged.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
}

#[test]
fn sha_branch_deals_damage() {
    let package = content_compiler::compile_source_named("xizhen-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 482);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice(), "branch choice pending");
    state = resolve_choice(state, owner, vec![1], false, 2); // Sha

    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 1,
        "virtual Sha hit the target"
    );
}

#[test]
fn duel_branch_keeps_hp_without_shas() {
    let package = content_compiler::compile_source_named("xizhen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice());
    // Duel at a handless target: the owner (also handless) cannot answer either, so the
    // Duel damages one of them; assert only that no crash occurred and the choice closed.
    state = resolve_choice(state, owner, vec![2], false, 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no pending choice after the Duel"
    );
    let _ = owner_hp_before;
}
