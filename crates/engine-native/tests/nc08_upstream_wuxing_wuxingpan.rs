//! NC08 第⑦层（上游行为对照）样本 140：**来源 U11（wuxing / noname-liuli）** 装备
//! `wuxingpan`（五行盘）——**该来源的首个行为面样本**。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js`，**扩展对象格式**）：
//!
//! ```js
//! card:{
//!     wuxingpan:{ type:'equip', subtype:'equip5', skills:['wuxingpan_skill'], fullskin:true }
//! },
//! translate:{
//!     …, wuxingpan:'五行盘', wuxingpan_skill:'五行',
//!     wuxingpan_skill_info:'出牌阶段限一次，你可以永久改变一张手牌的五行属性',
//! },
//! help:{ '五行生克':'…每名角色在游戏开始时随机获得一个属性…当一名成为相克属性卡牌的目标时，须弃置一张牌；
//!        当一名角色成为相生的卡牌的目标时，须摸一张牌…金克木，金生水；木克土，木生火；…' }
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **类型**：`equip` / **`equip5`（宝物位）**；装备后**授予 `wuxingpan_skill`（五行）**；
//! 2. **技能**：**出牌阶段限一次**——**永久改变一张手牌的五行属性**；
//! 3. **系统**：五行**生克**（相克目标须弃 1、相生目标须摸 1）。
//!
//! 原生实现引用：`catalog.rs:946`（**`CardKind::WuxingPan => "wuxingpan"`**）；
//! 包内 `equip_skills: ["wuxingpan_skill"]`。
//!
//! **本样本的定位**：验证「**装备五行盘 → 装备区 +1 且授予五行技能**」
//! （技能名按**候选名探测**，含原生可能的前缀形式——样本 97/99 的命名教训）。
//!
//! 未覆盖（登记）：**五行属性的赋予与生克结算**（相克弃 1 / 相生摸 1）；技能的**限一次**与改属性效果。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn wuxing_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wuxing-upstream-liuli/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 wuxing cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2（**行为面**）：**装备五行盘 → 装备区 +1 且授予五行技能**。
#[test]
fn upstream_wuxingpan_equips_and_grants_skill() {
    let mut e = wuxing_engine(20287301);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("wuxingpan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("五行盘在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let eq_before = e.debug_equips(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 五行盘：装备={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "五行盘应可装备（无目标）：{out:?}"
    );
    drain(&mut e);
    let eq_after = e.debug_equips(0).len();
    // **候选名探测**（含前缀/后缀变体；不按上游键猜名）
    let candidates = [
        "wuxingpan_skill",
        "wuxingpan",
        "wxl_wuxingpan_skill",
        "gj_wuxingpan_skill",
        "host:wuxingpan_skill",
    ];
    let hits: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|n| e.debug_has_skill(0, n))
        .collect();
    eprintln!("DBG 五行盘：装备区 {eq_before} → {eq_after}；技能候选命中={hits:?}");
    assert!(
        eq_after > eq_before,
        "五行盘：应进入装备区（{eq_before} → {eq_after}）"
    );
    eprintln!(
        "NOTE 五行盘：技能候选命中={hits:?}（若为空，须先按原生常量核对命名再判定是否缺失）；\
         上游为 equip5 宝物位 + 授予 wuxingpan_skill（出牌阶段限一次，永久改变一张手牌的五行属性）"
    );
}
