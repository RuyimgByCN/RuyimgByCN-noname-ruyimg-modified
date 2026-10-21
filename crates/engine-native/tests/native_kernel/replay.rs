use super::*;

#[test]
fn replay_determinism_same_seed_identical() {
    let (h1, hand1, hp2_a) = run_script(12345);
    let (h2, hand2, hp2_b) = run_script(12345);
    assert_eq!(h1, h2, "同种子重放必须得到同一状态");
    assert_eq!(hand1, hand2);
    assert_eq!(hp2_a, hp2_b);
}

#[test]
fn replay_different_seed_diverges() {
    let (h1, _, _) = run_script(12345);
    let (h2, _, _) = run_script(999);
    assert_ne!(h1, h2, "不同种子应产生不同牌局");
}

/// N06 重放/恢复逐命令对照：同一确定性脚本在三条线上逐命令比对
/// 状态哈希与完整日志事件序列——
/// 基准线（原实例）/ 重放线（同脚本从零重跑）/ 恢复线（半程 save→restore 续跑）。
#[test]
fn replay_and_restore_per_command_parity() {
    // —— 基准线：收集脚本与逐命令（哈希, 全量日志）——
    let mut base = Engine::new(307, 4);
    let mut script: Vec<GameCommand> = Vec::new();
    let mut hashes: Vec<[u8; 32]> = vec![base.state_hash()];
    let mut logs: Vec<Vec<LogEvent>> = vec![Vec::new()];
    while script.len() < 60 && base.outcome().is_none() {
        let cmd = next_script_command(&mut base);
        let _ = base.submit(cmd.clone());
        script.push(cmd);
        hashes.push(base.state_hash());
        logs.push(base.log().to_vec());
    }
    assert!(script.len() >= 40, "脚本应有足够命令量（含阶段推进与攻杀）");

    // —— 重放线：同脚本从零重跑，逐命令全量比对 ——
    let mut rep = Engine::new(307, 4);
    for (i, cmd) in script.iter().enumerate() {
        let _ = rep.submit(cmd.clone());
        assert_eq!(
            rep.state_hash(),
            hashes[i + 1],
            "重放第 {i} 条命令后哈希不一致"
        );
        assert_eq!(rep.log(), &logs[i + 1][..], "重放第 {i} 条命令后日志不一致");
    }

    // —— 恢复线：半程 save→restore 后续跑同脚本 ——
    let k = script.len() / 2;
    let mut half = Engine::new(307, 4);
    for cmd in &script[..k] {
        let _ = half.submit(cmd.clone());
    }
    assert_eq!(half.state_hash(), hashes[k], "半程状态应与基准一致");
    let snap = half.save();
    let mut r = Engine::restore(&snap).expect("restore");
    assert_eq!(r.state_hash(), hashes[k]);
    assert!(r.log().is_empty(), "恢复件宿主日志为空");
    let split = logs[k].len();
    for (j, cmd) in script[k..].iter().enumerate() {
        let _ = r.submit(cmd.clone());
        let idx = k + j + 1;
        assert_eq!(
            r.state_hash(),
            hashes[idx],
            "恢复线第 {idx} 条命令后哈希不一致"
        );
        // 恢复件日志 = 基准线自挂起点之后的完整事件增量
        assert_eq!(
            r.log(),
            &logs[idx][split..],
            "恢复线第 {idx} 条命令后事件增量不一致"
        );
    }
    assert_eq!(
        r.state_hash(),
        *hashes.last().unwrap(),
        "恢复线终态与基准一致"
    );
}
