//! Authoritative `U05 杀海拾遗` (`gujian` & `gwent`) host-side skill/card resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_gujian_gwent(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // card/gujian/skill.js:5-24
        "ziyangdan" => u05_exec(ctx, owner, "ziyangdan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, -1, U05Extra::None),
        // card/gujian/skill.js:25-60
        "luyugeng" => u05_exec(ctx, owner, "luyugeng", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // card/gujian/skill.js:98-104
        "xiajiao2" => { u05_mark_invoked(ctx, owner, "xiajiao2"); let _ = u05_discard_up_to(ctx, owner, 1); }
        // card/gujian/skill.js:106-150
        "mizhilianou" => u05_exec(ctx, owner, "mizhilianou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("tao")),
        // card/gujian/skill.js:151-177
        "chunbing" => u05_exec(ctx, owner, "chunbing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:chunbing", 1)),
        // card/gujian/skill.js:178-224
        "gudonggeng" => u05_exec(ctx, owner, "gudonggeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gudonggeng", 1)),
        // card/gujian/skill.js:225-260
        "qingtuan" => u05_exec(ctx, owner, "qingtuan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // card/gujian/skill.js:261-286
        "liyutang" => u05_exec(ctx, owner, "liyutang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // card/gujian/skill.js:287-312
        "yougeng" => u05_exec(ctx, owner, "yougeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // card/gujian/skill.js:313-342
        "molicha" => u05_exec(ctx, owner, "molicha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:molicha", 1)),
        // card/gujian/skill.js:343-371
        "yuanbaorou" => u05_exec(ctx, owner, "yuanbaorou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuanbaorou", 1)),
        // card/gujian/skill.js:372-403
        "tanhuadong" => u05_exec(ctx, owner, "tanhuadong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // card/gujian/skill.js:404-438
        "mapodoufu" => u05_exec(ctx, owner, "mapodoufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gujian/skill.js:439-455
        "yunvyuanshen_skill" => u05_exec(ctx, owner, "yunvyuanshen_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // card/gujian/skill.js:456-467
        "heilonglinpian" => u05_exec(ctx, owner, "heilonglinpian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:heilonglinpian", 1)),
        // card/gujian/skill.js:468-497
        "mutoumianju_skill" => u05_exec(ctx, owner, "mutoumianju_skill", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // card/gujian/skill.js:498-552
        "gjyuheng_skill" => u05_exec(ctx, owner, "gjyuheng_skill", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/gujian/skill.js:554-608
        "gjyuheng_plus_skill" => u05_exec(ctx, owner, "gjyuheng_plus_skill", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/gujian/skill.js:609-662
        "gjyuheng_pro_skill" => u05_exec(ctx, owner, "gjyuheng_pro_skill", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/gujian/skill.js:663-675
        "shihuifen" => u05_exec(ctx, owner, "shihuifen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:shihuifen", 1)),
        // character/gujian/skill.js:5-91
        "qingshu" => u05_exec(ctx, owner, "qingshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:qingshu")),
        // character/gujian/skill.js:92-129
        "yunyou" => u05_exec(ctx, owner, "yunyou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:229-267
        "xuanci" => u05_exec(ctx, owner, "xuanci", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("feibiao")),
        // character/gujian/skill.js:268-314
        "xianju" => u05_exec(ctx, owner, "xianju", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:315-397
        "humeng" => u05_exec(ctx, owner, "humeng", 0, true, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:398-416
        "xunjian" => u05_exec(ctx, owner, "xunjian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:417-480
        "lingyan" => u05_exec(ctx, owner, "lingyan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:717-802
        "cihong" => u05_exec(ctx, owner, "cihong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/gujian/skill.js:977-994
        "zuiji" => u05_exec(ctx, owner, "zuiji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jiu")),
        // character/gujian/skill.js:995-1010
        "manwu" => u05_exec(ctx, owner, "manwu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1011-1030
        "xfanghua" => u05_exec(ctx, owner, "xfanghua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gujian/skill.js:1031-1091
        "duhun" => u05_exec(ctx, owner, "duhun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/gujian/skill.js:1203-1236
        "shishui" => u05_exec(ctx, owner, "shishui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/gujian/skill.js:1237-1277
        "chizhen" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "chizhen", false) {
                u05_mark_invoked(ctx, owner, "chizhen");
                let num = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
                ctx.start_draw(owner, num);
                let _ = u05_discard_up_to(ctx, owner, num as usize);
            }
        }
        // character/gujian/skill.js:1278-1311
        "dangping" => u05_exec(ctx, owner, "dangping", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:dangping", 1)),
        // character/gujian/skill.js:1404-1555
        "yanjia" => u05_exec(ctx, owner, "yanjia", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1600-1623
        "zuizhan" => u05_exec(ctx, owner, "zuizhan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1624-1643
        "xidie" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "xidie", false) {
                u05_mark_invoked(ctx, owner, "xidie");
                let diff = ctx.core.player(owner).hand.len() as i32 - ctx.core.player(owner).hp;
                if diff > 0 {
                    let n = u05_discard_up_to(ctx, owner, (diff as usize).min(3));
                    set_flag(ctx.core, owner, "u05_storage:xidie", n as u32);
                }
            }
        }
        // character/gujian/skill.js:1644-1654
        "xidie2" => {
            let saved = flag(ctx.core, owner, "u05_storage:xidie");
            if saved > 0 {
                u05_mark_invoked(ctx, owner, "xidie2");
                set_flag(ctx.core, owner, "u05_storage:xidie", 0);
                ctx.start_draw(owner, saved);
            }
        }
        // character/gujian/skill.js:1689-1716
        "xlqianhuan" => u05_exec(ctx, owner, "xlqianhuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:1717-1752
        "fumo" => u05_exec(ctx, owner, "fumo", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1753-1823
        "fanyin" => u05_exec(ctx, owner, "fanyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:1852-1866
        "mingkong2" => u05_exec(ctx, owner, "mingkong2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1867-1875
        "gjqtwuxie" => u05_exec(ctx, owner, "gjqtwuxie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gjqtwuxie", 1)),
        // character/gujian/skill.js:1876-1898
        "yuehua" => u05_exec(ctx, owner, "yuehua", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1899-1947
        "qinglan" => u05_exec(ctx, owner, "qinglan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gujian/skill.js:1948-1963
        "fanshi" => u05_exec(ctx, owner, "fanshi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/gujian/skill.js:1964-2031
        "xuelu" => u05_exec(ctx, owner, "xuelu", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None),
        // character/gujian/skill.js:2032-2081
        "shahun" => u05_exec(ctx, owner, "shahun", 0, true, 0, 3, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:2097-2105
        "xuanning" => {
            u05_mark_invoked(ctx, owner, "xuanning");
            set_flag(ctx.core, owner, "xianjian_skill:xuanning", 1);
            set_flag(ctx.core, owner, "xianjian_ward_active", 1);
            let _ = u05_store_pile(ctx, owner, "xuanning");
            u05_recover(ctx, owner, 1);
        }
        // character/gujian/skill.js:2170-2234
        "liuguang" => u05_exec(ctx, owner, "liuguang", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:2267-2305
        "yangming2" => u05_exec(ctx, owner, "yangming2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gujian/skill.js:2306-2344
        "zhaolu" => u05_exec(ctx, owner, "zhaolu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/gujian/skill.js:2345-2379
        "jiehuo" => u05_exec(ctx, owner, "jiehuo", 0, true, 0, 0, 0, 2, DamageNature::Fire, 0, 0, U05Extra::None),
        // character/gujian/skill.js:2431-2444
        "yuling2" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "yuling2", false) {
                let h = ctx.core.player(owner).hand.len();
                if h < 5 {
                    u05_mark_invoked(ctx, owner, "yuling2");
                    ctx.start_draw(owner, (5 - h) as u32);
                }
            }
        }
        // character/gujian/skill.js:2445-2458
        "yuling3" => {
            let h = ctx.core.player(owner).hand.len();
            if h > 5 {
                u05_mark_invoked(ctx, owner, "yuling3");
                let _ = u05_discard_up_to(ctx, owner, h - 5);
            }
        }
        // character/gujian/skill.js:2459-2471
        "yuling4" => u05_exec(ctx, owner, "yuling4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuling4", 1)),
        // card/gwent/skill.js:5-35
        "gw_aerdeyin" => u05_exec(ctx, owner, "gw_aerdeyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // card/gwent/skill.js:80-122
        "gw_baobaoshu" => { u05_mark_invoked(ctx, owner, "gw_baobaoshu"); let _ = u05_discard_up_to(ctx, owner, 1); }
        // card/gwent/skill.js:173-197
        "gw_qinpendayu" => u05_exec(ctx, owner, "gw_qinpendayu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gw_qinpendayu", 1)),
        // card/gwent/skill.js:198-224
        "gw_birinongwu" => u05_exec(ctx, owner, "gw_birinongwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gw_birinongwu", 1)),
        // card/gwent/skill.js:299-335
        "gw_dieyi" => u05_exec(ctx, owner, "gw_dieyi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // card/gwent/skill.js:336-389
        "_gainspell" => u05_exec(ctx, owner, "_gainspell", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:31-73
        "sqlongnu" => u05_exec(ctx, owner, "sqlongnu", 0, false, 0, 1, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None),
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
        "gwjinli" => u05_exec(ctx, owner, "gwjinli", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwjinli", 1)),
        // character/gwent/skill.js:450-499
        "gwhuanbi" => u05_exec(ctx, owner, "gwhuanbi", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:500-575
        "gwminxiang" => u05_exec(ctx, owner, "gwminxiang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:576-605
        "gwlangshi" => u05_exec(ctx, owner, "gwlangshi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:606-703
        "gwjingtian" => u05_exec(ctx, owner, "gwjingtian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:794-861
        "gwweitu" => u05_exec(ctx, owner, "gwweitu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/gwent/skill.js:862-962
        "gwgouhun" => u05_exec(ctx, owner, "gwgouhun", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:963-1042
        "gwfutian" => u05_exec(ctx, owner, "gwfutian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:1062-1123
        "gwyewu" => u05_exec(ctx, owner, "gwyewu", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:1243-1320
        "gwchenshui" => u05_exec(ctx, owner, "gwchenshui", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
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
        "gwfusheng" => u05_exec(ctx, owner, "gwfusheng", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverOther),
        // character/gwent/skill.js:1414-1454
        "gwqinwu" => u05_exec(ctx, owner, "gwqinwu", 1, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwqinwu", 1)),
        // character/gwent/skill.js:1485-1520
        "huanshu2" => u05_exec(ctx, owner, "huanshu2", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1636-1700
        "zhengjun" => u05_exec(ctx, owner, "zhengjun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/gwent/skill.js:1701-1710
        "gwxuezhan" => u05_exec(ctx, owner, "gwxuezhan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1711-1774
        "bolang" => u05_exec(ctx, owner, "bolang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1895-1940
        "lingji" => u05_exec(ctx, owner, "lingji", 1, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1941-1990
        "gwjinyan" => u05_exec(ctx, owner, "gwjinyan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1991-2091
        "gwshenyu" => u05_exec(ctx, owner, "gwshenyu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gwent/skill.js:2092-2130
        "junchi" => u05_exec(ctx, owner, "junchi", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2131-2196
        "hupeng" => u05_exec(ctx, owner, "hupeng", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2197-2234
        "hupeng2" => u05_exec(ctx, owner, "hupeng2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hupeng2", 1)),
        // character/gwent/skill.js:2275-2353
        "hunmo" => u05_exec(ctx, owner, "hunmo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:2355-2392
        "shuijian" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "shuijian", false) && u05_pay_discard(ctx, owner, "shuijian", 1, false) {
                u05_mark_invoked(ctx, owner, "shuijian");
                u05_damage_aoe(ctx, owner, 1, DamageNature::Normal);
            }
        }
        // character/gwent/skill.js:2393-2417
        "yunhuo" => u05_exec(ctx, owner, "yunhuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2418-2445
        "yinzhang" => u05_exec(ctx, owner, "yinzhang", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2517-2539
        "gwxiaoshou" => u05_exec(ctx, owner, "gwxiaoshou", 2, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2540-2556
        "kuanglie" => u05_exec(ctx, owner, "kuanglie", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2558-2612
        "gwjiquan" => u05_exec(ctx, owner, "gwjiquan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gwent/skill.js:2613-2660
        "nuhou" => u05_exec(ctx, owner, "nuhou", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2661-2711
        "shewu" => u05_exec(ctx, owner, "shewu", 1, false, 1, 3, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:shewu", 1)),
        // character/gwent/skill.js:2712-2771
        "gwzhanjiang" => u05_exec(ctx, owner, "gwzhanjiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2773-2783
        "gwzhanjiang3" => u05_exec(ctx, owner, "gwzhanjiang3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2784-2809
        "gwchuanxin" => u05_exec(ctx, owner, "gwchuanxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gwchuanxin", 1)),
        // character/gwent/skill.js:2845-2901
        "huandie" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "huandie", false) {
                u05_mark_invoked(ctx, owner, "huandie");
                u05_toggle_link(ctx, owner);
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    u05_toggle_link(ctx, t);
                }
                ctx.start_draw(owner, 1);
            }
        }
        // character/gwent/skill.js:2946-2982
        "xuezhou_hp" => u05_exec(ctx, owner, "xuezhou_hp", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)),
        // character/gwent/skill.js:2984-3029
        "fayin" => u05_exec(ctx, owner, "fayin", 0, false, 0, 0, 1, 1, DamageNature::Fire, 0, 1, U05Extra::TurnOverOther),
        // character/gwent/skill.js:3030-3087
        "gwbaquan" => u05_exec(ctx, owner, "gwbaquan", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:3145-3197
        "lanquan" => u05_exec(ctx, owner, "lanquan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // card/gwent/skill.js:123-148
        "gw_nuhaifengbao" => u05_exec(ctx, owner, "gw_nuhaifengbao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gwent/skill.js:149-192
        "gw_youer" => u05_exec(ctx, owner, "gw_youer", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/gwent/skill.js:270-310
        "gw_leizhoushu" => u05_exec(ctx, owner, "gw_leizhoushu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:130-185
        "xuanzhen" => u05_exec(ctx, owner, "xuanzhen", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:481-501
        "woxue" => u05_exec(ctx, owner, "woxue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
        // character/gujian/skill.js:502-602
        "yange" => u05_exec(ctx, owner, "yange", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:603-678
        "lianjing" => u05_exec(ctx, owner, "lianjing", 0, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ExtraTurn),
        // character/gujian/skill.js:803-869
        "zhenying" => u05_exec(ctx, owner, "zhenying", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:870-926
        "lingnu" => u05_exec(ctx, owner, "lingnu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/gujian/skill.js:1092-1168
        "yunyin" => u05_exec(ctx, owner, "yunyin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/gujian/skill.js:1313-1345
        "xiuhua" => u05_exec(ctx, owner, "xiuhua", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:1346-1388
        "liuying" => u05_exec(ctx, owner, "liuying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:liuying")),
        // character/gujian/skill.js:1556-1589
        "meiying" => u05_exec(ctx, owner, "meiying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/gujian/skill.js:1590-1610
        "jianwu" => u05_exec(ctx, owner, "jianwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jianwu", 1)),
        // character/gujian/skill.js:1655-1695
        "meihu" => u05_exec(ctx, owner, "meihu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gujian/skill.js:1824-1848
        "mingkong" => u05_exec(ctx, owner, "mingkong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mingkong2", 1)),
        // character/gujian/skill.js:2082-2105
        "shahun2" => u05_exec(ctx, owner, "shahun2", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:2106-2144
        "xuanning1" => u05_exec(ctx, owner, "xuanning1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gujian/skill.js:2145-2172
        "xuanning2" => u05_exec(ctx, owner, "xuanning2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:2235-2270
        "yangming" => u05_exec(ctx, owner, "yangming", 1, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:2380-2387
        "jiehuo2" => u05_exec(ctx, owner, "jiehuo2", 0, false, 0, 1, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gujian/skill.js:2388-2485
        "yuling" | "yuling1" | "yuling5" | "yuling6" => u05_exec(ctx, owner, skill, 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuling2", 1)),
        // character/gwent/skill.js:5-40
        "sqlongyin" => u05_exec(ctx, owner, "sqlongyin", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:149-210
        "jielue" => u05_exec(ctx, owner, "jielue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gwent/skill.js:211-222
        "gwmaoxian_hengsaite_sha" => u05_exec(ctx, owner, "gwmaoxian_hengsaite_sha", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:223-280
        "gwhuanshuang" => u05_exec(ctx, owner, "gwhuanshuang", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:341-376
        "gw_xianzumaijiu" => u05_exec(ctx, owner, "gw_xianzumaijiu", 0, false, 1, 1, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:417-468
        "gwliaotian" => u05_exec(ctx, owner, "gwliaotian", 2, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/gwent/skill.js:705-765
        "gwjingshi" => u05_exec(ctx, owner, "gwjingshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gwent/skill.js:1043-1075
        "gwzhongmo" => u05_exec(ctx, owner, "gwzhongmo", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1124-1133
        "g_gw_yewu" => u05_exec(ctx, owner, "g_gw_yewu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:1134-1180
        "shuangxi" => u05_exec(ctx, owner, "shuangxi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
        // character/gwent/skill.js:1181-1200
        "yangfan" => u05_exec(ctx, owner, "yangfan", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1201-1260
        "gwfengshi" => u05_exec(ctx, owner, "gwfengshi", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:1321-1370
        "gwliedi" => u05_exec(ctx, owner, "gwliedi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/gwent/skill.js:1455-1460
        "gwqinwu2" => u05_exec(ctx, owner, "gwqinwu2", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwqinwu", 1)),
        // character/gwent/skill.js:1461-1529
        "huanshu" | "huanshu3" => u05_exec(ctx, owner, skill, 1, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:1530-1580
        "gwjieyin" => u05_exec(ctx, owner, "gwjieyin", 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/gwent/skill.js:1775-1834
        "gwjushi" | "gwjushi2" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gwent/skill.js:1835-1880
        "gwfengchi" => u05_exec(ctx, owner, "gwfengchi", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2235-2258
        "hupeng3" | "hupeng4" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:2446-2516
        "gwtianbian" => u05_exec(ctx, owner, "gwtianbian", 0, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/gwent/skill.js:2810-2844
        "fengjian" => u05_exec(ctx, owner, "fengjian", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::ModFlag("u05_mod:yunchuizhen", 1)),
        // character/gwent/skill.js:2902-2945
        "xuezhou" => u05_exec(ctx, owner, "xuezhou", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/gwent/skill.js:3088-3144
        "huihun" => u05_exec(ctx, owner, "huihun", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        _ => return false,
    }
    true
}
