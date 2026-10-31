//! NC07/NC02 缺陷复现：swd 扩展下**同一张牌在弃牌堆出现两次**导致读档被自己的
//! 所有权校验拒绝（NC12 基线 §5.1 最小复现的测试化版本）。
//!
//! 诊断策略：逐命令推进并在每个命令后扫描「弃牌堆重复卡」，报告**首次出现重复**
//! 的命令序号与卡牌 id，再给出该卡的完整区域分布——用于定位是哪条移动路径把
//! 同一张牌写进了弃牌堆两次。

use engine_native::ai::BoundedPlanner;
use engine_native::{CommandOutcome, Engine, GameCommand};

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

/// 弃牌堆内重复出现的卡（升序）。
fn duplicate_discards(e: &Engine) -> Vec<u32> {
    let mut seen = std::collections::BTreeMap::<u32, usize>::new();
    for c in &e.state_core().discard {
        *seen.entry(*c as u32).or_default() += 1;
    }
    seen.into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(c, _)| c)
        .collect()
}

/// 统计某张牌出现在多少个区域（手牌/装备/判定/牌堆/弃牌/场外）。
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
    let deck = core.deck.iter().filter(|c| **c as u32 == card).count();
    let discard = core.discard.iter().filter(|c| **c as u32 == card).count();
    if deck > 0 {
        out.push(format!("deck×{deck}"));
    }
    if discard > 0 {
        out.push(format!("discard×{discard}"));
    }
    out.join("+")
}

/// 回归（NC07 修复后）：swd 扩展下 60 步内**不得**出现弃牌堆重复卡，且读档成功。
///
/// 修复前：step=17 起弃牌堆出现同一张牌两次（card 13），读档被
/// `card_in_multiple_zones` 拒绝（NC12 基线 §5.1：30/30 被拒）。
#[test]
fn swd_discard_has_no_duplicates_and_restores() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(42007, 4, &[&pack]).expect("建局");
    let planner = BoundedPlanner::new(64).with_deadline_ms(50).with_policy_variant(0);

    let mut first: Option<(u64, Vec<u32>, String, String, Vec<String>)> = None;
    for step in 0..60u64 {
        if e.outcome().is_some() {
            break;
        }
        let seat = e.turn().current;
        let plan = e.plan_action(seat, &planner);
        let discard_before = e.state_core().discard.clone();
        let out = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: step + 1,
            action: plan.action.clone(),
        });
        if !matches!(out, CommandOutcome::Rejected(_)) {
            let dups = duplicate_discards(&e);
            if !dups.is_empty() && first.is_none() {
                let discard_after = e.state_core().discard.clone();
                let mut after = discard_after.clone();
                for c in &discard_before {
                    if let Some(pos) = after.iter().position(|x| x == c) {
                        after.remove(pos);
                    }
                }
                let tail: Vec<String> = e
                    .log()
                    .iter()
                    .rev()
                    .take(14)
                    .map(|ev| format!("{ev:?}"))
                    .collect();
                first = Some((
                    step,
                    dups.clone(),
                    zones_of(&e, dups[0]),
                    format!("seat={seat} action={:?} 新增入弃牌堆={after:?}", plan.action),
                    tail,
                ));
                break;
            }
        }
    }
    if let Some((step, dups, zones, action, tail)) = first {
        panic!(
            "出现弃牌堆重复卡：step={step} cards={dups:?} 分布={zones}\n{action}\n日志尾部:\n{}",
            tail.join("\n")
        );
    }
    // 牌区唯一性成立后，读档必须成功（修复前 30/30 被拒）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snap = engine_native::archive::decode_archive(&bytes).expect("解码");
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("swd 读档应成功");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "读档后摘要一致"
    );
}
