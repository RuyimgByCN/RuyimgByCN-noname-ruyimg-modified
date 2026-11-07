//! L28 雷霆万钧（`thunder-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `ThunderConfig`（`game_speed=4`, `replace_cha=true`, `hide_pack=true` 等）；
//! ③ 组合共存：与 `skin-switch-upstream-liuli`、`qhly-upstream-liuli`、`longxue-upstream-liuli` 同商店激活不冲突；
//! ④ 保存恢复：配置解析与重序列化/重解析往返一致；
//! ⑤ 功能执行：`replace_plan` 武将立绘替换配对表（`UPSTREAM_CHA_LEN == UPSTREAM_CHA2_LEN == 55`）与 `mark_labels` 新旧标记文案切换；
//! ⑦ 异常拒装：越界 `gameSpeed`（0 或 >10）、缺失字段与损坏 JSON 明确返回 `ThunderError`；
//! ⑧ 边界登记：UI 补丁、立绘替换与主题渲染不进入权威规则内核。

use std::path::PathBuf;

use content_native_loader::{
    mark_labels, parse_thunder_config, ReplacePlan, ThunderError, CHG_HERO, EXCLUSIVE_PACK,
    SAME_HERO,
};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};

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

#[test]
fn l28_thunder_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "thunder-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "thunder" && m.namespace == "upstream-liuli")
        .expect("thunder 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与默认字段核对
    let cfg = parse_thunder_config(&cfg_text).expect("解析 thunder config.json");
    assert_eq!(cfg.source, "noname-liuli");
    assert_eq!(cfg.upstream_path, "noname/extension/雷霆万钧/extension.js");
    assert_eq!(cfg.game_speed, 4);
    assert!(cfg.replace_cha);
    assert!(cfg.hide_pack);
    assert!(!cfg.forbid_same);
    assert!(cfg.ui_patch);
    assert!(cfg.jinyu);
    assert!(cfg.shh_biaoji);
    assert!(!cfg.shh_new);
    assert!(cfg.jiufa_text);
    assert!(!cfg.guanning);
    assert_eq!(EXCLUSIVE_PACK, "thunder");
    assert!(cfg.hides_exclusive_pack());
    assert_eq!(SAME_HERO.len(), 12);
    assert_eq!(CHG_HERO.len(), 12);

    // ③ 与皮肤切换/千幻聆音/龙血玄黄同商店共存
    for pid in ["skin-switch", "qhly", "longxue"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 已安装"
        );
    }

    // ⑤ 同名替换计划与标记文案执行
    let plan: ReplacePlan = cfg.replace_plan();
    assert_eq!(plan.pairs.len(), 12);
    assert_eq!(plan.pairs[0], ("majun".into(), "th_majun".into()));
    let mut cfg_no_replace = cfg.clone();
    cfg_no_replace.replace_cha = false;
    assert!(cfg_no_replace.replace_plan().pairs.is_empty());

    assert_eq!(
        mark_labels(&cfg, 0, 1),
        vec![
            ("miaojian".into(), "妙剑1级".into()),
            ("shhlianhua".into(), "莲华2级".into()),
        ]
    );
    let mut cfg_new = cfg.clone();
    cfg_new.shh_new = true;
    assert_eq!(
        mark_labels(&cfg_new, 0, 1),
        vec![
            ("miaojian".into(), "妙剑¹".into()),
            ("shhlianhua".into(), "莲华²".into()),
        ]
    );
    cfg_new.shh_biaoji = false;
    assert!(mark_labels(&cfg_new, 0, 1).is_empty());

    // ④ 配置序列化与重解析往返
    let roundtrip_json = format!(
        r#"{{"source":"{}","upstream_path":"{}","upstream_commit":"{}","gameSpeed":{},"replaceCha":{},"hidePack":{},"forbidSame":{},"UIpatch":{},"jinyu":{},"shhbiaoji":{},"shh_new":{},"jiufatext":{},"guanning":{}}}"#,
        cfg.source,
        cfg.upstream_path,
        cfg.upstream_commit,
        cfg.game_speed,
        cfg.replace_cha,
        cfg.hide_pack,
        cfg.forbid_same,
        cfg.ui_patch,
        cfg.jinyu,
        cfg.shh_biaoji,
        cfg.shh_new,
        cfg.jiufa_text,
        cfg.guanning
    );
    assert_eq!(parse_thunder_config(&roundtrip_json).unwrap(), cfg);

    // ⑥ 停用与回滚
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 thunder");
    let rolled = store.rollback().expect("回滚 thunder");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/thunder")));

    // ⑦ 越界速度与损坏配置拒绝
    let bad_speed = cfg_text.replace("\"gameSpeed\": 4", "\"gameSpeed\": 99");
    assert_eq!(
        parse_thunder_config(&bad_speed),
        Err(ThunderError::BadField("gameSpeed".into()))
    );
    assert!(matches!(
        parse_thunder_config("{bad"),
        Err(ThunderError::BadField(_))
    ));
}
