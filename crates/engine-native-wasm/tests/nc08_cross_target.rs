//! NC08 跨目标门禁：**原生引擎与 WASM 包装层同源**——同一输入必须得到同一
//! 运行轨迹与同一恢复结果，且门禁**可失败**（受控变异证明）。
//!
//! 口径：本文件直接调用 `engine-native-wasm` 的内部入口（`engine_from_packs` /
//! `engine_from_archive` / `NativeEngine::save_hex` / `restore_hex*`），不依赖
//! 浏览器或 wasm-bindgen 运行时；真正的 WASM 目标运行由 `wasm-pack` 产物冒烟
//! （`pnpm smoke:native`）覆盖，两者边界分别记录。
//!
//! 验收：
//! 1. 同一 seed + 同一包集合：原生 `Engine` 与 WASM 包装层**修订号与归档字节一致**；
//! 2. 同包恢复：WASM 包装层 `restore_hex_with_packs_json` 恢复后目录与存档一致；
//! 3. **受控变异（门禁可失败）**：换 seed、换包版本、用标准目录恢复扩展局，
//!    三者都必须**明确失败或不一致**——否则门禁无分辨力；
//! 4. 逐步推进 N 步后仍保持一致（不只比较初始状态）。

use engine_native::content::extension::ExtensionPack;

const PACKS: &str = r#"[{"pack":{"name":"nc08-pack","version":1,"description":"nc08","card_prefix":"nc08p_"},"cards":[{"name":"extra","kind":"sha","suit":"spade","rank":7},{"name":"extra2","kind":"shan","suit":"heart","rank":2}]}]"#;

fn native_engine(seed: u64, seats: u8, packs_json: &str) -> engine_native::Engine {
    // WASM 侧的 `packs_json` 是**包数组**；`bridge_pack` 收单个包对象 → 取首个元素
    let arr: Vec<serde_json::Value> = serde_json::from_str(packs_json).expect("包数组");
    let one = serde_json::to_string(&arr[0]).expect("序列化单包");
    let bridged = content_native_loader::bridge_pack(&one).expect("桥接包");
    let packs: Vec<ExtensionPack> = vec![bridged.pack];
    let refs: Vec<&ExtensionPack> = packs.iter().collect();
    engine_native::Engine::new_with_extensions(seed, seats as usize, &refs).expect("原生建局")
}

/// 同一输入 → 原生与 WASM 包装层**归档字节逐字节一致**。
#[test]
fn nc08_native_and_wasm_agree_on_archive_bytes() {
    let wasm = engine_native_wasm::test_support::engine_from_packs(4, 88001, PACKS)
        .expect("WASM 包装层建局（与导出同一实现路径）");
    let native = native_engine(88001, 4, PACKS);
    assert_eq!(
        wasm.revision(),
        native.revision(),
        "初始修订号一致"
    );
    assert_eq!(
        engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
            &wasm.save()
        )),
        engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
            &native.save()
        )),
        "归档字节逐字节一致（同一状态模型）"
    );
}

/// 逐步推进后仍一致：两侧用同一命令序列（NextPhase 轮转），比较修订号与归档。
#[test]
fn nc08_native_and_wasm_agree_after_steps() {
    let mut wasm = engine_native_wasm::test_support::engine_from_packs(4, 88002, PACKS)
        .expect("WASM 建局");
    let mut native = native_engine(88002, 4, PACKS);
    for step in 0..12u64 {
        // 原生：直接提交；WASM：走包装层命令入口
        let seat = native.turn().current;
        let _ = native.submit(engine_native::GameCommand {
            actor: seat,
            expected_revision: native.revision(),
            dedup_id: step + 1,
            action: engine_native::GameAction::NextPhase,
        });
        // WASM 侧同样直接提交（`engine_from_packs` 与导出共享同一 Engine 实现）
        let wasm_seat = wasm.turn().current;
        let _ = wasm.submit(engine_native::GameCommand {
            actor: wasm_seat,
            expected_revision: wasm.revision(),
            dedup_id: step + 1,
            action: engine_native::GameAction::NextPhase,
        });
        assert_eq!(
            wasm.revision(),
            native.revision(),
            "第 {step} 步修订号一致（两侧命令序列同源）"
        );
    }
    assert_eq!(
        engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
            &wasm.save()
        )),
        engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
            &native.save()
        )),
        "推进后归档仍逐字节一致"
    );
}

/// 同包恢复：WASM 包装层恢复后与原生恢复结果一致。
#[test]
fn nc08_wasm_restore_matches_native_restore() {
    let native = native_engine(88003, 4, PACKS);
    let bytes = engine_native::archive::encode_archive(&native.save());
    let hex = engine_native_wasm::test_support::hex_encode(&bytes);

    let restored = engine_native_wasm::test_support::engine_from_archive(&hex, PACKS)
        .expect("同包恢复");
    assert_eq!(
        restored.revision(),
        native.revision(),
        "恢复后修订号一致"
    );
    // 恢复后重新存档：**状态摘要必须一致**（归档字节含恢复代次等宿主字段，
    // 不要求逐字节相同；一致性以状态摘要为准）
    assert_eq!(
        restored.state_hash(),
        native.state_hash(),
        "恢复后状态摘要与原件一致（往返稳定）"
    );
    assert_eq!(
        engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
            &restored.save()
        ))
        .len(),
        hex.len(),
        "恢复后归档长度一致（结构稳定）"
    );
}

/// 受控变异（门禁可失败）：三类反例必须被拒绝或产生不一致。
#[test]
fn nc08_gate_can_fail_on_controlled_mutations() {
    let native = native_engine(88004, 4, PACKS);
    let hex = engine_native_wasm::test_support::hex_encode(&engine_native::archive::encode_archive(
        &native.save(),
    ));

    // ① 标准目录恢复扩展局 → 必须失败
    assert!(
        engine_native_wasm::test_support::engine_from_archive(&hex, "[]").is_err(),
        "标准目录不得恢复扩展局"
    );

    // ② 包版本不符 → 必须失败
    let wrong = PACKS.replace("\"version\":1", "\"version\":2");
    assert!(
        engine_native_wasm::test_support::engine_from_archive(&hex, &wrong).is_err(),
        "错版包必须拒绝"
    );

    // ③ 换 seed 的局 → 归档必须不同（否则门禁对输入不敏感）
    let other = native_engine(88005, 4, PACKS);
    let other_hex = engine_native_wasm::test_support::hex_encode(
        &engine_native::archive::encode_archive(&other.save()),
    );
    assert_ne!(hex, other_hex, "不同 seed 的归档不得相同");

    // ④ 篡改归档字节 → 必须失败（解码或校验拒绝）
    let mut tampered = hex.clone();
    let last = tampered.pop().expect("非空");
    tampered.push(if last == '0' { '1' } else { '0' });
    assert!(
        engine_native_wasm::test_support::engine_from_archive(&tampered, PACKS).is_err(),
        "篡改归档必须被拒绝"
    );
}
