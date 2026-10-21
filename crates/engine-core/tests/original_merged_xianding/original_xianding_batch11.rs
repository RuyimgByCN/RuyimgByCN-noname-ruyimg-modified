//! Golden for XIANDING（限定包）批次十一：
//! 独仗（dcduzhang）、风势（dcfengshi）。

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

const DCDUZHANG: u32 = 3_539_290_738;
const DCFENGSHI: u32 = 2_270_426_166;
const DCLINJIE: u32 = 3_067_413_516;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
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

fn inject_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
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

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCLINJIE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch11_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let duzhang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUZHANG)
        .expect("dcduzhang trigger");
    assert_eq!(duzhang.event, "card_targeted");
    assert_eq!(duzhang.filter, "owner_is_target_nonred_card");
    assert!(!duzhang.optional);

    let fengshi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCFENGSHI)
        .expect("dcfengshi trigger");
    assert_eq!(fengshi.event, "card_targeted");
    assert_eq!(
        fengshi.filter,
        "owner_source_sha_target_alive_owner_outside_target_attack_range"
    );
    assert!(fengshi.optional);

    let hand_limit = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == DCDUZHANG)
        .expect("dcduzhang modifier");
    assert_eq!(hand_limit.target, "hand_limit");
}

#[test]
fn dcduzhang_gains_mark_and_draws_when_targeted_by_nonred_card() {
    let (mut tx, owner, first, _second) = setup(20_260_826);
    add_skill(&mut tx, owner, DCDUZHANG);
    let before = tx.state().hand_count(owner);
    let card = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcfengshi_offers_discard_target_equip_when_owner_outside_target_range() {
    let (mut tx, owner, first, _second) = setup(20_260_827);
    add_skill(&mut tx, owner, DCFENGSHI);
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let horse = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(13),
        kind: CardKind::HorseDef,
        suit: Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(horse);
    let card = inject_sha(&mut tx, owner);
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
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
}
