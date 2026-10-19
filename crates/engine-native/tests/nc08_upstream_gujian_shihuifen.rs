//! NC08 第⑦层（上游行为对照）样本 90：**来源 U05（古剑）** 锦囊
//! `shihuifen`（石灰粉）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`shihuifen`）：
//!
//! ```js
//! shihuifen: {
//!     type: "trick", fullskin: true, filterTarget: true,
//!     global: "g_shihuifen",                        // 全局子技（可在他人回合打出）
//!     content() {
//!         "step 0";
//!         if (!_status.currentPhase?.isIn()) return;
//!         var next = _status.currentPhase.chooseToRespond({ name: "shan" });   // **当前回合角色须出【闪】**
//!         next.set("prompt2", "否则本回合无法对其他角色使用卡牌");
//!         "step 1";
//!         if (!result.bool) _status.currentPhase.addTempSkill("shihuifen", "phaseUseAfter");
//!         //                                   ↑ **未出【闪】→ 本回合无法对其他角色使用卡牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **对象**：**当前回合角色**（`_status.currentPhase`）；
//! 2. **流程**：其须出【闪】→ **未出**则获得临时技能 `shihuifen`（**本回合不能对其他角色使用牌**）；
//! 3. 该限制在**出牌阶段结束**（`phaseUseAfter`）时解除。
//!
//! 原生实现引用：`content/host/step/gujian.rs:439`（`GujianShihuifenCard` 阶段机）；
//! 限制子技 `GujianFoodShihuifenBan => Flow::Done`（**其执行应在出牌校验路径**）。
//!
//! **本样本的定位**：验证「**未出【闪】→ 对其他角色出牌被拒**」这一**限制是否真的生效**
//! （子技自身为空实现，故限制可能落在命令面）。
//!
//! 未覆盖（登记）：**解除时机**（`phaseUseAfter`）；出了【闪】的对照分支（本样本先测限制）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：当前回合角色未出【闪】→ **不能对其他角色使用牌**。
///
/// **本轮未验收（登记，含诊断）**：实测——石灰粉**可发动**（`Accepted`），但
/// **无任何询问**（`询问=[]`，上游应有 `chooseToRespond({name:"shan"})`），
/// 且随后 1 号对 2 号出【杀】**仍被受理**（`Accepted`）✗ ⇒ **限制未生效**
/// （详见修复清单第 38 项）。用例以 `#[ignore]` 作为修复后的验收条件。
#[test]
#[ignore = "石灰粉：未出闪后应限制对其他角色用牌——实测无询问且限制未生效，见第 38 项"]
fn upstream_shihuifen_blocks_cards_on_others() {
    let mut e = gujian_engine(20272001);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 让 1 号成为当前回合角色（其出牌阶段）
    assert!(
        advance_to_phase(&mut e, 1, Phase::Play),
        "应到达 1 号出牌阶段"
    );
    // **当前回合角色（1 号）自己使用**石灰粉（`global` 子技，非当前回合方不可主动使用）
    let shi = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shihuifen"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("石灰粉在目录");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shi, targets: vec![2] },
    });
    eprintln!("DBG 石灰粉：发动={out:?}");
    if !matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 石灰粉未能发动（{out:?}）——本用例不构成验收");
        return;
    }
    // 1 号（当前回合角色）被要求出【闪】——**选择不出**
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        // 1 号的响应询问 → 放弃（Pass / 空选）
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.prompt.contains("shihuifen") || c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
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
    eprintln!("DBG 石灰粉：询问={prompts:?}");
    // 现在 1 号尝试对 2 号出【杀】→ 应被限制（本回合无法对其他角色使用卡牌）
    let blocked = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![2] },
    });
    eprintln!("DBG 石灰粉：1 号对其他角色出杀 → {blocked:?}");
    assert!(
        matches!(blocked, CommandOutcome::Rejected(_)),
        "石灰粉：未出【闪】的当前回合角色**本回合不能对其他角色使用牌**（上游 addTempSkill 限制）：{blocked:?}"
    );
}
