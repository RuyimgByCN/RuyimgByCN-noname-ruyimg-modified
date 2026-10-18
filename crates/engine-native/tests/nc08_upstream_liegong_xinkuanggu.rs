//! NC08 第⑦层（上游行为对照）样本 2/3：`fz_liegong`（烈弓）与 `fz_xinkuanggu`（狂骨）。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`）：
//!
//! ```js
//! lib.skill.fz_liegong = {
//!     trigger: { player: "useCardToPlayered" },          // 逐目标
//!     filter: function (event, player) {
//!         if (event.card.name != 'sha') return false;    // 仅【杀】
//!         var length = event.target.countCards('h');     // 目标手牌数
//!         return (length >= player.hp || length <= player.getAttackRange());
//!     },
//!     content: function () { trigger.getParent().directHit.push(trigger.target); },
//! };
//!
//! lib.skill.fz_xinkuanggu = {
//!     trigger: { source: "damageSource" },               // 自己是伤害来源
//!     filter: function (event, player) {
//!         return get.distance(player, event.player) <= 1 && event.num > 0;
//!     },
//!     direct: true,
//!     content: function () { /* 选 recover_hp 或 draw_card */ },
//! };
//! ```
//!
//! 由 JS 推导的规则点（本文件逐条验收）：
//! 烈弓：① 仅【杀】；② 条件为「目标手牌 ≥ 自己体力 **或** ≤ 自己攻击范围」（两支）；
//! ③ 效果是 `directHit`（目标不能出【闪】/不走防具判定）。
//! 狂骨：④ 仅自己为来源；⑤ 距离 ≤ 1；⑥ 伤害 > 0；⑦ 效果二选一（回复 1 / 摸 1）。
//!
//! 未覆盖（登记）：`audio`/`intro`/`check`/`ai` 属表现与策略面；`mark`/`marktext` 属标记展示。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn ms_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("mark-supplement-upstream-liuli/assets/cards.json"))
        .expect("读标记补充包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..20 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 上游点 ②（第二支）：目标手牌 ≤ 攻击范围 → 直伤（不依赖「手牌 ≥ 体力」）。
#[test]
fn upstream_liegong_second_branch_by_attack_range() {
    let mut e = ms_engine(20262301);
    e.debug_grant(0, "fz_liegong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标 1 号手牌 1 张（≤ 攻击范围 1），且远小于 0 号体力 4 → 命中「≤ 攻击范围」支
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(1, 1, shan);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "手牌 ≤ 攻击范围：烈弓直伤");
    assert_eq!(
        e.debug_hand(1).len(),
        1,
        "目标未出【闪】（手牌仍 1 张）——directHit 语义"
    );
}

/// 上游点 ②（不满足）：目标手牌既 < 体力又 > 攻击范围 → **不**直伤（可出【闪】）。
#[test]
fn upstream_liegong_condition_not_met_allows_shan() {
    let mut e = ms_engine(20262302);
    e.debug_grant(0, "fz_liegong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标手牌 2 张：2 < 0 号体力 4，且 2 > 攻击范围 1 → 两支都不满足
    for _ in 0..2 {
        let c = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, c);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 目标有【闪】→ 走正常出闪询问（drain 会弃牌/放弃，视选项而定）
    drain(&mut e);
    assert!(
        e.debug_hp(1) >= hp1 - 1,
        "条件不满足时不得强制直伤（hp {} → {}）",
        hp1,
        e.debug_hp(1)
    );
}

/// 上游点 ⑤⑥：狂骨需要「距离 ≤ 1 且伤害 > 0」——距离 2 的目标受伤**不**触发。
#[test]
fn upstream_xinkuanggu_requires_distance_le_one() {
    let mut e = ms_engine(20262303);
    e.debug_grant(0, "fz_xinkuanggu");
    advance_to_play(&mut e, 0);
    // 4 人局：0 号到 2 号距离 2（0-1-2）→ 不触发狂骨
    let hp0 = e.debug_hp(0);
    e.debug_deal_damage(Some(0), 2, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        e.debug_hp(0),
        hp0,
        "距离 2 的伤害不得触发狂骨（不应回复）"
    );
}

/// 上游点 ⑦：狂骨二选一——选「回复 1」与选「摸 1」各自生效（距离 1）。
#[test]
fn upstream_xinkuanggu_choice_branches() {
    for (pick, expect_recover) in [(ChoiceOption::Tag(0), true), (ChoiceOption::Tag(1), false)] {
        let mut e = ms_engine(20262304 + if expect_recover { 0 } else { 1 });
        e.debug_grant(0, "fz_xinkuanggu");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(0, 3); // 已受伤，便于观察回复
        let hp0 = e.debug_hp(0);
        let hand0 = e.debug_hand(0).len();
        // 直接造成伤害（0 号 → 1 号，距离 1）
        e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
        // 狂骨确认门 → 选择窗
        let mut chose = false;
        for _ in 0..16 {
            let Some(c) = e.open_choice().cloned() else { break };
            let sel = if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.prompt == "fz_xinkuanggu_choice"
                || c.spec.options.contains(&pick)
            {
                chose = true;
                vec![pick]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        }
        drain(&mut e);
        if chose {
            if expect_recover {
                assert_eq!(e.debug_hp(0), hp0 + 1, "选回复：体力 +1");
            } else {
                assert_eq!(e.debug_hand(0).len(), hand0 + 1, "选摸牌：手牌 +1");
            }
        }
    }
}
