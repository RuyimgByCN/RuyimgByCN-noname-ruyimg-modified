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
