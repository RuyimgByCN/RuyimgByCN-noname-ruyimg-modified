//! R05-A：真实回合开始与准备阶段（裁决 28／29；合同 v3 §3「BeginStart→真实
//! phaseBegin→Prepare→Judge…；首回合也开真实 Begin/Prepare」）。
//!
//! 修复前（规则 332）：回合更替直接发判定阶段开始，没有准备阶段，也没有
//! 回合开始触发窗——上游 phaseZhunbei／phaseBegin 技能无原生落点。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase};

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let outcome = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action,
    });
    assert!(!matches!(outcome, CommandOutcome::Rejected(_)), "{outcome:?}");
}

/// 先按默认应答清空所有开放询问（弃牌到上限等），再推进一次阶段。
/// 返回本次是否真的提交了 NextPhase。
fn next_phase(e: &mut Engine) -> bool {
    if let Some(open) = e.open_choice().cloned() {
        let selection = if open.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if open.spec.allow_cancel {
            vec![]
        } else {
            open.spec.options.iter().take(open.spec.min).copied().collect()
        };
        submit(
            e,
            open.observer,
            GameAction::Answer {
                choice_id: open.id,
                selection,
            },
        );
        return false;
    }
    let actor = e.turn().current;
    submit(e, actor, GameAction::NextPhase);
    true
}

/// 推进到下一位角色的回合，返回期间新增日志。
fn run_until_next_turn(e: &mut Engine) -> Vec<LogEvent> {
    let start = e.log().len();
    let first = e.turn().current;
    for _ in 0..64 {
        if e.turn().current != first {
            break;
        }
        next_phase(e);
    }
    assert_ne!(e.turn().current, first, "一个完整回合后应轮到下一位");
    e.log()[start..].to_vec()
}

fn phase_changes(log: &[LogEvent], player: u8) -> Vec<(Phase, bool)> {
    log.iter()
        .filter_map(|x| match x {
            LogEvent::PhaseChanged { player: p, phase, begin } if *p == player => {
                Some((*phase, *begin))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn next_turn_runs_prepare_begin_and_end_before_judge() {
    let mut e = Engine::new(33301, 3);
    let log = run_until_next_turn(&mut e);
    let next = e.turn().current;
    let seq = phase_changes(&log, next);
    assert_eq!(
        &seq[..3],
        &[
            (Phase::Prepare, true),
            (Phase::Prepare, false),
            (Phase::Judge, true)
        ],
        "新回合应先开准备阶段再进判定：{seq:?}"
    );
    assert_eq!(e.turn().phase, Phase::Judge, "准备阶段自动推进，不额外消耗 NextPhase");
}

#[test]
fn whole_turn_still_takes_five_next_phase_commands() {
    let mut e = Engine::new(33302, 3);
    let _ = run_until_next_turn(&mut e);
    let player = e.turn().current;
    let mut next_phases = 0;
    for _ in 0..64 {
        if e.turn().current != player {
            break;
        }
        if next_phase(&mut e) {
            next_phases += 1;
        }
    }
    assert_ne!(e.turn().current, player);
    assert_eq!(next_phases, 5, "准备阶段不改变每回合 5 次 NextPhase");
}

/// K21：整回合结束窗（phaseEnd 段→phaseAfter 段）在交接前派发——两段
/// 日志都出现，且都先于下一位玩家的准备阶段开始；期间事件玩家＝刚结束
/// 回合的角色。修复前（无此窗）本测试红。
#[test]
fn turn_end_windows_fire_before_handoff() {
    let mut e = Engine::new(33401, 3);
    let log = run_until_next_turn(&mut e);
    let next = e.turn().current;
    let prev = (next + 2) % 3; // 3 人局上一手
    let idx = |pred: &dyn Fn(&LogEvent) -> bool| -> Option<usize> {
        log.iter().position(pred)
    };
    let end_win = idx(&|x| matches!(x, LogEvent::TurnPhaseEnded { player } if *player == prev))
        .expect("phaseEnd 段日志（交接前）");
    let after_win = idx(&|x| matches!(x, LogEvent::TurnPhaseAfter { player } if *player == prev))
        .expect("phaseAfter 段日志（交接前）");
    let next_prepare = idx(&|x| matches!(
        x,
        LogEvent::PhaseChanged { player, phase: Phase::Prepare, begin: true } if *player == next
    ))
    .expect("下一位准备阶段开始");
    assert!(end_win < after_win, "phaseEnd 段先于 phaseAfter 段");
    assert!(
        after_win < next_prepare,
        "两段结束窗都先于下一位回合开始（交接在 phaseAfter 段 Commit）"
    );
}

/// K25：回合重置（红莲回合开始效果为可观测锚）先于下一位准备阶段开始。
/// 修复前红莲在判定开始执行（晚于 TurnBegin 与准备阶段）。
#[test]
fn turn_reset_precedes_turn_begin_window() {
    let mut e = Engine::new(33402, 3);
    let log = run_until_next_turn(&mut e);
    let next = e.turn().current;
    // 无红莲持有者时 K25 的直接可观测面：cards_used_turn 在交接内清零。
    assert_eq!(e.state_core().turn.cards_used_turn, 0);
    // 交接窗（phaseAfter 段）先于下一位准备阶段——与 K21 测试同一序。
    let after_win = log
        .iter()
        .position(|x| matches!(x, LogEvent::TurnPhaseAfter { .. }))
        .expect("phaseAfter 段");
    let next_prepare = log
        .iter()
        .position(|x| matches!(
            x,
            LogEvent::PhaseChanged { phase: Phase::Prepare, begin: true, player } if *player == next
        ))
        .expect("准备阶段");
    assert!(after_win < next_prepare);
}

/// K10：翻面角色的整回合跳过发生在回合开始窗**之前**——翻面者不出现
/// 任何阶段帧（含准备阶段），翻回后轮到下一位。修复前（判定开始检测）
/// 翻面者会先跑完 TurnBegin 窗＋准备阶段再被跳过。
#[test]
fn face_down_turn_is_skipped_before_turn_begin() {
    let mut e = Engine::new(33403, 3);
    // 1 号翻面（0 号回合进行中）
    e.state_core_mut().player_mut(1).face_down = true;
    let start = e.log().len();
    let log = run_until_next_turn(&mut e);
    // 0 号回合结束后应直接落到 2 号（1 号整回合被消耗）
    assert_eq!(e.turn().current, 2, "翻面者回合槽被消耗，轮到 2 号");
    assert!(
        !e.state_core().player(1).face_down,
        "翻面者已翻回"
    );
    let p1_phases: Vec<_> = log[start..]
        .iter()
        .filter(|x| matches!(x, LogEvent::PhaseChanged { player: 1, .. }))
        .collect();
    assert!(
        p1_phases.is_empty(),
        "翻面者不得出现任何阶段帧（含准备阶段）：{p1_phases:?}"
    );
}

#[test]
fn prepare_phase_archive_round_trip_keeps_new_phase_tag() {
    let mut e = Engine::new(33303, 3);
    let _ = run_until_next_turn(&mut e);
    let mut snapshot = e.save();
    snapshot.core.turn.phase = Phase::Prepare;
    let decoded = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
        &snapshot,
    ))
    .expect("Prepare 编码可解");
    assert_eq!(decoded.core.turn.phase, Phase::Prepare);
    assert_eq!(Phase::ALL[..5], [Phase::Judge, Phase::Draw, Phase::Play, Phase::Discard, Phase::End]);
}
