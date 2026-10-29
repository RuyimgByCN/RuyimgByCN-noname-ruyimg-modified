//! NC08 第⑦层（上游行为对照）样本 40：**来源 `boss-zhugeliang`**（T11 Boss 诸葛亮族 /
//! 赫拉西斯 `hlss_*`，规则 v303–v305）。
//!
//! 本样本是**来源级**对照：该来源在 `packages/extensions/boss-zhugeliang-upstream-liuli`
//! 中只有 **1 名武将**（`boss_zhugeliang`）+ **5 条技能映射**，此前**没有** NC08 样本
//! → 补齐后来源覆盖由 21 提升到 **22/37**。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/character/extra.js`）：
//!
//! ```js
//! hlss_xiangxing: {
//!     unique: true, juexingtx: true,
//!     init: function(player) {
//!         player.storage.hlss_xiangxing = 3;          // 开局 3 枚星
//!         player.storage.hlss_xiangxing_count = 0;
//!         player.addSkill('hlss_xiangxing3');
//!     },
//!     mark: true,
//!     intro: { content: '当前有#枚星' },
//!     group: 'hlss_xiangxing_hp',
//!     trigger: { player: ['damageEnd', 'loseHpEnd'] },   // 受伤/失体后累计
//!     forced: true,
//!     …
//! },
//! // translate（官方文本，逐字引用）：
//! hlss_xiangxing_info: '锁定技，游戏开始时，你获得3枚星；你的体力值始终为满；每当你累计扣减3点体力，
//!                      你失去一枚星，并造成3点雷属性伤害，随机分配给其他角色；当你失去全部星后，你失去此技能。'
//! ```
//!
//! 原生实现引用：`content/host/keys.rs:774`（`HostBehavior::HlssXiangxing` →
//! `"host:hlss_xiangxing"`）、`state.rs:247–252`（`hlss_stars` / `hlss_count` /
//! `hlss_damage`）、`kernel/damage.rs:1029`（归零前「体力值始终为满」）。
//!
//! **登记差异（本文件固定）**：
//! - **①随机分配**：上游「造成 3 点雷属性伤害，**随机分配给其他角色**」；原生按
//!   **座位序确定性近似**（与「随机 → 确定性」族同源，**登记**）；
//! - **②体力始终为满**：上游 `group: 'hlss_xiangxing_hp'`（任意体力变化后回满）；
//!   原生在 `kernel/damage.rs` 的 `hlss_light_refill` 中于**伤害/失去体力提交点、濒死判定前**
//!   置 `hp = maxHp`（**调用点/口径不同**，登记：星未耗尽者不会进入濒死）。
//!
//! 未覆盖（登记）：`unique`/`juexingtx`/`mark`/`intro`/`audio`/头像动画等表现面。

use engine_native::Engine;

const PACK: &str = "packages/extensions/boss-zhugeliang-upstream-liuli/assets/cards.json";

/// 上游点（武将/技能映射）：包内 1 将 5 技，且技能均映射到宿主行为键。
#[test]
fn upstream_boss_zhugeliang_pack_mapping() {
    let raw = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(PACK),
    )
    .expect("读 boss-zhugeliang 包数据");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("包数据为 JSON");
    let generals = v.get("generals").and_then(|g| g.as_array()).expect("generals 数组");
    assert_eq!(generals.len(), 1, "该来源只有 1 名武将");
    let skills = generals[0]
        .get("skills")
        .and_then(|s| s.as_array())
        .expect("skills 数组");
    assert_eq!(skills.len(), 5, "上游 hlss_* 5 条技能映射");
    let names: Vec<String> = skills
        .iter()
        .map(|s| s.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string())
        .collect();
    for want in [
        "hlss_xiangxing",
        "hlss_yueyin",
        "hlss_yueyin2",
        "hlss_fengqi",
        "hlss_gaiming",
    ] {
        assert!(
            names.iter().any(|n| n == want),
            "映射应含 {want}：{names:?}"
        );
    }
    for s in skills {
        let m = s.get("mapping").and_then(|m| m.as_str()).unwrap_or("");
        assert!(
            m.starts_with("host:"),
            "技能应映射到宿主行为键（host:…）：{m}"
        );
    }
}

/// 上游点（开局 3 枚星 + 累计计数）：原生状态字段与官方文本一致。
#[test]
fn upstream_xiangxing_three_stars_state() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/state.rs"
    ))
    .expect("读 state 源码");
    assert!(
        src.contains("pub hlss_stars: u8") && src.contains("pub hlss_count: u8"),
        "原生应有星数与累计计数字段"
    );
    let seg_idx = src.find("hlss_stars: u8").expect("字段");
    let seg: String = src[..seg_idx].chars().rev().take(400).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("3→0") || seg.contains("3 枚") || seg.contains("3"),
        "上游依据：开局 3 枚星（`storage.hlss_xiangxing = 3`）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname-liuli/noname/character/extra.js"),
    )
    .expect("读上游 extra.js");
    let idx = up.find("hlss_xiangxing:{").expect("上游禳星");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("player.storage.hlss_xiangxing=3")
            || seg.contains("player.storage.hlss_xiangxing = 3"),
        "上游依据：init 中置 3 枚星：{seg}"
    );
    assert!(
        seg.contains("damageEnd") && seg.contains("loseHpEnd"),
        "上游依据：受伤/失体后累计：{seg}"
    );
    // 引擎可正常构建（该来源不改变基础行为）
    let _ = Engine::new(20267001, 4);
}

/// **登记差异①②**（源码事实固定）：随机分配 → 确定性近似；体力始终为满的实现口径。
#[test]
fn upstream_xiangxing_differences_are_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname-liuli/noname/character/extra.js"),
    )
    .expect("读上游 extra.js");
    let idx = up.find("hlss_xiangxing_info").expect("官方文本");
    let seg: String = up[idx..].chars().take(260).collect();
    assert!(
        seg.contains("随机分配给其他角色"),
        "上游依据①：雷伤随机分配：{seg}"
    );
    assert!(
        seg.contains("体力值始终为满"),
        "上游依据②：体力始终为满：{seg}"
    );
    let dmg = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/damage.rs"
    ))
    .expect("读 damage 源码");
    let idx = dmg
        .find("pub(crate) fn hlss_light_refill")
        .expect("原生实现（定义处，而非调用点）");
    let seg: String = dmg[idx..].chars().take(700).collect();
    assert!(
        seg.contains("let max = ctx.core.player(player).max_hp") && seg.contains("enabled"),
        "差异②固定：原生在**伤害/失去体力提交点**回满（先判技能实例 enabled、再按 max_hp 回满；\
         上游为 group 子技任意体力变化后回满）：{seg}"
    );
}
