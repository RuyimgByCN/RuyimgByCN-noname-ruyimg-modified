//! 第一百八十四批：refresh「志继」（OL姜维 olzhiji）觉醒技零新原语重写。
//! 上游：准备阶段/结束阶段（forced juexingji），若没有手牌：减 1 点体力上限，
//! 摸两张牌或回复 2 点体力（chooseDrawRecover），获得〖观星〗。
//! 实现：双臂 phase_prepare/phase_end + owner_hand_empty → once-per-game 以
//! 自身 def 标记门控；效果顺序按批 182 惯例（选择型 effect 置尾）——change_max_hp
//! /add_skill/add_mark 先行，choose_draw_or_recover 收尾。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLZHIJI: u32 = 4_244_253_651;
const REGUANXING: u32 = 4_220_147_649;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/olzhiji.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olzhiji-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(OLZHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn empty_hand(state: &mut GameState, player: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn fire(state: GameState, owner: PlayerId, phase: TurnPhaseTag) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let outcome = run_effects(&mut tx, effects);
    (tx.commit().state, outcome.map(|_| ()))
}

fn resolve(state: GameState, owner: PlayerId, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_awakening_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLZHIJI)
        .collect();
    assert_eq!(arms.len(), 2, "prepare + end arms");
    for arm in &arms {
        assert_eq!(arm.filter, "owner_hand_empty");
        assert!(!arm.optional);
    }
    assert!(arms.iter().any(|arm| arm.event == "phase_prepare"));
    assert!(arms.iter().any(|arm| arm.event == "phase_end"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == 3_456_200_434)
        .expect("ol_jiangwei in roster");
    assert!(character.skills.contains(&OLZHIJI));
}

#[test]
fn awakening_fires_on_empty_hand_draw_branch_and_grants_reguanxing() {
    let (mut state, owner, _other) = setup(20_260_909_03);
    empty_hand(&mut state, owner);
    let max_hp_before = state.players.get(owner).unwrap().max_hp;

    let (state, outcome) = fire(state, owner, TurnPhaseTag::Prepare);
    // 满血时恢复无意义：原语契约自动改为摸牌（不询问），效果链一次跑完。
    assert!(
        outcome.is_ok(),
        "healthy owner auto-draws, got {outcome:?}"
    );
    assert!(!state.has_pending_choice());
    // 上限-1、获得观星、自动摸 2。
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_hp_before - 1);
    assert!(has_skill(&state, owner, REGUANXING));
    assert_eq!(state.hand_count(owner), 2, "auto draw branch: draw two");
}

#[test]
fn recover_branch_and_mark_gate_prevent_re_awakening() {
    let (mut state, owner, _other) = setup(20_260_909_04);
    empty_hand(&mut state, owner);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2; // 可观察回复分支
        tx.commit().state
    };
    let (state, outcome) = fire(state, owner, TurnPhaseTag::End);
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let max_hp_after_awaken = state.players.get(owner).unwrap().max_hp;
    let (state, _) = resolve(state, owner, vec![2], 20_260_909_14);
    assert_eq!(state.players.get(owner).unwrap().hp, 3, "recover branch");
    assert!(!state.has_pending_choice());

    // 再次空手 + 阶段触发：过滤器成立但标记门控拦截，无任何效果。
    let mut state2 = state;
    empty_hand(&mut state2, owner);
    let hp_before = state2.players.get(owner).unwrap().hp;
    let hand_before = state2.hand_count(owner);
    let (state2, outcome) = fire(state2, owner, TurnPhaseTag::Prepare);
    assert!(
        outcome.is_ok(),
        "mark gate must suppress the effect chain, got {outcome:?}"
    );
    assert!(!state2.has_pending_choice());
    assert_eq!(state2.players.get(owner).unwrap().hp, hp_before);
    assert_eq!(state2.hand_count(owner), hand_before);
    assert_eq!(
        state2.players.get(owner).unwrap().max_hp,
        max_hp_after_awaken,
        "max hp must not drop twice"
    );
}
