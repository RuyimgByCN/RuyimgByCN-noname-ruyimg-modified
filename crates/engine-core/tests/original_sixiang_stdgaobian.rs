//! Golden coverage for 高变（sixiang 标王沈，ABI 468）.

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

const STDGAOBIAN: u32 = 4_024_217_730;
const STDANRAN: u32 = 2_536_614_283;
const ANRAN_MARK: u32 = 221_181_016;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdgaobian.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                selections,
                cancelled: false,
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

fn fire_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
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

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDGAOBIAN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn sha_branch_uses_virtual_sha() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 468);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    // Empty the victim's hand so the virtual Sha cannot be dodged.
    let victim_hand = tx.state().zones.find(ZoneKind::Hand, victim).unwrap();
    let victim_cards: Vec<CardId> = tx.state().zones.get(victim_hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        victim_cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(victim_hand),
            })
            .collect(),
    )
    .unwrap();
    // The chooser takes 1 damage this turn (exactly one damaged player).
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: chooser,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(
        tx.state().turn.player_received_damage(chooser),
        "chooser recorded as damaged this turn"
    );
    assert_eq!(tx.state().turn.damaged_player_count(), 1);
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "branch choice pending");
    state = drive_choice(state, chooser, vec![0]); // use a Sha
    assert!(state.has_pending_choice(), "Sha target choice pending");
    state = drive_choice(state, chooser, vec![victim.raw()]);

    // The virtual Sha hits the victim (no 闪 in hand).
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "virtual Sha dealt 1 damage"
    );
}

#[test]
fn reset_branch_clears_anran_counter() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, _victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(STDANRAN),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(ANRAN_MARK),
                delta: 2,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: chooser,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let mut state = tx.commit().state;
    state = drive_choice(state, chooser, vec![1]); // reset Anran

    let owner_state = state.players.get(owner).unwrap();
    assert!(
        !owner_state
            .marks
            .iter()
            .any(|(mark, count)| *mark == ANRAN_MARK && *count > 0),
        "Anran counter cleared: {:?}",
        owner_state.marks
    );
}

#[test]
fn no_choice_when_two_players_damaged() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DealDamage {
                source: owner,
                target: chooser,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: victim,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.damaged_player_count(), 2);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "no choice when two players were damaged"
    );
}
