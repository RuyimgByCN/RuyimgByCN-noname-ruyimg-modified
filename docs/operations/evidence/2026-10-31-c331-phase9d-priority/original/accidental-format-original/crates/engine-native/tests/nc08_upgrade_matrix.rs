//! NC08：**逐包「升级」矩阵**——把第 433 轮的「五来源升级」扩到**全部可装载包**。
//!
//! 判据（每个包）：
//! 1. v1 建局并 `save()`；
//! 2. **v2（仅 `def.version + 1`）恢复必须被拒**（`SnapshotError::RuleVersionMismatch`）；
//! 3. **v1 回滚恢复成功**且 `state_hash` 与保存前一致。
//!
//! 覆盖的资产形态：`cards.json`（+ 同目录 `character.json`）、仅 `character.json`、`pile.json`。
//! 边界：只验**版本号变化**；同版本内容漂移由 `nc08_upgrade_skill_mapping` 覆盖。

use std::path::{Path, PathBuf};

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::content::pile::{extension_pack_from_pile, PileEntry, PileSpec};

fn assets(pkg: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets")
}

fn load_pile(dir: &Path) -> Option<ExtensionPack> {
    let json = std::fs::read_to_string(dir.join("pile.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    let source = v
        .get("source")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let mut entries = Vec::new();
    for e in v.get("entries").and_then(|x| x.as_array())? {
        let card = e
            .get("card")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let mut counts: Vec<(String, u32)> = Vec::new();
        if let Some(obj) = e.get("counts").and_then(|x| x.as_object()) {
            for suit in ["spade", "heart", "club", "diamond"] {
                if let Some(n) = obj.get(suit).and_then(|x| x.as_u64()) {
                    counts.push((suit.to_string(), n as u32));
                }
            }
        }
        entries.push(PileEntry { card, counts });
    }
    extension_pack_from_pile(
        ExtensionDef {
            name: "matrix-pile",
            version: 1,
            description: "pile",
        },
        "matrix_pile",
        &PileSpec {
            source,
            total,
            entries,
        },
    )
    .ok()
}

fn load(pkg: &str) -> Option<ExtensionPack> {
    let dir = assets(pkg);
    if !dir.is_dir() {
        return None;
    }
    if let Some(p) = load_pile(&dir) {
        return Some(p);
    }
    for f in ["cards.json", "character.json"] {
        let path = dir.join(f);
        if !path.is_file() {
            continue;
        }
        let json = std::fs::read_to_string(&path).ok()?;
        let mut out = content_native_loader::bridge_pack(&json).ok()?;
        if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
            if let Ok(b) = content_native_loader::bridge_generals(&chars) {
                out.pack.generals = b.generals;
            }
        }
        return Some(out.pack);
    }
    None
}

/// 单包升级验收（返回 `Some(包名)` 表示通过；`None` 表示跳过）。
fn check(pkg: &str) -> Option<String> {
    let Some(v1) = load(pkg) else {
        eprintln!("NOTE 升级矩阵：{pkg} 无法装载 —— 跳过");
        return None;
    };
    let e = engine_native::Engine::new_with_extensions(59, 4, &[&v1]).expect("v1 建局");
    let snap = e.save();
    let before = e.state_hash();
    let mut v2 = load(pkg).expect("v2 可装载");
    let old = v2.def.version;
    v2.def.version = old + 1;
    match engine_native::Engine::restore_with_extensions(&snap, &[&v2]) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Ok(_) => panic!("{pkg}: 错版恢复**不应成功**（内容锁未生效）"),
        Err(other) => panic!("{pkg}: 错版恢复应被拒（RuleVersionMismatch），实得 {other:?}"),
    }
    let restored = engine_native::Engine::restore_with_extensions(&snap, &[&v1])
        .unwrap_or_else(|err| panic!("{pkg}: v1 恢复失败 {err:?}"));
    assert_eq!(
        before,
        restored.state_hash(),
        "{pkg}: v1 恢复后 hash 应一致"
    );
    eprintln!("DBG 升级矩阵：{pkg} v{old}→v{} 拒版 ✅、回滚 ✅", old + 1);
    Some(pkg.to_string())
}

/// **逐包升级矩阵**：候选包全部跑一遍；至少 12 个包通过。
#[test]
fn upgrade_matrix_over_packages() {
    let candidates = [
        "shahai-gujian-upstream-noname",
        "shahai-mtg-upstream-noname",
        "shahai-swd-upstream-noname",
        "shahai-hearth-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
        "shahai-gwent-upstream-noname",
        "huanle-upstream-noname",
        "wuxing-upstream-liuli",
        "wudian-upstream-noname",
        "d3-upstream-noname",
        "cardpile-upstream-noname",
        "cardpile-upstream-liuli",
        // 第二轮（第 493 轮）：补齐其余**有资产**的包（由 `nc12_bench probe` 的 22 个 OK 包反推）
        "boss-zhugeliang-upstream-liuli",
        "doomsday-brawl-upstream-liuli",
        "epic-cards-upstream-liuli",
        "hero-kill-upstream-noname",
        "longxue-upstream-liuli",
        "mark-supplement-upstream-liuli",
        "select-beautify-upstream-liuli",
        "shaai-ow-upstream-noname",
        "shaai-xianjian-upstream-noname",
    ];
    let mut passed = Vec::new();
    for pkg in candidates {
        if let Some(p) = check(pkg) {
            passed.push(p);
        }
    }
    eprintln!(
        "DBG 升级矩阵：通过 {}/{} 个包",
        passed.len(),
        candidates.len()
    );
    assert!(
        passed.len() >= 20,
        "应至少 20 个包通过升级矩阵，实得 {}：{passed:?}",
        passed.len()
    );
}
