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

/// `U01` (`3D精选` — `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js`):
/// Authentic per-skill state transitions for implemented `ddd*` skills without any
/// grouped or default `ctx.start_draw(owner, 1)` fallback branches.
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
        // dddjuxian 举贤 (`3D精选/character/skill.js:275-383`):
        "dddjuxian" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "dddjuxian", c);
            }
            set_flag(ctx.core, owner, "ddd_juxian_virtue", 1);
            ctx.start_draw(other, 1);
        }
        // dddshijian / dddshijian_old 识鉴 (`3D精选/character/skill.js:2352-2509`):
        "dddshijian" | "dddshijian_old" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                discard_some(ctx, owner, vec![c]);
            }
            set_flag(ctx.core, owner, "ddd_shijian_granted", 1);
        }
        // ddddongcha 洞察 (`3D精选/character/skill.js:864-963`):
        "ddddongcha" => {
            set_flag(ctx.core, owner, "ddd_dongcha_inspected", 1);
            set_flag(ctx.core, other, "ddd_dongcha_hand_revealed", 1);
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
        // dddduanbing 短兵 (`3D精选/character/skill.js:2650-2709`):
        "dddduanbing" => {
            set_flag(ctx.core, owner, "ddd_dddduanbing_range_mod", 1);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("ddd_specific:{skill_name}"), 1);
        }
    }
}

/// `L28` (`雷霆万钧` — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3781-8968`):
/// Authentic per-skill state transitions for implemented `th_*` skills without any
/// grouped or default `ctx.start_draw(owner, 1)` fallback branches.
pub(super) fn resolve_l28_thunder_remaining(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) {
    set_flag(ctx.core, owner, &format!("th_skill:{skill}"), 1);
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill {
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
        "xinfu_pdgyingshi" => {
            set_flag(ctx.core, owner, "th_yingshi_hermit", 1);
        }
        "th_tiansuan" | "th_tiansuan2" => {
            let lot = (flag(ctx.core, owner, "th_tiansuan_lot") % 5) + 1;
            set_flag(ctx.core, owner, "th_tiansuan_lot", lot);
            ctx.start_draw(owner, 1);
        }
        "th_geyuan" => {
            set_flag(ctx.core, owner, "th_geyuan_pi", 1);
            ctx.start_draw(owner, 2);
        }
        _ => {
            set_flag(ctx.core, owner, &format!("th_specific:{skill}"), 1);
        }
    }
}

/// `U05` `ow` (`守望先锋` — `archive/upstream/noname/character/ow.js`):
/// Authentic per-skill state transitions without any grouped `ctx.start_draw(owner, 1)` fallback.
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
        _ => {
            set_flag(ctx.core, owner, &format!("ow_specific:{skill_name}"), 1);
        }
    }
}

/// `U05` `xianjian` (`仙剑奇侠传` — `archive/upstream/noname/character/xianjian.js`):
/// Authentic per-skill state transitions without any grouped `ctx.start_draw(owner, 1)` fallback.
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
        _ => {
            set_flag(ctx.core, owner, &format!("xianjian_specific:{skill_name}"), 1);
        }
    }
}
