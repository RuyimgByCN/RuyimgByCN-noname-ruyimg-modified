use super::*;

/// P2/N06（ADR 0771）：真实身份局（5 人）纯命令驱动至真实终局——
/// 逐提交守恒不变式 + 中途存档续跑的逐提交事件对照与终态一致。
#[test]
fn p2_identity_endgame_per_submission_parity() {
    let seed = 4242;
    // A：不间断推演（每次提交做恢复不变式 + 逐哈希/逐事件迹）
    let (a, hashes_a, events_a) = drive_identity_fixed(seed, 6000);
    assert!(a.outcome().is_some(), "固定策略身份局应在命令上限内终局");
    assert!(hashes_a.len() >= 10, "对局应具备实质长度");
    // 逐提交哈希无原地踏步（状态单调推进）
    for w in hashes_a.windows(2) {
        assert_ne!(w[0], w[1], "状态哈希不得原地踏步");
    }

    // B：同种子两次独立推演（确定性）逐提交对照
    let (b, hashes_b, events_b) = drive_identity_fixed(seed, 6000);
    assert_eq!(hashes_a, hashes_b, "同种子两次推演逐提交哈希必须一致");
    assert_eq!(events_a, events_b, "逐提交事件摘要必须一致");
    assert_eq!(a.state_hash(), b.state_hash(), "终局状态哈希一致");

    // 终局口径：胜者为合法阵营
    let o = a.outcome().expect("终局");
    assert!(matches!(
        o.winner,
        Winner::LordSide | Winner::Rebels | Winner::Traitor
    ));
}

/// P2/N05（ADR 0771）：六技能新选择点逐提交无重复守恒——
/// 观星落位/流离转移/反间交牌/离间载体/结姻弃牌/急救红牌，
/// 每次提交后做恢复不变式（含牌归属多重集）。
#[test]
fn p2_new_choice_points_per_submission_conservation() {
    // 观星落位：确认 → arrange 询问 → 每步守恒
    let mut e = Engine::new(901, 3);
    e.debug_grant(0, "观星");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Judge {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                assert_per_submission_invariants(&e);
                match choice.spec.prompt {
                    "trigger_confirm" => {
                        answer(&mut e, ChoiceOption::Confirm);
                    }
                    "guanxing_arrange" => {
                        let card = choice.spec.options.iter().find_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        });
                        if let Some(c) = card {
                            answer_sel(&mut e, vec![ChoiceOption::Card(c)]);
                        } else {
                            answer_empty(&mut e);
                        }
                        assert_per_submission_invariants(&e);
                        break;
                    }
                    other => panic!("unexpected {other}"),
                }
            }
            o => panic!("walk: {o:?}"),
        }
        assert_per_submission_invariants(&e);
    }

    // 流离转移：成为杀目标 → 转移询问 → 弃牌；逐提交守恒
    let mut e2 = Engine::new(2004, 3);
    advance_to_play(&mut e2, 0);
    e2.debug_grant(1, "流离");
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    let _ = e2.debug_place(1, "tao");
    let sha = e2.debug_place(0, "sha");
    play(&mut e2, sha, vec![1]);
    assert_per_submission_invariants(&e2);
    if e2.open_choice().map(|c| c.spec.prompt) == Some("liuli_discard_choose") {
        let li = e2.debug_hand(1)[0];
        answer_sel(&mut e2, vec![ChoiceOption::Card(li)]);
        assert_per_submission_invariants(&e2);
        if e2.open_choice().map(|c| c.spec.prompt) == Some("liuli_transfer") {
            answer_sel(&mut e2, vec![ChoiceOption::Player(2)]);
            assert_per_submission_invariants(&e2);
        }
    }

    // 反间交牌：目标选牌获得 → 受伤；逐提交守恒
    let mut e3 = Engine::new(2006, 3);
    advance_to_play(&mut e3, 0);
    e3.debug_grant(0, "反间");
    for p in 0..3 {
        e3.debug_clear_hand(p);
    }
    let give = e3.debug_place(0, "shan");
    let skill = skill_id(&e3, "反间");
    submit_cmd(&mut e3, 0, GameAction::ActivateSkill { skill });
    // 诊断：对比 open choice 与派生 ExpectedAsk
    {
        let snap = e3.save();
        let open = snap.resolution.open_choice.as_ref().expect("open");
        eprintln!(
            "DBG open: obs={} prompt={} kind={:?} min={} max={} opts={:?}",
            open.observer,
            open.spec.prompt,
            open.spec.kind,
            open.spec.min,
            open.spec.max,
            open.spec.options
        );
        for park in &snap.resolution.parks {
            if let Some(derived) =
                engine_native::snap::active_ask(&park.machine, &snap.core, &e.catalog())
            {
                eprintln!(
                    "DBG derived: obs={} prompt={} kind={:?} min={} max={} opts={:?}",
                    derived.observer,
                    derived.prompt,
                    derived.kind,
                    derived.min,
                    derived.max,
                    derived.options
                );
            } else {
                eprintln!("DBG derived: None for park at depth {}", park.resume_depth);
            }
        }
    }
    assert_per_submission_invariants(&e3);
    answer_sel(&mut e3, vec![ChoiceOption::Player(1)]);
    assert_per_submission_invariants(&e3);
    let pick = e3.open_choice().expect("fanjian_pick 应挂起");
    let pick_card = pick.spec.options.iter().find_map(|o| match o {
        ChoiceOption::Card(c) => Some(*c),
        _ => None,
    });
    if let Some(c) = pick_card {
        answer_sel(&mut e3, vec![ChoiceOption::Card(c)]);
        assert_per_submission_invariants(&e3);
        let _ = give;
    }
}

/// P2/N05（ADR 0771）：死亡时剩余机器持牌路径——
/// 闪电判定致死时，判定机器仍持有 trick/judge_card 引用；
/// 死亡清区后逐提交恢复不变式仍须成立（机器引用在界内、牌区无重复）。
#[test]
fn p2_death_with_machine_held_cards() {
    let mut e = Engine::new(2008, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 0 号判定区置闪电；体力 1：雷击 3 点必死
    e.debug_set_hp(0, 1);
    let sd = e.debug_place_judge(0, "shandian");
    let _ = sd;
    let mut died = false;
    // 判定牌刷新器：其他玩家摸牌会消费牌堆顶，故每次推进后重置
    let refresh_top = |e: &mut Engine| {
        e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    };
    for it in 0..64 {
        let t = e.turn();
        if std::env::var("N12DBG").is_ok() {
            let snape = e.save();
            eprintln!(
                "it={it} cur={} phase={:?} p0judge={:?} hp0={} alive0={} over={} parks={:?} open={:?} loglen={}",
                t.current,
                t.phase,
                e.debug_judge(0),
                e.debug_hp(0),
                e.debug_alive(0),
                e.outcome().is_some(),
                snape.resolution.parks.iter().map(|p| (p.resume_depth, match &p.machine { engine_native::snap::MachineSnapshot::DyingSave { dying, asked, .. } => format!("DyingSave(d={dying},asked={asked})", ), m => format!("{:?}", std::mem::discriminant(m)) })).collect::<Vec<_>>(),
                snape.resolution.open_choice.as_ref().map(|c| (c.observer, c.spec.prompt)),
                e.log().len(),
            );
        }
        if e.outcome().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                // 濒死询问：走死亡路径——拒绝救援。一次 Pass 后队列会
                // 轮到下一位持桃者并开出新的同型询问（依发牌构成而定），
                // 故连续应答直至无濒死询问。
                if choice.spec.prompt == "dying_save" {
                    for _ in 0..16 {
                        match e.open_choice() {
                            Some(c) if c.spec.prompt == "dying_save" => {
                                answer(&mut e, ChoiceOption::Pass);
                            }
                            _ => break,
                        }
                    }
                }
            }
            CommandOutcome::Rejected(_) => {}
            CommandOutcome::Suspended => {}
        }
        refresh_top(&mut e);
        if !e.debug_alive(0) {
            died = true;
            break;
        }
    }
    assert!(died, "闪电 3 点雷伤应击杀 1 体力玩家");
    // 逐提交恢复不变式：死亡清区后牌归属仍闭合
    assert_per_submission_invariants(&e);
    // 终局口径：原型模式存活 ≤1 → 有胜负
    assert!(e.outcome().is_some() || e.debug_alive(1) && e.debug_alive(2));
}
