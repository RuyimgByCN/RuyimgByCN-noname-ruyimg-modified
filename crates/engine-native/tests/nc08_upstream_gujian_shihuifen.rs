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

/// 上游点 2（**行为面**）：目标未出【闪】→ **其回合内不能对其他角色使用牌**
///（Rule 52 登记差异：上游为 `g_shihuifen` 在出牌阶段开始时响应打出，原生为主动使用锦囊对目标施放，
/// 目标未出【闪】则获 `gj_food_shihuifen_ban`，在其回合内生效并于其回合结束清除）。
#[test]
fn upstream_shihuifen_blocks_cards_on_others() {
    let mut e = gujian_engine(20272001);
    assert!(
        advance_to_phase(&mut e, 0, Phase::Play),
        "应到达 0 号出牌阶段"
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号使用【石灰粉】指定 1 号；1 号持有一张【闪】但选择不出（Pass）
    let shi = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shihuifen"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("石灰粉在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let shan_name = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    e.debug_place(1, shan_name);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shi, targets: vec![1] },
    });
    eprintln!("DBG 石灰粉：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "石灰粉应可发动：{out:?}"
    );
    // 1 号被要求出【闪】——选择不出（Pass）
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
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
    assert!(
        prompts.iter().any(|p| p == "respond_shan@obs1"),
        "石灰粉：目标 1 号有【闪】时应出现 respond_shan 询问：{prompts:?}"
    );
    assert!(
        !e.debug_has_skill(0, "gj_food_shihuifen_ban"),
        "石灰粉：使用者 0 号本人不应被禁牌"
    );
    assert!(
        e.debug_has_skill(1, "gj_food_shihuifen_ban"),
        "石灰粉：未出【闪】的目标 1 号应获得 gj_food_shihuifen_ban"
    );
    // 推进到 1 号的出牌阶段，1 号尝试对 2 号出【杀】→ 应被限制
    assert!(
        advance_to_phase(&mut e, 1, Phase::Play),
        "应到达 1 号出牌阶段"
    );
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let blocked = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![2] },
    });
    eprintln!("DBG 石灰粉：1 号对其他角色出杀 → {blocked:?}");
    assert!(
        matches!(blocked, CommandOutcome::Rejected(_)),
        "石灰粉：未出【闪】的角色**本回合不能对其他角色使用牌**（上游 addTempSkill 限制）：{blocked:?}"
    );
}
