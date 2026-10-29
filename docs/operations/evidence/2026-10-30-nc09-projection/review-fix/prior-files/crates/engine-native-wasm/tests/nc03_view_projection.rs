//! NC03/NC09 本地投影（GT-013；裁决⑥⑦，2026-10-28）：`view_json` 座位对象
//! 附带**观察者过滤**的 `identity`；顶层 `cards` 只对观察者可见牌
//! （本人手牌 + 全场装备/判定 + 弃牌/处理区）附带真实牌面
//! （def/suit/rank/kind）。验收双向：
//! - 不泄露：他人手牌 id 与牌堆内容不出现在 `cards`；
//! - 不占位：可见牌的 def/suit/rank 均非空，identity 按可见性矩阵输出
//!   而非恒 null；恢复（save→restore）后隐私与牌面一致。
//!
//! 身份可见性与内核 `view_for`/服务端 `native_view.rs` 同口径（本人/主公/
//! 已死亡可见；死亡公开语义由内核 `t13_info_isolation` 覆盖，本文件只验
//! JSON 层 1:1 序列化与开局存活矩阵）。

use engine_native_wasm::NativeEngine;
use serde_json::Value;
use std::collections::BTreeSet;

fn seat_players(view: &Value) -> &Vec<Value> {
    view["players"].as_array().unwrap()
}

/// 视图里出现过的全部牌 id（手牌 own + equips + judge + discard + table）。
fn visible_ids(view: &Value) -> BTreeSet<u64> {
    let mut ids = BTreeSet::new();
    for p in seat_players(view) {
        if let Some(own) = p["hand"]["own"].as_array() {
            ids.extend(own.iter().filter_map(|c| c.as_u64()));
        }
        ids.extend(p["equips"].as_array().unwrap().iter().filter_map(|c| c.as_u64()));
        ids.extend(p["judge"].as_array().unwrap().iter().filter_map(|c| c.as_u64()));
    }
    ids.extend(view["discard"].as_array().unwrap().iter().filter_map(|c| c.as_u64()));
    ids.extend(view["table"].as_array().unwrap().iter().filter_map(|c| c.as_u64()));
    ids
}

/// 裁决⑥：identity 可见性矩阵 + 裁决⑦：可见牌面非占位、隐藏牌不泄露。
#[test]
fn nc03_view_json_identity_and_faces() {
    let engine = NativeEngine::new_identity(5, 20261050).unwrap();
    let mut lord_seat = None;
    for viewer in 0..5u8 {
        let view: Value = serde_json::from_str(&engine.view_json(viewer).unwrap()).unwrap();
        assert_eq!(view["seat"].as_u64(), Some(viewer as u64));
        let mut visible_identity_seats = BTreeSet::new();
        for p in seat_players(&view) {
            let seat = p["seat"].as_u64().unwrap() as u8;
            let identity = &p["identity"];
            if identity.is_null() {
                // 隐藏者必须是：存活、非主公、非本人。
                assert!(seat != viewer, "本人 identity 不应隐藏: viewer={viewer}");
                assert_ne!(
                    lord_seat, Some(seat),
                    "主公身份对所有人公开: viewer={viewer} seat={seat}"
                );
            } else {
                let tag = identity.as_str().unwrap();
                assert!(
                    matches!(tag, "Lord" | "Loyalist" | "Rebel" | "Renegade"),
                    "identity 取值在协议集内: got={tag}"
                );
                visible_identity_seats.insert(seat);
                if tag == "Lord" {
                    match lord_seat {
                        None => lord_seat = Some(seat),
                        Some(lord) => assert_eq!(lord, seat, "主公座位对所有 viewer 一致"),
                    }
                } else {
                    // 非主公身份对他人隐藏——开局无死亡，仅本人可见。
                    assert_eq!(seat, viewer, "非主公身份仅本人可见: viewer={viewer} seat={seat}");
                }
            }
        }
        // 可见集 = {主公} ∪ {本人}（开局无死亡）。
        let mut expect = BTreeSet::from([lord_seat.unwrap(), viewer]);
        expect.insert(viewer);
        assert_eq!(
            visible_identity_seats, expect,
            "identity 可见矩阵: viewer={viewer}"
        );
        // 牌面：可见 id 与 cards 键集一致；每张 def/suit/rank 非空非占位。
        let visible = visible_ids(&view);
        let face_keys: BTreeSet<u64> = view["cards"]
            .as_object()
            .unwrap()
            .keys()
            .filter_map(|k| k.parse().ok())
            .collect();
        assert_eq!(face_keys, visible, "cards 键集 = 可见牌集合 viewer={viewer}");
        assert!(!face_keys.is_empty(), "开局手牌应有可见牌面 viewer={viewer}");
        for (id, face) in view["cards"].as_object().unwrap() {
            // def = 牌种在目录枚举中的序号（与服务端 kind_code_u32 同口径；
            // 杀=0 是合法值，断言非 null 而非 > 0）。
            assert!(face["def"].is_u64(), "def 非占位: id={id} def={}", face["def"]);
            assert!(
                matches!(
                    face["suit"].as_str(),
                    Some("Spade") | Some("Heart") | Some("Club") | Some("Diamond")
                ),
                "suit 非占位: id={id} suit={}",
                face["suit"]
            );
            assert!(
                (1..=13).contains(&face["rank"].as_u64().unwrap_or(0)),
                "rank 非占位: id={id}"
            );
            assert!(face["kind"].is_string(), "kind 非空: id={id}");
        }
        // 不泄露：他人手牌（hidden）无 own 列表；牌堆只发数量。
        for p in seat_players(&view) {
            if p["seat"].as_u64().unwrap() as u8 != viewer {
                assert!(p["hand"]["own"].is_null(), "他人手牌 id 不下发");
                assert!(p["hand"]["hidden"].as_bool().unwrap_or(false));
            }
        }
        assert!(view["deckCount"].as_u64().is_some(), "牌堆仅数量");
    }
    // 主公座位由引擎分配（通常 0），但对所有 viewer 一致且公开。
    assert!(lord_seat.is_some(), "主公身份存在且公开");
}

/// 恢复后隐私一致：save → restore 同一观察者的 identity 矩阵与牌面不变。
#[test]
fn nc03_view_json_privacy_stable_across_restore() {
    let engine = NativeEngine::new_identity(5, 20261052).unwrap();
    let before: Value = serde_json::from_str(&engine.view_json(1).unwrap()).unwrap();
    let hex = engine.save_hex();
    let mut restored = NativeEngine::new_identity(5, 20261052).unwrap();
    restored.restore_hex(&hex).unwrap();
    let after: Value = serde_json::from_str(&restored.view_json(1).unwrap()).unwrap();
    assert_eq!(before["players"], after["players"], "座位投影（含 identity）恢复一致");
    assert_eq!(before["cards"], after["cards"], "可见牌面恢复一致");
    assert_eq!(before["discard"], after["discard"]);
    assert_eq!(before["table"], after["table"]);
}

#[test]
fn nc03_visible_general_definition_survives_local_projection_and_restore() {
    let mut engine = NativeEngine::new_identity(4, 1515).unwrap();
    engine.assign_general(1, "刘备").unwrap();
    let own: Value = serde_json::from_str(&engine.view_json(1).unwrap()).unwrap();
    let other: Value = serde_json::from_str(&engine.view_json(0).unwrap()).unwrap();
    let def = own["players"][1]["characterDef"].as_u64().unwrap();
    assert!(def > 0, "刘备不是目录 0 号");
    assert_eq!(other["players"][1]["characterDef"], def);
    let hex = engine.save_hex();
    engine.restore_hex(&hex).unwrap();
    let restored: Value = serde_json::from_str(&engine.view_json(0).unwrap()).unwrap();
    assert_eq!(restored["players"][1]["characterDef"], def);
}
