//! O2 全量合并第二批（2026-10-14）：家族 a1 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_a1_choose_target_hand_card_to_pile {
//! 第二百零八批：A1 他人置入原语 `choose_target_hand_card_to_pile` 行为测试。
//! 语义：选一名其他有手牌角色 → 选其一张手牌 → 移入 (player, pile_def) 技能堆
//! （两段选择续体 Cont 294）。消费样板：olquanji 权计族。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PILE_DEF: u32 = 3_141_592_653;

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let source = format!(
        "skill probe_a1({}) {{\n  trigger phase_prepare owner_is_target {{\n    effect choose_target_hand_card_to_pile(self, self_skill, {});\n  }}\n}}\n",
        9001005, PILE_DEF
    );
    let compiled = content_compiler::compile_source_named("a1-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(9001005),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    (next, result)
}

#[test]
fn a1_primitive_moves_target_hand_card_into_pile() {
    let (state, owner, other) = setup(20_260_918_01);
    let (state, card) = inject_hand_card(state, other, 9_700);

    // 发射：forced 触发 → dispatch 返回 effects → run_effects 执行
    // choose_target_hand_card_to_pile → stage 0 SelectTargets 打开（ChoiceRequested）。
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "stage 0 SelectTargets must open, got {outcome:?}"
    );
    let state = tx.commit().state;
    assert!(state.has_pending_choice());

    // 段一：选目标 other。
    let (state, _) = resolve(state, owner, vec![other.raw()], 20_260_918_11);
    assert!(state.has_pending_choice(), "SelectCards must follow");

    // 段二：选那张手牌 → 移入堆。
    let (state, _) = resolve(state, owner, vec![card.raw()], 20_260_918_12);
    assert!(!state.has_pending_choice());

    // 断言：卡在 (owner, PILE_DEF) 技能堆里。
    assert_eq!(
        state.skill_pile_cards(owner, PILE_DEF).len(),
        1,
        "card moved into pile"
    );
    assert_eq!(
        state.skill_pile_cards(owner, PILE_DEF)[0],
        card,
        "the chosen card"
    );
    // other 保留 bootstrap 手牌（本 harness 初始 4 张），仅被选的那张移出。
    assert_eq!(state.hand_count(other), 4, "other keeps bootstrap hand");
}
}
