//! NC08 附加：**牌归属分区不变式**（U01 差异 #6 的恢复面验证，2026-10-19）。
//!
//! 背景：U01（末焱）的差异 #6 是「空牌库与空弃牌堆的负例把多余的牌挪到 `core.table`，
//! **只为让恢复时每张牌仍有归属**」。据此，一个**可验证的通用不变式**是：
//!
//! > **每张牌在任意时刻恰好属于一个区**（手牌/装备/判定/牌堆/弃牌/桌面），
//! > 且**恢复前后归属不变**。
//!
//! 本用例对该不变式做**往返检查**（覆盖 `d3`（U01）与 `wudian`（U07）两包）。
//!
//! 边界：只检查「**唯一归属**」与「**往返一致**」，**不检查**归属**是否与原游戏语义相同**。

use std::collections::BTreeMap;
use std::path::PathBuf;

fn load(pkg: &str) -> engine_native::content::extension::ExtensionPack {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let json = std::fs::read_to_string(dir.join("cards.json"))
        .or_else(|_| std::fs::read_to_string(dir.join("character.json")))
        .expect("读资产");
    let mut out = content_native_loader::bridge_pack(&json).expect("桥接");
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            out.pack.generals = b.generals;
        }
    }
    out.pack
}

/// 统计每张牌的归属区（返回 `card -> zone` 映射）。
fn ownership(e: &engine_native::Engine) -> BTreeMap<u32, String> {
    let core = e.state_core();
    let mut m: BTreeMap<u32, String> = BTreeMap::new();
    let mut put = |card: u32, zone: String| {
        if let Some(prev) = m.insert(card, zone.clone()) {
            panic!("牌 {card} 同时属于 `{prev}` 与 `{zone}`（违反唯一归属）");
        }
    };
    for (i, p) in core.players.iter().enumerate() {
        for c in &p.hand {
            put(*c, format!("p{i}.hand"));
        }
        for c in &p.equips {
            put(*c, format!("p{i}.equips"));
        }
        for c in &p.judge {
            put(*c, format!("p{i}.judge"));
        }
    }
    for c in &core.deck {
        put(*c, "deck".into());
    }
    for c in &core.discard {
        put(*c, "discard".into());
    }
    for c in &core.table {
        put(*c, "table".into());
    }
    m
}

fn run(pkg: &str) {
    let pack = load(pkg);
    let mut e = engine_native::Engine::new_with_extensions(53, 4, &[&pack]).expect("建局");
    for _ in 0..6 {
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: e.revision() + 1,
            action: engine_native::GameAction::NextPhase,
        });
    }
    let before = ownership(&e);
    let total_before = before.len();
    let snap = e.save();
    let restored = engine_native::Engine::restore_with_extensions(&snap, &[&pack])
        .unwrap_or_else(|err| panic!("{pkg}: 恢复失败 {err:?}"));
    let after = ownership(&restored);
    eprintln!(
        "DBG 归属分区：{pkg} 牌数 {total_before}（有归属）→ {}（恢复后）；唯一归属 ✅",
        after.len()
    );
    assert_eq!(
        total_before,
        after.len(),
        "{pkg}: 恢复前后有归属的牌数应一致"
    );
    assert_eq!(before, after, "{pkg}: 恢复前后每张牌的归属区应完全一致");
}

/// **归属分区不变式**：每张牌唯一归属，且恢复前后一致。
#[test]
fn card_ownership_partition_roundtrip() {
    run("d3-upstream-noname");
    run("wudian-upstream-noname");
}
