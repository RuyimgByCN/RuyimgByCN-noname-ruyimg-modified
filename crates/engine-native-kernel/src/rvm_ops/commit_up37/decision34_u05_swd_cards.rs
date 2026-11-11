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
                if let Some(t) = hostile_target(ctx, owner) {
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
        "qiankundai" =>  { u05_mark_invoked(ctx, owner, "qiankundai"); set_flag(ctx.core, owner, "u05_mod:qiankundai", 1); },
        // card/swd/skill.js:13-29
        "g_hufu_sha" =>  { u05_mark_invoked(ctx, owner, "g_hufu_sha"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // card/swd/skill.js:30-46
        "g_hufu_shan" =>  { u05_mark_invoked(ctx, owner, "g_hufu_shan"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shan"), 1); },
        // card/swd/skill.js:47-58
        "g_hufu_jiu" =>  { u05_mark_invoked(ctx, owner, "g_hufu_jiu"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "jiu"), 1); },
        // card/swd/skill.js:59-69
        "zhiluxiaohu" =>  { u05_mark_invoked(ctx, owner, "zhiluxiaohu"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:70-83
        "zhufangshenshi" =>  { u05_mark_invoked(ctx, owner, "zhufangshenshi"); set_flag(ctx.core, owner, "u05_mod:zhufangshenshi", 1); },
        // card/swd/skill.js:84-94
        "g_zhufangshenshi" =>  { u05_mark_invoked(ctx, owner, "g_zhufangshenshi"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:95-112
        "huanpodan_skill" =>  { u05_mark_invoked(ctx, owner, "huanpodan_skill"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:128-155
        "g_yuchan_equip" =>  { u05_mark_invoked(ctx, owner, "g_yuchan_equip"); if !u05_check_usable(ctx, owner, "g_yuchan_equip", 1) { return true; } if !u05_pay_discard(ctx, owner, "g_yuchan_equip", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // card/swd/skill.js:328-365
        "yuruyi" =>  { u05_mark_invoked(ctx, owner, "yuruyi"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:366-412
        "shuchui" =>  { u05_mark_invoked(ctx, owner, "shuchui"); if !u05_check_usable(ctx, owner, "shuchui", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/swd/skill.js:446-484
        "gouhunluo" =>  { u05_mark_invoked(ctx, owner, "gouhunluo"); u05_lose_hp(ctx, owner, 1); },
        // card/swd/skill.js:513-526
        "g_qinglongzhigui" =>  { u05_mark_invoked(ctx, owner, "g_qinglongzhigui"); ctx.start_draw(owner, 2); },
        // card/swd/skill.js:527-553
        "g_baishouzhihu" =>  { u05_mark_invoked(ctx, owner, "g_baishouzhihu"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:554-571
        "g_zhuquezhizhang" =>  { u05_mark_invoked(ctx, owner, "g_zhuquezhizhang"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:572-582
        "g_xuanwuzhihuang" =>  { u05_mark_invoked(ctx, owner, "g_xuanwuzhihuang"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:583-594
        "g_huanglinzhicong" =>  { u05_mark_invoked(ctx, owner, "g_huanglinzhicong"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:595-619
        "g_cangchizhibi" =>  { u05_mark_invoked(ctx, owner, "g_cangchizhibi"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:620-638
        "cangchizhibi_equip1" =>  { u05_mark_invoked(ctx, owner, "cangchizhibi_equip1"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:639-641
        "cangchizhibi_equip2" =>  { u05_mark_invoked(ctx, owner, "cangchizhibi_equip2"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:642-644
        "cangchizhibi_equip3" =>  { u05_mark_invoked(ctx, owner, "cangchizhibi_equip3"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:645-647
        "cangchizhibi_equip4" =>  { u05_mark_invoked(ctx, owner, "cangchizhibi_equip4"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:648-650
        "cangchizhibi_equip5" =>  { u05_mark_invoked(ctx, owner, "cangchizhibi_equip5"); if let Some(t) = hostile_target(ctx, owner) { u05_toggle_link(ctx, t); } },
        // card/swd/skill.js:651-669
        "huanglinzhicong_equip1" =>  { u05_mark_invoked(ctx, owner, "huanglinzhicong_equip1"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:670-672
        "huanglinzhicong_equip2" =>  { u05_mark_invoked(ctx, owner, "huanglinzhicong_equip2"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:673-675
        "huanglinzhicong_equip3" =>  { u05_mark_invoked(ctx, owner, "huanglinzhicong_equip3"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:676-678
        "huanglinzhicong_equip4" =>  { u05_mark_invoked(ctx, owner, "huanglinzhicong_equip4"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:679-681
        "huanglinzhicong_equip5" =>  { u05_mark_invoked(ctx, owner, "huanglinzhicong_equip5"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:682-703
        "xuanwuzhihuang_equip1" =>  { u05_mark_invoked(ctx, owner, "xuanwuzhihuang_equip1"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:704-706
        "xuanwuzhihuang_equip2" =>  { u05_mark_invoked(ctx, owner, "xuanwuzhihuang_equip2"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:707-709
        "xuanwuzhihuang_equip3" =>  { u05_mark_invoked(ctx, owner, "xuanwuzhihuang_equip3"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:710-712
        "xuanwuzhihuang_equip4" =>  { u05_mark_invoked(ctx, owner, "xuanwuzhihuang_equip4"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:713-715
        "xuanwuzhihuang_equip5" =>  { u05_mark_invoked(ctx, owner, "xuanwuzhihuang_equip5"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:716-753
        "zhuquezhizhang_equip1" =>  { u05_mark_invoked(ctx, owner, "zhuquezhizhang_equip1"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:754-756
        "zhuquezhizhang_equip2" =>  { u05_mark_invoked(ctx, owner, "zhuquezhizhang_equip2"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:757-759
        "zhuquezhizhang_equip3" =>  { u05_mark_invoked(ctx, owner, "zhuquezhizhang_equip3"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:760-762
        "zhuquezhizhang_equip4" =>  { u05_mark_invoked(ctx, owner, "zhuquezhizhang_equip4"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:763-765
        "zhuquezhizhang_equip5" =>  { u05_mark_invoked(ctx, owner, "zhuquezhizhang_equip5"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // card/swd/skill.js:766-788
        "baishouzhihu_equip1" =>  { u05_mark_invoked(ctx, owner, "baishouzhihu_equip1"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:789-791
        "baishouzhihu_equip2" =>  { u05_mark_invoked(ctx, owner, "baishouzhihu_equip2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:792-794
        "baishouzhihu_equip3" =>  { u05_mark_invoked(ctx, owner, "baishouzhihu_equip3"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:795-797
        "baishouzhihu_equip4" =>  { u05_mark_invoked(ctx, owner, "baishouzhihu_equip4"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:798-800
        "baishouzhihu_equip5" =>  { u05_mark_invoked(ctx, owner, "baishouzhihu_equip5"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:801-807
        "qinglongzhigui_equip1" =>  { u05_mark_invoked(ctx, owner, "qinglongzhigui_equip1"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:808-810
        "qinglongzhigui_equip2" =>  { u05_mark_invoked(ctx, owner, "qinglongzhigui_equip2"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:811-813
        "qinglongzhigui_equip3" =>  { u05_mark_invoked(ctx, owner, "qinglongzhigui_equip3"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:814-816
        "qinglongzhigui_equip4" =>  { u05_mark_invoked(ctx, owner, "qinglongzhigui_equip4"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:817-819
        "qinglongzhigui_equip5" =>  { u05_mark_invoked(ctx, owner, "qinglongzhigui_equip5"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:820-862
        "kunlunjingc" =>  { u05_mark_invoked(ctx, owner, "kunlunjingc"); if !u05_check_usable(ctx, owner, "kunlunjingc", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/swd/skill.js:863-907
        "lianhua" =>  { u05_mark_invoked(ctx, owner, "lianhua"); if !u05_check_usable(ctx, owner, "lianhua", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/swd/skill.js:908-949
        "shouna" =>  { u05_mark_invoked(ctx, owner, "shouna"); if !u05_check_usable(ctx, owner, "shouna", 1) { return true; } if !u05_pay_discard(ctx, owner, "shouna", 1, true) { return true; } let _ = u05_store_pile(ctx, owner, "u05_pile:shouna"); },
        // card/swd/skill.js:1033-1044
        "xuanyuanjian3" =>  { u05_mark_invoked(ctx, owner, "xuanyuanjian3"); u05_lose_hp(ctx, owner, 1); },
        // card/swd/skill.js:1045-1059
        "pangufu" => { u05_mark_invoked(ctx, owner, "pangufu"); let _ = u05_prey_other(ctx, owner, false); }
        // card/swd/skill.js:1060-1144
        "haotianta" =>  { u05_mark_invoked(ctx, owner, "haotianta"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:haotianta", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // card/swd/skill.js:1145-1171
        "shennongding" =>  { u05_mark_invoked(ctx, owner, "shennongding"); if !u05_check_usable(ctx, owner, "shennongding", 1) { return true; } if !u05_pay_discard(ctx, owner, "shennongding", 2, true) { return true; } u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:1172-1186
        "kongdongyin" =>  { u05_mark_invoked(ctx, owner, "kongdongyin"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:1187-1208
        "nvwashi" =>  { u05_mark_invoked(ctx, owner, "nvwashi"); u05_recover(ctx, owner, 1); u05_lose_hp(ctx, owner, 1); },
        // card/swd/skill.js:1209-1264
        "kongxin" =>  { u05_mark_invoked(ctx, owner, "kongxin"); if !u05_check_usable(ctx, owner, "kongxin", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:1265-1280
        "kongxin2" =>  { u05_mark_invoked(ctx, owner, "kongxin2"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // card/swd/skill.js:1328-1367
        "yiluan" =>  { u05_mark_invoked(ctx, owner, "yiluan"); if !u05_check_usable(ctx, owner, "yiluan", 1) { return true; } if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:yiluan", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // card/swd/skill.js:1368-1379
        "qinglianxindeng" =>  { u05_mark_invoked(ctx, owner, "qinglianxindeng"); set_flag(ctx.core, owner, "u05_mod:qinglianxindeng", 1); },
        // card/swd/skill.js:1380-1391
        "hslingjian_xuanfengzhiren_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xuanfengzhiren_equip1"); set_flag(ctx.core, owner, "u05_mod:hslingjian_xuanfengzhiren_equip1", 1); },
        // card/swd/skill.js:1392-1400
        "hslingjian_xuanfengzhiren_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xuanfengzhiren_equip2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:1401-1407
        "hslingjian_xuanfengzhiren_equip3" | "hslingjian_xuanfengzhiren_equip3_dist" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:hslingjian_xuanfengzhiren_equip3_dist", 1); },
        // card/swd/skill.js:1418-1424
        "hslingjian_xuanfengzhiren_equip4" | "hslingjian_xuanfengzhiren_equip4_dist" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:hslingjian_xuanfengzhiren_equip4_dist", 1); },
        // card/swd/skill.js:1425-1454
        "hslingjian_xuanfengzhiren_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xuanfengzhiren_equip5"); if !u05_check_usable(ctx, owner, "hslingjian_xuanfengzhiren_equip5", 1) { return true; } if !u05_pay_discard(ctx, owner, "hslingjian_xuanfengzhiren_equip5", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:1455-1468
        "hslingjian_zhongxinghujia_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_zhongxinghujia_equip1"); u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:1469-1482
        "hslingjian_zhongxinghujia_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_zhongxinghujia_equip2"); set_flag(ctx.core, owner, "u05_mod:molicha", 1); },
        // card/swd/skill.js:1483-1491
        "hslingjian_zhongxinghujia_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_zhongxinghujia_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_zhongxinghujia_equip3", 1); },
        // card/swd/skill.js:1492-1500
        "hslingjian_zhongxinghujia_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_zhongxinghujia_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_zhongxinghujia_equip4", 1); },
        // card/swd/skill.js:1501-1531
        "hslingjian_zhongxinghujia_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_zhongxinghujia_equip5"); if !u05_check_usable(ctx, owner, "hslingjian_zhongxinghujia_equip5", 1) { return true; } if !u05_pay_discard(ctx, owner, "hslingjian_zhongxinghujia_equip5", 2, true) { return true; } u05_change_hujia(ctx, owner, 1); },
        // card/swd/skill.js:1532-1554
        "hslingjian_jinjilengdong_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong_equip1"); ctx.start_draw(owner, 2); u05_turn_over(ctx, owner); },
        // card/swd/skill.js:1555-1575
        "hslingjian_jinjilengdong_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong_equip2"); u05_turn_over(ctx, owner); },
        // card/swd/skill.js:1576-1584
        "hslingjian_jinjilengdong_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_jinjilengdong_equip3", 1); },
        // card/swd/skill.js:1585-1593
        "hslingjian_jinjilengdong_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_jinjilengdong_equip4", 1); },
        // card/swd/skill.js:1594-1618
        "hslingjian_jinjilengdong_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong_equip5"); u05_turn_over(ctx, owner); },
        // card/swd/skill.js:1619-1638
        "hslingjian_yinmilichang_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_yinmilichang_equip1"); set_flag(ctx.core, owner, "u05_mod:yunchuizhen", 1); },
        // card/swd/skill.js:1639-1658
        "hslingjian_yinmilichang_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_yinmilichang_equip2"); set_flag(ctx.core, owner, "u05_mod:hslingjian_yinmilichang_equip5", 1); },
        // card/swd/skill.js:1659-1667
        "hslingjian_yinmilichang_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_yinmilichang_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_yinmilichang_equip3", 1); },
        // card/swd/skill.js:1668-1676
        "hslingjian_yinmilichang_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_yinmilichang_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_yinmilichang_equip4", 1); },
        // card/swd/skill.js:1677-1697
        "hslingjian_yinmilichang_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_yinmilichang_equip5"); set_flag(ctx.core, owner, "u05_mod:hslingjian_yinmilichang_equip5", 1); },
        // card/swd/skill.js:1698-1707
        "hslingjian_xingtigaizao_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao_equip1"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:1708-1717
        "hslingjian_xingtigaizao_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao_equip2"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:1718-1727
        "hslingjian_xingtigaizao_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_xingtigaizao_equip3", 1); },
        // card/swd/skill.js:1728-1737
        "hslingjian_xingtigaizao_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_xingtigaizao_equip4", 1); },
        // card/swd/skill.js:1738-1749
        "hslingjian_xingtigaizao_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao_equip5"); set_flag(ctx.core, owner, "u05_mod:hslingjian_xingtigaizao_equip5", 1); },
        // card/swd/skill.js:1750-1759
        "hslingjian_shengxiuhaojiao_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shengxiuhaojiao_equip1"); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:hslingjian_chaofeng", 1); } },
        // card/swd/skill.js:1760-1768
        "hslingjian_shengxiuhaojiao_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shengxiuhaojiao_equip2"); set_flag(ctx.core, owner, "u05_mod:hslingjian_shengxiuhaojiao_equip2", 1); },
        // card/swd/skill.js:1769-1777
        "hslingjian_shengxiuhaojiao_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shengxiuhaojiao_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_shengxiuhaojiao_equip3", 1); },
        // card/swd/skill.js:1778-1786
        "hslingjian_shengxiuhaojiao_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shengxiuhaojiao_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_shengxiuhaojiao_equip4", 1); },
        // card/swd/skill.js:1787-1815
        "hslingjian_shengxiuhaojiao_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shengxiuhaojiao_equip5"); if !u05_check_usable(ctx, owner, "hslingjian_shengxiuhaojiao_equip5", 1) { return true; } if !u05_pay_discard(ctx, owner, "hslingjian_shengxiuhaojiao_equip5", 2, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:hslingjian_chaofeng", 1); } },
        // card/swd/skill.js:1816-1825
        "hslingjian_shijianhuisu_equip1" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shijianhuisu_equip1"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:1826-1835
        "hslingjian_shijianhuisu_equip2" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shijianhuisu_equip2"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:1836-1844
        "hslingjian_shijianhuisu_equip3" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shijianhuisu_equip3"); set_flag(ctx.core, owner, "u05_mod:hslingjian_shijianhuisu_equip3", 1); },
        // card/swd/skill.js:1845-1853
        "hslingjian_shijianhuisu_equip4" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shijianhuisu_equip4"); set_flag(ctx.core, owner, "u05_mod:hslingjian_shijianhuisu_equip4", 1); },
        // card/swd/skill.js:1854-1882
        "hslingjian_shijianhuisu_equip5" =>  { u05_mark_invoked(ctx, owner, "hslingjian_shijianhuisu_equip5"); if !u05_check_usable(ctx, owner, "hslingjian_shijianhuisu_equip5", 1) { return true; } if !u05_pay_discard(ctx, owner, "hslingjian_shijianhuisu_equip5", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // card/swd/skill.js:2157-2190
        "hslingjian_chaofeng" =>  { u05_mark_invoked(ctx, owner, "hslingjian_chaofeng"); set_flag(ctx.core, owner, "u05_mod:hslingjian_chaofeng", 1); },
        // card/swd/skill.js:2191-2212
        "hslingjian_xingtigaizao" =>  { u05_mark_invoked(ctx, owner, "hslingjian_xingtigaizao"); set_flag(ctx.core, owner, "u05_mod:hslingjian_xingtigaizao", 1); },
        // card/swd/skill.js:2213-2228
        "hslingjian_jinjilengdong" =>  { u05_mark_invoked(ctx, owner, "hslingjian_jinjilengdong"); set_flag(ctx.core, owner, "u05_mod:hslingjian_jinjilengdong", 1); },
        // card/swd/skill.js:2229-2252
        "qinglonglingzhu" =>  { u05_mark_invoked(ctx, owner, "qinglonglingzhu"); let _ = u05_prey_other(ctx, owner, true); },
        // card/swd/skill.js:2253-2272
        "guangshatianyi" =>  { u05_mark_invoked(ctx, owner, "guangshatianyi"); u05_change_hujia(ctx, owner, 1); set_flag(ctx.core, owner, "u05_mod:heilonglinpian", 1); },
        // card/swd/skill.js:2273-2283
        "baihupifeng" =>  { u05_mark_invoked(ctx, owner, "baihupifeng"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:2284-2312
        "fengxueren" =>  { u05_mark_invoked(ctx, owner, "fengxueren"); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // card/swd/skill.js:2324-2337
        "chilongya2" =>  { u05_mark_invoked(ctx, owner, "chilongya2"); set_flag(ctx.core, owner, "u05_buff:chilongya2", 1); },
        // card/swd/skill.js:2338-2346
        "chilongfengxue" =>  { u05_mark_invoked(ctx, owner, "chilongfengxue"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:2347-2402
        "shentou" =>  { u05_mark_invoked(ctx, owner, "shentou"); if !u05_check_usable(ctx, owner, "shentou", 1) { return true; } if !u05_pay_discard(ctx, owner, "shentou", 1, true) { return true; } if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:shentou", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // card/swd/skill.js:2403-2530
        "old_longfan" =>  { u05_mark_invoked(ctx, owner, "old_longfan"); if !u05_check_usable(ctx, owner, "old_longfan", 1) { return true; } u05_recover(ctx, owner, 1); u05_damage_other(ctx, owner, 1, DamageNature::Normal); ctx.start_draw(owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // card/swd/skill.js:2531-2579
        "longfan" =>  { u05_mark_invoked(ctx, owner, "longfan"); if !u05_check_usable(ctx, owner, "longfan", 1) { return true; } u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:longfan", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // card/swd/skill.js:2580-2615
        "touzhi" =>  { u05_mark_invoked(ctx, owner, "touzhi"); if !u05_check_usable(ctx, owner, "touzhi", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // card/swd/skill.js:2616-2626
        "xixue" =>  { u05_mark_invoked(ctx, owner, "xixue"); u05_recover(ctx, owner, 1); },
        // card/swd/skill.js:2676-2713
        "nigong2" =>  { u05_mark_invoked(ctx, owner, "nigong2"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:2714-2731
        "nigong3" =>  { u05_mark_invoked(ctx, owner, "nigong3"); ctx.start_draw(owner, 1); },
        // card/swd/skill.js:2732-2756
        "sadengjinhuan" =>  { u05_mark_invoked(ctx, owner, "sadengjinhuan"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:sadengjinhuan", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } },
        // card/swd/skill.js:2757-2783
        "guiyanfadao" =>  { u05_mark_invoked(ctx, owner, "guiyanfadao"); if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // card/swd/skill.js:2784-2791
        "guiyanfadao2" =>  { u05_mark_invoked(ctx, owner, "guiyanfadao2"); if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // card/swd/skill.js:2792-2815
        "tianxianjiu" =>  { u05_mark_invoked(ctx, owner, "tianxianjiu"); ctx.start_draw(owner, 2); },
        // card/mtg/skill.js:5-21
        "mtg_bingheyaosai_skill" => { u05_mark_invoked(ctx, owner, "mtg_bingheyaosai_skill"); let _ = u05_discard_up_to(ctx, owner, 1); }
        // card/mtg/skill.js:22-31
        "mtg_duzhao_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_duzhao_skill"); ctx.start_draw(owner, 1); },
        // card/mtg/skill.js:32-43
        "mtg_shamolvzhou_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_shamolvzhou_skill"); set_flag(ctx.core, owner, "u05_mod:mtg_shamolvzhou_skill", 1); },
        // card/mtg/skill.js:44-57
        "mtg_haidao_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_haidao_skill"); u05_recover(ctx, owner, 1); },
        // card/mtg/skill.js:58-92
        "mtg_yixialan_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_yixialan_skill"); if !u05_check_usable(ctx, owner, "mtg_yixialan_skill", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/mtg/skill.js:93-110
        "mtg_shuimomuxue_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_shuimomuxue_skill"); ctx.start_draw(owner, 1); set_flag(ctx.core, owner, "u05_mod:mtg_shuimomuxue_skill", 1); },
        // card/mtg/skill.js:111-126
        "mtg_feixu_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_feixu_skill"); ctx.start_draw(owner, 1); },
        // card/mtg/skill.js:127-149
        "mtg_youlin_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_youlin_skill"); if !u05_check_usable(ctx, owner, "mtg_youlin_skill", 1) { return true; } ctx.start_draw(owner, 1); },
        // card/mtg/skill.js:150-187
        "mtg_linzhongjianta_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_linzhongjianta_skill"); if !u05_check_usable(ctx, owner, "mtg_linzhongjianta_skill", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // card/mtg/skill.js:188-210
        "mtg_cangbaohaiwan_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_cangbaohaiwan_skill"); ctx.start_draw(owner, 1); },
        // card/mtg/skill.js:211-222
        "mtg_longlushanfeng_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_longlushanfeng_skill"); set_flag(ctx.core, owner, "u05_mod:mtg_longlushanfeng_skill", 1); },
        // card/mtg/skill.js:223-245
        "mtg_lindixiliu_skill" =>  { u05_mark_invoked(ctx, owner, "mtg_lindixiliu_skill"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/mtg/skill.js:5-33
        "mduohun" => { u05_mark_invoked(ctx, owner, "mduohun"); if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } }
        // character/mtg/skill.js:212-315
        "mtongnian" =>  { u05_mark_invoked(ctx, owner, "mtongnian"); ctx.start_draw(owner, 1); },
        // character/mtg/skill.js:316-328
        "mbaizhan" => { u05_mark_invoked(ctx, owner, "mbaizhan"); u05_change_hujia(ctx, owner, 1); }
        // character/mtg/skill.js:329-365
        "msilian" =>  { u05_mark_invoked(ctx, owner, "msilian"); u05_lose_hp(ctx, owner, 1); },
        // character/yunchou/skill.js:108-142
        "diykuanggu" =>  { u05_mark_invoked(ctx, owner, "diykuanggu"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); },
        // character/yunchou/skill.js:184-202
        "diy_jiaoxia" =>  { u05_mark_invoked(ctx, owner, "diy_jiaoxia"); ctx.start_draw(owner, 1); },
        // character/yunchou/skill.js:203-223
        "yiesheng" =>  { u05_mark_invoked(ctx, owner, "yiesheng"); if !u05_pay_discard(ctx, owner, "yiesheng", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/yunchou/skill.js:224-277
        "guihan" =>  { u05_mark_invoked(ctx, owner, "guihan"); if !u05_check_limited(ctx, owner, "guihan") { return true; } u05_recover(ctx, owner, 1); ctx.start_draw(owner, 2); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 2); } },
        // character/yunchou/skill.js:278-296
        "luweiyan" =>  { u05_mark_invoked(ctx, owner, "luweiyan"); if !u05_check_usable(ctx, owner, "luweiyan", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shuigong"), 1); },
        // character/yunchou/skill.js:329-347
        "jieyan" =>  { u05_mark_invoked(ctx, owner, "jieyan"); u05_damage_aoe(ctx, owner, 1, DamageNature::Fire); },
        // character/yunchou/skill.js:371-401
        "diyguhuo" =>  { u05_mark_invoked(ctx, owner, "diyguhuo"); ctx.start_draw(owner, 2); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/yunchou/skill.js:402-428
        "diychanyuan" =>  { u05_mark_invoked(ctx, owner, "diychanyuan"); u05_change_max_hp(ctx, owner, -(1)); },
        // character/yunchou/skill.js:458-488
        "shaoying" =>  { u05_mark_invoked(ctx, owner, "shaoying"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/yunchou/skill.js:489-554
        "xinfu_qiaosi" =>  { u05_mark_invoked(ctx, owner, "xinfu_qiaosi"); if !u05_check_usable(ctx, owner, "xinfu_qiaosi", 1) { return true; } if !u05_pay_discard(ctx, owner, "xinfu_qiaosi", 1, true) { return true; } ctx.start_draw(owner, 2); },
        // card/swd/skill.js:426-445
        "xuejibingbao" =>  { u05_mark_invoked(ctx, owner, "xuejibingbao"); if !u05_pay_discard(ctx, owner, "xuejibingbao", 1, true) { return true; } set_flag(ctx.core, owner, "u05_buff:xuejibingbao", 1); },
        // card/swd/skill.js:1917-2156
        "lingjianduanzao" | "_lingjianduanzao" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_limited(ctx, owner, skill) { return true; } if !u05_check_usable(ctx, owner, skill, 1) { return true; } if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } },
        // card/swd/skill.js:2253-2272
        "xingjunyan" =>  { u05_mark_invoked(ctx, owner, "xingjunyan"); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // card/swd/skill.js:2313-2323
        "chilongya" =>  { u05_mark_invoked(ctx, owner, "chilongya"); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // card/yunchou/skill.js:7-28
        "suolianjia" => {
            u05_mark_invoked(ctx, owner, "suolianjia");
            u05_toggle_link(ctx, owner);
            set_flag(ctx.core, owner, "u05_mod:mingkong2", 1);
        }
        // character/mtg/skill.js:34-77
        "lingyong" =>  { u05_mark_invoked(ctx, owner, "lingyong"); if !u05_pay_discard(ctx, owner, "lingyong", 2, true) { return true; } },
        // character/mtg/skill.js:78-172
        "mhuanyi" =>  { u05_mark_invoked(ctx, owner, "mhuanyi"); set_flag(ctx.core, owner, "l25_tianxiang_deflected", 1); },
        // character/mtg/skill.js:173-211
        "msuoling" =>  { u05_mark_invoked(ctx, owner, "msuoling"); if !u05_check_usable(ctx, owner, "msuoling", 1) { return true; } if !u05_pay_discard(ctx, owner, "msuoling", 1, true) { return true; } },
        _ => return false,
    }
    true
}
