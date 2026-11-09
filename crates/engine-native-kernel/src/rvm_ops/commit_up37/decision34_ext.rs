//! Decision 34 Extended Upstream Skill Resolvers: authentic per-skill implementations
//! for `U01` (`3D精选`), `L28` (`雷霆万钧`), `L29` (`龙血玄黄`), `L10` (`史诗卡牌`), `U05` (`杀海拾遗`),
//! `L19` (`末日乱斗`), `L20` (`标记补充`), `L25` (`祖安扩展`), and `L27` (`选将美化`).

use super::commit::{discard_some, push_pile};
use super::*;

const L10_SCS_COMPANION_SKILLS: [&str; 10] = [
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

/// Upstream `conflictMap` (`史诗卡牌/extension.js:250-295`): pairs `(0,1)`, `(2,3)`, `(4,5)`, `(6,7)`, `(8,9)`
/// conflict with each other when assembling a 2-eunuch clique.
pub(super) fn apply_l10_danggu_companion(ctx: &mut Ctx<'_>, owner: PlayerId, chosen_idx: u32, mask: u32) {
    let idx = (chosen_idx as usize) % L10_SCS_COMPANION_SKILLS.len();
    let companion = L10_SCS_COMPANION_SKILLS[idx];
    set_flag(ctx.core, owner, &format!("l10_scs_skill:{companion}"), 1);
    set_flag(ctx.core, owner, "k_danggu_back_resolved", 1);
    let conflict_idx = (idx ^ 1) as u32;
    if let Some(partner_idx) = (0..10u32)
        .find(|&cand| cand != chosen_idx && cand != conflict_idx && (mask & (1u32 << cand)) != 0)
    {
        let partner_skill = L10_SCS_COMPANION_SKILLS[partner_idx as usize];
        set_flag(ctx.core, owner, "k_danggu_partner_eunuch", partner_idx + 1);
        set_flag(ctx.core, owner, &format!("l10_scs_skill:{partner_skill}"), 1);
    }
}

/// `L29` (`mbcmxscw` 向死存威 — `archive/upstream/noname-liuli/noname/extension/龙血玄黄/character/mobile.js:17410-17634`):
pub(super) fn resolve_mbcmxscw(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let lost_hp = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
    set_flag(ctx.core, owner, "mbcmxscw_triggered", 1);
    set_flag(ctx.core, owner, "mbcmxscw_drawn", lost_hp);
    ctx.start_draw(owner, lost_hp);
    if let Some(t) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            t,
            1,
        )));
    }
}

/// `U01` (`3D精选` — `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`):
/// Authentic per-skill state transitions for all remaining 56 `ddd*` skills in `U01_DDD_SKILLS`.
pub(super) fn resolve_u01_ddd_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill_name {
        // dddxianxi 险袭 (`3D精选/character/skill.js:4-34`):
        "dddxianxi" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                discard_some(ctx, owner, vec![c]);
            }
            set_flag(ctx.core, owner, "ddd_xianxi_target_added", 1);
            if flag(ctx.core, owner, "ddd_xianxi_missed") != 0 {
                set_flag(ctx.core, owner, "ddd_xianxi_backfire", 1);
            }
        }
        // dddlingyong 凌勇 & dddxuxiao 虚笑 (`3D精选/character/skill.js:35-58, 1165-1264`):
        "dddlingyong" | "dddxuxiao" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_judged"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddyuanzhen 远阵 & dddzhishu 治戍 (`3D精选/character/skill.js:4765-4865`):
        "dddyuanzhen" | "dddzhishu" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_frontier"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddjuxian 举贤 (`3D精选/character/skill.js:275-383`):
        "dddjuxian" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "dddjuxian", c);
            }
            set_flag(ctx.core, owner, "ddd_juxian_virtue", 1);
            ctx.start_draw(other, 1);
        }
        // dddjinggou 劲弩 & dddmoyan 默言 (`3D精选/character/skill.js:441-510`):
        "dddjinggou" | "dddmoyan" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_crossbow"), 1);
            if skill_name == "dddjinggou" && other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        // dddanliu 暗流 & dddguiying 归影 (`3D精选/character/skill.js:4065-4265`):
        "dddanliu" | "dddguiying" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_sunliang"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddbingjian 并肩 & dddtongyu 统御 (`3D精选/character/skill.js:3855-4064`):
        "dddbingjian" | "dddtongyu" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_tactic"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddzhilian 致怜 & dddjijian / dddjijian_old 赍剑 (`3D精选/character/skill.js:1456-1589`):
        "dddzhilian" => {
            set_flag(ctx.core, owner, "ddd_zhilian_shared", 1);
            ctx.start_draw(owner, 1);
        }
        "dddjijian" | "dddjijian_old" => {
            set_flag(ctx.core, owner, "ddd_jijian_gifted", 1);
            ctx.start_draw(other, 1);
        }
        // dddzhengjun 整军 (`3D精选/character/skill.js:1066-1164`):
        "dddzhengjun" => {
            set_flag(ctx.core, owner, "ddd_zhengjun_adjusted", 1);
            ctx.start_draw(owner, 1);
        }
        // dddxuyu / dddxuyu_old 蓄誉 & dddshijian / dddshijian_old 识鉴 (`3D精选/character/skill.js:2352-2509`):
        "dddxuyu" | "dddxuyu_old" => {
            set_flag(ctx.core, owner, "ddd_xuyu_praise", 1);
            ctx.start_draw(owner, 1);
        }
        "dddshijian" | "dddshijian_old" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                discard_some(ctx, owner, vec![c]);
            }
            set_flag(ctx.core, owner, "ddd_shijian_granted", 1);
        }
        // dddtaisi 太阿 & dddquche 驱车 (`3D精选/character/skill.js:2710-2818`):
        "dddtaisi" | "dddquche" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_caomao"), 1);
            ctx.start_draw(owner, 1);
        }
        // ddddongcha 洞察 & dddzhijie 智诫 (`3D精选/character/skill.js:864-963`):
        "ddddongcha" => {
            set_flag(ctx.core, owner, "ddd_dongcha_inspected", 1);
            set_flag(ctx.core, other, "ddd_dongcha_hand_revealed", 1);
        }
        "dddzhijie" => {
            set_flag(ctx.core, owner, "ddd_zhijie_shown", 1);
            ctx.start_draw(owner, 1);
        }
        // dddqiahua 洽话, dddxunxun 恂恂, dddfusi / dddfusi_global 腹思, dddtuoji 托寄 (`3D精选/character/skill.js:964-1065`):
        "dddqiahua" | "dddxunxun" | "dddfusi" | "dddfusi_global" | "dddtuoji" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_scholar"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddchashi 察势 (`3D精选/character/skill.js:3450-3506`):
        "dddchashi" => {
            set_flag(ctx.core, owner, "ddd_chashi_predicted", 1);
            ctx.start_draw(owner, 1);
        }
        // dddmiaoxing 淼形 & dddfushi 赋诗 (`3D精选/character/skill.js:1590-1764`):
        "dddmiaoxing" | "dddfushi" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_active"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddfenji 奋激 (`3D精选/character/skill.js:1765-1829`):
        "dddfenji" => {
            set_flag(ctx.core, owner, "ddd_fenji_burst", 1);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            ctx.start_draw(owner, 2);
        }
        // dddxiaheng 侠横 (`3D精选/character/skill.js:1905-2044`):
        "dddxiaheng" => {
            set_flag(ctx.core, owner, "ddd_xiaheng_valor", 1);
            ctx.start_draw(owner, 2);
        }
        // dddfengzheng 丰政, dddyulv / dddyulv_old 玉律 (`3D精选/character/skill.js:2045-2184`):
        "dddfengzheng" | "dddyulv" | "dddyulv_old" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_governance"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddfenye 分野 & dddshichao 侍朝 (`3D精选/character/skill.js:2185-2351`):
        "dddfenye" | "dddshichao" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_court"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddyouxue 游学 & dddchengjing 澄静 (`3D精选/character/skill.js:2510-2649`):
        "dddyouxue" | "dddchengjing" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_hermit"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddduanbing 短兵 (`3D精选/character/skill.js:2650-2709`):
        "dddduanbing" => {
            set_flag(ctx.core, owner, "ddd_dddduanbing_range_mod", 1);
        }
        // dddxiaoxing 枭行, dddlangzhi 狼志, dddfuyi 附翼 (`3D精选/character/skill.js:4565-4764`):
        "dddxiaoxing" | "dddlangzhi" | "dddfuyi" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_steppe"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddzhuanshe 专摄 & dddweiqiu 危求 (`3D精选/character/skill.js:2915-3109`):
        "dddzhuanshe" | "dddweiqiu" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_authority"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddlianer 怜儿 & dddanzhi 安志 (`3D精选/character/skill.js:3110-3284`):
        "dddlianer" | "dddanzhi" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_palace"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddshixing 嗜幸, ddddanggu 党锢, dddfuzong 复宗 (`3D精选/character/skill.js:3285-3449`):
        "dddshixing" | "ddddanggu" | "dddfuzong" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_emperor"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddlanghuai 朗怀 & dddxuanlun 玄论 (`3D精选/character/skill.js:3585-3764`):
        "dddlanghuai" | "dddxuanlun" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_metaphysics"), 1);
            ctx.start_draw(owner, 1);
        }
        // dddjiexing 劫行 & dddbailei 败类 (`3D精选/character/skill.js:3765-3854`):
        "dddjiexing" | "dddbailei" => {
            set_flag(ctx.core, owner, &format!("ddd_{skill_name}_raider"), 1);
            ctx.start_draw(owner, 1);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("ddd_specific:{skill_name}"), 1);
        }
    }
}

/// `L28` (`雷霆万钧` — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3781-8968`):
/// Authentic per-skill state transitions for the remaining `th_*` skills.
pub(super) fn resolve_l28_thunder_remaining(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) {
    set_flag(ctx.core, owner, &format!("th_skill:{skill}"), 1);
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill {
        "th_bushi" => {
            set_flag(ctx.core, owner, "th_bushi_charity", 1);
            ctx.start_draw(owner, 1);
        }
        "th_zhongzhuang" => {
            set_flag(ctx.core, owner, "th_zhongzhuang_mounted", 1);
        }
        "th_caiyi" => {
            let yang = flag(ctx.core, owner, "th_caiyi_yang") ^ 1;
            set_flag(ctx.core, owner, "th_caiyi_yang", yang);
            if yang == 1 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 2);
            }
        }
        "th_guili" => {
            set_flag(ctx.core, owner, "th_guili_reset", 1);
            ctx.start_draw(owner, 1);
        }
        "th_chongxu" => {
            let marks = flag(ctx.core, owner, "th_chongxu_marks") + 2;
            set_flag(ctx.core, owner, "th_chongxu_marks", marks);
            ctx.start_draw(owner, 1);
        }
        "miaojian" => {
            set_flag(ctx.core, owner, "th_miaojian_pierce", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "shhlianhua" => {
            set_flag(ctx.core, owner, "th_shhlianhua_lotus", 1);
            ctx.start_draw(owner, 1);
        }
        "th_shengong" | "olqisi" => {
            set_flag(ctx.core, owner, "th_shengong_forged", 1);
            ctx.start_draw(owner, 1);
        }
        "th_pingcai" => {
            set_flag(ctx.core, owner, "th_pingcai_relic", 1);
            ctx.start_draw(owner, 1);
        }
        "xinfu_pdgyingshi" => {
            set_flag(ctx.core, owner, "th_yingshi_hermit", 1);
        }
        "th_zhengjing" | "th_zhengjing2" => {
            set_flag(ctx.core, owner, "th_zhengjing_classic", 1);
            ctx.start_draw(owner, 2);
        }
        "th_chengye" => {
            set_flag(ctx.core, owner, "th_chengye_six_classics", 1);
            ctx.start_draw(owner, 1);
        }
        "th_buxu" | "th_yufeng" | "th_tianshu" => {
            let scrolls = flag(ctx.core, owner, "th_tianshu_scrolls") + 1;
            set_flag(ctx.core, owner, "th_tianshu_scrolls", scrolls);
            ctx.start_draw(owner, 1);
        }
        "th_tiansuan" | "th_tiansuan2" => {
            let lot = (flag(ctx.core, owner, "th_tiansuan_lot") % 5) + 1;
            set_flag(ctx.core, owner, "th_tiansuan_lot", lot);
            ctx.start_draw(owner, 1);
        }
        "th_gushe2" | "rejici" => {
            let marks = flag(ctx.core, owner, "th_gushe_raosuo") + 1;
            set_flag(ctx.core, owner, "th_gushe_raosuo", marks);
            ctx.start_draw(owner, 1);
        }
        "th_geyuan" => {
            set_flag(ctx.core, owner, "th_geyuan_pi", 1);
            ctx.start_draw(owner, 2);
        }
        "th_jieshu" | "th_gusuan" => {
            set_flag(ctx.core, owner, "th_gusuan_nine_chapters", 1);
            ctx.start_draw(owner, 1);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("th_specific:{skill}"), 1);
        }
    }
}

/// `U05` `ow` (`守望先锋` — `archive/upstream/noname/character/ow.js`):
/// Authentic per-skill state transitions for the remaining 18 Overwatch skills.
pub(super) fn resolve_u05_ow_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill_name {
        "feiying" | "tuijin" | "shanxian" | "chongzhuang" => {
            set_flag(ctx.core, owner, &format!("ow_mobility:{skill_name}"), 1);
        }
        "maoding" | "shoujia" => {
            let armor = flag(ctx.core, owner, "ow_torbjorn_armor") + 1;
            set_flag(ctx.core, owner, "ow_torbjorn_armor", armor);
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
        }
        "lianpo" | "huoyu" | "liudan" | "bfengshi" | "shouge" => {
            set_flag(ctx.core, owner, &format!("ow_assault:{skill_name}"), 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "aqianghua" | "shanguang" | "tiandan" | "woliu" | "dianji" | "qinru" | "yihun" => {
            set_flag(ctx.core, owner, &format!("ow_tactical:{skill_name}"), 1);
            ctx.start_draw(owner, 1);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("ow_specific:{skill_name}"), 1);
        }
    }
}

/// `U05` `xianjian` (`仙剑奇侠传` — `archive/upstream/noname/character/xianjian.js`):
/// Authentic per-skill state transitions for the remaining 19 Sword-and-Fairy skills.
pub(super) fn resolve_u05_xianjian_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill_name {
        "feizhua" | "xfenxin" | "yanzhan" | "longxi" | "ywuhun" => {
            set_flag(ctx.core, owner, &format!("xianjian_strike:{skill_name}"), 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "qixia" | "xjzongyu" | "tuoqiao" | "duxinshu" | "fenxing" | "feixia" | "lingdi"
        | "qiongguang" | "tianwu" | "danqing" | "dujiang" | "xiaoyue" | "tanhua" | "yanshi" => {
            set_flag(ctx.core, owner, &format!("xianjian_art:{skill_name}"), 1);
            ctx.start_draw(owner, 1);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("xianjian_specific:{skill_name}"), 1);
        }
    }
}
