//! Authoritative `U05 杀海拾遗` (`gujian` & `gwent`) host-side skill/card resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_gujian_gwent(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // character/xianjian/skill.js / pal_murongziying xuanning (kept for U05_XIANJIAN)
        "xuanning" => {
            u05_mark_invoked(ctx, owner, "xuanning");
            set_flag(ctx.core, owner, "xianjian_skill:xuanning", 1);
            set_flag(ctx.core, owner, "xianjian_ward_active", 1);
            let _ = u05_store_pile(ctx, owner, "xuanning");
            u05_recover(ctx, owner, 1);
        }
        // card/gwent/skill.js:5-35
        "gw_aerdeyin" =>  { u05_mark_invoked(ctx, owner, "gw_aerdeyin"); u05_turn_over(ctx, owner); },
        // card/gwent/skill.js:80-122
        "gw_baobaoshu" => { u05_mark_invoked(ctx, owner, "gw_baobaoshu"); let _ = u05_discard_up_to(ctx, owner, 1); }
        // card/gwent/skill.js:173-197
        "gw_qinpendayu" =>  { u05_mark_invoked(ctx, owner, "gw_qinpendayu"); set_flag(ctx.core, owner, "u05_mod:gw_qinpendayu", 1); },
        // card/gwent/skill.js:198-224
        "gw_birinongwu" =>  { u05_mark_invoked(ctx, owner, "gw_birinongwu"); set_flag(ctx.core, owner, "u05_mod:gw_birinongwu", 1); },
        // card/gwent/skill.js:299-335
        "gw_dieyi" =>  { u05_mark_invoked(ctx, owner, "gw_dieyi"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // card/gwent/skill.js:336-389
        "_gainspell" =>  { u05_mark_invoked(ctx, owner, "_gainspell"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:31-73
        "sqlongnu" =>  { u05_mark_invoked(ctx, owner, "sqlongnu"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:74-119
        "sqlonghuo" => {
            if u05_check_usable(ctx, owner, "sqlonghuo", 1) {
                u05_mark_invoked(ctx, owner, "sqlonghuo");
                let n = ctx.core.player(owner).hand.len();
                if n > 0 {
                    let _ = u05_discard_up_to(ctx, owner, n);
                    ctx.start_draw(owner, n as u32);
                }
                u05_damage_other(ctx, owner, 1, DamageNature::Fire);
            }
        }
        // character/gwent/skill.js:120-148
        "sqlongwu" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "sqlongwu", false) && !ctx.core.player(owner).hand.is_empty() {
                u05_mark_invoked(ctx, owner, "sqlongwu");
                let mut max_dup = 1u32;
                let mut counts = std::collections::BTreeMap::<u8, u32>::new();
                for &cid in &ctx.core.player(owner).hand {
                    let k = ctx.catalog.card_kind(cid) as u8;
                    let e = counts.entry(k).or_insert(0);
                    *e += 1;
                    max_dup = max_dup.max(*e);
                }
                ctx.start_draw(owner, max_dup);
            }
        }
        // character/gwent/skill.js:368-416
        "gwjinli" =>  { u05_mark_invoked(ctx, owner, "gwjinli"); if !u05_check_usable(ctx, owner, "gwjinli", 1) { return true; } if !u05_pay_discard(ctx, owner, "gwjinli", 1, true) { return true; } ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:gwjinli", 1); } },
        // character/gwent/skill.js:450-499
        "gwhuanbi" =>  { u05_mark_invoked(ctx, owner, "gwhuanbi"); if !u05_check_usable(ctx, owner, "gwhuanbi", 1) { return true; } if !u05_pay_discard(ctx, owner, "gwhuanbi", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:500-575
        "gwminxiang" =>  { u05_mark_invoked(ctx, owner, "gwminxiang"); if !u05_check_usable(ctx, owner, "gwminxiang", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:576-605
        "gwlangshi" =>  { u05_mark_invoked(ctx, owner, "gwlangshi"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/gwent/skill.js:606-703
        "gwjingtian" =>  { u05_mark_invoked(ctx, owner, "gwjingtian"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:794-861
        "gwweitu" =>  { u05_mark_invoked(ctx, owner, "gwweitu"); u05_change_hujia(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:862-962
        "gwgouhun" =>  { u05_mark_invoked(ctx, owner, "gwgouhun"); if !u05_check_usable(ctx, owner, "gwgouhun", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:963-1042
        "gwfutian" =>  { u05_mark_invoked(ctx, owner, "gwfutian"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:1062-1123
        "gwyewu" =>  { u05_mark_invoked(ctx, owner, "gwyewu"); if !u05_check_usable(ctx, owner, "gwyewu", 1) { return true; } if !u05_pay_discard(ctx, owner, "gwyewu", 1, true) { return true; } ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:1243-1320
        "gwchenshui" =>  { u05_mark_invoked(ctx, owner, "gwchenshui"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/gwent/skill.js:1363-1381
        "julian" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "julian", false) {
                let my_h = ctx.core.player(owner).hand.len();
                let max_other = (0..ctx.core.players.len() as PlayerId)
                    .filter(|&p| p != owner && ctx.core.player(p).alive)
                    .map(|p| ctx.core.player(p).hand.len())
                    .max()
                    .unwrap_or(0);
                if max_other > my_h {
                    u05_mark_invoked(ctx, owner, "julian");
                    ctx.start_draw(owner, (max_other - my_h) as u32);
                }
            }
        }
        // character/gwent/skill.js:1382-1413
        "gwfusheng" =>  { u05_mark_invoked(ctx, owner, "gwfusheng"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 2); if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/gwent/skill.js:1414-1454
        "gwqinwu" =>  { u05_mark_invoked(ctx, owner, "gwqinwu"); if !u05_check_usable(ctx, owner, "gwqinwu", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:gwqinwu", 1); } },
        // character/gwent/skill.js:1485-1520
        "huanshu2" =>  { u05_mark_invoked(ctx, owner, "huanshu2"); ctx.start_draw(owner, 2); },
        // character/gwent/skill.js:1636-1700
        "zhengjun" =>  { u05_mark_invoked(ctx, owner, "zhengjun"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); u05_change_max_hp(ctx, owner, 1); },
        // character/gwent/skill.js:1701-1710
        "gwxuezhan" =>  { u05_mark_invoked(ctx, owner, "gwxuezhan"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:1711-1774
        "bolang" =>  { u05_mark_invoked(ctx, owner, "bolang"); if !u05_check_usable(ctx, owner, "bolang", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:1895-1940
        "lingji" =>  { u05_mark_invoked(ctx, owner, "lingji"); if !u05_check_usable(ctx, owner, "lingji", 1) { return true; } ctx.start_draw(owner, 2); },
        // character/gwent/skill.js:1941-1990
        "gwjinyan" =>  { u05_mark_invoked(ctx, owner, "gwjinyan"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:1991-2091
        "gwshenyu" =>  { u05_mark_invoked(ctx, owner, "gwshenyu"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2092-2130
        "junchi" =>  { u05_mark_invoked(ctx, owner, "junchi"); if !u05_check_usable(ctx, owner, "junchi", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2131-2196
        "hupeng" =>  { u05_mark_invoked(ctx, owner, "hupeng"); if !u05_check_usable(ctx, owner, "hupeng", 1) { return true; } if !u05_pay_discard(ctx, owner, "hupeng", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2197-2234
        "hupeng2" =>  { u05_mark_invoked(ctx, owner, "hupeng2"); set_flag(ctx.core, owner, "u05_mod:hupeng2", 1); },
        // character/gwent/skill.js:2275-2353
        "hunmo" =>  { u05_mark_invoked(ctx, owner, "hunmo"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:2355-2392
        "shuijian" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "shuijian", false) && u05_pay_discard(ctx, owner, "shuijian", 1, false) {
                u05_mark_invoked(ctx, owner, "shuijian");
                u05_damage_aoe(ctx, owner, 1, DamageNature::Normal);
            }
        }
        // character/gwent/skill.js:2393-2417
        "yunhuo" =>  { u05_mark_invoked(ctx, owner, "yunhuo"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/gwent/skill.js:2418-2445
        "yinzhang" =>  { u05_mark_invoked(ctx, owner, "yinzhang"); if !u05_check_usable(ctx, owner, "yinzhang", 1) { return true; } if !u05_pay_discard(ctx, owner, "yinzhang", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2517-2539
        "gwxiaoshou" =>  { u05_mark_invoked(ctx, owner, "gwxiaoshou"); if !u05_check_usable(ctx, owner, "gwxiaoshou", 2) { return true; } if !u05_pay_discard(ctx, owner, "gwxiaoshou", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/gwent/skill.js:2540-2556
        "kuanglie" =>  { u05_mark_invoked(ctx, owner, "kuanglie"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2558-2612
        "gwjiquan" =>  { u05_mark_invoked(ctx, owner, "gwjiquan"); if !u05_check_usable(ctx, owner, "gwjiquan", 1) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/gwent/skill.js:2613-2660
        "nuhou" =>  { u05_mark_invoked(ctx, owner, "nuhou"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/gwent/skill.js:2661-2711
        "shewu" =>  { u05_mark_invoked(ctx, owner, "shewu"); if !u05_check_usable(ctx, owner, "shewu", 1) { return true; } if !u05_pay_discard(ctx, owner, "shewu", 1, true) { return true; } u05_recover(ctx, owner, 1); ctx.start_draw(owner, 3); set_flag(ctx.core, owner, "u05_mod:shewu", 1); },
        // character/gwent/skill.js:2712-2771
        "gwzhanjiang" =>  { u05_mark_invoked(ctx, owner, "gwzhanjiang"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2773-2783
        "gwzhanjiang3" =>  { u05_mark_invoked(ctx, owner, "gwzhanjiang3"); ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2784-2809
        "gwchuanxin" =>  { u05_mark_invoked(ctx, owner, "gwchuanxin"); set_flag(ctx.core, owner, "u05_mod:gwchuanxin", 1); },
        // character/gwent/skill.js:2845-2901
        "huandie" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "huandie", false) {
                u05_mark_invoked(ctx, owner, "huandie");
                u05_toggle_link(ctx, owner);
                if let Some(t) = hostile_target(ctx, owner) {
                    u05_toggle_link(ctx, t);
                }
                ctx.start_draw(owner, 1);
            }
        }
        // character/gwent/skill.js:2946-2982
        "xuezhou_hp" =>  { u05_mark_invoked(ctx, owner, "xuezhou_hp"); u05_recover(ctx, owner, 1); u05_lose_hp(ctx, owner, 1); },
        // character/gwent/skill.js:2984-3029
        "fayin" =>  { u05_mark_invoked(ctx, owner, "fayin"); u05_change_hujia(ctx, owner, 1); u05_damage_other(ctx, owner, 1, DamageNature::Fire); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/gwent/skill.js:3030-3087
        "gwbaquan" =>  { u05_mark_invoked(ctx, owner, "gwbaquan"); if !u05_check_usable(ctx, owner, "gwbaquan", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/gwent/skill.js:3145-3197
        "lanquan" =>  { u05_mark_invoked(ctx, owner, "lanquan"); if !u05_check_usable(ctx, owner, "lanquan", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/gwent/skill.js:123-148
        "gw_nuhaifengbao" =>  { u05_mark_invoked(ctx, owner, "gw_nuhaifengbao"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/gwent/skill.js:149-192
        "gw_youer" =>  { u05_mark_invoked(ctx, owner, "gw_youer"); if !u05_pay_discard(ctx, owner, "gw_youer", 1, true) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // card/gwent/skill.js:270-310
        "gw_leizhoushu" =>  { u05_mark_invoked(ctx, owner, "gw_leizhoushu"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gujian/skill.js:130-185
        "xuanzhen" =>  { u05_mark_invoked(ctx, owner, "xuanzhen"); if !u05_pay_discard(ctx, owner, "xuanzhen", 1, true) { return true; } },
        // character/gujian/skill.js:481-501
        "woxue" =>  { u05_mark_invoked(ctx, owner, "woxue"); if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/gujian/skill.js:502-602
        "yange" =>  { u05_mark_invoked(ctx, owner, "yange"); if !u05_pay_discard(ctx, owner, "yange", 1, true) { return true; } },
        // character/gujian/skill.js:603-678
        "lianjing" =>  { u05_mark_invoked(ctx, owner, "lianjing"); if !u05_pay_discard(ctx, owner, "lianjing", 1, true) { return true; } ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:extra_turn", 1); ctx.start_draw(owner, 2); },
        // character/gujian/skill.js:803-869
        "zhenying" =>  { u05_mark_invoked(ctx, owner, "zhenying"); if !u05_check_usable(ctx, owner, "zhenying", 1) { return true; } if !u05_pay_discard(ctx, owner, "zhenying", 1, true) { return true; } },
        // character/gujian/skill.js:870-926
        "lingnu" =>  { u05_mark_invoked(ctx, owner, "lingnu"); if !u05_pay_discard(ctx, owner, "lingnu", 1, true) { return true; } set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/gujian/skill.js:1092-1168
        "yunyin" =>  { u05_mark_invoked(ctx, owner, "yunyin"); if !u05_check_usable(ctx, owner, "yunyin", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/gujian/skill.js:1313-1345
        "xiuhua" =>  { u05_mark_invoked(ctx, owner, "xiuhua"); if !u05_pay_discard(ctx, owner, "xiuhua", 1, true) { return true; } },
        // character/gujian/skill.js:1346-1388
        "liuying" =>  { u05_mark_invoked(ctx, owner, "liuying"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:liuying", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/gujian/skill.js:1556-1589
        "meiying" =>  { u05_mark_invoked(ctx, owner, "meiying"); if !u05_check_usable(ctx, owner, "meiying", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/gujian/skill.js:1590-1610
        "jianwu" =>  { u05_mark_invoked(ctx, owner, "jianwu"); set_flag(ctx.core, owner, "u05_mod:jianwu", 1); },
        // character/gujian/skill.js:1655-1695
        "meihu" =>  { u05_mark_invoked(ctx, owner, "meihu"); let _ = u05_prey_other(ctx, owner, true); },
        // character/gujian/skill.js:1824-1848
        "mingkong" =>  { u05_mark_invoked(ctx, owner, "mingkong"); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:mingkong2", 1); },
        // character/gujian/skill.js:2082-2105
        "shahun2" =>  { u05_mark_invoked(ctx, owner, "shahun2"); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/gujian/skill.js:2106-2144
        "xuanning1" =>  { u05_mark_invoked(ctx, owner, "xuanning1"); u05_recover(ctx, owner, 1); },
        // character/gujian/skill.js:2145-2172
        "xuanning2" =>  { u05_mark_invoked(ctx, owner, "xuanning2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gujian/skill.js:2235-2270
        "yangming" =>  { u05_mark_invoked(ctx, owner, "yangming"); if !u05_check_usable(ctx, owner, "yangming", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/gujian/skill.js:2380-2387
        "jiehuo2" =>  { u05_mark_invoked(ctx, owner, "jiehuo2"); ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/gujian/skill.js:2388-2485
        "yuling" | "yuling1" | "yuling5" | "yuling6" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } set_flag(ctx.core, owner, "u05_mod:yuling2", 1); },
        // character/gwent/skill.js:5-40
        "sqlongyin" =>  { u05_mark_invoked(ctx, owner, "sqlongyin"); if !u05_pay_discard(ctx, owner, "sqlongyin", 1, true) { return true; } },
        // character/gwent/skill.js:149-210
        "jielue" =>  { u05_mark_invoked(ctx, owner, "jielue"); ctx.start_draw(owner, 1); let _ = u05_prey_other(ctx, owner, true); },
        // character/gwent/skill.js:211-222
        "gwmaoxian_hengsaite_sha" =>  { u05_mark_invoked(ctx, owner, "gwmaoxian_hengsaite_sha"); if !u05_pay_discard(ctx, owner, "gwmaoxian_hengsaite_sha", 1, true) { return true; } },
        // character/gwent/skill.js:223-280
        "gwhuanshuang" =>  { u05_mark_invoked(ctx, owner, "gwhuanshuang"); if !u05_pay_discard(ctx, owner, "gwhuanshuang", 2, true) { return true; } },
        // character/gwent/skill.js:341-376
        "gw_xianzumaijiu" =>  { u05_mark_invoked(ctx, owner, "gw_xianzumaijiu"); if !u05_pay_discard(ctx, owner, "gw_xianzumaijiu", 1, true) { return true; } ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/gwent/skill.js:417-468
        "gwliaotian" =>  { u05_mark_invoked(ctx, owner, "gwliaotian"); if !u05_check_usable(ctx, owner, "gwliaotian", 2) { return true; } if !u05_pay_discard(ctx, owner, "gwliaotian", 2, true) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/gwent/skill.js:705-765
        "gwjingshi" =>  { u05_mark_invoked(ctx, owner, "gwjingshi"); let _ = u05_prey_other(ctx, owner, true); },
        // character/gwent/skill.js:1043-1075
        "gwzhongmo" =>  { u05_mark_invoked(ctx, owner, "gwzhongmo"); if !u05_pay_discard(ctx, owner, "gwzhongmo", 2, true) { return true; } },
        // character/gwent/skill.js:1124-1133
        "g_gw_yewu" =>  { u05_mark_invoked(ctx, owner, "g_gw_yewu"); if !u05_pay_discard(ctx, owner, "g_gw_yewu", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:1134-1180
        "shuangxi" =>  { u05_mark_invoked(ctx, owner, "shuangxi"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/gwent/skill.js:1181-1200
        "yangfan" =>  { u05_mark_invoked(ctx, owner, "yangfan"); if !u05_check_usable(ctx, owner, "yangfan", 1) { return true; } if !u05_pay_discard(ctx, owner, "yangfan", 1, true) { return true; } },
        // character/gwent/skill.js:1201-1260
        "gwfengshi" =>  { u05_mark_invoked(ctx, owner, "gwfengshi"); if !u05_pay_discard(ctx, owner, "gwfengshi", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:1321-1370
        "gwliedi" =>  { u05_mark_invoked(ctx, owner, "gwliedi"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/gwent/skill.js:1455-1460
        "gwqinwu2" =>  { u05_mark_invoked(ctx, owner, "gwqinwu2"); if !u05_pay_discard(ctx, owner, "gwqinwu2", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:gwqinwu", 1); } },
        // character/gwent/skill.js:1461-1529
        "huanshu" | "huanshu3" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } if !u05_pay_discard(ctx, owner, skill, 2, true) { return true; } },
        // character/gwent/skill.js:1530-1580
        "gwjieyin" =>  { u05_mark_invoked(ctx, owner, "gwjieyin"); if !u05_pay_discard(ctx, owner, "gwjieyin", 1, true) { return true; } u05_recover(ctx, owner, 1); },
        // character/gwent/skill.js:1775-1834
        "gwjushi" | "gwjushi2" =>  { u05_mark_invoked(ctx, owner, skill); let _ = u05_prey_other(ctx, owner, true); },
        // character/gwent/skill.js:1835-1880
        "gwfengchi" =>  { u05_mark_invoked(ctx, owner, "gwfengchi"); if !u05_pay_discard(ctx, owner, "gwfengchi", 1, true) { return true; } },
        // character/gwent/skill.js:2235-2258
        "hupeng3" | "hupeng4" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/gwent/skill.js:2446-2516
        "gwtianbian" =>  { u05_mark_invoked(ctx, owner, "gwtianbian"); if !u05_pay_discard(ctx, owner, "gwtianbian", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/gwent/skill.js:2810-2844
        "fengjian" =>  { u05_mark_invoked(ctx, owner, "fengjian"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); set_flag(ctx.core, owner, "u05_mod:yunchuizhen", 1); },
        // character/gwent/skill.js:2902-2945
        "xuezhou" =>  { u05_mark_invoked(ctx, owner, "xuezhou"); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // character/gwent/skill.js:3088-3144
        "huihun" =>  { u05_mark_invoked(ctx, owner, "huihun"); if !u05_pay_discard(ctx, owner, "huihun", 2, true) { return true; } },
        _ => return false,
    }
    true
}
