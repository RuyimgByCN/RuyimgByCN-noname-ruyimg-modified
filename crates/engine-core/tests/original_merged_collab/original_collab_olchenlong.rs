//! Golden coverage for COLLAB「辰龙」and the complete ol_chenlong closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLCHENLONG: u32 = 2_779_773_800;
const OL_CHENLONG: u32 = 3_917_415_586;
const SOURCE: &str = include_str!("../../../../content/original/ported/collab/olchenlong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

fn inject_tao(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(108_001),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn neutralize_existing_save_cards(state: &mut GameState) {
    let players = state.players.alive_by_seat();
    for player in players {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        let cards = state.zones.get(hand).unwrap().cards.clone();
        for card in cards {
            let physical = state.cards.get_mut(card).unwrap();
            if matches!(physical.kind, CardKind::Tao | CardKind::Jiu) {
                physical.kind = CardKind::Sha;
            }
        }
    }
}

fn setup() -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("olchenlong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, OLCHENLONG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLCHENLONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = *tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .last()
        .unwrap();
    (tx.commit().state, owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice: engine_api::ChoiceId,
    token: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![token],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_numeric_active_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package.manifest.capabilities.request_choice);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLCHENLONG && skill.name == "olchenlong"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLCHENLONG)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_CHENLONG
            && character.skills == vec![OLCHENLONG]
            && character.max_hp == 4
            && character.sex == "male"
            && character.group == "qun"
    }));
}

#[test]
fn nonlethal_choice_loses_and_deals_the_same_amount_without_max_hp_cost() {
    let (mut state, owner, target, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 4;
    state.players.get_mut(target).unwrap().hp = 4;
    let owner_max = state.players.get(owner).unwrap().max_hp;

    let (waiting, activated) = activate(state, owner, target, skill, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("1/2 amount choice");
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, resolved) = resolve(waiting, owner, choice.choice_id, 2, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(state.players.get(owner).unwrap().max_hp, owner_max);
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn exact_self_loss_dying_reduces_max_hp_and_defers_target_damage_until_saved() {
    let (mut state, owner, target, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(target).unwrap().hp = 4;
    let owner_max = state.players.get(owner).unwrap().max_hp;
    neutralize_existing_save_cards(&mut state);
    let tao = inject_tao(&mut state, target);

    let (waiting, activated) = activate(state, owner, target, skill, 10);
    let amount_choice = activated.choice.expect("amount choice");
    let (dying, amount_resolved) = resolve(waiting, owner, amount_choice.choice_id, 1, 11);
    assert!(matches!(
        amount_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(dying.players.get(owner).unwrap().hp, 0);
    assert!(dying.players.get(owner).unwrap().dying);
    assert_eq!(dying.players.get(owner).unwrap().max_hp, owner_max - 1);
    assert_eq!(
        dying.players.get(target).unwrap().hp,
        4,
        "target damage must wait behind the dying barrier"
    );
    assert!(amount_resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::MaxHpChanged { player, new_max }
            if *player == owner && *new_max == owner_max - 1
    )));

    let save = amount_resolved.choice.expect("dying save choice");
    assert_eq!(save.observer, target);
    let (state, saved) = resolve(dying, target, save.choice_id, 1, 12);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
