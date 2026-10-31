//! O2 全量合并第二批（2026-10-14）：家族 character 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_character_roster_closure {
    //! Golden: catalog character-roster closure after skill-order fix.
    //!
    //! Spot-checks previously source_synced characters whose skills were already
    //! ported but roster skill def_id order mismatched resolved_skills.

    use content_schema::ContentPackage;

    const OFFLINE: &[u8] =
        include_bytes!("../../../content/original/ported/offline/offline.npack.json");
    const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
    const CLAN: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

    const CA_LVBU: u32 = 2_521_438_895;
    const WUSHUANG: u32 = 18;
    const CALIYU: u32 = 3_404_441_188;

    const GONGSUNZAN: u32 = 3_561_346_810;
    const REYICONG: u32 = 3_426_755_583;

    const CLAN_ZHUGEZHAN: u32 = 4_147_014_673;
    const CLANCHENGWANG: u32 = 2_831_451_765;
    const CLANFENSHI: u32 = 2_827_730_630;

    #[test]
    fn offline_ca_lvbu_publishes_cross_pack_wushuang_then_caliyu() {
        let package = ContentPackage::from_json_bytes(OFFLINE).unwrap();
        let c = package
            .character_index
            .iter()
            .find(|c| c.name_key == "ca_lvbu" && c.id == CA_LVBU)
            .expect("ca_lvbu");
        // Legacy skill order: wushuang, caliyu (standard then offline)
        assert_eq!(c.skills, vec![WUSHUANG, CALIYU]);
        assert_eq!(c.portrait_asset.as_deref(), Some("character_ca_lvbu.jpg"));
    }

    #[test]
    fn standard_gongsunzan_uses_refresh_reyicong() {
        let package = ContentPackage::from_json_bytes(STANDARD).unwrap();
        let c = package
            .character_index
            .iter()
            .find(|c| c.name_key == "gongsunzan" && c.id == GONGSUNZAN)
            .expect("gongsunzan");
        assert_eq!(c.skills, vec![REYICONG]);
        assert_eq!(
            c.portrait_asset.as_deref(),
            Some("character_gongsunzan.jpg")
        );
        assert!(package.asset_paths.contains_key("character_gongsunzan.jpg"));
    }

    #[test]
    fn clan_zhugezhan_includes_chengwang_and_upstream_clanfenshi() {
        let package = ContentPackage::from_json_bytes(CLAN).unwrap();
        // Upstream 1.11.5 replaced the unpublished clanzhugezhan stub with the
        // shared 纷势 (clanfenshi) clan skill; the stub must be gone.
        assert!(!package
            .skill_index
            .iter()
            .any(|s| s.name == "clanzhugezhan"));
        assert!(package
            .skill_index
            .iter()
            .any(|s| s.skill_def_id == CLANFENSHI && s.name == "clanfenshi"));
        let c = package
            .character_index
            .iter()
            .find(|c| c.name_key == "clan_zhugezhan" && c.id == CLAN_ZHUGEZHAN)
            .expect("clan_zhugezhan");
        assert_eq!(c.skills, vec![CLANCHENGWANG, CLANFENSHI]);
    }
}
