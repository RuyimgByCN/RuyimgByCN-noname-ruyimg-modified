//! NC12 §5.3 复现：swd 扩展 variant-1 下**权威结算路径 panic**（`apply_plan`→`submit`
//! →`commit_frame`，seed 42009 step 68，`card 204 not in hand of 0`）。
//!
//! 与 §5.2（规划模拟内 panic）不同：本用例**不经** `plan_action` 的克隆模拟，
//! 直接逐命令 `submit` 真实引擎，故 panic 出现在权威路径上。
//!
//! 诊断：捕获 panic 前逐步记录「命令序号 / 座位 / 动作 / 手牌与区域分布」，
//! 定位是哪条动作在提交后触发了 `take_hand_card` 对不在手牌的牌。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, GameCommand};

fn pack_of(dir: &str) -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join(dir).join("assets/cards.json"))
        .unwrap_or_else(|e| panic!("读 {dir}: {e}"));
    content_native_loader::bridge_pack(&json)
        .unwrap_or_else(|e| panic!("桥接 {dir}: {e}"))
        .pack
}

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    pack_of("shahai-swd-upstream-noname")
}

/// 某张牌的区域分布（用于 panic 后的定位打印）。
fn zones_of(e: &Engine, card: u32) -> String {
    let core = e.state_core();
    let mut out = Vec::new();
    for (i, p) in core.players.iter().enumerate() {
        if p.hand.contains(&card) {
            out.push(format!("hand[{i}]"));
        }
        if p.equips.contains(&card) {
            out.push(format!("equip[{i}]"));
        }
        if p.judge.contains(&card) {
            out.push(format!("judge[{i}]"));
        }
        if p.off_court.contains(&card) {
            out.push(format!("offcourt[{i}]"));
        }
    }
    let deck = core.deck.iter().filter(|c| **c == card).count();
    let discard = core.discard.iter().filter(|c| **c == card).count();
    if deck > 0 {
        out.push(format!("deck×{deck}"));
    }
    if discard > 0 {
        out.push(format!("discard×{discard}"));
    }
    if out.is_empty() {
        out.push("（不在任何已建模区域）".to_string());
    }
    out.join("+")
}

/// 组合档（huanle + swd + gujian）稳定性：NC12 §5.2 的 combo 场景同口径复跑。
///
/// §5.2 记录：combo + variant1 在 `plan_action` 模拟提交内 panic
/// （`take_hand_card`「card 90 not in hand of 0」、`monkey.rs:73` 被拦截的桃）。
#[test]
fn combo_tier_variant1_is_panic_free() {
    let packs = [
        pack_of("huanle-upstream-noname"),
        swd_pack(),
        pack_of("shahai-gujian-upstream-noname"),
    ];
    let refs: Vec<&engine_native::content::extension::ExtensionPack> = packs.iter().collect();
    let planner = BoundedPlanner::new(64)
        .with_deadline_ms(50)
        .with_policy_variant(1);
    let mut steps = 0u64;
    for seed in [42007u64, 42008, 42009] {
        let mut e = Engine::new_with_extensions(seed, 4, &refs).expect("组合档建局");
        for step in 0..120u64 {
            if e.outcome().is_some() {
                break;
            }
            let seat = e.turn().current;
            let plan = e.plan_action(seat, &planner);
            let out = e.submit(GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: step + 1,
                action: plan.action.clone(),
            });
            let _ = out;
            steps += 1;
        }
    }
    assert!(steps > 50, "组合档实际推进 {steps} 步");
}

/// 权威路径稳定性：swd + variant-1 连续提交 N 步不得 panic。
///
/// 修复前：seed 42009 step 68 panic（`card 204 not in hand of 0`）。
#[test]
fn swd_authoritative_path_is_panic_free() {
    let pack = swd_pack();
    let planner = BoundedPlanner::new(64)
        .with_deadline_ms(50)
        .with_policy_variant(1);
    let mut steps = 0u64;
    for seed in [42009u64, 42010, 42011] {
        let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
        let mut last: Option<(u64, u8, String)> = None;
        let _ = &last;
        for step in 0..200u64 {
            if e.outcome().is_some() {
                break;
            }
            let seat = e.turn().current;
            let plan = e.plan_action(seat, &planner);
            last = Some((step, seat, format!("{:?}", plan.action)));
            let out = e.submit(GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: step + 1,
                action: plan.action.clone(),
            });
            let _ = out;
            steps += 1;
            // 每步检查牌区唯一性（与 NC07 修复同口径）
            for card in 0..e.catalog().cards.len() as u32 {
                let core = e.state_core();
                let mut zones = 0usize;
                for p in core.players.iter() {
                    for zone in [&p.hand, &p.equips, &p.judge, &p.off_court] {
                        if zone.contains(&card) {
                            zones += 1;
                        }
                    }
                }
                zones += core.deck.iter().filter(|c| **c == card).count();
                zones += core.discard.iter().filter(|c| **c == card).count();
                assert!(
                    zones <= 1,
                    "seed={seed} step={step} 牌 {card} 出现在 {zones} 个区域：{}（动作 {:?}）",
                    zones_of(&e, card),
                    last.as_ref().map(|l| l.2.clone()).unwrap_or_default()
                );
            }
        }
    }
    assert!(steps > 100, "实际推进 {steps} 步（样本量足够）");
}
