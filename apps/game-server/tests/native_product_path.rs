//! T6③ 产品路径回归：安装包 → 装载 → 原生建局 → 有界 AI 驱动。
//!
//! 全链路在服务端同进程内验证（HTTP 层薄封装，逻辑在
//! `src/native.rs`）：content-runtime 安装 → content-native-loader
//! 桥接 → engine-native 建局 → BoundedPlanner 驱动（T13 衔接）。

#[path = "../src/native.rs"]
mod native;

use native::{
    create_native_game, restore_match, save_fresh_match, NativeContent, NativeGameRequest,
};
use std::path::PathBuf;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 全链路：安装欢乐卡牌 → 装载 → 带扩展建局 → AI 驱动推进。
#[test]
fn t63_import_load_build_and_drive_with_extension() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let installed = content.import().expect("安装包");
    assert!(
        installed.iter().any(|p| p.package_id == "huanle-cards"),
        "欢乐卡牌应安装"
    );
    let listed = content.installed().expect("列出已安装");
    assert!(
        listed.iter().any(|p| p.package_id == "huanle-cards"),
        "存储应能列出已安装包"
    );

    let loaded = content.load("huanle-cards").expect("装载");
    assert_eq!(
        loaded.pack.card_count(),
        7,
        "与 G1 装载口径一致（含伴随技）"
    );

    let req = NativeGameRequest {
        packages: vec!["huanle-cards".into()],
        generals: Vec::new(),
        seed: 7301,
        seats: 3,
        max_steps: 600,
        planner_nodes: 64,
        planner_lookahead: 1,
        planner_variant: 0,
    };
    let summary = create_native_game(&content, &req).expect("建局+驱动");
    assert_eq!(
        summary.catalog_cards,
        summary.base_catalog_cards + 7,
        "扩展牌应并入目录"
    );
    assert!(summary.steps > 0, "驱动器应有推进");
    assert!(
        !summary.stalled,
        "驱动器不得停滞（steps={} finished={}）",
        summary.steps, summary.finished
    );
    println!(
        "T6③ 带扩展实测: steps={} finished={} winner={:?} elapsed_ms={}",
        summary.steps, summary.finished, summary.winner, summary.elapsed_ms
    );
}

/// T13-NATIVE-GENERALS-PRODUCT（武将包产品面）：玩点论杀武将包经
/// `/native` 产品路径装载——generals 面并入目录，建局与目录计数可见。
#[test]
fn t13_generals_package_loads_via_native_path() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let _ = content.import().expect("安装包");
    let listed = content.installed().expect("列出已安装");
    assert!(
        listed.iter().any(|p| p.package_id == "wudian"),
        "玩点论杀武将包应可发现"
    );

    let loaded = content.load("wudian").expect("装载武将包");
    assert_eq!(loaded.pack.generals.len(), 9, "九名武将与 G1 装载口径一致");
    assert_eq!(loaded.pack.card_count(), 14, "14 张实体牌与 G1 口径一致");

    let req = NativeGameRequest {
        packages: vec!["wudian".into()],
        generals: Vec::new(),
        seed: 4242,
        seats: 4,
        max_steps: 0,
        planner_nodes: 0,
        planner_lookahead: 1,
        planner_variant: 0,
    };
    let summary = create_native_game(&content, &req).expect("武将包建局");
    assert_eq!(summary.packages, vec!["wudian".to_string()]);
    assert_eq!(
        summary.catalog_generals,
        summary.base_catalog_generals + 9,
        "武将包九名武将并入目录"
    );
    assert_eq!(
        summary.catalog_cards,
        summary.base_catalog_cards + 14,
        "武将包 14 张实体牌并入目录"
    );
}

/// T8 增量 4：drop-in 编译包（package.npack.json 含 rule-vm 模块）→
/// 安装 → 装载器 bridge_vm_skill_decls → 建局 → 触发落账——不重编译
/// 安装含新技能的包（C1 数据流的服务端磁盘路径）。
#[test]
fn t8_native_disk_path_vm_package_end_to_end() {
    const JIUWEI_DSL: &str = "\
skill hnl_jiuwei_t8(990001) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pkg = content_compiler::compile_source_named("t8-disk-vm", JIUWEI_DSL).expect("DSL 编译");
    let npack_json = serde_json::to_string(&pkg).expect("npack 序列化");

    // drop-in 目录：仅 package.npack.json（vm-only 包，无 cards.json）
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("t8-vm-pack");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), npack_json).unwrap();

    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    let installed = content.import().expect("安装编译包");
    assert!(
        installed.iter().any(|p| p.package_id == "t8-disk-vm"),
        "编译包应安装"
    );

    let loaded = content.load("t8-disk-vm").expect("装载");
    assert_eq!(loaded.pack.vm_skills.len(), 1, "VM 声明经磁盘装载入包");
    assert_eq!(loaded.pack.vm_skills[0].name, "hnl_jiuwei_t8_phase_end");

    // 建局 → 授技 → 推进到持有者阶段结束：VM 效果落账（hp 2→3）
    let mut e =
        engine_native::Engine::new_with_extensions(920001, 4, &[&loaded.pack]).expect("建局");
    e.debug_grant(1, "hnl_jiuwei_t8_phase_end");
    e.debug_set_hp(1, 2);
    let mut converged = false;
    for _ in 0..40 {
        if e.turn().current == 1 && e.debug_hp(1) == 3 {
            converged = true;
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        let actor = e.turn().current;
        submit_next_phase(&mut e, actor);
        answer_pending(&mut e);
    }
    assert!(converged, "VM 技能：服务端磁盘路径阶段结束受伤回 1");
}

fn submit_next_phase(e: &mut engine_native::Engine, actor: u8) {
    let rev = e.revision();
    let _ = e.submit(engine_native::GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action: engine_native::GameAction::NextPhase,
    });
}

fn answer_pending(e: &mut engine_native::Engine) {
    if let Some(c) = e.open_choice() {
        let sel: Vec<engine_native::ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, engine_native::ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        let selection = if sel.is_empty() && !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            sel
        };
        let rev = e.revision();
        let _ = e.submit(engine_native::GameCommand {
            actor: obs,
            expected_revision: rev,
            dedup_id: (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64)
                | 1,
            action: engine_native::GameAction::Answer {
                choice_id: cid,
                selection,
            },
        });
    }
}

/// 纯标准目录建局 + 驱动（无扩展包的产品路径基线）。
#[test]
fn t63_standard_only_build_and_drive() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let req = NativeGameRequest {
        packages: vec![],
        generals: Vec::new(),
        seed: 7302,
        seats: 3,
        max_steps: 300,
        planner_nodes: 64,
        planner_lookahead: 1,
        planner_variant: 0,
    };
    let summary = create_native_game(&content, &req).expect("建局+驱动");
    assert_eq!(
        summary.catalog_cards, summary.base_catalog_cards,
        "无扩展时目录等于标准基线"
    );
    assert!(summary.steps > 0);
    assert!(!summary.stalled, "标准对局驱动器不得停滞");
    println!(
        "T6③ 标准实测: steps={} finished={} elapsed_ms={}",
        summary.steps, summary.finished, summary.elapsed_ms
    );
}

/// 已安装扩展进入新对局，保存后再用同一包恢复；缺包与另一包集合被拒绝。
#[test]
fn t13_installed_extension_enters_match_and_restores() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let packages = vec!["huanle-cards".to_string()];
    let (bytes, cards) = save_fresh_match(&content, &packages, 17017, 4).expect("新对局");
    let base = engine_native::Engine::new(1, 4).catalog().cards.len();
    assert_eq!(cards, base + 7, "欢乐卡牌 7 张进入新对局");

    let restored = restore_match(&content, &packages, &bytes).expect("同包恢复");
    assert_eq!(restored, cards, "恢复后目录牌数不变");

    let missing = restore_match(&content, &[], &bytes);
    match missing {
        Err(msg) => assert!(msg.contains("恢复失败"), "缺包必须拒绝: {msg}"),
        Ok(n) => panic!("缺包必须拒绝，得到 {n}"),
    }

    let other = restore_match(&content, &["wuxing".into()], &bytes);
    match other {
        Err(msg) => assert!(
            msg.contains("恢复失败"),
            "另一已安装包必须因内容锁拒绝: {msg}"
        ),
        Ok(n) => panic!("另一已安装包必须拒绝，得到 {n}"),
    }

    let unknown = restore_match(&content, &["not-installed".into()], &bytes);
    match unknown {
        Err(msg) => assert!(msg.contains("包未安装"), "{msg}"),
        Ok(n) => panic!("未安装包必须拒绝，得到 {n}"),
    }
}

/// 未安装包装载必须可处理报错（不 panic）。
#[test]
fn t63_load_unknown_package_errors_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    let err = match content.load("no-such-package") {
        Err(err) => err,
        Ok(_) => panic!("未安装包装载应报错"),
    };
    assert!(err.contains("包未安装"), "错误应指明未安装: {err}");
}

#[test]
fn native_character_only_installed_package_enters_match() {
    let dir = tempfile::tempdir().unwrap();
    let content = NativeContent::open(repo_packages_root(), dir.path().join("store")).unwrap();
    content.import().unwrap();
    let loaded = content
        .load("d3-select")
        .expect("仅 character.json 的正式包");
    assert_eq!(loaded.pack.generals.len(), 1);
    assert_eq!(loaded.pack.host_skills.len(), 2);
    assert!(loaded.pack.cards.is_empty());
    let mut engine = engine_native::Engine::new_with_extensions(4242, 4, &[&loaded.pack]).unwrap();
    let name = loaded.pack.generals[0].name;
    engine
        .assign_general_named(0, name)
        .expect("正式武将可选入局");
    let slot = engine.state_core().players[0].general_slots[0].unwrap();
    assert_eq!(engine.catalog().general_meta(slot).unwrap().0, name);
    assert_eq!(engine.state_core().players[0].hp, 4);
}

#[test]
fn native_pile_only_installed_package_adds_physical_draw_cards() {
    let dir = tempfile::tempdir().unwrap();
    let content = NativeContent::open(repo_packages_root(), dir.path().join("store")).unwrap();
    content.import().unwrap();
    let json = std::fs::read_to_string(
        repo_packages_root().join("cardpile-upstream-noname/assets/pile.json"),
    )
    .unwrap();
    let spec = content_native_loader::bridge_pile(&json).unwrap();
    let expected: usize = spec
        .entries
        .iter()
        .flat_map(|entry| entry.counts.iter().map(|(_, count)| *count as usize))
        .sum();
    assert!(
        content.load("cardpile").is_err(),
        "双来源同 id 必须要求命名空间"
    );
    let loaded = content
        .load_scoped(Some("upstream-noname"), "cardpile")
        .expect("仅 pile.json 的正式包");
    assert_eq!(loaded.pack.cards.len(), expected);
    assert!(expected > 0);
    let base = engine_native::Engine::new(4242, 4).catalog().cards.len();
    let engine = engine_native::Engine::new_with_extensions(4242, 4, &[&loaded.pack]).unwrap();
    let physical = engine.state_core().deck.iter().chain(
        engine
            .state_core()
            .players
            .iter()
            .flat_map(|player| player.hand.iter()),
    );
    assert_eq!(
        physical.filter(|card| **card as usize >= base).count(),
        expected,
        "扩展实体牌必须进入可抽取牌堆或已发手牌，而不只加入目录"
    );
    let reference = format!("{}@{}", loaded.pack.def.name, loaded.pack.def.version);
    let snapshot = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(
        &engine.save(),
    ))
    .unwrap();
    let reloaded = content.load_reference(&reference).unwrap();
    let restored = engine_native::Engine::restore_with_extensions(&snapshot, &[&reloaded]).unwrap();
    assert_eq!(restored.state_hash(), engine.state_hash());
    assert_eq!(restored.catalog().cards.len(), base + expected);
}

#[test]
fn native_installed_asset_tampering_rejects_load_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("store");
    let content = NativeContent::open(repo_packages_root(), store.clone()).unwrap();
    content.import().unwrap();
    let packages = vec!["huanle-cards".to_string()];
    let (bytes, _) = save_fresh_match(&content, &packages, 4242, 4).unwrap();
    let loaded = content.load("huanle-cards").unwrap();
    let reference = format!("{}@{}", loaded.pack.def.name, loaded.pack.def.version);
    let asset = store.join("packages/upstream-noname__huanle-cards/1.0.0/assets/cards.json");
    let mut data = std::fs::read(&asset).unwrap();
    data.push(b'\n');
    std::fs::write(&asset, data).unwrap();
    let error = content
        .load("huanle-cards")
        .err()
        .expect("篡改资产必须拒绝");
    assert!(error.contains("hash 不匹配"), "{error}");
    assert!(
        content.load_reference(&reference).is_err(),
        "房间内容锁不能绕过资产校验"
    );
    assert!(
        restore_match(&content, &packages, &bytes).is_err(),
        "存档恢复不能绕过资产校验"
    );
}

/// 服务端正式安装路径：npack（模块）+ `assets/character.json`（武将引用 VM 模块）
/// → 安装 → 桥接 → 建局 → 选将 → VM 技能真实触发。
#[test]
fn vm_package_with_characters_installs_loads_and_runs() {
    const DSL: &str = "\
skill vm_probe(990003) {
  trigger phase_end owner_is_target {
    effect recover_hp(self, 1);
  }
}
";
    let pkg = content_compiler::compile_source_named("server-vm-char", DSL).expect("compile");
    let characters = serde_json::json!({
            "source": "test",
            "upstream_path": "server-vm-char",
            "upstream_commit": "0000000000000000000000000000000000000000",
            "pack": { "name": "server-vm-char", "version": 1, "description": "test" },
            "generals": [{
                "id": "vm_probe",
                "name": "vm_probe",
                "faction": "qun",
                "hp": 4,
                "gender": "male",
                "skills": [{ "name": "vm_probe_phase_end", "title": "探针" }]
            }]
        })
        .to_string();
    let character_bytes = characters.as_bytes();
    let mut npack = serde_json::to_value(&pkg).unwrap();
    npack["asset_manifest"] = serde_json::json!({
        "schema_version": 1,
        "package_id": "server-vm-char",
        "total_compressed_size": 0,
        "total_decoded_size": 0,
        "entries": [{
            "asset_id": "server-vm-char:characters",
            "source_hash": blake3::hash(character_bytes).as_bytes(),
            "kind": "Data", "image_variants": [], "audio_variants": [],
            "priority": "Lazy", "depends_on": [], "platforms": []
        }]
    });
    npack["asset_paths"] = serde_json::json!({
        "server-vm-char:characters": "assets/character.json"
    });
    let mut package: content_schema::ContentPackage = serde_json::from_value(npack).unwrap();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();

    // 真实安装副本的资产哈希也要匹配：只改变引用名并重算资产哈希。
    let bad_characters = characters.replace("vm_probe_phase_end", "missing_vm_skill");
    let mut bad_package = serde_json::to_value(&package).unwrap();
    bad_package["asset_manifest"]["entries"][0]["source_hash"] =
        serde_json::json!(blake3::hash(bad_characters.as_bytes()).as_bytes());
    let mut bad_package: content_schema::ContentPackage = serde_json::from_value(bad_package).unwrap();
    bad_package.manifest.asset_hash = bad_package.compute_asset_hash();
    let bad_dir = tempfile::tempdir().unwrap();
    let bad_dropin = bad_dir.path().join("server-vm-char");
    std::fs::create_dir_all(bad_dropin.join("assets")).unwrap();
    std::fs::write(bad_dropin.join("package.npack.json"), bad_package.to_json_bytes().unwrap()).unwrap();
    std::fs::write(bad_dropin.join("assets/character.json"), bad_characters).unwrap();
    let bad_content = NativeContent::open(bad_dir.path().to_path_buf(), bad_dir.path().join("store")).unwrap();
    bad_content.import().expect("验证过的包可安装");
    let error = bad_content.load("server-vm-char").err().expect("未解析技能必须拒绝装载");
    assert!(error.contains("missing_vm_skill"), "{error}");

    let npack = package.to_json_bytes().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("server-vm-char");
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    std::fs::write(dropin.join("package.npack.json"), &npack).unwrap();
    std::fs::write(dropin.join("assets/character.json"), character_bytes).unwrap();

    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    let installed = content.import().expect("安装");
    assert!(installed.iter().any(|p| p.package_id == "server-vm-char"));
    let loaded = content.load("server-vm-char").expect("装载");
    assert_eq!(loaded.pack.vm_skills.len(), 1);
    assert_eq!(loaded.pack.generals.len(), 1);
    assert_eq!(
        loaded.pack.generals[0].skills,
        vec![engine_native::content::names::intern("vm_probe_phase_end".to_string())]
    );

    let mut engine =
        engine_native::Engine::new_with_extensions(990_003, 4, &[&loaded.pack]).expect("建局");
    engine
        .assign_general_named(0, "vm_probe")
        .expect("选将");
    assert!(engine.view_for(0).players[0].skills.contains(&"vm_probe_phase_end"));
    engine.debug_set_hp(0, 3);
    let mut restored = engine_native::Engine::restore_with_extensions(&engine.save(), &[&loaded.pack])
        .expect("按原包恢复");
    for branch in [&mut engine, &mut restored] {
        advance_until_vm_recovery(branch);
    }
    assert_eq!(engine.state_hash(), restored.state_hash());
}

fn advance_until_vm_recovery(engine: &mut engine_native::Engine) {
    for step in 0..80u64 {
        if engine.view_for(0).players[0].hp == 4 {
            assert_eq!(engine.turn().current, 0);
            assert_eq!(engine.turn().phase, engine_native::Phase::End);
            return;
        }
        if engine.outcome().is_some() {
            break;
        }
        if let Some(choice) = engine.open_choice() {
            let selection = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min as usize)
                .cloned()
                .collect();
            let actor = choice.observer;
            let choice_id = choice.id;
            let result = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 30_000 + step,
                action: engine_native::GameAction::Answer {
                    choice_id,
                    selection,
                },
            });
            assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)), "{result:?}");
        } else {
            let actor = engine.turn().current;
            let result = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 30_000 + step,
                action: engine_native::GameAction::NextPhase,
            });
            assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)), "{result:?}");
        }
    }
    panic!("服务端安装的 VM 武将技能未在结束阶段回复 1 点体力");
}
