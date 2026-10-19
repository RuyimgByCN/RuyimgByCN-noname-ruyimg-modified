//! NC08 附加：**回放确定性**（差异 #3「余随机代抽」的 RNG 面验证，2026-10-19）。
//!
//! 背景：U07 差异 #3 是「守卫弃牌上游一次性选 n 张——**原生首张自选 + 余随机代抽**」。
//! 「随机」在**同种子**下仍应**完全可复现**（引擎 RNG 为确定性），否则回放会漂移。
//!
//! 判据（**每个包**）：
//! 1. 用**同一 seed** 建两个引擎，跑**相同指令序列**；
//! 2. 两步之后的 `state_hash()` **必须相等**；
//! 3. 用**不同 seed** 应**大概率不同**（反向对照，确认 seed 真的影响状态）。
//!
//! 边界：只验证「**同 seed 同轨迹**」；**不验证**「与上游 RNG 流一致」（上游用 `Math.random`，不可比）。

use std::path::PathBuf;

/// `pile.json` → ExtensionPack（U03/L04 走该管线）。
fn load_pile(dir: &std::path::Path) -> Option<engine_native::content::extension::ExtensionPack> {
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
            name: "replay-probe",
            version: 1,
            description: "pile",
        },
        "replay_probe",
        &engine_native::content::pile::PileSpec { source, total, entries },
    )
    .ok()
}

fn load(pkg: &str) -> Option<engine_native::content::extension::ExtensionPack> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    if let Some(p) = load_pile(&dir) {
        return Some(p);
    }
    for f in ["cards.json", "character.json"] {
        let p = dir.join(f);
        if !p.is_file() {
            continue;
        }
        let Ok(json) = std::fs::read_to_string(&p) else { continue };
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

/// 跑 `steps` 步 `NextPhase`，返回最终 `state_hash`。
fn trace(pack: &engine_native::content::extension::ExtensionPack, seed: u64, steps: usize) -> [u8; 32] {
    let mut e = engine_native::Engine::new_with_extensions(seed, 4, &[pack]).expect("建局");
    for i in 0..steps {
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: (i + 1) as u64,
            action: engine_native::GameAction::NextPhase,
        });
    }
    e.state_hash()
}

/// **回放确定性**：同 seed 同轨迹；不同 seed 不同（反向对照）。
#[test]
fn replay_determinism_per_pack() {
    for pkg in [
        "cardpile-upstream-noname",
        "shahai-swd-upstream-noname",
        "d3-upstream-noname",
        "wudian-upstream-noname",
    ] {
        let Some(pack) = load(pkg) else {
            eprintln!("NOTE 回放确定性：{pkg} 无法桥接 —— 跳过");
            continue;
        };
        let a = trace(&pack, 61, 12);
        let b = trace(&pack, 61, 12);
        let c = trace(&pack, 62, 12);
        eprintln!(
            "DBG 回放确定性：{pkg} 同 seed 一致={}；异 seed 不同={}",
            a == b,
            a != c
        );
        assert_eq!(a, b, "{pkg}: 同 seed 的轨迹必须完全一致（回放可复现）");
        assert_ne!(a, c, "{pkg}: 不同 seed 应产生不同状态（确认 seed 生效）");
    }
}
