//! P2/NC08：真实 native 与真实 WASM 的同种子轨迹对照（第一阶段：标准 1 人局）。
//!
//! 由 `docs/operations/evidence/2026-10-24-native-wasm-cross-target/compare.py`
//! 驱动：本测试把 native 每步 revision/hash/公开事件码写到环境变量指定文件；
//! Node 侧用同一 seed 与命令序列驱动本轮生成的 wasm 包，再由 Python 逐项比较。
//! 不是 host 包装层自证：wasm 侧加载的是 `apps/web-client/src/worker/native-pkg`。

use engine_native::{Engine, GameAction, GameCommand};
use serde_json::json;
use std::io::Write;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn run(engine: &mut Engine, steps: std::ops::Range<usize>, dedup_base: u64) -> Vec<serde_json::Value> {
    let mut records = Vec::new();
    let mut cursor = engine.log().len();
    for (offset, index) in steps.enumerate() {
        let action = if let Some(choice) = engine.open_choice() {
            let observer = choice.observer;
            let choice_id = choice.id;
            let selection = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min as usize)
                .cloned()
                .collect();
            (observer, GameAction::Answer {
                choice_id,
                selection,
            })
        } else {
            (engine.turn().current, GameAction::NextPhase)
        };
        let (actor, action) = action;
        let outcome = engine.submit(GameCommand {
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
            "native step {index} rejected: {outcome:?}"
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
fn native_standard_one_player_trajectory() {
    let Some(output) = std::env::var_os("NONAME_CROSS_TARGET_NATIVE_OUT") else {
        return;
    };
    let seed = 20_261_024u64;
    let midpoint = 20usize;
    let total = 40usize;

    let mut direct = Engine::new(seed, 1);
    let direct_steps = run(&mut direct, 0..total, 1);

    let mut first = Engine::new(seed, 1);
    let before_restore = run(&mut first, 0..midpoint, 1);
    let snapshot = first.save();
    let bytes = engine_native::archive::encode_archive(&snapshot);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode archive");
    let mut restored = Engine::restore(&decoded).expect("restore archive");
    let after_restore = run(&mut restored, midpoint..total, 10_000);

    let payload = json!({
        "seed": seed,
        "players": 1,
        "direct": direct_steps,
        "restore": {
            "midpoint": midpoint,
            "before": before_restore,
            "after": after_restore,
        },
    });
    let mut file = std::fs::File::create(output).expect("create native trajectory output");
    file.write_all(serde_json::to_string_pretty(&payload).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}

#[test]
fn native_all_seven_modes_cross_target_and_restore_trajectories() {
    let seed = 20_261_106u64;
    let modes: Vec<(&str, Box<dyn Fn() -> Engine>)> = vec![
        ("identity", Box::new(move || Engine::new_identity(seed, 4))),
        ("kingdoms", Box::new(move || Engine::new_kingdoms(seed, 4))),
        (
            "versus",
            Box::new(move || Engine::new_versus(seed, [vec![0, 2], vec![1, 3]])),
        ),
        ("boss", Box::new(move || Engine::new_boss(seed, 4, 0, 2))),
        ("doudizhu", Box::new(move || Engine::new_doudizhu(seed, 3, 0))),
        ("brawl", Box::new(move || Engine::new_brawl(seed, 4))),
        ("single", Box::new(move || Engine::new_single(seed))),
    ];

    for (mode_name, make_engine) in modes {
        let mut direct = make_engine();
        let direct_steps = run(&mut direct, 0..24, 1);

        let mut split = make_engine();
        let before = run(&mut split, 0..12, 1);
        let bytes = engine_native::archive::encode_archive(&split.save());
        let decoded = engine_native::archive::decode_archive(&bytes).expect("decode archive");
        let mut restored = Engine::restore(&decoded).expect("restore archive");
        let after = run(&mut restored, 12..24, 5_000);

        let combined: Vec<serde_json::Value> = before.into_iter().chain(after).collect();
        assert_eq!(
            direct_steps.len(),
            combined.len(),
            "{mode_name}: step count matches"
        );
        for (i, (lhs, rhs)) in direct_steps.iter().zip(combined.iter()).enumerate() {
            assert_eq!(
                lhs["revision"], rhs["revision"],
                "{mode_name} step {i} revision mismatch"
            );
            assert_eq!(
                lhs["hash"], rhs["hash"],
                "{mode_name} step {i} state_hash mismatch"
            );
            assert_eq!(
                lhs["codes"], rhs["codes"],
                "{mode_name} step {i} public_event_codes mismatch"
            );
        }
    }
}
