#[test]
fn shahai_publishes_four_rewrites() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // tuteng1 / changnian2：结束阶段触发（回复 / 摸牌）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == TUTENG1 && t.event == "phase_end"));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == CHANGNIAN2 && t.event == "phase_end"));
    // ljifeng：距离 = 已损体力（负值源）。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == LJIFENG
            && m.target == "distance_from"
            && m.mode == "add"
            && m.value_source.as_deref() == Some("owner_lost_hp")
    }));
    // xjumo：手牌上限 +3，受伤时再 +1+1。
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|m| m.skill_def_id == XJUMO && m.target == "hand_limit")
            .count(),
        3
    );
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == XJUMO && m.mode == "add" && m.value == 3 && m.value_source.is_none()
    }));
}

#[test]
fn shahai_publishes_mingkong_tuji() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // mingkong：damage_before 强制触发（空手 → 减伤 + 来源摸牌）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == MINGKONG && t.event == "damage_before"));
    // tuji：距离减标记 + card_used（当前回合门控）累积 + turn_ended 清零，共两触发。
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == TUJI)
            .count(),
        2
    );
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == TUJI
            && m.target == "distance_from"
            && m.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

#[test]
fn shahai_publishes_gtiandao_suiyan_guanhu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // gtiandao：判定改写 modifier。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == GTIANDAO
            && m.target == "judgment_rewrite"
            && m.mode == "override"
            && m.value == 1
    }));
    // suiyan：伤害来源触发，自弃 1 + 弃对方全部装备。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == SUIYAN && t.event == "damage" && t.optional));
    // guanhu：杀伤害来源触发，弃对方一张 hand_equip。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUANHU && t.event == "damage" && t.filter == "owner_source_sha"
    }));
}

#[test]
fn shahai_publishes_hongxi_meihuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // hongxi：任意死亡触发回满（forced）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == HONGXI && t.event == "death" && !t.optional }));
    // meihuo：ADR 0118 最后装备失牌过滤器 + 获得目标手牌。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == MEIHUO
            && t.event == "cards_moved"
            && t.filter == "owner_alive_event_player_from_equip_empty"
    }));
}

#[test]
fn shahai_publishes_equip_presence_family() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let expect = [
        (PINGXU, "distance_from", "owner_no_weapon", -1),
        (PINGXU, "distance_to", "owner_armor_empty", 1),
        (JUFU, "sha_damage", "owner_has_weapon", 1),
        (HUJIA_E3, "distance_to", "owner_has_armor_equip", 1),
        (HUJIA_E4, "distance_from", "owner_has_armor_equip", -1),
        (
            SHIJIAN_E3,
            "distance_to",
            "owner_equip_count_exactly_one",
            1,
        ),
        (
            SHIJIAN_E4,
            "distance_from",
            "owner_equip_count_exactly_one",
            -1,
        ),
    ];
    for (def, target, source, value) in expect {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == target
                    && m.value_source.as_deref() == Some(source)
                    && m.value == value
            }),
            "missing modifier {def} {target} {source}"
        );
    }
}

#[test]
fn shahai_publishes_huimeng_shali_longyi_lvzhou() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == HUIMENG && t.event == "hp_recovered" && t.optional }));
    // shali：forced 回复触发 + 表达式护甲增量。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == SHALI && t.event == "hp_recovered" && !t.optional }));
    for (def, selector) in [(HLONGYI, "black"), (SHAMOLVZHOU, "basic")] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == "card_ignored_hand_limit"
                    && m.selector.as_deref() == Some(selector)
            }),
            "missing ignored-hand-limit {selector}"
        );
    }
}

#[test]
fn shahai_publishes_sha_enhancement_trio() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // shoulie / xiandan：card_used owner_source_sha 可选触发。
    for def in [SHOULIE, XIANDAN] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == def
                && t.event == "card_used"
                && t.filter == "owner_source_sha"
                && t.optional
        }));
    }
    // lianwu：杀目标上限 +1（常量合法）。
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == LIANWU
            && m.target == "card_target_limit"
            && m.selector.as_deref() == Some("sha")
            && m.value == 1
    }));
}

#[test]
fn shahai_publishes_draw_family_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == XINGTI_E1 && t.event == "damage" && !t.optional }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == JINGCU));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == YUEDONG && t.event == "phase_play_end" && t.optional }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == JIAOXIA && t.event == "card_targeted" && t.optional }));
}

#[test]
fn shahai_publishes_damage_family_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, event, filter) in [
        (MISHA, "damage", "owner_is_target"),
        (BAIZHAN, "damage", "owner_is_source"),
        (DALIAN, "damage", "owner_is_source"),
        (TUTENG3, "damage_before", "owner_is_target"),
    ] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def && t.event == event && t.filter == filter && !t.optional
            }),
            "missing trigger {def} {event}"
        );
    }
    for target in ["sha_damage", "sha_damage_taken"] {
        assert!(package
            .modifier_index
            .iter()
            .any(|m| { m.skill_def_id == XINGJUNYAN && m.target == target && m.value == 1 }));
    }
}

#[test]
fn shahai_publishes_misc_family_seven() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, optional) in [(TUTENG7, false), (LINGYU, true)] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == def && t.event == "phase_end" && t.optional == optional
        }));
    }
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == LINGXUE && t.event == "hp_recovered" && !t.optional }));
    let expect = [
        (RONGHUO, "card_nature", Some("sha")),
        (
            MOYAO,
            "card_target_cancel_from_current",
            Some("ordinary_trick"),
        ),
        (
            GJQTWUXIE,
            "card_target_cancel_from_current",
            Some("delayed_trick"),
        ),
        (
            SWD_WUXIE,
            "card_target_cancel_from_current",
            Some("delayed_trick"),
        ),
    ];
    for (def, target, selector) in expect {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def && m.target == target && m.selector.as_deref() == selector
            }),
            "missing modifier {def} {target} {selector:?}"
        );
    }
}

#[test]
fn shahai_publishes_equip_defense_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == YULIN && t.event == "damage_before" && t.optional }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == HUJIA_E2B && t.event == "damage" && t.optional }));
}

#[test]
fn shahai_publishes_mid_bucket_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, event, filter) in [
        (XIXUE, "damage", "owner_source_sha"),
        (NINGBIN, "damage", "owner_target_thunder"),
        (PANGUFU, "damage", "owner_is_source"),
        (SHENGZHANG, "phase_discard_end", "owner_is_target"),
    ] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def && t.event == event && t.filter == filter && !t.optional
            }),
            "missing trigger {def} {event} {filter}"
        );
    }
}

#[test]
fn shahai_yuchan_family_consumes_657() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 玉蝉族 40 块全部成为 active 消费者。
    let yuchan: Vec<_> = package
        .skill_index
        .iter()
        .filter(|s| s.name.starts_with("yuchan") && s.name.contains("_equip"))
        .collect();
    assert_eq!(yuchan.len(), 40);
    for skill in &yuchan {
        assert!(
            package
                .active_index
                .iter()
                .any(|a| a.skill_def_id == skill.skill_def_id),
            "yuchan {} missing active",
            skill.name
        );
    }
}

#[test]
fn shahai_relic_families_twenty_blocks() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 四神器族（每族 5 块）全部成为 phase_end 触发者。
    for prefix in [
        "huanglinzhicong",
        "xuanwuzhihuang",
        "zhuquezhizhang",
        "baishouzhihu",
    ] {
        let defs: Vec<_> = package
            .skill_index
            .iter()
            .filter(|s| s.name.starts_with(prefix) && s.name.contains("_equip"))
            .collect();
        assert_eq!(defs.len(), 5, "{prefix} family size");
        for skill in &defs {
            assert!(
                package.trigger_index.iter().any(|t| {
                    t.skill_def_id == skill.skill_def_id
                        && t.event == "phase_end"
                        && t.filter == "owner_is_target"
                        && t.optional
                }),
                "{} missing phase_end trigger",
                skill.name
            );
        }
    }
}

#[test]
fn shahai_publishes_kuixin_fengxian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == KUIXIN && t.event == "phase_end" && t.optional }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == FENGXIAN));
}

#[test]
fn shahai_bystander_filter_unlocks_trio() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for def in [DAOFA, CHENGJIAN, XUEZHOU] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def
                    && t.event == "damage"
                    && t.filter == "owner_alive_other_source_damage"
                    && t.optional
            }),
            "bystander trigger missing for {def}"
        );
    }
}

#[test]
fn shahai_publishes_heal_armor_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for (def, kind) in [
        (BAIHUPIFENG, "phase_end"),
        (FUYAN, "damage"),
        (BINXIN, "phase_end"),
    ] {
        assert!(package
            .trigger_index
            .iter()
            .any(|t| t.skill_def_id == def && t.event == kind && t.optional));
    }
    for def in [XINCI, SHENGGUANG] {
        assert!(package.active_index.iter().any(|a| a.skill_def_id == def));
    }
}

#[test]
fn shahai_publishes_shennong_zhiyuan_shijie() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHENNONGDING));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == ZHIYUAN && t.event == "damage_before" && t.optional }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == SHIJIE && t.event == "phase_end" && t.optional));
}

#[test]
fn shahai_publishes_hudun_guiyuan_mojian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == HUDUN && t.event == "damage" && !t.optional }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == GUIYUAN));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == MOJIAN && t.event == "card_targeted" && t.optional }));
}

#[test]
fn shahai_publishes_diesha_fengming_fanshi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DIESHA && t.event == "damage" && t.filter == "owner_source_sha"
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == FENGMING));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == FANSHI && t.event == "phase_discard_end" && !t.optional }));
}

#[test]
fn shahai_publishes_xueren_txianqu_yaliang() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // xueren：杀伤害 → 目标+自己各失去 1 体力，然后摸两张。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUEREN
            && t.event == "damage"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    // txianqu：伤害前可选防伤（出牌阶段限一次，storage 门控）。
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == TXIANQU && t.event == "damage_before" && t.optional }));
    // yaliang：造成或受到伤害双方各摸一张（双 trigger）。
    let yaliang_triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YALIANG)
        .collect();
    assert_eq!(yaliang_triggers.len(), 2);
    assert!(yaliang_triggers
        .iter()
        .any(|t| t.filter == "owner_is_source"));
    assert!(yaliang_triggers
        .iter()
        .any(|t| t.filter == "owner_is_target"));
}

#[test]
fn shahai_publishes_yougeng_liyutang_mieshi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YOUGENG
            && t.event == "phase_prepare"
            && t.filter == "owner_min_hp"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LIYUTANG
            && t.event == "phase_end"
            && t.filter == "owner_min_hp"
            && !t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == MIESHI && t.event == "phase_end" && !t.optional }));
}

#[test]
fn shahai_publishes_food_card_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == MOLICHA
            && m.target == "card_target_cancel"
            && m.selector.as_deref() == Some("black")
            && m.mode == "override"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == YUANBAOROU
            && m.target == "sha_use_limit"
            && m.mode == "add"
            && m.value == 1
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TANHUADONG
            && t.event == "phase_play_end"
            && t.filter == "owner_is_target"
            && !t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == MAPODOUFU && t.event == "phase_end" && !t.optional }));
}

#[test]
fn shahai_publishes_food_card_family_tail_six() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .modifier_index
        .iter()
        .any(|m| { m.skill_def_id == XIAOJIAO && m.target == "draw_count" && m.mode == "add" }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == CHUNBING && m.target == "hand_limit" && m.mode == "add" && m.value == 1
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIAOJIAO2
            && t.event == "phase_draw_end"
            && t.filter == "owner_is_target"
            && !t.optional
    }));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == MIZHILIANOU
            && v.as_kind == "tao"
            && v.filter == "heart"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == GUDONGGENG && t.event == "damage_before" && !t.optional }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == QINGTUAN && t.event == "card_used"));
}

#[test]
fn shahai_publishes_scatter_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUIYANFADAO
            && t.event == "damage_before"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == GUAILI && m.target == "sha_damage" && m.mode == "add" && m.value == 1
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == GUAILI && t.event == "damage" && !t.optional));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUIQI && t.event == "damage" && t.filter == "owner_is_target"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LINGZHOU
            && t.event == "card_used"
            && t.filter == "owner_source_ordinary_trick"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZIYU && t.event == "phase_end" && t.filter == "owner_alive"
    }));
}

#[test]
fn shahai_publishes_dying_and_active_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    for def in [NVWASHI, SHOUWANG] {
        assert!(
            package
                .trigger_index
                .iter()
                .any(|t| t.skill_def_id == def && t.event == "dying" && t.optional),
            "dying trigger missing for {def}"
        );
    }
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIELEI && t.event == "card_used" && t.filter == "owner_source_sha"
    }));
    assert!(package.active_index.iter().any(|a| a.skill_def_id == YULU));
}

#[test]
fn shahai_publishes_batch39_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YUELU && t.event == "dying" && t.filter == "owner_alive" && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == KBOLAN && m.target == "draw_count" && m.mode == "add" && m.value == 1
    }));
    assert!(package.active_index.iter().any(|a| a.skill_def_id == DIEWU));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == DIEWU && t.event == "turn_ended" && !t.optional }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIYONG && t.event == "phase_end" && t.filter == "owner_is_target"
    }));
}

#[test]
fn shahai_publishes_batch40_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUAJIAN
            && t.event == "phase_play_end"
            && t.filter == "owner_is_target"
            && t.optional
    }));
    for def in [GUOZAI, GUOZAIX] {
        assert!(package.active_index.iter().any(|a| a.skill_def_id == def));
        assert!(
            package
                .trigger_index
                .iter()
                .any(|t| t.skill_def_id == def && t.event == "phase_play_end" && !t.optional),
            "guozai debt trigger missing for {def}"
        );
    }
}

#[test]
fn shahai_publishes_batch41_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LIANZHAN
            && t.event == "damage"
            && t.filter == "owner_is_source"
            && !t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == LIANZHAN && t.event == "turn_ended" && !t.optional }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == LIECHAO));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == LINYUN
            && v.as_kind == "sha"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NUHOU && t.event == "damage" && t.filter == "owner_is_target"
    }));
}

#[test]
fn shahai_publishes_batch42_five() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == QINGLIU
            && m.target == "damage_taken_cancel"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "override"
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == CHILONGYA
            && m.target == "damage_amount"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DIYCHANYUAN
            && t.event == "death"
            && t.filter == "owner_is_target_source_alive"
            && !t.optional
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == SHENGDUN && t.event == "phase_prepare" && !t.optional }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LINGYUE
            && t.event == "card_targeted"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
}

#[test]
fn shahai_publishes_batch43_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == WANJUN
            && v.as_kind == "nanman"
            && v.filter == "card_equip"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
    // 批 46 收紧为红色限定过滤器。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XFANGHUA
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other_red_card"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CANGCHIZHIBI_E5
            && t.event == "phase_end"
            && t.filter == "owner_alive"
            && t.optional
    }));
}

#[test]
fn shahai_publishes_batch44_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JJLD_E1
            && t.event == "damage"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    // 新过滤器 owner_target_sha 首消费者。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JJLD_E2
            && t.event == "damage"
            && t.filter == "owner_target_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == XUANYAN
            && m.target == "damage_amount"
            && m.selector.as_deref() == Some("fire")
            && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUANYAN && t.event == "damage" && t.filter == "owner_source_fire"
    }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == REXUE && t.event == "phase_prepare" && t.optional }));
}

#[test]
fn shahai_publishes_batch45_target_sha_backfill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // xingtigaizao_equip2：受击杀伤害摸 1（forced，新过滤器第二消费者）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XTGZ_E2
            && t.event == "damage"
            && t.filter == "owner_target_sha"
            && !t.optional
    }));
    // gudonggeng：damage_before 收紧为 owner_target_sha（杀限定）。
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUDONGGENG
            && t.event == "damage_before"
            && t.filter == "owner_target_sha"
            && !t.optional
    }));
}

#[test]
fn shahai_publishes_batch46_red_targeted() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    // 新过滤器 owner_alive_targeted_by_other_red_card 两消费者收紧。
    for def in [DIY_JIAOXIA, XFANGHUA] {
        assert!(
            package.trigger_index.iter().any(|t| {
                t.skill_def_id == def
                    && t.event == "card_targeted"
                    && t.filter == "owner_alive_targeted_by_other_red_card"
            }),
            "red-targeted tighten missing for {def}"
        );
    }
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NINGSHUANG
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other_black_card"
            && t.optional
    }));
}

#[test]
fn shahai_publishes_counter_family_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SONG
            && t.event == "damage_before"
            && t.filter == "owner_is_source"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == YA && t.event == "damage_before" && t.filter == "owner_is_target"
    }));
    assert!(package
        .modifier_index
        .iter()
        .any(|m| { m.skill_def_id == FENG && m.target == "draw_count" && m.mode == "add" }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| t.skill_def_id == FENG && t.event == "cards_drawn"));
}

#[test]
fn shahai_publishes_trick_damage_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHENGTENG
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUAJING
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BINGSHUANG
            && t.event == "damage"
            && t.filter == "owner_source_trick"
            && t.optional
    }));
}

#[test]
fn shahai_publishes_batch49_four() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZUIZHAN
            && t.event == "card_used"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == SHENQIANG && m.target == "sha_use_limit" && m.mode == "add"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHONGJI
            && t.event == "damage_before"
            && t.filter == "owner_is_source"
            && t.optional
    }));
    assert!(package
        .modifier_index
        .iter()
        .any(|m| { m.skill_def_id == HUAJIN && m.target == "damage_amount" && m.mode == "add" }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == HUAJIN));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == HUAJIN && t.event == "phase_prepare" && !t.optional }));
}

#[test]
fn shahai_publishes_batch50_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TUTENG4
            && t.event == "damage_before"
            && t.filter == "owner_source_trick"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TUTENG6
            && t.event == "damage_before"
            && t.filter == "owner_source_sha"
            && !t.optional
    }));
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == HUANXIA
            && v.as_kind == "sha"
            && v.filter == "red"
            && v.source_zone.as_deref() == Some("hand_equip")
    }));
}

#[test]
fn shahai_publishes_batch51_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == SHENGYAN && t.event == "hp_recovered" && t.optional }));
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == SHENGYAN && t.event == "turn_ended" && !t.optional }));
    let xiongzi_mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == XIONGZI && m.target == "draw_count" && m.mode == "add")
        .collect();
    assert_eq!(xiongzi_mods.len(), 2);
    assert!(xiongzi_mods.iter().any(|m| m.value == 1));
    assert!(xiongzi_mods
        .iter()
        .any(|m| m.value_source.as_deref() == Some("owner_equip_count_half_floor")));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == SHUIMOMUXUE
            && m.target == "hand_limit"
            && m.mode == "add"
            && m.value == -1
    }));
}

#[test]
fn shahai_publishes_batch52_two_partials() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == QIANGGU));
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == MIEDAO
            && m.target == "draw_count"
            && m.mode == "add"
            && m.value_source.as_deref() == Some("owner_lost_hp")
    }));
}

#[test]
fn mobile_publishes_rw_equip_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    // "hand" 为默认源区，序列化时省略（None）。
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == RW_LANYINJIA
            && v.as_kind == "shan"
            && v.source_zone.as_deref().is_none_or(|z| z == "hand")
    }));
    for selector in ["sha_fire", "huogong"] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == RW_MINGUANGKAI
                    && m.target == "card_target_cancel"
                    && m.selector.as_deref() == Some(selector)
                    && m.mode == "override"
            }),
            "missing target-cancel for {selector}"
        );
    }
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == RW_MINGUANGKAI && m.target == "link_loss_cancel" && m.mode == "override"
    }));
}

#[test]
fn shahai_publishes_lxianglong_dodged_counter() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 657);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == LXIANG_LONG
            && t.event == "sha_dodged"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}
