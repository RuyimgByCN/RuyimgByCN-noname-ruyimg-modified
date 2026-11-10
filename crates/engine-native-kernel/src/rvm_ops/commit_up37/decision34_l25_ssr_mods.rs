//! Authentic upstream resolvers for the 43 `L25` (`祖安武将`) modifier, conversion,
//! and tactical `ssr_*` skills (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`).

use super::commit::{discard_some, pop_deck};
use super::decision34_l25_ssr::discard_first_he;
use super::*;
use crate::catalog::Suit;
use crate::event::DamageNature;

fn store_deck_to_pile(ctx: &mut Ctx<'_>, owner: PlayerId, pile_name: &str) -> bool {
    if let Some(c) = pop_deck(ctx) {
        ctx.core
            .vm_aux
            .piles
            .entry((owner, pile_name.to_string()))
            .or_default()
            .push(c);
        true
    } else {
        false
    }
}

/// Resolves the 43 passive/conversion/tactical `ssr_*` skills in `L25` (`祖安武将/extension.js`).
pub(super) fn try_resolve_l25_ssr_mod_or_tactical(
    ctx: &mut Ctx<'_>,
    skill_name: &str,
    owner: PlayerId,
    other: PlayerId,
) -> bool {
    match skill_name {
        "ssr_jianye" => {
            // `extension.js:5235`: card gain/discard balance trigger -> draws 1 card
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_jianye_active", 1);
        }
        "ssr_newlongdan" => {
            // `extension.js:5586`: Sha <-> Shan conversion (`d34_flags` + tactical draw on activation)
            set_flag(ctx.core, owner, "l25_longdan_viewas", 1);
            ctx.start_draw(owner, 1);
        }
        "ssr_jinnang" => {
            // `extension.js:20784`: `usable: 2`, discard 1 card to draw 2
            let used = flag(ctx.core, owner, "l25_jinnang_used");
            if used < 2 && discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l25_jinnang_used", used + 1);
                ctx.start_draw(owner, 2);
            }
        }
        "ssr_newbazhen" => {
            // `extension.js:21623`: `phaseZhunbeiBegin` Eight Trigrams judgment (red -> Shan guard & draw 1)
            if let Some(jc) = pop_deck(ctx) {
                let is_red = matches!(
                    ctx.catalog.cards[jc as usize].suit,
                    Suit::Heart | Suit::Diamond
                );
                discard_some(ctx, owner, vec![jc]);
                if is_red {
                    set_flag(ctx.core, owner, "l25_bazhen_shan_guard", 1);
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "ssr_shichou" => {
            // `extension.js:20615`: transfers Shichou curse onto `other` and draws 1
            if other != owner {
                set_flag(ctx.core, other, "l25_shichou_cursed", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "ssr_jijiang" => {
            // `extension.js:7619`: +1 Sha usable limit (`d34_flags`) + lord rally draw
            set_flag(ctx.core, owner, "l25_jijiang_sha_plus1", 1);
            ctx.start_draw(owner, 1);
        }
        "ssr_zhujiu" => {
            // `extension.js:8779`: `usable: 1`, activates Jiu boost & Jiu view-as
            ctx.core.turn.jiu_active = true;
            set_flag(ctx.core, owner, "l25_viewas_jiu", 1);
        }
        "ssr_wushuang" => {
            // `extension.js:8998`: forced Sha/Juedou unrespondable (`d34_flags`)
            set_flag(ctx.core, owner, "l25_wushuang_unrespondable", 1);
        }
        "ssr_jiwu" => {
            // `extension.js:31644`: spends 2 `ssr_kuangbao` marks for Jiu boost & distance bypass
            let kb = flag(ctx.core, owner, "l25_kuangbao_marks");
            if kb >= 2 {
                set_flag(ctx.core, owner, "l25_kuangbao_marks", kb - 2);
                ctx.core.turn.jiu_active = true;
                set_flag(ctx.core, owner, "l25_jiwu_boost", 1);
            }
        }
        "ssr_pojia" => {
            // `extension.js:9474`: `'破兵碎甲': '你的防具已失效'` -> nullifies target armor (`d34_flags`)
            if other != owner {
                set_flag(ctx.core, other, "l25_pojia_broken", 1);
            }
            set_flag(ctx.core, owner, "l25_pojia_ignore_armor", 1);
        }
        "ssr_shendao" => {
            // `extension.js:10377`: `phaseAfter` stores top-deck talisman (`shendao_fu`)
            if store_deck_to_pile(ctx, owner, "shendao_fu") {
                let n = flag(ctx.core, owner, "l25_shendao_fu") + 1;
                set_flag(ctx.core, owner, "l25_shendao_fu", n);
            }
        }
        "ssr_newjiahe" => {
            // `extension.js:10307`: `'驾鹤仙归'` (`qianxing` untargetable by others in `d34_flags`)
            set_flag(ctx.core, owner, "l25_jiahe_qianxing", 1);
        }
        "ssr_miedi" => {
            // `extension.js:10842`: `enable: 'phaseUse'`, once per turn challenges `other` unless `forbid`
            if flag(ctx.core, owner, "l25_miedi_forbid") == 0 && other != owner {
                set_flag(ctx.core, owner, "l25_miedi_forbid", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "ssr_biyue" => {
            // `extension.js:11950`: passive `mod.globalTo` (+1 per empty zone in `d34_flags`) + end-phase draw
            let p = ctx.core.player(owner);
            let bonus = u32::from(p.hand.is_empty())
                + u32::from(p.equips.is_empty())
                + u32::from(p.judge.is_empty());
            draw_cards_sync(ctx, owner, (1 + bonus).min(4));
        }
        "ssr_wusheng" => {
            // `extension.js:12929`: red cards as distance-unlimited Sha (`d34_flags`)
            set_flag(ctx.core, owner, "l25_wusheng_viewas", 1);
        }
        "ssr_yijue" => {
            // `extension.js:13133`: `usable: 1`, discards 1 card to lock `other` hand & armor this turn
            if other != owner && discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, other, "l25_yijue_locked", 1);
            }
        }
        "ssr_newyicong" => {
            // `extension.js:13604`: `mod.globalFrom` distance reduction (`d34_flags`) + draw 1
            ctx.start_draw(owner, 1);
        }
        "ssr_newguose" => {
            // `extension.js:13979`: Diamond as Lebusishu (`d34_flags`) + draw 1
            set_flag(ctx.core, owner, "l25_guose_diamond_lebu", 1);
            ctx.start_draw(owner, 1);
        }
        "ssr_newtongque" => {
            // `extension.js:14304`: gains 2 dice marks (`🎲`) and draws 1 when dice >= 4
            let dice = flag(ctx.core, owner, "l25_tongque_dice") + 2;
            set_flag(ctx.core, owner, "l25_tongque_dice", dice);
            if dice >= 4 {
                ctx.start_draw(owner, 1);
            }
        }
        "ssr_newtianxiang" => {
            // `extension.js:14157`: discard 1 card to deflect damage to `other` (`other` draws lost HP)
            if other != owner && discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l25_tianxiang_deflected", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
                ctx.start_draw(other, 1);
            }
        }
        "ssr_huchi" => {
            // `extension.js:14922`: +1 range & +1 Sha limit (`d34_flags`) + gains 1 hujia
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
        }
        "ssr_zhenhun" => {
            // `extension.js:15202`: forbids `other` from using/responding Sha (`d34_flags`)
            if other != owner {
                set_flag(ctx.core, other, "l25_zhenhun_forbid_sha", 1);
            }
        }
        "ssr_paoxiao" => {
            // `extension.js:15111`: unlimited Sha uses (`d34_flags`)
            set_flag(ctx.core, owner, "l25_paoxiao_unlimited", 1);
        }
        "ssr_tianlang" => {
            // `extension.js:16375`: `phaseZhunbeiBegin` cycles 7 Big Dipper stars & draws 1
            let next_star = (flag(ctx.core, owner, "l25_tianlang_star") % 7) + 1;
            set_flag(ctx.core, owner, "l25_tianlang_star", next_star);
            ctx.start_draw(owner, 1);
        }
        "ssr_newjilue" => {
            // `extension.js:17640`: `useCardAfter` on trick card -> draws 1 card
            ctx.start_draw(owner, 1);
        }
        "ssr_newjijie" => {
            // `extension.js:17980`: stores deck card into `ssr_jijie` expansion for Shan conversion
            let _ = store_deck_to_pile(ctx, owner, "ssr_jijie");
        }
        "ssr_qicai" => {
            // `extension.js:18242`: unlimited trick distance (`d34_flags`) + `jinchan` draw 1
            ctx.start_draw(owner, 1);
        }
        "ssr_manjia" => {
            // `extension.js:22026`: Vine Armor passive (`d34_flags`) + +1 hujia on activation
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
        }
        "ssr_weilong" => {
            // `extension.js:22562`: `mod.cardUsable(sha) + 1` (`d34_flags`) + draws 1
            ctx.start_draw(owner, 1);
        }
        "ssr_newjiang" => {
            // `extension.js:23364`: Red Sha / Juedou draws 1 + basic card as Sha (`d34_flags`)
            set_flag(ctx.core, owner, "l25_jiang_active", 1);
            ctx.start_draw(owner, 1);
        }
        "ssr_newhanzhan" => {
            // `extension.js:23719`: pindian/compare bonus & gains 1 card
            set_flag(ctx.core, owner, "l25_hanzhan_bonus", 3);
            ctx.start_draw(owner, 1);
        }
        "ssr_huwei" => {
            // `extension.js:23900`: `enterGame`/`phaseBefore` water strike (discard `other` or 1 thunder dmg)
            if other != owner && discard_first_he(ctx, other).is_none() {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    other,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "ssr_shensu" => {
            // `extension.js:24198`: fast virtual Sha strike against `other`
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "ssr_hubu" => {
            // `extension.js:24310`: `'丢盔卸甲'` nullifies `other` armor (`d34_flags`) + draws 1
            if other != owner {
                set_flag(ctx.core, other, "l25_hubu_unequip2", 1);
            }
            ctx.start_draw(owner, 1);
        }
        "ssr_jushou" => {
            // `extension.js:24390`: `phaseJieshu` turns over, gains 1 hujia, draws 3, records `defend`
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            ctx.start_draw(owner, 3);
            let def = flag(ctx.core, owner, "l25_jushou_defend") + 1;
            set_flag(ctx.core, owner, "l25_jushou_defend", def);
        }
        "ssr_zhuzhen" => {
            // `extension.js:24891`: consumes `l25_jushou_defend` to heal 1 & draw 2
            let def = flag(ctx.core, owner, "l25_jushou_defend");
            if def > 0 {
                set_flag(ctx.core, owner, "l25_jushou_defend", def - 1);
                if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                }
                ctx.start_draw(owner, 2);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "ssr_baonue" => {
            // `extension.js:25560`: Sha trigger recovers 1 HP if wounded, else draws 1
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "ssr_newmizhou" => {
            // `extension.js:25910`: stores talisman in `ssr_mizhou_lu` pile and draws 1
            let _ = store_deck_to_pile(ctx, owner, "ssr_mizhou_lu");
            ctx.start_draw(owner, 1);
        }
        "ssr_chanyuan" => {
            // `extension.js:26431`: inflicts `ssr_chanyuan_crazy` (`-1` hand limit in `d34_flags`)
            if other != owner {
                let c = flag(ctx.core, other, "l25_chanyuan_crazy") + 1;
                set_flag(ctx.core, other, "l25_chanyuan_crazy", c);
            }
        }
        "ssr_zhuhai" => {
            // `extension.js:33853`: `phaseJieshuBegin` punishes `other` with virtual Sha damage
            if other != owner {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    other,
                    1,
                )));
            }
        }
        "ssr_newfabing" => {
            // `extension.js:27618`: `useCardToPlayered` Sha forces `other` to discard 1 card
            if other != owner {
                let _ = discard_first_he(ctx, other);
            }
        }
        "ssr_newguidao" => {
            // `extension.js:32897`: black card retrial flips top deck card; black draws 1
            if let Some(jc) = pop_deck(ctx) {
                let is_black = matches!(
                    ctx.catalog.cards[jc as usize].suit,
                    Suit::Spade | Suit::Club
                );
                discard_some(ctx, owner, vec![jc]);
                if is_black {
                    ctx.start_draw(owner, 1);
                }
            }
            set_flag(ctx.core, owner, "l25_guidao_retrial", 1);
        }
        "ssr_zhuji" => {
            // `extension.js:32191`: thunder lord skill (`xinleiji`) deals 1 thunder damage
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
        _ => return false,
    }
    set_flag(ctx.core, owner, &format!("l25_skill:{skill_name}"), 1);
    true
}
