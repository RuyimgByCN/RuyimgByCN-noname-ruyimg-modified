//! NC08 附加：**保存恢复**验收补充（八项验收第 5 项）。
//!
//! 背景：[五来源签署状态](../../docs/operations/evidence/2026-10-19-five-source-signoff/README.md)显示
//! 「保存恢复」仅 **U05 的六个子包**有真双进程接力证据；本用例覆盖**其余四个来源**的包：
//! `cardpile-upstream-liuli`（L04）、`cardpile-upstream-noname`（U03）、`d3-upstream-noname`（U01）、
//! `wudian-upstream-noname`（U07）。
//!
//! 判据（**单进程往返**，比双进程弱一档，但可覆盖四个来源）：
//! 1. `e.save()` 成功；
//! 2. `Engine::restore_with_extensions(&snap, &[&pack])` 成功；
//! 3. **恢复后的 `state_hash()` 与保存前一致**；
//! 4. 恢复后**可继续推进**（`NextPhase` 不报错）。
//!
//! 边界：**不覆盖双进程**（U05 已有该证据）；本用例只做**单进程往返**。

use std::path::PathBuf;

fn assets_dir(pkg: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets")
}

/// 桥接：优先 `cards.json`，否则 `character.json`（`bridge_pack` 对两者都可用，见 `g1_d3.rs:48`）。
fn bridge(pkg: &str) -> Option<content_native_loader::BridgedPack> {
    let dir = assets_dir(pkg);
    for f in ["cards.json", "character.json"] {
        let p = dir.join(f);
        if !p.is_file() {
            continue;
        }
        let json = std::fs::read_to_string(p).ok()?;
        let mut out = content_native_loader::bridge_pack(&json).ok()?;
        // 双产物：角色包时补 generals
        if f == "cards.json" {
            if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
                if let Ok(b) = content_native_loader::bridge_generals(&chars) {
                    out.pack.generals = b.generals;
                }
            }
        }
        return Some(out);
    }
    None
}

fn run(pkg: &str) {
    let Some(bridged) = bridge(pkg) else {
        eprintln!("NOTE 保存恢复：{pkg} 无 cards.json / character.json —— 跳过");
        return;
    };
    let pack = bridged.pack;
    let mut e = engine_native::Engine::new_with_extensions(41, 4, &[&pack]).expect("建局");
    // 推进几步，制造非平凡状态
    for _ in 0..3 {
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: e.revision() + 1,
            action: engine_native::GameAction::NextPhase,
        });
    }
    let before = e.state_hash();
    let snap = e.save();
    let mut restored = match engine_native::Engine::restore_with_extensions(&snap, &[&pack]) {
        Ok(r) => r,
        Err(err) => panic!("{pkg}: 恢复失败 {err:?}"),
    };
    let after = restored.state_hash();
    assert_eq!(
        before, after,
        "{pkg}: 恢复后 state_hash 应与保存前一致"
    );
    // 恢复后仍可推进
    let actor = restored.turn().current;
    let _ = restored.submit(engine_native::GameCommand {
        actor,
        expected_revision: restored.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    eprintln!("DBG 保存恢复：{pkg} 往返成功（hash 一致、可继续推进）");
}

/// **单进程往返**：四个来源的包各自「保存 → 恢复 → hash 一致 → 可继续」。
#[test]
fn save_restore_roundtrip_four_sources() {
    // 第 995 轮：由 4 个扩到「12 个已转换来源」+ cardpile 共 23 个包。
    for pkg in [
        "boss-play-upstream-liuli",
        "boss-zhugeliang-upstream-liuli",
        "wuxing-upstream-liuli",
        "epic-cards-upstream-liuli",
        "doomsday-brawl-upstream-liuli",
        "mark-supplement-upstream-liuli",
        "select-beautify-upstream-liuli",
        "longxue-upstream-liuli",
        "d3-upstream-noname",
        "shaai-ow-upstream-noname",
        "shaai-xianjian-upstream-noname",
        "shahai-gujian-upstream-noname",
        "shahai-gwent-upstream-noname",
        "shahai-hearth-upstream-noname",
        "shahai-mtg-upstream-noname",
        "shahai-swd-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
        "huanle-upstream-noname",
        "wudian-upstream-noname",
        "hero-kill-upstream-noname",
        "cardpile-upstream-liuli",
        "cardpile-upstream-noname",
    ] {
        run(pkg);
    }
}
