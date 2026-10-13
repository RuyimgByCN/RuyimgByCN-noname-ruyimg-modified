//! 阶段 2 宿主渲染器原型（ADR 0764）：PresentationEvent 消费端引擎侧样例。
//!
//! 展示宿主渲染器的最小形态：驱动一局全座位自动化的对局
//! （策略只读 `AiSession` 可见信息），逐阶段消费 `PresentationEvent`
//! 流（座位可见性已由投影层保证），按回合产出确定性渲染帧。
//!
//! 运行：cargo run --release -p engine-native --example n10_render_host -- [games] [seed]

use engine_native::{
    presentation::PresentationEvent, CardKind, ChoiceOption, ChoiceView, CommandOutcome,
    Engine, GameAction, GameCommand,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 最小牌桌策略：有询问按 min 个选项应答；自己回合出牌阶段且手中有
/// 杀 → 攻下一位存活玩家；否则推进阶段。
fn seat_command(engine: &Engine, seat: u8) -> GameCommand {
    let session = engine.ai_session(seat);
    if let Some(ChoiceView::Own { id, options, min, .. }) = session.choice() {
        let sel: Vec<ChoiceOption> = options.into_iter().take(min.max(1)).collect();
        return GameCommand {
            actor: seat,
            expected_revision: engine.revision(),
            dedup_id: fresh(),
            action: GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        };
    }
    // 进攻尝试：本回合出牌阶段、手牌中有杀 → 打下一位存活座位
    if engine.turn().current == seat && engine.turn().phase == Phase::Play {
        let view = session.view();
        if let engine_native::view::HandView::Own(hand) = &view.players[seat as usize].hand {
            let sha = hand
                .iter()
                .copied()
                .find(|c| engine.catalog().cards[*c as usize].kind == CardKind::Sha);
            if let Some(sha) = sha {
                let n = view.players.len();
                let target = (1..n as u8)
                    .map(|d| (seat + d) % n as u8)
                    .find(|s| view.players[*s as usize].alive);
                if let Some(target) = target {
                    return GameCommand {
                        actor: seat,
                        expected_revision: engine.revision(),
                        dedup_id: fresh(),
                        action: GameAction::PlayCard {
                            card: sha,
                            targets: vec![target],
                        },
                    };
                }
            }
        }
    }
    GameCommand {
        actor: seat,
        expected_revision: engine.revision(),
        dedup_id: fresh(),
        action: GameAction::NextPhase,
    }
}

fn run_game(seed: u64, max_commands: usize) -> (Engine, usize, usize) {
    let mut e = Engine::new(seed, 3);
    let mut frames = 0usize;
    let mut events_consumed = 0usize;
    let mut consumed_log = 0usize;
    for _ in 0..max_commands {
        if e.outcome().is_some() {
            break;
        }
        // 行动座位：开放选择的 observer 优先，否则当前回合玩家
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        let out = e.submit(seat_command(&e, seat));
        match out {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            CommandOutcome::Rejected(r) => {
                // 策略端容错：同一座位推进阶段兜底
                let _ = r;
                let cmd = GameCommand {
                    actor: seat,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::NextPhase,
                };
                let _ = e.submit(cmd);
            }
            CommandOutcome::Suspended => {}
        }
        // 每次命令后：宿主渲染器**增量**消费表现事件流（真实宿主形态：
        // 只取上次消费位置之后的新增日志段做投影）
        let log = e.log();
        let delta = &log[consumed_log..];
        consumed_log = log.len();
        let events = engine_native::presentation::project(delta, e.catalog(), None);
        events_consumed += events.len();
        frames += events
            .iter()
            .filter(|ev| matches!(ev, PresentationEvent::Phase { phase: Phase::Play, begin: true, .. }))
            .count();
    }
    (e, frames, events_consumed)
}

use engine_native::Phase;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(99);

    println!("=== 宿主渲染器原型：PresentationEvent 消费 ===");
    for g in 0..games {
        let (e, frames, consumed) = run_game(seed + g as u64 * 100, 4096);
        let outcome = e
            .outcome()
            .map(|o| {
                match o.winner {
                    engine_native::state::Winner::Player(p) => format!("座位{p} 胜"),
                    ref w => format!("{w:?} 胜"),
                }
            })
            .unwrap_or_else(|| "未终局（达到命令上限）".to_string());
        // 最终帧：观战者视角确定性渲染
        let frame = e.render_frame(None, 64);
        println!("--- 局 {}（seed={}）：{outcome}；表现事件 {consumed} 条，渲染帧 {frames} 次 ---", g + 1, seed + g as u64 * 100);
        print!("{frame}");
        // 消费端契约自检：流中不得出现他人摸牌牌名（类型层面已保证，
        // 运行时再核一遍渲染帧不包含「摸牌 <数字>张(暗)」以外的暗牌名）
        let spec_events = e.presentation_for(None);
        let leak = spec_events.iter().any(|ev| match ev {
            PresentationEvent::Drawn { cards, .. } => cards.is_some(),
            _ => false,
        });
        assert!(!leak, "观战者事件流不得携带摸牌牌名");
    }
    println!("=== 全部局完成：PresentationEvent 消费端契约成立 ===");
}
