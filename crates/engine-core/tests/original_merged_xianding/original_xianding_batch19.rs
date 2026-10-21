//! Golden for XIANDING（限定包）批次十九：
//! 立刃（liren）、讽言（dcfengyan）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIREN: u32 = 3_368_059_686;
const DCFENGYAN: u32 = 2_692_338_281;
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
fn package_exposes_batch19_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let range = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == LIREN && modifier.target == "attack_range")
        .expect("liren attack range modifier");
    assert_eq!(range.mode, "override");
    assert_eq!(range.value, 2);

    let dodged = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIREN)
        .expect("liren trigger");
    assert_eq!(dodged.event, "sha_dodged");
    assert_eq!(dodged.filter, "owner_source_sha_dodged");
    assert!(!dodged.optional);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCFENGYAN)
        .expect("dcfengyan active");
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|t| t.hp_lte_owner));
}

#[test]
fn liren_draws_when_sha_dodged() {
    let (mut tx, owner, first, _second) = setup(20_260_845);
    add_skill(&mut tx, owner, LIREN);
    let sha = inject_card(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::ShaDodged {
            source: owner,
            target: first,
            card: sha,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcfengyan_takes_one_card_from_lower_hp_target() {
    let (mut tx, owner, first, _second) = setup(20_260_846);
    add_skill(&mut tx, owner, DCFENGYAN);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(first).unwrap().hp = owner_hp;
    let card = inject_card(&mut tx, first, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(DCFENGYAN))
        .expect("dcfengyan live skill")
        .id;
    let revision = tx.state().revision;
    let (tx2, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        tx2.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    let mut tx = Transaction::begin(tx2);
    let state = resolve(tx.state().clone(), owner, vec![card.raw()], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state
        .zones
        .get(state.zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .contains(&card));
}
