//! Golden coverage for MOBILE 界曹休「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIN_CAOXIU: u32 = 2_884_167_754;
const QIANJU: u32 = 2_828_017_083;
const XINQINGXI: u32 = 2_962_941_234;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinqingxi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    seats: Vec<PlayerId>,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinqingxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
        skill_def: SkillDefId::new(XINQINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        seats,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = deal_damage(state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

#[test]
fn package_publishes_abi322_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 322);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINQINGXI && skill.name == "xinqingxi"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINQINGXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_other_damage_before_once_per_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "xin_caoxiu");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [QIANJU, XINQINGXI]);

    let compiled = content_compiler::compile_source_named("xinqingxi", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn distance_two_target_discards_exactly_two_for_non_sha_damage() {
    let mut setup = setup(2_026_073_001);
    let target = setup.seats[2];
    let cards: Vec<_> = [99_001, 99_002, 99_003]
        .into_iter()
        .map(|def| inject_hand(&mut setup.state, target, def))
        .collect();
    let (state, accepted) = accept(setup.state, setup.owner, target, 1);
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 2);
    assert_eq!(choice.max_count, 2);
    let (state, result) = resolve(
        state,
        target,
        vec![cards[0].raw(), cards[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 9);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn insufficient_hand_or_cancel_increases_the_bound_damage() {
    let mut insufficient = setup(2_026_073_002);
    let target = insufficient.seats[1];
    for def in [99_011, 99_012] {
        inject_hand(&mut insufficient.state, target, def);
    }
    let (state, result) = accept(insufficient.state, insufficient.owner, target, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.hand_count(target), 2);

    let mut cancelled = setup(2_026_073_003);
    let target = cancelled.seats[1];
    for def in [99_021, 99_022, 99_023] {
        inject_hand(&mut cancelled.state, target, def);
    }
    let (state, accepted) = accept(cancelled.state, cancelled.owner, target, 4);
    assert_eq!(accepted.choice.unwrap().min_count, 3);
    let (state, result) = resolve(state, target, Vec::new(), true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.hand_count(target), 3);
}

#[test]
fn accepted_trigger_is_limited_to_once_in_the_current_turn() {
    let mut setup = setup(2_026_073_004);
    let target = setup.seats[2];
    for def in [99_031, 99_032] {
        inject_hand(&mut setup.state, target, def);
    }
    let (state, _) = accept(setup.state, setup.owner, target, 6);
    let (state, result) = resolve(state, target, Vec::new(), true, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);

    let (state, result) = deal_damage(state, setup.owner, target);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, 7);
}

#[test]
fn increased_elemental_damage_updates_chain_base_without_retriggering() {
    let mut setup = setup(2_026_073_005);
    let target = setup.seats[2];
    let other = setup.seats[3];
    for def in [99_041, 99_042] {
        inject_hand(&mut setup.state, target, def);
    }
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: target,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let mut tx = Transaction::begin(tx.commit().state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    );
    let state = tx.commit().state;
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], false, 8);
    let (state, result) = resolve(state, target, Vec::new(), true, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.players.get(other).unwrap().hp, 8);
}
