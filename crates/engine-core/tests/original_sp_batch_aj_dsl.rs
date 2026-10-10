//! Golden for sp batch-AJ DSL rewrites:
//! 青州 (pxqingzhou) on-gain optional: discard the whole hand, then draw the same
//!   count (count captured at plan time before the discard runs);
//! 寿春 (pxshouchun) on-gain optional: select any living player (self included);
//!   that player gains 1 max hp (new primitive choose_target_gain_max_hp, ABI 646).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXQINGZHOU: u32 = 2_570_205_671;
const PXSHOUCHUN: u32 = 3_720_575_836;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_700 + tag),
        kind,
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
fn package_publishes_batch_aj_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(trigger.skill_def_id, PXQINGZHOU | PXSHOUCHUN) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(
        PXQINGZHOU,
        "skill_added",
        "owner_skill_added_self_2570205671",
        true
    )));
    assert!(seen.contains(&(
        PXSHOUCHUN,
        "skill_added",
        "owner_skill_added_self_3720575836",
        true
    )));
}

#[test]
fn pxqingzhou_discards_all_hand_then_draws_same_count() {
    let (mut state, owner) = started(2_026_081_520);
    let marker = inject_hand(&mut state, owner, CardKind::Tao, 20);
    let hand_before = state.hand_count(owner);

    // Granting opens the optional on-gain confirm (OptionalTrigger).
    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXQINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // Confirm → discard all + draw the pre-discard count back.
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(!state.hand_cards(owner).contains(&marker));
}

#[test]
fn pxqingzhou_cancel_keeps_hand_intact() {
    let (mut state, owner) = started(2_026_081_521);
    let marker = inject_hand(&mut state, owner, CardKind::Tao, 21);
    let hand_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXQINGZHOU),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![], true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(state.hand_cards(owner).contains(&marker));
}

#[test]
fn pxshouchun_selected_other_gains_one_max_hp() {
    let (state, owner) = started(2_026_081_522);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_max_before = state.players.get(other).unwrap().max_hp;
    let other_hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // Confirm opens the target selection (any living player, self included).
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetGainMaxHp
    );
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let other_after = state.players.get(other).unwrap();
    assert_eq!(other_after.max_hp, other_max_before + 1);
    // Upstream gainMaxHp raises only the cap, never current hp.
    assert_eq!(other_after.hp, other_hp_before);
}

#[test]
fn pxshouchun_can_select_self() {
    let (state, owner) = started(2_026_081_523);
    let owner_max_before = state.players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, _) = resolve(tx.state, owner, vec![1], false, 5);
    let (state, result) = resolve(state, owner, vec![owner.raw()], false, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().max_hp, owner_max_before + 1);
}

#[test]
fn pxshouchun_cancelled_target_choice_is_noop() {
    let (state, owner) = started(2_026_081_524);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_max_before = state.players.get(other).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSHOUCHUN),
        }],
    );
    assert_eq!(effects.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, _) = resolve(tx.state, owner, vec![1], false, 7);
    // Cancel the target selection entirely.
    let (state, result) = resolve(state, owner, vec![], true, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().max_hp, other_max_before);
}
