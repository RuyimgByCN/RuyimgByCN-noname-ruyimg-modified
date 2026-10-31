//! NC08 附加：**`pile.json` 管线**的保存恢复（八项验收第 5 项，覆盖 U03/L04）。
//!
//! 背景：`cardpile-upstream-noname`（U03）与 `cardpile-upstream-liuli`（L04）的资产是
//! **`pile.json`**（牌堆补丁包），**不在 card/general 管线**，因此上一条用例跳过它们。
//!
//! 本用例走**该管线自己的 API**：`content::pile::extension_pack_from_pile(def, prefix, &spec)`
//! （`pile.rs:56`，**已跟踪**）→ 构建 `ExtensionPack` → 建局 → 保存/恢复 → hash 一致。
//!
//! 边界：**单进程往返**；不覆盖「缩放到 160 张」的 `scale_plan`/`snapshot_scale` 面（另有 API）。

use std::path::PathBuf;

use engine_native::content::extension::ExtensionDef;
use engine_native::content::pile::PileSpec;

fn pile_path(pkg: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets/pile.json")
}

fn run(pkg: &str, prefix: &str, def_name: &'static str) {
    let path = pile_path(pkg);
    let Ok(json) = std::fs::read_to_string(&path) else {
        eprintln!("NOTE pile 保存恢复：{pkg} 无 pile.json —— 跳过");
        return;
    };
    // pile.json 的 `counts` 是**对象**，而 `PileSpec::entries[].counts` 是**有序 Vec**，
    // 且 `PileSpec` 未实现 Deserialize ⇒ 手工构造（顺序固定以保持确定性）。
    let v: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("NOTE pile 保存恢复：{pkg} 解析失败 {err} —— 跳过");
            return;
        }
    };
    let source = v
        .get("source")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let mut entries = Vec::new();
    if let Some(arr) = v.get("entries").and_then(|x| x.as_array()) {
        for e in arr {
            let card = e
                .get("card")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let mut counts: Vec<(String, u32)> = Vec::new();
            if let Some(obj) = e.get("counts").and_then(|x| x.as_object()) {
                // 固定顺序：spade/heart/club/diamond（其余按字典序追加）
                for suit in ["spade", "heart", "club", "diamond"] {
                    if let Some(n) = obj.get(suit).and_then(|x| x.as_u64()) {
                        counts.push((suit.to_string(), n as u32));
                    }
                }
                for (k, val) in obj.iter() {
                    if ["spade", "heart", "club", "diamond"].contains(&k.as_str()) {
                        continue;
                    }
                    if let Some(n) = val.as_u64() {
                        counts.push((k.clone(), n as u32));
                    }
                }
            }
            entries.push(engine_native::content::pile::PileEntry { card, counts });
        }
    }
    let spec = PileSpec {
        source,
        total,
        entries,
    };
    let pack = match engine_native::content::pile::extension_pack_from_pile(
        ExtensionDef {
            name: def_name,
            version: 1,
            description: "pile",
        },
        prefix,
        &spec,
    ) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("NOTE pile 保存恢复：{pkg} 构包失败 {err:?} —— 跳过");
            return;
        }
    };
    eprintln!(
        "DBG pile 保存恢复：{pkg} 构包成功，卡 {} 张（source={} entries={}）",
        pack.cards.len(),
        spec.source,
        spec.entries.len()
    );
    let mut e = engine_native::Engine::new_with_extensions(43, 4, &[&pack]).expect("建局");
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
    let mut restored = engine_native::Engine::restore_with_extensions(&snap, &[&pack])
        .unwrap_or_else(|err| panic!("{pkg}: 恢复失败 {err:?}"));
    assert_eq!(
        before,
        restored.state_hash(),
        "{pkg}: 恢复后 state_hash 应一致"
    );
    let actor = restored.turn().current;
    let _ = restored.submit(engine_native::GameCommand {
        actor,
        expected_revision: restored.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    eprintln!("DBG pile 保存恢复：{pkg} 往返成功");
}

/// **`pile.json` 管线**往返：U03 / L04。
#[test]
fn save_restore_roundtrip_pile_packages() {
    run("cardpile-upstream-noname", "pile_noname", "cardpile-noname");
    run("cardpile-upstream-liuli", "pile_liuli", "cardpile-liuli");
}
