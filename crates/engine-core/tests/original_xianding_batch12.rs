//! Golden for XIANDING（限定包）批次十二：
//! 诛寇（zhukou，周夷分支②）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUKOU: u32 = 4_022_084_191;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch12_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUKOU)
        .expect("zhukou trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_phase_end_used_card_no_damage");
    assert!(trigger.optional);
}

#[test]
fn zhukou_offers_two_other_damage_at_phase_end_when_no_damage_this_turn() {
    let (mut tx, owner, first, second) = setup(20_260_828);
    add_skill(&mut tx, owner, ZHUKOU);

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));
    tx.state_mut().turn.phase = Phase::End;
    let hp_first = tx.state().players.get(first).unwrap().hp;
    let hp_second = tx.state().players.get(second).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    let state = resolve(state, owner, vec![first.raw(), second.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp_first - 1);
    assert_eq!(state.players.get(second).unwrap().hp, hp_second - 1);
}

#[test]
fn zhukou_does_not_trigger_when_owner_caused_damage_this_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_829);
    add_skill(&mut tx, owner, ZHUKOU);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));

    tx.state_mut().turn.phase = Phase::End;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));
}
