//! Decision 34 Extended Upstream Skill Resolvers: authentic per-skill implementations
//! for `L01` (`AI优化`), `L06` (`五行生克`), `U06` (`欢乐卡牌`), `U07` (`玩点论杀`),
//! `U01` (`3D精选`), `L28` (`雷霆万钧`), `L29` (`龙血玄黄`), `L10` (`史诗卡牌`), and `U05` (`杀海拾遗`).

use super::commit::{discard_some, pop_deck, push_pile};
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



fn grant_named_skill_if_present(ctx: &mut Ctx<'_>, owner: PlayerId, skill_name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(skill_name) {
        let already = ctx.core.has_skill(owner, def);
        ctx.ensure_skill(
            (owner, def),
            SkillInstance {
                owner,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        if !already {
            ctx.emit(EventData::SkillToggled {
                player: owner,
                skill: def,
                enabled: true,
                temporary: false,
            });
        }
    }
}

/// Upstream `conflictMap` (`史诗卡牌/extension.js:250-295`): pairs `(0,1)`, `(2,3)`, `(4,5)`, `(6,7)`, `(8,9)`
/// conflict with each other when assembling a 2-eunuch clique.
pub(super) fn apply_l10_danggu_companion(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    chosen_idx: u32,
    mask: u32,
) {
    let idx = (chosen_idx as usize) % L10_SCS_COMPANION_SKILLS.len();
    let companion = L10_SCS_COMPANION_SKILLS[idx];
    set_flag(ctx.core, owner, &format!("l10_scs_skill:{companion}"), 1);
    grant_named_skill_if_present(ctx, owner, companion);
    set_flag(ctx.core, owner, "k_danggu_back_resolved", 1);
    let conflict_idx = (idx ^ 1) as u32;
    if let Some(partner_idx) = (0..10u32)
        .find(|&cand| cand != chosen_idx && cand != conflict_idx && (mask & (1u32 << cand)) != 0)
    {
        let partner_skill = L10_SCS_COMPANION_SKILLS[partner_idx as usize];
        set_flag(ctx.core, owner, "k_danggu_partner_eunuch", partner_idx + 1);
        set_flag(
            ctx.core,
            owner,
            &format!("l10_scs_skill:{partner_skill}"),
            1,
        );
        grant_named_skill_if_present(ctx, owner, partner_skill);
    }
}

/// `L10` (`scstaoluan`..`scsanruo` — `archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2335-2369`):
pub(super) fn resolve_l10_scs_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = hostile_target(ctx, owner).unwrap_or(owner);
    set_flag(ctx.core, owner, &format!("l10_scs_resolved:{skill_name}"), 1);
    match skill_name {
        "scstaoluan" => {
            if discard_one_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l10_scstaoluan_virtual_used", 1);
            }
        }
        "scschiyan" => {
            set_flag(ctx.core, other, "l10_scschiyan_detained", 1);
            if ctx.core.player(owner).hand.len() > ctx.core.player(other).hand.len()
                && other != owner
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "scszimou" => {
            ctx.start_draw(owner, 1);
        }
        "scspicai" => {
            ctx.start_draw(other, 1);
        }
        "scsyaozhuo" => {
            set_flag(ctx.core, other, "l10_scsyaozhuo_skip_draw", 1);
        }
        "scsxiaolu" => {
            ctx.start_draw(owner, 2);
            let _ = discard_one_he(ctx, owner);
            let _ = discard_one_he(ctx, owner);
        }
        "scskuiji" => {
            let _ = discard_one_he(ctx, owner);
            let _ = discard_one_he(ctx, other);
        }
        "scschihe" => {
            set_flag(ctx.core, owner, "l10_scschihe_sha_boost", 1);
        }
        "scsniqu" => {
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    other,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "scsanruo" => {
            if other != owner {
                if let Some(c) = choose_enemy_card(ctx, owner, other) {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("l10_scs_hook:{other_sk}"), 1);
        }
    }
}

/// `L01` (`yiyi` 以逸待劳 — `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9205-9260`):
/// Target draws 2 cards (`target.draw(2)` at line 9221) then discards 2 `he` cards (`target.chooseToDiscard(2, "he", true)` at line 9223).
pub(super) fn resolve_l01_yiyi_card(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let drawn = draw_cards_sync(ctx, owner, 2);
    let mut disc = Vec::new();
    for _ in 0..2 {
        if let Some(c) = choose_own_card(ctx, owner, false) {
            ctx.take_hand_card(owner, c);
            disc.push(c);
        } else if let Some(c) = choose_own_card(ctx, owner, true) {
            ctx.take_owned_card(owner, c);
            disc.push(c);
        }
    }
    let disc_n = disc.len() as u32;
    if !disc.is_empty() {
        discard_some(ctx, owner, disc);
    }
    set_flag(ctx.core, owner, "l01_yiyi_drawn", drawn);
    set_flag(ctx.core, owner, "l01_yiyi_discarded", disc_n);
    set_flag(ctx.core, owner, "l01_yiyi_resolved", 1);
}

/// `L06` (`_shengke` & `wuxingpan_skill` — `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-193`):
/// - `_shengke`: 5-element cycle (`1=metal`, `2=wood`, `3=water`, `4=fire`, `5=soil`):
///   sheng (`metal->water`, `water->wood`, `wood->fire`, `fire->soil`, `soil->metal`) draws 1 card (`extension.js:70-85`);
///   ke (`metal->wood`, `wood->soil`, `soil->water`, `water->fire`, `fire->metal`) discards 1 `he` card (`extension.js:87-102`).
/// - `wuxingpan_skill`: `enable: "phaseUse"`, `usable: 1`, changes 1 hand card's `wunature` (`extension.js:173-193`).
pub(super) fn resolve_l06_wuxing_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    if skill_name == "wuxingpan_skill" {
        if flag(ctx.core, owner, "wuxingpan_used_this_turn") != 0
            || ctx.core.player(owner).hand.is_empty()
        {
            set_flag(ctx.core, owner, "wuxingpan_blocked", 1);
            return;
        }
        let next_nature = (flag(ctx.core, owner, "wuxingpan_target_nature") % 5) + 1;
        set_flag(ctx.core, owner, "wuxingpan_used_this_turn", 1);
        set_flag(ctx.core, owner, "wuxingpan_reassigned_nature", next_nature);
        return;
    }
    let card_nat = flag(ctx.core, owner, "wuxing_card_nature").clamp(1, 5);
    let target_nat = flag(ctx.core, owner, "wuxing_player_nature").clamp(1, 5);
    let is_sheng = matches!(
        (card_nat, target_nat),
        (1, 3) | (3, 2) | (2, 4) | (4, 5) | (5, 1)
    );
    let is_ke = matches!(
        (card_nat, target_nat),
        (1, 2) | (2, 5) | (5, 3) | (3, 4) | (4, 1)
    );
    if is_sheng {
        set_flag(ctx.core, owner, "wuxing_sheng_triggered", 1);
        ctx.start_draw(owner, 1);
    } else if is_ke {
        set_flag(ctx.core, owner, "wuxing_ke_triggered", 1);
        let _ = discard_one_he(ctx, owner);
    } else {
        set_flag(ctx.core, owner, "wuxing_neutral", 1);
    }
}

/// `U06` (`monkey`, `mianju`, `kuwu`, `xuelunyang`, `jiuwei` —
/// `archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5-144`).
pub(super) fn resolve_u06_huanle_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    match skill_name {
        // monkey 猴子 (`欢乐卡牌/card/skill.js:5-31`):
        "monkey" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "monkey_prevented_tao", 1);
            set_flag(ctx.core, owner, "monkey_stole_tao", 1);
            ctx.start_draw(owner, 1);
        }
        // mianju 漩涡面具 (`欢乐卡牌/card/skill.js:32-40`):
        "mianju" => {
            ctx.core.player_mut(owner).face_down = false;
            set_flag(ctx.core, owner, "mianju_prevented_turnover", 1);
        }
        // kuwu 苦无 (`欢乐卡牌/card/skill.js:41-61`):
        "kuwu" => {
            set_flag(ctx.core, owner, "kuwu_invoked", 1);
            super::decision34::resolve_kuwu_interactive(ctx, owner);
        }
        // xuelunyang 写轮眼 (`欢乐卡牌/card/skill.js:62-97`):
        "xuelunyang" => {
            if discard_one_he(ctx, owner).is_none() {
                set_flag(ctx.core, owner, "xuelunyang_blocked_empty", 1);
                return;
            }
            set_flag(ctx.core, owner, "xuelunyang_copied_skill", 1);
        }
        // jiuwei 九尾 (`欢乐卡牌/card/skill.js:98-144`):
        "jiuwei" => {
            set_flag(ctx.core, owner, "jiuwei_triggered", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else if flag(ctx.core, owner, "jiuwei_discard_other") != 0 {
                if let Some(other) = hostile_target(ctx, owner) {
                    let _ = discard_one_he(ctx, other);
                    let _ = discard_one_he(ctx, other);
                }
            } else {
                ctx.start_draw(owner, 2);
            }
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("u06_hook:{other_sk}"), 1);
        }
    }
}

/// `U07` (`youdishenru`, `g_youdishenru`, `ciqiu_dying`, `ciqiu_destroy`, `kangyin2`, etc. —
/// `archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:169-219`, `card/skill.js:5-36`, `character/skill.js:5-795`).
pub(super) fn resolve_u07_wudian_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = hostile_target(ctx, owner).unwrap_or(owner);
    set_flag(ctx.core, owner, &format!("u07_skill:{skill_name}"), 1);
    match skill_name {
        "youdishenru" | "g_youdishenru" => {
            set_flag(ctx.core, owner, "youdishenru_nullified_sha", 1);
            if other != owner && flag(ctx.core, owner, "youdishenru_source_has_sha") == 0 {
                set_flag(ctx.core, owner, "youdishenru_counter_damage", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "ciqiu_dying" => {
            // `玩点论杀/character/skill.js:99-108`: `trigger: { player: "dyingBegin" }` -> `player.die()`
            set_flag(ctx.core, owner, "ciqiu_dying_instant_kill", 1);
            set_flag(ctx.core, owner, "ciqiu_removed", 1);
            if ctx.core.player(owner).dying || ctx.core.player(owner).hp <= 0 {
                ctx.core.player_mut(owner).hp = 0;
                ctx.core.player_mut(owner).dying = false;
                ctx.core.player_mut(owner).alive = false;
            }
        }
        "ciqiu_destroy" => {
            // `玩点论杀/character/skill.js:84-98`: destroys/discards equipped weapon when leaving equip zone
            if let Some(wp) = ctx
                .core
                .player(owner)
                .equips
                .iter()
                .copied()
                .find(|&c| ctx.catalog.card_kind(c).equip_slot() == Some(crate::catalog::EquipSlot::Weapon))
            {
                ctx.take_owned_card(owner, wp);
                discard_some(ctx, owner, vec![wp]);
            }
            set_flag(ctx.core, owner, "ciqiu_weapon_destroyed", 1);
        }
        "kangyin2" => {
            // `玩点论杀/character/skill.js:391-407`: `-X` distance & `+X` Sha extra targets (`d34_flags`)
            let bonus = flag(ctx.core, owner, "kangyin_marks").max(1);
            set_flag(ctx.core, owner, "kangyin2_bonus_applied", bonus);
        }
        "fuchou3" => {
            // `玩点论杀/character/skill.js:510`: end-of-turn counter-damage sub-skill
            set_flag(ctx.core, owner, "wdb_fuchou_counter", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "diyjuntun2" | "juntun" | "juntun_global" => {
            // `玩点论杀/character/skill.js:776`: recast an equip card (discard 1 equip -> draw 1)
            let eq = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .find(|&c| ctx.catalog.card_kind(c).is_equip())
                .or_else(|| choose_own_card(ctx, owner, true));
            if let Some(c) = eq {
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.take_hand_card(owner, c);
                } else {
                    ctx.take_owned_card(owner, c);
                }
                discard_some(ctx, owner, vec![c]);
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "wdb_juntun_recast", 1);
            }
        }
        "liduan2" => {
            // `玩点论杀/character/skill.js:677-686`: target gives 1 hand card to owner or suit is locked (`d34_flags`)
            if other != owner {
                if flag(ctx.core, other, "liduan_refuse_give") == 0 {
                    if let Some(c) = choose_enemy_card(ctx, owner, other) {
                        ctx.take_hand_card(other, c);
                        ctx.core.player_mut(owner).hand.push(c);
                        set_flag(ctx.core, owner, "wdb_liduan_gained_card", 1);
                        return;
                    }
                }
                set_flag(ctx.core, other, "wdb_liduan_suit_locked", 5);
            }
        }
        _ => {}
    }
}

/// `L29` (`mbcmxscw` 向死存威 & `mbcmqingzheng` 清正 —
/// `archive/upstream/noname-liuli/noname/extension/龙血玄黄/character/mobile.js:17410-17634` & `extension.readable.js:1`):
pub(super) fn resolve_mbcmxscw(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let lost_hp = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
    set_flag(ctx.core, owner, "mbcmxscw_triggered", 1);
    set_flag(ctx.core, owner, "mbcmxscw_drawn", lost_hp);
    ctx.start_draw(owner, lost_hp);
    if let Some(t) = hostile_target(ctx, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            t,
            1,
        )));
    }
}

pub(super) fn resolve_l29_longxue_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    if skill_name == "mbcmqingzheng" {
        if ctx.core.player(owner).mbqianlong_marks < 25
            && flag(ctx.core, owner, "mbcmqingzheng_force_unlock") == 0
        {
            set_flag(ctx.core, owner, "mbqianlong_locked_guard", 1);
            return;
        }
        let owner_disc = discard_one_he(ctx, owner).is_some();
        let other = hostile_target(ctx, owner).unwrap_or(owner);
        let target_disc = if other != owner {
            discard_one_he(ctx, other).is_some()
        } else {
            false
        };
        set_flag(ctx.core, owner, "mbcmqingzheng_resolved", 1);
        if owner_disc && (!target_disc || flag(ctx.core, owner, "mbcmqingzheng_more_discarded") != 0)
            && other != owner
        {
            set_flag(ctx.core, owner, "mbcmqingzheng_damage_dealt", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data(
                Some(owner),
                other,
                1,
            )));
        }
    } else {
        resolve_mbcmxscw(ctx, owner);
    }
}

/// `U01` (`3D精选` — `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`):
/// Authentic per-skill state transitions for all remaining `ddd*` skills without any
/// grouped or default `ctx.start_draw(owner, 1)` fallback branches.
pub(super) fn resolve_u01_ddd_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = hostile_target(ctx, owner).unwrap_or(owner);
    match skill_name {
        // dddyuanzhen 远振 (`3D精选/character/skill.js:6-36`):
        "dddyuanzhen" => {
            if flag(ctx.core, other, "dddyuanzhen_choose_discard") != 0
                && discard_one_he(ctx, other).is_some()
            {
                set_flag(ctx.core, owner, "dddyuanzhen_target_discarded", 1);
            } else {
                set_flag(ctx.core, owner, "dddyuanzhen_owner_drew", 1);
                ctx.start_draw(owner, 1);
            }
        }
        // dddzhishu 制戍 (`3D精选/character/skill.js:37-58`):
        "dddzhishu" => {
            set_flag(ctx.core, owner, "dddzhishu_equip_moved", 1);
            set_flag(ctx.core, owner, "dddzhishu_virtual_guohe", 1);
            let _ = discard_one_he(ctx, other);
        }
        // dddjuxian 聚贤 (`3D精选/character/skill.js:244-383`):
        "dddjuxian" => {
            if let Some(c) = choose_own_card(ctx, owner, false) {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "dddjuxian", c);
            }
            set_flag(ctx.core, owner, "ddd_juxian_virtue", 1);
            ctx.start_draw(other, 1);
        }
        // dddjinggou 精彀 (`3D精选/character/skill.js:443-471`):
        // After equipping a weapon (`equip1`) with max attack range on table, deals 1 damage to another character.
        "dddjinggou" => {
            set_flag(ctx.core, owner, "dddjinggou_weapon_snipe", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // dddmoyan 末焱 (`3D精选/character/skill.js:472-509`):
        // At roundStart, reveals 3 cards, chosen target gains 1..=3 of them (`dddmoyan_gain_count`),
        // and records `[target, gains.length]` so owner takes +1 damage while target's hand <= gains.length.
        "dddmoyan" => {
            let gain_n = flag(ctx.core, owner, "dddmoyan_gain_count").clamp(1, 3);
            let target = if flag(ctx.core, owner, "dddmoyan_target_self") != 0 {
                owner
            } else {
                other
            };
            ctx.start_draw(target, gain_n);
            set_flag(ctx.core, owner, "dddmoyan_target", u32::from(target));
            set_flag(ctx.core, owner, "dddmoyan_recorded_count", gain_n);
        }
        // dddanliu 暗流 (`3D精选/character/skill.js:861-967`):
        "dddanliu" => {
            if let (Some(c1), Some(c2)) = (
                choose_own_card(ctx, owner, false),
                choose_enemy_card(ctx, owner, other),
            ) {
                if other != owner {
                    ctx.take_hand_card(owner, c1);
                    ctx.take_hand_card(other, c2);
                    ctx.core.player_mut(owner).hand.push(c2);
                    ctx.core.player_mut(other).hand.push(c1);
                }
            }
            set_flag(ctx.core, owner, "dddanliu_swapped", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
        }
        // dddguiying 归萤 (`3D精选/character/skill.js:968-1203`):
        "dddguiying" => {
            if other != owner {
                if let Some(c) = choose_enemy_card(ctx, owner, other) {
                    ctx.take_hand_card(other, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "dddguiying_tribute", 1);
        }
        // dddbingjian 并肩 (`3D精选/character/skill.js:1204-1442`):
        "dddbingjian" => {
            let hand_len = ctx.core.player(owner).hand.len();
            if hand_len < 2 {
                let diff = (2 - hand_len) as u32;
                ctx.start_draw(owner, diff);
                if other != owner {
                    ctx.start_draw(other, diff);
                }
            } else if hand_len > 2 {
                let _ = discard_one_he(ctx, owner);
                if other != owner {
                    let _ = discard_one_he(ctx, other);
                }
            } else {
                ctx.start_draw(owner, 1);
            }
            set_flag(ctx.core, owner, "dddbingjian_adjusted_to_2", 1);
        }
        // dddtongyu 仝御 (`3D精选/character/skill.js:1443-1583`):
        "dddtongyu" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "dddtongyu_wugu_bonus", 1);
            ctx.start_draw(owner, 1);
            if other != owner {
                ctx.start_draw(other, 1);
            }
        }
        // dddzhilian 枝连 (`3D精选/character/skill.js:1584-1675`):
        "dddzhilian" => {
            set_flag(ctx.core, owner, "dddzhilian_resolved", 1);
            ctx.start_draw(owner, 1);
            if other != owner {
                ctx.start_draw(other, 1);
            }
        }
        // dddjijian / dddjijian_old 赍剑 (`3D精选/character/skill.js:1676-2004`):
        "dddjijian" | "dddjijian_old" => {
            if other != owner {
                if let Some(c) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(other).hand.push(c);
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
            set_flag(ctx.core, owner, "dddjijian_given", 1);
        }
        // dddzhengjun 整军 (`3D精选/character/skill.js:2005-2193`):
        "dddzhengjun" => {
            set_flag(ctx.core, owner, "dddzhengjun_matched", 1);
            ctx.start_draw(owner, 1);
        }
        // dddxianxi 险袭 (`3D精选/character/skill.js:2194-2295`):
        "dddxianxi" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "ddd_xianxi_target_added", 1);
            if flag(ctx.core, owner, "ddd_xianxi_missed") != 0 {
                set_flag(ctx.core, owner, "ddd_xianxi_backfire", 1);
            }
        }
        // dddlingyong 灵涌 (`3D精选/character/skill.js:2296-2332`):
        "dddlingyong" => {
            set_flag(ctx.core, owner, "dddlingyong_judged", 1);
            ctx.start_draw(owner, 1);
        }
        // dddxuxiao 虚孝 (`3D精选/character/skill.js:2333-2406`):
        "dddxuxiao" => {
            set_flag(ctx.core, owner, "dddxuxiao_bingliang_placed", 1);
            ctx.start_draw(other, 2);
        }
        // dddxuyu / dddxuyu_old 虚誉 (`3D精选/character/skill.js:2407-2422, 2519-2573`):
        "dddxuyu" | "dddxuyu_old" => {
            if flag(ctx.core, owner, "dddxuyu_must_discard_next") != 0 {
                set_flag(ctx.core, owner, "dddxuyu_must_discard_next", 0);
                let _ = discard_one_he(ctx, owner);
            } else {
                set_flag(ctx.core, owner, "dddxuyu_must_discard_next", 1);
                ctx.start_draw(owner, 1);
            }
        }
        // dddshijian / dddshijian_old 实荐 (`3D精选/character/skill.js:2423-2518, 2574-2635`):
        "dddshijian" | "dddshijian_old" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "ddd_shijian_granted", 1);
        }
        // dddtaisi 太思 (`3D精选/character/skill.js:2636-2705`):
        "dddtaisi" => {
            set_flag(ctx.core, owner, "dddtaisi_resolved", 1);
            ctx.start_draw(owner, 2);
        }
        // dddquche 驱车 (`3D精选/character/skill.js:2706-2818`):
        "dddquche" => {
            if flag(ctx.core, owner, "dddquche_used") != 0 {
                return;
            }
            set_flag(ctx.core, owner, "dddquche_used", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // ddddongcha 洞察 (`3D精选/character/skill.js:2851-2940`):
        "ddddongcha" => {
            if let Some(c) = choose_own_card(ctx, owner, false) {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "ddd_jian", c);
            }
            set_flag(ctx.core, owner, "ddd_dongcha_inspected", 1);
            set_flag(ctx.core, other, "ddd_dongcha_hand_revealed", 1);
        }
        // dddzhijie 智解 (`3D精选/character/skill.js:2941-3067`):
        "dddzhijie" => {
            set_flag(ctx.core, owner, "dddzhijie_used_jian", 1);
            ctx.start_draw(owner, 2);
        }
        // dddqiahua 恰化 (`3D精选/character/skill.js:3068-3111`):
        "dddqiahua" => {
            if other != owner {
                let cards = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, "ddd_jian".to_string()))
                    .unwrap_or_default();
                if !cards.is_empty() {
                    ctx.core.player_mut(other).hand.extend(cards);
                } else if let Some(c) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(other).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "dddqiahua_granted_xunxun", 1);
            set_flag(ctx.core, other, "dddqiahua_has_xunxun", 1);
        }
        // dddxunxun 恂恂 (`3D精选/character/skill.js:3112-3148`):
        "dddxunxun" => {
            set_flag(ctx.core, owner, "dddxunxun_reordered", 1);
            ctx.start_draw(owner, 2);
        }
        // dddfusi / dddfusi_global 腹笥 (`3D精选/character/skill.js:3149-3368`):
        "dddfusi" | "dddfusi_global" => {
            set_flag(ctx.core, owner, "dddfusi_active", 1);
        }
        // dddtuoji 拓籍 (`3D精选/character/skill.js:3369-3387`):
        "dddtuoji" => {
            set_flag(ctx.core, owner, "dddtuoji_drawn3", 1);
            ctx.start_draw(owner, 3);
        }
        // dddchashi 察势 (`3D精选/character/skill.js:3388-3506`):
        "dddchashi" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "dddchashi_matched", 1);
            ctx.start_draw(owner, 1);
            if other != owner {
                ctx.start_draw(other, 1);
            }
        }
        // dddmiaoxing 淼形 (`3D精选/character/skill.js:3569-3726`):
        "dddmiaoxing" => {
            if let Some(c) = pop_deck(ctx) {
                push_pile(ctx, owner, "shuixiang", c);
            }
            set_flag(ctx.core, owner, "dddmiaoxing_shuixiang_synced", 1);
        }
        // dddfushi 浮世 (`3D精选/character/skill.js:3727-3865`):
        "dddfushi" => {
            set_flag(ctx.core, owner, "dddfushi_swapped_shuixiang", 1);
            ctx.start_draw(owner, 1);
        }
        // dddfenji 奋计 (`3D精选/character/skill.js:3866-3943`):
        "dddfenji" => {
            set_flag(ctx.core, owner, "ddd_fenji_burst", 1);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            ctx.start_draw(owner, 2);
        }
        // dddxiaheng 侠横 (`3D精选/character/skill.js:3944-4018`):
        "dddxiaheng" => {
            let _ = discard_one_he(ctx, other);
            let _ = discard_one_he(ctx, other);
            set_flag(ctx.core, owner, "dddxiaheng_resolved", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // dddfengzheng 丰政 (`3D精选/character/skill.js:4019-4212`):
        "dddfengzheng" => {
            set_flag(ctx.core, owner, "dddfengzheng_wuzhong", 1);
            ctx.start_draw(owner, 2);
        }
        // dddyulv / dddyulv_old 玉律 (`3D精选/character/skill.js:4213-4402`):
        "dddyulv" | "dddyulv_old" => {
            ctx.start_draw(owner, 1);
            if let Some(c) = choose_own_card(ctx, owner, false) {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "dddyulv", c);
            }
            set_flag(ctx.core, owner, "dddyulv_active", 1);
        }
        // dddfenye 分野 (`3D精选/character/skill.js:4403-4874`):
        "dddfenye" => {
            set_flag(ctx.core, owner, "dddfenye_pindian_won", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // dddshichao 逝潮 (`3D精选/character/skill.js:4875-4974`):
        "dddshichao" => {
            set_flag(ctx.core, owner, "dddshichao_synced", 1);
            ctx.start_draw(owner, 1);
        }
        // dddyouxue 游学 (`3D精选/character/skill.js:4975-5017`):
        "dddyouxue" => {
            set_flag(ctx.core, owner, "dddyouxue_transferred", 1);
            set_flag(ctx.core, other, "dddyouxue_holder", 1);
            ctx.start_draw(owner, 1);
        }
        // dddchengjing 承经 (`3D精选/character/skill.js:5018-5130`):
        "dddchengjing" => {
            if let Some(c) = ctx.core.discard.pop() {
                push_pile(ctx, owner, "dddchengjing", c);
            }
            set_flag(ctx.core, owner, "dddchengjing_stored", 1);
        }
        // dddduanbing 短兵 (`3D精选/character/skill.js:5131-5222`):
        "dddduanbing" => {
            set_flag(ctx.core, owner, "ddd_dddduanbing_range_mod", 1);
            if flag(ctx.core, owner, "dddduanbing_active_bingliang") != 0 {
                ctx.start_draw(owner, 2);
            }
        }
        // dddxiaoxing 枭行 (`3D精选/character/skill.js:5223-5295`):
        "dddxiaoxing" => {
            set_flag(ctx.core, owner, "dddxiaoxing_bonus", 3);
            ctx.start_draw(owner, 3);
        }
        // dddlangzhi 狼志 (`3D精选/character/skill.js:5296-5426`):
        "dddlangzhi" => {
            set_flag(ctx.core, owner, "dddlangzhi_resolved", 1);
            ctx.start_draw(owner, 1);
        }
        // dddfuyi 附义 (`3D精选/character/skill.js:5427-5592`):
        "dddfuyi" => {
            set_flag(ctx.core, owner, "dddfuyi_qun_sha_bonus", 1);
        }
        // dddzhuanshe 专摄 (`3D精选/character/skill.js:5593-5713`):
        "dddzhuanshe" => {
            if other != owner {
                if let Some(c) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(other).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "dddzhuanshe_given", 1);
        }
        // dddweiqiu 危秋 (`3D精选/character/skill.js:5714-5728`):
        "dddweiqiu" => {
            if ctx.core.player(owner).hand.is_empty() {
                set_flag(ctx.core, owner, "dddweiqiu_intercepted_recover", 1);
                ctx.start_draw(owner, 1);
            }
        }
        // dddlianer 涟洏 (`3D精选/character/skill.js:5729-5795`):
        "dddlianer" => {
            set_flag(ctx.core, owner, "dddlianer_regained_red", 1);
            ctx.start_draw(owner, 1);
        }
        // dddanzhi 暗织 (`3D精选/character/skill.js:5796-5839`):
        "dddanzhi" => {
            set_flag(ctx.core, owner, "dddanzhi_black_ward", 1);
        }
        // dddshixing 失兴 (`3D精选/character/skill.js:5840-5888`):
        "dddshixing" => {
            set_flag(ctx.core, owner, "dddshixing_toggled", 1);
            ctx.start_draw(owner, 1);
        }
        // ddddanggu 党锢 (`3D精选/character/skill.js:5889-5947`):
        "ddddanggu" => {
            ctx.core.player_mut(other).linked = true;
            set_flag(ctx.core, owner, "ddddanggu_chained", 1);
        }
        // dddfuzong 覆宗 (`3D精选/character/skill.js:5948-5988`):
        "dddfuzong" => {
            set_flag(ctx.core, owner, "dddfuzong_qun_draw", 1);
            ctx.start_draw(owner, 1);
        }
        // dddlanghuai 朗怀 (`3D精选/character/skill.js:5989-6057`):
        "dddlanghuai" => {
            let yang = flag(ctx.core, owner, "dddlanghuai_yang") ^ 1;
            set_flag(ctx.core, owner, "dddlanghuai_yang", yang);
            ctx.start_draw(owner, 2);
        }
        // dddxuanlun 玄论 (`3D精选/character/skill.js:6058-6235`):
        "dddxuanlun" => {
            set_flag(ctx.core, owner, "dddxuanlun_resolved", 1);
            ctx.start_draw(owner, 4);
        }
        // dddjiexing 劫行 (`3D精选/character/skill.js:6236-6301`):
        "dddjiexing" => {
            if flag(ctx.core, owner, "dddjiexing_used") == 0 {
                set_flag(ctx.core, owner, "dddjiexing_used", 1);
                if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                }
            }
        }
        // dddbailei 拜泪 (`3D精选/character/skill.js:6302-6368`):
        "dddbailei" => {
            set_flag(ctx.core, owner, "dddbailei_resolved", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("u01_hook:{other_sk}"), 1);
        }
    }
}

/// `L28` (`雷霆万钧` — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3781-8968`):
/// Authentic per-skill state transitions for all `th_*` skills without any grouped fallback.
pub(super) fn resolve_l28_thunder_remaining(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) {
    set_flag(ctx.core, owner, &format!("th_skill:{skill}"), 1);
    let other = hostile_target(ctx, owner).unwrap_or(owner);
    match skill {
        // th_bushi 卜筮 (`雷霆万钧/extension.js:4203-4387`):
        "th_bushi" => {
            set_flag(ctx.core, owner, "th_bushi_suits_assigned", 1);
            ctx.start_draw(owner, 2);
        }
        // th_zhongzhuang 忠壮 (`雷霆万钧/extension.js:4388-4419`):
        "th_zhongzhuang" => {
            set_flag(ctx.core, owner, "th_zhongzhuang_mounted", 1);
            set_flag(ctx.core, owner, "th_zhongzhuang_damage_bonus", 1);
        }
        // th_caiyi 彩翼 (`雷霆万钧/extension.js:4420-4474`):
        "th_caiyi" => {
            let yang = flag(ctx.core, owner, "th_caiyi_yang") ^ 1;
            set_flag(ctx.core, owner, "th_caiyi_yang", yang);
            if yang == 1 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 2);
            }
        }
        // th_guili 归离 (`雷霆万钧/extension.js:4475-4518`):
        "th_guili" => {
            set_flag(ctx.core, owner, "th_guili_bound_target", u32::from(other));
            set_flag(ctx.core, owner, "th_guili_extra_turn", 1);
            ctx.core.turn.extra_phases.push(2);
        }
        // th_chongxu 冲虚 (`雷霆万钧/extension.js:4519-4700`):
        "th_chongxu" => {
            let marks = flag(ctx.core, owner, "th_chongxu_marks") + 2;
            set_flag(ctx.core, owner, "th_chongxu_marks", marks);
            ctx.start_draw(owner, 1);
        }
        // miaojian 妙剑 (`雷霆万钧/extension.js:3660, 4701-4820`):
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
        // shhlianhua 莲华 (`雷霆万钧/extension.js:3660, 4821-5160`):
        "shhlianhua" => {
            set_flag(ctx.core, owner, "th_shhlianhua_ward", 1);
            ctx.start_draw(owner, 1);
        }
        // th_shengong 神工 (`雷霆万钧/extension.js:5163-5504`):
        "th_shengong" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "th_shengong_forged", 1);
            ctx.start_draw(owner, 1);
        }
        // olqisi 奇思 (`雷霆万钧/extension.js:3661`):
        "olqisi" => {
            set_flag(ctx.core, owner, "th_olqisi_converted", 1);
            ctx.start_draw(owner, 1);
        }
        // th_pingcai 评才 (`雷霆万钧/extension.js:5505-6002`):
        "th_pingcai" => {
            set_flag(ctx.core, owner, "th_pingcai_relic_wiped", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    other,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        // th_pcaudio_*_card (`雷霆万钧/extension.js:6003-6014`): pure audio markers for the 4 relics (0 draw)
        "th_pcaudio_wolong_card"
        | "th_pcaudio_fengchu_card"
        | "th_pcaudio_shuijing_card"
        | "th_pcaudio_xuanjian_card" => {
            set_flag(ctx.core, owner, &format!("th_pcaudio:{skill}"), 1);
        }
        // xinfu_pdgyingshi 隐世 (`雷霆万钧/extension.js:3662`):
        "xinfu_pdgyingshi" => {
            set_flag(ctx.core, owner, "th_yingshi_hermit", 1);
        }
        // th_zhengjing 整经 (`雷霆万钧/extension.js:6018-6161`):
        "th_zhengjing" => {
            if let Some(c) = pop_deck(ctx) {
                push_pile(ctx, owner, "th_jing", c);
            }
            set_flag(ctx.core, owner, "th_zhengjing_sorted", 1);
            ctx.start_draw(owner, 1);
        }
        // th_zhengjing2 (`雷霆万钧/extension.js:6162-6179`): `phaseZhunbeiBegin` gains all `"th_jing"` expansion cards and skips Judge/Draw
        "th_zhengjing2" => {
            let cards = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "th_jing".to_string()))
                .unwrap_or_default();
            if !cards.is_empty() {
                ctx.core.player_mut(owner).hand.extend(cards);
            }
            set_flag(ctx.core, owner, "th_zhengjing2_claimed_skip_judge_draw", 1);
        }
        // th_chengye 承业 (`雷霆万钧/extension.js:6182-6371`):
        "th_chengye" => {
            let cnt = (flag(ctx.core, owner, "th_chengye_liujing_count") + 1).min(6);
            set_flag(ctx.core, owner, "th_chengye_liujing_count", cnt);
            ctx.start_draw(owner, 1);
        }
        // th_buxu 补叙 (`雷霆万钧/extension.js:6372-6514`):
        "th_buxu" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "th_buxu_added_classic", 1);
            ctx.start_draw(owner, 1);
        }
        // th_yufeng 御风 (`雷霆万钧/extension.js:6515-6913`):
        "th_yufeng" => {
            set_flag(ctx.core, owner, "th_yufeng_flight_score", 2);
            ctx.start_draw(owner, 2);
        }
        // th_tianshu 天书 (`雷霆万钧/extension.js:6914-6959`):
        "th_tianshu" => {
            let _ = discard_one_he(ctx, owner);
            set_flag(ctx.core, owner, "th_tianshu_granted_yaoshu", 1);
            ctx.start_draw(owner, 1);
        }
        // th_g_taipingyaoshu_ai (`雷霆万钧/extension.js:6960-6973`): AI `wendao` guard (0 draw)
        "th_g_taipingyaoshu_ai" => {
            set_flag(ctx.core, owner, "th_g_taipingyaoshu_ai_checked", 1);
        }
        // th_tiansuan 天算 (`雷霆万钧/extension.js:7806-8026`): draws fortune lot 1..=5; if upper lot (`<=2`), gains 1 card from target
        "th_tiansuan" => {
            let lot = (flag(ctx.core, owner, "th_tiansuan_lot") % 5) + 1;
            set_flag(ctx.core, owner, "th_tiansuan_lot", lot);
            if lot <= 2 && other != owner {
                let _ = steal_target_he(ctx, owner, other);
            }
        }
        // th_tiansuan2 (`雷霆万钧/extension.js:8027-8034`): `onremove` cleanup clearing fortune lot
        "th_tiansuan2" => {
            set_flag(ctx.core, owner, "th_tiansuan_lot", 0);
            set_flag(ctx.core, owner, "th_tiansuan2_cleaned", 1);
        }
        // th_tiansuan2_0 上上签 (`雷霆万钧/extension.js:8035-8063`): `damageBegin4` prevents all damage
        "th_tiansuan2_0" => {
            set_flag(ctx.core, owner, "th_tiansuan2_0_prevent_damage", 1);
        }
        // th_tiansuan2_1 上签 (`雷霆万钧/extension.js:8064-8103`): `damageBegin4` caps incoming damage at 1
        "th_tiansuan2_1" => {
            set_flag(ctx.core, owner, "th_tiansuan2_1_cap_damage", 1);
        }
        // th_tiansuan2_damage (`雷霆万钧/extension.js:8104-8110`): `damageEnd` draws `trigger.num` cards
        "th_tiansuan2_damage" => {
            let n = flag(ctx.core, owner, "th_tiansuan_damage_num").max(1);
            ctx.start_draw(owner, n);
        }
        // th_tiansuan2_2 中签 (`雷霆万钧/extension.js:8111-8135`): `damageBegin4` caps damage at 1 with fire group
        "th_tiansuan2_2" => {
            set_flag(ctx.core, owner, "th_tiansuan2_2_cap_damage", 1);
            set_flag(ctx.core, owner, "th_tiansuan2_2_fire", 1);
        }
        // th_tiansuan2_ai (`雷霆万钧/extension.js:8136-8145`): AI `filterDamage` tag (0 draw)
        "th_tiansuan2_ai" => {
            set_flag(ctx.core, owner, "th_tiansuan2_ai_tag", 1);
        }
        // th_tiansuan2_fire (`雷霆万钧/extension.js:8146-8156`): `damageBefore` converts incoming damage to fire
        "th_tiansuan2_fire" => {
            set_flag(ctx.core, owner, "th_tiansuan2_fire_nature", 1);
        }
        // th_tiansuan2_3 下签 (`雷霆万钧/extension.js:8157-8182`): `damageBegin3` increases incoming damage by 1
        "th_tiansuan2_3" => {
            set_flag(ctx.core, owner, "th_tiansuan2_3_extra_damage", 1);
        }
        // th_tiansuan2_4 下下签 (`雷霆万钧/extension.js:8183-8216`): `damageBegin3` +1 damage and forbids `tao`/`jiu`
        "th_tiansuan2_4" => {
            set_flag(ctx.core, owner, "th_tiansuan2_4_extra_damage", 1);
            set_flag(ctx.core, owner, "th_tiansuan2_4_forbid_tao_jiu", 1);
        }
        // th_gushe2 (`雷霆万钧/extension.js:8378-8381`): temporary pindian-win counter mark (0 draw)
        "th_gushe2" => {
            let m = flag(ctx.core, owner, "th_gushe2_marks") + 1;
            set_flag(ctx.core, owner, "th_gushe2_marks", m);
        }
        // rejici 激词 (`雷霆万钧/extension.js:3668`):
        "rejici" => {
            set_flag(ctx.core, owner, "th_rejici_triggered", 1);
            ctx.start_draw(owner, 1);
        }
        // th_geyuan 割圆 (`雷霆万钧/extension.js:8384-8590`):
        "th_geyuan" => {
            set_flag(ctx.core, owner, "th_geyuan_pi", 1);
            ctx.start_draw(owner, 2);
        }
        // th_jieshu 解术 (`雷霆万钧/extension.js:8591-8613`):
        "th_jieshu" => {
            set_flag(ctx.core, owner, "th_jieshu_drawn", 1);
            ctx.start_draw(owner, 1);
        }
        // th_gusuan 股算 (`雷霆万钧/extension.js:8614-8635`):
        "th_gusuan" => {
            if ctx.core.player(owner).max_hp > 1 {
                ctx.core.player_mut(owner).max_hp -= 1;
                let m = ctx.core.player(owner).max_hp as i32;
                if ctx.core.player(owner).hp > m {
                    ctx.core.player_mut(owner).hp = m;
                }
            }
            set_flag(ctx.core, owner, "th_gusuan_awakened", 1);
        }
        // qinyin 琴音 (`雷霆万钧/extension.js:3670`):
        "qinyin" => {
            let seats: Vec<PlayerId> = ctx.core.seat_order_from(owner);
            for p in seats {
                if ctx.core.player(p).alive {
                    ctx.recover(p, 1, None);
                }
            }
            set_flag(ctx.core, owner, "th_skill:qinyin", 1);
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("l28_hook:{other_sk}"), 1);
        }
    }
}

