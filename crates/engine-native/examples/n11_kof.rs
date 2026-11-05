//! N11 KOF 会话层（ADR 0755/0769）：将池消耗 + 败者轮换 + 会话存档续跑。
//!
//! 会话状态机（宿主侧，内核不感知轮换）：
//! - 两位选手各持 3 名武将的队列（官方将池确定性抽取，互不重复）；
//! - 每局双方各出队首武将（Prototype 2 人局，明置装配：授予技能与体力）；
//! - 败者的武将淘汰出池，胜者的武将保留回队尾；一方池空 → 会话负；
//! - 会话状态可序列化为文本并存档续跑（save/restore 往返一致）。
//!
//! 运行：cargo run --release -p engine-native --example n11_kof -- [seed]

use engine_native::{
    content::standard::{general_hp, general_skills, STANDARD_GENERALS},
    CardKind, ChoiceOption, ChoiceView, Engine, GameAction, GameCommand, Phase,
    view::HandView,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// KOF 会话：双方将池 + 比分 + 已赛场次。
struct KofSession {
    pools: [Vec<u16>; 2],
    wins: [u32; 2],
    games: u32,
}

impl KofSession {
    /// 从官方将池确定性抽取两队各 3 将（互不重复）。
    fn new(host_seed: u64) -> Self {
        let mut pool: Vec<u16> = (0..STANDARD_GENERALS.len() as u16)
            .filter(|i| !STANDARD_GENERALS[*i as usize].name.starts_with("样例"))
            .collect();
        let mut s = host_seed | 1;
        for i in (1..pool.len()).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = ((s >> 33) as usize) % (i + 1);
            pool.swap(i, j);
        }
        pool.truncate(6);
        let b = pool.split_off(3);
        Self { pools: [pool, b], wins: [0, 0], games: 0 }
    }

    fn name(g: u16) -> &'static str {
        STANDARD_GENERALS[g as usize].name
    }

    /// 会话存档（文本行格式，宿主可落盘；空池显式保留为空字段）。
    fn save(&self) -> String {
        let dump = |p: &[u16]| -> String {
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

    /// 会话续跑（有界可失败解析：空池/损坏/越界/格式不符均返回 None）。
    fn restore(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 6 || parts[0] != "KOF1" {
            return None;
        }
        let parse_pool = |s: &str| -> Option<Vec<u16>> {
            // 空字符串 = 空池（显式支持；split(',') 对空串会产出 [""]）
            if s.is_empty() {
                return Some(Vec::new());
            }
            s.split(',').map(|x| x.parse().ok()).collect()
        };
        let p0 = parse_pool(parts[1])?;
        let p1 = parse_pool(parts[2])?;
        let w0: u32 = parts[3].parse().ok()?;
        let w1: u32 = parts[4].parse().ok()?;
        let games: u32 = parts[5].parse().ok()?;
        // 越界与一致性负例
        for g in p0.iter().chain(p1.iter()) {
            if (*g as usize) >= STANDARD_GENERALS.len() {
                return None;
            }
        }
        Some(Self { pools: [p0, p1], wins: [w0, w1], games })
    }

    fn finished(&self) -> Option<usize> {
        match (self.pools[0].is_empty(), self.pools[1].is_empty()) {
            (true, _) => Some(1),
            (_, true) => Some(0),
            _ => None,
        }
    }
}

/// 明置装配：把武将技能集与体力配置进 2 人局（KOF 单局为非国战
/// Prototype 形态，技能实例直接授予，体力按武将元数据配置）。
fn assemble_duel(e: &mut Engine, generals: [u16; 2]) {
    for (seat, g) in generals.iter().enumerate() {
        for skill in general_skills(*g) {
            e.debug_grant(seat as u8, skill);
        }
        let hp = general_hp(*g).unwrap_or(4);
        e.debug_set_max_hp(seat as u8, hp);
    }
}

/// 最小进攻策略：observer 优先应答（取 min 选项）；本回合有杀攻对手；
/// 否则推进阶段。
fn seat_command(engine: &Engine, seat: u8) -> GameCommand {
    let session = engine.ai_session(seat);
    if let Some(ChoiceView::Own { id, options, min, .. }) = session.choice() {
        let sel: Vec<ChoiceOption> = options.into_iter().take(min.max(1)).collect();
        return GameCommand {
            actor: seat,
            expected_revision: engine.revision(),
            dedup_id: fresh(),
            action: GameAction::Answer { choice_id: id, selection: sel },
        };
    }
    if engine.turn().current == seat && engine.turn().phase == Phase::Play {
        let view = session.view();
        if let HandView::Own(hand) = &view.players[seat as usize].hand {
            let sha = hand
                .iter()
                .copied()
                .find(|c| engine.catalog().cards[*c as usize].kind == CardKind::Sha);
            if let Some(sha) = sha {
                let enemy = (1 - seat) as u8;
                if view.players[enemy as usize].alive {
                    return GameCommand {
                        actor: seat,
                        expected_revision: engine.revision(),
                        dedup_id: fresh(),
                        action: GameAction::PlayCard { card: sha, targets: vec![enemy] },
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

/// 驱动一局至终局；返回胜者座位（0/1）。命令超上限视为 0 号超时负。
fn play_game(table_seed: u64, generals: [u16; 2], max_commands: usize) -> (usize, usize) {
    let mut e = Engine::new(table_seed, 2);
    assemble_duel(&mut e, generals);
    let mut commands = 0usize;
    for _ in 0..max_commands {
        if e.outcome().is_some() {
            break;
        }
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        let _ = e.submit(seat_command(&e, seat));
        commands += 1;
    }
    let winner = match e.outcome() {
        Some(o) => match o.winner {
            engine_native::Winner::Player(p) => p as usize,
            _ => 0,
        },
        None => 1, // 超时：1 号判胜（0 号策略超时）
    };
    (winner, commands)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host_seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(11);

    let mut session = KofSession::new(host_seed);
    println!("=== KOF 会话：将池 A={:?} B={:?} ===", session.pools[0], session.pools[1]);

    let mut round = 0;
    while session.finished().is_none() && round < 6 {
        round += 1;
        let ga = session.pools[0][0];
        let gb = session.pools[1][0];
        let (winner_seat, cmds) = play_game(host_seed + round as u64 * 77, [ga, gb], 2048);
        let loser_seat = 1 - winner_seat;
        println!(
            "第{}局：{} vs {} → 座位{}（{}）胜；命令数 {cmds}",
            round,
            KofSession::name(ga),
            KofSession::name(gb),
            winner_seat,
            KofSession::name(session.pools[winner_seat][0]),
        );
        // 败者武将淘汰出池；胜者武将保留回队尾
        let lost = session.pools[loser_seat].remove(0);
        let kept = session.pools[winner_seat].remove(0);
        session.pools[winner_seat].push(kept);
        session.wins[winner_seat] += 1;
        session.games += 1;
        println!("  淘汰：{}（座位{} 剩池 {:?}）", KofSession::name(lost), loser_seat, session.pools[loser_seat]);
        // 会话存档续跑演示：第 1 局后存档、恢复、继续
        let save_line = session.save();
        if round == 1 {
            let restored = KofSession::restore(&save_line).expect("合法会话应可恢复");
            assert_eq!(restored.save(), save_line, "会话存档往返必须一致");
            println!("  会话存档续跑校验: OK（{save_line}）");
        }
    }
    match session.finished() {
        Some(w) => println!(
            "=== 会话终局：选手{w} 胜（{}:{}，共 {} 局）===",
            session.wins[0], session.wins[1], session.games
        ),
        None => println!("=== 会话未完（达到局数上限）：{}:{} ===", session.wins[0], session.wins[1]),
    }
}
