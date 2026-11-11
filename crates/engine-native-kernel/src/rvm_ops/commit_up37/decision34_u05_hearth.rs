//! Authoritative `U05 杀海拾遗` (`hearth` & `stone_chenshui`) host-side skill/card resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_hearth(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // main/content.js:397-410: stone_chenshui (Ysera end-phase dream card grant)
        "stone_chenshui" | "stone_yisela_stonecharacter" => {
            u05_mark_invoked(ctx, owner, skill);
            let idx = (flag(ctx.core, owner, "u05_ysera_dream_idx") + 1) % 5;
            set_flag(ctx.core, owner, "u05_ysera_dream_idx", idx);
            set_flag(ctx.core, owner, "u05_ysera_dream_granted", 1);
            ctx.start_draw(owner, 1);
        }
        // card/hearth/skill.js:7-13
        "shihuawuqi" =>  { u05_mark_invoked(ctx, owner, "shihuawuqi"); set_flag(ctx.core, owner, "u05_mod:shihuawuqi", 1); },
        // card/hearth/skill.js:14-20
        "jihuocard2" =>  { u05_mark_invoked(ctx, owner, "jihuocard2"); set_flag(ctx.core, owner, "u05_mod:jihuocard2", 1); },
        // character/hearth/skill.js:5-56
        "hshuanyu" =>  { u05_mark_invoked(ctx, owner, "hshuanyu"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:57-130
        "wxuying" =>  { u05_mark_invoked(ctx, owner, "wxuying"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:145-154
        "yibian" =>  { u05_mark_invoked(ctx, owner, "yibian"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:481-561
        "asyouzhang" =>  { u05_mark_invoked(ctx, owner, "asyouzhang"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:562-678
        "ylyuchu" =>  { u05_mark_invoked(ctx, owner, "ylyuchu"); let _ = u05_store_pile(ctx, owner, "u05_pile:ylyuchu"); },
        // character/hearth/skill.js:679-709
        "nsaiqi" =>  { u05_mark_invoked(ctx, owner, "nsaiqi"); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:nsaiqi", 1); },
        // character/hearth/skill.js:710-729
        "nsbeiming" =>  { u05_mark_invoked(ctx, owner, "nsbeiming"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:730-902
        "hsnitai" =>  { u05_mark_invoked(ctx, owner, "hsnitai"); ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:hsnitai", 1); } },
        // character/hearth/skill.js:903-938
        "hspuzhao" =>  { u05_mark_invoked(ctx, owner, "hspuzhao"); if !u05_check_usable(ctx, owner, "hspuzhao", 1) { return true; } if !u05_pay_discard(ctx, owner, "hspuzhao", 1, true) { return true; } ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:939-973
        "hsyanxin" =>  { u05_mark_invoked(ctx, owner, "hsyanxin"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:974-1106
        "hstianqi" =>  { u05_mark_invoked(ctx, owner, "hstianqi"); if !u05_check_usable(ctx, owner, "hstianqi", 1) { return true; } ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:1107-1116
        "hstianqi_dalian" =>  { u05_mark_invoked(ctx, owner, "hstianqi_dalian"); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:1117-1126
        "hstianqi_shali" => { u05_mark_invoked(ctx, owner, "hstianqi_shali"); u05_change_hujia(ctx, owner, 1); }
        // character/hearth/skill.js:1127-1279
        "ysjqisha" =>  { u05_mark_invoked(ctx, owner, "ysjqisha"); u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:1280-1320
        "yindan" =>  { u05_mark_invoked(ctx, owner, "yindan"); if !u05_check_usable(ctx, owner, "yindan", 1) { return true; } if !u05_pay_discard(ctx, owner, "yindan", 1, true) { return true; } u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:1321-1391
        "hllingxi" =>  { u05_mark_invoked(ctx, owner, "hllingxi"); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:1424-1453
        "xiyong" =>  { u05_mark_invoked(ctx, owner, "xiyong"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1454-1466
        "srjici" =>  { u05_mark_invoked(ctx, owner, "srjici"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1467-1481
        "yinzong" =>  { u05_mark_invoked(ctx, owner, "yinzong"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1482-1519
        "tansuo" =>  { u05_mark_invoked(ctx, owner, "tansuo"); if !u05_check_usable(ctx, owner, "tansuo", 1) { return true; } if !u05_pay_discard(ctx, owner, "tansuo", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1520-1591
        "lieqi" =>  { u05_mark_invoked(ctx, owner, "lieqi"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1677-1704
        "shouwang" =>  { u05_mark_invoked(ctx, owner, "shouwang"); u05_change_hujia(ctx, owner, 1); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:shouwang", 1); } },
        // character/hearth/skill.js:1747-1814
        "qianfu2" =>  { u05_mark_invoked(ctx, owner, "qianfu2"); ctx.start_draw(owner, 3); set_flag(ctx.core, owner, "u05_mod:qianfu2", 1); },
        // character/hearth/skill.js:1815-1828
        "shimo" =>  { u05_mark_invoked(ctx, owner, "shimo"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1829-1846
        "lieyang" =>  { u05_mark_invoked(ctx, owner, "lieyang"); if !u05_check_usable(ctx, owner, "lieyang", 3) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1847-1855
        "lieyang2" =>  { u05_mark_invoked(ctx, owner, "lieyang2"); set_flag(ctx.core, owner, "u05_mod:lieyang2", 1); },
        // character/hearth/skill.js:1856-1892
        "zhuilie" =>  { u05_mark_invoked(ctx, owner, "zhuilie"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:1927-1963
        "kekao" =>  { u05_mark_invoked(ctx, owner, "kekao"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:2028-2082
        "kqizhou" =>  { u05_mark_invoked(ctx, owner, "kqizhou"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:2083-2109
        "jingcu" =>  { u05_mark_invoked(ctx, owner, "jingcu"); ctx.start_draw(owner, 2); u05_change_max_hp(ctx, owner, -(1)); },
        // character/hearth/skill.js:2110-2119
        "shengzhang" =>  { u05_mark_invoked(ctx, owner, "shengzhang"); u05_change_max_hp(ctx, owner, 1); },
        // character/hearth/skill.js:2120-2164
        "pyuhuo" =>  { u05_mark_invoked(ctx, owner, "pyuhuo"); ctx.start_draw(owner, 4); u05_turn_over(ctx, owner); },
        // character/hearth/skill.js:2165-2209
        "mengye" =>  { u05_mark_invoked(ctx, owner, "mengye"); u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:2251-2265
        "lianzhan" =>  { u05_mark_invoked(ctx, owner, "lianzhan"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 2); u05_change_max_hp(ctx, owner, 1); },
        // character/hearth/skill.js:2412-2448
        "moxie" =>  { u05_mark_invoked(ctx, owner, "moxie"); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:2514-2522
        "hlongyi" =>  { u05_mark_invoked(ctx, owner, "hlongyi"); set_flag(ctx.core, owner, "u05_mod:hlongyi", 1); },
        // character/hearth/skill.js:2523-2549
        "zhongji" => {
            if u05_pay_discard(ctx, owner, "zhongji", 1, false) {
                u05_mark_invoked(ctx, owner, "zhongji");
                set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1);
            }
        }
        // character/hearth/skill.js:2550-2566
        "fuwen" =>  { u05_mark_invoked(ctx, owner, "fuwen"); u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:2604-2629
        "midian" =>  { u05_mark_invoked(ctx, owner, "midian"); if !u05_check_usable(ctx, owner, "midian", 1) { return true; } if !u05_pay_discard(ctx, owner, "midian", 1, true) { return true; } ctx.start_draw(owner, 3); },
        // character/hearth/skill.js:2630-2683
        "xingluo" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "xingluo", false) {
                let my_h = ctx.core.player(owner).hand.len();
                let richer: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
                    .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).hand.len() > my_h)
                    .collect();
                if !richer.is_empty() {
                    u05_mark_invoked(ctx, owner, "xingluo");
                    for t in richer {
                        let _ = u05_discard_up_to(ctx, t, 1);
                    }
                }
            }
        }
        // character/hearth/skill.js:2684-2715
        "yuelu" =>  { u05_mark_invoked(ctx, owner, "yuelu"); u05_change_hujia(ctx, owner, 1); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:2716-2796
        "yushou" =>  { u05_mark_invoked(ctx, owner, "yushou"); if !u05_pay_discard(ctx, owner, "yushou", 1, true) { return true; } u05_change_hujia(ctx, owner, 1); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/hearth/skill.js:2813-2834
        "yushou_misha" =>  { u05_mark_invoked(ctx, owner, "yushou_misha"); u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:2835-2853
        "yushou_huofu" =>  { u05_mark_invoked(ctx, owner, "yushou_huofu"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "juedou"), 1); },
        // character/hearth/skill.js:2862-2923
        "qingzun" =>  { u05_mark_invoked(ctx, owner, "qingzun"); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:qingzun", 1); },
        // character/hearth/skill.js:2924-2956
        "ayuling" =>  { u05_mark_invoked(ctx, owner, "ayuling"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:2957-2978
        "aoshu" =>  { u05_mark_invoked(ctx, owner, "aoshu"); if !u05_check_usable(ctx, owner, "aoshu", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "wuzhong"), 1); },
        // character/hearth/skill.js:2979-2995
        "bzhuiji" =>  { u05_mark_invoked(ctx, owner, "bzhuiji"); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:3187-3224
        "shouji" =>  { u05_mark_invoked(ctx, owner, "shouji"); if !u05_check_usable(ctx, owner, "shouji", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3370-3380
        "shengteng" =>  { u05_mark_invoked(ctx, owner, "shengteng"); u05_recover(ctx, owner, 1); u05_change_max_hp(ctx, owner, 1); },
        // character/hearth/skill.js:3401-3428
        "chouhuo" =>  { u05_mark_invoked(ctx, owner, "chouhuo"); if !u05_check_limited(ctx, owner, "chouhuo") { return true; } u05_change_hujia(ctx, owner, 2); u05_change_max_hp(ctx, owner, -(1)); },
        // character/hearth/skill.js:3429-3512
        "nuyan2" =>  { u05_mark_invoked(ctx, owner, "nuyan2"); if !u05_check_usable(ctx, owner, "nuyan2", 3) { return true; } u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:3603-3625
        "duxin" =>  { u05_mark_invoked(ctx, owner, "duxin"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3626-3645
        "oldduxin" =>  { u05_mark_invoked(ctx, owner, "oldduxin"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3730-3771
        "mobao" =>  { u05_mark_invoked(ctx, owner, "mobao"); if !u05_check_usable(ctx, owner, "mobao", 1) { return true; } if !u05_pay_discard(ctx, owner, "mobao", 1, true) { return true; } u05_damage_aoe(ctx, owner, 1, DamageNature::Thunder); },
        // character/hearth/skill.js:3845-3858
        "xianji3" =>  { u05_mark_invoked(ctx, owner, "xianji3"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3859-3885
        "tanmi" =>  { u05_mark_invoked(ctx, owner, "tanmi"); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:3886-3914
        "xueren" =>  { u05_mark_invoked(ctx, owner, "xueren"); ctx.start_draw(owner, 2); u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:3988-4001
        "yiwen" =>  { u05_mark_invoked(ctx, owner, "yiwen"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4058-4087
        "zhuizong" =>  { u05_mark_invoked(ctx, owner, "zhuizong"); if !u05_check_usable(ctx, owner, "zhuizong", 1) { return true; } if !u05_pay_discard(ctx, owner, "zhuizong", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4153-4161
        "hsbaowu_cangbaotu" =>  { u05_mark_invoked(ctx, owner, "hsbaowu_cangbaotu"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4162-4176
        "hsbaowu_huangjinyuanhou" =>  { u05_mark_invoked(ctx, owner, "hsbaowu_huangjinyuanhou"); set_flag(ctx.core, owner, "u05_mod:hsbaowu_huangjinyuanhou", 1); },
        // character/hearth/skill.js:4185-4208
        "xieneng" =>  { u05_mark_invoked(ctx, owner, "xieneng"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4209-4262
        "fbeifa" =>  { u05_mark_invoked(ctx, owner, "fbeifa"); if !u05_check_usable(ctx, owner, "fbeifa", 3) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4263-4308
        "oldfbeifa" =>  { u05_mark_invoked(ctx, owner, "oldfbeifa"); if !u05_check_usable(ctx, owner, "oldfbeifa", 3) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4309-4334
        "yufa" =>  { u05_mark_invoked(ctx, owner, "yufa"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4396-4441
        "shifa" =>  { u05_mark_invoked(ctx, owner, "shifa"); if !u05_check_usable(ctx, owner, "shifa", 3) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4442-4486
        "oldshifa" =>  { u05_mark_invoked(ctx, owner, "oldshifa"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4487-4496
        "yuanzheng" =>  { u05_mark_invoked(ctx, owner, "yuanzheng"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:4497-4535
        "yulu" =>  { u05_mark_invoked(ctx, owner, "yulu"); if !u05_check_usable(ctx, owner, "yulu", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:4536-4574
        "oldyulu" =>  { u05_mark_invoked(ctx, owner, "oldyulu"); if !u05_check_usable(ctx, owner, "oldyulu", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:4601-4614
        "duzhang2" =>  { u05_mark_invoked(ctx, owner, "duzhang2"); set_flag(ctx.core, owner, "u05_mod:duzhang2", 1); },
        // character/hearth/skill.js:4615-4673
        "hannu" => {
            let nh = ctx.core.player(owner).hand.len();
            if nh > 0 {
                u05_mark_invoked(ctx, owner, "hannu");
                ctx.start_draw(owner, nh as u32);
            }
        }
        // character/hearth/skill.js:4674-4683
        "chuidiao" => {
            if guard_phase_auto_trigger(ctx, owner, "u05", "chuidiao", false) {
                u05_mark_invoked(ctx, owner, "chuidiao");
                let num = ctx.core.rng.gen_range(3);
                if num > 0 {
                    ctx.start_draw(owner, num);
                }
            }
        }
        // character/hearth/skill.js:4684-4715
        "hhudun" =>  { u05_mark_invoked(ctx, owner, "hhudun"); u05_change_hujia(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:4829-4876
        "xiaorong" =>  { u05_mark_invoked(ctx, owner, "xiaorong"); u05_recover(ctx, owner, 1); set_flag(ctx.core, owner, "u05_mod:xiaorong", 1); },
        // character/hearth/skill.js:4916-4930
        "jixuan" =>  { u05_mark_invoked(ctx, owner, "jixuan"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5084-5091
        "bingdong" =>  { u05_mark_invoked(ctx, owner, "bingdong"); if !u05_check_usable(ctx, owner, "bingdong", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5146-5182
        "fushi" =>  { u05_mark_invoked(ctx, owner, "fushi"); if !u05_check_usable(ctx, owner, "fushi", 1) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { u05_change_max_hp(ctx, t, -(1)); } },
        // character/hearth/skill.js:5183-5218
        "oldfushi" =>  { u05_mark_invoked(ctx, owner, "oldfushi"); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { u05_change_max_hp(ctx, t, -(1)); } },
        // character/hearth/skill.js:5219-5229
        "moyao" =>  { u05_mark_invoked(ctx, owner, "moyao"); set_flag(ctx.core, owner, "u05_mod:moyao", 1); },
        // character/hearth/skill.js:5258-5270
        "bimeng" =>  { u05_mark_invoked(ctx, owner, "bimeng"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5271-5284
        "zhoujiang" =>  { u05_mark_invoked(ctx, owner, "zhoujiang"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5312-5321
        "xjumo" =>  { u05_mark_invoked(ctx, owner, "xjumo"); set_flag(ctx.core, owner, "u05_mod:xjumo", 1); },
        // character/hearth/skill.js:5322-5345
        "malymowang" =>  { u05_mark_invoked(ctx, owner, "malymowang"); if !u05_check_usable(ctx, owner, "malymowang", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5346-5368
        "oldmalymowang" =>  { u05_mark_invoked(ctx, owner, "oldmalymowang"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5403-5420
        "mieshi" =>  { u05_mark_invoked(ctx, owner, "mieshi"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:5444-5459
        "xshixin" =>  { u05_mark_invoked(ctx, owner, "xshixin"); u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:5460-5513
        "enze" =>  { u05_mark_invoked(ctx, owner, "enze"); if !u05_check_usable(ctx, owner, "enze", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:5514-5561
        "oldenze" =>  { u05_mark_invoked(ctx, owner, "oldenze"); if !u05_check_usable(ctx, owner, "oldenze", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:5562-5623
        "chongsheng" =>  { u05_mark_invoked(ctx, owner, "chongsheng"); u05_turn_over(ctx, owner); },
        // character/hearth/skill.js:5624-5646
        "guozai" => {
            let h = ctx.core.player(owner).hand.len();
            if h < 4 && u05_check_usable(ctx, owner, "guozai", 1) {
                u05_mark_invoked(ctx, owner, "guozai");
                let d = (4 - h) as u32;
                ctx.start_draw(owner, d);
                let prev = flag(ctx.core, owner, "u05_storage:guozai2");
                set_flag(ctx.core, owner, "u05_storage:guozai2", prev + d);
            }
        }
        // character/hearth/skill.js:5647-5659
        "guozai2" => {
            let debt = flag(ctx.core, owner, "u05_storage:guozai2") as usize;
            if debt > 0 {
                u05_mark_invoked(ctx, owner, "guozai2");
                set_flag(ctx.core, owner, "u05_storage:guozai2", 0);
                let _ = u05_discard_up_to(ctx, owner, debt);
            }
        }
        // character/hearth/skill.js:5660-5682
        "guozaix" => {
            let h = ctx.core.player(owner).hand.len();
            if h < 4 && u05_check_usable(ctx, owner, "guozaix", 2) {
                u05_mark_invoked(ctx, owner, "guozaix");
                let d = (4 - h) as u32;
                ctx.start_draw(owner, d);
                let prev = flag(ctx.core, owner, "u05_storage:guozaix2");
                set_flag(ctx.core, owner, "u05_storage:guozaix2", prev + d);
            }
        }
        // character/hearth/skill.js:5683-5695
        "guozaix2" => {
            let debt = flag(ctx.core, owner, "u05_storage:guozaix2") as usize;
            if debt > 0 {
                u05_mark_invoked(ctx, owner, "guozaix2");
                set_flag(ctx.core, owner, "u05_storage:guozaix2", 0);
                let _ = u05_discard_up_to(ctx, owner, debt);
            }
        }
        // character/hearth/skill.js:5696-5719
        "hanshuang" =>  { u05_mark_invoked(ctx, owner, "hanshuang"); u05_lose_hp(ctx, owner, 1); },
        // character/hearth/skill.js:5720-5742
        "oldhanshuang" =>  { u05_mark_invoked(ctx, owner, "oldhanshuang"); u05_turn_over(ctx, owner); },
        // character/hearth/skill.js:5747-5767
        "bingshi2" =>  { u05_mark_invoked(ctx, owner, "bingshi2"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); },
        // character/hearth/skill.js:5768-5799
        "huanwu" =>  { u05_mark_invoked(ctx, owner, "huanwu"); if !u05_check_usable(ctx, owner, "huanwu", 1) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 2); } u05_change_max_hp(ctx, owner, 1); },
        // character/hearth/skill.js:5800-5824
        "fengnu" =>  { u05_mark_invoked(ctx, owner, "fengnu"); if !u05_check_usable(ctx, owner, "fengnu", 3) { return true; } ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:fengnu", 1); },
        // character/hearth/skill.js:5825-5835
        "shengdun" =>  { u05_mark_invoked(ctx, owner, "shengdun"); u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:5836-5854
        "jingmeng" =>  { u05_mark_invoked(ctx, owner, "jingmeng"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5855-5867
        "hswuji" => {
            let used = ctx.core.turn.cards_used_play_phase;
            if used > 0 && guard_phase_auto_trigger(ctx, owner, "u05", "hswuji", false) {
                u05_mark_invoked(ctx, owner, "hswuji");
                ctx.start_draw(owner, used);
            }
        }
        // character/hearth/skill.js:5868-5890
        "bingshuang" =>  { u05_mark_invoked(ctx, owner, "bingshuang"); ctx.start_draw(owner, 2); u05_turn_over(ctx, owner); },
        // character/hearth/skill.js:5891-5909
        "yanshu" =>  { u05_mark_invoked(ctx, owner, "yanshu"); if !u05_check_usable(ctx, owner, "yanshu", 1) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5910-5927
        "oldyanshu" =>  { u05_mark_invoked(ctx, owner, "oldyanshu"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:5928-5944
        "shengyan" =>  { u05_mark_invoked(ctx, owner, "shengyan"); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:5946-5963
        "liechao" =>  { u05_mark_invoked(ctx, owner, "liechao"); if !u05_check_usable(ctx, owner, "liechao", 1) { return true; } ctx.start_draw(owner, 4); u05_turn_over(ctx, owner); },
        // character/hearth/skill.js:5984-6016
        "zhongjia" =>  { u05_mark_invoked(ctx, owner, "zhongjia"); u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:6017-6047
        "dunji" =>  { u05_mark_invoked(ctx, owner, "dunji"); if !u05_check_usable(ctx, owner, "dunji", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/hearth/skill.js:6048-6071
        "fengxing" =>  { u05_mark_invoked(ctx, owner, "fengxing"); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:6073-6103
        "fengxian" =>  { u05_mark_invoked(ctx, owner, "fengxian"); if !u05_check_usable(ctx, owner, "fengxian", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:6120-6140
        "zhanhou" =>  { u05_mark_invoked(ctx, owner, "zhanhou"); if !u05_check_usable(ctx, owner, "zhanhou", 1) { return true; } if !u05_pay_discard(ctx, owner, "zhanhou", 1, true) { return true; } u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:6141-6163
        "oldzhanhou" =>  { u05_mark_invoked(ctx, owner, "oldzhanhou"); if !u05_pay_discard(ctx, owner, "oldzhanhou", 1, true) { return true; } u05_change_hujia(ctx, owner, 1); },
        // character/hearth/skill.js:6164-6193
        "shijie" =>  { u05_mark_invoked(ctx, owner, "shijie"); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/hearth/skill.js:6194-6240
        "shengguang" =>  { u05_mark_invoked(ctx, owner, "shengguang"); if !u05_check_usable(ctx, owner, "shengguang", 1) { return true; } if !u05_pay_discard(ctx, owner, "shengguang", 1, true) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // character/hearth/skill.js:6241-6268
        "xinci" =>  { u05_mark_invoked(ctx, owner, "xinci"); if !u05_check_usable(ctx, owner, "xinci", 1) { return true; } if !u05_pay_discard(ctx, owner, "xinci", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // character/hearth/skill.js:6269-6303
        "anying" =>  { u05_mark_invoked(ctx, owner, "anying"); if !u05_check_limited(ctx, owner, "anying") { return true; } if !u05_pay_discard(ctx, owner, "anying", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/hearth/skill.js:6304-6423
        "huanjue" =>  { u05_mark_invoked(ctx, owner, "huanjue"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:6424-6448
        "bingjia" =>  { u05_mark_invoked(ctx, owner, "bingjia"); set_flag(ctx.core, owner, "u05_buff:bingjia", 1); },
        // character/hearth/skill.js:6449-6482
        "bingjia2" =>  { u05_mark_invoked(ctx, owner, "bingjia2"); u05_change_hujia(ctx, owner, 1); set_flag(ctx.core, owner, "u05_buff:bingjia2", 1); },
        // character/hearth/skill.js:6592-6636
        "xianzhi" =>  { u05_mark_invoked(ctx, owner, "xianzhi"); ctx.start_draw(owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:xianzhi", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // character/hearth/skill.js:6703-6722
        "wlianji" =>  { u05_mark_invoked(ctx, owner, "wlianji"); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:6779-6794
        "mengun2" =>  { u05_mark_invoked(ctx, owner, "mengun2"); set_flag(ctx.core, owner, "u05_mod:mengun2", 1); },
        // character/hearth/skill.js:6795-6821
        "jianren" =>  { u05_mark_invoked(ctx, owner, "jianren"); if !u05_check_usable(ctx, owner, "jianren", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/hearth/skill.js:7031-7058
        "zuling" =>  { u05_mark_invoked(ctx, owner, "zuling"); if !u05_check_limited(ctx, owner, "zuling") { return true; } u05_change_max_hp(ctx, owner, -(1)); },
        // character/hearth/skill.js:7104-7110
        "tuteng_h" =>  { u05_mark_invoked(ctx, owner, "tuteng_h"); set_flag(ctx.core, owner, "u05_mod:tuteng_h", 1); },
        // character/hearth/skill.js:7137-7151
        "tuteng1" =>  { u05_mark_invoked(ctx, owner, "tuteng1"); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:7152-7163
        "tuteng2" =>  { u05_mark_invoked(ctx, owner, "tuteng2"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:7196-7207
        "tuteng5" =>  { u05_mark_invoked(ctx, owner, "tuteng5"); ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:7224-7250
        "tuteng7" =>  { u05_mark_invoked(ctx, owner, "tuteng7"); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:7251-7262
        "tuteng8" =>  { u05_mark_invoked(ctx, owner, "tuteng8"); set_flag(ctx.core, owner, "u05_mod:tuteng8", 1); },
        // character/hearth/skill.js:7263-7275
        "hongxi" =>  { u05_mark_invoked(ctx, owner, "hongxi"); u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:133-154
        "buwendingyibian_lose" =>  { u05_mark_invoked(ctx, owner, "buwendingyibian_lose"); if !u05_pay_discard(ctx, owner, "buwendingyibian_lose", 1, true) { return true; } },
        // character/hearth/skill.js:155-207
        "hualing" =>  { u05_mark_invoked(ctx, owner, "hualing"); if !u05_pay_discard(ctx, owner, "hualing", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/hearth/skill.js:208-235
        "muyin" =>  { u05_mark_invoked(ctx, owner, "muyin"); set_flag(ctx.core, owner, "u05_mod:yunchuizhen", 1); },
        // character/hearth/skill.js:236-272
        "tqchuanyue" =>  { u05_mark_invoked(ctx, owner, "tqchuanyue"); if !u05_pay_discard(ctx, owner, "tqchuanyue", 1, true) { return true; } },
        // character/hearth/skill.js:273-366
        "oldhuanjue" =>  { u05_mark_invoked(ctx, owner, "oldhuanjue"); if !u05_pay_discard(ctx, owner, "oldhuanjue", 1, true) { return true; } },
        // character/hearth/skill.js:367-464
        "hshuanling" =>  { u05_mark_invoked(ctx, owner, "hshuanling"); if !u05_check_limited(ctx, owner, "hshuanling") { return true; } if !u05_check_usable(ctx, owner, "hshuanling", 1) { return true; } if !u05_pay_discard(ctx, owner, "hshuanling", 1, true) { return true; } },
        // character/hearth/skill.js:465-505
        "hsxingyi" =>  { u05_mark_invoked(ctx, owner, "hsxingyi"); if !u05_pay_discard(ctx, owner, "hsxingyi", 1, true) { return true; } },
        // character/hearth/skill.js:1392-1413
        "zhaochao" =>  { u05_mark_invoked(ctx, owner, "zhaochao"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/hearth/skill.js:1414-1425
        "zhaochao2" =>  { u05_mark_invoked(ctx, owner, "zhaochao2"); if !u05_pay_discard(ctx, owner, "zhaochao2", 1, true) { return true; } },
        // character/hearth/skill.js:1592-1628
        "azaowu" =>  { u05_mark_invoked(ctx, owner, "azaowu"); if !u05_check_usable(ctx, owner, "azaowu", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/hearth/skill.js:1705-1711
        "shouwang2" =>  { u05_mark_invoked(ctx, owner, "shouwang2"); set_flag(ctx.core, owner, "u05_debuff:shouwang", 1); },
        // character/hearth/skill.js:1712-1727
        "qingtian" =>  { u05_mark_invoked(ctx, owner, "qingtian"); set_flag(ctx.core, owner, "u05_debuff:shouwang", 1); },
        // character/hearth/skill.js:1728-1790
        "qianfu" =>  { u05_mark_invoked(ctx, owner, "qianfu"); if !u05_pay_discard(ctx, owner, "qianfu", 3, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } set_flag(ctx.core, owner, "u05_mod:qianfu2", 1); },
        // character/hearth/skill.js:1893-1963
        "szbianshen" =>  { u05_mark_invoked(ctx, owner, "szbianshen"); if !u05_pay_discard(ctx, owner, "szbianshen", 1, true) { return true; } u05_change_hujia(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/hearth/skill.js:1964-2002
        "jinhua" =>  { u05_mark_invoked(ctx, owner, "jinhua"); if !u05_pay_discard(ctx, owner, "jinhua", 1, true) { return true; } },
        // character/hearth/skill.js:2210-2266
        "mengye2" =>  { u05_mark_invoked(ctx, owner, "mengye2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:2267-2305
        "kuixin" =>  { u05_mark_invoked(ctx, owner, "kuixin"); let _ = u05_prey_other(ctx, owner, true); },
        // character/hearth/skill.js:2306-2448
        "fuhua" | "fuhua2" | "fuhua3" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "juedou"), 1); },
        // character/hearth/skill.js:2449-2566
        "qianhou" =>  { u05_mark_invoked(ctx, owner, "qianhou"); if !u05_pay_discard(ctx, owner, "qianhou", 1, true) { return true; } },
        // character/hearth/skill.js:2567-2610
        "jinzhou" =>  { u05_mark_invoked(ctx, owner, "jinzhou"); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:guijiang2", 1); } },
        // character/hearth/skill.js:2797-2853
        "yushou_lose" =>  { u05_mark_invoked(ctx, owner, "yushou_lose"); if !u05_pay_discard(ctx, owner, "yushou_lose", 1, true) { return true; } },
        // character/hearth/skill.js:2854-2868
        "yushou_leiouke" =>  { u05_mark_invoked(ctx, owner, "yushou_leiouke"); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/hearth/skill.js:2996-3080
        "lianjin" =>  { u05_mark_invoked(ctx, owner, "lianjin"); if !u05_check_usable(ctx, owner, "lianjin", 1) { return true; } if !u05_pay_discard(ctx, owner, "lianjin", 1, true) { return true; } },
        // character/hearth/skill.js:3225-3264
        "yingxi" | "yingxi2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/hearth/skill.js:3265-3305
        "hsguimou" =>  { u05_mark_invoked(ctx, owner, "hsguimou"); let _ = u05_prey_other(ctx, owner, true); },
        // character/hearth/skill.js:3306-3338
        "peiyu" | "peiyu2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/hearth/skill.js:3339-3380
        "wzhanyi" =>  { u05_mark_invoked(ctx, owner, "wzhanyi"); if !u05_pay_discard(ctx, owner, "wzhanyi", 3, true) { return true; } },
        // character/hearth/skill.js:3381-3420
        "yuansu" =>  { u05_mark_invoked(ctx, owner, "yuansu"); u05_damage_other(ctx, owner, 2, DamageNature::Thunder); u05_change_max_hp(ctx, owner, -(1)); },
        // character/hearth/skill.js:3513-3575
        "nuyan" =>  { u05_mark_invoked(ctx, owner, "nuyan"); if !u05_check_usable(ctx, owner, "nuyan", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/hearth/skill.js:3646-3660
        "hsdusu_shinancao" =>  { u05_mark_invoked(ctx, owner, "hsdusu_shinancao"); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/hearth/skill.js:3661-3687
        "xiubu" =>  { u05_mark_invoked(ctx, owner, "xiubu"); if !u05_pay_discard(ctx, owner, "xiubu", 1, true) { return true; } },
        // character/hearth/skill.js:3688-3730
        "zengli" =>  { u05_mark_invoked(ctx, owner, "zengli"); if !u05_pay_discard(ctx, owner, "zengli", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3772-3794
        "mobao2" | "mobao3" =>  { u05_mark_invoked(ctx, owner, skill); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/hearth/skill.js:3795-3860
        "xianji" | "xianji2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/hearth/skill.js:3915-4002
        "maoxian" =>  { u05_mark_invoked(ctx, owner, "maoxian"); if !u05_pay_discard(ctx, owner, "maoxian", 1, true) { return true; } },
        // character/hearth/skill.js:4003-4087
        "qianghuax" =>  { u05_mark_invoked(ctx, owner, "qianghuax"); if !u05_check_limited(ctx, owner, "qianghuax") { return true; } if !u05_check_usable(ctx, owner, "qianghuax", 1) { return true; } if !u05_pay_discard(ctx, owner, "qianghuax", 1, true) { return true; } },
        // character/hearth/skill.js:4088-4185
        "xunbao" | "xunbao2" | "hsbaowu_huangjinyuanhou2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } u05_recover(ctx, owner, 1); },
        // character/hearth/skill.js:4335-4351
        "yufa2" | "yufa3" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:guijiang2", 1); } },
        // character/hearth/skill.js:4352-4410
        "bingyan" =>  { u05_mark_invoked(ctx, owner, "bingyan"); if !u05_check_usable(ctx, owner, "bingyan", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "chiyuxi"), 1); },
        // character/hearth/skill.js:4575-4620
        "duzhang" =>  { u05_mark_invoked(ctx, owner, "duzhang"); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:duzhang2", 1); } },
        // character/hearth/skill.js:4716-4796
        "fenlie" | "oldfenlie" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/hearth/skill.js:4797-4830
        "nianfu" =>  { u05_mark_invoked(ctx, owner, "nianfu"); let _ = u05_prey_other(ctx, owner, true); },
        // character/hearth/skill.js:4877-4930
        "shixu" =>  { u05_mark_invoked(ctx, owner, "shixu"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:4931-4996
        "qianghua" =>  { u05_mark_invoked(ctx, owner, "qianghua"); if !u05_pay_discard(ctx, owner, "qianghua", 1, true) { return true; } },
        // character/hearth/skill.js:4997-5038
        "biri" =>  { u05_mark_invoked(ctx, owner, "biri"); if !u05_check_usable(ctx, owner, "biri", 1) { return true; } set_flag(ctx.core, owner, "u05_mod:mingkong2", 1); },
        // character/hearth/skill.js:5039-5091
        "stuxi" | "stuxi2" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:gwjinli", 1); } },
        // character/hearth/skill.js:5092-5115
        "luoshi" =>  { u05_mark_invoked(ctx, owner, "luoshi"); if !u05_check_usable(ctx, owner, "luoshi", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:5116-5150
        "ronghuo" | "ronghuo2" =>  { u05_mark_invoked(ctx, owner, skill); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/hearth/skill.js:5230-5284
        "jiaohui" =>  { u05_mark_invoked(ctx, owner, "jiaohui"); if !u05_pay_discard(ctx, owner, "jiaohui", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/hearth/skill.js:5285-5320
        "liehun" =>  { u05_mark_invoked(ctx, owner, "liehun"); if !u05_pay_discard(ctx, owner, "liehun", 1, true) { return true; } },
        // character/hearth/skill.js:5369-5420
        "lingzhou" =>  { u05_mark_invoked(ctx, owner, "lingzhou"); if !u05_pay_discard(ctx, owner, "lingzhou", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/hearth/skill.js:5421-5460
        "xmojian" =>  { u05_mark_invoked(ctx, owner, "xmojian"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/hearth/skill.js:5743-5770
        "bingshi" =>  { u05_mark_invoked(ctx, owner, "bingshi"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/hearth/skill.js:5964-5980
        "qingliu" =>  { u05_mark_invoked(ctx, owner, "qingliu"); set_flag(ctx.core, owner, "u05_mod:mingkong2", 1); },
        // character/hearth/skill.js:6104-6130
        "hsmengjing_mengye" =>  { u05_mark_invoked(ctx, owner, "hsmengjing_mengye"); ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/hearth/skill.js:6484-6521
        "zuzhou" =>  { u05_mark_invoked(ctx, owner, "zuzhou"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); },
        // character/hearth/skill.js:6522-6591
        "mdzhoufu" | "mdzhoufu2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // character/hearth/skill.js:6637-6702
        "jingxiang" =>  { u05_mark_invoked(ctx, owner, "jingxiang"); if !u05_pay_discard(ctx, owner, "jingxiang", 1, true) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/hearth/skill.js:6723-6778
        "mengun" =>  { u05_mark_invoked(ctx, owner, "mengun"); if !u05_check_usable(ctx, owner, "mengun", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:mengun2", 1); } },
        // character/hearth/skill.js:6822-6842
        "jihuo" =>  { u05_mark_invoked(ctx, owner, "jihuo"); if !u05_check_usable(ctx, owner, "jihuo", 1) { return true; } set_flag(ctx.core, owner, "u05_mod:extra_turn", 1); ctx.start_draw(owner, 2); },
        // character/hearth/skill.js:6843-7136
        "tuteng_s" | "s_tuteng" | "tuteng" | "huanfeng" | "tuteng_lose" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } u05_recover(ctx, owner, 1); set_flag(ctx.core, owner, "u05_mod:tuteng_h", 1); },
        // character/hearth/skill.js:7164-7179
        "tuteng3" =>  { u05_mark_invoked(ctx, owner, "tuteng3"); set_flag(ctx.core, owner, "u05_mod:mingkong2", 1); },
        // character/hearth/skill.js:7180-7223
        "tuteng4" | "tuteng6" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        _ => return false,
    }
    true
}
