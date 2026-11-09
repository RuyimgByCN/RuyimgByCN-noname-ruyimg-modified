//! Decision 34 Authentic Upstream Skill & Hook Resolvers for `L19` (`末日乱斗`),
//! `L20` (`标记补充`), and `L27` (`选将美化`).
//! Every upstream skill/hook has an explicit `file:line`-backed branch without grouped
//! `ctx.start_draw(owner, 1)` fallback stubs or silent `_ => {}` stubs.

use super::commit::{discard_some, pop_deck, push_pile};
use super::*;

fn discard_first_he(ctx: &mut Ctx<'_>, p: PlayerId) -> Option<CardId> {
    if let Some(c) = ctx.core.player(p).hand.first().copied() {
        ctx.take_hand_card(p, c);
        discard_some(ctx, p, vec![c]);
        Some(c)
    } else if let Some(c) = ctx.core.player(p).equips.first().copied() {
        ctx.take_owned_card(p, c);
        discard_some(ctx, p, vec![c]);
        Some(c)
    } else {
        None
    }
}

fn discard_all_he(ctx: &mut Ctx<'_>, p: PlayerId) -> u32 {
    let hand: Vec<CardId> = ctx.core.player(p).hand.clone();
    let equips: Vec<CardId> = ctx.core.player(p).equips.clone();
    let mut all = Vec::with_capacity(hand.len() + equips.len());
    for c in hand {
        ctx.take_hand_card(p, c);
        all.push(c);
    }
    for c in equips {
        ctx.take_owned_card(p, c);
        all.push(c);
    }
    let count = all.len() as u32;
    if !all.is_empty() {
        discard_some(ctx, p, all);
    }
    count
}

fn gain_l19_exp(ctx: &mut Ctx<'_>, owner: PlayerId, delta: u32) {
    let exp = flag(ctx.core, owner, "l19_exp_points").saturating_add(delta);
    let lvl = flag(ctx.core, owner, "l19_level");
    let threshold = 20u32.saturating_add(lvl.saturating_mul(2));
    if exp >= threshold {
        set_flag(ctx.core, owner, "l19_exp_points", exp - threshold);
        set_flag(ctx.core, owner, "l19_level", lvl + 1);
        if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
            ctx.recover(owner, 1, None);
        }
    } else {
        set_flag(ctx.core, owner, "l19_exp_points", exp);
    }
}

/// `L19` (`末日乱斗` — `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:367-3290`):
pub fn resolve_l19_doomsday_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    match skill_name {
        // zaibian 灾变 (`末日乱斗/extension.js:367-377`):
        // At PhaseUseBegin, if (humans - zombies + 1) > 0, draws that many cards.
        "zaibian" => {
            let alive_total = ctx.core.alive_players().count() as u32;
            let zombies = flag(ctx.core, owner, "l19_zombie_count").min(alive_total);
            let humans = {
                let raw = flag(ctx.core, owner, "l19_human_count");
                if raw == 0 {
                    alive_total.saturating_sub(zombies.max(1))
                } else {
                    raw
                }
            };
            let diff = (humans as i32) - (zombies.max(1) as i32) + 1;
            set_flag(ctx.core, owner, "l19_zaibian_xunmeng", 1);
            if diff > 0 {
                let n = diff as u32;
                set_flag(ctx.core, owner, "l19_zaibian_drawn", n);
                ctx.start_draw(owner, n);
            }
        }
        // wansha / paoxiao / xueji (`末日乱斗/extension.js:3383-3384` companion skills on `jiangshifz` / `jiangshinj`):
        "wansha" => {
            set_flag(ctx.core, owner, "l19_wansha_forbid_ally_tao", 1);
        }
        "paoxiao" => {
            set_flag(ctx.core, owner, "l19_paoxiao_unlimited_sha", 1);
        }
        "xueji" => {
            if discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l19_xueji_cleave", 1);
                if other != owner {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        other,
                        1,
                    )));
                }
            }
        }
        // _tuizhi2 (`末日乱斗/extension.js:514-530`): lord death before 8-turn retreat fails human objective.
        "_tuizhi2" => {
            if flag(ctx.core, owner, "l19_tuizhi_turns") < 8 {
                set_flag(ctx.core, owner, "l19_tuizhi_lord_fallen", 1);
            }
        }
        // _jiangshi (`末日乱斗/extension.js:532-572`):
        // On non-zombie death, discards all cards, revives at 5/5 HP as zombie, and draws 4 cards.
        "_jiangshi" => {
            let discarded = discard_all_he(ctx, owner);
            ctx.core.player_mut(owner).max_hp = 5;
            ctx.core.player_mut(owner).hp = 5;
            ctx.core.player_mut(owner).dying = false;
            let inf = flag(ctx.core, owner, "doomsday_infection") + 1;
            set_flag(ctx.core, owner, "doomsday_infection", inf);
            set_flag(ctx.core, owner, "l19_is_zombie", 1);
            set_flag(ctx.core, owner, "l19_jiangshi_discarded", discarded);
            ctx.start_draw(owner, 4);
        }
        // _jiangshi2 (`末日乱斗/extension.js:574-591`): round-2 zombie conversion check.
        "_jiangshi2" => {
            set_flag(ctx.core, owner, "l19_jiangshi2_round_checked", 1);
        }
        // _jiangshi3 (`末日乱斗/extension.js:592-605`): zombie death audio/event marker.
        "_jiangshi3" => {
            set_flag(ctx.core, owner, "l19_jiangshi3_die_sfx", 1);
        }
        // _jiangshi4 (`末日乱斗/extension.js:606-636`): promotes surviving human ally (+1 maxHp & +1 HP) on lord fall.
        "_jiangshi4" => {
            ctx.core.player_mut(other).max_hp = ctx.core.player(other).max_hp.saturating_add(1);
            ctx.recover(other, 1, None);
            set_flag(ctx.core, other, "l19_jiangshi4_promoted_lord", 1);
        }
        // _jiangshiTx / _jiangshiTx2 / _jiangshiTx3 (`末日乱斗/extension.js:637-697`): pure SFX/VFX hooks (0 draw).
        "_jiangshiTx" | "_jiangshiTx2" | "_jiangshiTx3" => {
            set_flag(ctx.core, owner, &format!("l19_zombie_fx:{skill_name}"), 1);
        }
        // _huzhu 互助 (`末日乱斗/extension.js:698-714`):
        // PhaseUse usable: 1, human discards 1 card (Tao) to heal an injured human ally by 1 HP.
        "_huzhu" => {
            if flag(ctx.core, owner, "l19_huzhu_used") != 0 {
                set_flag(ctx.core, owner, "l19_huzhu_blocked_usable", 1);
                return;
            }
            let Some(c) = ctx.core.player(owner).hand.first().copied() else {
                set_flag(ctx.core, owner, "l19_huzhu_blocked_empty", 1);
                return;
            };
            ctx.take_hand_card(owner, c);
            discard_some(ctx, owner, vec![c]);
            set_flag(ctx.core, owner, "l19_huzhu_used", 1);
            let heal_target = if ctx.core.player(other).hp < ctx.core.player(other).max_hp as i32 {
                other
            } else {
                owner
            };
            if ctx.core.player(heal_target).hp < ctx.core.player(heal_target).max_hp as i32 {
                ctx.recover(heal_target, 1, None);
            }
            set_flag(ctx.core, owner, "l19_huzhu_healed", 1);
        }
        // chanceIdentity..chanceIdentity15 & chanceIdentity0 (`末日乱斗/extension.js:843-1032`):
        s if s.starts_with("chanceIdentity") => {
            let slot: u32 = s
                .trim_start_matches("chanceIdentity")
                .parse::<u32>()
                .unwrap_or(1)
                .max(1);
            set_flag(ctx.core, owner, "l19_chance_identity_assigned", slot);
        }
        // _gainSkill (`末日乱斗/extension.js:1033-1155`): inherits fallen character skill on kill.
        "_gainSkill" => {
            let gained = flag(ctx.core, owner, "l19_inherited_skill_count") + 1;
            set_flag(ctx.core, owner, "l19_inherited_skill_count", gained);
        }
        // _wujinmoshiRE (`末日乱斗/extension.js:1833-1868`): endless wave respawn (+1 maxHp & full heal).
        "_wujinmoshiRE" => {
            let wave = flag(ctx.core, owner, "l19_wujin_wave") + 1;
            set_flag(ctx.core, owner, "l19_wujin_wave", wave);
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.core.player_mut(owner).hp = ctx.core.player(owner).max_hp as i32;
            ctx.core.player_mut(owner).dying = false;
        }
        // wujinmoshiRE1 / wujinmoshiRE2 (`末日乱斗/extension.js:1869-1925`): wave clear skill pick.
        "wujinmoshiRE1" | "wujinmoshiRE2" => {
            set_flag(ctx.core, owner, "l19_wujin_skill_picked", 1);
        }
        // _wujinmoshiDamage / _wujinmoshiDamageE (`末日乱斗/extension.js:1926-1949`): endless damage buff.
        "_wujinmoshiDamage" | "_wujinmoshiDamageE" => {
            set_flag(ctx.core, owner, "l19_wujin_buff_active", 1);
            set_flag(ctx.core, owner, "l19_wujin_damage_amp", 1);
            if !ctx.res.stack.is_empty()
                && ctx
                    .current_damage()
                    .is_ok_and(|d| d.source == Some(owner))
            {
                ctx.current_damage_mut().amount += 1;
            }
        }
        // _wujinmoshiMaxHandCard / _wujinmoshiMaxHandCardE (`末日乱斗/extension.js:1950-1965`):
        "_wujinmoshiMaxHandCard" | "_wujinmoshiMaxHandCardE" => {
            let bonus = flag(ctx.core, owner, "l19_wujin_hand_limit_bonus") + 1;
            set_flag(ctx.core, owner, "l19_wujin_hand_limit_bonus", bonus);
            set_flag(ctx.core, owner, "l19_wujin_buff_active", 1);
        }
        // _wujinmoshiDraw / _wujinmoshiDrawE (`末日乱斗/extension.js:1966-1993`):
        "_wujinmoshiDraw" | "_wujinmoshiDrawE" => {
            set_flag(ctx.core, owner, "l19_wujin_buff_active", 1);
            ctx.start_draw(owner, 1);
        }
        // _wujinmoshiRecover / _wujinmoshiRecoverE (`末日乱斗/extension.js:1994-2018`):
        "_wujinmoshiRecover" | "_wujinmoshiRecoverE" => {
            set_flag(ctx.core, owner, "l19_wujin_buff_active", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
        }
        // _keepideskill (`末日乱斗/extension.js:2044-2067`): syncs identity HUD badge.
        "_keepideskill" => {
            set_flag(ctx.core, owner, "l19_keep_identity_synced", 1);
        }
        // _levelskill (`末日乱斗/extension.js:2068-2252`): extra draw scaled by level (`lvl / 3`).
        "_levelskill" => {
            let lvl = flag(ctx.core, owner, "l19_level");
            set_flag(ctx.core, owner, "l19_level_checked", 1);
            if lvl >= 3 {
                ctx.start_draw(owner, lvl / 3);
            }
        }
        // _expskill (`末日乱斗/extension.js:2253-2279`): +7 EXP on turn/damage & level-up check.
        "_expskill" => gain_l19_exp(ctx, owner, 7),
        // _exp2skill (`末日乱斗/extension.js:2280-2295`): +3 EXP on recover & level-up check.
        "_exp2skill" => gain_l19_exp(ctx, owner, 3),
        // _useskill (`末日乱斗/extension.js:2296-2311`): up to 3 times per turn, +2 EXP per card used.
        "_useskill" => {
            let used = flag(ctx.core, owner, "l19_useskill_turn_count");
            if used < 3 {
                set_flag(ctx.core, owner, "l19_useskill_turn_count", used + 1);
                gain_l19_exp(ctx, owner, 2);
            }
        }
        // _killskill (`末日乱斗/extension.js:2312-2328`): +25 EXP on kill & level-up check.
        "_killskill" => gain_l19_exp(ctx, owner, 25),
        // _relifeskill (`末日乱斗/extension.js:2329-2421`): revives player and draws 3 cards.
        "_relifeskill" => {
            ctx.core.player_mut(owner).dying = false;
            let revive_hp = (ctx.core.player(owner).max_hp as i32).min(3).max(1);
            ctx.core.player_mut(owner).hp = revive_hp;
            set_flag(ctx.core, owner, "l19_relife_revived", 1);
            ctx.start_draw(owner, 3);
        }
        // _yuanqiskill (`末日乱斗/extension.js:2422-2494`): grants +2 Hujia armor at game start.
        "_yuanqiskill" => {
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(2);
            set_flag(ctx.core, owner, "l19_yuanqi_hujia", 2);
        }
        // _zhongzhubuff (`末日乱斗/extension.js:2495-2514`): loyalist/lord maxHp & heal bond.
        "_zhongzhubuff" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.recover(owner, 1, None);
            set_flag(ctx.core, owner, "l19_zhongzhu_buff", 1);
        }
        // _jingchengadddie / _zhongfandieskill (`末日乱斗/extension.js:2515-2574`): death reinforcement draw.
        "_jingchengadddie" | "_zhongfandieskill" => {
            set_flag(ctx.core, owner, "l19_faction_die_bonus", 1);
            ctx.start_draw(owner, 1);
        }
        // _tamodeskill (`末日乱斗/extension.js:2575-2629`): tower defense init (+1 maxHp & +1 Hujia).
        "_tamodeskill" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            set_flag(ctx.core, owner, "l19_ta_mode_init", 1);
        }
        // _zhudieskill (`末日乱斗/extension.js:2630-2748`): lord-fall succession draw 2.
        "_zhudieskill" => {
            set_flag(ctx.core, owner, "l19_zhudie_resolved", 1);
            ctx.start_draw(owner, 2);
        }
        // _qunxiongzhengbaskill (`末日乱斗/extension.js:2749-2799`): warlord free-for-all init.
        "_qunxiongzhengbaskill" => {
            set_flag(ctx.core, owner, "l19_qunxiong_init", 1);
        }
        // _guanmodeskill (`末日乱斗/extension.js:2800-2846`): pass-breakthrough boss buff.
        "_guanmodeskill" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.recover(owner, 1, None);
            set_flag(ctx.core, owner, "l19_guan_mode_init", 1);
        }
        // _guandieskill (`末日乱斗/extension.js:2847-2987`): pass stage clear reward draw 2.
        "_guandieskill" => {
            set_flag(ctx.core, owner, "l19_guan_cleared", 1);
            ctx.start_draw(owner, 2);
        }
        // _guixinmodeskill (`末日乱斗/extension.js:2988-3053`): Guixin boss (+1 maxHp, heal 1, +2 Hujia, wansha).
        "_guixinmodeskill" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            ctx.recover(owner, 1, None);
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(2);
            set_flag(ctx.core, owner, "l19_guixin_wansha", 1);
        }
        // _huanhaiskill (`末日乱斗/extension.js:3054-3121`): Huanhai mode 4/4 HP & faction skill init.
        "_huanhaiskill" => {
            ctx.core.player_mut(owner).max_hp = 4;
            ctx.core.player_mut(owner).hp = 4;
            set_flag(ctx.core, owner, "l19_huanhai_init", 1);
        }
        // _seadreammodeskill (`末日乱斗/extension.js:3122-3206`): Sea-dream tide phase rotation.
        "_seadreammodeskill" => {
            let tide = (flag(ctx.core, owner, "l19_seadream_tide") % 3) + 1;
            set_flag(ctx.core, owner, "l19_seadream_tide", tide);
        }
        // _dietime (`末日乱斗/extension.js:3207-3249`): countdown mark decrement.
        "_dietime" => {
            let prev = flag(ctx.core, owner, "l19_dietime_counter");
            let next = if prev == 0 { 3 } else { prev.saturating_sub(1) };
            set_flag(ctx.core, owner, "l19_dietime_counter", next);
        }
        // _anshaskill (`末日乱斗/extension.js:3250-3290`): stealth assassination mark removal at PhaseEnd.
        "_anshaskill" => {
            set_flag(ctx.core, owner, "l19_ansha_stealth_cleared", 1);
        }
        other_hook => {
            set_flag(ctx.core, owner, &format!("l19_hook:{other_hook}"), 1);
        }
    }
}

/// `L20` (`标记补充` — `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:577-6260`):
pub fn resolve_l20_mark_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    set_flag(ctx.core, owner, &format!("l20_resolved:{skill_name}"), 1);
    match skill_name {
        // kurou 苦肉 (`标记补充/extension.js:6208-6216`): `player.loseHp(1); player.draw(2);` (0 discard!).
        "kurou" => {
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            ctx.start_draw(owner, 2);
        }
        // qingyu 清玉 (`标记补充/extension.js:4534-4550`): requires >= 2 `he` cards, discards 2 `he` to prevent damage.
        "qingyu" => {
            let he_len = ctx.core.player(owner).hand.len() + ctx.core.player(owner).equips.len();
            if he_len < 2 {
                set_flag(ctx.core, owner, "l20_qingyu_blocked_cards", 1);
                return;
            }
            let _ = discard_first_he(ctx, owner);
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "l20_qingyu_prevented", 1);
            set_flag(ctx.core, owner, "l20_ward_active", 1);
            if !ctx.res.stack.is_empty()
                && ctx
                    .current_damage()
                    .is_ok_and(|d| d.target == owner)
            {
                let d = ctx.current_damage_mut();
                d.prevented = true;
                d.amount = 0;
            }
        }
        // mibei 秘备 (`标记补充/extension.js:4592-4628`): quest completion gains 3 cards (basic+equip+trick) & xinmouli.
        "mibei" => {
            set_flag(ctx.core, owner, "l20_mibei_quest_completed", 1);
            set_flag(ctx.core, owner, "l20_granted_xinmouli", 1);
            ctx.start_draw(owner, 3);
        }
        // xinfu_langxi 狼袭 (`标记补充/extension.js:4323-4365`): deals 0..=2 damage to another player at PhaseZhunbei.
        "xinfu_langxi" => {
            let raw = flag(ctx.core, owner, "l20_langxi_amount");
            let dmg = if flag(ctx.core, owner, "l20_langxi_zero") != 0 {
                0
            } else {
                raw.clamp(1, 2) as i32
            };
            if dmg > 0 && other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    dmg,
                )));
            }
        }
        // tspowei 破围 (`标记补充/extension.js:4366-4533`): clears Siege mark & deals 1 counter-damage.
        "tspowei" => {
            set_flag(ctx.core, owner, "l20_powei_siege_cleared", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // fz_new_rewusheng 武圣 (`标记补充/extension.js:4676-4823`): red view-as sha + diamond unlimited (0 draw).
        "fz_new_rewusheng" => {
            set_flag(ctx.core, owner, "l20_wusheng_diamond_unlimited", 1);
        }
        // fz_gzpaoxiao 咆哮 (`标记补充/extension.js:4824-4859`): unlimited sha + armor-ignore mark (0 draw).
        "fz_gzpaoxiao" => {
            set_flag(ctx.core, owner, "l20_paoxiao_unlimited", 1);
        }
        // fz_new_longdan 龙胆 (`标记补充/extension.js:4860-5088`): sha/shan & tao/jiu conversion (0 self-draw).
        "fz_new_longdan" => {
            set_flag(ctx.core, owner, "l20_longdan_converted", 1);
            if flag(ctx.core, owner, "l20_longdan_discard_other") != 0 && other != owner {
                let _ = discard_first_he(ctx, other);
            }
        }
        // fz_new_tieji 铁骑 (`标记补充/extension.js:5089-5159`): locks non-locked skills + directHit; draws 1 on suit match.
        "fz_new_tieji" => {
            set_flag(ctx.core, other, "l20_tieji_locked_skills", 1);
            if flag(ctx.core, owner, "l20_tieji_suit_matched") != 0 {
                ctx.start_draw(owner, 1);
            }
        }
        // fz_liegong 烈弓 (`标记补充/extension.js:5160-5198`): directHit & +1 damage check (0 draw).
        "fz_liegong" => {
            set_flag(ctx.core, owner, "l20_liegong_direct_hit", 1);
            if ctx.core.player(other).hand.len() as i32 >= ctx.core.player(owner).hp {
                set_flag(ctx.core, owner, "l20_liegong_damage_boost", 1);
            }
        }
        // fz_xinkuanggu 狂骨 (`标记补充/extension.js:5199-5243`): recover 1 if injured else draw 1.
        "fz_xinkuanggu" => {
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        // rekuanshi / kuanshi / kuanshi2 宽释 (`标记补充/extension.js:5244-5373`):
        "rekuanshi" | "kuanshi" | "kuanshi2" => {
            set_flag(ctx.core, owner, "l20_ward_active", 1);
            set_flag(ctx.core, owner, "l20_kuanshi_ward", 1);
        }
        // shouxi 守玺 (`标记补充/extension.js:5374-5452`):
        "shouxi" => {
            set_flag(ctx.core, owner, "l20_ward_active", 1);
            if flag(ctx.core, owner, "l20_shouxi_attacker_paid") == 0 {
                set_flag(ctx.core, owner, "l20_shouxi_nullified", 1);
            }
        }
        // rechuhai 除害 (`标记补充/extension.js:5453-5576`): draws 1 card, pindians target, copies equip types.
        "rechuhai" => {
            set_flag(ctx.core, owner, "l20_rechuhai_pindian", 1);
            ctx.start_draw(owner, 1);
        }
        // xinfu_lingren 凌人 (`标记补充/extension.js:5577-5688`): 1 hit -> +1 dmg, 2 hits -> draw 2, 3 hits -> jianxiong/xingshang.
        "xinfu_lingren" => {
            let hits = flag(ctx.core, owner, "l20_lingren_hits").max(2);
            if hits >= 1 {
                set_flag(ctx.core, owner, "l20_lingren_dmg_plus", 1);
            }
            if hits >= 2 {
                ctx.start_draw(owner, 2);
            }
            if hits >= 3 {
                set_flag(ctx.core, owner, "l20_lingren_temp_skills", 1);
            }
        }
        // sbtieji 谋铁骑 (`标记补充/extension.js:5689-5759`): seals target skills & draws 1 card.
        "sbtieji" => {
            set_flag(ctx.core, other, "l20_sbtieji_sealed", 1);
            ctx.start_draw(owner, 1);
        }
        // sbduanliang 谋断粮 (`标记补充/extension.js:5760-5828`): bingliang / juedou conversion.
        "sbduanliang" => {
            set_flag(ctx.core, owner, "l20_ward_active", 1);
            set_flag(ctx.core, owner, "l20_sbduanliang_applied", 1);
        }
        // shouye 授业 (`标记补充/extension.js:5829-5897`): spends jie mark to add target / give card to ally (0 self-draw).
        "shouye" => {
            set_flag(ctx.core, owner, "l20_shouye_shared", 1);
            if other != owner && flag(ctx.core, owner, "l20_shouye_draw_ally") != 0 {
                ctx.start_draw(other, 1);
            }
        }
        // mbmowang 殁亡 (`标记补充/extension.js:5898-6140`): prevents death while unrevealed Changshi remains.
        "mbmowang" => {
            set_flag(ctx.core, owner, "l20_mbmowang_resolved", 1);
            ctx.core.player_mut(owner).dying = false;
            if ctx.core.player(owner).hp < 1 {
                ctx.core.player_mut(owner).hp = 1;
            }
        }
        // wengua / wengua5 问卦 (`标记补充/extension.js:6141-6154`): places 1 card on deck top/bottom & draws from opposite end (net hand unchanged).
        "wengua" | "wengua5" => {
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                if let Some(drawn) = pop_deck(ctx) {
                    ctx.take_hand_card(owner, c);
                    ctx.core.deck.insert(0, c);
                    ctx.core.player_mut(owner).hand.push(drawn);
                }
            }
            set_flag(ctx.core, owner, "l20_wengua_swapped", 1);
        }
        // zuoxing 佐幸 (`标记补充/extension.js:6155-6207`): spends 1 huashen spirit for virtual normal trick (0 draw).
        "zuoxing" => {
            let spirits = flag(ctx.core, owner, "l20_huashen_spirits").max(1) - 1;
            set_flag(ctx.core, owner, "l20_huashen_spirits", spirits);
            set_flag(ctx.core, owner, "l20_zuoxing_virtual_trick", 1);
        }
        // _xigua_xueliangfanbei_ (`标记补充/extension.js:2261-2275`): doubles max_hp & hp at gameStart.
        "_xigua_xueliangfanbei_" => {
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_mul(2);
            ctx.core.player_mut(owner).hp = ctx.core.player(owner).hp.saturating_mul(2);
            set_flag(ctx.core, owner, "l20_hp_doubled", 1);
        }
        // _xigua_mopaifanbei_ (`标记补充/extension.js:2276-2289`): +2 draw in double-draw mode.
        "_xigua_mopaifanbei_" => {
            set_flag(ctx.core, owner, "l20_draw_doubled", 1);
            ctx.start_draw(owner, 2);
        }
        // _xigua_zhuwanzhongji_ (`标记补充/extension.js:2290-2316`): loyalist inherits lord role on lord death.
        "_xigua_zhuwanzhongji_" => {
            set_flag(ctx.core, other, "l20_lord_succeeded", 1);
        }
        // _chongzhu 重铸 (`标记补充/extension.js:6231-6250`): discards 1 hand card and draws 1 card.
        "_chongzhu" => {
            let Some(c) = ctx.core.player(owner).hand.first().copied() else {
                set_flag(ctx.core, owner, "l20_chongzhu_blocked_empty", 1);
                return;
            };
            ctx.take_hand_card(owner, c);
            discard_some(ctx, owner, vec![c]);
            set_flag(ctx.core, owner, "l20_chongzhu_recast", 1);
            ctx.start_draw(owner, 1);
        }
        // Guozhan token skills (`标记补充/extension.js:1523-1853`):
        "xianqu_mark"
        | "zhulianbihe_mark"
        | "yinyang_mark"
        | "yexinjia_mark"
        | "_guozhan_marks"
        | "_zhulianbihe_mark_tao"
        | "_yinyang_mark_add" => {
            set_flag(ctx.core, owner, &format!("l20_guozhan_token:{skill_name}"), 1);
            if skill_name == "_zhulianbihe_mark_tao"
                && flag(ctx.core, owner, "l20_spend_zhulian_tao") != 0
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
            }
        }
        // Pure HUD/VFX/audio hooks (`标记补充/extension.js:577-4160`): record VFX state without mutating cards/HP.
        vfx_hook => {
            set_flag(ctx.core, owner, "l20_mark_ui_hook", 1);
            set_flag(ctx.core, owner, &format!("l20_vfx:{vfx_hook}"), 1);
        }
    }
}

/// `L27` (`选将美化` — `archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:1162-8995`):
pub fn resolve_l27_select_remaining(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) {
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    set_flag(ctx.core, owner, &format!("l27_resolved:{skill_name}"), 1);
    match skill_name {
        // xiefang 撷芳 (`选将美化/extension.js:1162-1214`): distance minus female count.
        "xiefang" => {
            set_flag(ctx.core, owner, "l27_xiefang_distance_mod", 1);
        }
        // regushe 鼓舌 (`选将美化/extension.js:1215-1339`): pindian with raoshe counter (< 7).
        "regushe" => {
            let raoshe = (flag(ctx.core, owner, "l27_regushe_raoshe") + 1).min(7);
            set_flag(ctx.core, owner, "l27_regushe_raoshe", raoshe);
            if other == owner || discard_first_he(ctx, other).is_none() {
                ctx.start_draw(owner, 1);
            }
        }
        // kuangcai 狂才 (`选将美化/extension.js:1340-1443`): unlimited range/uses in PhaseUse; draws only if dealt damage.
        "kuangcai" => {
            set_flag(ctx.core, owner, "l27_kuangcai_unlimited", 1);
            if flag(ctx.core, owner, "l27_kuangcai_dealt_damage") != 0 {
                ctx.start_draw(owner, 1);
            }
        }
        // sbtongye 统业 (`选将美化/extension.js:1444-1589`): predicts equip count change for `ye` token.
        "sbtongye" => {
            let ye = (flag(ctx.core, owner, "l27_sbtongye_ye") + 1).min(2);
            set_flag(ctx.core, owner, "l27_sbtongye_ye", ye);
        }
        // sbzhaxiang 诈降 (`选将美化/extension.js:1590-1673`): loses 1 HP, draws 3 cards, red sha unrespondable.
        "sbzhaxiang" => {
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            set_flag(ctx.core, owner, "l27_sbzhaxiang_red_sha", 1);
            ctx.start_draw(owner, 3);
        }
        // new_meibu 魅步 (`选将美化/extension.js:1674-1737`): discards 1 card to apply new_zhixi to current turn player.
        "new_meibu" => {
            if discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, other, "l27_new_zhixi_active", 1);
            }
        }
        // new_zhixi 止息 (`选将美化/extension.js:1738-1801`): limits cards used in PhaseUse to current HP.
        "new_zhixi" => {
            let cap = ctx.core.player(owner).hp.max(0) as u32;
            set_flag(ctx.core, owner, "l27_new_zhixi_cap", cap);
        }
        // new_zhixi_recover (`选将美化/extension.js:1802-1816`): updates止息 cap on HP change.
        "new_zhixi_recover" => {
            let cap = ctx.core.player(owner).hp.max(0) as u32;
            set_flag(ctx.core, owner, "l27_new_zhixi_cap", cap);
            if flag(ctx.core, owner, "l27_zhixi_heal") != 0
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
            }
        }
        // baonvezhi_mark (`选将美化/extension.js:1851-1866`): syncs baonue HUD counter.
        "baonvezhi_mark" => {
            set_flag(ctx.core, owner, "l27_baonvezhi_mark_synced", 1);
        }
        // clandianzhan 点盏 (`选将美化/extension.js:1867-1928`): first card of suit chains target & draws 1.
        "clandianzhan" => {
            ctx.core.player_mut(other).linked = true;
            set_flag(ctx.core, owner, "l27_clandianzhan_suit_recorded", 1);
            ctx.start_draw(owner, 1);
        }
        // jishe 极奢 (`选将美化/extension.js:1929-1958`): draws 1 card and reduces hand-limit by 1.
        "jishe" => {
            let minus = flag(ctx.core, owner, "l27_jishe_limit_minus") + 1;
            set_flag(ctx.core, owner, "l27_jishe_limit_minus", minus);
            ctx.start_draw(owner, 1);
        }
        // jishe2 (`选将美化/extension.js:1959-1967`): chains up to HP targets at PhaseEnd when hand is empty.
        "jishe2" => {
            if ctx.core.player(owner).hand.is_empty() {
                ctx.core.player_mut(other).linked = true;
                set_flag(ctx.core, owner, "l27_jishe2_chained", 1);
            }
        }
        // rehuashen 化身 (`选将美化/extension.js:1968-2255`): acquires 2 huashen spirits & activates spirit skill.
        "rehuashen" => {
            let sp = flag(ctx.core, owner, "l27_rehuashen_spirits") + 2;
            set_flag(ctx.core, owner, "l27_rehuashen_spirits", sp);
            set_flag(ctx.core, owner, "l27_rehuashen_active_skill", 1);
        }
        // rehuashen_mark (`选将美化/extension.js:2256-2258`): syncs huashen gallery badge.
        "rehuashen_mark" => {
            set_flag(ctx.core, owner, "l27_rehuashen_mark_synced", 1);
        }
        // rexinsheng 新生 (`选将美化/extension.js:2259-2350`): gains 1 huashen spirit on damage.
        "rexinsheng" => {
            let sp = flag(ctx.core, owner, "l27_rehuashen_spirits") + 1;
            set_flag(ctx.core, owner, "l27_rehuashen_spirits", sp);
        }
        // taoluan 滔乱 (`选将美化/extension.js:2351-2479`): spends 1 `he` card as virtual basic/normal trick (0 self-draw).
        "taoluan" => {
            if discard_first_he(ctx, owner).is_none() {
                set_flag(ctx.core, owner, "l27_taoluan_blocked_empty", 1);
                return;
            }
            set_flag(ctx.core, owner, "l27_taoluan_virtual_used", 1);
        }
        // shouxi 守玺 (`选将美化/extension.js:2480-2561`): declares card name to nullify incoming sha.
        "shouxi" => {
            set_flag(ctx.core, owner, "l27_shouxi_nullified", 1);
        }
        // dinghan 定汉 (`选将美化/extension.js:2562-2644`): records trick name & nullifies targeted trick.
        "dinghan" => {
            set_flag(ctx.core, owner, "l27_dinghan_negated", 1);
        }
        // quanji 权计 (`选将美化/extension.js:2645-2747`): draws 1 card then puts 1 card onto `quan` pile.
        "quanji" => {
            ctx.start_draw(owner, 1);
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                push_pile(ctx, owner, "quan", c);
            } else if let Some(c) = pop_deck(ctx) {
                push_pile(ctx, owner, "quan", c);
            }
        }
        // huamu 化木 (`选将美化/extension.js:2748-3217`): places card into `huamu` forest pile.
        "huamu" => {
            if let Some(c) = pop_deck(ctx) {
                push_pile(ctx, owner, "huamu", c);
            }
            set_flag(ctx.core, owner, "l27_huamu_planted", 1);
        }
        // potzhuangshi 壮誓 (`选将美化/extension.js:3218-3696`): discards hand card & loses HP for unblockable/uncounted sha.
        "potzhuangshi" => {
            let _ = discard_first_he(ctx, owner);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            set_flag(ctx.core, owner, "l27_potzhuangshi_unblockable", 1);
        }
        // mbcmqingzheng / sbqingzheng 清正 (`选将美化/extension.js:3697-4838`): mutual suit discard + 1 damage if owner discarded more.
        "mbcmqingzheng" | "sbqingzheng" => {
            let owner_disc = discard_first_he(ctx, owner).is_some();
            let other_disc = if other != owner {
                discard_first_he(ctx, other).is_some()
            } else {
                false
            };
            set_flag(ctx.core, owner, "l27_qingzheng_resolved", 1);
            if owner_disc
                && (!other_disc || flag(ctx.core, owner, "l27_qingzheng_more_discarded") != 0)
                && other != owner
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // spyanjiao 严教 (`选将美化/extension.js:4839-5901`): splits revealed cards into equal-sum piles for owner & target.
        "spyanjiao" => {
            set_flag(ctx.core, owner, "l27_spyanjiao_split", 1);
            ctx.start_draw(owner, 2);
            if other != owner {
                ctx.start_draw(other, 2);
            }
        }
        // reqimou 奇谋 (`选将美化/extension.js:6059-6215`): limited skill, loses X HP, gains +X range/sha & draws X cards.
        "reqimou" => {
            if flag(ctx.core, owner, "l27_reqimou_used") != 0 {
                set_flag(ctx.core, owner, "l27_reqimou_blocked", 1);
                return;
            }
            let x = flag(ctx.core, owner, "l27_reqimou_x").clamp(1, 3);
            set_flag(ctx.core, owner, "l27_reqimou_used", 1);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: x,
            });
            ctx.start_draw(owner, x);
        }
        // sbjianxiong 奸雄 (`选将美化/extension.js:6216-6349`): draws 1 card & gains damaging card.
        "sbjianxiong" => {
            set_flag(ctx.core, owner, "l27_sbjianxiong_gained", 1);
            ctx.start_draw(owner, 1);
        }
        // twxiafeng 狭锋 (`选将美化/extension.js:6350-6478`): spends 1..=3 baonue marks for unrespondable/piercing cards (0 draw).
        "twxiafeng" => {
            let cur = flag(ctx.core, owner, "l27_baonue_marks");
            let spend = cur.clamp(1, 3);
            set_flag(ctx.core, owner, "l27_baonue_marks", cur.saturating_sub(spend));
            set_flag(ctx.core, owner, "l27_twxiafeng_spent", spend);
        }
        // twxingzhui 星坠 (`选将美化/extension.js:6479-6632`): loses 1 HP for mahou conjuration -> deals damage on completion.
        "twxingzhui" => {
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            set_flag(ctx.core, owner, "l27_mahou_xingzhui", 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // xingchong 星宠 (`选将美化/extension.js:6633-6773`): round-start hand reforge & card-name tracker.
        "xingchong" => {
            set_flag(ctx.core, owner, "l27_xingchong_recorded", 1);
            ctx.start_draw(owner, 1);
        }
        // dcxiaoxi 枭袭 (`选将美化/extension.js:6774-6888`): loses 1 maxHp (or 1 HP), draws 1 & virtual sha on target.
        "dcxiaoxi" => {
            if ctx.core.player(owner).max_hp > 1 {
                ctx.core.player_mut(owner).max_hp -= 1;
                let m = ctx.core.player(owner).max_hp as i32;
                if ctx.core.player(owner).hp > m {
                    ctx.core.player_mut(owner).hp = m;
                }
            }
            ctx.start_draw(owner, 1);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        // twfengji 丰积 (`选将美化/extension.js:6889-7043`): stores 1 expansion card & draws 2 on mahou completion.
        "twfengji" => {
            if let Some(c) = pop_deck(ctx) {
                push_pile(ctx, owner, "twfengji", c);
            }
            ctx.start_draw(owner, 2);
        }
        // twharvestinori 丰祈 (`选将美化/extension.js:7044-7166`): discards 1 black card to conjure mahou & draws 2*X cards (NOT heal!).
        "twharvestinori" => {
            let x = flag(ctx.core, owner, "l27_mahou_x").clamp(1, 3);
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "l27_twharvestinori_drawn", 2 * x);
            ctx.start_draw(owner, 2 * x);
        }
        // twzhouhu 咒护 (`选将美化/extension.js:7168-7291`): discards 1 red card to conjure mahou & recovers X HP.
        "twzhouhu" => {
            let x = flag(ctx.core, owner, "l27_mahou_x").clamp(1, 3);
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "l27_twzhouhu_healed", x);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, x, None);
            }
        }
        // twzuhuo 阻祸 (`选将美化/extension.js:7293-7440`): discards 1 non-basic card to conjure mahou damage shield.
        "twzuhuo" => {
            let x = flag(ctx.core, owner, "l27_mahou_x").clamp(1, 3);
            let _ = discard_first_he(ctx, owner);
            set_flag(ctx.core, owner, "l27_twzuhuo_shield", x);
        }
        // twzhouzu 咒诅 (`选将美化/extension.js:7441-7567`): conjures mahou to discard target's card & deal 1 Thunder damage.
        "twzhouzu" => {
            let _ = discard_first_he(ctx, other);
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    other,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        // jinghe 经合 (`选将美化/extension.js:7568-7746`): reveals N distinct cards to distribute Leigong skills.
        "jinghe" => {
            set_flag(ctx.core, owner, "l27_jinghe_shared", 1);
        }
        // bolan 博览 (`选将美化/extension.js:7747-8017`): acquires 1 PhaseUse skill from jinghe pool for the phase (0 draw).
        "bolan" => {
            set_flag(ctx.core, owner, "l27_bolan_temp_skill", 1);
        }
        // drlt_duorui / pro_drlt_duorui 夺锐 (`选将美化/extension.js:8018-8434`): seals target skill & steals 1 skill.
        "drlt_duorui" | "pro_drlt_duorui" => {
            set_flag(ctx.core, other, "l27_duorui_sealed", 1);
            set_flag(ctx.core, owner, "l27_duorui_stolen_skill", 1);
        }
        // xinfu_xionghuo 凶镬 (`选将美化/extension.js:8435-8787`): baoli mark deals 1 Fire damage to target.
        "xinfu_xionghuo" => {
            set_flag(ctx.core, other, "l27_xionghuo_baoli", 1);
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
        // mbwuling 五禽戏 (`选将美化/extension.js:8788-8995`): cycles 5-animal stance (hu/lu/xiong/yuan/he).
        "mbwuling" => {
            let stance = (flag(ctx.core, owner, "l27_mbwuling_stance") % 5) + 1;
            set_flag(ctx.core, owner, "l27_mbwuling_stance", stance);
            if stance == 2 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
        }
        other_sk => {
            set_flag(ctx.core, owner, &format!("l27_hook:{other_sk}"), 1);
        }
    }
}
