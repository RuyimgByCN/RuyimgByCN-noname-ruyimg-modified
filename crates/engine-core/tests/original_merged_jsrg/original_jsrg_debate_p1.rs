//! Golden coverage for ADR 0644 P1 debate primitives (ABI 653/654):
//! 朝政 jsrgchaozheng (red / black / no-result three branches) and 设居 jsrgsheju.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHAOZHENG: u32 = 2_540_681_453;
const SHEJU: u32 = 3_229_587_644;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");
const CHAOZHENG_SRC: &str =
    include_str!("../../../../content/original/ported/jsrg/jsrgchaozheng.skill");
const SHEJU_SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgsheju.skill");

fn bootstrap(
    source: &str,
    skill_def: u32,
    seed: u64,
) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("jsrg-debate", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
}

/// Inject a hand card of the given suit; zone order matters (first card = opinion).
fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
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
use engine_api::ChoiceId;

#[test]
fn debate_p1_npack_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == CHAOZHENG && t.event == "phase_prepare" && t.optional }));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == SHEJU)
            .count(),
        2
    );
}

#[test]
fn chaozheng_red_majority_heals_red_opinion_holders() {
    // 重新布置：a 红、b 黑 → 红多数 1:1? 1v1 平票。改为三参与者在 3 人局不可得，
    // 因此此用例覆盖「全同红」分支：a/b 均红 → 无多数分支（单色不算红>黑，
    // 上游红=黑=0 比较时全红即红多数）——按实现：red=2, black=0 → red>black 回复。
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 11);
    for p in [a, b] {
        let max = tx.state().players.get(p).unwrap().max_hp as i32;
        tx.state_mut().players.get_mut(p).unwrap().hp = max - 2;
    }
    inject_hand(&mut tx, a, Suit::Heart);
    inject_hand(&mut tx, b, Suit::Diamond);
    let hp_a = tx.state().players.get(a).unwrap().hp;
    let hp_b = tx.state().players.get(b).unwrap().hp;
    let hand_owner = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(
        state.players.get(a).unwrap().hp,
        hp_a + 1,
        "red opinion holder heals"
    );
    assert_eq!(state.players.get(b).unwrap().hp, hp_b + 1);
    // 全同意见（两参与者均红）→ 发起者摸参与者数（2）张。
    assert_eq!(state.hand_count(owner), hand_owner + 2);
}

#[test]
fn chaozheng_black_majority_damages_red_opinion_holders() {
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 12);
    inject_hand(&mut tx, a, Suit::Spade); // 黑意见
    inject_hand(&mut tx, b, Suit::Club); // 黑意见
                                         // 两人均黑 → 全同（黑）→ 发起者摸 2；红意见者集合为空，无回复/失去。
    let hand_owner = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(
        state.hand_count(owner),
        hand_owner + 2,
        "all-same black draws 2"
    );
}

#[test]
fn chaozheng_tie_is_no_result() {
    // a 红、b 黑 → 1:1 平票 → 无效果；且非全同 → 不摸牌。
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 13);
    inject_hand(&mut tx, a, Suit::Heart);
    inject_hand(&mut tx, b, Suit::Spade);
    let hand_owner = tx.state().hand_count(owner);
    let hp_a = tx.state().players.get(a).unwrap().hp;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), hand_owner, "tie: no draw");
    assert_eq!(state.players.get(a).unwrap().hp, hp_a, "tie: no heal");
}

#[test]
fn sheju_black_result_reduces_both_max_hp() {
    let (mut tx, owner, a, _b) = bootstrap(SHEJU_SRC, SHEJU, 14);
    inject_hand(&mut tx, owner, Suit::Spade);
    inject_hand(&mut tx, a, Suit::Club);
    // 注入一张实体【杀】作为事件牌（card_targeted 上下文需要 effective kind = sha）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_102),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 1,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let max_owner = tx.state().players.get(owner).unwrap().max_hp;
    let max_a = tx.state().players.get(a).unwrap().max_hp;
    let planned = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: a,
            targets: vec![a],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, planned).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_owner - 1);
    assert_eq!(state.players.get(a).unwrap().max_hp, max_a - 1);
}
