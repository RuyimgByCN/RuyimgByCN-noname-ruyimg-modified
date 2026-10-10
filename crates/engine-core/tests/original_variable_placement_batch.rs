//! 第二百一十七批：变张置入原语 `choose_hand_cards_to_skill_pile` 契约与消费测试
//! （tag 714 / ABI 691 / Cont 298，ADR 0641 §8.12）。
//! 语义：从自身手牌选 [min,max] 张置入自身技能堆；可取消=不置入；
//! 上限/下限以当前手牌数钳位；max==0 或空手 no-op。
//! 消费：tunchu 屯储（mobile）、ymfriendyance（offline）、tsumugi_huilang 回浪（key）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TUNCHU: u32 = 3_943_350_142;
const TUNCHU_SRC: &str = include_str!("../../../content/original/ported/mobile/tunchu.skill");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const YMFRIENDYANCE: u32 = 3_114_639_769;
const YMFRIENDYANCE_SRC: &str =
    include_str!("../../../content/original/ported/offline/ymfriendyance.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const HUILANG: u32 = 4_008_390_212;
const HUILANG_SRC: &str =
    include_str!("../../../content/original/ported/key/tsumugi_huilang.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

fn setup_with(
    source: &str,
    tag: &str,
    skill_def: u32,
    seed: u64,
    players: u8,
) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Result<(), String>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
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
    match result.result {
        CommandResult::Accepted { .. } => (next, Ok(())),
        other => (next, Err(format!("{other:?}"))),
    }
}

fn probe_source() -> String {
    format!(
        "skill probe_var({}) {{\n  trigger phase_end owner_is_target optional {{\n    effect choose_hand_cards_to_skill_pile(self, self_skill, 1, 2);\n  }}\n}}\n",
        900_5001
    )
}

#[test]
fn variable_placement_places_two_and_cancel_places_none() {
    let source = probe_source();
    let (state, owner, _other) =
        setup_with(&source, "var-place-b217", 900_5001, 20_260_923_01, 2);
    let (state, c0) = inject_hand_card(state, owner, 9_910);
    let (state, c1) = inject_hand_card(state, owner, 9_911);
    let (state, _c2) = inject_hand_card(state, owner, 9_912);
    let hand_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "confirm must open"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], false, 20_260_923_11).0;
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChooseHandCardsToSkillPile,
        "Cont 298 routing"
    );

    // 取消 = 不置入（合法路径）。
    let (state, outcome) = resolve(state, owner, vec![], true, 20_260_923_12);
    assert!(outcome.is_ok(), "cancel must be accepted: {outcome:?}");
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before, "cancel: hand intact");
    assert_eq!(state.skill_pile_cards(owner, 900_5001).len(), 0);

    // 再次触发：选两张（min=1 max=2 → 2 可行）。
    let (state, _) = fire_again_and_confirm(state, owner, 20_260_923_13);
    let (state, outcome) = resolve(state, owner, vec![c0.raw(), c1.raw()], false, 20_260_923_14);
    assert!(outcome.is_ok(), "place 2: {outcome:?}");
    assert_eq!(state.hand_count(owner), hand_before - 2);
    assert_eq!(state.skill_pile_cards(owner, 900_5001).len(), 2);
}

fn fire_again_and_confirm(
    state: GameState,
    owner: PlayerId,
    key: u64,
) -> (GameState, GameState) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![1], false, key);
    (state.clone(), state)
}

#[test]
fn tunchu_draws_two_and_places() {
    // 契约：包内触发存在（phase_draw、optional）。
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 691);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TUNCHU)
        .expect("tunchu trigger");
    assert_eq!(trigger.event, "phase_draw");

    // 行为：摸牌阶段 → confirm → 摸 2 + 变张置入 1 张。
    let (state, owner, _other) = setup_with(TUNCHU_SRC, "tunchu-b217", TUNCHU, 20_260_923_02, 2);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![1], false, 20_260_923_21);
    assert!(state.has_pending_choice(), "placement choice must open");
    let pick = state.hand_cards(owner).first().copied().unwrap();
    let (state, outcome) = resolve(state, owner, vec![pick.raw()], false, 20_260_923_22);
    assert!(outcome.is_ok());
    assert_eq!(state.hand_count(owner), hand_before + 2 - 1, "draw 2, place 1");
    assert_eq!(state.skill_pile_cards(owner, TUNCHU).len(), 1);
}

#[test]
fn ymfriendyance_contract_and_five_card_placement() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 691);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == YMFRIENDYANCE)
        .expect("ymfriendyance trigger");
    assert_eq!(trigger.event, "round_started");

    // 行为：每轮开始 → confirm → 摸 5 + 恰好置 5 张。
    let (state, owner, _other) =
        setup_with(YMFRIENDYANCE_SRC, "ymfy-b217", YMFRIENDYANCE, 20_260_923_03, 2);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::RoundStarted {
            round: 1,
            current: owner,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![1], false, 20_260_923_31);
    let picks: Vec<u32> = state
        .hand_cards(owner)
        .iter()
        .take(5)
        .map(|card| card.raw())
        .collect();
    assert_eq!(picks.len(), 5, "hand must hold at least 5 after draw");
    let (state, outcome) = resolve(state, owner, picks, false, 20_260_923_32);
    assert!(outcome.is_ok());
    assert_eq!(state.hand_count(owner), hand_before, "draw 5, place 5");
    assert_eq!(state.skill_pile_cards(owner, YMFRIENDYANCE).len(), 5);
}

#[test]
fn huilang_contract_and_placement() {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 691);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HUILANG)
        .expect("huilang trigger");
    assert_eq!(trigger.event, "phase_end");

    // 行为：回合结束 → confirm → 置入一张。
    let (state, owner, _other) = setup_with(HUILANG_SRC, "huilang-b217", HUILANG, 20_260_923_04, 2);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![1], false, 20_260_923_41);
    let pick = state.hand_cards(owner).first().copied().unwrap();
    let (state, outcome) = resolve(state, owner, vec![pick.raw()], false, 20_260_923_42);
    assert!(outcome.is_ok());
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(state.skill_pile_cards(owner, HUILANG).len(), 1);
}
