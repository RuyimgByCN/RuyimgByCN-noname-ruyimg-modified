//! N12 跨进程验收（ADR 0735）：字节存档落盘 → 另一进程恢复 → 同哈希续跑。
//!
//! 用法：
//! - `n12_crossproc write <archive-file>`：seed 443 布局 25 条确定性命令后
//!   存档落盘，打印 H25；继续 10 条后打印 H35。
//! - `n12_crossproc read <archive-file>`：同种子独立重放至同状态，从文件
//!   恢复存档并校验 H25 一致；继续 10 条后打印 H35。
//!
//! 验收：read 进程的 H25/H35 与 write 进程完全一致——存档跨进程保真、
//! 续跑确定性成立。

use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn fresh() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// 确定性命令策略：选择窗口按「牌选项优先、无牌取首选项」应答；
/// 出牌阶段在本回合未出杀且距离可达时出杀；否则推进阶段。
fn policy_cmd(e: &mut Engine) -> GameCommand {
    if let Some(c) = e.open_choice() {
        let mut sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        if sel.is_empty() {
            sel = vec![c.spec.options[0]];
        }
        return GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        };
    }
    let t = e.turn();
    if t.phase == Phase::Play && !t.sha_used_play_phase {
        let actor = t.current;
        let sha = e
            .debug_hand(actor)
            .iter()
            .copied()
            .find(|c| e.catalog().card_kind(*c) == CardKind::Sha);
        if let Some(sha) = sha {
            let core = e.state_core();
            let target = core.next_alive(actor);
            if core.computed_distance(e.catalog(), actor, target)
                <= core.attack_range(e.catalog(), actor)
            {
                return GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::PlayCard { card: sha, targets: vec![target] },
                };
            }
        }
    }
    let actor = e.turn().current;
    GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: fresh(),
        action: GameAction::NextPhase,
    }
}

/// 推进 n 条命令（拒绝的策略会自动重算，不计数）。
fn drive(e: &mut Engine, n: usize) {
    let mut done = 0;
    let mut guard = 0;
    while done < n {
        guard += 1;
        assert!(guard < 8 * n + 16, "drive 未收敛");
        let cmd = policy_cmd(e);
        match e.submit(cmd) {
            CommandOutcome::Rejected(_) => continue,
            _ => done += 1,
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "write".into());
    let path = args.next().unwrap_or_else(|| "/tmp/n12_archive.bin".into());

    // 两个进程各自独立：同种子 → 同状态 → 同命令序列
    let mut e = Engine::new(443, 4);
    drive(&mut e, 25);
    let h25 = hex(&e.state_hash());

    // write：存档落盘；read：从文件恢复（跨进程）
    let mut e = match mode.as_str() {
        "write" => {
            let snap = e.save();
            let bytes = engine_native::archive::encode_archive(&snap);
            std::fs::write(&path, &bytes).expect("写存档失败");
            println!("writer H25={h25} archive_bytes={}", bytes.len());
            e
        }
        "read" => {
            let bytes = std::fs::read(&path).expect("读存档失败");
            let snap = engine_native::archive::decode_archive(&bytes).expect("存档解码");
            let r = Engine::restore(&snap).expect("存档恢复");
            let restored_h25 = hex(&r.state_hash());
            println!(
                "reader replay_H25={h25} restored_H25={restored_h25} match={}",
                h25 == restored_h25
            );
            r
        }
        m => panic!("未知模式 {m}（write|read）"),
    };

    // 续跑 10 条并输出终态哈希（两进程应一致）
    drive(&mut e, 10);
    println!("{mode} H35={}", hex(&e.state_hash()));
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
