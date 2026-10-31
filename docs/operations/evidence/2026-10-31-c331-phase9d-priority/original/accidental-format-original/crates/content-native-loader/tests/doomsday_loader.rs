//! L19 末日乱斗装载器：身份链、灾变摸牌、无尽模式配置。

use content_native_loader::{
    identity_chain, parse_wujinmoshi, zaibian_draw, WujinMoshiConfig, IDENTITY_NAMES,
};
use std::collections::BTreeMap;

#[test]
fn identity_chain_follows_upstream_gates() {
    // 首项无门；其后按 players.length > gate 续链
    // 上游门为 `players.length > N`（N 见 IDENTITY_GATES）：3 人只分 er；
    // 4 人分 er/san；5 人加 si；6 人加 wu1
    assert_eq!(identity_chain(2), vec!["er"]);
    assert_eq!(identity_chain(3), vec!["er"]);
    assert_eq!(identity_chain(4), vec!["er", "san"]);
    assert_eq!(identity_chain(5), vec!["er", "san", "si"]);
    assert_eq!(identity_chain(6), vec!["er", "san", "si", "wu1"]);
    // 15 项链：16 人通过 er..shiwu（门 >15）→ 14 项；17 人再过 shiliu（门 >16）→ 15 项
    assert_eq!(identity_chain(16).len(), 14, "16 人 → 14 项");
    assert_eq!(identity_chain(17).len(), 15, "17 人 → 全链 15 项");
    assert_eq!(IDENTITY_NAMES.len(), 15, "上游链共 15 项（er..shiliu）");
    assert_eq!(IDENTITY_NAMES[0], "er");
    assert_eq!(
        IDENTITY_NAMES[3], "wu1",
        "上游第 4 项为 wu1（跳过 si 命名差异保留）"
    );
}

#[test]
fn zaibian_draw_counts_identities() {
    // 忠 2 反 1 内 0 → 2+2-1-0 = 3
    assert_eq!(zaibian_draw(2, 1, 0), 3);
    // 忠 0 反 3 内 1 → 0+2-3-1 = -2 → 0
    assert_eq!(zaibian_draw(0, 3, 1), 0);
    assert_eq!(zaibian_draw(1, 1, 1), 1);
    assert_eq!(zaibian_draw(4, 0, 0), 6);
}

#[test]
fn wujinmoshi_bonuses_apply_to_human_seat_only() {
    let mut values = BTreeMap::new();
    values.insert("wujinmoshiDraw".to_string(), 2);
    values.insert("wujinmoshiDamage".to_string(), 1);
    values.insert("wujinmoshiRecover".to_string(), 3);
    values.insert("wujinmoshiMaxHandCard".to_string(), 4);
    values.insert("wujinmoshiDamageI".to_string(), 2);
    values.insert("wujinmoshiDXB".to_string(), 45);
    values.insert("humanSeat".to_string(), 0);
    let cfg = parse_wujinmoshi(&values);
    assert!(cfg.applies_to(0) && !cfg.applies_to(1));
    assert_eq!(cfg.draw_bonus(0), 2);
    assert_eq!(cfg.draw_bonus(1), 0, "非人类位无加成");
    assert_eq!(cfg.damage_bonus(0), 1);
    assert_eq!(cfg.recover_bonus(0), 3);
    assert_eq!(cfg.hand_card_bonus(0), 4);
    // floor(45/15) - 2 = 1
    assert_eq!(cfg.non_human_damage_delta(), 1);
    let negative = WujinMoshiConfig {
        damage_divisor: 10,
        damage_taken: 3,
        ..Default::default()
    };
    assert_eq!(
        negative.non_human_damage_delta(),
        -3,
        "floor(10/15)=0 → 0-3"
    );
}
