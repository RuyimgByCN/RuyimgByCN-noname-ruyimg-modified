//! NC08 附加：**「配置」验收**（八项验收第 3 项，纯 Rust 可做）。
//!
//! 依据：八项验收清单（[证据](../../docs/operations/evidence/2026-10-19-eight-item-acceptance/README.md)）——
//! 「配置」= 配置项可读写、默认值正确、**非法值被拒**。
//!
//! 本用例覆盖四个可机械核验的配置面：
//! 1. **元数据完整性**：每个已转换包 `def.name` 非空、`version >= 1`、`description` 非空；
//! 2. **`enabled` 开关语义**：把 `enabled` 置 `false` ⇒ 该包的卡**不进入目录**（且建局不报错）；
//! 3. **`init_hook` 解析**：包数据里的 `init_hook` 字符串**必须能解析**为已知枚举（未知值应被拒）；
//! 4. **非法数据拒绝**：畸形 `cards.json`（缺 `cards`/类型错误）⇒ `bridge_pack` 返回 **Err**。
//!
//! 边界：本用例**不覆盖**「运行时改配置」与「配置持久化」——二者登记为待办。

use std::path::{Path, PathBuf};

fn assets_dirs() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path().join("assets/cards.json");
            if p.is_file() {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).expect("读 cards.json")
}

/// 配置面 1（**元数据完整性**）：`name` 非空、`version >= 1`、`description` 非空。
#[test]
fn config_metadata_complete() {
    let mut checked = 0;
    for p in assets_dirs() {
        let pack = content_native_loader::bridge_pack(&read(&p)).expect("桥接应成功").pack;
        assert!(!pack.def.name.is_empty(), "{}: def.name 不应为空", p.display());
        assert!(pack.def.version >= 1, "{}: version 应 >= 1（实测 {}）", p.display(), pack.def.version);
        assert!(!pack.def.description.is_empty(), "{}: description 不应为空", p.display());
        checked += 1;
    }
    eprintln!("DBG 配置-元数据：检查 {checked} 个包，全部通过");
    assert!(checked >= 10, "应检查至少 10 个已转换包（实测 {checked}）");
}

/// 配置面 2（**`enabled` 开关语义**）：置 `false` ⇒ 卡不进入目录，且建局不报错。
#[test]
fn config_enabled_flag_gates_cards() {
    let on_path = assets_dirs()
        .into_iter()
        .find(|p| p.display().to_string().contains("shahai-swd"))
        .expect("swd 包");
    let json = read(&on_path);
    // 开：目录含该包的卡
    let on = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let on_cards = on.cards.len();
    // 关：置 enabled=false 后建局，目录不应含该包卡
    let mut off = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    off.enabled = false;
    let e_on = engine_native::Engine::new_with_extensions(11, 4, &[&on]).expect("开-建局");
    let e_off = engine_native::Engine::new_with_extensions(11, 4, &[&off]).expect("关-建局");
    let names_on = e_on.catalog().cards.len();
    let names_off = e_off.catalog().cards.len();
    eprintln!(
        "DBG 配置-enabled：包卡 {on_cards}；开-目录 {names_on}；关-目录 {names_off}（应更少）"
    );
    assert!(
        names_off < names_on,
        "enabled=false 时目录应更少（开 {names_on} → 关 {names_off}）"
    );
}

/// 配置面 3（**`init_hook` 解析**）：包数据的 `init_hook` 必须能解析为已知枚举。
#[test]
fn config_init_hook_parses() {
    let mut hooks = std::collections::BTreeSet::new();
    for p in assets_dirs() {
        let json = read(&p);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(h) = v.pointer("/pack/init_hook").and_then(|x| x.as_str()) {
                hooks.insert(h.to_string());
                // 已知枚举值应被接受（未知值由 bridge_pack 拒绝——见配置面 4）
                assert!(
                    ["wuxing_natures", "swd_jiqi_held"].contains(&h),
                    "{}: 未登记的 init_hook `{h}`（若确为新增，请补 InitHook 枚举与解析）",
                    p.display()
                );
            }
        }
    }
    eprintln!("DBG 配置-init_hook：出现的取值 = {hooks:?}");
}

/// 配置面 4（**非法数据拒绝**）：畸形 `cards.json` ⇒ `bridge_pack` 返回 `Err`。
#[test]
fn config_invalid_data_rejected() {
    // ① 非 JSON
    assert!(
        content_native_loader::bridge_pack("not json at all").is_err(),
        "非 JSON 应被拒"
    );
    // ② 缺 cards 字段
    assert!(
        content_native_loader::bridge_pack("{\"pack\":{\"name\":\"x\",\"version\":1}}").is_err(),
        "缺 cards 字段应被拒（或按约定视为空包——若如此请调整本断言并登记）"
    );
    // ③ 未知 init_hook
    let bad_hook = "{\"pack\":{\"name\":\"x\",\"version\":1,\"init_hook\":\"nope\"},\"cards\":[]}";
    assert!(
        content_native_loader::bridge_pack(bad_hook).is_err(),
        "未知 init_hook 应被拒"
    );
    eprintln!("DBG 配置-非法数据：三类畸形输入均被拒");
}

fn ext_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn install_options() -> content_runtime::InstallOptions {
    content_runtime::InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn verify_store_lifecycle_and_read_config(package_id: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let store = content_runtime::PackageStore::open(dir.path().join("store")).unwrap();
    let report =
        content_runtime::import_extensions(&store, &ext_root(), &install_options()).expect("导入扩展");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == package_id)
        .unwrap_or_else(|| panic!("{package_id} 应已安装"))
        .clone();
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用配置包");
    let rolled = store.rollback().expect("回滚激活配置包");
    assert!(
        rolled
            .packages
            .iter()
            .any(|(slot, _)| slot.ends_with(&format!("{}/{}", meta.namespace, meta.package_id))),
        "{package_id} 回滚后恢复激活"
    );
    let root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    std::fs::read_to_string(root.join("assets/config.json")).expect("读取 config.json")
}

#[test]
fn config_acceptance_l08_decade_ui_package() {
    let raw = verify_store_lifecycle_and_read_config("decade-ui");
    let mut cfg = content_native_loader::parse_decade_ui_config(&raw).expect("解析 decade-ui 配置");
    assert_eq!(cfg, content_native_loader::DecadeUiConfig::default());
    content_native_loader::apply_decade_ui_line(&mut cfg, "style onlineUI").unwrap();
    let snap = content_native_loader::snapshot_decade_ui(&cfg);
    assert_eq!(content_native_loader::restore_decade_ui(&snap).unwrap(), cfg);
    assert!(content_native_loader::parse_decade_ui_config(
        &raw.replace("\"newDecadeStyle\":\"on\"", "\"newDecadeStyle\":\"bad\"")
    )
    .is_err());
}

#[test]
fn config_acceptance_l11_taixu_package() {
    let raw = verify_store_lifecycle_and_read_config("taixu");
    let cfg = content_native_loader::parse_taixu_config(&raw).expect("解析 taixu 配置");
    assert_eq!(cfg, content_native_loader::TaixuConfig::default());
    let mut state = content_native_loader::TaixuRunState::from_config(&cfg);
    content_native_loader::apply_taixu_line(&cfg, &mut state, "advance 15").unwrap();
    let snap = content_native_loader::snapshot_taixu(&state);
    assert_eq!(content_native_loader::restore_taixu(&snap).unwrap(), state);
    assert!(content_native_loader::parse_taixu_config(
        &raw.replace("\"buff_limit\":6", "\"buff_limit\":0")
    )
    .is_err());
}

#[test]
fn config_acceptance_l12_rzsh_package() {
    let raw = verify_store_lifecycle_and_read_config("rzsh");
    let mut cfg = content_native_loader::parse_rzsh_config(&raw).expect("解析 rzsh 配置");
    assert_eq!(cfg, content_native_loader::RzshConfig::default());
    cfg.ui_scale = 125;
    let snap = content_native_loader::snapshot_rzsh(&cfg);
    assert_eq!(content_native_loader::restore_rzsh(&snap).unwrap(), cfg);
    assert!(content_native_loader::parse_rzsh_config(
        &raw.replace("\"ui_scale\":100", "\"ui_scale\":10")
    )
    .is_err());
}

#[test]
fn config_acceptance_l14_shousha_ui_package() {
    let raw = verify_store_lifecycle_and_read_config("shousha-ui");
    let mut cfg = content_native_loader::parse_shousha_ui_config(&raw).expect("解析 shousha-ui 配置");
    assert_eq!(cfg, content_native_loader::ShoushaUiConfig::default());
    cfg.lbtn_enabled = false;
    let snap = content_native_loader::snapshot_shousha_ui(&cfg);
    assert_eq!(content_native_loader::restore_shousha_ui(&snap).unwrap(), cfg);
    assert!(content_native_loader::parse_shousha_ui_config(
        &raw.replace("\"card_layout\":\"right\"", "\"card_layout\":\"diagonal\"")
    )
    .is_err());
}

#[test]
fn config_acceptance_l25_zuan_package() {
    let raw = verify_store_lifecycle_and_read_config("zuan");
    let mut cfg = content_native_loader::parse_zuan_config(&raw).expect("解析 zuan 配置");
    assert_eq!(cfg, content_native_loader::ZuanConfig::default());
    cfg.taunt_pack = false;
    let snap = content_native_loader::snapshot_zuan(&cfg);
    assert_eq!(content_native_loader::restore_zuan(&snap).unwrap(), cfg);
    assert!(content_native_loader::parse_zuan_config(
        &raw.replace("\"enable_editor\":false", "\"enable_editor\":true")
            .replace("\"offline_debug_only\":true", "\"offline_debug_only\":false")
    )
    .is_err());
}

#[test]
fn hero_kill_config_matches_upstream() {
    let raw = std::fs::read_to_string(ext_root().join("hero-kill-upstream-noname/assets/config.json"))
        .expect("读取 hero-kill config.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON 解析");
    assert_eq!(v["source"].as_str(), Some("noname"));
    assert_eq!(
        v["upstream_path"].as_str(),
        Some("apps/core/extension/英雄杀/character/skill.js")
    );
    let skills: Vec<&str> = v["skills"]
        .as_array()
        .expect("skills 数组")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(
        skills,
        vec!["yxs_fanji", "yxs_fanji2", "yxs_menshen", "yxs_menshen3", "yxs_menshen2"]
    );
}

#[test]
fn doomsday_brawl_config_matches_upstream() {
    let raw = std::fs::read_to_string(
        ext_root().join("doomsday-brawl-upstream-liuli/assets/config.json"),
    )
    .expect("读取 doomsday-brawl config.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON 解析");
    assert_eq!(v["source"].as_str(), Some("noname-liuli"));
    assert_eq!(
        v["upstream_path"].as_str(),
        Some("noname/extension/末日乱斗/extension.js")
    );
    let skills: Vec<&str> = v["skills"]
        .as_array()
        .expect("skills 数组")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(skills, vec!["shishi", "xunmeng", "zaibian"]);
}

#[test]
fn mark_supplement_config_matches_upstream() {
    let raw = std::fs::read_to_string(
        ext_root().join("mark-supplement-upstream-liuli/assets/config.json"),
    )
    .expect("读取 mark-supplement config.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON 解析");
    assert_eq!(v["source"].as_str(), Some("noname-liuli"));
    assert_eq!(
        v["upstream_path"].as_str(),
        Some("noname/extension/标记补充/extension.js")
    );
    assert_eq!(v["mpfbmoshi"].as_bool(), Some(false));
    assert_eq!(v["xueliangfanbei"].as_bool(), Some(false));
    assert_eq!(v["daojianfu"].as_bool(), Some(false));
}

#[test]
fn select_beautify_config_matches_upstream() {
    let raw = std::fs::read_to_string(
        ext_root().join("select-beautify-upstream-liuli/assets/config.json"),
    )
    .expect("读取 select-beautify config.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON 解析");
    assert_eq!(v["source"].as_str(), Some("noname-liuli"));
    assert_eq!(
        v["upstream_path"].as_str(),
        Some("noname/extension/选将美化/extension.js")
    );
    let skills: Vec<&str> = v["skills"]
        .as_array()
        .expect("skills 数组")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(
        skills,
        vec![
            "xiefang",
            "new_zhixi",
            "new_zhixi_mark2",
            "new_meibu",
            "new_meibu_range",
            "sbzhaxiang",
            "jishe"
        ]
    );
}

#[test]
fn thunder_config_matches_upstream() {
    let raw = std::fs::read_to_string(ext_root().join("thunder-upstream-liuli/assets/config.json"))
        .expect("读取 thunder config.json");
    let cfg = content_native_loader::parse_thunder_config(&raw).expect("解析 thunder 配置");
    assert_eq!(cfg.source, "noname-liuli");
    assert_eq!(cfg.upstream_path, "noname/extension/雷霆万钧/extension.js");
    assert_eq!(cfg.game_speed, 4);
    assert!(cfg.replace_cha);
    assert!(cfg.hide_pack);
}

#[test]
fn all_46_extension_packages_coexist_in_store_and_bridge_into_live_engine() {
    let dir = tempfile::tempdir().unwrap();
    let store = content_runtime::PackageStore::open(dir.path().join("store")).unwrap();
    let report =
        content_runtime::import_extensions(&store, &ext_root(), &install_options()).expect("导入全部扩展包");
    assert_eq!(
        report.installed.len(),
        48,
        "全部 48 个包目录（46 个扩展包 + 2 个核心武将包 sp2/xianding）同时安装成功"
    );
    let upstream_installed = report
        .installed
        .iter()
        .filter(|m| m.package_id != "sp2" && m.package_id != "xianding")
        .count();
    assert_eq!(upstream_installed, 46, "全部 46 个上游扩展包目录同时安装成功");
    let active = store.read_active().expect("读取激活集");
    let upstream_active = active
        .packages
        .keys()
        .filter(|k| !k.ends_with("/sp2") && !k.ends_with("/xianding"))
        .count();
    assert_eq!(upstream_active, 46, "全部 46 个上游扩展包同时处于激活集");

    let mut bridged = Vec::new();
    for p in assets_dirs() {
        let out = content_native_loader::bridge_pack(&read(&p)).expect("桥接扩展包");
        bridged.push(out.pack);
    }
    let refs: Vec<&engine_native::content::extension::ExtensionPack> = bridged.iter().collect();
    let e = engine_native::Engine::new_with_extensions(194600, 4, &refs)
        .expect("全部内容型扩展包同局联合建局");
    let snap = e.save();
    let restored = engine_native::Engine::restore_with_extensions(&snap, &refs)
        .expect("全部内容型扩展包同局存档恢复");
    assert_eq!(restored.state_hash(), e.state_hash());
}

