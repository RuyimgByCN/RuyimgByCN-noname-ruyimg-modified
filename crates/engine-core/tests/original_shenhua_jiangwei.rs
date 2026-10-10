//! Golden coverage for 挑衅/志继（shenhua 姜维，ABI 537-538）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TIAOXIN: u32 = 695_232_805;
const ZHIJI: u32 = 1_739_094_753;
const GUANXING: u32 = 32;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/jiangwei.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jiangwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIAOXIN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
        .unwrap()
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = skill_instance(&state, owner, TIAOXIN);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

#[test]
fn package_exposes_tiaoxin_zhiji_contract() {
    let compiled = content_compiler::compile_source_named("jiangwei", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 538);
    let tiaoxin = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == TIAOXIN)
        .unwrap();
    assert_eq!(tiaoxin.window, "play_phase");
    assert_eq!(tiaoxin.uses_per_turn, Some(1));
    let zhiji = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZHIJI)
        .unwrap();
    assert_eq!(zhiji.event, "phase_prepare");
    assert_eq!(zhiji.filter, "owner_phase_begin_hand_empty");
    assert!(!zhiji.optional);
    assert_eq!(compiled.skill_index.len(), 2);
    assert_eq!(compiled.trigger_index.len(), 1);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn tiaoxin_refusal_discards_one_card_of_the_target() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let victim = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_400,
        Suit::Heart,
        1,
    );
    let spare = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_401,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    // 挑衅: owner picks the adjacent target, the target refuses, the owner discards one card.
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let target_choice = activated.choice.as_ref().unwrap();
    assert_eq!(target_choice.observer, setup.owner);
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        option_tokens(&activated),
        [setup.other.raw()],
        "cardless far target excluded"
    );
    let (state, picked) = resolve(state, vec![setup.other.raw()], false, 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let response_choice = picked.choice.as_ref().unwrap();
    assert_eq!(response_choice.observer, setup.other);
    assert_eq!(response_choice.kind, ChoiceKind::Custom);
    // Refuse: selections [0].
    let (state, refused) = resolve(state, vec![0], false, 3);
    assert!(matches!(refused.result, CommandResult::Accepted { .. }));
    let discard_choice = refused.choice.as_ref().unwrap();
    assert_eq!(discard_choice.observer, setup.owner);
    assert_eq!(discard_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&refused), [victim.raw(), spare.raw()]);
    let (state, discarded) = resolve(state, vec![victim.raw()], false, 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard_zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard_zone);
    assert_eq!(state.cards.get(spare).unwrap().zone, other_hand);
    // Second activation this play phase is rejected by the once-per-turn contract.
    let (_, repeated) = activate(state, setup.owner, 5);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn tiaoxin_forced_sha_hits_the_owner() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_410,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, vec![setup.other.raw()], false, 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    // The target chooses to use a Sha.
    let (state, chose_sha) = resolve(state, vec![1], false, 3);
    assert!(matches!(chose_sha.result, CommandResult::Accepted { .. }));
    let source_choice = chose_sha.choice.as_ref().unwrap();
    assert_eq!(source_choice.observer, setup.other);
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&chose_sha), [sha.raw()]);
    let (state, used) = resolve(state, vec![sha.raw()], false, 4);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    // The Sha targeted only the owner: no dodge was injected, so the lord owner
    // (5 max HP) takes 1 damage.
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 4);
    assert!(!state.has_pending_choice());
}

#[test]
fn tiaoxin_skips_out_of_range_or_cardless_targets() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Give the adjacent player a card and nothing to the far player.
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_420,
        Suit::Heart,
        1,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        option_tokens(&activated),
        [setup.other.raw()],
        "cardless far target excluded"
    );
    // Cancelling the target choice rejects (挑衅 requires exactly one target).
    let (_, rejected) = resolve(state, vec![], true, 2);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn zhiji_awakens_with_empty_hand() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // The owner has no hand cards and full HP; lord bonus max hp is 5.
    assert!(tx.state().hand_cards(setup.owner).is_empty());
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut tx = match fired {
        Ok(effects) => {
            let mut next = Transaction::begin(state);
            run_effects(&mut next, effects).unwrap();
            next
        }
        Err(EffectError::ChoiceRequested) => Transaction::begin(state),
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        4,
        "recovered to max 4/5"
    );
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced from 5 to 4"
    );
    assert_eq!(state.hand_cards(setup.owner).len(), 2, "drew two cards");
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == GUANXING),
        "观星 granted"
    );
}

#[test]
fn zhiji_does_not_fire_with_hand_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_430,
        Suit::Heart,
        1,
    );
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        5,
        "no awakening with hand cards"
    );
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == setup.owner && skill.def.raw() == GUANXING));
}
