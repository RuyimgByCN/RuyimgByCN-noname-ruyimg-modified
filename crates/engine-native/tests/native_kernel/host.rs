use super::*;

/// 阶段 2 宿主装配器原型契约（ADR 0764，examples/host_assembler 同款
/// 逻辑锁定进套件）：官方将池确定性分发 → 全暗置入局 → 装配后
/// 视图隔离（观战者帧零将领名、本人帧含自己暗置主将）与同种子确定性。
#[test]
fn host_assembler_prototype_contract() {
    // 将池：剔除机制样例后的官方条目
    let pool: Vec<(u16, &'static str)> = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .enumerate()
        .filter(|(_, (name, _, _, _, _))| !name.starts_with("样例"))
        .map(|(i, (name, _, _, _, _))| (i as u16, *name))
        .collect();
    assert!(
        pool.len() >= 25,
        "官方将池应含 25 名武将（张角组合转机制样例）"
    );

    // 宿主种子确定性洗牌（Fisher-Yates，宿主 RNG 独立于对局 RNG）
    let deal = |host_seed: u64| -> Vec<(u16, &'static str)> {
        let mut p = pool.clone();
        let mut s = host_seed | 1;
        for i in (1..p.len()).rev() {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = ((s >> 33) as usize) % (i + 1);
            p.swap(i, j);
        }
        p[..6].to_vec() // 3 人局 × 2 槽
    };
    let dealt = deal(7);
    let dealt2 = deal(7);
    assert_eq!(dealt, dealt2, "同宿主种子分发必须确定");

    // 注入国战局（全暗置）并恢复
    let build = |d: &[(u16, &'static str)]| -> Engine {
        use engine_native::state::GameMode;
        let e = Engine::new(42, 3);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        for (seat, slot) in snap.core.players.iter_mut().enumerate() {
            slot.general_slots = [Some(d[seat * 2].0), Some(d[seat * 2 + 1].0)];
            slot.revealed_generals = [false, false];
        }
        Engine::restore(&snap).expect("装配快照可恢复")
    };
    let e = build(&dealt);

    // 视图隔离：观战者帧零将领名；本人帧含自己暗置主将
    let spectator = e.render_frame(None, 72);
    for (_, name) in &pool {
        assert!(!spectator.contains(name), "观战者帧不得出现将领 {name}");
    }
    let own = e.render_frame(Some(0), 72);
    assert!(own.contains(dealt[0].1), "本人帧应含自己暗置主将");
    assert!(!own.contains(dealt[2].1), "他人暗置将不得入本人帧");

    // 同种子（对局 + 宿主）重装配逐哈希一致
    let e2 = build(&dealt);
    assert_eq!(e.state_hash(), e2.state_hash(), "装配必须确定");
}
