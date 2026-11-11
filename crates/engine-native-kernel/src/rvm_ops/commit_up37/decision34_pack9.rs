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
pub const U05_SHAHAI_SKILL_ID: u32 = 1_926_002_010;
pub const WUDIAN_JIANBI_ID: u32 = 1_926_003_001;
pub const WUDIAN_CHOUDU_ID: u32 = 1_926_003_002;
pub const L08_DECADE_SKILL_ID: u32 = 1_926_004_001;
pub const L14_SHOUSHA_SKILL_ID: u32 = 1_926_005_001;
pub const L19_DOOMSDAY_SKILL_ID: u32 = 1_926_006_001;
pub const L20_SKILL_DISPATCH_ID: u32 = 1_926_007_001;
pub const L24_SKIN_SKILL_ID: u32 = 1_926_008_001;
pub const L25_ZUAN_SKILL_ID: u32 = 1_926_009_001;
pub const L27_SELECT_SKILL_ID: u32 = 1_926_010_001;
pub const L11_TAIXU_SKILL_ID: u32 = 1_926_011_001;



pub(super) fn try_commit_decision34_pack9(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    value: u32,
) -> bool {
    match value {
        DDD_SKILL_DISPATCH_ID => resolve_u01_ddd_skill(ctx, executing, owner),
        SWD_DUIJUE_ID => resolve_u05_shahai_skill(ctx, executing, "duijue", owner),
        SWD_TANLIN_ID => resolve_u05_shahai_skill(ctx, executing, "tanlin", owner),
        YUNCHOU_SKILL_ID => resolve_u05_shahai_skill(ctx, executing, "ycnxinfu_qiaosi", owner),
        MTG_LAND_CARD_ID => resolve_u05_shahai_skill(ctx, executing, "mtg_cangbaohaiwan", owner),
        ZHENFA_POZHENJUE_ID => resolve_u05_shahai_skill(ctx, executing, "pozhenjue", owner),
        OW_SKILL_DISPATCH_ID => resolve_u05_shahai_skill(ctx, executing, "shanhui", owner),
        XIANJIAN_SKILL_DISPATCH_ID => resolve_u05_shahai_skill(ctx, executing, "tianjian", owner),
        U05_SHAHAI_SKILL_ID => resolve_u05_shahai_skill(ctx, executing, "stone_chenshui", owner),
        WUDIAN_JIANBI_ID => resolve_wudian_jianbi(ctx, owner),
        WUDIAN_CHOUDU_ID => resolve_wudian_choudu(ctx, owner),
        L08_DECADE_SKILL_ID => resolve_l08_decade_skill(ctx, executing, owner),
        L14_SHOUSHA_SKILL_ID => resolve_l14_shousha_skill(ctx, executing, owner),
        L19_DOOMSDAY_SKILL_ID => resolve_l19_doomsday_skill(ctx, executing, owner),
        L20_SKILL_DISPATCH_ID => resolve_l20_skill(ctx, executing, owner),
        L24_SKIN_SKILL_ID => resolve_l24_skin_skill(ctx, executing, owner),
        L25_ZUAN_SKILL_ID => resolve_l25_zuan_skill(ctx, executing, owner),
        L27_SELECT_SKILL_ID => resolve_l27_select_skill(ctx, executing, owner),
        L11_TAIXU_SKILL_ID => {
            super::decision34_l11_skills::resolve_l11_taixu_skill(ctx, executing, owner)
        }
        _ => match executing {
            "duijue"
            | "tanlin"
            | "ycndiyjianxiong"
            | "xinfu_jingxie"
            | "ycnxinfu_qiaosi"
            | "mtg_duzhao"
            | "mtg_cangbaohaiwan"
            | "g_mtg_duzhao"
            | "g_mtg_cangbaohaiwan"
            | "pozhenjue"
            | "g_pozhenjue" => resolve_u05_shahai_skill(ctx, executing, executing, owner),
            "jianbi" => resolve_wudian_jianbi(ctx, owner),
            "choudu" => resolve_wudian_choudu(ctx, owner),
            "_changeEquipExt"
            | "_ui_auto_cheat"
            | "_kill_framebg"
            | "_rarity"
            | "_tiesuo_ying"
            | "_usecard"
            | "_decadeUI_usecardBegin"
            | "_discard"
            | "_decadeUI_dieKillEffect"
            | "_chongzhu" => resolve_l08_decade_skill(ctx, executing, owner),
            "_dieNoTuoguan" | "_menuCheat" | "_jindutiao" | "spshidi" | "zhefu" | "yidu" => {
                resolve_l14_shousha_skill(ctx, executing, owner)
            }
            "baozou" | "ganran" | "ganran2" | "_jisuangailv" | "_tuizhi" => {
                resolve_l19_doomsday_skill(ctx, executing, owner)
            }
            "zhengsu" | "zhenyi_spade" | "xinhongyan" | "diaohulishan" | "_xigua_feiyanbahu_" => {
                resolve_l20_skill(ctx, executing, owner)
            }
            "_tsx" | "_gj" | "_ts" | "_pfqh_reapply_saved_params" | "_pfqh_check_hp_change" => {
                resolve_l24_skin_skill(ctx, executing, owner)
            }
            "g_du" | "_diycharacterskill" | "_diyskill" | "_player_xxgg_zhishixian_xxxxx_ggg" => {
                resolve_l25_zuan_skill(ctx, executing, owner)
            }
            "baonvezhi" | "bolan" | "kuangcai" | "qimou" | "quanji" | "taoluan" | "rehuashen"
            | "clandianzhan" | "drlt_duorui" | "_chooseCharacterTwo" => {
                resolve_l27_select_skill(ctx, executing, owner)
            }
            s if s.starts_with("ddd") => resolve_u01_ddd_skill(ctx, s, owner),
            s if s.starts_with("_pfqh_") => resolve_l24_skin_skill(ctx, s, owner),
            _ => return false,
        },
    }
    true
}

/// `U01` (`3D精选` — `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js`):
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
        // dddxiaxing 侠行 (`3D精选/character/skill.js:59-242`):
        // Without "侠" mark (`dddxiaxing_xia == 0`), requires >= 1 `he` card (`skill.js:72, 129`),
        // puts 1 card on deck top to view-as tao/jiu and gains "侠" mark (`dddxiaxing_xia = 1`);
        // with "侠" mark (`dddxiaxing_viewas` at `skill.js:163-208`), removes "侠" mark
        // (`dddxiaxing_xia = 0`) to view-as `sha`/`shan` (`selectCard: -1`, `cards: []`) WITHOUT drawing cards.
        "dddxiaxing" => {
            if flag(ctx.core, owner, "dddxiaxing_xia") == 0 {
                let Some(c) = choose_own_card(ctx, owner, true) else {
                    set_flag(ctx.core, owner, "dddxiaxing_blocked_empty", 1);
                    return;
                };
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.take_hand_card(owner, c);
                } else {
                    ctx.take_owned_card(owner, c);
                }
                ctx.core.deck.push(c);
                set_flag(ctx.core, owner, "dddxiaxing_xia", 1);
                set_flag(ctx.core, owner, "dddxiaxing_viewas_taojiu", 1);
            } else {
                set_flag(ctx.core, owner, "dddxiaxing_xia", 0);
                set_flag(ctx.core, owner, "dddxiaxing_viewas_basic", 1);
            }
        }
        // dddyeshen 夜审 (`3D精选/character/skill.js:656`):
        // At PhaseJieshu, reveals bottom cards (starting at 3, decrementing by 1 each use),
        // lets target use/recast a black card as tiesuo, and when count reaches 0 resets to 3
        // and takes 1 fire damage (`DamageNature::Fire`).
        "dddyeshen" => {
            let prev = flag(ctx.core, owner, "dddyeshen_count");
            let current = if prev == 0 { 3 } else { prev };
            let next = current.saturating_sub(1);
            set_flag(ctx.core, owner, "ddd_yeshen_triggered", 1);
            let yeshen_target = if flag(ctx.core, owner, "dddyeshen_recast") != 0 {
                friendly_target(ctx, owner)
            } else {
                hostile_target(ctx, owner)
            };
            if let Some(t) = yeshen_target {
                if flag(ctx.core, owner, "dddyeshen_recast") != 0 {
                    ctx.start_draw(t, 1);
                } else {
                    ctx.core.player_mut(t).linked = !ctx.core.player(t).linked;
                }
            }
            if next == 0 {
                set_flag(ctx.core, owner, "dddyeshen_count", 3);
                set_flag(ctx.core, owner, "dddyeshen_fire_backfire", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    owner,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            } else {
                set_flag(ctx.core, owner, "dddyeshen_count", next);
            }
        }
        // dddqiaoduan 巧断 (`3D精选/character/skill.js:747`):
        // When a character unchains (resets), lets up to X characters draw 1 card each;
        // when a character chains (links), puts X cards on bottom of deck and recovers 1 HP.
        "dddqiaoduan" => {
            let chained_count = (0..ctx.core.players.len() as PlayerId)
                .filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).linked)
                .count()
                .max(1) as u32;
            if flag(ctx.core, owner, "ddd_qiaoduan_from_link") != 0
                || ctx.core.player(owner).linked
            {
                for _ in 0..chained_count {
                    if let Some(c) = choose_own_card(ctx, owner, false) {
                        ctx.take_hand_card(owner, c);
                        ctx.core.deck.insert(0, c);
                    }
                }
                if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                }
                set_flag(ctx.core, owner, "ddd_qiaoduan_mode", 2);
            } else {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "ddd_qiaoduan_mode", 1);
            }
        }
        // dddqianlong 潜龙 (`3D精选/character/skill.js:2819`):
        // Lord skill on `ddd_caomao` — cancels reward/punishment draw/discard, and draws 3 cards
        // when killing a character (`player.draw(3)` at `skill.js:2842`).
        "dddqianlong" => {
            set_flag(ctx.core, owner, "ddd_qianlong_kill_reward", 1);
            ctx.start_draw(owner, 3);
        }
        // dddqice 奇策 (`3D精选/character/skill.js:3507`):
        // At PhaseZhunbei (`phaseZhunbeiBegin`), draws 2 cards (`player.draw(2)`) and disables
        // `dddqice` (`ddd_qice_disabled = 1`) until the player uses a trick card.
        "dddqice" => {
            if flag(ctx.core, owner, "ddd_qice_trick_used") != 0 {
                set_flag(ctx.core, owner, "ddd_qice_disabled", 0);
                set_flag(ctx.core, owner, "ddd_qice_trick_used", 0);
                return;
            }
            if flag(ctx.core, owner, "ddd_qice_disabled") != 0 {
                set_flag(ctx.core, owner, "ddd_qice_blocked_disabled", 1);
                return;
            }
            set_flag(ctx.core, owner, "ddd_qice_disabled", 1);
            ctx.start_draw(owner, 2);
        }
        // dddjungui 隽轨 (`3D精选/character/skill.js:384-441`):
        // At PhaseJieshu, target draws 2 cards (`target.draw(2)`), shows all handcards, and
        // discards X cards of different suits (`X = suits - decreased`).
        "dddjungui" => {
            let t = friendly_target(ctx, owner).unwrap_or(owner);
            set_flag(ctx.core, owner, "ddd_jungui_triggered", 1);
            set_flag(ctx.core, t, "ddd_jungui_shown_hand", 1);
            ctx.start_draw(t, 2);
            let decreased = flag(ctx.core, owner, "dddjungui_decreased") as usize;
            let discard_n = (2usize).saturating_sub(decreased);
            let mut to_discard = Vec::new();
            for _ in 0..discard_n {
                if let Some(c) = choose_own_card(ctx, t, false) {
                    ctx.take_hand_card(t, c);
                    to_discard.push(c);
                }
            }
            let disc_len = to_discard.len() as u32;
            if !to_discard.is_empty() {
                discard_some(ctx, t, to_discard);
            }
            set_flag(ctx.core, owner, "ddd_jungui_discarded", disc_len);
        }
        // dddshilie 示烈 (`3D精选/character/skill.js:511-654`):
        // Requires `!dddshilie_used` and non-empty hand (`skill.js:613-623`). Shows hand cards
        // whose rank sum >= `player.hp + current.hp` to view-as `sha`/`shan` (cards stay in hand);
        // only draws `shown_count` cards if rank sum == `player.hp + current.hp` (`dddshilie_exact_sum != 0`).
        "dddshilie" => {
            if flag(ctx.core, owner, "dddshilie_used") != 0 {
                set_flag(ctx.core, owner, "dddshilie_blocked_used", 1);
                return;
            }
            if ctx.core.player(owner).hand.is_empty() {
                set_flag(ctx.core, owner, "dddshilie_blocked_empty", 1);
                return;
            }
            set_flag(ctx.core, owner, "dddshilie_used", 1);
            set_flag(ctx.core, owner, "ddd_shilie_triggered", 1);
            set_flag(ctx.core, owner, "ddd_shilie_viewas_shashan", 1);
            if flag(ctx.core, owner, "dddshilie_exact_sum") != 0 {
                let n = flag(ctx.core, owner, "dddshilie_shown_count").max(1) as u32;
                ctx.start_draw(owner, n);
            }
        }
        // ddddiedang 跌宕 (`3D精选/character/skill.js:802-860`):
        // Conversion skill (`ddddiedang_state` 0/1):
        // - state 0: draws 3 cards then discards 1 `he` card;
        // - state 1: requires >= 3 `he` cards (`skill.js:805`), discards 3 `he` cards then draws 1 card;
        // toggles conversion state if owner has max or min handcards.
        "ddddiedang" => {
            let state = flag(ctx.core, owner, "ddddiedang_state") & 1;
            if state == 0 {
                set_flag(ctx.core, owner, "ddd_diedang_triggered", 1);
                ctx.start_draw(owner, 3);
                let _ = discard_one_he(ctx, owner);
            } else {
                let he_len =
                    ctx.core.player(owner).hand.len() + ctx.core.player(owner).equips.len();
                if he_len < 3 {
                    set_flag(ctx.core, owner, "ddddiedang_blocked_cards", 1);
                    return;
                }
                set_flag(ctx.core, owner, "ddd_diedang_triggered", 1);
                for _ in 0..3 {
                    let _ = discard_one_he(ctx, owner);
                }
                ctx.start_draw(owner, 1);
            }
            set_flag(ctx.core, owner, "ddddiedang_state", state ^ 1);
        }
        other => {
            super::decision34_ext::resolve_u01_ddd_remaining(ctx, other, owner);
        }
    }
}

/// `U05` `杀海拾遗` unified skill & card-skill dispatcher across all subpackages
/// (`swd`, `mtg`, `yunchou`, `zhenfa`, `play/wuxing`, `ow`, `xianjian`, `gujian`, `gwent`, `hearth`, `stone`).
fn resolve_u05_shahai_skill(ctx: &mut Ctx<'_>, executing: &str, fallback: &str, owner: PlayerId) {
    let raw_name = if executing.is_empty() {
        fallback
    } else {
        executing
    };
    let skill_name = raw_name
        .rsplit_once("__ref_")
        .map(|(_, s)| s)
        .unwrap_or(raw_name);
    if super::decision34_u05_swd_chars::try_resolve_u05_swd_chars(ctx, skill_name, owner) {
        return;
    }
    if super::decision34_u05_swd_cards::try_resolve_u05_swd_cards_and_tactics(ctx, skill_name, owner)
    {
        return;
    }
    if super::decision34_u05_ow_xianjian::try_resolve_u05_ow_xianjian(ctx, skill_name, owner) {
        return;
    }
    if super::decision34_u05_gujian_gwent::try_resolve_u05_gujian_gwent(ctx, skill_name, owner) {
        return;
    }
    if super::decision34_u05_hearth::try_resolve_u05_hearth(ctx, skill_name, owner) {
        return;
    }
    set_flag(
        ctx.core,
        owner,
        &format!("u05_unhandled_skill:{skill_name}"),
        1,
    );
}

/// `U07` `wudian` (`jianbi` 坚壁 — `archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js:735-775`):
/// Upstream `filter`: `get.type(event.card) === "trick" && !multitarget && event.targets.length >= 2`.
/// Excludes 1 target (`excluded.add(event.targets[0])`); does NOT draw cards.
fn resolve_wudian_jianbi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "jianbi_targets_len") == 1 {
        set_flag(ctx.core, owner, "jianbi_blocked_single_target", 1);
        return;
    }
    set_flag(ctx.core, owner, "wudian_jianbi_nullified", 1);
    let shield = flag(ctx.core, owner, "wudian_jianbi_shield") + 1;
    set_flag(ctx.core, owner, "wudian_jianbi_shield", shield);
}

/// `U07` `wudian` (`choudu` 筹度 — `archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js:586-615`):
/// `enable: "phaseUse"`, `usable: 1`, `position: "he"`, discards 1 card so `target` uses virtual `diaobingqianjiang`.
fn resolve_wudian_choudu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.player(owner).hand.is_empty() && ctx.core.player(owner).equips.is_empty() {
        set_flag(ctx.core, owner, "choudu_blocked_empty", 1);
        return;
    }
    if flag(ctx.core, owner, "choudu_used_this_turn") == 1 {
        set_flag(ctx.core, owner, "choudu_blocked_usable", 1);
        return;
    }
    let _ = discard_one_he(ctx, owner);
    set_flag(ctx.core, owner, "choudu_used_this_turn", 1);
    set_flag(ctx.core, owner, "wudian_choudu_dispatched", 1);
    let raw_t = flag(ctx.core, owner, "choudu_target") as usize;
    let target = if raw_t > 0 && raw_t <= ctx.core.players.len() {
        (raw_t - 1) as PlayerId
    } else {
        owner
    };
    super::decision34::resolve_diaobingqianjiang(ctx, target);
}

/// `L08` (`十周年UI` — `archive/upstream/noname-liuli/noname/extension/十周年UI/skill.js`):
/// Pure UI/animation/audio hooks (`_usecard`, `_chongzhu`, `_changeEquipExt`, `_discard`, etc.).
/// Does NOT draw cards or mutate game rules.
fn resolve_l08_decade_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "_usecard"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l08_skill:{skill_name}"), 1);
    let seq = flag(ctx.core, owner, "l08_decade_banner_seq") + 1;
    set_flag(ctx.core, owner, "l08_decade_banner_seq", seq);
    if matches!(skill_name, "_chongzhu" | "_changeEquipExt" | "_usecard") {
        let rc = flag(ctx.core, owner, "l08_recast_count") + 1;
        set_flag(ctx.core, owner, "l08_recast_count", rc);
    }
}

/// `L14` (`手杀ui` — `archive/upstream/noname-liuli/noname/extension/手杀ui/skill/main1.js` & `lbtn/main1.js`):
/// UI hooks (`_dieNoTuoguan`, `_menuCheat`, `_jindutiao`, `_chupaiA`, etc.) record UI hook
/// triggers without drawing cards or mutating combat state.
fn resolve_l14_shousha_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "_dieNoTuoguan"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l14_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l14_shousha_trigger_count") + 1;
    set_flag(ctx.core, owner, "l14_shousha_trigger_count", count);
}

/// `L19` (`末日乱斗` — `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js`):
fn resolve_l19_doomsday_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "baozou"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l19_skill:{skill_name}"), 1);
    match skill_name {
        // baozou 暴走 (`extension.js:333-347`): mod.cardUsable(sha) -> Infinity (does NOT draw cards)
        "baozou" => {
            set_flag(ctx.core, owner, "l19_baozou_active", 1);
            set_flag(ctx.core, owner, "l19_baozou_sha_unlimited", 1);
        }
        // shishi 嗜尸 (`extension.js:348-355`): dieAfter -> gainMaxHp(1) + recover()
        "shishi" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.recover(owner, 1, None);
            set_flag(ctx.core, owner, "l19_shishi_gained_maxhp", 1);
        }
        // xunmeng 迅猛 (`extension.js:356-366`): sha damageBegin -> trigger.num++ & if(hp>1) loseHp()
        "xunmeng" => {
            set_flag(ctx.core, owner, "l19_xunmeng_damage_bonus", 1);
            if ctx.core.player(owner).hp > 1 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            }
        }
        // ganran / ganran2 感染 (`extension.js:378-383`):
        "ganran" | "ganran2" => {
            let inf = flag(ctx.core, owner, "doomsday_infection") + 1;
            set_flag(ctx.core, owner, "doomsday_infection", inf);
            set_flag(ctx.core, owner, "l19_ganran_spread", 1);
            if inf >= 3 {
                set_flag(ctx.core, owner, "doomsday_mutated", 1);
            }
        }
        // _jisuangailv (`extension.js:483-496`): updates infection probability after each phase.
        "_jisuangailv" => {
            let rate = (flag(ctx.core, owner, "l19_infection_prob_pct") + 15).min(100);
            set_flag(ctx.core, owner, "l19_infection_prob_pct", rate);
        }
        // _tuizhi 退治 (`extension.js:497-513`): advances lord retreat counter; reaching 8 achieves human victory.
        "_tuizhi" => {
            let turns = flag(ctx.core, owner, "l19_tuizhi_turns") + 1;
            set_flag(ctx.core, owner, "l19_tuizhi_turns", turns);
            if turns >= 8 {
                set_flag(ctx.core, owner, "l19_tuizhi_victory", 1);
            }
        }
        other => {
            super::decision34_u08_l19_l20_l27::resolve_l19_doomsday_remaining(ctx, other, owner);
        }
    }
}

/// `L20` (`标记补充` — `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`):
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
            if let Some(t) = hostile_target(ctx, owner) {
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
        }
        "diaohulishan" => {
            set_flag(ctx.core, owner, "l20_diaohulishan_exiled", 1);
        }
        other => {
            super::decision34_u08_l19_l20_l27::resolve_l20_mark_remaining(ctx, other, owner);
        }
    }
}

/// `L24` (`皮肤切换` — `archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js`):
/// Pure skin/animation parameter hooks (`_tsx`, `_gj`, `_ts`, `_pfqh_*`). Does NOT draw cards.
fn resolve_l24_skin_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "_tsx"
    } else {
        executing
    };
    set_flag(ctx.core, owner, &format!("l24_skill:{skill_name}"), 1);
    let next_slot = (flag(ctx.core, owner, "l24_skin_slot") % 4) + 1;
    set_flag(ctx.core, owner, "l24_skin_slot", next_slot);
    set_flag(ctx.core, owner, "l24_skin_anim_state", 1);
}

/// `L25` (`祖安武将` — `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`):
fn resolve_l25_zuan_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "g_du"
    } else {
        executing
    };
    if super::decision34_l25_rules::try_resolve_l25_rule_skill(ctx, skill_name, owner) {
        return;
    }
    let _ = super::decision34_l25_ssr::try_resolve_l25_ssr_skill(ctx, skill_name, owner);
}

/// `L27` (`选将美化` — `archive/upstream/noname-liuli/noname/extension/选将美化/extension.js`):
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
        // qimou 奇谋 (`选将美化/extension.js:5902-6026`): limited skill, loses X HP for -X distance & +X sha uses (NO card draw; only reqimou at 6059 draws X).
        if flag(ctx.core, owner, "l27_qimou_used") == 1 {
            set_flag(ctx.core, owner, "l27_qimou_blocked", 1);
            return;
        }
        let x = flag(ctx.core, owner, "l27_qimou_x").clamp(1, 3);
        set_flag(ctx.core, owner, "l27_qimou_used", 1);
        set_flag(ctx.core, owner, "l27_skill:qimou", 1);
        set_flag(ctx.core, owner, "l27_qimou_range_bonus", x);
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: x,
        });
        return;
    }
    set_flag(ctx.core, owner, &format!("l27_skill:{skill_name}"), 1);
    let count = flag(ctx.core, owner, "l27_select_trigger_count") + 1;
    set_flag(ctx.core, owner, "l27_select_trigger_count", count);
    if skill_name == "baonvezhi" {
        // baonvezhi 暴虐值 (`extension.js:1817`):
        set_flag(ctx.core, owner, "l27_baonvezhi_resolved", 1);
        let marks = (flag(ctx.core, owner, "l27_baonue_marks") + 1).min(5);
        set_flag(ctx.core, owner, "l27_baonue_marks", marks);
    } else {
        super::decision34_u08_l19_l20_l27::resolve_l27_select_remaining(ctx, skill_name, owner);
    }
}
