//! O2 全量合并第二批（2026-10-14）：家族 spshicai 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_spshicai_active_trade {
//! 第二百二十八批：spshicai 识财晋升行为测试（active 弃一摸一，给予族复测散批）。
//! 主面：出牌阶段 active → discard_one_hand_or_equip(self)（SelectCards 1/1）
//! → draw_cards(self, 1)（牌堆顶入手）。子面（令一名角色使用所得+摸一张）登记债。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const SPSHICAI_DEF: u32 = 2_681_606_731;
const SPSHICAI_SRC: &str = include_str!("../../../content/original/ported/sp2/spshicai.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("spshicai-b228", SPSHICAI_SRC).unwrap();
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
        skill_def: SkillDefId::new(SPSHICAI_DEF),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, other)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

#[test]
fn spshicai_discard_then_draw_top() {
    let (state, owner, _other) = setup(20_261_001_01);
    let hand_before = state.hand_count(owner);
    let sid = skill_id(&state, owner, SPSHICAI_DEF);
    let revision = state.revision;

    // 发动：active → discard_one_hand_or_equip 的 SelectCards 打开。
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );
    assert!(state.has_pending_choice(), "discard selection opens");

    // 弃手牌首张 → 摸 1（净手牌数不变，牌堆顶一张入手）。
    let pick = state.hand_cards(owner).first().copied().unwrap();
    let state = resolve(state, owner, vec![pick.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before, "discard 1, draw 1");
    // 弃牌堆包含所弃之牌。
    let discard = state
        .zones
        .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(
        state.zones.get(discard).unwrap().cards.contains(&pick),
        "discarded card in discard pile"
    );
}
}
