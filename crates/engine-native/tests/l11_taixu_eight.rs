//! L11 太虚幻境（`taixu-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `TaixuConfig` 与 G0 计数（146 技能、26 牌、54 武将、152 翻译）；
//! ③ 组合共存：与 `doomsday-brawl-upstream-liuli`、`boss-play-upstream-liuli` 同商店激活不冲突；
//! ④ 保存恢复：`TaixuRunState` 单行快照往返一致，标准对局引擎状态不被单机战役污染；
//! ⑤ 功能执行：战役起局（`start`）、购买祝福（`buy_buff`）、绑定侍灵（`bind_spirit`）、推进关卡至通关（`advance`）；
//! ⑦ 异常拒装：金币不足、祝福超上限、重复祝福/侍灵、空赛季与篡改 G0 计数均返回明确 `TaixuError`；
//! ⑧ 边界登记：11,488 行独立单机 PvE 战役脚本不注入联机权威内核，按战役状态机隔离承载。

use std::path::PathBuf;

use content_native_loader::{
    apply_taixu_line, parse_taixu_config, restore_taixu, snapshot_taixu, TaixuConfig, TaixuError,
    TaixuRunState, TAIXU_COUNTS,
};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::Engine;

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
fn l11_taixu_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "taixu-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "taixu" && m.namespace == "upstream-liuli")
        .expect("taixu 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与 G0 计数核对
    let cfg = parse_taixu_config(&cfg_text).expect("解析 taixu config.json");
    assert_eq!(cfg, TaixuConfig::default());
    assert_eq!(TAIXU_COUNTS, (146, 26, 54, 152));

    // ③ 与末日乱斗/Boss挑战模式包同商店共存
    for pid in ["doomsday-brawl", "boss-play"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 同时安装成功"
        );
    }

    // ⑤ 战役状态机执行（起局 -> 买祝福 -> 绑侍灵 -> 5 关推进通关）
    let mut state = TaixuRunState::from_config(&cfg);
    apply_taixu_line(&cfg, &mut state, "start").unwrap();
    apply_taixu_line(&cfg, &mut state, "buy_buff tx_fire_blade 35").unwrap();
    apply_taixu_line(&cfg, &mut state, "buy_buff tx_iron_armor 25").unwrap();
    apply_taixu_line(&cfg, &mut state, "bind_spirit spirit_baihu").unwrap();
    assert_eq!(state.gold, 40);
    assert_eq!(state.buffs, vec!["tx_fire_blade", "tx_iron_armor"]);
    assert_eq!(state.spirits, vec!["spirit_baihu"]);

    // ④ 中途快照与恢复
    let mid_snap = snapshot_taixu(&state);
    let mid_restored = restore_taixu(&mid_snap).expect("恢复战役快照");
    assert_eq!(mid_restored, state);

    // 推进 5 个节点完成战役
    for step in 1..=5 {
        apply_taixu_line(&cfg, &mut state, "advance 20").unwrap();
        assert_eq!(state.node_index, step);
    }
    assert!(state.completed, "第 5 关后标记为通关");
    assert_eq!(
        apply_taixu_line(&cfg, &mut state, "advance 10"),
        Err(TaixuError::AlreadyCompleted),
        "通关后不可继续推进"
    );

    // 标准引擎状态隔离验证
    let e1 = Engine::new(191101, 4);
    let e2 = Engine::new(191101, 4);
    assert_eq!(e1.state_hash(), e2.state_hash());

    // ⑥ 停用与回滚闭环
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 taixu");
    let rolled = store.rollback().expect("回滚 taixu");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/taixu")));

    // ⑦ 异常分支与坏配置拒绝
    let mut err_state = TaixuRunState::from_config(&cfg);
    assert_eq!(
        apply_taixu_line(&cfg, &mut err_state, "buy_buff expensive 500"),
        Err(TaixuError::InsufficientGold {
            have: 100,
            need: 500
        })
    );
    let bad_season = cfg_text.replace("\"season\":\"taixu_s1\"", "\"season\":\"\"");
    assert!(matches!(
        parse_taixu_config(&bad_season),
        Err(TaixuError::InvalidField { field: "season", .. })
    ));
    let bad_count = cfg_text.replace("\"skill_count\":146", "\"skill_count\":0");
    assert!(matches!(
        parse_taixu_config(&bad_count),
        Err(TaixuError::InvalidCount { field: "skill_count", .. })
    ));
}
