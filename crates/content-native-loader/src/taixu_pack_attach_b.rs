//! Attach all `L11` (`太虚幻境`) cross-package NPC skills Part B (`[80..164]`) + companion
//! derived skills and off-deck special equips (`taipingyaoshu`, `meiyingqiang`) cleanly via
//! `HostBehavior::L11CrossNpcB`, leaving `xinfu_sidao` on `HostBehavior::L11GeneralBossC`.

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, Suit};
use engine_native::content::extension::VmSkillDecl;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::rvm_bridge::VmFilterPolicy;

pub const L11_NPC_B_SKILLS: &[&str] = &[
    // Part B `[80..164]` (excluding `xinfu_sidao`, preserved under `L11GeneralBossC`)
    "satomi_luodao",
    "sbliegong",
    "sbyingzi",
    "shangyi",
    "shefu",
    "shensu",
    "shizhi",
    "shunshi",
    "spshiji",
    "sptunjiang",
    "spwenji",
    "spxiangzhen",
    "spyishi",
    "tianming",
    "tianshu",
    "tuifeng",
    "twfenming",
    "twhanyu",
    "twhengjiang",
    "twhongju",
    "twhuangjin",
    "twjianming",
    "twjiefan",
    "twjuntun",
    "twliancai",
    "twlingfa",
    "twlinglu",
    "twmibei",
    "twqianxi",
    "twqirang",
    "twqueshi",
    "twwuhun",
    "twwushen",
    "twxiangyu",
    "twxingqi",
    "twxiongxi",
    "twxiongzheng",
    "twxuanhuo",
    "twxuechang",
    "twxuhe",
    "twyilie",
    "twyingjia",
    "twyuanhu",
    "twyuhua",
    "twzhengrong",
    "waishi",
    "wangzun",
    "weifeng",
    "weijing",
    "wulie",
    "xianfu",
    "xiehui",
    "xinfenyue",
    "xinfu_daigong",
    "xinfu_fuyin",
    "xinfu_kannan",
    "xinfu_limu",
    "xinfu_shajue",
    "xinfu_xingluan",
    "xinfu_zengdao",
    "xinfu_zhaoxin",
    "xinjiewei",
    "xinjushou",
    "xinkuangfu",
    "xinsheng",
    "yechou",
    "yinbing",
    "yise",
    "yjjianshu",
    "yongjin",
    "yongsi",
    "yuce",
    "zhanyi",
    "zhendu",
    "zhichi",
    "zhidao",
    "zhiman",
    "zhiyan",
    "zhongyong",
    "zhuide",
    "zhuixi",
    "zili",
    "zongkui",
    // Derived / companion / card skills
    "twqingce",
    "twsaotao",
    "twmouli",
    "twzhian",
    "twxiongjun",
    "paiyi",
    "juedi",
    "xinfu_zengdao2",
    "meiyingqiang",
    "taipingyaoshu",
    "baonvezhi",
];

pub fn attach_l11_npc_b_skills(
    extra_host_skills: &mut Vec<HostSkillDecl>,
    extra_cards: &mut Vec<CardDef>,
    seen_skills: &mut BTreeSet<String>,
    seen_cards: &mut BTreeSet<String>,
) {
    if seen_cards.insert("taipingyaoshu".to_string()) {
        extra_cards.push(
            CardDef::armor("taipingyaoshu")
                .with_suit_rank(Suit::Heart, 3)
                .with_equip_skills(&["taipingyaoshu"])
                .with_off_deck(),
        );
    }
    if seen_cards.insert("meiyingqiang".to_string()) {
        extra_cards.push(
            CardDef::weapon("meiyingqiang", 3)
                .with_suit_rank(Suit::Diamond, 12)
                .with_equip_skills(&["meiyingqiang"])
                .with_off_deck(),
        );
    }
    for &skill in L11_NPC_B_SKILLS {
        crate::push_host_skill_if_unseen(
            skill,
            HostBehavior::L11CrossNpcB,
            extra_host_skills,
            seen_skills,
        );
    }
}

pub fn attach_taixu_native_extras(
    taixu_skill_id: u32,
    extra_host_skills: &mut Vec<HostSkillDecl>,
    extra_vm_skills: &mut Vec<VmSkillDecl>,
    extra_cards: &mut Vec<CardDef>,
    seen_skills: &mut BTreeSet<String>,
    seen_cards: &mut BTreeSet<String>,
) {
    // Authentic passive `feiying` (distance from others +1, 0 `U05` PhaseBegin mutation):
    if seen_skills.insert("feiying".to_string()) {
        let code = vec![0x01u8];
        let module = engine_native::rvm_bridge::BytecodeModule {
            logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
                "feiying#modifier",
                &code,
                0,
                0,
                0,
            ),
            name: "feiying#modifier".to_string(),
            code,
            local_count: 0,
            max_stack: 1,
            entry_pc: 0,
            caps_mask: 0,
        };
        extra_vm_skills.push(VmSkillDecl {
            declaration_hash: None,
            presentation_binding: None,
            name: "feiying".to_string(),
            module,
            trigger: engine_native::catalog::TriggerMeta {
                event: engine_native::event::EventKind::SkillToggled,
                timing: engine_native::event::Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            },
            filter: VmFilterPolicy::Handlers(std::sync::Arc::new(
                engine_native::rvm_bridge::VmHandlerSet {
                    skill_def_id: taixu_skill_id.wrapping_add(10_000),
                    active: None,
                    limits: Default::default(),
                    triggers: Vec::new(),
                    modifiers: vec![engine_native::rvm_bridge::modifiers::VmModifierRule {
                        target: engine_native::rvm_bridge::modifiers::VmModifierTarget::DistanceFromOthers,
                        mode: engine_native::rvm_bridge::modifiers::VmModifierMode::Add,
                        selector: None,
                        value: 1,
                        value_source: None,
                    }],
                    view_as: Vec::new(),
                },
            )),
            r05_identity: None,
        });
    }
    // Authentic Guan Yinping `xueji` (`sp.js:21742-21777`, distinct from L19 zombie stub)
    crate::push_host_skill_if_unseen(
        "xueji",
        HostBehavior::L11GeneralBossC,
        extra_host_skills,
        seen_skills,
    );
    attach_l11_npc_b_skills(extra_host_skills, extra_cards, seen_skills, seen_cards);
    for &ep_sk in &[
        "kuanggu",
        "hongyan",
        "txhj_tanshi",
        "chanyuan",
        "xingshang",
        "hengzheng",
        "xinfu_yisuan",
        "xinfu_lveming",
        "xinfu_tunjun",
        "xinfu_tanbei",
        "xinfu_sidao",
    ] {
        crate::push_host_skill_if_unseen(
            ep_sk,
            HostBehavior::L11GeneralBossC,
            extra_host_skills,
            seen_skills,
        );
    }
}
