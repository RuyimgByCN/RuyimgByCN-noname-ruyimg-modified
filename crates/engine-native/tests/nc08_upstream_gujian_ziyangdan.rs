//! NC08 第⑦层（上游行为对照）样本 95：**来源 U05（古剑）** 基本牌
//! `ziyangdan`（紫阳丹）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`ziyangdan`）：
//!
//! ```js
//! ziyangdan: {
//!     type: "basic", enable: true, filterTarget: true,     // 任意角色（含自己）
//!     content() {
//!         target.changeHujia(3);                            // **+3 护甲**
//!         if (target.hasSkill("ziyangdan")) target.storage.ziyangdan += 3;
//!         else target.addSkill("ziyangdan");                // 计时子技（累计）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果**：目标**获得 3 点护甲**；并授予/累计计时子技（`storage` 累加）。
//!
//! 原生实现引用：`content/host/step/gujian.rs:219`（`CardKind::Ziyangdan => (GJ_FOOD_ZIYANGDAN_KEY, …)`）、
//! `:424`（`GujianFoodZiyangdan` 分支）。
//!
//! **本样本的定位**：验证「**护甲 +3**」这一**即时可见**的效果是否执行
//! （对照：同批食物牌的「本体归属」缺失——本卡属**另一机制**，故单独抽样）。
//!
//! 未覆盖（登记）：`storage` 累计与计时子技的逐轮结算；`ai.result` 策略面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：目标**获得 3 点护甲**。
#[test]
fn upstream_ziyangdan_grants_hujia() {
    let mut e = gujian_engine(20272501);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("ziyangdan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("紫阳丹在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hujia_before = e.state_core().player(1).hujia;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 紫阳丹：发动={out:?}（目标护甲前值={hujia_before}）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "紫阳丹应可发动：{out:?}"
    );
    drain(&mut e);
    let p1 = e.state_core().player(1);
    eprintln!(
        "DBG 紫阳丹：1 号护甲 {hujia_before} → {}（是否获得子技 gj_food_ziyangdan={}）",
        p1.hujia,
        e.debug_has_skill(1, "gj_food_ziyangdan")
    );
    assert_eq!(
        p1.hujia,
        hujia_before + 3,
        "紫阳丹：目标应**获得 3 点护甲**（上游 `target.changeHujia(3)`；{hujia_before} → {}）",
        p1.hujia
    );
}
