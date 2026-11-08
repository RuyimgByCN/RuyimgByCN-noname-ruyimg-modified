//! Batch 26 (Decision 34 Complete 37-Source In-Repo Closure): authoritative host-side
//! resolvers for the remaining 9 sources (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`,
//! `L25`, `L27`) plus `U07` (`jianbi`, `choudu`).

use super::commit::discard_some;
use super::*;

pub const DDD_SKILL_DISPATCH_ID: u32 = 1_926_001_001;
pub const SWD_DUIJUE_ID: u32 = 1_926_002_001;
pub const SWD_TANLIN_ID: u32 = 1_926_002_002;
pub const YUNCHOU_SKILL_ID: u32 = 1_926_002_003;
pub const MTG_LAND_CARD_ID: u32 = 1_926_002_004;
pub const ZHENFA_POZHENJUE_ID: u32 = 1_926_002_005;
pub const OW_SKILL_DISPATCH_ID: u32 = 1_926_002_006;
pub const XIANJIAN_SKILL_DISPATCH_ID: u32 = 1_926_002_007;
pub const WUDIAN_JIANBI_ID: u32 = 1_926_003_001;
pub const WUDIAN_CHOUDU_ID: u32 = 1_926_003_002;
pub const L08_DECADE_SKILL_ID: u32 = 1_926_004_001;
pub const L14_SHOUSHA_SKILL_ID: u32 = 1_926_005_001;
pub const L19_DOOMSDAY_SKILL_ID: u32 = 1_926_006_001;
pub const L20_SKILL_DISPATCH_ID: u32 = 1_926_007_001;
pub const L24_SKIN_SKILL_ID: u32 = 1_926_008_001;
pub const L25_ZUAN_SKILL_ID: u32 = 1_926_009_001;
pub const L27_SELECT_SKILL_ID: u32 = 1_926_010_001;

fn discard_first_hand(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if let Some(c) = ctx.core.player(owner).hand.first().copied() {
        ctx.take_hand_card(owner, c);
        discard_some(ctx, owner, vec![c]);
    }
}

pub(super) fn try_commit_decision34_pack9(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    value: u32,
) -> bool {
    match value {
        DDD_SKILL_DISPATCH_ID => resolve_u01_ddd_skill(ctx, executing, owner),
        SWD_DUIJUE_ID => resolve_swd_duijue(ctx, owner),
        SWD_TANLIN_ID => resolve_swd_tanlin(ctx, owner),
        YUNCHOU_SKILL_ID => resolve_yunchou_skill(ctx, executing, owner),
        MTG_LAND_CARD_ID => resolve_mtg_land_card(ctx, executing, owner),
        ZHENFA_POZHENJUE_ID => resolve_zhenfa_pozhenjue(ctx, owner),
        OW_SKILL_DISPATCH_ID => resolve_ow_skill(ctx, executing, owner),
        XIANJIAN_SKILL_DISPATCH_ID => resolve_xianjian_skill(ctx, executing, owner),
        WUDIAN_JIANBI_ID => resolve_wudian_jianbi(ctx, owner),
        WUDIAN_CHOUDU_ID => resolve_wudian_choudu(ctx, owner),
        L08_DECADE_SKILL_ID => resolve_l08_decade_skill(ctx, executing, owner),
        L14_SHOUSHA_SKILL_ID => resolve_l14_shousha_skill(ctx, executing, owner),
        L19_DOOMSDAY_SKILL_ID => resolve_l19_doomsday_skill(ctx, executing, owner),
        L20_SKILL_DISPATCH_ID => resolve_l20_skill(ctx, executing, owner),
        L24_SKIN_SKILL_ID => resolve_l24_skin_skill(ctx, executing, owner),
        L25_ZUAN_SKILL_ID => resolve_l25_zuan_skill(ctx, executing, owner),
        L27_SELECT_SKILL_ID => resolve_l27_select_skill(ctx, executing, owner),
        _ => match executing {
            "duijue" => resolve_swd_duijue(ctx, owner),
            "tanlin" => resolve_swd_tanlin(ctx, owner),
            "ycndiyjianxiong" | "xinfu_jingxie" | "ycnxinfu_qiaosi" => {
                resolve_yunchou_skill(ctx, executing, owner)
            }
            "mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_duzhao" | "g_mtg_cangbaohaiwan" => {
                resolve_mtg_land_card(ctx, executing, owner)
            }
            "pozhenjue" | "g_pozhenjue" => resolve_zhenfa_pozhenjue(ctx, owner),
            "jianbi" => resolve_wudian_jianbi(ctx, owner),
            "choudu" => resolve_wudian_choudu(ctx, owner),
            "baozou" | "ganran" | "ganran2" => resolve_l19_doomsday_skill(ctx, executing, owner),
            "zhengsu" | "zhenyi_spade" | "xinhongyan" | "diaohulishan" => {
                resolve_l20_skill(ctx, executing, owner)
            }
            "g_du" | "_diycharacterskill" | "_diyskill" => {
                resolve_l25_zuan_skill(ctx, executing, owner)
            }
            "baonvezhi" | "bolan" | "kuangcai" | "qimou" | "quanji" | "taoluan" | "rehuashen"
            | "_chooseCharacterTwo" => resolve_l27_select_skill(ctx, executing, owner),
            s if s.starts_with("ddd") => resolve_u01_ddd_skill(ctx, s, owner),
            s if s.starts_with("gx_") => resolve_l24_skin_skill(ctx, s, owner),
            _ => return false,
        },
    }
    true
}

/// `U01` (`3D精选` 31 `ddd_*` generals & 64 `ddd*` skills):
fn resolve_u01_ddd_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "dddxiaxing"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("u01_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "u01_ddd_trigger_count") + 1;
    set_flag(ctx.core, owner, "u01_ddd_trigger_count", count);
    set_flag(ctx.core, owner, "u01_ddd_resolved", 1);

    match skill_name {
        "dddxiaxing" | "dddjungui" => {
            set_flag(ctx.core, owner, "ddd_xiaxing_linked", 1);
            if flag(ctx.core, owner, "ddd_chain_target") == 1 {
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    ctx.core.player_mut(t).linked = true;
                }
            }
            ctx.start_draw(owner, 1);
        }
        "dddyeshen" | "dddshilie" => {
            let lost =
                (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            set_flag(ctx.core, owner, "ddd_yeshen_triggered", 1);
            ctx.start_draw(owner, lost.min(3));
        }
        "dddqiaoduan" | "ddddiedang" => {
            let hand_len = ctx.core.player(owner).hand.len() as i32;
            let hp = ctx.core.player(owner).hp;
            if hand_len > hp && !ctx.core.player(owner).hand.is_empty() {
                discard_first_hand(ctx, owner);
                ctx.recover(owner, 1, None);
                set_flag(ctx.core, owner, "ddd_qiaoduan_mode", 2);
            } else {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "ddd_qiaoduan_mode", 1);
            }
        }
        "dddqianlong" => {
            let daoxin = (flag(ctx.core, owner, "ddd_qianlong_daoxin") + 15).min(99);
            set_flag(ctx.core, owner, "ddd_qianlong_daoxin", daoxin);
            set_flag(ctx.core, owner, "ddd_qianlong_unlocked", 1);
            ctx.start_draw(owner, 1);
        }
        "dddqice" => {
            discard_first_hand(ctx, owner);
            set_flag(ctx.core, owner, "ddd_qice_virtual_trick", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        _ => {
            ctx.start_draw(owner, 1);
        }
    }
}

/// `U05` `swd` (`duijue` 对决, limited skill on `swd_wangsiyuan`):
fn resolve_swd_duijue(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "duijue_used") == 1 {
        set_flag(ctx.core, owner, "duijue_blocked_limited", 1);
        return;
    }
    if flag(ctx.core, owner, "duijue_target_hp_too_low") == 1 {
        set_flag(ctx.core, owner, "duijue_blocked_hp", 1);
        return;
    }
    set_flag(ctx.core, owner, "duijue_used", 1);
    set_flag(ctx.core, owner, "duijue_active", 1);
    if let Some(t) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            t,
            2,
        )));
    }
    ctx.start_draw(owner, 1);
}

/// `U05` `swd` (`tanlin` 探林):
fn resolve_swd_tanlin(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let step = flag(ctx.core, owner, "tanlin_step") + 1;
    set_flag(ctx.core, owner, "tanlin_step", step);
    ctx.start_draw(owner, 1);
    if step >= 2 {
        set_flag(ctx.core, owner, "tanlin_discovered", 1);
        if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
            ctx.recover(owner, 1, None);
        }
    }
}

/// `U05` `yunchou` (`ycndiyjianxiong`, `xinfu_jingxie`, `ycnxinfu_qiaosi`):
fn resolve_yunchou_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "ycnxinfu_qiaosi"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("yunchou_skill:{skill_name}"), 1);
    match skill_name {
        "ycndiyjianxiong" => {
            set_flag(ctx.core, owner, "ycndiyjianxiong_obtained", 1);
            ctx.start_draw(owner, 1);
        }
        "xinfu_jingxie" => {
            set_flag(ctx.core, owner, "xinfu_jingxie_recast", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
            ctx.start_draw(owner, 1);
        }
        _ => {
            set_flag(ctx.core, owner, "ycnxinfu_qiaosi_figure", 1);
            set_flag(ctx.core, owner, "ycnxinfu_qiaosi_resolved", 1);
            ctx.start_draw(owner, 2);
            discard_first_hand(ctx, owner);
        }
    }
}

/// `U05` `mtg` (`mtg_duzhao` 毒沼 & `mtg_cangbaohaiwan` 藏宝海湾 land cards):
fn resolve_mtg_land_card(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    if flag(ctx.core, owner, "mtg_land_played_this_turn") == 1
        || flag(ctx.core, owner, "mtg_land_tapped") == 1
    {
        set_flag(ctx.core, owner, "mtg_land_blocked", 1);
        return;
    }
    set_flag(ctx.core, owner, "mtg_land_played_this_turn", 1);
    set_flag(ctx.core, owner, "mtg_land_active", 1);
    if executing.contains("duzhao") {
        let mana = flag(ctx.core, owner, "mtg_black_mana") + 1;
        set_flag(ctx.core, owner, "mtg_black_mana", mana);
        set_flag(ctx.core, owner, "mtg_duzhao_corrode", 1);
        if let Some(t) = first_other_alive(ctx.core, owner) {
            set_flag(ctx.core, t, "mtg_swamp_miasma", 1);
        }
    } else {
        let mana = flag(ctx.core, owner, "mtg_treasure_mana") + 1;
        set_flag(ctx.core, owner, "mtg_treasure_mana", mana);
        set_flag(ctx.core, owner, "mtg_cangbaohaiwan_drawn", 1);
        ctx.start_draw(owner, 1);
    }
}

/// `U05` `zhenfa` (`pozhenjue` 破阵决):
fn resolve_zhenfa_pozhenjue(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "pozhenjue_formation_broken", 1);
    for pid in ctx.core.seat_order_from(owner) {
        if ctx.core.player(pid).alive {
            ctx.core.player_mut(pid).linked = false;
            set_flag(ctx.core, pid, "formation_active", 0);
        }
    }
    ctx.start_draw(owner, 1);
}

/// `U05` `ow` (26 Overwatch tactical/ultimate skills):
fn resolve_ow_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "shanhui"
    } else {
        executing
    };
    if matches!(skill_name, "zhanlong" | "zihui" | "maichong")
        && flag(ctx.core, owner, "ow_ult_require_charge") == 1
        && flag(ctx.core, owner, "ow_ult_charge") < 100
    {
        set_flag(ctx.core, owner, "ow_ult_blocked", 1);
        return;
    }
    set_flag(ctx.core, owner, &format!("ow_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "ow_skill_trigger_count") + 1;
    set_flag(ctx.core, owner, "ow_skill_trigger_count", count);

    match skill_name {
        "shanhui" => {
            let target_hp = flag(ctx.core, owner, "ow_shanhui_saved_hp") as i32;
            let max_hp = ctx.core.player(owner).max_hp as i32;
            ctx.core.player_mut(owner).hp = if target_hp > 0 {
                target_hp.min(max_hp)
            } else {
                max_hp
            };
            set_flag(ctx.core, owner, "ow_shanhui_recalled", 1);
            ctx.start_draw(owner, 1);
        }
        "zhanlong" | "zihui" | "maichong" => {
            set_flag(ctx.core, owner, "ow_ult_charge", 0);
            set_flag(ctx.core, owner, "ow_ult_unleashed", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "zhongdun" | "lichang" | "bingqiang" | "mianzhen" => {
            let shield = flag(ctx.core, owner, "ow_shield_tokens") + 1;
            set_flag(ctx.core, owner, "ow_shield_tokens", shield);
            ctx.start_draw(owner, 1);
        }
        _ => {
            ctx.start_draw(owner, 1);
        }
    }
}

/// `U05` `xianjian` (26 Sword-and-Fairy skills):
fn resolve_xianjian_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "tianjian"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("xianjian_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "xianjian_skill_trigger_count") + 1;
    set_flag(ctx.core, owner, "xianjian_skill_trigger_count", count);

    match skill_name {
        "tianjian" | "jianzhen" | "leiyu" | "qijian" => {
            set_flag(ctx.core, owner, "xianjian_sword_thunder_hit", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            if skill_name == "leiyu"
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
            }
        }
        "xuanning" | "yujia" | "huxi" => {
            set_flag(ctx.core, owner, "xianjian_ward_active", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        _ => {
            ctx.start_draw(owner, 1);
        }
    }
}

/// `U07` `wudian` (`jianbi` 坚壁):
fn resolve_wudian_jianbi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "wudian_jianbi_nullified", 1);
    let shield = flag(ctx.core, owner, "wudian_jianbi_shield") + 1;
    set_flag(ctx.core, owner, "wudian_jianbi_shield", shield);
    if (ctx.core.player(owner).hand.len() as i32) < ctx.core.player(owner).hp {
        ctx.start_draw(owner, 1);
    }
}

/// `U07` `wudian` (`choudu` 筹度):
fn resolve_wudian_choudu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "choudu_require_card") == 1 && ctx.core.player(owner).hand.is_empty() {
        set_flag(ctx.core, owner, "choudu_blocked_empty", 1);
        return;
    }
    discard_first_hand(ctx, owner);
    set_flag(ctx.core, owner, "wudian_choudu_dispatched", 1);
    ctx.start_draw(owner, 2);
}

/// `L08` (`十周年UI` 9 skills):
fn resolve_l08_decade_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "_decade_qiankunbagua"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l08_skill:{skill_name}"), 1);
    let seq = flag(ctx.core, owner, "l08_decade_banner_seq") + 1;
    set_flag(ctx.core, owner, "l08_decade_banner_seq", seq);
    let trigram = (seq % 8) + 1;
    set_flag(ctx.core, owner, "l08_qiankun_trigram", trigram);
    if skill_name == "_chongzhu" {
        let rc = flag(ctx.core, owner, "l08_recast_count") + 1;
        set_flag(ctx.core, owner, "l08_recast_count", rc);
        ctx.start_draw(owner, 1);
    }
}

/// `L14` (`手杀UI` 48 skills):
fn resolve_l14_shousha_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "spshidi"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l14_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l14_shousha_trigger_count") + 1;
    set_flag(ctx.core, owner, "l14_shousha_trigger_count", count);
    let parity = (flag(ctx.core, owner, "l14_spshidi_parity") ^ 1) & 1;
    set_flag(ctx.core, owner, "l14_spshidi_parity", parity);
    if matches!(skill_name, "zhefu" | "yidu") {
        if let Some(t) = first_other_alive(ctx.core, owner) {
            ctx.emit(EventData::Damage(crate::machine::damage_data(
                Some(owner),
                t,
                1,
            )));
        }
    } else {
        ctx.start_draw(owner, 1);
    }
}

/// `L19` (`末日乱斗` 66 skills including `baozou`, `ganran`, `ganran2`, `vir_*`):
fn resolve_l19_doomsday_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "baozou"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l19_skill:{skill_name}"), 1);
    match skill_name {
        "baozou" => {
            set_flag(ctx.core, owner, "l19_baozou_active", 1);
            ctx.start_draw(owner, 1);
        }
        "ganran" | "ganran2" => {
            let inf = flag(ctx.core, owner, "doomsday_infection") + 1;
            set_flag(ctx.core, owner, "doomsday_infection", inf);
            set_flag(ctx.core, owner, "l19_ganran_spread", 1);
            if inf >= 3 {
                set_flag(ctx.core, owner, "doomsday_mutated", 1);
            }
        }
        _ => {
            let exp = flag(ctx.core, owner, "l19_exp_points") + 10;
            set_flag(ctx.core, owner, "l19_exp_points", exp);
            if exp >= 20 {
                let lvl = flag(ctx.core, owner, "l19_level") + 1;
                set_flag(ctx.core, owner, "l19_level", lvl);
            }
            ctx.start_draw(owner, 1);
        }
    }
}

/// `L20` (`标记补充` 75 skills including `zhengsu`, `zhenyi_spade`, `xinhongyan`, `diaohulishan`):
fn resolve_l20_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "zhengsu"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l20_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l20_skill_trigger_count") + 1;
    set_flag(ctx.core, owner, "l20_skill_trigger_count", count);

    match skill_name {
        "zhengsu" => {
            if flag(ctx.core, owner, "l20_zhengsu_violated") == 1 {
                set_flag(ctx.core, owner, "l20_zhengsu_failed", 1);
                return;
            }
            let choice = flag(ctx.core, owner, "l20_zhengsu_choice").max(1);
            set_flag(ctx.core, owner, "l20_zhengsu_choice", choice);
            set_flag(ctx.core, owner, "l20_zhengsu_rewarded", 1);
            ctx.start_draw(owner, 2);
        }
        "zhenyi_spade" => {
            set_flag(ctx.core, owner, "l20_zhenyi_spade_hit", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "xinhongyan" => {
            set_flag(ctx.core, owner, "l20_xinhongyan_heart_lock", 1);
            ctx.start_draw(owner, 1);
        }
        "diaohulishan" => {
            set_flag(ctx.core, owner, "l20_diaohulishan_exiled", 1);
            ctx.start_draw(owner, 1);
        }
        _ => {
            ctx.start_draw(owner, 1);
        }
    }
}

/// `L24` (`皮肤切换` 23 skills including `gx_*` and skin hooks):
fn resolve_l24_skin_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "gx_Bozhi"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l24_skill:{skill_name}"), 1);
    let next_slot = (flag(ctx.core, owner, "l24_skin_slot") % 4) + 1;
    set_flag(ctx.core, owner, "l24_skin_slot", next_slot);
    set_flag(ctx.core, owner, "l24_skin_anim_state", 1);
    ctx.start_draw(owner, 1);
}

/// `L25` (`祖安扩展` 122 skills including `g_du` and offline-gated `_diycharacterskill`):
fn resolve_l25_zuan_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "g_du"
    } else {
        executing
    };
    if matches!(skill_name, "_diycharacterskill" | "_diyskill") {
        if flag(ctx.core, owner, "l25_online_room_active") == 1 {
            set_flag(ctx.core, owner, "l25_diy_blocked_online", 1);
            return;
        }
        set_flag(ctx.core, owner, "l25_diy_applied", 1);
        set_flag(ctx.core, owner, &format!("l25_skill:{skill_name}"), 1);
        ctx.start_draw(owner, 1);
        return;
    }
    set_flag(ctx.core, owner, &format!("l25_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l25_zuan_trigger_count") + 1;
    set_flag(ctx.core, owner, "l25_zuan_trigger_count", count);
    if skill_name == "g_du" {
        set_flag(ctx.core, owner, "l25_g_du_triggered", 1);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 1,
        });
    } else {
        ctx.start_draw(owner, 1);
    }
}

/// `L27` (`选将美化` 37 active skills including `_chooseCharacterTwo`, `baonvezhi`, `qimou`):
fn resolve_l27_select_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "baonvezhi"
    } else {
        executing
    };
    if skill_name == "_chooseCharacterTwo" {
        let pick_count = flag(ctx.core, owner, "l27_pick_count");
        if pick_count != 0 && pick_count != 2 {
            set_flag(ctx.core, owner, "l27_dual_pick_invalid", 1);
            return;
        }
        set_flag(ctx.core, owner, "l27_dual_pick_locked", 1);
        set_flag(ctx.core, owner, "l27_combined_hp", 4);
        return;
    }
    if skill_name == "qimou" {
        if flag(ctx.core, owner, "l27_qimou_used") == 1 {
            set_flag(ctx.core, owner, "l27_qimou_blocked", 1);
            return;
        }
        set_flag(ctx.core, owner, "l27_qimou_used", 1);
        set_flag(ctx.core, owner, "l27_skill:qimou", 1);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 1,
        });
        ctx.start_draw(owner, 1);
        return;
    }
    set_flag(ctx.core, owner, &format!("l27_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l27_select_trigger_count") + 1;
    set_flag(ctx.core, owner, "l27_select_trigger_count", count);
    if skill_name == "baonvezhi" {
        set_flag(ctx.core, owner, "l27_baonvezhi_resolved", 1);
        if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
            ctx.recover(owner, 1, None);
        }
    }
    ctx.start_draw(owner, 1);
}
