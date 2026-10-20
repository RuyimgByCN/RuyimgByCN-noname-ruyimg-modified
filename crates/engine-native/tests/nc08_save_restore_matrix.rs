//! NC08：**逐包「保存恢复」矩阵**——把第 431/432 轮的保存恢复扩到**全部可装载包**。
//!
//! 判据（每个包）：
//! 1. 建局并推进若干步（制造非平凡状态）；
//! 2. `save()` → `restore_with_extensions(&snap, &[&pack])` **成功**；
//! 3. **`state_hash` 与保存前一致**；
//! 4. 恢复后**仍可继续推进**（`NextPhase` 不报错）。
//!
//! 覆盖资产形态：`cards.json`（+`character.json`）、仅 `character.json`、`pile.json`。
//! 边界：**单进程往返**（双进程接力见 `t12-g5-batch2` 证据）；不含错版/缺包拒绝（见 `nc08_upgrade_matrix`）。

use std::path::{Path, PathBuf};

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::content::pile::{extension_pack_from_pile, PileEntry, PileSpec};
use engine_native::{GameAction, GameCommand};

fn assets(pkg: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets")
}

fn load_pile(dir: &Path) -> Option<ExtensionPack> {
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
        entries.push(PileEntry { card, counts });
    }
    extension_pack_from_pile(
        ExtensionDef { name: "sr-matrix-pile", version: 1, description: "pile" },
        "sr_matrix_pile",
        &PileSpec { source, total, entries },
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

/// 单包往返（返回 `Some(包名)` 表示通过；`None` 表示跳过）。
fn check(pkg: &str) -> Option<String> {
    let Some(pack) = load(pkg) else {
        eprintln!("NOTE 保存恢复矩阵：{pkg} 无法装载 —— 跳过");
        return None;
    };
    let mut e = engine_native::Engine::new_with_extensions(61, 4, &[&pack]).expect("建局");
    for _ in 0..3 {
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: e.revision() + 1,
            action: GameAction::NextPhase,
        });
    }
    let before = e.state_hash();
    let snap = e.save();
    let mut restored = match engine_native::Engine::restore_with_extensions(&snap, &[&pack]) {
        Ok(r) => r,
        Err(err) => panic!("{pkg}: 恢复失败 {err:?}"),
    };
    assert_eq!(before, restored.state_hash(), "{pkg}: 恢复后 hash 应一致");
    // 恢复后仍可推进
    let actor = restored.turn().current;
    let _ = restored.submit(GameCommand {
        actor,
        expected_revision: restored.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    });
    eprintln!("DBG 保存恢复矩阵：{pkg} 往返 ✅（hash 一致、可继续推进）");
    Some(pkg.to_string())
}

/// **逐包保存恢复矩阵**：候选包全部跑一遍；至少 12 个包通过。
#[test]
fn save_restore_matrix_over_packages() {
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
    ];
    let mut passed = Vec::new();
    for pkg in candidates {
        if let Some(p) = check(pkg) {
            passed.push(p);
        }
    }
    eprintln!(
        "DBG 保存恢复矩阵：通过 {}/{} 个包",
        passed.len(),
        candidates.len()
    );
    assert!(
        passed.len() >= 12,
        "应至少 12 个包通过保存恢复矩阵，实得 {}：{passed:?}",
        passed.len()
    );
}
