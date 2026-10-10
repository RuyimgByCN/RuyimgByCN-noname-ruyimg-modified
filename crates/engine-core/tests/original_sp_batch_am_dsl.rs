//! Golden for sp batch-AM DSL rewrites:
//! 庐江 (pxlujiang) on-gain optional: compare with a chosen player; the winner
//!   draws two (win/lose synthetic sub-skills via the existing
//!   choose_hand_compare primitive);
//! 肃州 (pxsizhou) on-gain optional: gain one hand card of another player
//!   (approximated by choose_other_give_one_hand_equip, chooser/zone annotated).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PXLUJIANG: u32 = 3_832_025_639;
const PXLUJIANG_WIN: u32 = 1_378_056_785;
const PXLUJIANG_LOSE: u32 = 1_747_940_174;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp/pxlujiang.skill"),
);

fn run_big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("batch-am", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_081_560, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 100;
        participant.max_hp = 100;
    }
    (tx.commit().state, seats[0], seats[1])
}

fn add_skill(state: GameState, player: PlayerId, skill_def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn dispatch_compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: CardId::NONE,
            target_card: CardId::NONE,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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

fn inject_hand(state: &mut GameState, player: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(engine_core::state::Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_000 + tag),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_am_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(
            trigger.skill_def_id,
            PXLUJIANG | PXLUJIANG_WIN | PXLUJIANG_LOSE
        ) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(
        PXLUJIANG,
        "skill_added",
        "owner_skill_added_self_3832025639",
        true
    )));
    assert!(seen.contains(&(
        PXLUJIANG_WIN,
        "hand_compare_finished",
        "owner_hand_compare_finished_participant",
        false
    )));
    assert!(seen.contains(&(
        PXLUJIANG_LOSE,
        "hand_compare_finished",
        "owner_hand_compare_finished_participant",
        false
    )));

}

#[test]
fn pxlujiang_winner_subskill_draws_two() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // The holder of the win sub-skill draws two when the compare finishes.
        let state = add_skill(state, target, PXLUJIANG_WIN);
        let owner_before = state.hand_count(owner);
        let target_before = state.hand_count(target);
        let state = dispatch_compare(state, owner, target, target);
        assert_eq!(state.hand_count(target), target_before + 2);
        assert_eq!(state.hand_count(owner), owner_before);
    });
}

#[test]
fn pxlujiang_loser_subskill_draws_nothing() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        let state = add_skill(state, target, PXLUJIANG_LOSE);
        let target_before = state.hand_count(target);
        // The opponent wins: the loser's sub-skill only removes itself.
        let state = dispatch_compare(state, owner, target, owner);
        assert_eq!(state.hand_count(target), target_before);
    });
}

#[test]
fn pxlujiang_on_gain_opens_compare_target_selection() {
    run_big_stack(|| {
        let (mut state, owner, _target) = setup();
        // Both sides need hand cards for the compare to be legal.
        inject_hand(&mut state, owner, 1);
        inject_hand(&mut state, _target, 2);
        let mut tx = Transaction::begin(state);
        let effects = run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(PXLUJIANG),
            }],
        );
        assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
        assert_eq!(
            tx.state().resolution.continuation_kind,
            ContinuationKind::OptionalTrigger
        );
        let tx = tx.commit();
        // Confirm → the compare target selection opens.
        let (state, result) = resolve(tx.state, owner, vec![1], 1);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert!(state.has_pending_choice());
    });
}

