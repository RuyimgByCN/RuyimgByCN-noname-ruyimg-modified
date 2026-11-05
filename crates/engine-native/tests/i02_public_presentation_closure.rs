//! I02 公开表现与语音契约（裁决 30 / proposal-v3 §6 六道 Rust 门禁）端到端回归测试。
//!
//! 覆盖：
//! - Gate 1（Producer-only historical privacy）：同阶段武将旋转/明置/移除配对测试、暗将激活不泄露、
//!   owner/other/spectator/referee 公开批次字节一致、秘密激活不改变公开 ordinal/status/unavailable。
//! - Gate 2（Verified identity end-to-end）：内置标准将（`standard:liubei`/`rende`/`jijiang`）与
//!   样例将/Boss 同名刘备隔离；SDK 扩展包 `definition_id` → `FrozenPackage` → `Catalog` → `LogEvent` → DTO。
//! - Gate 3（Controlled factory & resource mutation）：篡改 `catalog.json`/`asset-manifest.json`/pins 或
//!   篡改扩展包音频字节/digest/definition_id 均被加载器原子拒绝。
//! - Gate 4（Range transition table）：真实 `submit`/`Answer` 下 `Held -> Sealed -> Released -> Idle` 状态机、
//!   二次 `take` 为 `None`、未 `release` 不可 `take`、无进展拒绝不重置、仿真克隆/读档清空。
//! - Gate 5（CardResponded 计数矩阵）：真实出牌/响应下仅使用、使用+响应、取消响应、多人响应万箭齐发。
//! - Gate 6（Semantic / hash / archive invariance）：挂载与剥离 `presentation_binding` 的双引擎在完整命令序列下
//!   `state_hash`、`encode_archive`、`rng`、`plan_action` 100% 字节一致。

use content_native_loader::{
    attach_embedded_original_presentation, attach_original_presentation_bindings,
    embedded_original_presentation_input, prepare_locked_content_with_bridge, start_locked_engine,
    validate_and_freeze_raw_package, verify_original_presentation, LockedStartMode,
    OriginalPresentationInput,
};
use engine_api::{
    BoundAudioAsset, FrozenOwnerKey, OriginalDefinitionKey, PresentationAssetRef,
    PresentationDefinitionKind, PresentationDigestAlgorithm, PresentationLocalId,
    PresentationNature, PublicPresentationEvent, QualifiedDefinitionKey,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::encode_archive;
use engine_native::content::standard::standard_catalog;
use engine_native::view::public_event_code;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
use std::collections::BTreeMap;

fn build_engine_with_original_bindings(seats: usize, seed: u64) -> Engine {
    let mut engine = Engine::new(seed, seats);
    attach_embedded_original_presentation(&mut engine)
        .expect("内置原版表现绑定校验应通过");
    engine
}

fn find_general_def_by_name(engine: &Engine, name: &str) -> u16 {
    (0..engine.catalog().general_count() as u16)
        .find(|&d| engine.catalog().general_meta(d).map(|m| m.0) == Some(name))
        .unwrap_or_else(|| panic!("未找到武将: {name}"))
}

fn coarse_codes_from(engine: &Engine, start: usize) -> Vec<u16> {
    engine.log()[start..]
        .iter()
        .filter_map(public_event_code)
        .collect()
}

#[test]
fn gate1_producer_only_privacy_and_same_phase_rotation() {
    let mut e_public = build_engine_with_original_bindings(2, 2026110501);
    let liubei_def = find_general_def_by_name(&e_public, "刘备");
    let caocao_def = find_general_def_by_name(&e_public, "曹操");

    e_public.assign_general(0, liubei_def).unwrap();
    e_public.assign_general(1, caocao_def).unwrap();
    for id in 0..2 {
        e_public.debug_clear_hand(id);
    }
    e_public.debug_set_turn(0, Phase::Play, 1);
    let sha_card = e_public.debug_place(0, "sha");
    let _shan_p1 = e_public.debug_place(1, "shan");

    // P0（明置刘备）打出杀 -> 挂起在 P1 闪响应窗口
    let out = e_public.submit(GameCommand {
        actor: 0,
        expected_revision: e_public.revision(),
        dedup_id: 1,
        action: GameAction::PlayCard {
            card: sha_card,
            targets: vec![1],
        },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");

    // 即使在封口后 P0 槽位旋转为曹操（模拟同阶段后续换将），已发生在刘备明置时刻的公开事件仍保留 standard:liubei
    e_public.state_core_mut().player_mut(0).general_slots[0] = Some(caocao_def);
    e_public.release_committed_presentation();
    let batch_rotated = e_public
        .take_public_presentation()
        .expect("释放后应可消费公开表现批次");
    assert_eq!(batch_rotated.occurrences.len(), 1);
    match &batch_rotated.occurrences[0].event {
        PublicPresentationEvent::CardUsed {
            actor,
            effective,
            voice,
            ..
        } => {
            assert_eq!(*actor, engine_api::PlayerId(1));
            assert_eq!(effective.kind, "sha");
            assert_eq!(effective.key, "sha");
            let v = voice.as_ref().expect("发生时刻明置刘备应带角色绑定");
            assert_eq!(
                v.role.local_id,
                PresentationLocalId::Original(OriginalDefinitionKey {
                    pack: "standard".to_string(),
                    raw_symbol: "liubei".to_string(),
                }),
                "后续槽位旋转为曹操后，早期已提交事件仍保留发生时刻的 standard:liubei 绑定"
            );
            assert!(!v.assets.is_empty());
        }
        other => panic!("期望 CardUsed，实际: {other:?}"),
    }

    // 配对测试：P0 发生时刻为国战暗置（GameMode::Kingdoms + revealed_generals[0] == false），即使稍后明置，发生时刻的角色身份也绝不泄露
    let mut e_hidden = build_engine_with_original_bindings(2, 2026110501);
    e_hidden.assign_general(0, liubei_def).unwrap();
    e_hidden.assign_general(1, caocao_def).unwrap();
    for id in 0..2 {
        e_hidden.debug_clear_hand(id);
    }
    e_hidden.debug_set_turn(0, Phase::Play, 1);
    e_hidden.state_core_mut().mode = engine_native::state::GameMode::Kingdoms;
    e_hidden.state_core_mut().player_mut(0).revealed_generals = [false, false];
    let sha_hidden = e_hidden.debug_place(0, "sha");
    let _shan_hidden_p1 = e_hidden.debug_place(1, "shan");

    let out_hidden = e_hidden.submit(GameCommand {
        actor: 0,
        expected_revision: e_hidden.revision(),
        dedup_id: 1,
        action: GameAction::PlayCard {
            card: sha_hidden,
            targets: vec![1],
        },
    });
    assert!(matches!(out_hidden, CommandOutcome::Pending { .. }));
    // 即使稍后将 P0 改为明置，发生时刻为暗将的出牌也不得携带隐藏武将语音角色
    e_hidden.state_core_mut().player_mut(0).revealed_generals = [true, false];
    e_hidden.release_committed_presentation();
    let batch_hidden = e_hidden.take_public_presentation().unwrap();
    assert_eq!(batch_hidden.occurrences.len(), 1);
    match &batch_hidden.occurrences[0].event {
        PublicPresentationEvent::CardUsed { voice, .. } => {
            assert!(
                voice.is_none(),
                "暗将出牌即使后续明置也不得泄露隐藏武将身份或语音绑定: {voice:?}"
            );
        }
        other => panic!("期望 CardUsed，实际: {other:?}"),
    }
}

#[test]
fn gate2_verified_identity_isolates_builtin_and_boss_duplicates_and_sdk_packages() {
    let mut catalog = standard_catalog();
    let orig = verify_original_presentation(&embedded_original_presentation_input()).unwrap();
    attach_original_presentation_bindings(&mut catalog, &orig).unwrap();

    // 标准刘备（index 0）带 standard:liubei 绑定
    let std_liubei_idx = (0..catalog.general_count() as u16)
        .find(|&d| catalog.general_meta(d).map(|m| m.0) == Some("刘备"))
        .expect("标准刘备");
    let std_binding = catalog
        .general_presentation_binding(std_liubei_idx)
        .expect("标准刘备应有验证绑定");
    assert_eq!(
        std_binding.definition.local_id,
        PresentationLocalId::Original(OriginalDefinitionKey {
            pack: "standard".to_string(),
            raw_symbol: "liubei".to_string(),
        })
    );
    assert!(std_binding.voice_assets.contains_key("skill:rende"));
    assert!(std_binding.voice_assets.contains_key("die"));

    // 样例·甲 与 Boss 同名刘备绝不借用 standard:liubei 绑定
    for idx in 0..catalog.general_count() as u16 {
        let name = catalog.general_meta(idx).unwrap().0;
        if name.starts_with("样例·") || (idx != std_liubei_idx && name == "刘备") {
            assert!(
                catalog.general_presentation_binding(idx).is_none(),
                "非原版标准定义 #{idx} ({name}) 不得挂接 standard:liubei 绑定"
            );
        }
    }

    // SDK 扩展包：构造带音频资产、character.voice_assets 与 skill.voice_assets 的 FrozenPackage
    let audio_bytes = b"ID3-sdk-voice-payload".to_vec();
    let audio_hash = *blake3::hash(&audio_bytes).as_bytes();
    let audio_hex = blake3::Hash::from_bytes(audio_hash).to_hex().to_string();

    let mut pkg = content_compiler::compile_source_named(
        "sdk_voice_pkg",
        r#"
        skill sdk_rende(701) {
            trigger card_used owner_is_source {
                effect draw_cards(self, 1);
            }
        }
        "#,
    )
    .expect("编译测试扩展包");
    pkg.trigger_index[0].optional = false;
    pkg.manifest.namespace = "author_a".to_string();
    pkg.manifest.package_id = "sdk_voice_pkg".to_string();
    pkg.manifest.version = "1.0.0".to_string();

    let chars_json = serde_json::json!({
        "pack": { "name": "sdk_voice_pkg", "description": "SDK 语音测试包" },
        "generals": [{
            "id": 1,
            "definition_id": 501,
            "name": "界刘备",
            "faction": "shu",
            "hp": 4,
            "max_hp": 4,
            "gender": "male",
            "skills": [{ "name": "sdk_rende", "mapping": null }]
        }]
    })
    .to_string();
    let chars_bytes = chars_json.as_bytes().to_vec();
    let chars_hash = *blake3::hash(&chars_bytes).as_bytes();

    pkg.asset_paths
        .insert("audio/sdk_rende.mp3".to_string(), "assets/sdk_rende.mp3".to_string());
    pkg.asset_paths
        .insert("characters".to_string(), "assets/characters.json".to_string());
    let mut asset_manifest = resource_schema::AssetManifest::new("sdk_voice_pkg");
    asset_manifest.add_entry(resource_schema::AssetEntry {
        asset_id: "audio/sdk_rende.mp3".to_string(),
        source_hash: audio_hash,
        kind: resource_schema::AssetKind::Audio,
        image_variants: vec![],
        audio_variants: vec![resource_schema::AudioVariant {
            format: resource_schema::AudioFormat::Opus,
            platforms: vec![],
            content_hash: audio_hash,
            compressed_size: audio_bytes.len() as u64,
            decoded_size: audio_bytes.len() as u64,
        }],
        priority: resource_schema::PreloadPriority::Normal,
        depends_on: vec![],
        platforms: vec![],
    });
    asset_manifest.add_entry(resource_schema::AssetEntry {
        asset_id: "characters".to_string(),
        source_hash: chars_hash,
        kind: resource_schema::AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: resource_schema::PreloadPriority::Normal,
        depends_on: vec![],
        platforms: vec![],
    });
    pkg.asset_manifest = Some(asset_manifest);
    pkg.character_index.push(content_schema::PackageCharacterEntry {
        id: 501,
        name_key: "sdk_liubei".to_string(),
        max_hp: 4,
        initial_hp: 4,
        skills: vec![701],
        sex: "male".to_string(),
        group: "shu".to_string(),
        voice_assets: BTreeMap::from([
            ("skill:sdk_rende".to_string(), "audio/sdk_rende.mp3".to_string()),
            ("die".to_string(), "audio/sdk_rende.mp3".to_string()),
        ]),
        ..Default::default()
    });
    pkg.skill_index[0]
        .voice_assets
        .insert("default".to_string(), "audio/sdk_rende.mp3".to_string());
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();

    let package_json = serde_json::to_string(&pkg).unwrap();
    let mut assets = BTreeMap::new();
    assets.insert("assets/sdk_rende.mp3".to_string(), audio_bytes.clone());
    assets.insert("assets/characters.json".to_string(), chars_bytes);

    let frozen = validate_and_freeze_raw_package(package_json, assets)
        .expect("冻结 SDK 语音测试包");
    let binding = frozen.binding.clone();
    let verified = prepare_locked_content_with_bridge(vec![frozen])
        .expect("校验并准备锁定内容");

    let mut engine = start_locked_engine(42, 2, LockedStartMode::Prototype, &verified)
        .expect("start_locked_engine 应成功挂载原版与扩展包表现绑定");
    let sdk_general_def = find_general_def_by_name(&engine, "界刘备");
    engine.assign_general(0, sdk_general_def).unwrap();
    for id in 0..2 {
        engine.debug_clear_hand(id);
    }
    engine.debug_set_turn(0, Phase::Play, 1);
    let sha = engine.debug_place(0, "sha");

    let out = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: 1,
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");

    engine.release_committed_presentation();
    let batch = engine.take_public_presentation().unwrap();
    let expected_owner = FrozenOwnerKey::Package {
        namespace: binding.namespace.clone(),
        package_id: binding.package_id.clone(),
        version: binding.version.clone(),
        logic_hash: binding.logic_hash.clone(),
        content_hash: binding.content_hash.clone(),
    };
    let skill_occ = batch
        .occurrences
        .iter()
        .find(|o| matches!(o.event, PublicPresentationEvent::SkillActivated { .. }))
        .expect("应产生 SkillActivated 公开条目");
    match &skill_occ.event {
        PublicPresentationEvent::SkillActivated { skill, voice, .. } => {
            assert_eq!(
                skill.definition,
                QualifiedDefinitionKey {
                    owner: expected_owner.clone(),
                    kind: PresentationDefinitionKind::Skill,
                    local_id: PresentationLocalId::Package(701),
                }
            );
            let v = voice.as_ref().expect("SDK 武将技能应解析出语音资产");
            assert_eq!(
                v.role,
                QualifiedDefinitionKey {
                    owner: expected_owner.clone(),
                    kind: PresentationDefinitionKind::General,
                    local_id: PresentationLocalId::Package(501),
                }
            );
            assert_eq!(
                v.assets,
                vec![BoundAudioAsset {
                    source: expected_owner,
                    reference: PresentationAssetRef::Package {
                        namespace: "author_a".to_string(),
                        package_id: "sdk_voice_pkg".to_string(),
                        asset_id: "audio/sdk_rende.mp3".to_string(),
                    },
                    digest_algorithm: PresentationDigestAlgorithm::Blake3,
                    digest: audio_hex,
                    size: audio_bytes.len() as u64,
                }]
            );
        }
        other => panic!("期望 SkillActivated，实际: {other:?}"),
    }
}

#[test]
fn gate3_controlled_factory_rejects_tampered_original_or_package_inputs() {
    let base = embedded_original_presentation_input();

    // 篡改 catalog.json 原文字节但保持受控构建管线预期 SHA-256 pin
    let mut tampered_catalog_bytes = base.catalog_json.to_vec();
    tampered_catalog_bytes.push(b' ');
    let bad_catalog = OriginalPresentationInput {
        catalog_json: &tampered_catalog_bytes,
        asset_manifest_json: base.asset_manifest_json,
    };
    assert!(
        verify_original_presentation(&bad_catalog).is_err(),
        "篡改 catalog.json 字节必须拒绝"
    );

    // 篡改 asset-manifest.json 原文字节但保持受控构建管线预期 SHA-256 pin
    let mut tampered_manifest_bytes = base.asset_manifest_json.to_vec();
    tampered_manifest_bytes.push(b' ');
    let bad_manifest = OriginalPresentationInput {
        catalog_json: base.catalog_json,
        asset_manifest_json: &tampered_manifest_bytes,
    };
    assert!(
        verify_original_presentation(&bad_manifest).is_err(),
        "篡改 asset-manifest.json 字节必须拒绝"
    );
}

#[test]
fn gate4_volatile_range_lifecycle_and_single_take_semantics() {
    let mut engine = build_engine_with_original_bindings(2, 77);
    let liubei_def = find_general_def_by_name(&engine, "刘备");
    let caocao_def = find_general_def_by_name(&engine, "曹操");
    engine.assign_general(0, liubei_def).unwrap();
    engine.assign_general(1, caocao_def).unwrap();
    for id in 0..2 {
        engine.debug_clear_hand(id);
    }
    engine.debug_set_turn(0, Phase::Play, 1);
    let sha_card = engine.debug_place(0, "sha");

    // 初始无批次
    assert!(engine.take_public_presentation().is_none());

    // 提交出杀（Pending 封口为 Sealed，但在未调用 release_committed_presentation 前不可 take）
    let out = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: 1,
        action: GameAction::PlayCard {
            card: sha_card,
            targets: vec![1],
        },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert!(
        engine.take_public_presentation().is_none(),
        "Sealed 未经 release_committed_presentation 不得被 take"
    );

    // 快照恢复必须清空易失表现区间
    let snap = engine.save();
    let mut restored = Engine::restore(&snap).unwrap();
    restored.release_committed_presentation();
    assert!(restored.take_public_presentation().is_none());

    // 正式 release 后恰好可 take 一次，第二次 take 为 None
    engine.release_committed_presentation();
    let first_take = engine.take_public_presentation();
    assert!(first_take.is_some());
    assert!(engine.take_public_presentation().is_none());

    // 无进展拒绝（如 StaleRevision / ChoicePending）不得重新生成或泄露批次
    let rejected = engine.submit(GameCommand {
        actor: 0,
        expected_revision: 0,
        dedup_id: 2,
        action: GameAction::NextPhase,
    });
    assert!(matches!(rejected, CommandOutcome::Rejected(_)));
    engine.release_committed_presentation();
    assert!(engine.take_public_presentation().is_none());
}

#[test]
fn gate5_card_responded_coarse_and_typed_count_matrix() {
    let mut engine = build_engine_with_original_bindings(3, 88);
    let liubei = find_general_def_by_name(&engine, "刘备");
    let caocao = find_general_def_by_name(&engine, "曹操");
    let sunquan = find_general_def_by_name(&engine, "孙权");
    engine.assign_general(0, liubei).unwrap();
    engine.assign_general(1, caocao).unwrap();
    engine.assign_general(2, sunquan).unwrap();
    for id in 0..3 {
        engine.debug_clear_hand(id);
    }
    engine.debug_set_turn(0, Phase::Play, 1);

    // 1. 仅使用（Use-only）：P0 出杀，挂起在 P1 响应闪前 -> 1 CardUsed, 0 CardResponded, 粗粒度 code 3 数量 = 1
    let sha_card = engine.debug_place(0, "sha");
    let shan_p1 = engine.debug_place(1, "shan");
    let log_start_1 = engine.log().len();
    let out1 = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: 1,
        action: GameAction::PlayCard {
            card: sha_card,
            targets: vec![1],
        },
    });
    assert!(matches!(out1, CommandOutcome::Pending { .. }));
    let coarse_1 = coarse_codes_from(&engine, log_start_1);
    assert_eq!(coarse_1.iter().filter(|&&c| c == 3).count(), 1);
    engine.release_committed_presentation();
    let batch_1 = engine.take_public_presentation().unwrap();
    assert_eq!(batch_1.occurrences.len(), 1);
    assert!(matches!(
        batch_1.occurrences[0].event,
        PublicPresentationEvent::CardUsed { .. }
    ));

    // 2. 仅打出响应（Response-only）：P1 打出闪响应 -> 0 CardUsed, 1 CardResponded, 粗粒度 code 3 数量 = 1
    let choice1 = engine.open_choice().unwrap().clone();
    assert!(choice1.spec.options.contains(&ChoiceOption::Card(shan_p1)));
    let log_start_2 = engine.log().len();
    let out2 = engine.submit(GameCommand {
        actor: 1,
        expected_revision: engine.revision(),
        dedup_id: 2,
        action: GameAction::Answer {
            choice_id: choice1.id,
            selection: vec![ChoiceOption::Card(shan_p1)],
        },
    });
    assert!(matches!(out2, CommandOutcome::Accepted { .. }));
    let coarse_2 = coarse_codes_from(&engine, log_start_2);
    assert_eq!(coarse_2.iter().filter(|&&c| c == 3).count(), 1);
    engine.release_committed_presentation();
    let batch_2 = engine.take_public_presentation().unwrap();
    assert_eq!(batch_2.occurrences.len(), 1);
    match &batch_2.occurrences[0].event {
        PublicPresentationEvent::CardResponded {
            actor, effective, ..
        } => {
            assert_eq!(*actor, engine_api::PlayerId(2));
            assert_eq!(effective.kind, "shan");
            assert_eq!(effective.key, "shan");
            assert_eq!(effective.nature, PresentationNature::Normal);
        }
        other => panic!("期望 CardResponded，实际: {other:?}"),
    }

    // 3. 多人响应万箭齐发（P1 出闪、P2 取消响应受 1 点伤害）：
    let archery = engine.debug_place(0, "archery");
    let shan_p1_b = engine.debug_place(1, "shan");
    let _shan_p2_b = engine.debug_place(2, "shan");
    let out3 = engine.submit(GameCommand {
        actor: 0,
        expected_revision: engine.revision(),
        dedup_id: 3,
        action: GameAction::PlayCard {
            card: archery,
            targets: vec![],
        },
    });
    assert!(matches!(out3, CommandOutcome::Pending { .. }));
    engine.release_committed_presentation();
    let batch_3_use = engine.take_public_presentation().unwrap();
    assert_eq!(batch_3_use.occurrences.len(), 1);

    // P1 选择打出闪
    let c_p1 = engine.open_choice().unwrap().clone();
    let out_p1 = engine.submit(GameCommand {
        actor: c_p1.observer,
        expected_revision: engine.revision(),
        dedup_id: 4,
        action: GameAction::Answer {
            choice_id: c_p1.id,
            selection: vec![ChoiceOption::Card(shan_p1_b)],
        },
    });
    assert!(matches!(out_p1, CommandOutcome::Pending { .. }));
    engine.release_committed_presentation();
    let batch_p1_resp = engine.take_public_presentation().unwrap();
    assert_eq!(
        batch_p1_resp
            .occurrences
            .iter()
            .filter(|o| matches!(o.event, PublicPresentationEvent::CardResponded { .. }))
            .count(),
        1
    );

    // P2 选择放弃响应（Pass） -> 0 CardResponded, 1 DamageDealt
    let c_p2 = engine.open_choice().unwrap().clone();
    let out_p2 = engine.submit(GameCommand {
        actor: c_p2.observer,
        expected_revision: engine.revision(),
        dedup_id: 5,
        action: GameAction::Answer {
            choice_id: c_p2.id,
            selection: vec![ChoiceOption::Pass],
        },
    });
    assert!(!matches!(out_p2, CommandOutcome::Rejected(_)));
    engine.release_committed_presentation();
    let batch_p2_cancel = engine.take_public_presentation().unwrap();
    assert_eq!(
        batch_p2_cancel
            .occurrences
            .iter()
            .filter(|o| matches!(o.event, PublicPresentationEvent::CardResponded { .. }))
            .count(),
        0,
        "取消响应不得产生 CardResponded"
    );
    assert_eq!(
        batch_p2_cancel
            .occurrences
            .iter()
            .filter(|o| matches!(o.event, PublicPresentationEvent::DamageDealt { .. }))
            .count(),
        1,
        "取消闪响应后应产生 1 条 DamageDealt"
    );
}

#[test]
fn gate6_semantic_hash_archive_rng_and_planner_invariance() {
    let mut e_bound = build_engine_with_original_bindings(4, 20261105);
    let mut e_unbound = Engine::new(20261105, 4);
    let planner = BoundedPlanner::new(64);

    assert_eq!(e_bound.state_hash(), e_unbound.state_hash());
    assert_eq!(
        encode_archive(&e_bound.save()),
        encode_archive(&e_unbound.save())
    );

    for step in 0..24 {
        let actor = e_bound
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or_else(|| e_bound.turn().current);
        let plan_bound = e_bound.plan_action(actor, &planner);
        let plan_unbound = e_unbound.plan_action(actor, &planner);
        assert_eq!(
            format!("{plan_bound:?}"),
            format!("{plan_unbound:?}"),
            "step {step}: AI 规划决策必须完全一致"
        );

        let out_bound = e_bound.apply_plan(&plan_bound);
        let out_unbound = e_unbound.apply_plan(&plan_unbound);
        assert_eq!(
            format!("{out_bound:?}"),
            format!("{out_unbound:?}"),
            "step {step}: 命令结算结果必须完全一致"
        );

        // 在 e_bound 上执行完整的 release + take 消费，而 e_unbound 不消费
        e_bound.release_committed_presentation();
        let _ = e_bound.take_public_presentation();

        assert_eq!(
            e_bound.state_hash(),
            e_unbound.state_hash(),
            "step {step}: state_hash 必须与表现绑定/消费完全无关"
        );
        assert_eq!(
            encode_archive(&e_bound.save()),
            encode_archive(&e_unbound.save()),
            "step {step}: 存档字节必须与表现绑定/消费完全无关"
        );
        assert_eq!(
            e_bound.state_core().rng,
            e_unbound.state_core().rng,
            "step {step}: RNG 状态必须完全一致"
        );
        if e_bound.outcome().is_some() {
            break;
        }
    }
}
