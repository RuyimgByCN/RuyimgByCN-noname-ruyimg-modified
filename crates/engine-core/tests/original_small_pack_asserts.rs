//! O2 试点合并（2026-10-14）：原 8 个独立小测试文件平铺合并为单一
//! 链接目标（1,078→1,071 个测试二进制）。**#[test] 函数名逐一不变**
//!（cargo test 过滤行为不变）；各文件顶层 const 按来源加前缀防碰撞；
//! 测试数量与断言逐字保留。见
//! docs/operations/evidence/2026-10-14-o2-build-efficiency/README.md。


// Golden coverage for mobile 连袭 partial rewrite (2026-08-21 placeholder-header batch):
// 【杀】不计入手牌上限 via `modifier sha_ignored_hand_limit override 1`.
// 第二句（杀被弃置后视为使用无距无次杀）无对应过滤器，未建模（近似注记在 .skill 头）。

// --- 合并自 original_mobile_lianxi_rewrite.rs（const 前缀 LIANXI_M_）---
const LIANXI_M_LIANXI: u32 = 289_965_7484;
const LIANXI_M_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn lianxi_source_and_npack_publish_sha_ignored_hand_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(LIANXI_M_PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == LIANXI_M_LIANXI
            && modifier.target == "sha_ignored_hand_limit"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
}

// 第二百八十八批：yixiang 义襄随机获得族状态机契约测试（ABI 734 / tag 755，
// 单发无续体）。

// --- 合并自 original_yixiang_machine.rs（const 前缀 YIXIANG_M_）---
const YIXIANG_M_YIXIANG: u32 = 2_667_775_149;
const YIXIANG_M_MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn package_publishes_yixiang_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(YIXIANG_M_MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YIXIANG_M_YIXIANG)
        .expect("yixiang trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_source_hp_gt_owner_per_role_turn"
    );
}

// Golden coverage for REFRESH「资守·制衡」追加技.

// --- 合并自 original_refresh_decadezishou_zhiheng.rs（const 前缀 ZZSH_）---
use content_schema::ContentPackage;
const ZZSH_DECADEZISHOU_ZHIHENG: u32 = 3_869_142_148;
const ZZSH_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/decadezishou_zhiheng.skill");
const ZZSH_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn source_and_npack_publish_end_phase_recast() {
    let compiled = content_compiler::compile_source_named("decadezishou_zhiheng", ZZSH_SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZZSH_DECADEZISHOU_ZHIHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);
    assert_eq!(compiled.skill_index[0].skill_def_id, ZZSH_DECADEZISHOU_ZHIHENG);

    let pack = ContentPackage::from_json_bytes(ZZSH_PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZZSH_DECADEZISHOU_ZHIHENG && trigger.event == "phase_end"
    }));
}

// Source-exact TW character skill lists without exposed implementation helpers.

// --- 合并自 original_tw_character_helper_folding.rs（const 前缀 TW_FOLD_）---
const TW_FOLD_TW_ZHOUCHU: u32 = 3_263_858_015;
const TW_FOLD_TWGUOYI: u32 = 3_002_953_620;
const TW_FOLD_TWCHUHAI: u32 = 3_782_702_310;
const TW_FOLD_TW_QIAORUI: u32 = 3_792_078_394;
const TW_FOLD_TWXIAWEI: u32 = 3_007_668_730;
const TW_FOLD_TWQIONGJI: u32 = 2_723_606_029;
const TW_FOLD_OLD_TWCHUHAI_HELPER: u32 = 3_469_371_130;
const TW_FOLD_OLD_TWQIONGJI_HELPER: u32 = 3_469_371_140;
const TW_FOLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

#[test]
fn package_folds_helpers_into_source_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(TW_FOLD_PACKAGE).unwrap();
    let skills = |id| {
        package
            .character_index
            .iter()
            .find(|character| character.id == id)
            .unwrap()
            .skills
            .as_slice()
    };
    assert_eq!(skills(TW_FOLD_TW_ZHOUCHU), [TW_FOLD_TWGUOYI, TW_FOLD_TWCHUHAI]);
    assert_eq!(skills(TW_FOLD_TW_QIAORUI), [TW_FOLD_TWXIAWEI, TW_FOLD_TWQIONGJI]);
    assert!(!package
        .skill_index
        .iter()
        .any(|skill| [TW_FOLD_OLD_TWCHUHAI_HELPER, TW_FOLD_OLD_TWQIONGJI_HELPER].contains(&skill.skill_def_id)));
}

// Character closure for REFRESH 界貂蝉 and SP 邹氏.

// --- 合并自 original_refresh_re_diaochan_sp_zoushi.rs（const 前缀 RC_SPZ_）---
const RC_SPZ_REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const RC_SPZ_SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn committed_packages_publish_exact_legacy_character_definitions() {
    let refresh = content_schema::ContentPackage::from_json_bytes(RC_SPZ_REFRESH).unwrap();
    let diaochan = refresh
        .character_index
        .iter()
        .find(|character| character.id == 2_743_786_675)
        .expect("re_diaochan must be published");
    assert_eq!(diaochan.name_key, "re_diaochan");
    assert_eq!(diaochan.sex, "female");
    assert_eq!(diaochan.group, "qun");
    assert_eq!(diaochan.max_hp, 3);
    assert_eq!(diaochan.skills, [2_292_478_631, 4_285_757_447]);

    let sp = content_schema::ContentPackage::from_json_bytes(RC_SPZ_SP).unwrap();
    let zoushi = sp
        .character_index
        .iter()
        .find(|character| character.id == 2_442_478_990)
        .expect("zoushi must be published");
    assert_eq!(zoushi.name_key, "zoushi");
    assert_eq!(zoushi.sex, "female");
    assert_eq!(zoushi.group, "qun");
    assert_eq!(zoushi.max_hp, 3);
    assert!(zoushi.skills.is_empty());
}

// Golden coverage for extension characters whose complete skill dependencies are already ported.

// --- 合并自 original_ready_character_entries.rs（const 前缀 READY_）---
#[test]
fn old_yujin_and_offline_lord_lvbu_keep_exact_character_metadata() {
    let old = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/old/old.npack.json"
    ))
    .unwrap();
    let yujin = old
        .character_index
        .iter()
        .find(|character| character.id == 3_388_457_974)
        .expect("毅重于禁 should be packaged");
    assert_eq!(yujin.name_key, "re_yujin");
    assert_eq!((yujin.sex.as_str(), yujin.group.as_str()), ("male", "wei"));
    assert_eq!(yujin.max_hp, 4);
    assert_eq!(yujin.skills, vec![2_156_296_205]);
    assert!(!yujin.is_lord);

    let offline = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/offline/offline.npack.json"
    ))
    .unwrap();
    let lvbu = offline
        .character_index
        .iter()
        .find(|character| character.id == 2_640_290_591)
        .expect("君吕布 should be packaged");
    assert_eq!(lvbu.name_key, "jun_lvbu");
    assert_eq!((lvbu.sex.as_str(), lvbu.group.as_str()), ("male", "qun"));
    assert_eq!(lvbu.max_hp, 4);
    assert_eq!(lvbu.skills, vec![18, 4_273_235_044]);
    assert!(lvbu.is_lord);
}

// Exact source character closure for three previously partial SP2 entries.

// --- 合并自 original_sp2_character_skill_closure.rs（const 前缀 SP2_CLOSURE_）---
const SP2_CLOSURE_DC_JSP_GUANYU: u32 = 4_047_577_994;
const SP2_CLOSURE_NEW_REWUSHENG: u32 = 3_038_912_803;
const SP2_CLOSURE_DCDANJI: u32 = 3_851_102_612;
const SP2_CLOSURE_ZHANGWEN: u32 = 2_318_285_269;
const SP2_CLOSURE_SONGSHU: u32 = 3_706_775_979;
const SP2_CLOSURE_SIBIAN: u32 = 3_021_138_179;
const SP2_CLOSURE_ZHUJUN: u32 = 2_614_075_935;
const SP2_CLOSURE_GONGJIAN: u32 = 3_309_202_552;
const SP2_CLOSURE_KUIMANG: u32 = 2_972_733_766;
const SP2_CLOSURE_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn package_preserves_all_source_character_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(SP2_CLOSURE_PACKAGE).unwrap();
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    let skills = |id| {
        package
            .character_index
            .iter()
            .find(|character| character.id == id)
            .unwrap()
            .skills
            .as_slice()
    };
    assert_eq!(skills(SP2_CLOSURE_DC_JSP_GUANYU), [SP2_CLOSURE_NEW_REWUSHENG, SP2_CLOSURE_DCDANJI]);
    assert_eq!(skills(SP2_CLOSURE_ZHANGWEN), [SP2_CLOSURE_SONGSHU, SP2_CLOSURE_SIBIAN]);
    assert_eq!(skills(SP2_CLOSURE_ZHUJUN), [SP2_CLOSURE_GONGJIAN, SP2_CLOSURE_KUIMANG]);
}

// Cross-package character closure for HUICUI 孟优 and 界阚泽.

// --- 合并自 original_huicui_cross_package_character_closure.rs（const 前缀 HUICUI_）---
const HUICUI_MENGYOU: u32 = 3_321_725_449;
const HUICUI_MANYI: u32 = 3_456_707_526;
const HUICUI_DCMANZHI: u32 = 4_194_306_664;
const HUICUI_RE_KANZE: u32 = 3_301_308_945;
const HUICUI_XIASHU: u32 = 2_873_874_515;
const HUICUI_REKUANSHI: u32 = 3_423_208_328;
const HUICUI_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

#[test]
fn package_declares_dependencies_and_complete_skill_lists() {
    let package = content_schema::ContentPackage::from_json_bytes(HUICUI_PACKAGE).unwrap();
    let dependencies: Vec<_> = package
        .manifest
        .dependencies
        .iter()
        .map(|dependency| dependency.package_id.as_str())
        .collect();
    assert!(dependencies.contains(&"xianding"));
    assert!(dependencies.contains(&"sp"));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUICUI_MENGYOU)
            .unwrap()
            .skills,
        [HUICUI_MANYI, HUICUI_DCMANZHI]
    );
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUICUI_RE_KANZE)
            .unwrap()
            .skills,
        [HUICUI_XIASHU, HUICUI_REKUANSHI]
    );
}
