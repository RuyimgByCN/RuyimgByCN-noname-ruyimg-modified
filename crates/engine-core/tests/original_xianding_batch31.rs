//! Golden for XIANDING（限定包）批次三十一：
//! 追还·二（zhuihuan2）、沙舞（shawu）。

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

const ZHUHUAN2: u32 = 3_807_480_043;
const SHAWU: u32 = 3_835_073_328;
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
fn package_exposes_batch31_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let zhuihuan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUHUAN2)
        .expect("zhuihuan2 trigger");
    assert_eq!(zhuihuan.event, "damage");
    assert_eq!(
        zhuihuan.filter,
        "owner_is_target_other_source_alive_per_damage_point"
    );
    assert!(!zhuihuan.optional);

    let shawu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHAWU)
        .expect("shawu trigger");
    assert_eq!(shawu.event, "card_targeted");
    assert_eq!(shawu.filter, "owner_is_target_event_card_sha");
    assert!(shawu.optional);
}

#[test]
fn zhuihuan2_else_branch_discards_source_card_when_hp_not_higher() {
    let (mut tx, owner, first, _second) = setup(20_260_861);
    add_skill(&mut tx, owner, ZHUHUAN2);
    inject_card(&mut tx, first, CardKind::Sha);
    let hand_before = tx.state().hand_count(first);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let discard = tx
        .state()
        .zones
        .get(tx.state().zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .first()
        .copied()
        .unwrap();
    let state = resolve(tx.state().clone(), first, vec![discard.raw()], false, 1);
    assert_eq!(state.hand_count(first), hand_before - 1);
}

#[test]
fn shawu_damages_sha_source() {
    let (mut tx, owner, first, _second) = setup(20_260_862);
    add_skill(&mut tx, owner, SHAWU);
    let card = inject_card(&mut tx, first, CardKind::Sha);
    let hp = tx.state().players.get(first).unwrap().hp;

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
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
