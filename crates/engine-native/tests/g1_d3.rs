//! G1 3D精选纵向样本（ADR 0776）：角色包管线（catalog 扩展武将）→
//! 精彀宿主行为（触发 + 目标选择 + 伤害）→ 内容锁。
//!
//! 切片：刘冲（群/4血/男，技能 精彀+末焱 @ 5c02a745）。精彀映射为
//! 宿主原语 max_range_weapon_damage；末焱（展示牌/获得/标记/伤害增幅）
//! 需更多宿主原语——包数据中登记 mapping:null，测试断言其暂缓状态。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::catalog::{CardDef, Faction};
use engine_native::content::standard::Gender;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, GameMode, Phase,
    SnapshotError,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 桥接统一走 C0 装载器（ADR 0777/0778）：武将元数据与宿主技能映射
/// 来自 character.json 的 pack 段与数据键；末焱暂缓计数经伴随装载入口
/// 读取，测试只校验包元数据一致。
fn bridge(pack_name: &'static str, json: &str) -> (ExtensionPack, Vec<String>) {
    let out =
        content_native_loader::bridge_pack(json).expect("character.json 可装载");
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    let generals = content_native_loader::bridge_generals(json).expect("generals 可装载");
    let unmapped: Vec<String> = {
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        let mut names = Vec::new();
        for g in parsed["generals"].as_array().unwrap() {
            for sk in g["skills"].as_array().unwrap() {
                if sk["mapping"].is_null() {
                    names.push(sk["name"].as_str().unwrap().to_string());
                }
            }
        }
        assert_eq!(generals.unmapped_skills, names.len(), "暂缓计数一致");
        names
    };
    (out.pack, unmapped)
}

/// G1 纵向：发现 → 导入 → 桥接（武将元数据 + 宿主技能；末焱暂缓登记）。
#[test]
fn g1_d3_install_bridge_and_metadata() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "d3-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "d3-select")
        .expect("3D精选安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/character.json"),
    )
    .unwrap();
    let (pack, unmapped) = bridge("d3-upstream-noname", &json);
    assert_eq!(unmapped, vec!["dddmoyan".to_string()], "末焱登记暂缓");
    assert_eq!(pack.general_count(), 1);
    assert_eq!(pack.host_skills.len(), 1);

    // 目录元数据：扩展武将接在基础表之后
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let e = Engine::new_with_extensions(3401, 3, &[&pack]).expect("建局");
    let (name, faction, skills, hp, gender) = e
        .catalog()
        .general_meta(base_count as u16)
        .expect("扩展武将在目录");
    assert_eq!(name, "ddd刘冲");
    assert_eq!(faction, Faction::Qun);
    assert_eq!(skills, vec!["dddjinggou"]);
    assert_eq!(hp, 4);
    assert_eq!(gender, Gender::Male);
    // boss 内置段（v221 起）位于扩展段后——越界以动态总尾为准
    assert!(
        e.catalog()
            .general_meta(e.catalog().general_count() as u16)
            .is_none(),
        "越界 None"
    );
}

/// G1 纵向：明置扩展武将（国战）→ 势力/体力来自包元数据；
/// 越界武将槽恢复拒绝；精彀宿主行为生效并支持字节恢复。
#[test]
fn g1_d3_vertical_reveal_jinggou_and_restore() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let metas = report_two(&store);
    let meta_of = |pid: &str| {
        metas
            .iter()
            .find(|m| m.package_id == pid)
            .unwrap_or_else(|| panic!("{pid} 缺失"))
    };
    let huanle_json = std::fs::read_to_string(
        PathBuf::from(meta_of("huanle-cards").extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let d3_json = std::fs::read_to_string(
        PathBuf::from(meta_of("d3-select").extension_root.as_ref().unwrap())
            .join("assets/character.json"),
    )
    .unwrap();
    let (huanle_pack, _) = {
        // 复用 g1_huanle 的桥接逻辑（本地简化版：仅苦无）
        let parsed: serde_json::Value = serde_json::from_str(&huanle_json).unwrap();
        let mut cards = Vec::new();
        for c in parsed["cards"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap();
            if name != "kuwu" {
                continue;
            }
            let suit = engine_native::catalog::Suit::Spade;
            let rank = c["rank"].as_u64().unwrap() as u8;
            cards.push(
                CardDef::weapon("hnl_kuwu", c["range"].as_u64().unwrap() as u32)
                    .with_suit_rank(suit, rank)
                    .with_equip_skills(&["huanle_kuwu"]),
            );
        }
        (
            ExtensionPack {
                def: ExtensionDef { name: "huanle-upstream-noname", version: 1, description: "" },
                generals: Vec::new(),
                cards,
                host_skills: vec![HostSkillDecl {
                    name: "huanle_kuwu",
                    behavior: HostBehavior::ShaDamageForceDiscard,
                }],
                vm_skills: Vec::new(),
                init_hook: engine_native::content::extension::InitHook::None,
                enabled: true,
            },
            (),
        )
    };
    let (d3_pack, _) = bridge("d3-upstream-noname", &d3_json);
    let packs_ref: Vec<&ExtensionPack> = vec![&huanle_pack, &d3_pack];

    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let e =
        Engine::new_with_extensions(3402, 3, &packs_ref).expect("双包建局");

    // 国战模式 + 配置扩展武将（下标 = 基础表长度）→ 明置命令授予技能/体力
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    snap.core.players[0].general_slots[0] = Some(base_count as u16);
    let mut e = Engine::restore_with_extensions(&snap, &packs_ref).expect("扩展武将恢复");
    // 推进到 0 号出牌阶段（明置非补偿路径要求出牌阶段）
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel }) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let out = submit(&mut e, 0, GameAction::RevealGeneral { slot: 0 });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(
        e.save().core.effective_faction(e.catalog(), 0),
        Some(Faction::Qun),
        "扩展武将势力来自包元数据"
    );
    assert_eq!(
        e.save().core.players[0].max_hp,
        4,
        "扩展武将体力来自包元数据"
    );
    assert!(
        e.save()
            .core
            .skills
            .keys()
            .any(|(o, d)| *o == 0 && e.catalog().skill_name(*d) == "dddjinggou"),
        "明置应授予精彀实例"
    );

    // 精彀：装备武器（全场最大射程）→ 强制触发 → 目标选择挂起
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (kuwu,) = {
        let catalog = e.catalog();
        let k = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_kuwu"))
            .expect("苦无在目录") as u32;
        (k,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != kuwu);
    snap.core.players[0].hand.push(kuwu);
    let mut e = Engine::restore_with_extensions(&snap, &packs_ref).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 精彀：目标选择（其他角色）
    let c = e.open_choice().expect("精彀应挂起目标选择");
    assert_eq!(c.spec.prompt, "d3_jinggou_target");
    assert_eq!(c.observer, 0);
    // 字节恢复后应答 → 1 点伤害
    let bytes = engine_native::archive::encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &packs_ref,
    )
    .expect("精彀挂起字节恢复");
    let cid = restored.open_choice().unwrap().id;
    let hp_before = restored.debug_hp(1);
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Player(1)],
    });
    assert_eq!(restored.debug_hp(1), hp_before - 1, "精彀造成 1 点伤害");
    let _ = (kuwu, hp_before);
}

/// 内容锁：越界扩展武将槽拒绝；带包存档被纯标准恢复拒绝。
#[test]
fn g1_d3_content_lock_and_bounds() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let metas = report_two(&store);
    let d3_meta = metas
        .iter()
        .find(|m| m.package_id == "d3-select")
        .expect("d3-select");
    let d3_json = std::fs::read_to_string(
        PathBuf::from(d3_meta.extension_root.as_ref().unwrap()).join("assets/character.json"),
    )
    .unwrap();
    let (d3_pack, _) = bridge("d3-upstream-noname", &d3_json);
    let _base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let e = Engine::new_with_extensions(3403, 3, &[&d3_pack]).expect("建局");
    // 越界武将槽（扩展表之后）→ 恢复拒绝
    let mut snap = e.save();
    snap.core.players[0].general_slots[0] =
        Some(e.catalog().general_count() as u16 + 7);
    assert!(matches!(
        Engine::restore_with_extensions(&snap, &[&d3_pack]),
        Err(SnapshotError::IdOutOfBounds { what: "general_slot_index", .. })
    ));
    // 内容锁
    let saved = Engine::new_with_extensions(1, 3, &[&d3_pack]).unwrap().save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));
}

fn report_two(
    _store: &PackageStore,
) -> Vec<content_runtime::InstalledMeta> {
    let root = repo_extensions_root();
    // 重新发现以取 extension_root（store 不暴露 installed 列表时用来源目录）
    discover_extensions(&root)
        .expect("discover")
        .into_iter()
        .filter(|d| d.name == "huanle-upstream-noname" || d.name == "d3-upstream-noname")
        .map(|d| content_runtime::InstalledMeta {
            namespace: "upstream-noname".into(),
            package_id: if d.name == "d3-upstream-noname" { "d3-select".into() } else { "huanle-cards".into() },
            version: "1.0.0".into(),
            logic_hash: String::new(),
            content_hash: String::new(),
            signed: false,
            key_id: None,
            extension_root: Some(d.root.display().to_string()),
        })
        .collect()
}

/// 整包验收 U01（2026-10-08，来源级第四例）：3D精选真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局（扩展武将入目录）→ 对局中卸载
/// → 在局锁定副本续玩（扩展武将元数据仍可查）→ 撤销后重装/重新
/// import 被拒。
#[test]
fn d3_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "d3-select")
        .expect("3D精选安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（扩展武将 ddd刘冲 入目录）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/character.json"),
    )
    .unwrap();
    let (pack, unmapped) = bridge("d3-upstream-noname", &json);
    assert_eq!(unmapped, vec!["dddmoyan".to_string()], "末焱登记暂缓");
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let mut e = Engine::new_with_extensions(5901, 3, &[&pack]).expect("建局");
    assert!(
        e.catalog()
            .general_meta(base_count.try_into().expect("下标溢出"))
            .is_some(),
        "扩展武将入目录"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "d3-select", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "d3-select"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩（扩展武将元数据仍可查）
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert!(
        resumed
            .catalog()
            .general_meta(base_count.try_into().expect("下标溢出"))
            .is_some(),
        "锁定副本扩展武将仍可用"
    );
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
