//! O2 全量合并第二批（2026-10-14）：家族 stale 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_stale_gap_batch {
    //! 2026-08-15 stale-gap batch: three skills whose noop blockers referenced
    //! primitives that already existed (remove_mark/mark_count, choose(1,2),
    //! choose_other_draw_or_recover, choose_add_or_remove_current_card_target).

    const PACKAGE_REFRESH: &[u8] =
        include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
    const PACKAGE_KEY: &[u8] =
        include_bytes!("../../../content/original/ported/key/key.npack.json");

    #[test]
    fn committed_packages_publish_stale_gap_family_triggers() {
        let refresh = content_schema::ContentPackage::from_json_bytes(PACKAGE_REFRESH).unwrap();
        let chunlaox = refresh
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_527_886_704)
            .expect("decadechunlao2 trigger");
        assert_eq!(chunlaox.event, "damage");
        assert_eq!(chunlaox.filter, "owner_is_source");
        assert!(chunlaox.optional);
        let key = content_schema::ContentPackage::from_json_bytes(PACKAGE_KEY).unwrap();
        let wuxin = key
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_940_257_990)
            .expect("yuzuru_wuxin trigger");
        assert_eq!(wuxin.event, "phase_end");
        assert_eq!(wuxin.filter, "owner_is_target");
        assert!(wuxin.optional);
        let tiankou = key
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 2_861_751_685)
            .expect("komari_tiankou trigger");
        assert_eq!(tiankou.event, "card_used");
        assert_eq!(tiankou.filter, "owner_is_source");
        assert!(tiankou.optional);
    }

    #[test]
    fn ollihuo_publishes_expansion_place_contract() {
        // 契约级：place_hand_card_to_expansion（ABI 650/tag 669/Cont 269）随包发布，
        // ollihuo 触发器（card_used owner_source_sha optional）在包内。
        // 行为链（确认→选牌→SkillPileCount+1）待行为验证：合成 CardUsed 事件未触发
        // owner_source_sha 过滤器（原因未排查，可能需真实 PlayCard 管线驱动），
        // 已在 delta 文档登记。
        let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE_REFRESH).unwrap();
        assert_eq!(pkg.manifest.rule_vm_abi_version, 723);
        // 2026-08-15 疠火重组：主技=转火 modifier + 火杀加目标；首杀置入条款移至子技 ollihuo4。
        let trigger = pkg
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 4_066_393_641)
            .expect("ollihuo trigger");
        assert_eq!(trigger.event, "card_used");
        assert_eq!(trigger.filter, "owner_source_fire_sha");
        assert!(trigger.optional);
        let sub = pkg
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_911_445_524 && t.filter == "owner_source_sha")
            .expect("ollihuo4 first-sha place trigger");
        assert!(sub.optional);
    }

    #[test]
    fn judgment_rewrite_mini_batch_publishes_modifiers() {
        // 判定交互迷你批（§6d 解体为零原语）：解兆/终愿经既有 judgment_rewrite
        // modifier 目标获得判定改写资格（RewriteJudgment 续体引擎原生支持）。
        let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE_KEY).unwrap();
        for (name, def) in [
            ("shiroha_jiezhao", 4_008_645_358u32),
            ("misuzu_zhongyuan", 2_446_245_954),
        ] {
            let entry = pkg
                .modifier_index
                .iter()
                .find(|m| m.skill_def_id == def)
                .unwrap_or_else(|| panic!("{name} modifier missing"));
            assert_eq!(entry.target, "judgment_rewrite", "{name}");
        }
    }

    #[test]
    fn fire_sha_family_publishes_triggers() {
        // 火杀体查询批次（第七次白名单解体：owner_source_fire_sha/owner_source_fire 早已在位）。
        let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE_REFRESH).unwrap();
        let zeng = pkg
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_239_729_881)
            .expect("decadelihuo2 trigger");
        assert_eq!(zeng.event, "card_used");
        assert_eq!(zeng.filter, "owner_source_fire_sha");
        assert!(zeng.optional);
        let sun = pkg
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 4_165_258_449)
            .expect("decadelihuo3 trigger");
        assert_eq!(sun.event, "damage");
        assert_eq!(sun.filter, "owner_source_fire");
        assert!(sun.optional);
    }

    #[test]
    fn nature_write_family_publishes_modifiers() {
        // nature 写入批次（第八次白名单解体：card_nature modifier 目标早已在位）。
        let refresh = content_schema::ContentPackage::from_json_bytes(PACKAGE_REFRESH).unwrap();
        let zhuan = refresh
            .modifier_index
            .iter()
            .find(|m| m.skill_def_id == 2_548_121_026)
            .expect("decadelihuo modifier");
        assert_eq!(zhuan.target, "card_nature");
        assert_eq!(zhuan.selector.as_deref(), Some("sha"));
        let onlyol = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/onlyOL/onlyOL.npack.json"
        ))
        .unwrap();
        let lihuo_mod = onlyol
            .modifier_index
            .iter()
            .find(|m| m.skill_def_id == 2_601_425_521)
            .expect("dclihuo modifier");
        assert_eq!(lihuo_mod.target, "card_nature");
        assert_eq!(lihuo_mod.selector.as_deref(), Some("sha"));
        assert!(onlyol.trigger_index.iter().any(|t| {
            t.skill_def_id == 2_601_425_521 && t.filter == "owner_source_fire_sha" && t.optional
        }));
    }

    #[test]
    fn consolidated_sweep_batch_publishes_contracts() {
        // 终极合并反向扫描批次：clanfenchai（judgment_rewrite 变体）+ dcxingwu（place 前半）。
        let clan = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/clan/clan.npack.json"
        ))
        .unwrap();
        let fenchai = clan
            .modifier_index
            .iter()
            .find(|m| m.skill_def_id == 2_281_018_381)
            .expect("clanfenchai modifier");
        assert_eq!(fenchai.target, "judgment_rewrite_red_hand_equip_expansion");
        let xianding = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/xianding/xianding.npack.json"
        ))
        .unwrap();
        let xingwu = xianding
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 4_112_903_133)
            .expect("dcxingwu trigger");
        assert_eq!(xingwu.event, "phase_discard");
        assert!(xingwu.optional);
    }

    #[test]
    fn expansion_p1_gain_batch_publishes_contracts() {
        // 2026-08-16 P1 取用批次（用户批准门槛降至实面 2）：
        // GainFromSkillPile（tag 671/ABI 651）+ dchaoshi/ao_diegui。
        let collab = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/collab/collab.npack.json"
        ))
        .unwrap();
        assert_eq!(collab.manifest.rule_vm_abi_version, 694);
        let place_trigger = collab
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_952_022_861 && t.filter == "owner_is_target")
            .expect("dchaoshi damage trigger");
        assert_eq!(place_trigger.event, "damage");
        let gain_trigger = collab
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == 3_952_022_861 && t.event == "phase_end")
            .expect("dchaoshi gain trigger");
        assert!(gain_trigger.optional);
        let key = content_schema::ContentPackage::from_json_bytes(PACKAGE_KEY).unwrap();
        assert_eq!(key.manifest.rule_vm_abi_version, 691);
        let active = key
            .active_index
            .iter()
            .find(|a| a.skill_def_id == 3_867_140_701)
            .expect("ao_diegui active");
        assert_eq!(active.window, "play_phase");
    }
}
