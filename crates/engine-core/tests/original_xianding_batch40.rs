//! Golden for XIANDING（限定包）批次四十：
//! 势援（shiyuan）。

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

const SHIYUAN: u32 = 4_115_301_600;
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
fn package_exposes_batch40_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIYUAN)
        .expect("shiyuan trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_is_target_other_source_alive");
    assert!(trigger.optional);
}

#[test]
fn shiyuan_draws_three_when_source_hp_higher() {
    let (mut tx, owner, first, _second) = setup(20_260_894);
    add_skill(&mut tx, owner, SHIYUAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    let card = inject_card(&mut tx, first, CardKind::Sha);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 3);
}
