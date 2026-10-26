//! P2/NC08 第二阶段：正式安装包（huanle-cards）在 native 与真实 WASM 的同轨迹。
//!
//! packs JSON 由证据目录的 `huanle-packs.json` 提供；两边使用同一份字符串，
//! 因此 installed pack 身份/内容指纹一致。命令与选择策略同 `nc08_cross_target_real`。

use engine_native_wasm::test_support::{engine_from_archive, engine_from_packs, hex_encode};
use serde_json::json;
use std::io::Write;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn run(
    engine: &mut engine_native::Engine,
    steps: std::ops::Range<usize>,
    dedup_base: u64,
) -> Vec<serde_json::Value> {
    let mut records = Vec::new();
    let mut cursor = engine.log().len();
    for (offset, index) in steps.enumerate() {
        let (actor, action) = if let Some(choice) = engine.open_choice() {
            (
                choice.observer,
                engine_native::GameAction::Answer {
                    choice_id: choice.id,
                    selection: choice
                        .spec
                        .options
                        .iter()
                        .take(choice.spec.min as usize)
                        .cloned()
                        .collect(),
                },
            )
        } else {
            (0, engine_native::GameAction::NextPhase)
        };
        let outcome = engine.submit(engine_native::GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: dedup_base + offset as u64,
            action,
        });
        assert!(
            matches!(
                outcome,
                engine_native::CommandOutcome::Accepted { .. }
                    | engine_native::CommandOutcome::Pending { .. }
            ),
            "pack step {index} rejected: {outcome:?}"
        );
        let end = engine.log().len();
        let codes = engine.public_event_codes_from(cursor);
        cursor = end;
        records.push(json!({
            "revision": engine.revision(),
            "hash": hex(&engine.state_hash()),
            "codes": codes,
        }));
    }
    records
}

#[test]
fn native_huanle_package_trajectory() {
    let Some(output) = std::env::var_os("NONAME_CROSS_TARGET_NATIVE_OUT") else {
        return;
    };
    let packs_path = std::env::var("NONAME_CROSS_TARGET_PACKS").expect("packs path");
    let packs = std::fs::read_to_string(packs_path).expect("packs json");
    let seed = 20_261_025u64;
    let midpoint = 10usize;
    let total = 20usize;

    let mut direct = engine_from_packs(1, seed, &packs).expect("native pack match");
    let direct_steps = run(&mut direct, 0..total, 1);

    let mut first = engine_from_packs(1, seed, &packs).expect("native pack match");
    let before = run(&mut first, 0..midpoint, 1);
    let saved = hex_encode(&engine_native::archive::encode_archive(&first.save()));
    let mut restored = engine_from_archive(&saved, &packs).expect("native pack restore");
    let after = run(&mut restored, midpoint..total, 10_000);

    let payload = json!({
        "seed": seed,
        "players": 1,
        "package": "huanle-cards",
        "direct": direct_steps,
        "restore": { "midpoint": midpoint, "before": before, "after": after },
    });
    let mut file = std::fs::File::create(output).expect("output");
    file.write_all(serde_json::to_string_pretty(&payload).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}
