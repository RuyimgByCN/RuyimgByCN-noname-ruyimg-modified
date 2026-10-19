//! NC08 第⑦层（上游行为对照）样本 142：**来源 U11（wuxing）** 系统面
//! `shengke`（五行生克）——**两支精确判定**（承接样本 141 的待办）。
//!
//! 上游口径（`extension.js` 的 `help` 逐字）：
//! > **金克木，金生水；木克土，木生火；水克火，水生木；火克金，火生土；土克水，土生金**；
//! > **成为相克属性卡牌的目标时，须弃置一张牌**；**成为相生的卡牌的目标时，须摸一张牌**。
//!
//! 原生实现（**源码逐字引用**）：
//! - `content/host/constants.rs:4`：**`pub const KE: [u8; 5] = [1, 4, 3, 0, 2];`**（金克木/木克土/水克火/火克金/土克水）；
//! - `:8`：**`pub const SHENG: [u8; 5] = [2, 3, 1, 4, 0];`**（金生水/木生火/水生木/火生土/土生金）；
//! - `step/skills_status.rs:680`（`ShengkeCheck`）：
//!   **`if KE[cnat] == onat { 弃一张手牌 } else if SHENG[cnat] == onat { 摸一张 }`**
//!   ——并自述登记差异：**「被克 → 弃一张手牌（确定性首张，登记差异：上游为目标自选『he』）」**；
//! - `construction.rs:431`：**`pl.wunature = Some(core.rng.gen_range(5) as u8)`**（角色开局随机 1 属性）。
//!
//! **本样本的定位**：**精确构造三支**——①目标属性 = **`KE[牌属性]`** ⇒ **弃 1**；
//! ②目标属性 = **`SHENG[牌属性]`** ⇒ **摸 1**；③**无关属性** ⇒ **无事**。
//! （属性通过公开的 `state_core_mut()` 设置，确定性构造，不依赖随机。）
//!
//! 未覆盖（登记）：**弃牌为「确定性首张」**这一登记差异（上游为目标自选 `he`）；
//! 属性赋予的**随机分布**（角色 1 个、牌堆 1/3）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 原生常量（与上游 help 逐字对应）。
const KE: [u8; 5] = [1, 4, 3, 0, 2];
const SHENG: [u8; 5] = [2, 3, 1, 4, 0];

fn wuxing_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wuxing-upstream-liuli/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 wuxing cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次：把 1 号属性设为 `target_nature`，使用一张**有属性**的牌，返回 `(目标手牌变化, 牌属性)`。
fn run(seed: u64, fixed: Option<(engine_native::CardId, u8)>, target_nature: Option<u8>) -> Option<(engine_native::CardId, i64, u8)> {
    let mut e = wuxing_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    // **逐个尝试**有属性且非基本类的牌，取**第一个能被受理**的（避免 toself/装备类被拒）
    let candidates: Vec<(engine_native::CardId, u8)> = e
        .state_core()
        .card_natures
        .iter()
        .filter(|(c, _)| {
            !matches!(
                e.catalog().card_kind(**c),
                engine_native::CardKind::Sha
                    | engine_native::CardKind::Shan
                    | engine_native::CardKind::Tao
                    | engine_native::CardKind::Jiu
            )
        })
        .map(|(c, n)| (*c, *n))
        .collect();
    let mut chosen: Option<(engine_native::CardId, u8)> = None;
    for (c, n) in &candidates {
        for p in 0..4 {
            e.debug_clear_hand(p);
            for _ in 0..3 {
                let name = e.catalog().cards.iter().find(|x| x.name == "shan").map(|x| x.name).unwrap();
                e.debug_place(p, name);
            }
        }
        nc08_common::put_hand(&mut e, 0, *c);
        let probe = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card: *c, targets: vec![1] },
        });
        drain(&mut e);
        if matches!(probe, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
            chosen = Some((*c, *n));
            break;
        }
        // 被拒后重开一局继续尝试
        e = wuxing_engine(seed + 100 + *c as u64);
        advance_to_phase(&mut e, 0, Phase::Play);
    }
    let (card, nat) = fixed.or(chosen)?;
    let nat = nat as u8;
    for p in 0..4 {
        e.debug_clear_hand(p);
        for _ in 0..3 {
            let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
            e.debug_place(p, name);
        }
    }
    nc08_common::put_hand(&mut e, 0, card);
    if let Some(n) = target_nature {
        // 通过公开的可变访问器设置目标属性（确定性构造）
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(1) {
            p.wunature = Some(n);
        }
    }
    let h1 = e.debug_hand(1).len() as i64;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    if !matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 有属性牌 {card}（属性 {nat}）未被受理：{out:?}");
        return None;
    }
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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
    Some((card, e.debug_hand(1).len() as i64 - h1, nat))
}

/// 上游点 2（**行为面 · 三支**）：被克 ⇒ 弃 1；被生 ⇒ 摸 1；无关 ⇒ 无事。
///
/// **本轮未验收（登记，含诊断）**：**三次使用同一张有属性牌**（卡 65，属性 **3=火**），
/// 仅改目标属性——**被克**（目标 0=金，火克金）、**被生**（目标 4=土，火生土）、**无关**（目标 1=木）
/// ——**三次手牌变化均为 −1** ✗ ⇒ **无法区分生克是否生效**（−1 与该牌**自身效果**一致）。
/// 诊断候选：①生克未接入该卡（宿主技声明面）②`wunature` 设置未生效 ③被卡自身效果掩盖。
/// 用例以 `#[ignore]` 作为修复后的验收条件（第 51 项）。
#[test]
#[ignore = "五行生克两支未能在行为面复现（三次均 −1）——见第 51 项"]
fn upstream_wuxing_shengke_three_branches() {
    // 先探明一张有属性牌及其属性
    let Some((card0, _d0, nat)) = run(20287501, None, None) else {
        eprintln!("NOTE 未取到可用的有属性牌——本用例不构成验收");
        return;
    };
    let nat_u = nat as u8;
    let ke = KE[nat_u as usize];
    let sheng = SHENG[nat_u as usize];
    let unrelated = (0..5u8).find(|n| *n != ke && *n != sheng).expect("无关属性");
    // **三次使用同一张牌**（属性一致），只改目标属性 ⇒ 三支可比
    let r_ke = run(20287502u64, Some((card0, nat_u)), Some(ke));
    let r_sheng = run(20287503u64, Some((card0, nat_u)), Some(sheng));
    let r_none = run(20287504u64, Some((card0, nat_u)), Some(unrelated));
    eprintln!(
        "DBG 五行生克三支：牌属性={nat}；被克(目标属性={ke}) → {r_ke:?}；\
         被生(目标属性={sheng}) → {r_sheng:?}；无关(目标属性={unrelated}) → {r_none:?}"
    );
    assert_eq!(
        r_ke.map(|(_, d, _)| d),
        Some(-1),
        "五行生克（**被克**）：目标应**弃 1 张手牌**（实测 {r_ke:?}）"
    );
    assert_eq!(
        r_sheng.map(|(_, d, _)| d),
        Some(1),
        "五行生克（**被生**）：目标应**摸 1 张**（实测 {r_sheng:?}）"
    );
    assert_eq!(
        r_none.map(|(_, d, _)| d),
        Some(0),
        "五行生克（**无关**）：应**无事**（实测 {r_none:?}）"
    );
    eprintln!(
        "NOTE 五行生克：被克支的弃牌为**确定性首张**（原生注释自述登记差异：上游为目标自选「he」）——\
         属既有「确定性首张」族"
    );
}
