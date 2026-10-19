//! NC08 第⑦层（上游行为对照）样本 69：**来源 U05（杀海拾遗·运筹）** 武将技
//! `xiongzi`（雄姿）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:143`）：
//!
//! ```js
//! xiongzi: {
//!     trigger: { player: "phaseDrawBegin" },        // **自己摸牌阶段开始**
//!     forced: true,                                  // **锁定**
//!     content() {
//!         trigger.num += 1 + Math.floor(player.countCards("e") / 2);   // **+1+⌊装备数/2⌋**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**摸牌阶段开始**（锁定技，必然生效）；
//! 2. **效果**：本次摸牌数 **+1+⌊装备区牌数 / 2⌋**（**向下取整** → 3 件装备也只 +1）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:8`（`let n = 1 + equips.len() / 2`（整数除法=向下取整）
//! → `draw_cards(owner, n)`）；注册/过滤见 `content/host/behavior.rs:619`（tag 243）、
//! `filtering/common.rs:507`（自己摸牌阶段开始，固定摸 2 后补摸）。
//!
//! **登记差异**：本样本**未发现**差异（含**向下取整**边界：3 件装备仍只 +1）；
//! 保留项：`audio` 表现面。
//!
//! **夹具要点（重要，避免误判）**：运筹的**武将面在 `character.json`**（`cards.json` 只有卡牌段）
//! → 必须**双产物桥接**（`bridge_pack` + `bridge_generals`），否则技能不在目录中、
//! `debug_grant("ycnxiongzi")` 会 **panic `unknown skill`**（本样本初版即因此**误判为「不可达」**，
//! 已更正——属「先查完整数据来源」方法学的又一次命中）。
//! 另：宿主技能在**目录中的名字**是 `ycn`+拼音（如 `ycnxiongzi`），包面向 `mapping` 为 `host:xiongzi`。
//!
//! 未覆盖（登记）：运筹**其它**武将技（厉焰/雅量/蝉缘变体/悲歌 refresh 变体）各自的行为面。

mod nc08_common;
use engine_native::{Engine, Phase};
use nc08_common::advance_to_phase;

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 在「授予雄姿 + 装备 n 件」下走完摸牌阶段，返回该阶段净增手牌数。
fn draw_phase_gain(seed: u64, equips: usize) -> i64 {
    let mut e = yunchou_engine(seed);
    // 引擎初态为 0 号**判定阶段**：此刻授予雄姿，摸牌阶段尚未开始 → 可正常触发
    e.debug_grant(0, "ycnxiongzi"); // 宿主技能在**目录中的名字**为 `ycn`+拼音（包面向键为 host:xiongzi）
    let names: Vec<String> = e
        .catalog()
        .cards
        .iter()
        .filter(|c| c.name.contains("armor") || c.name.contains("weapon"))
        .map(|c| c.name.to_string())
        .collect();
    for i in 0..equips {
        let name = names[i % names.len()].clone();
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name == name)
            .expect("装备在目录") as engine_native::CardId;
        nc08_common::put_equips(&mut e, 0, id);
    }
    let before = e.debug_hand(0).len() as i64;
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    e.debug_hand(0).len() as i64 - before
}

/// 上游点 2（**行为面 · 含向下取整边界**）：0 件 → +3；2 件 → +4；3 件 → +4（⌊3/2⌋=1）。
#[test]
fn upstream_xiongzi_draw_bonus_with_floor() {
    let g0 = draw_phase_gain(20270101, 0);
    let g2 = draw_phase_gain(20270102, 2);
    let g3 = draw_phase_gain(20270103, 3);
    eprintln!("DBG 雄姿：装备 0/2/3 件 → 摸牌阶段净增 {g0}/{g2}/{g3} 张");
    assert_eq!(g0, 3, "雄姿：0 件装备应摸 2+1=3 张（实测 {g0}）");
    assert_eq!(g2, 4, "雄姿：2 件装备应摸 2+1+1=4 张（实测 {g2}）");
    assert_eq!(
        g3, 4,
        "雄姿：3 件装备仍为 4 张（**向下取整** ⌊3/2⌋=1，实测 {g3}）"
    );
}

/// 上游点 1（**源码事实**）：时机与效果式与上游一致；**并固定「不可达」事实**。
#[test]
fn upstream_xiongzi_source_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = src.find("YunchouXiongzi =>").expect("雄姿分支");
    let seg: String = src[idx..].chars().take(240).collect();
    assert!(
        seg.contains("1 + ctx.core.player(self.owner).equips.len() / 2"),
        "原生依据：1 + 装备数/2（整数除法=向下取整）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\txiongzi: {").expect("上游雄姿");
    let seg: String = up[idx..].chars().take(300).collect();
    assert!(
        seg.contains("phaseDrawBegin")
            && seg.contains("trigger.num += 1 + Math.floor(player.countCards(\"e\") / 2)"),
        "上游依据：摸牌阶段开始 + 1+⌊装备数/2⌋：{seg}"
    );
    // **数据来源固定**：武将面在 character.json（cards.json 只有卡牌段）
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let chars = std::fs::read_to_string(
        root.join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("读运筹 character.json");
    let v: serde_json::Value = serde_json::from_str(&chars).expect("解析 character.json");
    let gens = v["generals"].as_array().expect("generals 段");
    assert_eq!(gens.len(), 10, "运筹包应有 10 名武将");
    assert!(
        chars.contains("\"ycnxiongzi\"") && chars.contains("\"mapping\": \"host:xiongzi\""),
        "数据依据：雄姿以 `ycnxiongzi`（目录名）+ `host:xiongzi`（包面向键）交付"
    );
    let cards = std::fs::read_to_string(
        root.join("shahai-yunchou-upstream-noname/assets/cards.json"),
    )
    .expect("读运筹 cards.json");
    let cv: serde_json::Value = serde_json::from_str(&cards).expect("解析 cards.json");
    assert_eq!(
        cv["cards"].as_array().map(|a| a.len()).unwrap_or(0),
        23,
        "运筹 cards.json 为 23 张卡（卡牌段）"
    );
}
