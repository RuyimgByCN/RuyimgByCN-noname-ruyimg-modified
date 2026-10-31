//! 阶段 2 宿主装配器原型（ADR 0764）：武将配置宿主装配的引擎侧样例。
//!
//! 展示宿主侧「将池分发 → 暗置入局 → 配置校验 → 开局」的最小装配流：
//! 1. 从目录将池（`STANDARD_GENERALS`，剔除「样例」条目）按宿主种子
//!    确定性洗牌，为每座位发主将/副将（暗置）；
//! 2. 经存档快照注入国战局（引擎不感知宿主装配过程）；
//! 3. 校验装配结果：视图隔离（他人暗置将不可见）+ 声明势力互异约束
//!    由对局胜负判定消费。
//!
//! 运行：cargo run --release -p engine-native --example host_assembler -- [seats] [seed]

use engine_native::{
    content::standard::{faction_of_general, STANDARD_GENERALS},
    state::GameMode,
    Engine,
};

/// 宿主侧将池条目（目录下标 + 名称 + 势力）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolEntry {
    pub index: u16,
    pub name: &'static str,
    pub hp: u32,
}

/// 从目录构建官方将池（剔除「样例」前缀的机制样例条目）。
pub fn official_pool() -> Vec<PoolEntry> {
    STANDARD_GENERALS
        .iter()
        .enumerate()
        .filter(|(_, (name, _, _, _, _))| !name.starts_with("样例"))
        .map(|(i, (name, _, _, hp, _))| PoolEntry {
            index: i as u16,
            name,
            hp: *hp,
        })
        .collect()
}

/// 确定性洗牌（宿主种子独立于对局种子；Fisher-Yates，乘法混入）。
pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut s = seed | 1;
    for i in (1..items.len()).rev() {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((s >> 33) as usize) % (i + 1);
        items.swap(i, j);
    }
}

/// 装配一局 `seats` 人国战：从洗好的将池为每座位发主/副将（暗置），
/// 返回装配完成的引擎与其快照（宿主可继续下发/持久化）。
pub fn assemble_kingdoms(
    seats: usize,
    table_seed: u64,
    host_seed: u64,
) -> (Engine, Vec<PoolEntry>) {
    let mut e = Engine::new(table_seed, seats);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;

    let mut pool = official_pool();
    shuffle(&mut pool, host_seed);
    assert!(pool.len() >= seats * 2, "将池不足以支持 {seats} 人局");
    let dealt: Vec<PoolEntry> = pool.drain(..seats * 2).collect();

    for (seat, slot) in snap.core.players.iter_mut().enumerate() {
        let main = dealt[seat * 2];
        let vice = dealt[seat * 2 + 1];
        slot.general_slots = [Some(main.index), Some(vice.index)];
        slot.revealed_generals = [false, false]; // 全暗置：明置由对局内门控驱动
    }

    e = Engine::restore(&snap).expect("装配快照必须可恢复");
    (e, dealt)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seats: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let table_seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
    let host_seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(7);

    let (e, dealt) = assemble_kingdoms(seats, table_seed, host_seed);
    println!(
        "=== 宿主装配器原型：{} 人国战（table_seed={table_seed} host_seed={host_seed}）===",
        seats
    );
    for (seat, pair) in dealt.chunks(2).enumerate() {
        println!(
            "座位{seat}: 主将 {} / 副将 {}（均暗置）",
            pair[0].name, pair[1].name
        );
    }
    // 装配校验①：宿主装配声明可被胜负判定消费（declared_faction 查询）
    for seat in 0..seats as u8 {
        let snap = e.save();
        let main = snap.core.players[seat as usize].general_slots[0]
            .and_then(faction_of_general)
            .map(|f| format!("{f:?}"))
            .unwrap_or_else(|| "?".into());
        let declared = snap
            .core
            .declared_faction(e.catalog(), seat)
            .map(|f| format!("{f:?}"))
            .unwrap_or_else(|| "未明置".into());
        println!("座位{seat}: 主将势力={main} 当前声明势力={declared}");
    }
    // 装配校验②：视图隔离——观战者帧不含任何将领名，本人帧含自己的
    let spectator = e.render_frame(None, 72);
    let own = e.render_frame(Some(0), 72);
    let leaked_to_spectator = official_pool().iter().any(|g| spectator.contains(g.name));
    println!("观战者帧泄将领: {leaked_to_spectator}（必须 false）");
    assert!(!leaked_to_spectator, "装配后暗置将不得泄入观战者帧");
    let own_main = dealt[0].name;
    assert!(own.contains(own_main), "本人帧应含自己的暗置主将");
    // 装配校验③：确定性——同种子重装配逐哈希一致
    let (e2, _) = assemble_kingdoms(seats, table_seed, host_seed);
    assert_eq!(e.state_hash(), e2.state_hash(), "同种子装配必须确定");
    println!("确定性校验: OK（终态哈希一致）");
    println!("装配完成，哈希 = {:?}", &e.state_hash()[..8]);
}
