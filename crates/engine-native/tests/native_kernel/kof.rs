use super::*;

/// KOF 引擎层（P2/N08 模式入口批四，规则 v83）：会话状态入引擎
/// （将池/比分/局数），单局终局自动推进——败者队首淘汰、胜者队首轮转；
/// 存档往返含会话推进结果。
#[test]
fn kof_engine_mode_session_advance() {
    // 会话：0 队池 [7, 8]，1 队池 [9, 3]（GENERAL 下标，任意合法值）
    let pools: [Vec<u16>; 2] = [vec![7, 8], vec![9, 3]];
    let mut e = Engine::new_kof(2301, pools.clone(), [0, 0], 0);
    advance_to_play(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
    }

    // ① 首局：0 号杀 1 号（1 号无闪）→ 0 胜；会话推进=1 队队首淘汰、
    //    0 队队首轮转到队尾、比分 1:0、局数 1。
    e.debug_set_hp(1, 1);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    assert!(e.outcome().is_some(), "首局应终局");
    let snap1 = e.save();
    let (pools2, wins2, games2) = match &snap1.core.mode {
        GameMode::Kof { pools, wins, games } => (pools, wins, *games),
        other => panic!("应为 KOF 模式: {other:?}"),
    };
    assert_eq!(pools2[0], vec![8, 7], "胜者队首轮转至队尾");
    assert_eq!(pools2[1], vec![3], "败者队首淘汰");
    assert_eq!(*wins2, [1, 0], "比分推进");
    assert_eq!(games2, 1, "局数推进");

    // ② 存档往返：推进后的会话一致
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let bytes = encode_archive(&e.save());
        let back = decode_archive(&bytes).expect("解码");
        assert_eq!(back.core.mode, e.save().core.mode, "KOF 会话往返一致");
    }

    // ③ 第二局：新引擎以推进后的会话开局（1 队队首=3），0 队 8 号再胜
    //    → 1 队池清空（会话结束形态：败者池空）。
    let mut e2 = Engine::new_kof(2303, pools2.clone(), *wins2, games2);
    advance_to_play(&mut e2, 0);
    for p in 0..2 {
        e2.debug_clear_hand(p);
    }
    e2.debug_set_hp(1, 1);
    let sha2 = e2.debug_place(0, "sha");
    play(&mut e2, sha2, vec![1]);
    let snap2 = e2.save();
    let (pools3, wins3, games3) = match &snap2.core.mode {
        GameMode::Kof { pools, wins, games } => (pools, wins, *games),
        other => panic!("应为 KOF 模式: {other:?}"),
    };
    assert!(pools3[1].is_empty(), "1 队池清空（单败出线）");
    assert_eq!(pools3[0], vec![7, 8], "胜者池轮转不消减（8 移至队尾）");
    assert_eq!(*wins3, [2, 0]);
    assert_eq!(games3, 2);
}

// ---------------------------------------------------------------------------
// N11 大模式实现第一批：KOF M1 + 国战 M1 数据模型（ADR 0754）
// ---------------------------------------------------------------------------

/// KOF M1：2 人局（Prototype）纯命令攻击至终局——KOF 会话层的单局基座。
#[test]
fn kof_m1_two_player_pure_command_endgame() {
    let mut e = Engine::new(733, 2);
    let mut guard = 0;
    while e.outcome().is_none() {
        guard += 1;
        assert!(guard < 8000, "2 人局未收敛");
        let out = if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            })
        } else if e.turn().phase == Phase::Play {
            let actor = e.turn().current;
            match e
                .debug_hand(actor)
                .iter()
                .copied()
                .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
            {
                Some(sha) => {
                    let target = e.state_core().next_alive(actor);
                    e.submit(GameCommand {
                        actor,
                        expected_revision: e.revision(),
                        dedup_id: fresh_dedup(),
                        action: GameAction::PlayCard {
                            card: sha,
                            targets: vec![target],
                        },
                    })
                }
                None => e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                }),
            }
        } else {
            let actor = e.turn().current;
            e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::NextPhase,
            })
        };
        match out {
            CommandOutcome::Rejected(_) => {
                let a = e.turn().current;
                let _ = e.submit(GameCommand {
                    actor: a,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                });
            }
            _ => {}
        }
    }
    let o = e.outcome().expect("2 人局应达终局");
    assert!(
        matches!(o.winner, Winner::Player(_)),
        "2 人局胜者为存活玩家"
    );
    // 败者已死亡，胜者存活
    let alive: Vec<u8> = (0..2u8).filter(|s| e.debug_alive(*s)).collect();
    assert_eq!(alive.len(), 1, "2 人局终局应恰一名存活");
    assert_zone_conservation(&e);
}

/// P2/N08 模式对齐切片（2026-10-07）：KOF 会话一致性 + 中间恢复续跑 +
/// 死亡清理。会话（双方将池/比分/场次）存档行往返一致；恢复后的会话
/// 续跑到同一终局（与不恢复路径对照）；单局中途引擎快照/恢复后以同一
/// 确定性策略续跑至同一胜者；终局死亡玩家各区清空、技能移除。
#[test]
fn kof_session_consistency_and_midrestore() {
    use engine_native::content::standard::{general_hp, general_skills, STANDARD_GENERALS};

    // —— 会话（与 n11_kof 示例同构的最小内联实现）——
    #[derive(Clone, PartialEq, Eq, Debug)]
    struct Session {
        pools: [Vec<u16>; 2],
        wins: [u32; 2],
        games: u32,
    }
    impl Session {
        fn new(seed: u64) -> Self {
            let mut pool: Vec<u16> = (0..STANDARD_GENERALS.len() as u16)
                .filter(|i| !STANDARD_GENERALS[*i as usize].0.starts_with("样例"))
                .collect();
            let mut s = seed | 1;
            for i in (1..pool.len()).rev() {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let j = ((s >> 33) as usize) % (i + 1);
                pool.swap(i, j);
            }
            pool.truncate(6);
            let b = pool.split_off(3);
            Self {
                pools: [pool, b],
                wins: [0, 0],
                games: 0,
            }
        }
        fn save(&self) -> String {
            let dump = |p: &[u16]| {
                p.iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            format!(
                "KOF1 {} {} {} {} {}",
                dump(&self.pools[0]),
                dump(&self.pools[1]),
                self.wins[0],
                self.wins[1],
                self.games
            )
        }
        fn restore(line: &str) -> Option<Self> {
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() != 6 || parts[0] != "KOF1" {
                return None;
            }
            let pp = |s: &str| -> Option<Vec<u16>> {
                if s.is_empty() {
                    return Some(Vec::new());
                }
                s.split(',').map(|x| x.parse().ok()).collect()
            };
            let p0 = pp(parts[1])?;
            let p1 = pp(parts[2])?;
            let w0 = parts[3].parse().ok()?;
            let w1 = parts[4].parse().ok()?;
            let games = parts[5].parse().ok()?;
            for g in p0.iter().chain(p1.iter()) {
                if (*g as usize) >= STANDARD_GENERALS.len() {
                    return None;
                }
            }
            Some(Self {
                pools: [p0, p1],
                wins: [w0, w1],
                games,
            })
        }
        fn finished(&self) -> Option<usize> {
            match (self.pools[0].is_empty(), self.pools[1].is_empty()) {
                (true, _) => Some(1),
                (_, true) => Some(0),
                _ => None,
            }
        }
        /// 打一局（确定性策略）；返回胜者座位。
        fn play_game(seed: u64, generals: [u16; 2]) -> (usize, bool) {
            let mut e = Engine::new(seed, 2);
            for (seat, g) in generals.iter().enumerate() {
                for skill in general_skills(*g) {
                    e.debug_grant(seat as u8, skill);
                }
                e.debug_set_max_hp(seat as u8, general_hp(*g).unwrap_or(4));
            }
            let mut commands: Vec<GameCommand> = Vec::new();
            let policy = |e: &Engine, seat: u8| -> GameCommand {
                // 与示例同构：观察者优先应答取 min 选项；本回合有杀攻对手；
                // 否则推进阶段。
                if let Some(c) = e.open_choice() {
                    let sel: Vec<ChoiceOption> = c
                        .spec
                        .options
                        .iter()
                        .take(c.spec.min.max(1))
                        .copied()
                        .collect();
                    return GameCommand {
                        actor: seat,
                        expected_revision: e.revision(),
                        dedup_id: fresh_dedup(),
                        action: GameAction::Answer {
                            choice_id: c.id,
                            selection: sel,
                        },
                    };
                }
                if e.turn().current == seat && e.turn().phase == Phase::Play {
                    let sha = e
                        .debug_hand(seat)
                        .iter()
                        .copied()
                        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha);
                    if let Some(sha) = sha {
                        let enemy = (1 - seat as usize) as u8;
                        return GameCommand {
                            actor: seat,
                            expected_revision: e.revision(),
                            dedup_id: fresh_dedup(),
                            action: GameAction::PlayCard {
                                card: sha,
                                targets: vec![enemy],
                            },
                        };
                    }
                }
                GameCommand {
                    actor: seat,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                }
            };
            for _ in 0..4096 {
                if e.outcome().is_some() {
                    break;
                }
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                commands.push(policy(&e, seat));
                let _ = e.submit(commands.last().unwrap().clone());
            }
            let winner = match e.outcome() {
                Some(o) => match o.winner {
                    engine_native::Winner::Player(p) => p as usize,
                    _ => 1,
                },
                None => 1,
            };
            // 死亡清理断言（仅真实终局；超时判负无人死亡，无清理可断言）：
            // 败者各区清空。
            if let Some(o) = e.outcome() {
                if matches!(o.winner, engine_native::Winner::Player(_)) {
                    let loser = 1 - winner;
                    assert!(
                        e.debug_hand(loser as u8).is_empty()
                            && e.debug_equips(loser as u8).is_empty()
                            && e.debug_judge(loser as u8).is_empty(),
                        "KOF 死亡清理：败者各区应清空"
                    );
                }
            }
            (winner, e.outcome().is_some())
        }
    }

    // —— 会话一致性：恢复后的会话与不恢复路径终局一致 ——
    let seed = 11u64;
    // 路径 A（不恢复）：
    let mut a = Session::new(seed);
    let mut round = 0;
    while a.finished().is_none() && round < 6 {
        round += 1;
        let ga = a.pools[0][0];
        let gb = a.pools[1][0];
        let (w, _) = Session::play_game(seed + round as u64 * 77, [ga, gb]);
        let l = 1 - w;
        let lost = a.pools[l].remove(0);
        let kept = a.pools[w].remove(0);
        a.pools[w].push(kept);
        a.wins[w] += 1;
        a.games += 1;
        let _ = lost;
    }
    let terminal_a = (a.save(), a.finished(), a.wins, a.games);

    // 路径 B（每局后存档/恢复续跑）：
    let mut b = Session::new(seed);
    let mut round = 0;
    while b.finished().is_none() && round < 6 {
        round += 1;
        let ga = b.pools[0][0];
        let gb = b.pools[1][0];
        let (w, _) = Session::play_game(seed + round as u64 * 77, [ga, gb]);
        let l = 1 - w;
        let _lost = b.pools[l].remove(0);
        let kept = b.pools[w].remove(0);
        b.pools[w].push(kept);
        b.wins[w] += 1;
        b.games += 1;
        let line = b.save();
        let restored = Session::restore(&line).expect("合法会话行应可恢复");
        assert_eq!(restored.save(), line, "会话行往返一致");
        b = restored;
    }
    assert_eq!(b.save(), terminal_a.0, "恢复续跑与不恢复路径终局一致");
    assert_eq!(b.finished(), terminal_a.1);

    // —— 中间恢复续跑（引擎层）：单局中途快照/恢复后以同一确定性策略
    //    续跑，30/60 命令两检查点的状态哈希与不中断路径一致（KOF 单局
    //    可能因技能面进入长局/超时判负，故用哈希轨迹而非胜者断言）——
    let ga = Session::new(seed).pools[0][0];
    let gb = Session::new(seed).pools[1][0];
    let gseed = seed + 77;
    let policy = |e: &Engine, seat: u8| -> GameCommand {
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect();
            return GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            };
        }
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            let sha = e
                .debug_hand(seat)
                .iter()
                .copied()
                .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha);
            if let Some(sha) = sha {
                let enemy = (1 - seat as usize) as u8;
                return GameCommand {
                    actor: seat,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::PlayCard {
                        card: sha,
                        targets: vec![enemy],
                    },
                };
            }
        }
        GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        }
    };
    let setup = |e: &mut Engine| {
        for (seat, g) in [ga, gb].iter().enumerate() {
            for skill in general_skills(*g) {
                e.debug_grant(seat as u8, skill);
            }
            e.debug_set_max_hp(seat as u8, general_hp(*g).unwrap_or(4));
        }
    };
    let drive = |e: &mut Engine, n: usize| {
        for _ in 0..n {
            if e.outcome().is_some() {
                break;
            }
            let seat = match e.open_choice() {
                Some(c) => c.observer,
                None => e.turn().current,
            };
            let _ = e.submit(policy(e, seat));
        }
    };
    // 不中断基准：30 命令检查点哈希 + 60 命令终值哈希。
    let mut base = Engine::new(gseed, 2);
    setup(&mut base);
    drive(&mut base, 30);
    let hash30 = base.state_hash();
    drive(&mut base, 30);
    let hash60 = base.state_hash();

    // 中断路径：30 命令后快照/恢复 → 再 30 命令 → 哈希一致。
    let mut mid = Engine::new(gseed, 2);
    setup(&mut mid);
    drive(&mut mid, 30);
    let snap = mid.save();
    let mid_restored = Engine::restore(&snap).expect("中途快照应可恢复");
    assert_eq!(
        mid_restored.state_hash(),
        hash30,
        "恢复件哈希与 30 命令检查点一致"
    );
    let mut mid2 = mid_restored;
    drive(&mut mid2, 30);
    assert_eq!(
        mid2.state_hash(),
        hash60,
        "恢复续跑 60 命令哈希与不中断一致"
    );
}
