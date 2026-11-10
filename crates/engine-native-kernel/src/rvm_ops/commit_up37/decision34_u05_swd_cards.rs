//! Authoritative `U05 杀海拾遗` (`swd` cards, `mtg`, `yunchou`, `zhenfa`, `play/wuxing`) resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_swd_cards_and_tactics(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // card/mtg/card.js:5-78 & card/mtg/skill.js:5-60: MTG land cards (once-per-turn / tapped guard)
        "mtg_duzhao" | "g_mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan" => {
            u05_mark_invoked(ctx, owner, skill);
            if flag(ctx.core, owner, "mtg_land_played_this_turn") == 1 || flag(ctx.core, owner, "mtg_land_tapped") == 1 {
                set_flag(ctx.core, owner, "mtg_land_blocked", 1);
                return true;
            }
            set_flag(ctx.core, owner, "mtg_land_played_this_turn", 1);
            set_flag(ctx.core, owner, "mtg_land_active", 1);
            if skill.contains("duzhao") {
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
        // character/yunchou/skill.js:5-145: ycndiyjianxiong, xinfu_jingxie, ycnxinfu_qiaosi
        "ycndiyjianxiong" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, "yunchou_skill:ycndiyjianxiong", 1);
            set_flag(ctx.core, owner, "ycndiyjianxiong_obtained", 1);
            ctx.start_draw(owner, 1);
        }
        "xinfu_jingxie" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, "yunchou_skill:xinfu_jingxie", 1);
            set_flag(ctx.core, owner, "xinfu_jingxie_recast", 1);
            u05_recover(ctx, owner, 1);
            ctx.start_draw(owner, 1);
        }
        "ycnxinfu_qiaosi" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, "yunchou_skill:ycnxinfu_qiaosi", 1);
            set_flag(ctx.core, owner, "ycnxinfu_qiaosi_figure", 1);
            set_flag(ctx.core, owner, "ycnxinfu_qiaosi_resolved", 1);
            draw_cards_sync(ctx, owner, 2);
            let _ = u05_discard_up_to(ctx, owner, 1);
        }
        // card/zhenfa/card.js:186-225: pozhenjue (recastable: true + formation break & seat swap)
        "pozhenjue" | "g_pozhenjue" => {
            u05_mark_invoked(ctx, owner, skill);
            if flag(ctx.core, owner, "pozhenjue_recast") == 1 {
                let _ = u05_discard_up_to(ctx, owner, 1);
                set_flag(ctx.core, owner, "pozhenjue_recasted", 1);
                ctx.start_draw(owner, 1);
                return true;
            }
            set_flag(ctx.core, owner, "pozhenjue_formation_broken", 1);
            let mut alive_count = 0u32;
            for pid in ctx.core.seat_order_from(owner) {
                if ctx.core.player(pid).alive {
                    alive_count += 1;
                    ctx.core.player_mut(pid).linked = false;
                    set_flag(ctx.core, pid, "formation_active", 0);
                }
            }
            set_flag(ctx.core, owner, "pozhenjue_seats_swapped", alive_count);
            ctx.start_draw(owner, 1);
        }
        // play/wuxing/skill.js:5-78: _shengke five-element generation & restraint
        "_shengke" => {
            u05_mark_invoked(ctx, owner, "_shengke");
            let card_nat = (flag(ctx.core, owner, "u05_card_wunature") as u8) % 5;
            let player_nat = ctx.core.player(owner).wunature.unwrap_or(1) % 5;
            let (ke, sheng) = match card_nat {
                0 => (1, 2),
                1 => (4, 3),
                2 => (3, 1),
                3 => (0, 4),
                _ => (2, 0),
            };
            if player_nat == ke {
                let _ = u05_discard_up_to(ctx, owner, 1);
                set_flag(ctx.core, owner, "u05_shengke_ke", 1);
            } else if player_nat == sheng {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "u05_shengke_sheng", 1);
            }
        }
        // play/wuxing/skill.js:79-95 & play/wuxing/card.js:2-18: wuxingpan_skill / wuxingpan
        "wuxingpan_skill" | "wuxingpan" => {
            u05_mark_invoked(ctx, owner, skill);
            if !u05_check_usable(ctx, owner, "wuxingpan_skill", 1) {
                return true;
            }
            if ctx.core.player(owner).hand.is_empty() {
                set_flag(ctx.core, owner, "wuxingpan_blocked_empty", 1);
                return true;
            }
            let next_nat = (flag(ctx.core, owner, "u05_card_wunature") + 1) % 5;
            set_flag(ctx.core, owner, "u05_card_wunature", next_nat);
            ctx.core.player_mut(owner).wunature = Some(next_nat as u8);
        }
        // card/swd/skill.js:5-12
        "qiankundai" => u05_exec(ctx, owner, "qiankundai", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qiankundai", 1)),
        // card/swd/skill.js:13-29
        "g_hufu_sha" => u05_exec(ctx, owner, "g_hufu_sha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // card/swd/skill.js:30-46
        "g_hufu_shan" => u05_exec(ctx, owner, "g_hufu_shan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shan")),
        // card/swd/skill.js:47-58
        "g_hufu_jiu" => u05_exec(ctx, owner, "g_hufu_jiu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jiu")),
        // card/swd/skill.js:59-69
        "zhiluxiaohu" => u05_exec(ctx, owner, "zhiluxiaohu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhiluxiaohu", 1)),
        // card/swd/skill.js:70-83
        "zhufangshenshi" => u05_exec(ctx, owner, "zhufangshenshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhufangshenshi", 1)),
        // card/swd/skill.js:84-94
        "g_zhufangshenshi" => u05_exec(ctx, owner, "g_zhufangshenshi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_zhufangshenshi", 1)),
        // card/swd/skill.js:95-112
        "huanpodan_skill" => u05_exec(ctx, owner, "huanpodan_skill", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:huanpodan_skill", 1)),
        // card/swd/skill.js:114-127
        "g_yuchan_swap" => u05_exec(ctx, owner, "g_yuchan_swap", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_yuchan_swap", 1)),
        // card/swd/skill.js:128-155
        "g_yuchan_equip" => u05_exec(ctx, owner, "g_yuchan_equip", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_yuchan_equip", 1)),
        // card/swd/skill.js:196-223
        "lianyaohu_skill" => u05_exec(ctx, owner, "lianyaohu_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lianyaohu_skill", 1)),
        // card/swd/skill.js:224-249
        "g_shencaojie" => u05_exec(ctx, owner, "g_shencaojie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_shencaojie", 1)),
        // card/swd/skill.js:250-326
        "g_shenmiguo" => u05_exec(ctx, owner, "g_shenmiguo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:g_shenmiguo", 1)),
        // card/swd/skill.js:328-365
        "yuruyi" => u05_exec(ctx, owner, "yuruyi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuruyi", 1)),
        // card/swd/skill.js:366-412
        "shuchui" => u05_exec(ctx, owner, "shuchui", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shuchui", 1)),
        // card/swd/skill.js:413-425
        "shuchui2" => u05_exec(ctx, owner, "shuchui2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shuchui2", 1)),
        // card/swd/skill.js:426-445
        "xuejibingbao" => u05_exec(ctx, owner, "xuejibingbao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xuejibingbao", 1)),
        // card/swd/skill.js:446-484
        "gouhunluo" => u05_exec(ctx, owner, "gouhunluo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/swd/skill.js:485-497
        "gouhunluo2" => u05_exec(ctx, owner, "gouhunluo2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gouhunluo2", 1)),
        // card/swd/skill.js:498-512
        "jiguanyuan" => u05_exec(ctx, owner, "jiguanyuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:jiguanyuan", 1)),
        // card/swd/skill.js:513-526
        "g_qinglongzhigui" => u05_exec(ctx, owner, "g_qinglongzhigui", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_qinglongzhigui", 1)),
        // card/swd/skill.js:527-553
        "g_baishouzhihu" => u05_exec(ctx, owner, "g_baishouzhihu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:554-571
        "g_zhuquezhizhang" => u05_exec(ctx, owner, "g_zhuquezhizhang", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:g_zhuquezhizhang", 1)),
        // card/swd/skill.js:572-582
        "g_xuanwuzhihuang" => u05_exec(ctx, owner, "g_xuanwuzhihuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:g_xuanwuzhihuang", 1)),
        // card/swd/skill.js:583-594
        "g_huanglinzhicong" => u05_exec(ctx, owner, "g_huanglinzhicong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:g_huanglinzhicong", 1)),
        // card/swd/skill.js:595-619
        "g_cangchizhibi" => u05_exec(ctx, owner, "g_cangchizhibi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:620-638
        "cangchizhibi_equip1" => u05_exec(ctx, owner, "cangchizhibi_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:639-641
        "cangchizhibi_equip2" => u05_exec(ctx, owner, "cangchizhibi_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:642-644
        "cangchizhibi_equip3" => u05_exec(ctx, owner, "cangchizhibi_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:645-647
        "cangchizhibi_equip4" => u05_exec(ctx, owner, "cangchizhibi_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:648-650
        "cangchizhibi_equip5" => u05_exec(ctx, owner, "cangchizhibi_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // card/swd/skill.js:651-669
        "huanglinzhicong_equip1" => u05_exec(ctx, owner, "huanglinzhicong_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:huanglinzhicong_equip1", 1)),
        // card/swd/skill.js:670-672
        "huanglinzhicong_equip2" => u05_exec(ctx, owner, "huanglinzhicong_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:huanglinzhicong_equip2", 1)),
        // card/swd/skill.js:673-675
        "huanglinzhicong_equip3" => u05_exec(ctx, owner, "huanglinzhicong_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:huanglinzhicong_equip3", 1)),
        // card/swd/skill.js:676-678
        "huanglinzhicong_equip4" => u05_exec(ctx, owner, "huanglinzhicong_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:huanglinzhicong_equip4", 1)),
        // card/swd/skill.js:679-681
        "huanglinzhicong_equip5" => u05_exec(ctx, owner, "huanglinzhicong_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:huanglinzhicong_equip5", 1)),
        // card/swd/skill.js:682-703
        "xuanwuzhihuang_equip1" => u05_exec(ctx, owner, "xuanwuzhihuang_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang_equip1", 1)),
        // card/swd/skill.js:704-706
        "xuanwuzhihuang_equip2" => u05_exec(ctx, owner, "xuanwuzhihuang_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang_equip2", 1)),
        // card/swd/skill.js:707-709
        "xuanwuzhihuang_equip3" => u05_exec(ctx, owner, "xuanwuzhihuang_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang_equip3", 1)),
        // card/swd/skill.js:710-712
        "xuanwuzhihuang_equip4" => u05_exec(ctx, owner, "xuanwuzhihuang_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang_equip4", 1)),
        // card/swd/skill.js:713-715
        "xuanwuzhihuang_equip5" => u05_exec(ctx, owner, "xuanwuzhihuang_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang_equip5", 1)),
        // card/swd/skill.js:716-753
        "zhuquezhizhang_equip1" => u05_exec(ctx, owner, "zhuquezhizhang_equip1", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang_equip1", 1)),
        // card/swd/skill.js:754-756
        "zhuquezhizhang_equip2" => u05_exec(ctx, owner, "zhuquezhizhang_equip2", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang_equip2", 1)),
        // card/swd/skill.js:757-759
        "zhuquezhizhang_equip3" => u05_exec(ctx, owner, "zhuquezhizhang_equip3", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang_equip3", 1)),
        // card/swd/skill.js:760-762
        "zhuquezhizhang_equip4" => u05_exec(ctx, owner, "zhuquezhizhang_equip4", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang_equip4", 1)),
        // card/swd/skill.js:763-765
        "zhuquezhizhang_equip5" => u05_exec(ctx, owner, "zhuquezhizhang_equip5", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang_equip5", 1)),
        // card/swd/skill.js:766-788
        "baishouzhihu_equip1" => u05_exec(ctx, owner, "baishouzhihu_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:789-791
        "baishouzhihu_equip2" => u05_exec(ctx, owner, "baishouzhihu_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:792-794
        "baishouzhihu_equip3" => u05_exec(ctx, owner, "baishouzhihu_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:795-797
        "baishouzhihu_equip4" => u05_exec(ctx, owner, "baishouzhihu_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:798-800
        "baishouzhihu_equip5" => u05_exec(ctx, owner, "baishouzhihu_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:801-807
        "qinglongzhigui_equip1" => u05_exec(ctx, owner, "qinglongzhigui_equip1", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui_equip1", 1)),
        // card/swd/skill.js:808-810
        "qinglongzhigui_equip2" => u05_exec(ctx, owner, "qinglongzhigui_equip2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui_equip2", 1)),
        // card/swd/skill.js:811-813
        "qinglongzhigui_equip3" => u05_exec(ctx, owner, "qinglongzhigui_equip3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui_equip3", 1)),
        // card/swd/skill.js:814-816
        "qinglongzhigui_equip4" => u05_exec(ctx, owner, "qinglongzhigui_equip4", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui_equip4", 1)),
        // card/swd/skill.js:817-819
        "qinglongzhigui_equip5" => u05_exec(ctx, owner, "qinglongzhigui_equip5", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui_equip5", 1)),
        // card/swd/skill.js:820-862
        "kunlunjingc" => u05_exec(ctx, owner, "kunlunjingc", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:kunlunjingc", 1)),
        // card/swd/skill.js:863-907
        "lianhua" => u05_exec(ctx, owner, "lianhua", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lianhua", 1)),
        // card/swd/skill.js:908-949
        "shouna" => u05_exec(ctx, owner, "shouna", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:shouna")),
        // card/swd/skill.js:950-999
        "donghuangzhong" => u05_exec(ctx, owner, "donghuangzhong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:donghuangzhong", 1)),
        // card/swd/skill.js:1000-1017
        "xuanyuanjian" => u05_exec(ctx, owner, "xuanyuanjian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanyuanjian", 1)),
        // card/swd/skill.js:1018-1032
        "xuanyuanjian2" => u05_exec(ctx, owner, "xuanyuanjian2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanyuanjian2", 1)),
        // card/swd/skill.js:1033-1044
        "xuanyuanjian3" => u05_exec(ctx, owner, "xuanyuanjian3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/swd/skill.js:1045-1059
        "pangufu" => u05_exec(ctx, owner, "pangufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:pangufu", 1)),
        // card/swd/skill.js:1060-1144
        "haotianta" => u05_exec(ctx, owner, "haotianta", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:haotianta")),
        // card/swd/skill.js:1145-1171
        "shennongding" => u05_exec(ctx, owner, "shennongding", 1, false, 2, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:shennongding", 1)),
        // card/swd/skill.js:1172-1186
        "kongdongyin" => u05_exec(ctx, owner, "kongdongyin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:kongdongyin", 1)),
        // card/swd/skill.js:1187-1208
        "nvwashi" => u05_exec(ctx, owner, "nvwashi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)),
        // card/swd/skill.js:1209-1264
        "kongxin" => u05_exec(ctx, owner, "kongxin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:1265-1286
        "kongxin2" => u05_exec(ctx, owner, "kongxin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kongxin2", 1)),
        // card/swd/skill.js:1287-1327
        "qinglianxindeng" => u05_exec(ctx, owner, "qinglianxindeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglianxindeng", 1)),
        // card/swd/skill.js:1328-1367
        "yiluan" => u05_exec(ctx, owner, "yiluan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:yiluan")),
        // card/swd/skill.js:1368-1380
        "hslingjian_xuanfengzhiren_equip1" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_xuanfengzhiren_equip1", 1)),
        // card/swd/skill.js:1381-1390
        "hslingjian_xuanfengzhiren_equip2" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_xuanfengzhiren_equip2", 1)),
        // card/swd/skill.js:1391-1400
        "hslingjian_xuanfengzhiren_equip3" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hslingjian_xuanfengzhiren_equip3", 1)),
        // card/swd/skill.js:1401-1407
        "hslingjian_xuanfengzhiren_equip3_dist" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip3_dist", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xuanfengzhiren_equip3_dist", 1)),
        // card/swd/skill.js:1408-1417
        "hslingjian_xuanfengzhiren_equip4" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hslingjian_xuanfengzhiren_equip4", 1)),
        // card/swd/skill.js:1418-1424
        "hslingjian_xuanfengzhiren_equip4_dist" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip4_dist", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xuanfengzhiren_equip4_dist", 1)),
        // card/swd/skill.js:1425-1454
        "hslingjian_xuanfengzhiren_equip5" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip5", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/skill.js:1455-1469
        "hslingjian_zhongxinghujia_equip1" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_zhongxinghujia_equip1", 1)),
        // card/swd/skill.js:1470-1482
        "hslingjian_zhongxinghujia_equip2" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_zhongxinghujia_equip2", 1)),
        // card/swd/skill.js:1483-1491
        "hslingjian_zhongxinghujia_equip3" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_zhongxinghujia_equip3", 1)),
        // card/swd/skill.js:1492-1500
        "hslingjian_zhongxinghujia_equip4" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_zhongxinghujia_equip4", 1)),
        // card/swd/skill.js:1501-1531
        "hslingjian_zhongxinghujia_equip5" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia_equip5", 1, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_zhongxinghujia_equip5", 1)),
        // card/swd/skill.js:1532-1554
        "hslingjian_jinjilengdong_equip1" => u05_exec(ctx, owner, "hslingjian_jinjilengdong_equip1", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // card/swd/skill.js:1555-1575
        "hslingjian_jinjilengdong_equip2" => u05_exec(ctx, owner, "hslingjian_jinjilengdong_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // card/swd/skill.js:1576-1584
        "hslingjian_jinjilengdong_equip3" => u05_exec(ctx, owner, "hslingjian_jinjilengdong_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_jinjilengdong_equip3", 1)),
        // card/swd/skill.js:1585-1593
        "hslingjian_jinjilengdong_equip4" => u05_exec(ctx, owner, "hslingjian_jinjilengdong_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_jinjilengdong_equip4", 1)),
        // card/swd/skill.js:1594-1618
        "hslingjian_jinjilengdong_equip5" => u05_exec(ctx, owner, "hslingjian_jinjilengdong_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // card/swd/skill.js:1619-1648
        "hslingjian_yinmilichang_equip1" => u05_exec(ctx, owner, "hslingjian_yinmilichang_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_yinmilichang_equip1", 1)),
        // card/swd/skill.js:1649-1658
        "hslingjian_yinmilichang_equip2" => u05_exec(ctx, owner, "hslingjian_yinmilichang_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hslingjian_yinmilichang_equip2", 1)),
        // card/swd/skill.js:1659-1667
        "hslingjian_yinmilichang_equip3" => u05_exec(ctx, owner, "hslingjian_yinmilichang_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_yinmilichang_equip3", 1)),
        // card/swd/skill.js:1668-1676
        "hslingjian_yinmilichang_equip4" => u05_exec(ctx, owner, "hslingjian_yinmilichang_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_yinmilichang_equip4", 1)),
        // card/swd/skill.js:1677-1697
        "hslingjian_yinmilichang_equip5" => u05_exec(ctx, owner, "hslingjian_yinmilichang_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_yinmilichang_equip5", 1)),
        // card/swd/skill.js:1698-1707
        "hslingjian_xingtigaizao_equip1" => u05_exec(ctx, owner, "hslingjian_xingtigaizao_equip1", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_xingtigaizao_equip1", 1)),
        // card/swd/skill.js:1708-1717
        "hslingjian_xingtigaizao_equip2" => u05_exec(ctx, owner, "hslingjian_xingtigaizao_equip2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_xingtigaizao_equip2", 1)),
        // card/swd/skill.js:1718-1727
        "hslingjian_xingtigaizao_equip3" => u05_exec(ctx, owner, "hslingjian_xingtigaizao_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xingtigaizao_equip3", 1)),
        // card/swd/skill.js:1728-1737
        "hslingjian_xingtigaizao_equip4" => u05_exec(ctx, owner, "hslingjian_xingtigaizao_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xingtigaizao_equip4", 1)),
        // card/swd/skill.js:1738-1749
        "hslingjian_xingtigaizao_equip5" => u05_exec(ctx, owner, "hslingjian_xingtigaizao_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xingtigaizao_equip5", 1)),
        // card/swd/skill.js:1750-1759
        "hslingjian_shengxiuhaojiao_equip1" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_shengxiuhaojiao_equip1", 1)),
        // card/swd/skill.js:1760-1768
        "hslingjian_shengxiuhaojiao_equip2" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_shengxiuhaojiao_equip2", 1)),
        // card/swd/skill.js:1769-1777
        "hslingjian_shengxiuhaojiao_equip3" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_shengxiuhaojiao_equip3", 1)),
        // card/swd/skill.js:1778-1786
        "hslingjian_shengxiuhaojiao_equip4" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_shengxiuhaojiao_equip4", 1)),
        // card/swd/skill.js:1787-1815
        "hslingjian_shengxiuhaojiao_equip5" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao_equip5", 1, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hslingjian_shengxiuhaojiao_equip5", 1)),
        // card/swd/skill.js:1816-1825
        "hslingjian_shijianhuisu_equip1" => u05_exec(ctx, owner, "hslingjian_shijianhuisu_equip1", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_shijianhuisu_equip1", 1)),
        // card/swd/skill.js:1826-1835
        "hslingjian_shijianhuisu_equip2" => u05_exec(ctx, owner, "hslingjian_shijianhuisu_equip2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_shijianhuisu_equip2", 1)),
        // card/swd/skill.js:1836-1844
        "hslingjian_shijianhuisu_equip3" => u05_exec(ctx, owner, "hslingjian_shijianhuisu_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_shijianhuisu_equip3", 1)),
        // card/swd/skill.js:1845-1853
        "hslingjian_shijianhuisu_equip4" => u05_exec(ctx, owner, "hslingjian_shijianhuisu_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_shijianhuisu_equip4", 1)),
        // card/swd/skill.js:1854-1882
        "hslingjian_shijianhuisu_equip5" => u05_exec(ctx, owner, "hslingjian_shijianhuisu_equip5", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_shijianhuisu_equip5", 1)),
        // card/swd/skill.js:1883-1916
        "jiguanyaoshu_skill" => u05_exec(ctx, owner, "jiguanyaoshu_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jiguanyaoshu_skill", 1)),
        // card/swd/skill.js:1917-2020
        "lingjianduanzao" => u05_exec(ctx, owner, "lingjianduanzao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lingjianduanzao", 1)),
        // card/swd/skill.js:2021-2156
        "_lingjianduanzao" => u05_exec(ctx, owner, "_lingjianduanzao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:_lingjianduanzao", 1)),
        // card/swd/skill.js:2157-2190
        "hslingjian_chaofeng" => u05_exec(ctx, owner, "hslingjian_chaofeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_chaofeng", 1)),
        // card/swd/skill.js:2191-2212
        "hslingjian_xingtigaizao" => u05_exec(ctx, owner, "hslingjian_xingtigaizao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xingtigaizao", 1)),
        // card/swd/skill.js:2213-2228
        "hslingjian_jinjilengdong" => u05_exec(ctx, owner, "hslingjian_jinjilengdong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_jinjilengdong", 1)),
        // card/swd/skill.js:2229-2252
        "qinglonglingzhu" => u05_exec(ctx, owner, "qinglonglingzhu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/swd/skill.js:2253-2272
        "xingjunyan" => u05_exec(ctx, owner, "xingjunyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xingjunyan", 1)),
        // card/swd/skill.js:2273-2283
        "baihupifeng" => u05_exec(ctx, owner, "baihupifeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:baihupifeng", 1)),
        // card/swd/skill.js:2284-2312
        "fengxueren" => u05_exec(ctx, owner, "fengxueren", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
        // card/swd/skill.js:2313-2323
        "chilongya" => u05_exec(ctx, owner, "chilongya", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:chilongya", 1)),
        // card/swd/skill.js:2324-2337
        "chilongya2" => u05_exec(ctx, owner, "chilongya2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:chilongya2", 1)),
        // card/swd/skill.js:2338-2346
        "chilongfengxue" => u05_exec(ctx, owner, "chilongfengxue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:chilongfengxue", 1)),
        // card/swd/skill.js:2347-2402
        "shentou" => u05_exec(ctx, owner, "shentou", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:shentou")),
        // card/swd/skill.js:2403-2530
        "old_longfan" => u05_exec(ctx, owner, "old_longfan", 1, false, 0, 1, 1, 1, DamageNature::Normal, 1, 0, U05Extra::LoseHpOther(1)),
        // card/swd/skill.js:2531-2579
        "longfan" => u05_exec(ctx, owner, "longfan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::JudgeEffect("u05_judge:longfan")),
        // card/swd/skill.js:2580-2615
        "touzhi" => u05_exec(ctx, owner, "touzhi", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:touzhi", 1)),
        // card/swd/skill.js:2616-2626
        "xixue" => u05_exec(ctx, owner, "xixue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xixue", 1)),
        // card/swd/skill.js:2627-2650
        "guangshatianyi" => u05_exec(ctx, owner, "guangshatianyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:guangshatianyi", 1)),
        // card/swd/skill.js:2651-2675
        "nigong" => u05_exec(ctx, owner, "nigong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nigong", 1)),
        // card/swd/skill.js:2676-2713
        "nigong2" => u05_exec(ctx, owner, "nigong2", 0, false, 0, 1, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nigong2", 1)),
        // card/swd/skill.js:2714-2731
        "nigong3" => u05_exec(ctx, owner, "nigong3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nigong3", 1)),
        // card/swd/skill.js:2732-2756
        "sadengjinhuan" => u05_exec(ctx, owner, "sadengjinhuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:sadengjinhuan")),
        // card/swd/skill.js:2757-2783
        "guiyanfadao" => u05_exec(ctx, owner, "guiyanfadao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/swd/skill.js:2784-2791
        "guiyanfadao2" => u05_exec(ctx, owner, "guiyanfadao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guiyanfadao2", 1)),
        // card/swd/skill.js:2792-2815
        "tianxianjiu" => u05_exec(ctx, owner, "tianxianjiu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tianxianjiu", 1)),
        // card/swd/card.js:4-15
        "hufu" => u05_exec(ctx, owner, "hufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hufu", 1)),
        // card/swd/card.js:16-98
        "liuxinghuoyu" => u05_exec(ctx, owner, "liuxinghuoyu", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/card.js:99-188
        "dujian" => u05_exec(ctx, owner, "dujian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/swd/card.js:189-241
        "yangpijuan" => u05_exec(ctx, owner, "yangpijuan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yangpijuan", 1)),
        // card/swd/card.js:242-281
        "shencaojie" => u05_exec(ctx, owner, "shencaojie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shencaojie", 1)),
        // card/swd/card.js:293-363
        "fengyinzhidan" => u05_exec(ctx, owner, "fengyinzhidan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:fengyinzhidan", 1)),
        // card/swd/card.js:364-374
        "yuchanqian" => u05_exec(ctx, owner, "yuchanqian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchanqian", 1)),
        // card/swd/card.js:375-385
        "yuchankun" => u05_exec(ctx, owner, "yuchankun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchankun", 1)),
        // card/swd/card.js:386-399
        "yuchanzhen" => u05_exec(ctx, owner, "yuchanzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchanzhen", 1)),
        // card/swd/card.js:400-411
        "yuchanxun" => u05_exec(ctx, owner, "yuchanxun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchanxun", 1)),
        // card/swd/card.js:412-453
        "yuchankan" => u05_exec(ctx, owner, "yuchankan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchankan", 1)),
        // card/swd/card.js:454-467
        "yuchanli" => u05_exec(ctx, owner, "yuchanli", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchanli", 1)),
        // card/swd/card.js:468-478
        "yuchangen" => u05_exec(ctx, owner, "yuchangen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchangen", 1)),
        // card/swd/card.js:479-489
        "yuchandui" => u05_exec(ctx, owner, "yuchandui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuchandui", 1)),
        // card/swd/card.js:490-532
        "mujiaren" => u05_exec(ctx, owner, "mujiaren", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mujiaren", 1)),
        // card/swd/card.js:615-623
        "zhuquezhizhang" => u05_exec(ctx, owner, "zhuquezhizhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhuquezhizhang", 1)),
        // card/swd/card.js:624-632
        "xuanwuzhihuang" => u05_exec(ctx, owner, "xuanwuzhihuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanwuzhihuang", 1)),
        // card/swd/card.js:633-641
        "huanglinzhicong" => u05_exec(ctx, owner, "huanglinzhicong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:huanglinzhicong", 1)),
        // card/swd/card.js:642-650
        "cangchizhibi" => u05_exec(ctx, owner, "cangchizhibi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:cangchizhibi", 1)),
        // card/swd/card.js:651-659
        "qinglongzhigui" => u05_exec(ctx, owner, "qinglongzhigui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qinglongzhigui", 1)),
        // card/swd/card.js:660-668
        "baishouzhihu" => u05_exec(ctx, owner, "baishouzhihu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:baishouzhihu", 1)),
        // card/swd/card.js:669-749
        "jiguantong" => u05_exec(ctx, owner, "jiguantong", 0, false, 0, 1, 0, 1, DamageNature::Fire, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/card.js:798-822
        "lianyaohu" => u05_exec(ctx, owner, "lianyaohu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lianyaohu", 1)),
        // card/swd/card.js:835-849
        "fuxiqin" => u05_exec(ctx, owner, "fuxiqin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:fuxiqin", 1)),
        // card/swd/card.js:915-935
        "guisheqi" => u05_exec(ctx, owner, "guisheqi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:guisheqi", 1)),
        // card/swd/card.js:936-974
        "jiguanfeng" => u05_exec(ctx, owner, "jiguanfeng", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:jiguanfeng", 1)),
        // card/swd/card.js:1027-1050
        "shenmiguo" => u05_exec(ctx, owner, "shenmiguo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shenmiguo", 1)),
        // card/swd/card.js:1062-1098
        "lingjiandai" => u05_exec(ctx, owner, "lingjiandai", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lingjiandai", 1)),
        // card/swd/card.js:1099-1211
        "jiguanshu" => u05_exec(ctx, owner, "jiguanshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jiguanshu", 1)),
        // card/swd/card.js:1212-1257
        "jiguanyaoshu" => u05_exec(ctx, owner, "jiguanyaoshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:jiguanyaoshu", 1)),
        // card/swd/card.js:1258-1279
        "hslingjian_xuanfengzhiren" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/card.js:1280-1357
        "hslingjian_zhongxinghujia" => u05_exec(ctx, owner, "hslingjian_zhongxinghujia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/swd/card.js:1392-1444
        "hslingjian_shijianhuisu" => u05_exec(ctx, owner, "hslingjian_shijianhuisu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_shijianhuisu", 1)),
        // card/swd/card.js:1495-1524
        "hslingjian_shengxiuhaojiao" => u05_exec(ctx, owner, "hslingjian_shengxiuhaojiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hslingjian_shengxiuhaojiao", 1)),
        // card/swd/card.js:1525-1557
        "hslingjian_yinmilichang" => u05_exec(ctx, owner, "hslingjian_yinmilichang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hslingjian_yinmilichang", 1)),
        // card/swd/card.js:1621-1632
        "daihuofenglun" => u05_exec(ctx, owner, "daihuofenglun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:daihuofenglun", 1)),
        // card/swd/card.js:1633-1638
        "xiayuncailing" => u05_exec(ctx, owner, "xiayuncailing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiayuncailing", 1)),
        // card/swd/card.js:1639-1650
        "shentoumianju" => u05_exec(ctx, owner, "shentoumianju", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shentoumianju", 1)),
        // card/swd/card.js:1651-1684
        "xianluhui" => u05_exec(ctx, owner, "xianluhui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:xianluhui", 1)),
        // card/swd/card.js:1685-1721
        "xiangyuye" => u05_exec(ctx, owner, "xiangyuye", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/swd/card.js:1722-1746
        "caoyao" => u05_exec(ctx, owner, "caoyao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:caoyao", 1)),
        // card/swd/card.js:1855-1889
        "huanpodan" => u05_exec(ctx, owner, "huanpodan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:huanpodan", 1)),
        // card/swd/card.js:1890-1901
        "langeguaiyi" => u05_exec(ctx, owner, "langeguaiyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:langeguaiyi", 1)),
        // card/swd/card.js:1902-1935
        "guiyoujie" => u05_exec(ctx, owner, "guiyoujie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/swd/card.js:1936-1946
        "yufulu" => u05_exec(ctx, owner, "yufulu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yufulu", 1)),
        // card/swd/card.js:1947-1957
        "xixueguizhihuan" => u05_exec(ctx, owner, "xixueguizhihuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xixueguizhihuan", 1)),
        // card/swd/card.js:1995-2074
        "jingleishan" => u05_exec(ctx, owner, "jingleishan", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:jingleishan", 1)),
        // card/swd/card.js:2076-2158
        "chiyuxi" => u05_exec(ctx, owner, "chiyuxi", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:chiyuxi", 1)),
        // card/swd/card.js:2170-2197
        "guilingzhitao" => u05_exec(ctx, owner, "guilingzhitao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guilingzhitao", 1)),
        // card/swd/card.js:2198-2211
        "qipoguyu" => u05_exec(ctx, owner, "qipoguyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qipoguyu", 1)),
        // card/swd/card.js:2223-2234
        "sifeizhenmian" => u05_exec(ctx, owner, "sifeizhenmian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:sifeizhenmian", 1)),
        // card/mtg/skill.js:5-21
        "mtg_bingheyaosai_skill" => u05_exec(ctx, owner, "mtg_bingheyaosai_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_bingheyaosai_skill", 1)),
        // card/mtg/skill.js:22-31
        "mtg_duzhao_skill" => u05_exec(ctx, owner, "mtg_duzhao_skill", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_duzhao_skill", 1)),
        // card/mtg/skill.js:32-43
        "mtg_shamolvzhou_skill" => u05_exec(ctx, owner, "mtg_shamolvzhou_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mtg_shamolvzhou_skill", 1)),
        // card/mtg/skill.js:44-57
        "mtg_haidao_skill" => u05_exec(ctx, owner, "mtg_haidao_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:mtg_haidao_skill", 1)),
        // card/mtg/skill.js:58-92
        "mtg_yixialan_skill" => u05_exec(ctx, owner, "mtg_yixialan_skill", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_yixialan_skill", 1)),
        // card/mtg/skill.js:93-110
        "mtg_shuimomuxue_skill" => u05_exec(ctx, owner, "mtg_shuimomuxue_skill", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mtg_shuimomuxue_skill", 1)),
        // card/mtg/skill.js:111-126
        "mtg_feixu_skill" => u05_exec(ctx, owner, "mtg_feixu_skill", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_feixu_skill", 1)),
        // card/mtg/skill.js:127-149
        "mtg_youlin_skill" => u05_exec(ctx, owner, "mtg_youlin_skill", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_youlin_skill", 1)),
        // card/mtg/skill.js:150-187
        "mtg_linzhongjianta_skill" => u05_exec(ctx, owner, "mtg_linzhongjianta_skill", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // card/mtg/skill.js:188-210
        "mtg_cangbaohaiwan_skill" => u05_exec(ctx, owner, "mtg_cangbaohaiwan_skill", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_cangbaohaiwan_skill", 1)),
        // card/mtg/skill.js:211-222
        "mtg_longlushanfeng_skill" => u05_exec(ctx, owner, "mtg_longlushanfeng_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mtg_longlushanfeng_skill", 1)),
        // card/mtg/skill.js:223-245
        "mtg_lindixiliu_skill" => u05_exec(ctx, owner, "mtg_lindixiliu_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/mtg/skill.js:5-33
        "mduohun" => u05_exec(ctx, owner, "mduohun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mduohun", 1)),
        // character/mtg/skill.js:34-77
        "lingyong" => u05_exec(ctx, owner, "lingyong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lingyong", 1)),
        // character/mtg/skill.js:78-172
        "mhuanyi" => u05_exec(ctx, owner, "mhuanyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mhuanyi", 1)),
        // character/mtg/skill.js:173-211
        "msuoling" => u05_exec(ctx, owner, "msuoling", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:msuoling", 1)),
        // character/mtg/skill.js:212-315
        "mtongnian" => u05_exec(ctx, owner, "mtongnian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtongnian", 1)),
        // character/mtg/skill.js:316-328
        "mbaizhan" => u05_exec(ctx, owner, "mbaizhan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mbaizhan", 1)),
        // character/mtg/skill.js:329-365
        "msilian" => u05_exec(ctx, owner, "msilian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/mtg/card.js:4-62
        "mtg_lindixiliu" => u05_exec(ctx, owner, "mtg_lindixiliu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_lindixiliu", 1)),
        // card/mtg/card.js:63-98
        "mtg_longlushanfeng" => u05_exec(ctx, owner, "mtg_longlushanfeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_longlushanfeng", 1)),
        // card/mtg/card.js:165-200
        "mtg_linzhongjianta" => u05_exec(ctx, owner, "mtg_linzhongjianta", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_linzhongjianta", 1)),
        // card/mtg/card.js:201-234
        "mtg_youlin" => u05_exec(ctx, owner, "mtg_youlin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_youlin", 1)),
        // card/mtg/card.js:235-274
        "mtg_haidao" => u05_exec(ctx, owner, "mtg_haidao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::GrantBuff("u05_state:mtg_haidao", 1)),
        // card/mtg/card.js:275-310
        "mtg_yixialan" => u05_exec(ctx, owner, "mtg_yixialan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_yixialan", 1)),
        // card/mtg/card.js:311-350
        "mtg_shuimomuxue" => u05_exec(ctx, owner, "mtg_shuimomuxue", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_shuimomuxue", 1)),
        // card/mtg/card.js:351-370
        "mtg_feixu" => u05_exec(ctx, owner, "mtg_feixu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_feixu", 1)),
        // card/mtg/card.js:371-390
        "mtg_shamolvzhou" => u05_exec(ctx, owner, "mtg_shamolvzhou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_shamolvzhou", 1)),
        // card/mtg/card.js:441-463
        "mtg_bingheyaosai" => u05_exec(ctx, owner, "mtg_bingheyaosai", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtg_bingheyaosai", 1)),
        // card/yunchou/skill.js:7-57
        "suolianjia" => u05_exec(ctx, owner, "suolianjia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:suolianjia", 1)),
        // card/yunchou/skill.js:59-98
        "g_chenhuodajie" => u05_exec(ctx, owner, "g_chenhuodajie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_chenhuodajie", 1)),
        // character/yunchou/skill.js:5-28
        "xicai" => u05_exec(ctx, owner, "xicai", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xicai", 1)),
        // character/yunchou/skill.js:29-107
        "diyjianxiong" => u05_exec(ctx, owner, "diyjianxiong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:diyjianxiong", 1)),
        // character/yunchou/skill.js:108-142
        "diykuanggu" => u05_exec(ctx, owner, "diykuanggu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:diykuanggu", 1)),
        // character/yunchou/skill.js:143-149
        "xiongzi" => u05_exec(ctx, owner, "xiongzi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiongzi", 1)),
        // character/yunchou/skill.js:150-183
        "yaliang" => u05_exec(ctx, owner, "yaliang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yaliang", 1)),
        // character/yunchou/skill.js:184-202
        "diy_jiaoxia" => u05_exec(ctx, owner, "diy_jiaoxia", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:diy_jiaoxia", 1)),
        // character/yunchou/skill.js:203-223
        "yiesheng" => u05_exec(ctx, owner, "yiesheng", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yiesheng", 1)),
        // character/yunchou/skill.js:224-277
        "guihan" => u05_exec(ctx, owner, "guihan", 0, true, 0, 2, 2, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:guihan", 1)),
        // character/yunchou/skill.js:278-296
        "luweiyan" => u05_exec(ctx, owner, "luweiyan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shuigong")),
        // character/yunchou/skill.js:297-328
        "luweiyan2" => u05_exec(ctx, owner, "luweiyan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:luweiyan2", 1)),
        // character/yunchou/skill.js:329-347
        "jieyan" => u05_exec(ctx, owner, "jieyan", 0, false, 0, 0, 0, -1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:jieyan", 1)),
        // character/yunchou/skill.js:348-370
        "honglian" => u05_exec(ctx, owner, "honglian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:honglian", 1)),
        // character/yunchou/skill.js:371-401
        "diyguhuo" => u05_exec(ctx, owner, "diyguhuo", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/yunchou/skill.js:402-428
        "diychanyuan" => u05_exec(ctx, owner, "diychanyuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/yunchou/skill.js:429-457
        "zonghuo" => u05_exec(ctx, owner, "zonghuo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zonghuo", 1)),
        // character/yunchou/skill.js:458-488
        "shaoying" => u05_exec(ctx, owner, "shaoying", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:shaoying", 1)),
        // character/yunchou/skill.js:489-554
        "xinfu_qiaosi" => u05_exec(ctx, owner, "xinfu_qiaosi", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xinfu_qiaosi", 1)),
        // card/yunchou/card.js:4-61
        "caochuanjiejian" => u05_exec(ctx, owner, "caochuanjiejian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/yunchou/card.js:62-131
        "geanguanhuo" => u05_exec(ctx, owner, "geanguanhuo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/yunchou/card.js:132-244
        "shezhanqunru" => u05_exec(ctx, owner, "shezhanqunru", 0, false, 0, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::PindianOther),
        // card/yunchou/card.js:266-287
        "chenhuodajie" => u05_exec(ctx, owner, "chenhuodajie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/yunchou/card.js:288-332
        "fudichouxin" => u05_exec(ctx, owner, "fudichouxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::PindianOther),
        // card/yunchou/card.js:333-362
        "shuigong" => u05_exec(ctx, owner, "shuigong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/yunchou/card.js:363-441
        "toulianghuanzhu" => u05_exec(ctx, owner, "toulianghuanzhu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:toulianghuanzhu", 1)),
        // card/yunchou/card.js:442-504
        "huoshan" => u05_exec(ctx, owner, "huoshan", 0, false, 0, 0, 0, -2, DamageNature::Fire, 0, 0, U05Extra::JudgeEffect("u05_judge:huoshan")),
        // card/yunchou/card.js:505-569
        "hongshui" => u05_exec(ctx, owner, "hongshui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/zhenfa/card.js:24-69
        "changshezhen" => u05_exec(ctx, owner, "changshezhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:changshezhen", 1)),
        // card/zhenfa/card.js:70-95
        "tianfuzhen" => u05_exec(ctx, owner, "tianfuzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/zhenfa/card.js:96-122
        "dizaizhen" => u05_exec(ctx, owner, "dizaizhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:dizaizhen", 1)),
        // card/zhenfa/card.js:123-143
        "fengyangzhen" => u05_exec(ctx, owner, "fengyangzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:fengyangzhen", 1)),
        // card/zhenfa/card.js:144-164
        "yunchuizhen" => u05_exec(ctx, owner, "yunchuizhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:yunchuizhen", 1)),
        // card/zhenfa/card.js:165-203
        "qixingzhen" => u05_exec(ctx, owner, "qixingzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/zhenfa/card.js:204-236
        "shepanzhen" => u05_exec(ctx, owner, "shepanzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shepanzhen", 1)),
        // card/zhenfa/card.js:237-268
        "longfeizhen" => u05_exec(ctx, owner, "longfeizhen", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/zhenfa/card.js:269-319
        "huyizhen" => u05_exec(ctx, owner, "huyizhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:huyizhen", 1)),
        // card/zhenfa/card.js:320-365
        "niaoxiangzhen" => u05_exec(ctx, owner, "niaoxiangzhen", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:niaoxiangzhen", 1)),
        _ => return false,
    }
    true
}
