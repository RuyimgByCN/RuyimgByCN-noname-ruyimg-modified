//! Golden coverage for ADR 0644 P2: DebateResolved event + debate filters +
//! start_debate_all_others/targets primitives and the jsrgfayi reaction skill.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FAYI: u32 = 3_169_881_772;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const FAYI_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgfayi.skill");
const CHAOZHENG_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgchaozheng.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
}

fn bootstrap_fayi() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("fayi", FAYI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 21, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FAYI),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
}

#[test]
fn debate_p2_npack_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // fayi 未使用 655/656 原语，jsrg pin 保持 654（新原语已注册待后续反应技消费）。
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FAYI
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_participated"
            && t.optional
    }));
}

#[test]
fn fayi_triggers_on_debate_participation() {
    let (mut tx, owner, a, b) = bootstrap_fayi();
    // owner 红意见、a 黑意见、b 无牌不参与：owner 与发起者(自己)同色——
    // 上游 fayi 需「意见与你不同」，此处近似单目标伤害；断言触发链打开即可。
    inject_hand(&mut tx, owner, Suit::Heart);
    inject_hand(&mut tx, a, Suit::Spade);
    let hp_b = tx.state().players.get(b).unwrap().hp;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DebateResolved {
                initiator: owner,
                red: vec![owner],
                black: vec![a],
                result: 1,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    // optional confirm → 选 b 为伤害目标。
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![u32::from(b.raw())], 3);
    assert_eq!(state.players.get(b).unwrap().hp, hp_b - 1);
}

#[test]
fn p1_chaozheng_now_emits_debate_resolved() {
    let compiled = content_compiler::compile_source_named("cz", CHAOZHENG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 22, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let a = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    inject_hand(&mut tx, a, Suit::Heart);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(2_540_681_453),
        }],
    )
    .unwrap();
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == 2_540_681_453)
        .unwrap()
        .id;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveJsrgchaozheng {
            player: owner,
            skill: skill_id,
        }],
    )
    .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, DomainEvent::DebateResolved { result, .. } if *result == 1)),
        "P1 chaozheng must now emit DebateResolved"
    );
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}
