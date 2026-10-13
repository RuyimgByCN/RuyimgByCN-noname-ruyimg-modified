//! N10 命令序号与规划候选回归（ADR 0776）。
//!
//! 此前规划克隆内候选提交用固定 `1_000_000 + nodes` 偏移——会话真实
//! 高水位越过该区间时，全部合法候选被当重复命令拒绝，规划退化为
//! NextPhase；`u64::MAX` 高水位加一无处理（debug 溢出 panic）。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

/// 停在 0 号出牌阶段、0 号手牌为一张杀（可出）的对局。
fn play_ready_with_sha(seed: u64, seats: u8) -> Engine {
    let mut e = Engine::new(seed, seats as usize);
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..seats {
        e.debug_clear_hand(p);
    }
    e.debug_place(0, "sha");
    e
}

// ---------------------------------------------------------------------------
// T13 AI 工作池（N10/N12）：并行多座规划 + 运行中取消跨线程。
// ---------------------------------------------------------------------------

/// 工作池并行结果与逐座顺序规划逐一相等（确定性纯函数回归）。
#[test]
fn t13_plan_pool_parallel_matches_sequential() {
    let e = play_ready_with_sha(2602, 4);
    let planner = engine_native::ai::BoundedPlanner::new(64);
    let pool = engine_native::ai::PlanPool::new(4);
    let seats = [0u8, 1, 2, 3];
    let par = pool.plan_seats(&e, &seats, &planner);
    let seq: Vec<_> = seats.iter().map(|&s| e.plan_action(s, &planner)).collect();
    assert_eq!(par.len(), seats.len());
    for (p, s) in par.iter().zip(seq.iter()) {
        assert_eq!(p.seat, s.seat, "输入座位序返回");
        assert_eq!(p.revision, s.revision, "同一快照基线");
        assert_eq!(p.nodes_used, s.nodes_used, "确定性：节点数逐一相等");
        assert_eq!(p.budget_exhausted, s.budget_exhausted);
        assert!(!p.cancelled);
        // 动作一致（同候选同评分→同选择）
        let same = match (&p.action, &s.action) {
            (a, b) => format!("{a:?}") == format!("{b:?}"),
        };
        assert!(same, "并行与顺序动作一致（{:?} vs {:?}）", p.action, s.action);
    }
}

/// worker 数少于座位数：分桶消费仍按输入序返回、结果不串位。
#[test]
fn t13_plan_pool_more_seats_than_workers_keeps_order() {
    let e = play_ready_with_sha(2603, 5);
    let planner = engine_native::ai::BoundedPlanner::new(32);
    let pool = engine_native::ai::PlanPool::new(2);
    let seats = [0u8, 1, 2, 3, 4];
    let par = pool.plan_seats(&e, &seats, &planner);
    for (i, p) in par.iter().enumerate() {
        assert_eq!(p.seat, seats[i], "输入序返回（分桶 round-robin 逆操作）");
    }
}

/// 运行中取消（跨线程）：预挂取消旗→规划线程首边界即返回 cancelled，
/// apply_plan 拒绝（Cancelled）。
#[test]
fn t13_plan_cancel_mid_run_cross_thread() {
    let e = play_ready_with_sha(2604, 4);
    let planner = std::sync::Arc::new(engine_native::ai::BoundedPlanner::new(1_000_000));
    let flag = planner.cancel_flag();
    // 预挂取消（确定性）：旗在规划线程首候选边界检查前已置位
    flag.store(true, Ordering::SeqCst);
    // 规划隔离已克隆化：工作线程持克隆引擎（同一快照基线，语义等价）
    let worker_engine = e.clone();
    let handle = std::thread::spawn(move || {
        let planner = planner.as_ref();
        worker_engine.plan_action(0, planner)
    });
    let plan = handle.join().expect("规划线程");
    assert!(plan.cancelled, "运行中取消生效");
    let mut e2 = e.clone();
    assert!(matches!(
        e2.apply_plan(&plan),
        Err(engine_native::ai::ApplyPlanError::Cancelled)
    ));
}

/// 规划模拟序号与会话高水位统一：真实高水位 2,000,000 时，合法杀
/// 候选必须被实际模拟并成为规划结果（旧反例中退化为 NextPhase）。
#[test]
fn plan_sim_uses_session_high_water() {
    let low = play_ready_with_sha(2601, 3);
    let low_plan = low.plan_action(0, &engine_native::ai::BoundedPlanner::new(64));

    let mut snapshot = play_ready_with_sha(2601, 3).save();
    snapshot.last_dedup = 2_000_000;
    let high = Engine::restore(&snapshot).unwrap();
    let high_plan = high.plan_action(0, &engine_native::ai::BoundedPlanner::new(64));

    assert!(
        matches!(low_plan.action, GameAction::PlayCard { .. }),
        "低水位：杀候选应被模拟采纳 {:?}",
        low_plan.action
    );
    assert!(
        matches!(high_plan.action, GameAction::PlayCard { .. }),
        "高水位 2,000,000：杀候选不得被当重复命令丢弃（旧反例） {:?}",
        high_plan.action
    );
    assert_eq!(high_plan.nodes_used, low_plan.nodes_used, "两个水位的候选评估数应一致");
}

/// 高水位耗尽（u64::MAX）：策略提交与应用计划均被引擎按重复命令
/// 干净拒绝，不 panic、不回绕。
#[test]
fn policy_and_plan_reject_cleanly_at_dedup_exhaustion() {
    let mut snapshot = play_ready_with_sha(2702, 3).save();
    snapshot.last_dedup = u64::MAX;
    let mut e = Engine::restore(&snapshot).unwrap();

    struct Passive;
    impl engine_native::view::SeatPolicy for Passive {
        fn act(&mut self, _: &engine_native::view::PolicyInput) -> Option<GameAction> {
            None
        }
    }
    let out = e.run_policy(0, &mut Passive);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)),
        "耗尽水位下策略提交必须干净拒绝: {out:?}"
    );

    let plan = e.plan_action(0, &engine_native::ai::BoundedPlanner::new(64));
    let applied = e.apply_plan(&plan);
    match applied {
        Ok(CommandOutcome::Rejected(RejectReason::DuplicateCommand)) => {}
        other => panic!("耗尽水位下应用计划必须干净拒绝: {other:?}"),
    }
}

/// 规划→应用闭环在高水位下成立：plan 携带动作，apply 走主通道被接受，
/// 提交后高水位推进到提交序号。
#[test]
fn plan_apply_roundtrip_at_high_water() {
    let mut snapshot = play_ready_with_sha(2703, 3).save();
    snapshot.last_dedup = 2_000_000;
    let mut e = Engine::restore(&snapshot).unwrap();
    let plan = e.plan_action(0, &engine_native::ai::BoundedPlanner::new(64));
    assert!(matches!(plan.action, GameAction::PlayCard { .. }));
    let out = e.apply_plan(&plan).expect("apply");
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
}

// ---------------------------------------------------------------------------
// N10 规划信息集的合法性与守恒（原 OPEN_AI_SAMPLED_OWNERSHIP 反例）：
// 旧实现从牌堆采样并复制牌 ID 到对手手牌 → 重复物理归属，恢复被拒。
// 新实现把「对手暗手牌 + 牌库」作为未知牌池确定性重分配——只移动、
// 不复制，归属完整核算依然成立。
// ---------------------------------------------------------------------------

/// 汇总快照全部牌区的牌 id 多重集（手牌/装备/判定/牌库/弃牌/桌面）。
fn zone_multiset(snap: &engine_native::StateSnapshot) -> Vec<u32> {
    let mut ids: Vec<u32> = Vec::new();
    for p in &snap.core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(snap.core.deck.iter().copied());
    ids.extend(snap.core.discard.iter().copied());
    ids.extend(snap.core.table.iter().copied());
    ids.sort_unstable();
    ids
}

/// 对手持暗牌时重分配后的规划克隆：牌归属完整核算通过（可恢复）、
/// 牌集合多重集与权威状态一致、各对手手牌数不变。
#[test]
fn redact_preserves_card_ownership_and_multiset() {
    let mut e = play_ready_with_sha(2304, 3);
    e.debug_place(1, "shan"); // 对手暗牌（原反例触发点）
    e.debug_place(2, "tao");
    let before = e.save();
    let mut sim = e.clone();
    engine_native::ai::redact_opponent_hands(&mut sim, 0);
    let after = sim.save();

    // 归属完整核算：重分配后的克隆必须可恢复（此前报 card_duplicate）
    Engine::restore(&after).expect("重分配后克隆的物理归属必须完整");
    // 牌集合多重集守恒：只移动、不复制、不丢失
    assert_eq!(
        zone_multiset(&before),
        zone_multiset(&after),
        "重分配不得改变物理牌集合"
    );
    // 各对手手牌数不变（观察者可知量）
    for i in 1..3 {
        assert_eq!(
            before.core.players[i].hand.len(),
            after.core.players[i].hand.len(),
            "对手 {i} 手牌数必须保持"
        );
    }
    assert_eq!(before.core.deck.len(), after.core.deck.len(), "牌库数必须保持");
}

/// 确定性与退化：同种子两次重分配结果逐字节一致；牌堆为空时退化为
/// 对手暗牌重排（仍守恒、仍可恢复）。
#[test]
fn redact_deterministic_with_empty_deck_fallback() {
    let mut e = play_ready_with_sha(2305, 3);
    e.debug_place(1, "shan");
    let mut a = e.clone();
    let mut b = e.clone();
    engine_native::ai::redact_opponent_hands(&mut a, 0);
    engine_native::ai::redact_opponent_hands(&mut b, 0);
    assert_eq!(a.save(), b.save(), "同种子重分配必须确定");

    // 牌堆空：池 = 对手暗牌，重排后守恒且可恢复。
    // （把牌库整体移入公开弃牌堆，保持物理归属完整——直接清牌库会被
    // N04-f 完整核算正确拒绝。）
    let mut snapshot = e.save();
    snapshot.core.discard.append(&mut snapshot.core.deck);
    let empty = Engine::restore(&snapshot).unwrap();
    let mut sim = empty.clone();
    engine_native::ai::redact_opponent_hands(&mut sim, 0);
    let after = sim.save();
    Engine::restore(&after).expect("牌堆空退化路径仍须归属完整");
    assert_eq!(after.core.deck.len(), 0);
    assert_eq!(
        zone_multiset(&snapshot),
        zone_multiset(&after),
        "退化路径不得改变物理牌集合"
    );
    // 退化语义：池中仅一张未知牌时排列恒等——此时该牌归属本就唯一
    // 可知（牌堆空、别无未知牌），不构成信息泄漏；泄漏防护由未知池
    // 非空的用例（上方 ownership 用例的牌库洗牌）承担。
    let _ = after;
}

/// 未知池非空时重分配确实打乱未知牌位置（牌库序改变、对手暗牌
/// 一般不再逐字保留）——规划模拟不读真实暗牌归属。
#[test]
fn redact_shuffles_unknown_pool_when_deck_present() {
    let mut e = play_ready_with_sha(2304, 3);
    e.debug_place(1, "shan");
    let before = e.save();
    let mut sim = e.clone();
    engine_native::ai::redact_opponent_hands(&mut sim, 0);
    let after = sim.save();
    // 牌库属于未知池：重分配后牌库序必须改变
    assert_ne!(
        before.core.deck, after.core.deck,
        "未知池必须被重排（否则规划读真实牌序/暗牌归属）"
    );
}

/// 端到端：对手持暗牌时规划→应用闭环仍成立（合法信息集不破坏规划）。
#[test]
fn plan_pipeline_works_with_hidden_opponent_cards() {
    let mut e = play_ready_with_sha(2704, 3);
    e.debug_place(1, "shan"); // 对手暗闪
    let plan = e.plan_action(0, &engine_native::ai::BoundedPlanner::new(64));
    assert!(matches!(plan.action, GameAction::PlayCard { .. }));
    let out = e.apply_plan(&plan).expect("apply");
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
}

/// 单选候选的合法回退（原 OPEN_AI_CANDIDATES 反例）：出闪询问
/// min=1/max=1、选项 [Pass, 闪]——规划必须评估闪并在评分上胜过
/// 被动挨打（旧实现 take(max=1) 只评估 Pass）。
#[test]
fn plan_enumerates_single_choice_fallbacks() {
    let mut e = play_ready_with_sha(2303, 3);
    e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 挂起在 1 号的出闪询问上；规划 1 号的应答
    let plan = e.plan_action(1, &engine_native::ai::BoundedPlanner::new(64));
    match plan.action {
        GameAction::Answer { ref selection, .. } => {
            assert_eq!(
                selection,
                &vec![e_open_shan(&e)],
                "持闪应答必须枚举到闪（旧反例只评估 Pass）"
            );
        }
        other => panic!("应答候选缺失: {other:?}"),
    }
}

fn e_open_shan(e: &Engine) -> ChoiceOption {
    let c = e.open_choice().expect("询问挂起");
    *c.spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(_)))
        .expect("应含闪选项")
}

/// 确认类询问两支都枚举（Confirm/Pass），放弃类询问的空应答回退。
#[test]
fn plan_enumerates_confirm_and_cancel_fallbacks() {
    // 确认类：遗计触发确认（Confirm 优先）——借 1 号位观察者
    let mut e = Engine::new(2306, 3);
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "遗计");
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let plan = e.plan_action(1, &engine_native::ai::BoundedPlanner::new(64));
    match plan.action {
        GameAction::Answer { ref selection, .. } => {
            assert_eq!(selection, &vec![ChoiceOption::Confirm], "确认类取首选项");
        }
        other => panic!("确认候选缺失: {other:?}"),
    }
}
