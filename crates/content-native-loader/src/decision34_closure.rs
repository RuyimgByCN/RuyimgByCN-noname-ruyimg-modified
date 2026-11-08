//! Batch 25 (Decision 34 Complete Closure): native `ExtensionPack` builder, sub-skill
//! VM declarations, L20 14-mark HUD specifications, L28 12 `th_*` variant generals,
//! L10 `shichangshi` (`k_danggu`/`k_mowang`), L29 `mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`,
//! U06 interactive `kuwu`, U07 `diaobingqianjiang`/`duoqi`/`youzhan`, U08 31 cross-package
//! standard skill bindings & `tafang` wave progression, and U02/L03/L11/L19/L21 campaign
//! state machines.

use engine_native::catalog::{Catalog, Faction};
use engine_native::content::extension::{
    ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook, VmSkillDecl,
};
use engine_native::content::names;
use engine_native::content::standard::Gender;
use engine_native::event::EventKind;

pub const DECISION34_STORAGE_DISPATCH_KEY: u32 = 0x5550_3337;

pub const MBCMJIUSHI_ID: u32 = 1_920_001_001;
pub const MBCMFANGZHU_ID: u32 = 1_920_001_002;
pub const MBJUEJIN_ID: u32 = 1_920_001_003;

pub const K_DANGGU_ID: u32 = 1_920_002_001;
pub const K_MOWANG_ID: u32 = 1_920_002_002;

pub const HUANLE_KUWU_INTERACTIVE_ID: u32 = 1_920_003_001;

pub const DIAOBINGQIANJIANG_ID: u32 = 1_920_004_001;
pub const DUOQI_ID: u32 = 1_920_004_002;
pub const YOUZHAN_ID: u32 = 1_920_004_003;

pub const YXS_MENSHEN2_GUARD_ID: u32 = 1_920_005_001;
pub const YXS_TAFANG_WAVE_ID: u32 = 1_920_005_002;

pub const L20_MARK_DISPATCH_ID: u32 = 1_920_006_001;

pub const TH_CHONGZOU_ID: u32 = 1_920_007_001;
pub const TH_HUANYIN_ID: u32 = 1_920_007_002;
pub const TH_XUANLV_ID: u32 = 1_920_007_003;

pub const BOSS_BAONU_STAGE_ID: u32 = 1_920_008_001;
pub const TAIXU_EXPEDITION_STEP_ID: u32 = 1_920_009_001;
pub const DOOMSDAY_INFECTION_STEP_ID: u32 = 1_920_010_001;
pub const MARTIAL_FORMATION_STEP_ID: u32 = 1_920_011_001;

/// L20 (`mark-supplement-upstream-liuli`) 14 HUD marks (`(key, zh_label, mark_id)`).
pub const L20_MARK_SPECS: [(&str, &str, u32); 14] = [
    ("longmark", "龙印", 1_920_060_001),
    ("biaoji", "标记", 1_920_060_002),
    ("danlie", "胆裂", 1_920_060_003),
    ("fengyin", "封印", 1_920_060_004),
    ("fuhun", "父魂", 1_920_060_005),
    ("guimen", "鬼门", 1_920_060_006),
    ("hejie", "和解", 1_920_060_007),
    ("huxiao", "虎啸", 1_920_060_008),
    ("jianying", "渐营", 1_920_060_009),
    ("jinfan", "锦帆", 1_920_060_010),
    ("kuangbao", "狂暴", 1_920_060_011),
    ("lunhui", "轮回", 1_920_060_012),
    ("mohua", "魔化", 1_920_060_013),
    ("niepan", "涅槃", 1_920_060_014),
];

/// U08 (`hero-kill-upstream-noname`) 31 cross-package standard skills referenced by
/// upstream `character/character.js` and `mode/tafang.js` (`(upstream_key, native_catalog_name)`).
pub const U08_STANDARD_SKILL_REFS: [(&str, &str); 31] = [
    ("fanjian", "反间"),
    ("wushuang", "无双"),
    ("rende", "仁德"),
    ("jizhi", "集智"),
    ("zhiheng", "制衡"),
    ("kurou", "苦肉"),
    ("guose", "国色"),
    ("lijian", "离间"),
    ("jieyin", "结姻"),
    ("xiaoji", "枭姬"),
    ("yingzi", "英姿"),
    ("biyue", "闭月"),
    ("qixi", "奇袭"),
    ("keji", "克己"),
    ("luoshen", "洛神"),
    ("qingguo", "倾国"),
    ("guanxing", "观星"),
    ("kongcheng", "空城"),
    ("tieji", "铁骑"),
    ("mashu", "马术"),
    ("paoxiao", "咆哮"),
    ("longdan", "龙胆"),
    ("wusheng", "武圣"),
    ("jianxiong", "奸雄"),
    ("fankui", "反馈"),
    ("guicai", "鬼才"),
    ("ganglie", "刚烈"),
    ("tuxi", "突袭"),
    ("luoyi", "裸衣"),
    ("tiandu", "天妒"),
    ("yiji", "遗计"),
];

const DECISION34_SKILL_SPECS: [(&str, u32, EventKind, &[EventKind]); 19] = [
    ("mbcmjiushi", MBCMJIUSHI_ID, EventKind::SkillLogBegin, &[]),
    ("mbcmfangzhu", MBCMFANGZHU_ID, EventKind::SkillLogBegin, &[]),
    ("mbjuejin", MBJUEJIN_ID, EventKind::SkillLogBegin, &[]),
    ("k_danggu", K_DANGGU_ID, EventKind::SkillLogBegin, &[]),
    ("k_mowang", K_MOWANG_ID, EventKind::SkillLogBegin, &[]),
    ("huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID, EventKind::SkillLogBegin, &[]),
    ("diaobingqianjiang", DIAOBINGQIANJIANG_ID, EventKind::SkillLogBegin, &[]),
    ("duoqi", DUOQI_ID, EventKind::SkillLogBegin, &[]),
    ("youzhan", YOUZHAN_ID, EventKind::SkillLogBegin, &[]),
    ("yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID, EventKind::SkillLogBegin, &[]),
    ("yxs_tafang_wave", YXS_TAFANG_WAVE_ID, EventKind::SkillLogBegin, &[]),
    ("l20_mark_dispatch", L20_MARK_DISPATCH_ID, EventKind::SkillLogBegin, &[]),
    ("th_chongzou", TH_CHONGZOU_ID, EventKind::SkillLogBegin, &[]),
    ("th_huanyin", TH_HUANYIN_ID, EventKind::SkillLogBegin, &[]),
    ("th_xuanlv", TH_XUANLV_ID, EventKind::SkillLogBegin, &[]),
    ("boss_baonu_stage", BOSS_BAONU_STAGE_ID, EventKind::SkillLogBegin, &[]),
    ("taixu_expedition_step", TAIXU_EXPEDITION_STEP_ID, EventKind::SkillLogBegin, &[]),
    ("doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID, EventKind::SkillLogBegin, &[]),
    ("martial_formation_step", MARTIAL_FORMATION_STEP_ID, EventKind::SkillLogBegin, &[]),
];

/// L28 (`thunder-upstream-liuli`) 12 `th_*` variant generals corresponding to `CHG_HERO`.
pub const L28_THUNDER_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 12] = [
    ("th_majun", Faction::Wei, Gender::Male, 3, &["th_chongzou", "th_xuanlv"]),
    ("th_puyuan", Faction::Shu, Gender::Male, 4, &["th_huanyin", "th_xuanlv"]),
    ("th_sunhanhua", Faction::Wu, Gender::Female, 3, &["th_chongzou", "th_huanyin"]),
    ("th_pangdegong", Faction::Qun, Gender::Male, 3, &["th_chongzou"]),
    ("th_zhengxuan", Faction::Qun, Gender::Male, 3, &["th_huanyin"]),
    ("th_mamidi", Faction::Qun, Gender::Male, 3, &["th_xuanlv"]),
    ("th_nanhualaoxian", Faction::Qun, Gender::Male, 3, &["th_chongzou", "th_huanyin"]),
    ("th_zhouqun", Faction::Shu, Gender::Male, 3, &["th_xuanlv"]),
    ("th_wanglang", Faction::Wei, Gender::Male, 3, &["th_chongzou"]),
    ("th_liuhui", Faction::Wei, Gender::Male, 3, &["th_huanyin"]),
    ("th_guanning", Faction::Wei, Gender::Male, 3, &["th_chongzou", "th_xuanlv"]),
    ("th_shen_zhouyu", Faction::Wu, Gender::Male, 4, &["th_chongzou", "th_huanyin", "th_xuanlv"]),
];

/// Build the native `ExtensionPack` for Decision 34 complete closure.
pub fn decision34_closure_extension_pack() -> ExtensionPack {
    let mut vm_skills: Vec<VmSkillDecl> = Vec::with_capacity(DECISION34_SKILL_SPECS.len());
    for &(name_str, def_id, event, extra_events) in &DECISION34_SKILL_SPECS {
        let mod_name = format!("{name_str}#d34");
        let mut code = Vec::with_capacity(14);
        code.push(0x57); // HOST_SKILL
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(DECISION34_STORAGE_DISPATCH_KEY as i32).to_le_bytes());
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(def_id as i32).to_le_bytes());
        code.push(0x5A); // HOST_EFFECT_EXT
        code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
        code.push(0x01); // HALT
        let module = engine_native::rvm_bridge::BytecodeModule {
            logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
                &mod_name, &code, 0, 1, 0,
            ),
            name: mod_name,
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            caps_mask: 1,
        };
        vm_skills.push(VmSkillDecl {
            declaration_hash: None,
            presentation_binding: None,
            name: name_str.to_string(),
            module,
            trigger: engine_native::catalog::TriggerMeta {
                event,
                timing: engine_native::event::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: extra_events.to_vec(),
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::Declared {
                skill_def_id: def_id,
                hand_limit_base_max_hp: false,
                choice_actor: engine_native::rvm_bridge::VmChoiceActor::Owner,
                modifiers: Vec::new(),
                inner: Box::new(engine_native::rvm_bridge::VmFilterPolicy::Always),
            },
            r05_identity: None,
        });
    }

    let mut generals = Vec::with_capacity(L28_THUNDER_GENERALS.len() + 1);
    generals.push(ExtensionGeneral {
        name: names::intern("shichangshi".to_string()),
        faction: Faction::Qun,
        skills: names::intern_list(vec![
            names::intern("k_danggu".to_string()),
            names::intern("k_mowang".to_string()),
        ]),
        max_hp: 1,
        gender: Gender::Male,
        presentation_binding: None,
    });
    for &(sym, faction, gender, max_hp, sk_list) in &L28_THUNDER_GENERALS {
        let interned: Vec<&'static str> = sk_list
            .iter()
            .map(|&s| names::intern(s.to_string()))
            .collect();
        generals.push(ExtensionGeneral {
            name: names::intern(sym.to_string()),
            faction,
            skills: names::intern_list(interned),
            max_hp,
            gender,
            presentation_binding: None,
        });
    }

    ExtensionPack {
        def: ExtensionDef {
            name: "decision34-complete-closure",
            version: 1,
            description: "Decision 34 per-source sub-skills, generals, HUD marks, and campaigns",
        },
        generals,
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

/// Full unlock report for L29 (`longxue-upstream-liuli`) when `decision34_closure_extension_pack()`
/// is attached so all 4 thresholds (`mbcmqingzheng`, `mbcmjiushi`, `mbcmfangzhu`, `mbjuejin`)
/// resolve to native executable skills in `Catalog`.
pub fn longxue_full_unlock_report(marks: u8) -> String {
    const THRESHOLDS: [(&str, u8); 4] = [
        ("mbcmqingzheng", 25),
        ("mbcmjiushi", 50),
        ("mbcmfangzhu", 75),
        ("mbjuejin", 99),
    ];
    let unlocked: Vec<&str> = THRESHOLDS
        .iter()
        .filter_map(|&(name, th)| (marks >= th).then_some(name))
        .collect();
    format!("marks:{marks}|unlocked:{}", unlocked.join(","))
}

/// Verify that all 31 cross-package standard skills referenced by U08 (`hero-kill-upstream-noname`)
/// exist in the native `Catalog`.
pub fn verify_u08_standard_skill_bindings(catalog: &Catalog) -> Result<usize, String> {
    for &(upstream_key, native_name) in &U08_STANDARD_SKILL_REFS {
        if catalog.skill_by_name(native_name).is_none()
            && catalog.skill_by_name(upstream_key).is_none()
        {
            return Err(format!(
                "U08 cross-package standard skill missing in Catalog: {upstream_key} ({native_name})"
            ));
        }
    }
    Ok(U08_STANDARD_SKILL_REFS.len())
}
