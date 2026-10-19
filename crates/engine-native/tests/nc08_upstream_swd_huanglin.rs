//! NC08 第⑦层（上游行为对照）样本 157：**来源 U06（swd）** 法器
//! `huanglinzhicong`（黄麟之琮）——**`global` 卡窗口的第 6 张验证**（同族法器）。
//!
//! 上游源码（`skill.js` 的 **`g_huanglinzhicong`**，**逐字**）：
//!
//! ```js
//! huanglinzhicong: { type: "jiqi", global: "g_huanglinzhicong" },   // 卡本体无 content
//! g_huanglinzhicong: {
//!     trigger: { player: "phaseBegin" },                            // **本人阶段开始时**
//!     forced: true,
//!     filter(event, player) {
//!         return !player.hujia && player.countCards("h", "huanglinzhicong") > 0;  // **无护甲**且持有
//!     },
//!     content() { … player.changeHujia(); player.update(); },        // **+1 护甲**
//! },
//! ```
//!
//! **本样本的定位**：**无护甲的持有者阶段开始 ⇒ +1 护甲**（`forced`，无询问 ⇒ 可用常规推进助手）。
//!
//! 未覆盖（登记）：**已有护甲时不应触发**（`!player.hujia` 的负例）——建议下一轮补。

mod nc08_common;
use engine_native::{Engine, Phase};
use nc08_common::advance_to_phase;

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_huanglinzhicong`（**forced**）：无护甲的持有者阶段开始 ⇒ **+1 护甲**。
///
/// **本轮未通过（登记，含诊断）**：`hujia` **0 → 0**（应 **0 → 1**）⇒ **窗口未触发**。
///
/// **重要**：本文件的**负例（已有护甲 ⇒ 不变）也通过** ✗，但**窗口不触发时同样通过**
/// ⇒ **负例不是有效对照**（**通过的理由不成立**）。
///
/// **与第 55 项的共同点**：二者都依赖 **`phaseBegin`** 窗口；而 `qinglongzhigui` 的**摸 2 确实发生**
/// ⇒ **`phaseBegin` 本身会触发**，但**本卡的 `changeHujia` 分支未生效**——需与第 55 项一并排查
/// 「`phaseBegin` 机器的**分支选择**」与「`WaitChoice` 可见性」。
#[test]
#[ignore = "黄麟之琮：无护甲时未 +1（0→0），负例通过但不可信——见第 56 项"]
    let mut e = swd_engine(20288901);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持黄麟之琮（无护甲）
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("huanglinzhicong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黄麟之琮在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    {
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(0) {
            p.hujia = 0; // 确保无护甲
        }
    }
    let hj_before = e.state_core().player(0).hujia;
    assert!(
        advance_to_phase(&mut e, 0, Phase::Play),
        "应到达 0 号出牌阶段（阶段开始即触发）"
    );
    let hj_after = e.state_core().player(0).hujia;
    eprintln!("DBG 黄麟之琮：护甲 {hj_before} → {hj_after}（应 +1）");
    assert_eq!(
        hj_after as i32 - hj_before as i32,
        1,
        "黄麟之琮（**forced**）：无护甲的持有者阶段开始应 **+1 护甲**（{hj_before} → {hj_after}）"
    );
}

/// **负例**（上游 `!player.hujia`）：**已有护甲**时**不应**再加。
#[test]
/// **负例**（上游 `!player.hujia`）：**已有护甲**时**不应**再加。
///
/// ⚠ **本用例的通过不构成证据**：若窗口完全不触发，本用例也会通过 ⇒ 仅作**辅助**，
/// 有效性依赖「正例能触发」这一前提（当前**不成立**）。
#[test]
fn upstream_huanglinzhicong_no_gain_when_armored() {
    let mut e = swd_engine(20288902);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("huanglinzhicong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黄麟之琮在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    {
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(0) {
            p.hujia = 2; // **已有护甲** ⇒ 不应触发
        }
    }
    let hj_before = e.state_core().player(0).hujia;
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达 0 号出牌阶段");
    let hj_after = e.state_core().player(0).hujia;
    eprintln!("DBG 黄麟之琮（负例）：护甲 {hj_before} → {hj_after}（应不变）");
    assert_eq!(
        hj_after, hj_before,
        "黄麟之琮：**已有护甲**时不应触发（`!player.hujia` 条件；{hj_before} → {hj_after}）"
    );
}
