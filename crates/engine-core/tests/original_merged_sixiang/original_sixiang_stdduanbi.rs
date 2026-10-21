//! Golden coverage for 标刘巴「锻币」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIUBA: u32 = 2_185_226_076;
const STDDUANBI: u32 = 3_811_377_323;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdduanbi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(player_count: u8) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("stdduanbi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDUANBI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDDUANBI && skill.name == "stdduanbi" && !skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBA)
        .unwrap();
    assert_eq!(character.name_key, "std_liuba");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDDUANBI]);
}

#[test]
fn discards_all_hand_then_forces_two_distinct_targets_and_draws_in_seat_order() {
    let (mut tx, players) = setup(3);
    let [owner, second, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let owner = *owner;
    let second = *second;
    let third = *third;
    let discarded = [
        inject(&mut tx, owner, 97_001),
        inject(&mut tx, owner, 97_002),
        inject(&mut tx, owner, 97_003),
    ];
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );

    let (state, target_choice) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(owner), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(discarded
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let choice = target_choice.choice.unwrap();
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![owner.raw(), second.raw(), third.raw()]
    );

    let before = canonical_state_hash(&state);
    let (same, duplicate) = resolve(state, owner, vec![second.raw(), second.raw()], 2);
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let draw_pile = same.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top_four: Vec<_> = same
        .zones
        .get(draw_pile)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(4)
        .copied()
        .collect();
    let third_hand = same.hand_count(third);
    let (state, drawn) = resolve(same, owner, vec![second.raw(), owner.raw()], 3);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(second), 2);
    assert_eq!(state.hand_count(third), third_hand);
    assert_eq!(state.cards.get(top_four[0]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_four[1]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_four[2]).unwrap().owner, second);
    assert_eq!(state.cards.get(top_four[3]).unwrap().owner, second);
}

#[test]
fn with_only_one_survivor_the_cost_is_paid_without_opening_target_choice() {
    let (mut tx, players) = setup(2);
    let owner = players[0];
    let other = players[1];
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    inject(&mut tx, owner, 97_011);
    inject(&mut tx, owner, 97_012);
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, vec![1], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 0);
    assert!(!state.has_pending_choice());
}
