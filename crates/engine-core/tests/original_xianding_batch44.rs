//! Golden for XIANDING（限定包）批次四十四：
//! 决（旧版，oldjue）、张采（dczhangcai）。

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

const OLDJUE: u32 = 2_642_038_327;
const DCZHANGCAI: u32 = 4_008_750_021;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
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
fn package_exposes_batch44_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDJUE)
        .expect("oldjue trigger");
    assert_eq!(jue.event, "phase_prepare");
    assert!(jue.optional);

    let zhangcai = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHANGCAI)
        .expect("dczhangcai trigger");
    assert_eq!(zhangcai.event, "card_used");
    assert!(zhangcai.optional);
}

#[test]
fn oldjue_chooses_virtual_sha_target_at_prepare_phase() {
    let (mut tx, owner, first, _second) = setup(20_260_895);
    add_skill(&mut tx, owner, OLDJUE);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
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
        ContinuationKind::VirtualShaTargets
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
}

#[test]
fn dczhangcai_draws_one_when_owner_uses_card() {
    let (mut tx, owner, first, _second) = setup(20_260_896);
    add_skill(&mut tx, owner, DCZHANGCAI);
    let card = inject_card(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}
