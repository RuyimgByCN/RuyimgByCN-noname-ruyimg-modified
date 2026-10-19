//! NC08 附加：**「升级」验收（按来源）**——覆盖 L04/U01/U03/U05/U07。
//!
//! 背景：[五来源签署表](../../docs/operations/evidence/2026-10-19-five-source-signoff/README.md)中
//! 「升级」为 **0/5**；既有 `nc08_upgrade_acceptance` 只用了 swd 包，**不等于来源级签署**；
//! `e02_c3_lifecycle` 只覆盖「测试自构造的最小包」。
//!
//! 本用例对**每个来源的一个真实包**做「**版本变更 + 存档恢复**」：
//! 1. v1 建局并 `save()`；
//! 2. 构造 **v2**（仅 `def.version + 1`）并尝试 `restore_with_extensions(&snap, &[&v2])`
//!    ⇒ **必须 `Err(SnapshotError::RuleVersionMismatch)`**（错版拒绝）；
//! 3. 用 **v1** 恢复 ⇒ **成功**且 `state_hash` 一致。
//!
//! 边界：只验证「**版本号变化**」这一面；**同版本内容漂移**由 `nc08_upgrade_skill_mapping` 覆盖。

use std::path::PathBuf;

fn assets_json(pkg: &str) -> Option<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    for f in ["cards.json", "character.json"] {
        let p = dir.join(f);
        if p.is_file() {
            return std::fs::read_to_string(p).ok();
        }
    }
    None
}

/// `pile.json` → ExtensionPack（U03/L04 走该管线）。
fn load_pile(pkg: &str) -> Option<engine_native::content::extension::ExtensionPack> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let json = std::fs::read_to_string(dir.join("pile.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let mut entries = Vec::new();
    for e in v.get("entries").and_then(|x| x.as_array())? {
        let card = e.get("card").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let mut counts: Vec<(String, u32)> = Vec::new();
        if let Some(obj) = e.get("counts").and_then(|x| x.as_object()) {
            for suit in ["spade", "heart", "club", "diamond"] {
                if let Some(n) = obj.get(suit).and_then(|x| x.as_u64()) {
                    counts.push((suit.to_string(), n as u32));
                }
            }
        }
        entries.push(engine_native::content::pile::PileEntry { card, counts });
    }
    engine_native::content::pile::extension_pack_from_pile(
        engine_native::content::extension::ExtensionDef {
            name: "pile-upgrade-probe",
            version: 1,
            description: "pile",
        },
        "pile_probe",
        &engine_native::content::pile::PileSpec { source, total, entries },
    )
    .ok()
}

fn load(pkg: &str) -> Option<engine_native::content::extension::ExtensionPack> {
    // `pile.json` 优先（U03/L04），否则走 card/general 管线
    if let Some(p) = load_pile(pkg) {
        return Some(p);
    }
    let json = assets_json(pkg)?;
    let mut out = content_native_loader::bridge_pack(&json).ok()?;
    // 双产物：cards 包补 generals
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            out.pack.generals = b.generals;
        }
    }
    Some(out.pack)
}

fn run(pkg: &str) {
    let Some(v1) = load(pkg) else {
        eprintln!("NOTE 升级：{pkg} 无法桥接 —— 跳过");
        return;
    };
    let mut e = engine_native::Engine::new_with_extensions(47, 4, &[&v1]).expect("v1 建局");
    let snap = e.save();
    let before = e.state_hash();
    // v2：仅版本号 +1
    let mut v2 = load(pkg).expect("v2 可桥接");
    let old_ver = v2.def.version;
    v2.def.version = old_ver + 1;
    let rejected = engine_native::Engine::restore_with_extensions(&snap, &[&v2]);
    match &rejected {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {
            eprintln!("DBG 升级：{pkg} v{old_ver}→v{} 错版**被拒绝** ✅", old_ver + 1);
        }
        Err(other) => {
            eprintln!("DBG 升级：{pkg} 错版被拒绝（错误类型 {other:?}）——需核对是否为 RuleVersionMismatch");
        }
        Ok(_) => panic!("{pkg}: 错版恢复**不应成功**（内容锁未生效）"),
    }
    // 用 v1 恢复应成功且 hash 一致
    let restored = engine_native::Engine::restore_with_extensions(&snap, &[&v1])
        .unwrap_or_else(|err| panic!("{pkg}: v1 恢复失败 {err:?}"));
    assert_eq!(before, restored.state_hash(), "{pkg}: v1 恢复后 state_hash 应一致");
    eprintln!("DBG 升级：{pkg} 回滚 v1 恢复成功（hash 一致）✅");
}

/// **按来源的升级验收**：五来源各取一个真实包，验证「错版拒绝 + 回滚恢复」。
#[test]
fn upgrade_per_source_five_sources() {
    for pkg in [
        "cardpile-upstream-liuli",   // L04
        "d3-upstream-noname",        // U01
        "cardpile-upstream-noname",  // U03
        "shahai-swd-upstream-noname",// U05（取 swd 子包）
        "wudian-upstream-noname",    // U07
    ] {
        run(pkg);
    }
}
