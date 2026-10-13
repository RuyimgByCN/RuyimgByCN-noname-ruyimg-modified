//! O2 全量合并第二批（2026-10-14）：家族 jsrgzuozhan 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_jsrgzuozhan_machine {
//! 第二百四十二批：jsrgzuozhan 坐瞻专属状态机行为测试（ABI 698 / tag 721 / Cont 304）。
//! 全链：GameStarted → optional 确认 → SelectTargets 1..2（其他存活角色）→
//! 射程标记 min(所选最高体力, 5) 加于自身（标记 id = 技能 def，
//! attack_range add owner_skill_mark 修饰面消费）+「坐瞻」标记记于所选角色。
//! 债：enterGame/phaseBefore 双时点近似、组技「坐瞻死亡补牌」面。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::player_in_attack_range;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZUOZHAN: u32 = 4_241_034_616;
const ZUOZHAN_SIT_MARK: u32 = 0x5A55_0002;
const ZUOZHAN_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgzuozhan.skill");

fn setup(seed: u64, player_count: u8) -> (GameState, Vec<PlayerId>) {
    let compiled = content_compiler::compile_source_named("jsrgzuozhan-b242", ZUOZHAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, player_count, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZUOZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(existing, _)| *existing == mark)
        .map_or(0, |(_, count)| *count)
}

fn fire_game_start(state: GameState) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 4 }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
    (tx.commit().state, opened)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, CommandResult) {
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
    (next, result.result)
}

#[test]
fn jsrgzuozhan_two_targets_marks_and_range() {
    // 选两名其他角色：射程标记 = min(所选最高体力, 5)，所选各记「坐瞻」标记。
    let (state, players) = setup(20_260_912_01, 4);
    let owner = players[0];
    let (state, opened) = fire_game_start(state);
    assert!(opened, "confirm opens");

    let hp = |state: &GameState, p: PlayerId| state.players.get(p).unwrap().hp;
    let expect = hp(&state, players[1]).max(hp(&state, players[2])).min(5);

    let (state, result) = resolve(state, owner, vec![1], false, 11);
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "confirm rejected: {result:?}"
    );
    assert!(state.has_pending_choice(), "target selection opens");
    let (state, result) = resolve(
        state,
        owner,
        vec![players[1].raw(), players[2].raw()],
        false,
        12,
    );
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "target selection rejected: {result:?}"
    );
    assert!(!state.has_pending_choice());
    assert_eq!(
        mark_count(&state, owner, ZUOZHAN),
        expect,
        "range mark = min(highest selected hp, 5)"
    );
    assert_eq!(mark_count(&state, players[1], ZUOZHAN_SIT_MARK), 1);
    assert_eq!(mark_count(&state, players[2], ZUOZHAN_SIT_MARK), 1);
    assert_eq!(mark_count(&state, players[3], ZUOZHAN_SIT_MARK), 0);
}

#[test]
fn jsrgzuozhan_range_extends_attack_distance() {
    // 四人局：owner 与对家（座 2）距离 2，基础射程 1 不可及；射程标记 ≥1 后可达。
    let (state, players) = setup(20_260_912_02, 4);
    let owner = players[0];
    let far = players[2];
    assert!(
        !player_in_attack_range(&state, owner, far),
        "precondition: far seat beyond base range"
    );

    let (state, opened) = fire_game_start(state);
    assert!(opened);
    let (state, result) = resolve(state, owner, vec![1], false, 21);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice(), "target selection opens");
    let (state, result) = resolve(state, owner, vec![far.raw()], false, 22);
    assert!(matches!(result, CommandResult::Accepted { .. }));

    let range = mark_count(&state, owner, ZUOZHAN);
    assert!(range >= 1, "range mark granted");
    assert!(
        player_in_attack_range(&state, owner, far),
        "range modifier extends reach to far seat"
    );
}

#[test]
fn jsrgzuozhan_cancel_places_no_marks() {
    // 取消 = 不发动：无射程标记、无「坐瞻」标记。
    let (state, players) = setup(20_260_912_03, 4);
    let owner = players[0];
    let (state, opened) = fire_game_start(state);
    assert!(opened);
    let (state, result) = resolve(state, owner, vec![], true, 31);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, owner, ZUOZHAN), 0);
    for player in &players[1..] {
        assert_eq!(mark_count(&state, *player, ZUOZHAN_SIT_MARK), 0);
    }
}

#[test]
fn jsrgzuozhan_self_selection_rejected() {
    // resolver 拒绝把 owner 自己选为「坐瞻」角色。
    let (state, players) = setup(20_260_912_04, 4);
    let owner = players[0];
    let (state, opened) = fire_game_start(state);
    assert!(opened);
    let (state, _) = resolve(state, owner, vec![1], false, 41);
    assert!(state.has_pending_choice(), "target selection opens");
    let (_, result) = resolve(state, owner, vec![owner.raw()], false, 42);
    assert!(
        !matches!(result, CommandResult::Accepted { .. }),
        "self as sit-target must be rejected"
    );
}

// ---- 批 243：组技补牌面（ABI 699 / tag 722 / Cont 305）----

use engine_core::state::{CardKind, ZoneKind};

fn discard_tail(state: &GameState) -> Vec<CardId> {
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get(discard).unwrap().cards.clone()
}

fn discard_len(state: &GameState) -> usize {
    discard_tail(state).len()
}

/// 从牌堆顶取一张指定牌种的牌移入弃牌堆（压尾）。
fn move_kind_to_discard(state: &mut GameState, kind: CardKind) {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let draw_cards = state.zones.get(draw).unwrap().cards.clone();
    let pos = draw_cards
        .iter()
        .rev()
        .position(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.kind == kind)
        })
        .unwrap_or_else(|| panic!("no {kind:?} in draw pile"));
    let pos = draw_cards.len() - 1 - pos;
    let card = state.zones.get_mut(draw).unwrap().cards.remove(pos);
    state.zones.get_mut(discard).unwrap().cards.push(card);
    if let Some(card_state) = state.cards.get_mut(card) {
        card_state.zone = discard;
    }
}

fn run_main_chain(state: GameState, owner: PlayerId, targets: &[PlayerId], key: u64) -> GameState {
    let (state, opened) = fire_game_start(state);
    assert!(opened);
    let (state, _) = resolve(state, owner, vec![1], false, key);
    let (state, result) = resolve(
        state,
        owner,
        targets.iter().map(|p| p.raw()).collect(),
        false,
        key + 1,
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn jsrgzuozhan_gain_full_chain_after_marked_ally_dies() {
    // 主面标记 p1/p2 → p1 死亡 → 确认 → 选 p2 → p2 从弃牌堆获得至多 X 张
    // 牌名互不相同的基本牌（弃牌堆压入 sha/sha/tao/shan/jiu/guohe，同名杀
    // 去重、锦囊跳过）。
    let (state, players) = setup(20_260_912_11, 4);
    let owner = players[0];
    let (p1, p2) = (players[1], players[2]);
    let state = run_main_chain(state, owner, &[p1, p2], 51);
    let quota = state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(m, _)| *m == ZUOZHAN)
        .map_or(0, |(_, c)| *c) as usize;
    assert!(quota >= 1);

    // p1 死亡（其手牌入弃牌堆，随后压入的已知牌位于堆顶）。
    let mut tx = Transaction::begin(state);
    let outcome = match engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: p1,
            killer: PlayerId::NONE,
        }],
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(_) => Ok(()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "marked ally death opens confirm"
    );
    let mut state = tx.commit().state;
    // 压入已知牌（堆顶 → 堆底次序：guohe, jiu, shan, tao, sha, sha）。
    move_kind_to_discard(&mut state, CardKind::TrickGuohe);
    move_kind_to_discard(&mut state, CardKind::Jiu);
    move_kind_to_discard(&mut state, CardKind::Shan);
    move_kind_to_discard(&mut state, CardKind::Tao);
    move_kind_to_discard(&mut state, CardKind::Sha);
    move_kind_to_discard(&mut state, CardKind::Sha);
    let discard_before = discard_len(&state);
    let p2_hand = state.hand_count(p2) as usize;

    // 确认 → 选 p2 → 补牌。
    let (state, _) = resolve(state, owner, vec![1], false, 52);
    assert!(state.has_pending_choice(), "target selection opens");
    let (state, result) = resolve(state, owner, vec![p2.raw()], false, 53);
    assert!(matches!(result, CommandResult::Accepted { .. }), "target resolve: {result:?}");

    let expected = quota.min(4) as usize; // 堆顶可取的不同名基本牌：jiu/shan/tao/sha
    assert_eq!(state.hand_count(p2) as usize, p2_hand + expected);
    assert_eq!(discard_len(&state), discard_before - expected);
    // 死亡者标记保留；补牌目标为存活坐瞻角色。
    assert!(!state.players.get(p1).unwrap().alive);
    assert_eq!(mark_count(&state, p1, ZUOZHAN_SIT_MARK), 1);
    assert_eq!(mark_count(&state, p2, ZUOZHAN_SIT_MARK), 1);
}

#[test]
fn jsrgzuozhan_gain_unmarked_death_noop_after_confirm() {
    // 未标记角色死亡：optional 确认先行打开（quanjia 同款登记债），确认后
    // 机内权威校验拦截——不再打开选目标，技能空转。
    let (state, players) = setup(20_260_912_12, 4);
    let owner = players[0];
    let state = run_main_chain(state, owner, &[players[1]], 61);
    let mut tx = Transaction::begin(state);
    let outcome = match engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: players[3],
            killer: PlayerId::NONE,
        }],
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(_) => Ok(()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "optional confirm opens before machine validation"
    );
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![1], false, 62);
    assert!(
        !state.has_pending_choice(),
        "machine validation must close the chain for unmarked deaths"
    );
}
}
