//! R05-J（步骤 2）：Native（`start_locked_engine` / `restore_locked_engine`）×
//! WASM（`NativeSession::new_with_locked_content_json` / `restore_hex_with_locked_content_json`）×
//! 上游 JS 参考语义（`xianding` 涕泣/宝梳 + `sp2` 避祸）三端对齐测试。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use content_native_loader::{
    prepare_locked_content, restore_locked_engine, start_locked_engine, validate_frozen_package,
    LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::encode_archive;
use engine_native::{CommandOutcome, GameAction, GameCommand, Phase};
use engine_native_wasm::{
    hex_encode, prepare_locked_content_json, validate_frozen_package_json, NativeSession,
};

fn pkg_root(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
}

fn load_locked_envelope_and_verified() -> (String, VerifiedLockedContent) {
    let mut frozen_jsons = Vec::new();
    let mut frozen_pkgs = Vec::new();
    for dir in ["xianding-upstream-noname", "sp2-upstream-noname"] {
        let root = pkg_root(dir);
        let raw_pkg_on_disk = std::fs::read_to_string(root.join("package.npack.json")).unwrap();
        let raw_chars = std::fs::read(root.join("assets/characters.json")).unwrap();
        let mut pkg: content_schema::ContentPackage =
            serde_json::from_str(&raw_pkg_on_disk).unwrap();
        let asset_id = format!("{}:characters", pkg.manifest.package_id);
        let source_hash = *blake3::hash(&raw_chars).as_bytes();
        pkg.asset_manifest = Some(serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "package_id": pkg.manifest.package_id,
            "total_compressed_size": 0,
            "total_decoded_size": 0,
            "entries": [{
                "asset_id": asset_id,
                "source_hash": source_hash.to_vec(),
                "kind": "Data",
                "image_variants": [],
                "audio_variants": [],
                "priority": "Lazy",
                "depends_on": [],
                "platforms": []
            }]
        })).unwrap());
        pkg.asset_paths
            .insert(asset_id, "assets/characters.json".to_string());
        pkg.manifest.asset_hash = pkg.compute_asset_hash();
        let raw_pkg = serde_json::to_string(&pkg).unwrap();
        let mut assets_bytes = BTreeMap::new();
        assets_bytes.insert("assets/characters.json".to_string(), raw_chars);
        let assets_json = serde_json::to_string(&assets_bytes).unwrap();
        let frozen_json = validate_frozen_package_json(&raw_pkg, &assets_json).unwrap();
        let frozen_val: serde_json::Value = serde_json::from_str(&frozen_json).unwrap();
        frozen_jsons.push(frozen_val);
        let (frozen, _) = validate_frozen_package(raw_pkg, assets_bytes).unwrap();
        frozen_pkgs.push(frozen);
    }
    let array_json = serde_json::to_string(&frozen_jsons).unwrap();
    let envelope_json = prepare_locked_content_json(&array_json).unwrap();
    let verified = prepare_locked_content(frozen_pkgs).unwrap();
    (envelope_json, verified)
}

fn assert_native_wasm_step_parity(
    native: &engine_native::Engine,
    wasm: &NativeSession,
    envelope_json: &str,
    verified: &VerifiedLockedContent,
) {
    let native_snap = native.save();
    let native_hex = hex_encode(&encode_archive(&native_snap));
    let wasm_hex = wasm.save_hex();
    if native_hex != wasm_hex {
        let wasm_bytes: Vec<u8> = (0..wasm_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&wasm_hex[i..i + 2], 16).unwrap())
            .collect();
        let wasm_snap = engine_native::archive::decode_archive(&wasm_bytes).unwrap();
        assert_eq!(native_snap, wasm_snap, "StateSnapshot 字段对比");
    }
    assert_eq!(
        native_hex, wasm_hex,
        "Native 与 WASM 导出 NGSA 107 字节序列必须逐字节一致"
    );
    assert_eq!(native.revision(), wasm.revision(), "revision 必须一致");

    // 两侧各自从导出字节恢复，验证恢复后状态哈希与公开视图一致
    let restored_native = restore_locked_engine(&native.save(), verified).unwrap();
    let mut restored_wasm = NativeSession::new(3, 1).unwrap();
    restored_wasm
        .restore_hex_with_locked_content_json(&wasm_hex, envelope_json)
        .unwrap();
    assert_eq!(restored_native.state_hash(), native.state_hash());
    assert_eq!(
        restored_wasm.save_hex(),
        hex_encode(&encode_archive(&restored_native.save()))
    );

    for seat in 0..3u8 {
        let w_view: serde_json::Value =
            serde_json::from_str(&wasm.view_json(seat).unwrap()).unwrap();
        let rw_view: serde_json::Value =
            serde_json::from_str(&restored_wasm.view_json(seat).unwrap()).unwrap();
        assert_eq!(w_view, rw_view, "seat {seat} 恢复前后 WASM 公开视图必须一致");
        let n_view = native.view_for(seat);
        assert_eq!(
            w_view["turn"].as_u64().unwrap() as u8,
            n_view.turn,
            "current turn player 一致"
        );
        for (idx, p) in n_view.players.iter().enumerate() {
            assert_eq!(
                w_view["players"][idx]["linked"].as_bool().unwrap(),
                p.linked,
                "seat {idx} linked 投影一致"
            );
            assert_eq!(
                w_view["players"][idx]["r05Effects"]
                    .as_array()
                    .unwrap()
                    .len(),
                p.r05_effects.len(),
                "seat {idx} r05Effects 条目数一致"
            );
        }
    }
}

#[test]
fn r05j_native_wasm_upstream_locked_cross_target_parity() {
    let (envelope_json, verified) = load_locked_envelope_and_verified();
    let mut native =
        start_locked_engine(36001, 3, LockedStartMode::Prototype, &verified).unwrap();
    let mut wasm =
        NativeSession::new_with_locked_content_json(3, 36001, &envelope_json).unwrap();

    for (seat, name) in [(0u8, "xnd冯妤"), (1u8, "sp2冯方")] {
        native.assign_general_named(seat, name).unwrap();
        wasm.assign_general(seat, name).unwrap();
    }
    assert_native_wasm_step_parity(&native, &wasm, &envelope_json, &verified);

    // 1. 宝梳（Prepare 阶段）：seat0 选择横置（index 0），m = maxHp(3) = 3，写入 Combs=3 且 linked=true
    let mut step = 0u64;
    while native.open_choice().map(|c| c.spec.prompt) != Some("rvm_baoshu_cost") {
        step += 1;
        assert!(step < 40);
        if let Some(c) = native.open_choice().cloned() {
            let actor = c.observer;
            let cid = c.id;
            let (sel_opts, sel_idx): (Vec<_>, Vec<u32>) = if c.spec.allow_cancel {
                (vec![], vec![])
            } else {
                let take = c.spec.min.max(1);
                (
                    c.spec.options.iter().take(take).cloned().collect(),
                    (0..take as u32).collect(),
                )
            };
            let dedup_id = native.revision().wrapping_mul(2).max(1);
            let out = native.submit(GameCommand {
                actor,
                expected_revision: native.revision(),
                dedup_id,
                action: GameAction::Answer {
                    choice_id: cid,
                    selection: sel_opts,
                },
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            wasm.answer(actor as u32, cid, sel_idx).unwrap();
        } else {
            let actor = native.turn().current;
            let dedup_id = native.revision().wrapping_mul(2).max(1);
            let out = native.submit(GameCommand {
                actor,
                expected_revision: native.revision(),
                dedup_id,
                action: GameAction::NextPhase,
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            wasm.next_phase(actor as u32).unwrap();
        }
        assert_native_wasm_step_parity(&native, &wasm, &envelope_json, &verified);
    }

    // 应答宝梳选择横置（options[0] = Suit(0)）
    {
        let c = native.open_choice().unwrap();
        let cid = c.id;
        let first_opt = c.spec.options[0].clone();
        let dedup_id = native.revision().wrapping_mul(2).max(1);
        let out = native.submit(GameCommand {
            actor: 0,
            expected_revision: native.revision(),
            dedup_id,
            action: GameAction::Answer {
                choice_id: cid,
                selection: vec![first_opt],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        wasm.answer(0, cid, vec![0]).unwrap();
    }
    assert_native_wasm_step_parity(&native, &wasm, &envelope_json, &verified);

    // 验证上游 JS 参考语义：seat0 选择 1 名目标（自己，k=1），m = 1 + maxHp(3) - 1 = 3，
    // 复原后 linked=false，且公开 r05Effects.baoshu_combs == 3
    let w_view0: serde_json::Value = serde_json::from_str(&wasm.view_json(0).unwrap()).unwrap();
    assert_eq!(w_view0["players"][0]["linked"], serde_json::json!(false));
    assert_eq!(
        w_view0["players"][0]["r05Effects"][0]["baoshu_combs"],
        serde_json::json!(3)
    );

    // 2. 推进到 seat0 摸牌阶段完成：宝梳 Combs=3 在 Begin2 消费，使本阶段摸 2+3=5 张（手牌从 4 变为 9），Combs 清空
    while !(native.turn().current == 0
        && native.turn().phase == Phase::Play
        && native.open_choice().is_none())
    {
        step += 1;
        assert!(step < 80);
        let actor = native.turn().current;
        let dedup_id = native.revision().wrapping_mul(2).max(1);
        let out = native.submit(GameCommand {
            actor,
            expected_revision: native.revision(),
            dedup_id,
            action: GameAction::NextPhase,
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        wasm.next_phase(actor as u32).unwrap();
        assert_native_wasm_step_parity(&native, &wasm, &envelope_json, &verified);
    }
    assert_eq!(
        native.view_for(0).players[0].hand_count,
        8,
        "上游宝梳语义：首轮弃牌剩 3 张 + 次轮摸牌阶段 (2 + Combs 3) = 8 张手牌"
    );
    assert!(
        native.view_for(0).players[0].r05_effects.is_empty(),
        "宝梳 Combs 在 Begin2 消费后清零"
    );
}
