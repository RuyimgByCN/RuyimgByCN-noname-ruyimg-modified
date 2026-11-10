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
        // card/swd/skill.js:128-155
        "g_yuchan_equip" => u05_exec(ctx, owner, "g_yuchan_equip", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_yuchan_equip", 1)),
        // card/swd/skill.js:328-365
        "yuruyi" => u05_exec(ctx, owner, "yuruyi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuruyi", 1)),
        // card/swd/skill.js:366-412
        "shuchui" => u05_exec(ctx, owner, "shuchui", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shuchui", 1)),
        // card/swd/skill.js:446-484
        "gouhunluo" => u05_exec(ctx, owner, "gouhunluo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
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
        // card/swd/skill.js:1033-1044
        "xuanyuanjian3" => u05_exec(ctx, owner, "xuanyuanjian3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // card/swd/skill.js:1045-1059
        "pangufu" => { u05_mark_invoked(ctx, owner, "pangufu"); let _ = u05_prey_other(ctx, owner, false); }
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
        // card/swd/skill.js:1328-1367
        "yiluan" => u05_exec(ctx, owner, "yiluan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:yiluan")),
        // card/swd/skill.js:1401-1407
        "hslingjian_xuanfengzhiren_equip3_dist" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip3_dist", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xuanfengzhiren_equip3_dist", 1)),
        // card/swd/skill.js:1418-1424
        "hslingjian_xuanfengzhiren_equip4_dist" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip4_dist", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xuanfengzhiren_equip4_dist", 1)),
        // card/swd/skill.js:1425-1454
        "hslingjian_xuanfengzhiren_equip5" => u05_exec(ctx, owner, "hslingjian_xuanfengzhiren_equip5", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
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
        // card/swd/skill.js:2157-2190
        "hslingjian_chaofeng" => u05_exec(ctx, owner, "hslingjian_chaofeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_chaofeng", 1)),
        // card/swd/skill.js:2191-2212
        "hslingjian_xingtigaizao" => u05_exec(ctx, owner, "hslingjian_xingtigaizao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_xingtigaizao", 1)),
        // card/swd/skill.js:2213-2228
        "hslingjian_jinjilengdong" => u05_exec(ctx, owner, "hslingjian_jinjilengdong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hslingjian_jinjilengdong", 1)),
        // card/swd/skill.js:2229-2252
        "qinglonglingzhu" => u05_exec(ctx, owner, "qinglonglingzhu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/swd/skill.js:2273-2283
        "baihupifeng" => u05_exec(ctx, owner, "baihupifeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:baihupifeng", 1)),
        // card/swd/skill.js:2284-2312
        "fengxueren" => u05_exec(ctx, owner, "fengxueren", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
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
        // card/swd/skill.js:2676-2713
        "nigong2" => u05_exec(ctx, owner, "nigong2", 0, false, 0, 1, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nigong2", 1)),
        // card/swd/skill.js:2714-2731
        "nigong3" => u05_exec(ctx, owner, "nigong3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nigong3", 1)),
        // card/swd/skill.js:2732-2756
        "sadengjinhuan" => u05_exec(ctx, owner, "sadengjinhuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:sadengjinhuan")),
        // card/swd/skill.js:2757-2783
        "guiyanfadao" => u05_exec(ctx, owner, "guiyanfadao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/swd/skill.js:2792-2815
        "tianxianjiu" => u05_exec(ctx, owner, "tianxianjiu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tianxianjiu", 1)),
        // card/mtg/skill.js:5-21
        "mtg_bingheyaosai_skill" => { u05_mark_invoked(ctx, owner, "mtg_bingheyaosai_skill"); let _ = u05_discard_up_to(ctx, owner, 1); }
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
        "mduohun" => { u05_mark_invoked(ctx, owner, "mduohun"); if let Some(t) = first_other_alive(ctx.core, owner) { u05_lose_hp(ctx, t, 1); } }
        // character/mtg/skill.js:212-315
        "mtongnian" => u05_exec(ctx, owner, "mtongnian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mtongnian", 1)),
        // character/mtg/skill.js:316-328
        "mbaizhan" => { u05_mark_invoked(ctx, owner, "mbaizhan"); u05_change_hujia(ctx, owner, 1); }
        // character/mtg/skill.js:329-365
        "msilian" => u05_exec(ctx, owner, "msilian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/yunchou/skill.js:108-142
        "diykuanggu" => u05_exec(ctx, owner, "diykuanggu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:diykuanggu", 1)),
        // character/yunchou/skill.js:184-202
        "diy_jiaoxia" => u05_exec(ctx, owner, "diy_jiaoxia", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:diy_jiaoxia", 1)),
        // character/yunchou/skill.js:203-223
        "yiesheng" => u05_exec(ctx, owner, "yiesheng", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yiesheng", 1)),
        // character/yunchou/skill.js:224-277
        "guihan" => u05_exec(ctx, owner, "guihan", 0, true, 0, 2, 2, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:guihan", 1)),
        // character/yunchou/skill.js:278-296
        "luweiyan" => u05_exec(ctx, owner, "luweiyan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shuigong")),
        // character/yunchou/skill.js:329-347
        "jieyan" => u05_exec(ctx, owner, "jieyan", 0, false, 0, 0, 0, -1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:jieyan", 1)),
        // character/yunchou/skill.js:371-401
        "diyguhuo" => u05_exec(ctx, owner, "diyguhuo", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/yunchou/skill.js:402-428
        "diychanyuan" => u05_exec(ctx, owner, "diychanyuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/yunchou/skill.js:458-488
        "shaoying" => u05_exec(ctx, owner, "shaoying", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:shaoying", 1)),
        // character/yunchou/skill.js:489-554
        "xinfu_qiaosi" => u05_exec(ctx, owner, "xinfu_qiaosi", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xinfu_qiaosi", 1)),
        _ => return false,
    }
    true
}
