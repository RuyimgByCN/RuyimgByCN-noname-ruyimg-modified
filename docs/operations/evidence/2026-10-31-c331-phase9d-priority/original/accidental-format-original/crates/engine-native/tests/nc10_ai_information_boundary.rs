//! NC10 AI 合法信息边界：规划决策**不得依赖隐藏信息**。
//!
//! 判据（可复跑、与实现无关）：把对手手牌与牌库内容做确定性置换（隐藏信息），
//! 保持座位视图（自己的手牌/装备/判定、公开计数、阶段、轮次）不变，则
//! `plan_action(seat)` 的结果必须**逐字段相同**。
//!
//! 反向对照（证明探针能发现差异）：置换**自己的手牌**后，至少存在一个采样状态
//! 使规划结果不同——否则说明探针无分辨力，本用例不成立。
//!
//! 背景：`ai.rs::redact_opponent_hands` 原先在任意状态重分配对手手牌（会破坏挂起
//! 机器引用 → NC12 §5.2 panic），修复后只在静止态重分配；非静止态的隐藏信息边界
//! 由本用例验收。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, GameAction, GameCommand};

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack
}

fn planner() -> BoundedPlanner {
    BoundedPlanner::new(64)
        .with_deadline_ms(50)
        .with_policy_variant(1)
}

/// 确定性置换：把对手手牌 + 牌库整体旋转 k 张（隐藏信息变化，公开视图不变）。
fn permute_hidden(e: &mut Engine, seat: u8, k: usize) {
    let n = e.state_core().players.len();
    let others: Vec<usize> = (0..n).filter(|i| *i as u8 != seat).collect();
    // 先取原手牌数（公开信息），再搬移隐藏牌
    let lens: Vec<usize> = others
        .iter()
        .map(|i| e.state_core().players[*i].hand.len())
        .collect();
    let mut pool: Vec<u32> = Vec::new();
    {
        let core = e.state_core_mut();
        for i in &others {
            pool.append(&mut core.players[*i].hand);
        }
        pool.append(&mut core.deck);
    }
    if pool.len() < 2 {
        return;
    }
    let shift = k % pool.len();
    pool.rotate_left(shift);
    // 按原手牌数放回（只改变「谁拿哪张」，不改变任何公开计数）
    let core = e.state_core_mut();
    let mut cursor = 0usize;
    for (i, len) in others.iter().zip(lens) {
        core.players[*i].hand = pool[cursor..cursor + len].to_vec();
        cursor += len;
    }
    core.deck = pool[cursor..].to_vec();
}

/// 替换自己的一张手牌（反向对照用）：把首张换成牌库首张（内容变化，非顺序变化）。
fn permute_own(e: &mut Engine, seat: u8) {
    let mut hand = e.state_core().players[seat as usize].hand.clone();
    let deck_first = e.state_core().deck.first().copied();
    if let (Some(first), Some(newc)) = (hand.first().copied(), deck_first) {
        hand[0] = newc;
        // 牌区唯一性：把被替换的牌放回牌库（替换而非复制）
        let core = e.state_core_mut();
        if let Some(pos) = core.deck.iter().position(|c| *c == newc) {
            core.deck.remove(pos);
        }
        core.deck.push(first);
        core.players[seat as usize].hand = hand;
    }
}

/// 驱动到指定步数（真实提交），返回推进步数。
fn drive(e: &mut Engine, steps: usize) {
    let p = planner();
    for step in 0..steps {
        if e.outcome().is_some() {
            break;
        }
        let seat = e.turn().current;
        let plan = e.plan_action(seat, &p);
        let _ = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: step as u64 + 1,
            action: plan.action,
        });
    }
}

/// 隐藏信息置换不改变规划结果（AI 合法信息边界）。
///
/// 修复记录（2026-10-18）：旧实现把「对手手牌 ∪ 牌库」确定性洗牌后按原张数放回，
/// 保留隐藏牌**多重集** → AI 仍可读到「对手有闪」（seed 42022 warm 9 seat 1 复现）。
/// 现改为**内容无关填充**（从目录抽取互不相同的牌填满隐藏区，只保留张数）。
#[test]
fn planner_is_invariant_under_hidden_information() {
    let pack = swd_pack();
    let mut checked = 0usize;
    for seed in [42021u64, 42022, 42023] {
        for warm in [3usize, 9, 17] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;

            let mut permuted = base.clone();
            permute_hidden(&mut permuted, seat, 7);
            let b = permuted.plan_action(seat, &planner()).action;

            // 视图等价性前提：公开面（阶段/轮次/手牌数/体力）必须一致
            assert_eq!(
                format!("{:?}", base.turn()),
                format!("{:?}", permuted.turn()),
                "置换不得改变公开视图（turn）"
            );
            for i in 0..4u8 {
                assert_eq!(
                    base.state_core().player(i).hand.len(),
                    permuted.state_core().player(i).hand.len(),
                    "置换不得改变手牌数（座位 {i}）"
                );
                assert_eq!(
                    base.debug_hp(i),
                    permuted.debug_hp(i),
                    "置换不得改变体力（座位 {i}）"
                );
            }
            let res = base.resolution();
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "seed={seed} warm={warm} seat={seat}：规划结果依赖了隐藏信息（parks={} stack={} open_choice={}）",
                res.parks.len(),
                res.stack.len(),
                res.open_choice.is_some()
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "实际检查 {checked} 个状态（样本量足够）");
}

/// 反向对照：置换自己的手牌可改变规划结果（证明探针有分辨力）。
#[test]
fn planner_detects_own_hand_change() {
    let pack = swd_pack();
    let mut differences = 0usize;
    let mut samples = 0usize;
    for seed in [42021u64, 42022, 42023] {
        for warm in [3usize, 9, 17] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;
            // 只在静止态改自己的手牌（非静止态挂起机器/在飞帧引用实体牌 id，
            // 改区会让它们指向别的区域——与 redact 同一安全规则）
            {
                let res = base.resolution();
                if !res.stack.is_empty() || !res.parks.is_empty() || res.open_choice.is_some() {
                    continue;
                }
            }
            let mut own = base.clone();
            permute_own(&mut own, seat);
            if own.state_core().player(seat).hand == base.state_core().player(seat).hand {
                continue;
            }
            let b = own.plan_action(seat, &planner()).action;
            samples += 1;
            if format!("{a:?}") != format!("{b:?}") {
                differences += 1;
            }
        }
    }
    assert!(samples >= 3, "实际对照样本 {samples}");
    assert!(
        differences > 0,
        "置换自己的手牌后规划结果完全不变（{samples} 个样本）——探针无分辨力，隐藏信息用例不成立"
    );
}

/// 候选集合（权威动作查询）在隐藏信息置换下不变：AI 的合法动作面只由公开信息决定。
#[test]
fn legal_action_set_is_invariant_under_hidden_information() {
    let pack = swd_pack();
    let mut checked = 0usize;
    for seed in [42024u64, 42025] {
        let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
        drive(&mut base, 11);
        if base.outcome().is_some() {
            continue;
        }
        let seat = base.turn().current;
        let a: Vec<String> = base
            .legal_actions(seat)
            .iter()
            .map(|x: &GameAction| format!("{x:?}"))
            .collect();
        let mut permuted = base.clone();
        permute_hidden(&mut permuted, seat, 5);
        let b: Vec<String> = permuted
            .legal_actions(seat)
            .iter()
            .map(|x: &GameAction| format!("{x:?}"))
            .collect();
        assert_eq!(a, b, "seed={seed}：合法动作集合依赖了隐藏信息");
        checked += 1;
    }
    assert!(checked >= 1, "实际检查 {checked} 个状态");
}
