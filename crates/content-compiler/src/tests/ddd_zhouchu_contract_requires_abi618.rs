use super::*;

#[test]
fn ddd_zhouchu_contract_requires_abi618() {
    let package = compile_source_named(
        "ddd-zhouchu-test",
        include_str!("../../../../content/original/ported/ddd/zhouchu.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 618);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(package.trigger_index[0].event, "phase_play");
    assert_eq!(package.trigger_index[0].filter, "owner_is_target");
    assert!(!package.trigger_index[0].optional);
}

#[test]
fn ddd_liuchong_contract_requires_abi619() {
    let package = compile_source_named(
        "ddd-liuchong-test",
        include_str!("../../../../content/original/ported/ddd/liuchong.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 619);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 3);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_resolved" && trigger.filter == "owner_source" && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_begin" && trigger.filter == "owner_alive" && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "damage_before" && trigger.filter == "owner_is_target" && !trigger.optional
    }));
}

#[test]
fn ddd_luoxian_contract_requires_abi620() {
    let package = compile_source_named(
        "ddd-luoxian-test",
        include_str!("../../../../content/original/ported/ddd/luoxian.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 620);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    let view = &package.view_as_index[0];
    assert_eq!(view.as_kind, "basic");
    assert_eq!(view.window, "any");
    assert_eq!(view.source_zone.as_deref(), Some("none"));
    assert_eq!(view.cost.as_deref(), Some("dddshilie"));
    assert_eq!(view.uses_per_turn, Some(1));
}

#[test]
fn ddd_dingfeng_contract_requires_abi621() {
    let package = compile_source_named(
        "ddd-dingfeng-test",
        include_str!("../../../../content/original/ported/ddd/dingfeng.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 621);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(package.trigger_index[0].event, "damage");
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_card_damage_target_can_receive_owner_bingliang"
    );
    assert!(package.trigger_index[0].optional);
}

#[test]
fn ddd_wuzhi_contract_requires_abi622() {
    let package = compile_source_named(
        "ddd-wuzhi-test",
        include_str!("../../../../content/original/ported/ddd/wuzhi.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 622);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    let lingyong = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.event == "phase_any_skipped")
        .unwrap();
    assert_eq!(lingyong.filter, "owner_alive");
    assert!(lingyong.optional);
    let xuxiao = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.event == "cards_moved")
        .unwrap();
    assert_eq!(
        xuxiao.filter,
        "owner_alive_discarded_black_basic_to_discard"
    );
    assert!(!xuxiao.optional);
}

#[test]
fn ddd_xujing_contract_requires_abi623() {
    let package = compile_source_named(
        "ddd-xujing-test",
        include_str!("../../../../content/original/ported/ddd/xujing.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 623);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 6);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_resolved"
            && trigger.filter
                == "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_used"
            && trigger.filter == "ddd_shijian_watched_source_card_used_targeted"
            && !trigger.optional
    }));
}

#[test]
fn ddd_caomao_contract_requires_abi624() {
    let package = compile_source_named(
        "ddd-caomao-test",
        include_str!("../../../../content/original/ported/ddd/caomao.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 624);
    assert_eq!(package.skill_index.len(), 4);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modifier_index.len(), 1);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_end"
            && trigger.filter == "owner_alive_hp_changed_this_turn_entered_discard_available"
            && trigger.optional
    }));
}

#[test]
fn ddd_xinxianying_contract_requires_abi625() {
    let package = compile_source_named(
        "ddd-xinxianying-test",
        include_str!("../../../../content/original/ported/ddd/xinxianying.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 625);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package.view_as_index.iter().all(|view| {
        view.cost.as_deref() == Some("dddzhijie")
            && view.min_cards == 2
            && view.max_cards == 2
            && view.post_draw == 2
    }));
}

#[test]
fn ddd_jianshuo_contract_requires_abi633() {
    let package = compile_source_named(
        "ddd-jianshuo-test",
        include_str!("../../../../content/original/ported/ddd/jianshuo.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 633);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
    let active = &package.active_index[0];
    assert_eq!(active.uses_per_play_phase, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_prepare" && trigger.filter == "owner_is_target" && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "damage_before" && trigger.filter == "owner_is_target" && !trigger.optional
    }));
}

#[test]
fn ddd_guanning_contract_requires_abi634() {
    let package = compile_source_named(
        "ddd-guanning-test",
        include_str!("../../../../content/original/ported/ddd/guanning.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 634);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.active_index[0].uses_per_play_phase, Some(1));
    assert_eq!(
        package.active_index[0].selection.as_ref().unwrap().zone,
        "hand_equip"
    );
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_begin"
            && trigger.filter == "owner_alive_ddd_youxue_round_pending"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_end"
            && trigger.filter == "owner_alive_ddd_chengjing_available"
            && !trigger.optional
    }));
}

#[test]
fn ddd_caoshuang_contract_requires_abi635() {
    let package = compile_source_named(
        "ddd-caoshuang-test",
        include_str!("../../../../content/original/ported/ddd/caoshuang.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 635);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 4);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "recover_before"
            && trigger.filter == "owner_alive_ddd_weiqiu"
            && !trigger.optional
    }));
}

#[test]
fn ddd_xuelingyun_contract_requires_abi636() {
    let package = compile_source_named(
        "ddd-xuelingyun-test",
        include_str!("../../../../content/original/ported/ddd/xuelingyun.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 636);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 7);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_targeted"
            && trigger.filter == "owner_alive_ddd_anzhi_target_black"
            && !trigger.optional
    }));
}

#[test]
fn ddd_liuhong_contract_requires_abi637() {
    let package = compile_source_named(
        "ddd-liuhong-test",
        include_str!("../../../../content/original/ported/ddd/liuhong.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 10);
    assert_eq!(package.modifier_index.len(), 1);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "death"
            && trigger.filter == "owner_alive_ddd_fuzong_death"
            && !trigger.optional
    }));
}

#[test]
fn ddd_xiahouxuan_contract_requires_abi638() {
    let package = compile_source_named(
        "ddd-xiahouxuan-test",
        include_str!("../../../../content/original/ported/ddd/xiahouxuan.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 638);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 3);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "turn_ended"
            && trigger.filter == "owner_alive_ddd_xuanlun_pending"
            && !trigger.optional
    }));
}

#[test]
fn ddd_zhangkai_contract_requires_abi639() {
    let package = compile_source_named(
        "ddd-zhangkai-test",
        include_str!("../../../../content/original/ported/ddd/zhangkai.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 639);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 5);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "recover_before"
            && trigger.filter == "owner_alive_ddd_jiexing_recover_once_per_game"
            && trigger.optional
    }));
}

#[test]
fn ddd_liangxi_contract_requires_abi640() {
    let package = compile_source_named(
        "ddd-liangxi-test",
        include_str!("../../../../content/original/ported/ddd/liangxi.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 640);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(package.view_as_index.len(), 1);
    let view = &package.view_as_index[0];
    assert_eq!(view.as_kind, "wugu");
    assert_eq!(view.min_cards, 1);
    assert_eq!(view.max_cards, 4);
    assert_eq!(view.cost.as_deref(), Some("dddtongyu"));
}

#[test]
fn yijiang_jiushi_contract_requires_abi637() {
    let source = format!(
        "{}\n{}",
        include_str!("../../../../content/original/ported/yijiang/jiushi1.skill"),
        include_str!("../../../../content/original/ported/yijiang/jiushi3.skill")
    );
    let package = compile_source_named("yijiang-jiushi-test", &source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == 3_647_393_232
            && view.as_kind == "jiu"
            && view.source_zone.as_deref() == Some("none")
            && view.cost.as_deref() == Some("turn_over")
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 4_156_803_269
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_face_down"
            && trigger.optional
    }));
}

/// Guard: every shipped `content/standard/*.skill` must compile. Prevents
/// silent regressions as the DSL surface and content grow (§44 M0/M1).
#[test]
fn all_standard_skill_files_compile() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/standard");
    let mut count = 0usize;
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read content/standard") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("skill") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        count += 1;
        if let Err(e) = compile_source(&src) {
            failures.push(format!("{}: {e:?}", path.display()));
        }
    }
    assert!(
        count >= 15,
        "expected the M0 sample surface (>=15), got {count}"
    );
    assert!(
        failures.is_empty(),
        "{} skill file(s) failed to compile:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
