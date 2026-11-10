//! Decision 34 per-source authentic upstream sub-skills, generals, and cards for
//! `L29` (`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`), `L10` (`shichangshi`: `k_danggu`/`k_mowang`),
//! `U06` (`huanle_kuwu_interactive`), `U07` (`diaobingqianjiang`/`duoqi`/`youzhan`),
//! `U08` (`yxs_menshen2_guard` & 31 cross-package standard skill bindings),
//! `L20` (14 HUD marks), and `L28` (`雷霆万钧/extension.js:3657-3670,3783-8968` 14 `th_*`
//! generals, skills, and `th_taipingyaoshu` card).

use engine_native::catalog::{CardDef, CardKind, Catalog, Faction, Suit};
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
pub const L10_SCS_SKILL_ID: u32 = 1_920_002_003;

/// `L10` (`史诗卡牌/extension.js:249-310`) Ten Eunuchs (`十常侍`) companion skills granted by `k_danggu`.
pub const L10_SCS_COMPANION_SKILLS: [&str; 10] = [
    "scstaoluan",
    "scschiyan",
    "scszimou",
    "scspicai",
    "scsyaozhuo",
    "scsxiaolu",
    "scskuiji",
    "scschihe",
    "scsniqu",
    "scsanruo",
];

pub const HUANLE_KUWU_INTERACTIVE_ID: u32 = 1_920_003_001;

pub const DIAOBINGQIANJIANG_ID: u32 = 1_920_004_001;
pub const DUOQI_ID: u32 = 1_920_004_002;
pub const YOUZHAN_ID: u32 = 1_920_004_003;

pub const YXS_MENSHEN2_GUARD_ID: u32 = 1_920_005_001;

pub const L20_MARK_DISPATCH_ID: u32 = 1_920_006_001;

/// Authentic upstream `L28` (`雷霆万钧/extension.js:3783-8968`) skill dispatch IDs.
pub const TH_DUNSHI_ID: u32 = 1_920_007_011;
pub const TH_JINGXIE_ID: u32 = 1_920_007_012;
pub const TH_QIAOSI_ID: u32 = 1_920_007_013;
pub const TH_YEYAN_ID: u32 = 1_920_007_014;
pub const TH_TAIPINGYAOSHU_ID: u32 = 1_920_007_015;
pub const TH_SKILL_DISPATCH_ID: u32 = 1_920_007_016;

/// Authentic upstream `L01` (`AI优化/extension.js:9205-9260`), `L06` (`五行生克/extension.js:18-167`),
/// `U06`, `U07`, `U08`, and `L29` dispatch IDs.
pub const L01_YIYI_ID: u32 = 1_920_008_001;
pub const L06_WUXING_ID: u32 = 1_920_008_002;
pub const U06_HUANLE_SKILL_ID: u32 = 1_920_008_003;
pub const U07_WUDIAN_SKILL_ID: u32 = 1_920_008_004;
pub const U08_YXS_SKILL_ID: u32 = 1_920_008_005;
pub const L29_LONGXUE_SKILL_ID: u32 = 1_920_008_006;

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
/// upstream `character/character.js` (`(upstream_key, native_catalog_name)`).
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

const DECISION34_SKILL_SPECS: [(
    &str,
    u32,
    EventKind,
    &[EventKind],
    engine_native::rvm_bridge::VmFilterPolicy,
); 16] = [
    (
        "mbcmjiushi",
        MBCMJIUSHI_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "mbcmfangzhu",
        MBCMFANGZHU_ID,
        EventKind::Damage,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "mbjuejin",
        MBJUEJIN_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "k_danggu",
        K_DANGGU_ID,
        EventKind::TurnBegin,
        &[EventKind::PhaseBegin],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "k_mowang",
        K_MOWANG_ID,
        EventKind::Dying,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "huanle_kuwu_interactive",
        HUANLE_KUWU_INTERACTIVE_ID,
        EventKind::Damage,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsSource,
    ),
    (
        "diaobingqianjiang",
        DIAOBINGQIANJIANG_ID,
        EventKind::CardUsed,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsSource,
    ),
    (
        "duoqi",
        DUOQI_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "youzhan",
        YOUZHAN_ID,
        EventKind::Damage,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "yxs_menshen2_guard",
        YXS_MENSHEN2_GUARD_ID,
        EventKind::Damage,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "l20_mark_dispatch",
        L20_MARK_DISPATCH_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "th_dunshi",
        TH_DUNSHI_ID,
        EventKind::CardUsed,
        &[EventKind::Damage],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsSource,
    ),
    (
        "th_jingxie",
        TH_JINGXIE_ID,
        EventKind::PhaseBegin,
        &[EventKind::Dying],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "th_qiaosi",
        TH_QIAOSI_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "th_yeyan",
        TH_YEYAN_ID,
        EventKind::PhaseBegin,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
    (
        "th_taipingyaoshu",
        TH_TAIPINGYAOSHU_ID,
        EventKind::Damage,
        &[EventKind::CardsLost],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    ),
];

/// Authentic `L28` (`雷霆万钧/extension.js:3657-3670`) 14 `th_*` generals.
pub const L28_THUNDER_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 14] = [
    ("th_guanning", Faction::Qun, Gender::Male, 7, &["th_dunshi"]),
    ("th_lukai", Faction::Wu, Gender::Male, 4, &["th_bushi", "th_zhongzhuang"]),
    ("th_caohua", Faction::Wei, Gender::Female, 3, &["th_caiyi", "th_guili"]),
    ("th_sunhanhua", Faction::Wu, Gender::Female, 3, &["th_chongxu", "miaojian", "shhlianhua"]),
    ("th_puyuan", Faction::Shu, Gender::Male, 4, &["th_shengong", "olqisi"]),
    ("th_pangdegong", Faction::Qun, Gender::Male, 3, &["th_pingcai", "xinfu_pdgyingshi"]),
    ("th_zhengxuan", Faction::Qun, Gender::Male, 3, &["th_zhengjing"]),
    ("th_mamidi", Faction::Qun, Gender::Male, 3, &["th_chengye", "th_buxu"]),
    ("th_nanhualaoxian", Faction::Qun, Gender::Male, 3, &["th_yufeng", "th_tianshu"]),
    ("th_majun", Faction::Wei, Gender::Male, 3, &["th_jingxie", "th_qiaosi"]),
    ("th_zhouqun", Faction::Shu, Gender::Male, 3, &["th_tiansuan"]),
    ("th_wanglang", Faction::Wei, Gender::Male, 3, &["th_gushe", "rejici"]),
    ("th_liuhui", Faction::Qun, Gender::Male, 4, &["th_geyuan", "th_jieshu", "th_gusuan"]),
    ("th_shen_zhouyu", Faction::Wu, Gender::Male, 4, &["th_yeyan", "qinyin"]),
];

/// Authentic `L28` (`雷霆万钧/extension.js:3660-3670, 4203-8614`) additional general and sub-skills.
pub const L28_THUNDER_EXTRA_SKILLS: [&str; 39] = [
    "th_bushi",
    "th_zhongzhuang",
    "th_caiyi",
    "th_guili",
    "th_chongxu",
    "miaojian",
    "shhlianhua",
    "th_shengong",
    "olqisi",
    "th_pingcai",
    "th_pcaudio_wolong_card",
    "th_pcaudio_fengchu_card",
    "th_pcaudio_shuijing_card",
    "th_pcaudio_xuanjian_card",
    "xinfu_pdgyingshi",
    "th_zhengjing",
    "th_zhengjing2",
    "th_chengye",
    "th_buxu",
    "th_yufeng",
    "th_tianshu",
    "th_g_taipingyaoshu_ai",
    "th_tiansuan",
    "th_tiansuan2",
    "th_tiansuan2_0",
    "th_tiansuan2_1",
    "th_tiansuan2_damage",
    "th_tiansuan2_2",
    "th_tiansuan2_ai",
    "th_tiansuan2_fire",
    "th_tiansuan2_3",
    "th_tiansuan2_4",
    "th_gushe",
    "th_gushe2",
    "rejici",
    "th_geyuan",
    "th_jieshu",
    "th_gusuan",
    "qinyin",
];

fn build_d34_vm_decl(
    idx: usize,
    name_str: &str,
    def_id: u32,
    event: EventKind,
    extra_events: &[EventKind],
    filter_inner: engine_native::rvm_bridge::VmFilterPolicy,
) -> VmSkillDecl {
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
    VmSkillDecl {
        declaration_hash: None,
        presentation_binding: None,
        name: name_str.to_string(),
        module,
        trigger: engine_native::catalog::TriggerMeta {
            event,
            timing: engine_native::event::Timing::Normal,
            priority: 20 - (idx as i32),
            order_selectable: false,
            optional: false,
            extra_events: extra_events.to_vec(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Declared {
            skill_def_id: def_id,
            hand_limit_base_max_hp: false,
            choice_actor: engine_native::rvm_bridge::VmChoiceActor::Owner,
            modifiers: Vec::new(),
            inner: Box::new(filter_inner),
        },
        r05_identity: None,
    }
}

/// Build the native `ExtensionPack` for Decision 34 authentic upstream rules.
pub fn decision34_closure_extension_pack() -> ExtensionPack {
    let mut vm_skills: Vec<VmSkillDecl> = Vec::with_capacity(
        DECISION34_SKILL_SPECS.len() + L28_THUNDER_EXTRA_SKILLS.len() + L10_SCS_COMPANION_SKILLS.len() + 1,
    );
    for (idx, (name_str, def_id, event, extra_events, filter_inner)) in
        DECISION34_SKILL_SPECS.iter().enumerate()
    {
        vm_skills.push(build_d34_vm_decl(
            idx,
            name_str,
            *def_id,
            *event,
            extra_events,
            filter_inner.clone(),
        ));
    }
    for &sk_name in &L28_THUNDER_EXTRA_SKILLS {
        vm_skills.push(build_d34_vm_decl(
            vm_skills.len(),
            sk_name,
            TH_SKILL_DISPATCH_ID,
            EventKind::PhaseBegin,
            &[],
            engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
        ));
    }
    vm_skills.push(build_d34_vm_decl(
        vm_skills.len(),
        "kuwu",
        HUANLE_KUWU_INTERACTIVE_ID,
        EventKind::CardUsed,
        &[],
        engine_native::rvm_bridge::VmFilterPolicy::OwnerIsSource,
    ));
    for &sk_name in &L10_SCS_COMPANION_SKILLS {
        vm_skills.push(build_d34_vm_decl(
            vm_skills.len(),
            sk_name,
            L10_SCS_SKILL_ID,
            EventKind::PhaseBegin,
            &[],
            engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
        ));
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

    // Authentic `L28` card: `th_taipingyaoshu` (`雷霆万钧/extension.js:8968`)
    let cards = vec![
        CardDef::basic(names::intern("th_taipingyaoshu".to_string()), CardKind::Wuzhong)
            .with_suit_rank(Suit::Heart, 3),
    ];

    ExtensionPack {
        def: ExtensionDef {
            name: "decision34-complete-closure",
            version: 1,
            description: "Decision 34 authentic upstream sub-skills, generals, and cards",
        },
        generals,
        cards,
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

#[path = "zuan_data.rs"]
pub mod zuan_data;
pub use zuan_data::*;

#[path = "taixu_generals_data.rs"]
pub mod taixu_generals_data;
pub use taixu_generals_data::*;

#[path = "taixu_skills_data.rs"]
pub mod taixu_skills_data;
pub use taixu_skills_data::*;

#[path = "taixu_campaign_data.rs"]
pub mod taixu_campaign_data;
pub use taixu_campaign_data::*;

#[path = "shahai_config.rs"]
pub mod shahai_config;
pub use shahai_config::*;

#[path = "shahai_generals_data.rs"]
pub mod shahai_generals_data;
pub use shahai_generals_data::*;

#[path = "shahai_cards_skills_data.rs"]
pub mod shahai_cards_skills_data;
pub use shahai_cards_skills_data::*;

#[path = "host_skill_bindings.rs"]
pub mod host_skill_bindings;
pub use host_skill_bindings::*;

#[path = "decision34_pack9.rs"]
pub mod decision34_pack9;
pub use decision34_pack9::*;

#[path = "shahai_attach.rs"]
pub mod shahai_attach;
pub use shahai_attach::*;

