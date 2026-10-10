//! Golden coverage for 宽释（huicui 阚泽，ABI 480）.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REKUANSHI: u32 = 3_423_208_328;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/rekuanshi.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
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

#[test]
fn second_damage_of_turn_recovers_watched_player() {
    let package = content_compiler::compile_source_named("rekuanshi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 480);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, watched) = (seats[0], seats[1]);
    // Strip skills; give the owner 宽释.
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKUANSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    // Wound the watched player so the recover is meaningful.
    let max_hp = tx.state().players.get(watched).unwrap().max_hp;
    tx.state_mut().players.get_mut(watched).unwrap().hp = max_hp as i32 - 1;

    // End phase: choose the watched player.
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "watch target choice pending");
    state = resolve_choice(state, owner, vec![watched.raw()], false, 1);
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 1,
        "no recover yet"
    );

    // First damage this turn: no recover.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: watched,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 2,
        "first damage applied without recover"
    );

    // Second damage this turn: the watched player recovers 1.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: watched,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 2,
        "second damage applied, then recovered 1"
    );
}

#[test]
fn only_once_per_turn() {
    let package = content_compiler::compile_source_named("rekuanshi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, watched) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKUANSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    // Fixed 5 max HP so four damages keep the watched player alive even with
    // the single recovery, independent of the booted roster's max_hp.
    tx.state_mut().players.get_mut(watched).unwrap().max_hp = 5;
    let max_hp = 5;
    tx.state_mut().players.get_mut(watched).unwrap().hp = max_hp as i32;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![watched.raw()], false, 1);

    // Damage 1, 2 (recover), 3, 4: only the second damage recovers.
    for _ in 0u32..4 {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::CommitDamage {
                damage_id: 0,
                source: owner,
                target: watched,
                amount: 1,
                base_amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }],
        )
        .unwrap();
        state = tx.commit().state;
    }
    // 4 damage - 1 recovery = net 3 lost from full HP -> hp = maxHp - 3.
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 3,
        "only one recovery across four damages"
    );
}
