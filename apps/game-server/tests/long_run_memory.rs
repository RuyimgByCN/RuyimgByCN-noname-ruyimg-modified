//! **多盘连续 + 堆内存观测**（产品级长程的内存面）。
//!
//! 环境限制（**已实测**）：本环境 `/bin/ps` 被沙箱拒绝（`Operation not permitted`）⇒ **拿不到 RSS**；
//! 且工区未依赖 `libc` ⇒ 不用 `getrusage`。因此改用**不引入依赖**的做法：
//! 在本测试二进制里装一个**计数式全局分配器**（`#[global_allocator]`，基于 `std::alloc::System`），
//! 精确统计**当前堆占用**与**峰值**。
//!
//! 口径（**务必注意**）：
//! * 测的是**本进程 Rust 堆**（经全局分配器的申请），**不是** RSS（不含栈、mmap 直映射、代码段）✗；
//! * 结论只对「**是否存在随盘数增长的堆泄漏**」负责，不宣称「内存占用多少 MB」的绝对值。
//!
//! 判据：
//! 1. **每一盘都跑到终局**（`MatchOutcome != InProgress`）；
//! 2. **连续 6 盘后，堆占用相对首盘后不出现阶跃式增长**（阈值宽松，见 `SLACK_BYTES`）；
//! 3. 记录**每盘结束时的堆占用曲线**，供人工判断。

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use engine_api::{CommandKind, ExpectedRevision, IdempotencyKey, LegalAction, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use room::{spawn_room, RoomMsg};
use session::Outbound;
use tokio::sync::mpsc;

// ---------- 计数式全局分配器（仅本测试二进制） ----------

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let cur = CURRENT.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(cur, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                let cur = CURRENT.fetch_add(new_size - layout.size(), Ordering::Relaxed) + new_size - layout.size();
                PEAK.fetch_max(cur, Ordering::Relaxed);
            } else {
                CURRENT.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        p
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn heap_now() -> usize {
    CURRENT.load(Ordering::Relaxed)
}
fn heap_peak() -> usize {
    PEAK.load(Ordering::Relaxed)
}

// ---------- 机器人与驱动（与 long_run_bot.rs 同构的最小子集） ----------

const CONNS: usize = 2;
/// 连续盘数。
const MATCHES: usize = 30;
/// 单盘步数上限（防止不收敛）。
const MAX_STEPS: usize = 4000;
/// 允许的堆增长（宽松阈值，只用于抓**阶跃式**泄漏）。
const SLACK_BYTES: usize = 32 * 1024 * 1024;

async fn join(handle: &room::RoomHandle, conn: usize, seat: u8, name: &str) -> PlayerId {
    let (tx, _rx) = mpsc::channel::<String>(256);
    let (r, w) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: (conn + 1) as u64,
            display_name: name.into(),
            preferred_seat: Some(seat),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx),
            reply: r,
        })
        .await
        .expect("join send");
    let j = w.await.expect("join reply").expect("join ok");
    PlayerId::new(j.player_id)
}

async fn snapshot(
    handle: &room::RoomHandle,
    id: u64,
    conn: usize,
    seq: u64,
    rev: u64,
) -> Option<(u64, Vec<LegalAction>, Option<engine_api::ChoiceRequest>, engine_api::MatchOutcome)> {
    let (rs, ws) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Snapshot {
            connection_id: (conn + 1) as u64,
            header: MessageHeader::new(id, seq, rev),
            reply: rs,
        })
        .await
        .ok()?;
    match ws.await {
        Ok(Ok(ServerMessage::Snapshot {
            view,
            choice,
            legal_actions,
            ..
        })) => Some((view.revision, legal_actions, choice, view.outcome)),
        _ => None,
    }
}

async fn submit(
    handle: &room::RoomHandle,
    id: u64,
    conn: usize,
    seq: u64,
    cmd: engine_api::Command,
) -> (bool, u64, Vec<LegalAction>) {
    let (rs, ws) = tokio::sync::oneshot::channel();
    if handle
        .send(RoomMsg::Submit {
            connection_id: (conn + 1) as u64,
            header: MessageHeader::new(id, seq, cmd.expected_revision.0),
            command: cmd.clone(),
            idempotency_key: cmd.idempotency_key,
            reply: rs,
        })
        .await
        .is_err()
    {
        return (false, 0, Vec::new());
    }
    match ws.await {
        Ok(Ok(ServerMessage::CommandOutcome {
            result,
            view,
            legal_actions,
            ..
        })) => (
            matches!(result, engine_api::CommandResult::Accepted { .. }),
            view.revision,
            legal_actions,
        ),
        _ => (false, 0, Vec::new()),
    }
}

fn pick(legal: &[LegalAction], cursor: &mut usize) -> Option<LegalAction> {
    if legal.is_empty() {
        return None;
    }
    if let Some(a) = legal.iter().find(|a| matches!(a, LegalAction::ResolveChoice { .. })) {
        return Some(a.clone());
    }
    let start = *cursor % legal.len();
    let rotated: Vec<LegalAction> = legal[start..]
        .iter()
        .chain(legal[..start].iter())
        .cloned()
        .collect();
    for a in &rotated {
        if matches!(
            a,
            LegalAction::PlayCard { .. }
                | LegalAction::ActivateSkill { .. }
                | LegalAction::UseSkillCard { .. }
        ) {
            return Some(a.clone());
        }
    }
    rotated.first().cloned()
}

/// 跑**一整盘**，返回 (是否终局, 结局, 步数, 最终修订号)。
async fn play_one(room_id: u64, seed: u64) -> (bool, engine_api::MatchOutcome, usize, u64) {
    let handle = spawn_room(room_id, 2, seed, String::new(), 64, 100);
    let actors = [join(&handle, 0, 0, "m-a").await, join(&handle, 1, 1, "m-b").await];
    let mut legal: [Vec<LegalAction>; CONNS] = [Vec::new(), Vec::new()];
    let mut cursor = [0usize; CONNS];
    let mut rev = 0u64;
    let mut seq = 0u64;
    let mut outcome = engine_api::MatchOutcome::InProgress;
    let mut steps = 0usize;

    // 引导
    for conn in 0..CONNS {
        seq += 1;
        let cmd = engine_api::Command {
            actor: actors[conn],
            expected_revision: ExpectedRevision(rev),
            idempotency_key: IdempotencyKey(seq),
            kind: CommandKind::EndPhase,
        };
        let (_ok, r, l) = submit(&handle, room_id, conn, seq, cmd).await;
        if r > 0 {
            rev = r;
        }
        legal[conn] = l;
    }

    for _ in 0..MAX_STEPS {
        let mut acted = false;
        for conn in 0..CONNS {
            if legal[conn].is_empty() {
                if let Some((r, l, choice, o)) = snapshot(&handle, room_id, conn, seq + 1, rev).await {
                    outcome = o;
                    if r > 0 {
                        rev = r;
                    }
                    legal[conn] = l;
                    if legal[conn].is_empty() {
                        if let Some(c) = choice {
                            if c.observer.0 == actors[conn].0 {
                                let take = (c.min_count as usize).min(c.options.len());
                                legal[conn].push(LegalAction::ResolveChoice {
                                    choice_id: c.choice_id,
                                    selections: (0..take as u32).collect(),
                                    cancelled: take == 0 && c.allow_cancel,
                                });
                            }
                        }
                    }
                }
            }
            let Some(action) = pick(&legal[conn], &mut cursor[conn]) else {
                continue;
            };
            let cmd = action.to_command(actors[conn], rev);
            seq += 1;
            let (ok, r, l) = submit(&handle, room_id, conn, seq, cmd).await;
            if ok {
                cursor[conn] = 0;
            } else {
                cursor[conn] += 1;
            }
            if r > 0 {
                rev = r;
            }
            legal[conn] = l;
            acted = true;
            steps += 1;
            if outcome != engine_api::MatchOutcome::InProgress {
                break;
            }
        }
        if !acted {
            if let Some((_, _, _, o)) = snapshot(&handle, room_id, 0, seq + 1, rev).await {
                if o != engine_api::MatchOutcome::InProgress {
                    outcome = o;
                }
            }
            if outcome != engine_api::MatchOutcome::InProgress {
                break;
            }
            // 无动作且未终局：再试一轮（避免立刻判定卡死）
            continue;
        }
        if outcome != engine_api::MatchOutcome::InProgress {
            break;
        }
    }
    drop(handle);
    (
        outcome != engine_api::MatchOutcome::InProgress,
        outcome,
        steps,
        rev,
    )
}

/// **多盘连续**：30 盘逐盘跑完，记录每盘后的堆占用与结局分布，
/// 并比较**前 5 盘均值**与**后 5 盘均值**（用于发现缓慢上升趋势）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn thirty_consecutive_matches_do_not_leak_heap() {
    let t0 = Instant::now();
    let base = heap_now();
    let mut after_first: Option<usize> = None;
    let mut curve: Vec<(usize, bool, String, usize, usize)> = Vec::new();

    for i in 0..MATCHES {
        let (finished, outcome, steps, rev) = play_one(1_000 + i as u64, 12_345 + i as u64).await;
        let now = heap_now();
        if i == 0 {
            after_first = Some(now);
        }
        curve.push((i, finished, format!("{outcome:?}"), steps, now));
        println!(
            "RESULT mem_match idx={i} finished={finished} outcome={outcome:?} steps={steps} rev={rev} heap_bytes={now}"
        );
        assert!(finished, "第 {i} 盘应在 {MAX_STEPS} 步内跑到终局");
    }

    let end = heap_now();
    let first = after_first.unwrap_or(base);
    let growth = end.saturating_sub(first);
    println!(
        "RESULT mem_summary matches={MATCHES} base_bytes={base} after_first_bytes={first} end_bytes={end} growth_bytes={growth} peak_bytes={} slack_bytes={SLACK_BYTES} elapsed_ms={}",
        heap_peak(),
        t0.elapsed().as_millis()
    );
    for (i, finished, outcome, steps, heap) in &curve {
        println!("RESULT mem_curve idx={i} finished={finished} outcome={outcome} steps={steps} heap_bytes={heap}");
    }
    // 结局分布
    let mut tally: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (_, _, outcome, _, _) in &curve {
        *tally.entry(outcome.clone()).or_insert(0) += 1;
    }
    let steps_total: usize = curve.iter().map(|c| c.3).sum();
    // 前后各 5 盘的堆均值（趋势判据）
    let head: usize = curve.iter().take(5).map(|c| c.4).sum::<usize>() / 5;
    let tail: usize = curve.iter().rev().take(5).map(|c| c.4).sum::<usize>() / 5;
    println!(
        "RESULT mem_trend head5_avg={head} tail5_avg={tail} delta={} steps_total={steps_total} outcomes={tally:?}",
        tail as i64 - head as i64
    );
    assert!(
        (tail as i64 - head as i64).abs() < SLACK_BYTES as i64,
        "后 5 盘堆均值与首 5 盘相差过大（head={head}, tail={tail}）"
    );
    assert!(
        growth < SLACK_BYTES,
        "连续 {MATCHES} 盘后堆增长 {growth} 字节，超出宽松阈值 {SLACK_BYTES}（疑似泄漏）"
    );
}
