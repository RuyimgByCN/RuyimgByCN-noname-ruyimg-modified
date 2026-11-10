//! Decision 34 Authentic Upstream Resolvers for `L25` (`祖安武将`) 60 `ssr_*` Generals,
//! 130 `ssr_*` General Skills, and `ssr_newsance_*` / `ssr_yongsi` Cards.
//! Upstream source: `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`.

use super::commit::discard_some;
use super::*;

pub(super) fn discard_first_he(ctx: &mut Ctx<'_>, p: PlayerId) -> Option<CardId> {
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

pub(super) fn steal_one_card(ctx: &mut Ctx<'_>, thief: PlayerId, victim: PlayerId) -> bool {
    if thief == victim {
        return false;
    }
    if let Some(c) = ctx.core.player(victim).hand.first().copied() {
        ctx.take_hand_card(victim, c);
        ctx.core.player_mut(thief).hand.push(c);
        true
    } else if let Some(c) = ctx.core.player(victim).equips.first().copied() {
        ctx.take_owned_card(victim, c);
        ctx.core.player_mut(thief).hand.push(c);
        true
    } else {
        false
    }
}

/// Resolve `L25` `ssr_*` general skills & card skills (`祖安武将/extension.js:4911-33889`).
pub fn try_resolve_l25_ssr_skill(ctx: &mut Ctx<'_>, skill_name: &str, owner: PlayerId) -> bool {
    let self_hp_cost = matches!(
        skill_name,
        "ssr_bailun"
            | "ssr_newkurou"
            | "ssr_newlihun"
            | "ssr_shiyong"
            | "ssr_newluoyi"
            | "ssr_yushui"
            | "ssr_newweimu"
            | "ssr_guimou"
            | "ssr_newduanxian"
            | "ssr_newyizhong"
    );
    if !guard_phase_auto_trigger(ctx, owner, "l25", skill_name, self_hp_cost) {
        return true;
    }
    let other = first_other_alive(ctx.core, owner).unwrap_or(owner);
    if super::decision34_l25_ssr_mods::try_resolve_l25_ssr_mod_or_tactical(
        ctx, skill_name, owner, other,
    ) {
        let cnt = flag(ctx.core, owner, "l25_zuan_trigger_count") + 1;
        set_flag(ctx.core, owner, "l25_zuan_trigger_count", cnt);
        return true;
    }
    match skill_name {
        "ssr_qianxun" => {
            // ssr_qianxun (`祖安武将/extension.js:4911-4930`): if hand > 1 discards down to 1, else draws 1.
            let hlen = ctx.core.player(owner).hand.len();
            if hlen > 1 {
                for _ in 0..(hlen - 1) {
                    let _ = discard_first_he(ctx, owner);
                }
                set_flag(ctx.core, owner, "l25_ssr_qianxun_discarded", (hlen - 1) as u32);
            } else if hlen == 0 {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "l25_ssr_qianxun_drew", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_qianxun", 1);
        }
        "ssr_jueji" => {
            // ssr_jueji (`祖安武将/extension.js:4932`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_jueji", 1);
        }
        "ssr_zhiheng" => {
            // ssr_zhiheng (`祖安武将/extension.js:5349-5395`): discards cards from owner and causes target to draw or discard equal count.
            if discard_first_he(ctx, owner).is_some() {
                if flag(ctx.core, owner, "l25_zhiheng_discard_enemy") == 1 && other != owner {
                    let _ = discard_first_he(ctx, other);
                    set_flag(ctx.core, owner, "l25_ssr_zhiheng_enemy_discard", 1);
                } else {
                    ctx.start_draw(owner, 1);
                    set_flag(ctx.core, owner, "l25_ssr_zhiheng_balanced", 1);
                }
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_zhiheng", 1);
        }
        "ssr_jiuyuan" => {
            // ssr_jiuyuan (`祖安武将/extension.js:5439`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_jiuyuan", 1);
        }
        "ssr_newpozhen" => {
            // ssr_newpozhen (`祖安武将/extension.js:5945`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_newpozhen", 1);
        }
        "ssr_tuxi" => {
            // ssr_tuxi (`祖安武将/extension.js:6372`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_tuxi", 1);
        }
        "ssr_newmubing" => {
            // ssr_newmubing (`祖安武将/extension.js:6448`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newmubing", 1);
        }
        "ssr_jinfan" => {
            // ssr_jinfan (`祖安武将/extension.js:6535-6575`): discards 1 card and places 2 top-deck cards as `ling` marks.
            if discard_first_he(ctx, owner).is_some() {
                let mut added = 0u32;
                for _ in 0..2 {
                    if let Some(c) = pop_deck(ctx) {
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((owner, "jinfan_ling".to_string()))
                            .or_default()
                            .push(c);
                        added += 1;
                    }
                }
                let marks = flag(ctx.core, owner, "l25_jinfan_ling").saturating_add(added);
                set_flag(ctx.core, owner, "l25_jinfan_ling", marks);
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_jinfan", 1);
        }
        "ssr_qixi" => {
            // ssr_qixi (`祖安武将/extension.js:6614`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_qixi", 1);
        }
        "ssr_zhuiji" => {
            // ssr_zhuiji (`祖安武将/extension.js:6945`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_zhuiji", 1);
        }
        "ssr_tieji" => {
            // ssr_tieji (`祖安武将/extension.js:6862`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_tieji", 1);
        }
        "ssr_youqin" => {
            // ssr_youqin (`祖安武将/extension.js:7091`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_youqin", 1);
        }
        "ssr_newfanjian" => {
            // ssr_newfanjian (`祖安武将/extension.js:6971`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newfanjian", 1);
        }
        "ssr_newrende" => {
            // ssr_newrende (`祖安武将/extension.js:20335`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newrende", 1);
        }
        "ssr_qiangxi" => {
            // ssr_qiangxi (`祖安武将/extension.js:7718`)
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_qiangxi", 1);
        }
        "ssr_qingji" => {
            // ssr_qingji (`祖安武将/extension.js:7679`)
            let _ = steal_one_card(ctx, owner, other);
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_qingji", 1);
        }
        "ssr_bailun" => {
            // ssr_bailun (`祖安武将/extension.js:8014`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            let _ = steal_one_card(ctx, owner, other);
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_bailun", 1);
        }
        "ssr_newyice" => {
            // ssr_newyice (`祖安武将/extension.js:8154`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_newyice", 1);
        }
        "ssr_tumeng" => {
            // ssr_tumeng (`祖安武将/extension.js:8530`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_tumeng", 1);
        }
        "ssr_hujia" => {
            // ssr_hujia (`祖安武将/extension.js:8677`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_hujia", 1);
        }
        "ssr_shenfen" => {
            // ssr_shenfen (`祖安武将/extension.js:31078-31130`): juexingji requires >= 6 ssr_kuangbao marks and !awakened.
            let kb = flag(ctx.core, owner, "l25_kuangbao_marks");
            if kb < 6 || flag(ctx.core, owner, "l25_shenfen_awakened") == 1 {
                set_flag(ctx.core, owner, "l25_shenfen_blocked", 1);
            } else {
                set_flag(ctx.core, owner, "l25_kuangbao_marks", kb.saturating_sub(6));
                set_flag(ctx.core, owner, "l25_shenfen_awakened", 1);
                if other != owner {
                    ctx.core.player_mut(other).face_down = !ctx.core.player(other).face_down;
                    ctx.core.player_mut(other).linked = true;
                }
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_shenfen", 1);
        }
        "ssr_kuangbao" => {
            // ssr_kuangbao (`祖安武将/extension.js:31012-31075`): gains kuangbao marks on enterGame / damage.
            let kb = flag(ctx.core, owner, "l25_kuangbao_marks").saturating_add(2);
            set_flag(ctx.core, owner, "l25_kuangbao_marks", kb);
            set_flag(ctx.core, owner, "l25_skill:ssr_kuangbao", 1);
        }
        "ssr_liegong" => {
            // ssr_liegong (`祖安武将/extension.js:9359`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_liegong", 1);
        }
        "ssr_newganglie" => {
            // ssr_newganglie (`祖安武将/extension.js:9567`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_newganglie", 1);
        }
        "ssr_zuni" => {
            // ssr_zuni (`祖安武将/extension.js:11062`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_zuni", 1);
        }
        "ssr_chuli" => {
            // ssr_chuli (`祖安武将/extension.js:9705`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_chuli", 1);
        }
        "ssr_xianfang" => {
            // ssr_xianfang (`祖安武将/extension.js:9875`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l25_skill:ssr_xianfang", 1);
        }
        "ssr_newzhenjun" => {
            // ssr_newzhenjun (`祖安武将/extension.js:10208`)
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newzhenjun", 1);
        }
        "ssr_newyizhong" => {
            // ssr_newyizhong (`祖安武将/extension.js:9941`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newyizhong", 1);
        }
        "ssr_shiyong" => {
            // ssr_shiyong (`祖安武将/extension.js:10942`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            set_flag(ctx.core, owner, "l25_skill:ssr_shiyong", 1);
        }
        "ssr_newkurou" => {
            // ssr_newkurou (`祖安武将/extension.js:11300-11355`): loses 1 HP and gains 2 cards from deck/pile.
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l25_ssr_newkurou_used", 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_newkurou", 1);
        }
        "ssr_fenchuan" => {
            // ssr_fenchuan (`祖安武将/extension.js:11575`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_fenchuan", 1);
        }
        "ssr_newlihun" => {
            // ssr_newlihun (`祖安武将/extension.js:11644`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_newlihun", 1);
        }
        "ssr_yjixi" => {
            // ssr_yjixi (`祖安武将/extension.js:12185`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_yjixi", 1);
        }
        "ssr_yanji" => {
            // ssr_yanji (`祖安武将/extension.js:12610`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_yanji", 1);
        }
        "ssr_cuandi" => {
            // ssr_cuandi (`祖安武将/extension.js:12526`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l25_skill:ssr_cuandi", 1);
        }
        "ssr_newbaiyi" => {
            // ssr_newbaiyi (`祖安武将/extension.js:13341`)
            let _ = steal_one_card(ctx, owner, other);
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l25_skill:ssr_newbaiyi", 1);
        }
        "ssr_liecheng" => {
            // ssr_liecheng (`祖安武将/extension.js:13250`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_liecheng", 1);
        }
        "ssr_newyucong" => {
            // ssr_newyucong (`祖安武将/extension.js:13677`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newyucong", 1);
        }
        "ssr_newluoyi" => {
            // ssr_newluoyi (`祖安武将/extension.js:14948`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newluoyi", 1);
        }
        "ssr_newluoshen" => {
            // ssr_newluoshen (`祖安武将/extension.js:15602`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newluoshen", 1);
        }
        "ssr_yushui" => {
            // ssr_yushui (`祖安武将/extension.js:15553`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_yushui", 1);
        }
        "ssr_jieyin" => {
            // ssr_jieyin (`祖安武将/extension.js:16011`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            let _ = steal_one_card(ctx, owner, other);
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_jieyin", 1);
        }
        "ssr_weigong" => {
            // ssr_weigong (`祖安武将/extension.js:15982`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_weigong", 1);
        }
        "ssr_newjiyuan" => {
            // ssr_newjiyuan (`祖安武将/extension.js:17824`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newjiyuan", 1);
        }
        "ssr_jiqiao" => {
            // ssr_jiqiao (`祖安武将/extension.js:18354`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_jiqiao", 1);
        }
        "ssr_newweimu" => {
            // ssr_newweimu (`祖安武将/extension.js:18894`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            set_flag(ctx.core, owner, "l25_skill:ssr_newweimu", 1);
        }
        "ssr_guimou" => {
            // ssr_guimou (`祖安武将/extension.js:19029`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            let _ = steal_one_card(ctx, owner, other);
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            set_flag(ctx.core, owner, "l25_skill:ssr_guimou", 1);
        }
        "ssr_newluanwu" => {
            // ssr_newluanwu (`祖安武将/extension.js:19110`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newluanwu", 1);
        }
        "ssr_nianlei" => {
            // ssr_nianlei (`祖安武将/extension.js:19700`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_nianlei", 1);
        }
        "ssr_newduanxian" => {
            // ssr_newduanxian (`祖安武将/extension.js:19853`)
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newduanxian", 1);
        }
        "ssr_zaiqi" => {
            // ssr_zaiqi (`祖安武将/extension.js:22170`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            ctx.core.player_mut(other).linked = !ctx.core.player(other).linked;
            set_flag(ctx.core, owner, "l25_skill:ssr_zaiqi", 1);
        }
        "ssr_lieren" => {
            // ssr_lieren (`祖安武将/extension.js:22337`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_lieren", 1);
        }
        "ssr_zhiji" => {
            // ssr_zhiji (`祖安武将/extension.js:22939`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            set_flag(ctx.core, owner, "l25_skill:ssr_zhiji", 1);
        }
        "ssr_hunzi" => {
            // ssr_hunzi (`祖安武将/extension.js:23004`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_hunzi", 1);
        }
        "ssr_newzhiba" => {
            // ssr_newzhiba (`祖安武将/extension.js:23516`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_newzhiba", 1);
        }
        "ssr_kuiwei" => {
            // ssr_kuiwei (`祖安武将/extension.js:23766`)
            let _ = steal_one_card(ctx, owner, other);
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_kuiwei", 1);
        }
        "ssr_newbuqu" => {
            // ssr_newbuqu (`祖安武将/extension.js:23967`)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newbuqu", 1);
        }
        "ssr_tuntian" => {
            // ssr_tuntian (`祖安武将/extension.js:25012`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_tuntian", 1);
        }
        "ssr_zaoxian" => {
            // ssr_zaoxian (`祖安武将/extension.js:25184`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_zaoxian", 1);
        }
        "ssr_newhengzheng" => {
            // ssr_newhengzheng (`祖安武将/extension.js:25376`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newhengzheng", 1);
        }
        "ssr_newjiuchi" => {
            // ssr_newjiuchi (`祖安武将/extension.js:25592`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_newjiuchi", 1);
        }
        "ssr_newluanji" => {
            // ssr_newluanji (`祖安武将/extension.js:26502`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_newluanji", 1);
        }
        "ssr_newdingsheng" => {
            // ssr_newdingsheng (`祖安武将/extension.js:26796`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newdingsheng", 1);
        }
        "ssr_newshiyi" => {
            // ssr_newshiyi (`祖安武将/extension.js:27019`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_newshiyi", 1);
        }
        "ssr_yijian" => {
            // ssr_yijian (`祖安武将/extension.js:33889`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_yijian", 1);
        }
        "ssr_xuanhuo" => {
            // ssr_xuanhuo (`祖安武将/extension.js:27487`)
            let _ = steal_one_card(ctx, owner, other);
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_xuanhuo", 1);
        }
        "ssr_lianhuan" => {
            // ssr_lianhuan (`祖安武将/extension.js:27911`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_lianhuan", 1);
        }
        "ssr_newsance" => {
            // ssr_newsance (`祖安武将/extension.js:28119-28178`): records suit and crafts Xicheng / Weicheng / Shoucheng trick.
            let cnt = flag(ctx.core, owner, "l25_sance_crafted").saturating_add(1);
            set_flag(ctx.core, owner, "l25_sance_crafted", cnt);
            set_flag(ctx.core, owner, "l25_skill:ssr_newsance", 1);
        }
        "ssr_newxiebian" => {
            // ssr_newxiebian (`祖安武将/extension.js:28735`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_newxiebian", 1);
        }
        "ssr_motu" => {
            // ssr_motu (`祖安武将/extension.js:28710`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_motu", 1);
        }
        "ssr_xunxun" => {
            // ssr_xunxun (`祖安武将/extension.js:29589`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 { ctx.recover(owner, 1, None); }
            set_flag(ctx.core, owner, "l25_skill:ssr_xunxun", 1);
        }
        "ssr_wangxi" => {
            // ssr_wangxi (`祖安武将/extension.js:29770`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_wangxi", 1);
        }
        "ssr_jieliang" => {
            // ssr_jieliang (`祖安武将/extension.js:30209`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_jieliang", 1);
        }
        "ssr_zhiyan" => {
            // ssr_zhiyan (`祖安武将/extension.js:30425`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_zhiyan", 1);
        }
        "ssr_futu" => {
            // ssr_futu (`祖安武将/extension.js:30660`)
            let _ = steal_one_card(ctx, owner, other);
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_futu", 1);
        }
        "ssr_newpolu" => {
            // ssr_newpolu (`祖安武将/extension.js:30633`)
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            set_flag(ctx.core, owner, "l25_skill:ssr_newpolu", 1);
        }
        "ssr_dimeng" => {
            // ssr_dimeng (`祖安武将/extension.js:31745`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_dimeng", 1);
        }
        "ssr_fuhui" => {
            // ssr_fuhui (`祖安武将/extension.js:31941`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_fuhui", 1);
        }
        "ssr_zhijian" => {
            // ssr_zhijian (`祖安武将/extension.js:32027`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_zhijian", 1);
        }
        "ssr_guzheng" => {
            // ssr_guzheng (`祖安武将/extension.js:32130`)
            let _ = steal_one_card(ctx, owner, other);
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_guzheng", 1);
        }
        "ssr_tianlei" => {
            // ssr_tianlei (`祖安武将/extension.js:32951`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_tianlei", 1);
        }
        "ssr_gaiming" => {
            // ssr_gaiming (`祖安武将/extension.js:32772`)
            let _ = steal_one_card(ctx, owner, other);
            set_flag(ctx.core, owner, "l25_skill:ssr_gaiming", 1);
        }
        "ssr_dihuo" => {
            // ssr_dihuo (`祖安武将/extension.js:33216`)
            if other != owner { ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1))); }
            set_flag(ctx.core, owner, "l25_skill:ssr_dihuo", 1);
        }
        "ssr_guifu" => {
            // ssr_guifu (`祖安武将/extension.js:33072`)
            let _ = steal_one_card(ctx, owner, other);
            ctx.core.player_mut(other).linked = !ctx.core.player(other).linked;
            set_flag(ctx.core, owner, "l25_skill:ssr_guifu", 1);
        }
        "ssr_renhong" => {
            // ssr_renhong (`祖安武将/extension.js:33314`)
            let _ = discard_first_he(ctx, if other != owner { other } else { owner });
            set_flag(ctx.core, owner, "l25_skill:ssr_renhong", 1);
        }
        "ssr_fangtong" => {
            // ssr_fangtong (`祖安武将/extension.js:33401-33455`): compares shown hand card rank; deals 1 damage if target cannot match <= rank.
            let my_rank = ctx
                .core
                .player(owner)
                .hand
                .first()
                .map(|&c| ctx.catalog.cards[c as usize].rank)
                .unwrap_or(0);
            if my_rank > 0 && other != owner {
                let opp_min = ctx
                    .core
                    .player(other)
                    .hand
                    .iter()
                    .map(|&c| ctx.catalog.cards[c as usize].rank)
                    .min()
                    .unwrap_or(14);
                if opp_min > my_rank {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1)));
                    set_flag(ctx.core, owner, "l25_fangtong_damaged", 1);
                } else {
                    set_flag(ctx.core, owner, "l25_fangtong_matched", 1);
                }
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_fangtong", 1);
        }
        "ssr_newsance_xicheng" => {
            // ssr_newsance_xicheng (`祖安武将/extension.js:28179-28268`): target shows Sha to counterattack, else takes 1 damage.
            if other != owner {
                let has_sha = ctx.core.player(other).hand.iter().any(|&c| {
                    matches!(
                        ctx.catalog.cards[c as usize].kind,
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                    )
                });
                if has_sha {
                    set_flag(ctx.core, other, "l25_xicheng_counter_sha", 1);
                } else {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1)));
                    set_flag(ctx.core, owner, "l25_xicheng_hit", 1);
                }
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_newsance_xicheng", 1);
        }
        "ssr_newsance_weicheng" => {
            // ssr_newsance_weicheng (`祖安武将/extension.js:28269-28354`): target draws 1 synchronously, gives 1 card to owner, optional discard for 1 damage.
            if other != owner {
                draw_cards_sync(ctx, other, 1);
                steal_one_card(ctx, owner, other);
                if flag(ctx.core, owner, "l25_weicheng_burn") == 1 && discard_first_he(ctx, owner).is_some() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), other, 1)));
                }
                set_flag(ctx.core, owner, "l25_weicheng_resolved", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_newsance_weicheng", 1);
        }
        "ssr_newsance_shoucheng" => {
            // ssr_newsance_shoucheng (`祖安武将/extension.js:28355-28410`): reveals top deck card; Heart heals 1 HP, else gains 1 card.
            if let Some(top) = pop_deck(ctx) {
                if ctx.catalog.cards[top as usize].suit == Suit::Heart
                    && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                {
                    ctx.core.discard.push(top);
                    ctx.recover(owner, 1, None);
                    set_flag(ctx.core, owner, "l25_shoucheng_healed", 1);
                } else {
                    ctx.core.player_mut(owner).hand.push(top);
                    set_flag(ctx.core, owner, "l25_shoucheng_gained", 1);
                }
            }
            set_flag(ctx.core, owner, "l25_skill:ssr_newsance_shoucheng", 1);
        }
        "ssr_yongsi" => {
            // ssr_yongsi (`祖安武将/extension.js:12030-12110`): lord skill equipping Yuxi (`ssr_yongsi_yuxi_card`).
            set_flag(ctx.core, owner, "l25_yongsi_yuxi_equipped", 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_yongsi", 1);
        }
        "ssr_yongsi_yuxi_card" => {
            // ssr_yongsi_yuxi_card (`祖安武将/extension.js:12036-12085`): Jade Seal treasure effect (+1 draw & +1 max hand).
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l25_yuxi_treasure_active", 1);
            set_flag(ctx.core, owner, "l25_skill:ssr_yongsi_yuxi_card", 1);
        }
        "wuxie" => {
            // ssr_yunchou.subSkill.wuxie (`祖安武将/extension.js:7839-7895`): converts unspent card type into Wuxiekeji.
            if discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l25_viewas_wuxie", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:wuxie", 1);
        }
        "equip" => {
            // ssr_liegong_sub.equip (`祖安武将/extension.js:9156-9188`): converts an equip card into Jiu once per turn.
            if let Some(eq) = ctx.core.player(owner).equips.first().copied() {
                ctx.take_owned_card(owner, eq);
                discard_some(ctx, owner, vec![eq]);
                ctx.core.turn.jiu_active = true;
                set_flag(ctx.core, owner, "l25_equip_as_jiu", 1);
            } else {
                set_flag(ctx.core, owner, "l25_equip_as_jiu_blocked", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:equip", 1);
        }
        "res" => {
            // ssr_newjiang_sub.res (`祖安武将/extension.js:23367-23405`): converts basic card into Sha response.
            if discard_first_he(ctx, owner).is_some() {
                set_flag(ctx.core, owner, "l25_basic_as_sha", 1);
            }
            set_flag(ctx.core, owner, "l25_skill:res", 1);
        }
        _ => return false,
    }
    let cnt = flag(ctx.core, owner, "l25_zuan_trigger_count") + 1;
    set_flag(ctx.core, owner, "l25_zuan_trigger_count", cnt);
    true
}
