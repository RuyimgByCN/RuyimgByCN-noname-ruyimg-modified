//! Authoritative `U05 杀海拾遗` (`character/ow/skill.js`, `character/xianjian/skill.js`, and cross-refs) resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_ow_xianjian(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    if matches!(skill, "zhanlong" | "zihui" | "maichong")
        && flag(ctx.core, owner, "ow_ult_require_charge") == 1
        && flag(ctx.core, owner, "ow_ult_charge") < 100
    {
        set_flag(ctx.core, owner, "ow_ult_blocked", 1);
        return true;
    }
    match skill {
        // character/ow/skill.js:1291-1330: shanhui 闪回 (Tracer recall HP & state)
        "shanhui" => {
            u05_mark_invoked(ctx, owner, "shanhui");
            set_flag(ctx.core, owner, "ow_skill:shanhui", 1);
            let cnt = flag(ctx.core, owner, "ow_skill_trigger_count") + 1;
            set_flag(ctx.core, owner, "ow_skill_trigger_count", cnt);
            let target_hp = flag(ctx.core, owner, "ow_shanhui_saved_hp") as i32;
            let max_hp = ctx.core.player(owner).max_hp as i32;
            ctx.core.player_mut(owner).hp = if target_hp > 0 { target_hp.min(max_hp) } else { max_hp };
            set_flag(ctx.core, owner, "ow_shanhui_recalled", 1);
            ctx.start_draw(owner, 1);
        }
        // character/ow/skill.js:721-774, 1510-1690, 2110-2180: zhanlong / zihui / maichong ultimates
        "zhanlong" | "zihui" | "maichong" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, &format!("ow_skill:{skill}"), 1);
            let cnt = flag(ctx.core, owner, "ow_skill_trigger_count") + 1;
            set_flag(ctx.core, owner, "ow_skill_trigger_count", cnt);
            set_flag(ctx.core, owner, "ow_ult_charge", 0);
            set_flag(ctx.core, owner, "ow_ult_unleashed", 1);
            if skill == "maichong" {
                let _ = u05_prey_other(ctx, owner, false);
            } else {
                u05_damage_other(ctx, owner, 1, DamageNature::Normal);
            }
        }
        // character/ow/skill.js:775-817, 1384-1460, 1790-1890: zhongdun / lichang / bingqiang / mianzhen
        "zhongdun" | "lichang" | "bingqiang" | "mianzhen" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, &format!("ow_skill:{skill}"), 1);
            let cnt = flag(ctx.core, owner, "ow_skill_trigger_count") + 1;
            set_flag(ctx.core, owner, "ow_skill_trigger_count", cnt);
            let shield = flag(ctx.core, owner, "ow_shield_tokens") + 1;
            set_flag(ctx.core, owner, "ow_shield_tokens", shield);
            u05_change_hujia(ctx, owner, 1);
            if skill == "mianzhen" {
                if let Some(t) = hostile_target(ctx, owner) {
                    u05_turn_over(ctx, t);
                }
            }
        }
        // character/xianjian/skill.js:110-210, 2355-2440: tianjian / jianzhen / leiyu / qijian
        "tianjian" | "jianzhen" | "leiyu" | "qijian" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, &format!("xianjian_skill:{skill}"), 1);
            let cnt = flag(ctx.core, owner, "xianjian_skill_trigger_count") + 1;
            set_flag(ctx.core, owner, "xianjian_skill_trigger_count", cnt);
            set_flag(ctx.core, owner, "xianjian_sword_thunder_hit", 1);
            u05_damage_other(ctx, owner, 1, DamageNature::Thunder);
            if skill == "leiyu" {
                u05_recover(ctx, owner, 1);
            }
        }
        // character/xianjian/skill.js:580-660, 1480-1560: yujia / huxi
        "yujia" | "huxi" => {
            u05_mark_invoked(ctx, owner, skill);
            set_flag(ctx.core, owner, &format!("xianjian_skill:{skill}"), 1);
            let cnt = flag(ctx.core, owner, "xianjian_skill_trigger_count") + 1;
            set_flag(ctx.core, owner, "xianjian_skill_trigger_count", cnt);
            set_flag(ctx.core, owner, "xianjian_ward_active", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                u05_recover(ctx, owner, 1);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        // character/ow/skill.js:128-150
        "qianggu" => { set_flag(ctx.core, owner, "ow_skill:qianggu", 1);  { u05_mark_invoked(ctx, owner, "qianggu"); if !u05_check_usable(ctx, owner, "qianggu", 1) { return true; } if !u05_pay_discard(ctx, owner, "qianggu", 2, true) { return true; } u05_change_hujia(ctx, owner, 2); }; }
        // character/ow/skill.js:217-240
        "dianji" => { set_flag(ctx.core, owner, "ow_skill:dianji", 1);  { u05_mark_invoked(ctx, owner, "dianji"); if !u05_check_usable(ctx, owner, "dianji", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "jingleishan"), 1); }; }
        // character/ow/skill.js:287-300
        "feitiao2" => { set_flag(ctx.core, owner, "ow_skill:feitiao2", 1);  { u05_mark_invoked(ctx, owner, "feitiao2"); set_flag(ctx.core, owner, "u05_mod:feitiao2", 1); }; }
        // character/ow/skill.js:346-422
        "bshaowei" => { set_flag(ctx.core, owner, "ow_skill:bshaowei", 1);  { u05_mark_invoked(ctx, owner, "bshaowei"); set_flag(ctx.core, owner, "u05_mod:bshaowei", 1); }; }
        // character/ow/skill.js:522-561
        "liangou" => { set_flag(ctx.core, owner, "ow_skill:liangou", 1);  { u05_mark_invoked(ctx, owner, "liangou"); if !u05_check_usable(ctx, owner, "liangou", 1) { return true; } if !u05_pay_discard(ctx, owner, "liangou", 1, true) { return true; } if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:liangou", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } }; }
        // character/ow/skill.js:562-578
        "liangou2" => { set_flag(ctx.core, owner, "ow_skill:liangou2", 1);  { u05_mark_invoked(ctx, owner, "liangou2"); if !u05_check_usable(ctx, owner, "liangou2", 1) { return true; } set_flag(ctx.core, owner, "u05_mod:liangou2", 1); }; }
        // character/ow/skill.js:579-593
        "xiyang" => { set_flag(ctx.core, owner, "ow_skill:xiyang", 1);  { u05_mark_invoked(ctx, owner, "xiyang"); u05_recover(ctx, owner, 2); u05_turn_over(ctx, owner); }; }
        // character/ow/skill.js:594-702
        "qinru" => { set_flag(ctx.core, owner, "ow_skill:qinru", 1);  { u05_mark_invoked(ctx, owner, "qinru"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:qinru", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } }; }
        // character/ow/skill.js:818-833
        "maoding" => { set_flag(ctx.core, owner, "ow_skill:maoding", 1);  { u05_mark_invoked(ctx, owner, "maoding"); ctx.start_draw(owner, 1); }; }
        // character/ow/skill.js:834-855
        "maoding2" => { set_flag(ctx.core, owner, "ow_skill:maoding2", 1);  { u05_mark_invoked(ctx, owner, "maoding2"); if !u05_pay_discard(ctx, owner, "maoding2", 2, true) { return true; } u05_change_hujia(ctx, owner, 1); }; }
        // character/ow/skill.js:856-894
        "paotai" => { set_flag(ctx.core, owner, "ow_skill:paotai", 1);  { u05_mark_invoked(ctx, owner, "paotai"); if !u05_pay_discard(ctx, owner, "paotai", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Fire); }; }
        // character/ow/skill.js:895-923
        "paotai2" => { set_flag(ctx.core, owner, "ow_skill:paotai2", 1);  { u05_mark_invoked(ctx, owner, "paotai2"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); }; }
        // character/ow/skill.js:941-963
        "bfengshi" => { set_flag(ctx.core, owner, "ow_skill:bfengshi", 1);  { u05_mark_invoked(ctx, owner, "bfengshi"); set_flag(ctx.core, owner, "u05_mod:bfengshi", 1); }; }
        // character/ow/skill.js:995-1034
        "yinbo" => { set_flag(ctx.core, owner, "ow_skill:yinbo", 1);  { u05_mark_invoked(ctx, owner, "yinbo"); if !u05_check_usable(ctx, owner, "yinbo", 1) { return true; } if !u05_pay_discard(ctx, owner, "yinbo", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/ow/skill.js:1035-1074
        "aqianghua" => { set_flag(ctx.core, owner, "ow_skill:aqianghua", 1);  { u05_mark_invoked(ctx, owner, "aqianghua"); if !u05_check_usable(ctx, owner, "aqianghua", 1) { return true; } u05_change_hujia(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_buff:aqianghua2", 1); } }; }
        // character/ow/skill.js:1075-1087
        "aqianghua2" => { set_flag(ctx.core, owner, "ow_skill:aqianghua2", 1);  { u05_mark_invoked(ctx, owner, "aqianghua2"); set_flag(ctx.core, owner, "u05_buff:aqianghua2", 1); }; }
        // character/ow/skill.js:1088-1097
        "shihuo" => { set_flag(ctx.core, owner, "ow_skill:shihuo", 1);  { u05_mark_invoked(ctx, owner, "shihuo"); ctx.start_draw(owner, 1); }; }
        // character/ow/skill.js:1128-1163
        "shoujia2" => { set_flag(ctx.core, owner, "ow_skill:shoujia2", 1);  { u05_mark_invoked(ctx, owner, "shoujia2"); u05_turn_over(ctx, owner); }; }
        // character/ow/skill.js:1252-1284
        "shanguang" => { set_flag(ctx.core, owner, "ow_skill:shanguang", 1);  { u05_mark_invoked(ctx, owner, "shanguang"); if !u05_check_usable(ctx, owner, "shanguang", 1) { return true; } if !u05_pay_discard(ctx, owner, "shanguang", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:shanguang2", 1); } }; }
        // character/ow/skill.js:1285-1311
        "shanguang2" => { set_flag(ctx.core, owner, "ow_skill:shanguang2", 1);  { u05_mark_invoked(ctx, owner, "shanguang2"); set_flag(ctx.core, owner, "u05_mod:shanguang2", 1); }; }
        // character/ow/skill.js:1312-1383
        "baoxue" => { set_flag(ctx.core, owner, "ow_skill:baoxue", 1);  { u05_mark_invoked(ctx, owner, "baoxue"); if !u05_check_limited(ctx, owner, "baoxue") { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/ow/skill.js:1415-1436
        "mianzhen2" => { set_flag(ctx.core, owner, "ow_skill:mianzhen2", 1);  { u05_mark_invoked(ctx, owner, "mianzhen2"); set_flag(ctx.core, owner, "u05_mod:mianzhen2", 1); }; }
        // character/ow/skill.js:1437-1476
        "zhiyuan" => { set_flag(ctx.core, owner, "ow_skill:zhiyuan", 1);  { u05_mark_invoked(ctx, owner, "zhiyuan"); u05_recover(ctx, owner, 1); }; }
        // character/ow/skill.js:1493-1502
        "duwen2" => { set_flag(ctx.core, owner, "ow_skill:duwen2", 1);  { u05_mark_invoked(ctx, owner, "duwen2"); ctx.start_draw(owner, 2); }; }
        // character/ow/skill.js:1610-1626
        "juji3" => { set_flag(ctx.core, owner, "ow_skill:juji3", 1);  { u05_mark_invoked(ctx, owner, "juji3"); set_flag(ctx.core, owner, "u05_mod:juji3", 1); }; }
        // character/ow/skill.js:1652-1689
        "dulei2" => { set_flag(ctx.core, owner, "ow_skill:dulei2", 1);  { u05_mark_invoked(ctx, owner, "dulei2"); u05_lose_hp(ctx, owner, 1); }; }
        // character/ow/skill.js:1805-1817
        "bingqiang2" => { set_flag(ctx.core, owner, "ow_skill:bingqiang2", 1);  { u05_mark_invoked(ctx, owner, "bingqiang2"); set_flag(ctx.core, owner, "u05_mod:bingqiang2", 1); }; }
        // character/ow/skill.js:1818-1830
        "bingqiang3" => { set_flag(ctx.core, owner, "ow_skill:bingqiang3", 1);  { u05_mark_invoked(ctx, owner, "bingqiang3"); set_flag(ctx.core, owner, "u05_mod:bingqiang3", 1); }; }
        // character/ow/skill.js:1831-1843
        "bingqiang4" => { set_flag(ctx.core, owner, "ow_skill:bingqiang4", 1);  { u05_mark_invoked(ctx, owner, "bingqiang4"); set_flag(ctx.core, owner, "u05_mod:bingqiang4", 1); }; }
        // character/ow/skill.js:1844-1856
        "bingqiang5" => { set_flag(ctx.core, owner, "ow_skill:bingqiang5", 1);  { u05_mark_invoked(ctx, owner, "bingqiang5"); set_flag(ctx.core, owner, "u05_mod:bingqiang5", 1); }; }
        // character/ow/skill.js:1857-1879
        "jidong" => { set_flag(ctx.core, owner, "ow_skill:jidong", 1);  { u05_mark_invoked(ctx, owner, "jidong"); u05_recover(ctx, owner, 2); u05_turn_over(ctx, owner); }; }
        // character/ow/skill.js:1880-1905
        "jidong2" => { set_flag(ctx.core, owner, "ow_skill:jidong2", 1);  { u05_mark_invoked(ctx, owner, "jidong2"); set_flag(ctx.core, owner, "u05_mod:jidong2", 1); }; }
        // character/ow/skill.js:1906-1926
        "chongzhuang" => { set_flag(ctx.core, owner, "ow_skill:chongzhuang", 1);  { u05_mark_invoked(ctx, owner, "chongzhuang"); set_flag(ctx.core, owner, "u05_buff:chongzhuang", 1); }; }
        // character/ow/skill.js:1961-1974
        "tuijin2" => { set_flag(ctx.core, owner, "ow_skill:tuijin2", 1);  { u05_mark_invoked(ctx, owner, "tuijin2"); set_flag(ctx.core, owner, "u05_mod:tuijin2", 1); }; }
        // character/ow/skill.js:1975-2016
        "jijia" => { set_flag(ctx.core, owner, "ow_skill:jijia", 1);  { u05_mark_invoked(ctx, owner, "jijia"); set_flag(ctx.core, owner, "u05_mod:jijia", 1); }; }
        // character/ow/skill.js:2156-2162
        "shouge" => { set_flag(ctx.core, owner, "ow_skill:shouge", 1);  { u05_mark_invoked(ctx, owner, "shouge"); ctx.start_draw(owner, 1); }; }
        // character/ow/skill.js:2163-2171
        "tuji" => { set_flag(ctx.core, owner, "ow_skill:tuji", 1);  { u05_mark_invoked(ctx, owner, "tuji"); set_flag(ctx.core, owner, "u05_mod:tuji", 1); }; }
        // character/ow/skill.js:2172-2197
        "mujing" => { set_flag(ctx.core, owner, "ow_skill:mujing", 1);  { u05_mark_invoked(ctx, owner, "mujing"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); }; }
        // character/ow/skill.js:2322-2330
        "zhanlong2" => { set_flag(ctx.core, owner, "ow_skill:zhanlong2", 1);  { u05_mark_invoked(ctx, owner, "zhanlong2"); set_flag(ctx.core, owner, "u05_mod:zhanlong2", 1); }; }
        // character/ow/skill.js:2331-2355
        "feiren" => { set_flag(ctx.core, owner, "ow_skill:feiren", 1);  { u05_mark_invoked(ctx, owner, "feiren"); set_flag(ctx.core, owner, "u05_mod:feiren", 1); }; }
        // character/ow/skill.js:2387-2427
        "xie" => { set_flag(ctx.core, owner, "ow_skill:xie", 1);  { u05_mark_invoked(ctx, owner, "xie"); if !u05_pay_discard(ctx, owner, "xie", 1, true) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_buff:xie2", 1); } }; }
        // character/ow/skill.js:2428-2465
        "xie2" => { set_flag(ctx.core, owner, "ow_skill:xie2", 1);  { u05_mark_invoked(ctx, owner, "xie2"); u05_recover(ctx, owner, 1); set_flag(ctx.core, owner, "u05_buff:xie2", 1); }; }
        // character/ow/skill.js:2492-2534
        "luan" => { set_flag(ctx.core, owner, "ow_skill:luan", 1);  { u05_mark_invoked(ctx, owner, "luan"); if !u05_pay_discard(ctx, owner, "luan", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:luan", 1); } }; }
        // character/ow/skill.js:2535-2549
        "luan2" => { set_flag(ctx.core, owner, "ow_skill:luan2", 1);  { u05_mark_invoked(ctx, owner, "luan2"); u05_lose_hp(ctx, owner, 1); }; }
        // character/ow/skill.js:2571-2642
        "sheng" => { set_flag(ctx.core, owner, "ow_skill:sheng", 1);  { u05_mark_invoked(ctx, owner, "sheng"); if !u05_check_limited(ctx, owner, "sheng") { return true; } u05_recover(ctx, owner, 1); u05_turn_over(ctx, owner); }; }
        // character/ow/skill.js:2643-2657
        "sheng2" => { set_flag(ctx.core, owner, "ow_skill:sheng2", 1);  { u05_mark_invoked(ctx, owner, "sheng2"); set_flag(ctx.core, owner, "u05_mod:sheng2", 1); }; }
        // character/ow/skill.js:2702-2723
        "yihun2" => { set_flag(ctx.core, owner, "ow_skill:yihun2", 1);  { u05_mark_invoked(ctx, owner, "yihun2"); set_flag(ctx.core, owner, "u05_mod:yihun2", 1); }; }
        // character/ow/skill.js:2724-2779
        "huoyu" => { set_flag(ctx.core, owner, "ow_skill:huoyu", 1);  { u05_mark_invoked(ctx, owner, "huoyu"); if !u05_check_limited(ctx, owner, "huoyu") { return true; } if !u05_pay_discard(ctx, owner, "huoyu", 2, true) { return true; } u05_damage_aoe(ctx, owner, 2, DamageNature::Fire); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "chiyuxi"), 1); }; }
        // character/ow/skill.js:2780-2838
        "feidan" => { set_flag(ctx.core, owner, "ow_skill:feidan", 1);  { u05_mark_invoked(ctx, owner, "feidan"); u05_damage_other(ctx, owner, 1, DamageNature::Normal); set_flag(ctx.core, owner, "u05_mod:feidan", 1); }; }
        // character/ow/skill.js:2891-2930
        "huhuan" => { set_flag(ctx.core, owner, "ow_skill:huhuan", 1);  { u05_mark_invoked(ctx, owner, "huhuan"); if !u05_pay_discard(ctx, owner, "huhuan", 2, true) { return true; } u05_recover(ctx, owner, 1); }; }
        // character/ow/skill.js:2931-2999
        "kuoyin" => { set_flag(ctx.core, owner, "ow_skill:kuoyin", 1);  { u05_mark_invoked(ctx, owner, "kuoyin"); if !u05_pay_discard(ctx, owner, "kuoyin", 1, true) { return true; } ctx.start_draw(owner, 2); }; }
        // character/ow/skill.js:3009-3065
        "guangshu" => { set_flag(ctx.core, owner, "ow_skill:guangshu", 1);  { u05_mark_invoked(ctx, owner, "guangshu"); if !u05_pay_discard(ctx, owner, "guangshu", 1, true) { return true; } u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } }; }
        // character/ow/skill.js:3066-3077
        "guangshu_diamond" => { set_flag(ctx.core, owner, "ow_skill:guangshu_diamond", 1);  { u05_mark_invoked(ctx, owner, "guangshu_diamond"); ctx.start_draw(owner, 2); }; }
        // character/ow/skill.js:3078-3090
        "guangshu_heart" => { set_flag(ctx.core, owner, "ow_skill:guangshu_heart", 1);  { u05_mark_invoked(ctx, owner, "guangshu_heart"); u05_recover(ctx, owner, 1); }; }
        // character/ow/skill.js:3110-3121
        "guangshu_spade" => { set_flag(ctx.core, owner, "ow_skill:guangshu_spade", 1);  { u05_mark_invoked(ctx, owner, "guangshu_spade"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); }; }
        // character/ow/skill.js:3132-3160
        "shouhu" => { set_flag(ctx.core, owner, "ow_skill:shouhu", 1);  { u05_mark_invoked(ctx, owner, "shouhu"); if !u05_pay_discard(ctx, owner, "shouhu", 1, true) { return true; } u05_recover(ctx, owner, 1); set_flag(ctx.core, owner, "u05_mod:shouhu", 1); }; }
        // character/ow/skill.js:3161-3238
        "shanxian" => { set_flag(ctx.core, owner, "ow_skill:shanxian", 1);  { u05_mark_invoked(ctx, owner, "shanxian"); u05_turn_over(ctx, owner); }; }
        // character/xianjian/skill.js:5-25
        "lingquan" => { set_flag(ctx.core, owner, "xianjian_skill:lingquan", 1);  { u05_mark_invoked(ctx, owner, "lingquan"); if !u05_check_limited(ctx, owner, "lingquan") { return true; } ctx.start_draw(owner, 3); }; }
        // character/xianjian/skill.js:26-42
        "shenwu" => { set_flag(ctx.core, owner, "xianjian_skill:shenwu", 1);  { u05_mark_invoked(ctx, owner, "shenwu"); if !u05_check_limited(ctx, owner, "shenwu") { return true; } u05_recover(ctx, owner, 1); u05_change_max_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:93-151
        "txianqu" => { set_flag(ctx.core, owner, "xianjian_skill:txianqu", 1);  { u05_mark_invoked(ctx, owner, "txianqu"); u05_recover(ctx, owner, 1); ctx.start_draw(owner, 2); }; }
        // character/xianjian/skill.js:297-357
        "minsha" => { set_flag(ctx.core, owner, "xianjian_skill:minsha", 1);  { u05_mark_invoked(ctx, owner, "minsha"); if !u05_check_usable(ctx, owner, "minsha", 1) { return true; } if !u05_pay_discard(ctx, owner, "minsha", 2, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Thunder); }; }
        // character/xianjian/skill.js:411-438
        "xiaoyue" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue", 1);  { u05_mark_invoked(ctx, owner, "xiaoyue"); u05_change_hujia(ctx, owner, 1); }; }
        // character/xianjian/skill.js:439-447
        "xiaoyue2" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue2", 1);  { u05_mark_invoked(ctx, owner, "xiaoyue2"); set_flag(ctx.core, owner, "u05_mod:xiaoyue2", 1); }; }
        // character/xianjian/skill.js:448-463
        "huanlei" => { set_flag(ctx.core, owner, "xianjian_skill:huanlei", 1);  { u05_mark_invoked(ctx, owner, "huanlei"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); }; }
        // character/xianjian/skill.js:513-573
        "xtanxi" => { set_flag(ctx.core, owner, "xianjian_skill:xtanxi", 1);  { u05_mark_invoked(ctx, owner, "xtanxi"); if !u05_check_usable(ctx, owner, "xtanxi", 1) { return true; } if !u05_pay_discard(ctx, owner, "xtanxi", 1, true) { return true; } let _ = u05_prey_other(ctx, owner, true); }; }
        // character/xianjian/skill.js:574-592
        "linghuo" => { set_flag(ctx.core, owner, "xianjian_skill:linghuo", 1);  { u05_mark_invoked(ctx, owner, "linghuo"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); }; }
        // character/xianjian/skill.js:593-703
        "guijin" => { set_flag(ctx.core, owner, "xianjian_skill:guijin", 1);  { u05_mark_invoked(ctx, owner, "guijin"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:719-749
        "chengxin" => { set_flag(ctx.core, owner, "xianjian_skill:chengxin", 1);  { u05_mark_invoked(ctx, owner, "chengxin"); u05_recover(ctx, owner, 1); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:chengxin", 1); } }; }
        // character/xianjian/skill.js:873-900
        "xiepan" => { set_flag(ctx.core, owner, "xianjian_skill:xiepan", 1);  { u05_mark_invoked(ctx, owner, "xiepan"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:973-1026
        "yanshi" => { set_flag(ctx.core, owner, "xianjian_skill:yanshi", 1);  { u05_mark_invoked(ctx, owner, "yanshi"); if !u05_check_limited(ctx, owner, "yanshi") { return true; } u05_recover(ctx, owner, 1); u05_change_max_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:1028-1050
        "tanhua" => { set_flag(ctx.core, owner, "xianjian_skill:tanhua", 1);  { u05_mark_invoked(ctx, owner, "tanhua"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:1074-1248
        "ywuhun" => { set_flag(ctx.core, owner, "xianjian_skill:ywuhun", 1);  { u05_mark_invoked(ctx, owner, "ywuhun"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:1385-1408
        "xjzongyu" => { set_flag(ctx.core, owner, "xianjian_skill:xjzongyu", 1);  { u05_mark_invoked(ctx, owner, "xjzongyu"); if !u05_check_usable(ctx, owner, "xjzongyu", 1) { return true; } if !u05_pay_discard(ctx, owner, "xjzongyu", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "feibiao"), 1); }; }
        // character/xianjian/skill.js:1409-1422
        "fanling" => { set_flag(ctx.core, owner, "xianjian_skill:fanling", 1);  { u05_mark_invoked(ctx, owner, "fanling"); if !u05_check_usable(ctx, owner, "fanling", 1) { return true; } u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:1423-1446
        "dujiang" => { set_flag(ctx.core, owner, "xianjian_skill:dujiang", 1);  { u05_mark_invoked(ctx, owner, "dujiang"); if !u05_check_usable(ctx, owner, "dujiang", 1) { return true; } if !u05_pay_discard(ctx, owner, "dujiang", 1, true) { return true; } ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:1447-1467
        "sheying" => { set_flag(ctx.core, owner, "xianjian_skill:sheying", 1);  { u05_mark_invoked(ctx, owner, "sheying"); if !u05_check_usable(ctx, owner, "sheying", 1) { return true; } if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:sheying", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } }; }
        // character/xianjian/skill.js:1468-1518
        "huahu" => { set_flag(ctx.core, owner, "xianjian_skill:huahu", 1);  { u05_mark_invoked(ctx, owner, "huahu"); if !u05_check_limited(ctx, owner, "huahu") { return true; } u05_change_hujia(ctx, owner, 1); u05_recover(ctx, owner, 1); u05_change_max_hp(ctx, owner, -(1)); }; }
        // character/xianjian/skill.js:1519-1534
        "binxin" => { set_flag(ctx.core, owner, "xianjian_skill:binxin", 1);  { u05_mark_invoked(ctx, owner, "binxin"); u05_change_hujia(ctx, owner, 1); }; }
        // character/xianjian/skill.js:1755-1768
        "fenshi" => { set_flag(ctx.core, owner, "xianjian_skill:fenshi", 1);  { u05_mark_invoked(ctx, owner, "fenshi"); if !u05_check_limited(ctx, owner, "fenshi") { return true; } u05_change_hujia(ctx, owner, 2); ctx.start_draw(owner, 2); }; }
        // character/xianjian/skill.js:1769-1798
        "longhuo" => { set_flag(ctx.core, owner, "xianjian_skill:longhuo", 1);  { u05_mark_invoked(ctx, owner, "longhuo"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); }; }
        // character/xianjian/skill.js:1799-1829
        "yanzhan" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan", 1);  { u05_mark_invoked(ctx, owner, "yanzhan"); if !u05_check_usable(ctx, owner, "yanzhan", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); }; }
        // character/xianjian/skill.js:1841-1849
        "yanzhan3" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan3", 1);  { u05_mark_invoked(ctx, owner, "yanzhan3"); set_flag(ctx.core, owner, "u05_mod:yanzhan3", 1); }; }
        // character/xianjian/skill.js:1850-1858
        "yanzhan4" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan4", 1);  { u05_mark_invoked(ctx, owner, "yanzhan4"); set_flag(ctx.core, owner, "u05_mod:yanzhan4", 1); }; }
        // character/xianjian/skill.js:1884-1909
        "feixia" => { set_flag(ctx.core, owner, "xianjian_skill:feixia", 1);  { u05_mark_invoked(ctx, owner, "feixia"); if !u05_check_usable(ctx, owner, "feixia", 1) { return true; } if !u05_pay_discard(ctx, owner, "feixia", 1, true) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); }; }
        // character/xianjian/skill.js:1910-1958
        "lueying" => { set_flag(ctx.core, owner, "xianjian_skill:lueying", 1);  { u05_mark_invoked(ctx, owner, "lueying"); if !u05_check_usable(ctx, owner, "lueying", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/xianjian/skill.js:2061-2074
        "longxiang" => { set_flag(ctx.core, owner, "xianjian_skill:longxiang", 1);  { u05_mark_invoked(ctx, owner, "longxiang"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/xianjian/skill.js:2105-2136
        "xuanmo" => { set_flag(ctx.core, owner, "xianjian_skill:xuanmo", 1);  { u05_mark_invoked(ctx, owner, "xuanmo"); if !u05_check_usable(ctx, owner, "xuanmo", 1) { return true; } ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:2231-2297
        "zhaoyao" => { set_flag(ctx.core, owner, "xianjian_skill:zhaoyao", 1);  { u05_mark_invoked(ctx, owner, "zhaoyao"); ctx.start_draw(owner, 2); if let Some(t) = hostile_target(ctx, owner) { if let (Some(mc), Some(tc)) = (choose_own_card(ctx, owner, false), choose_enemy_card(ctx, owner, t)) { let mr = ctx.catalog.cards.get(mc as usize).map(|c| c.rank).unwrap_or(0); let tr = ctx.catalog.cards.get(tc as usize).map(|c| c.rank).unwrap_or(0); ctx.take_hand_card(owner, mc); super::commit::discard_some(ctx, owner, vec![mc]); ctx.take_hand_card(t, tc); super::commit::discard_some(ctx, t, vec![tc]); let won = mr > tr; set_flag(ctx.core, owner, &format!("u05_pindian_won:{}", "zhaoyao"), u32::from(won)); if won { ctx.start_draw(owner, 1); } } } }; }
        // character/xianjian/skill.js:2420-2431
        "leiyu3" => { set_flag(ctx.core, owner, "xianjian_skill:leiyu3", 1);  { u05_mark_invoked(ctx, owner, "leiyu3"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2502-2508
        "lingxue" => { set_flag(ctx.core, owner, "xianjian_skill:lingxue", 1);  { u05_mark_invoked(ctx, owner, "lingxue"); u05_change_hujia(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2509-2525
        "diesha" => { set_flag(ctx.core, owner, "xianjian_skill:diesha", 1);  { u05_mark_invoked(ctx, owner, "diesha"); u05_recover(ctx, owner, 1); u05_lose_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2526-2559
        "guijiang" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang", 1);  { u05_mark_invoked(ctx, owner, "guijiang"); if !u05_check_usable(ctx, owner, "guijiang", 1) { return true; } if !u05_pay_discard(ctx, owner, "guijiang", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_mod:guijiang2", 1); } }; }
        // character/xianjian/skill.js:2560-2582
        "guijiang2" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang2", 1);  { u05_mark_invoked(ctx, owner, "guijiang2"); set_flag(ctx.core, owner, "u05_mod:guijiang2", 1); }; }
        // character/xianjian/skill.js:2583-2591
        "guijiang3" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang3", 1);  { u05_mark_invoked(ctx, owner, "guijiang3"); set_flag(ctx.core, owner, "u05_mod:guijiang3", 1); }; }
        // character/xianjian/skill.js:2619-2649
        "diewu" => { set_flag(ctx.core, owner, "xianjian_skill:diewu", 1);  { u05_mark_invoked(ctx, owner, "diewu"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:2651-2676
        "lingyu" => { set_flag(ctx.core, owner, "xianjian_skill:lingyu", 1);  { u05_mark_invoked(ctx, owner, "lingyu"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2722-2739
        "feixu" => { set_flag(ctx.core, owner, "xianjian_skill:feixu", 1);  { u05_mark_invoked(ctx, owner, "feixu"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:2754-2764
        "xuanyan3" => { set_flag(ctx.core, owner, "xianjian_skill:xuanyan3", 1);  { u05_mark_invoked(ctx, owner, "xuanyan3"); u05_lose_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2765-2786
        "ningbin" => { set_flag(ctx.core, owner, "xianjian_skill:ningbin", 1);  { u05_mark_invoked(ctx, owner, "ningbin"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2787-2809
        "xfenxin" => { u05_mark_invoked(ctx, owner, "xfenxin"); ctx.start_draw(owner, 1); }
        // character/xianjian/skill.js:2810-2817
        "xfenxin2" => { set_flag(ctx.core, owner, "xianjian_skill:xfenxin2", 1);  { u05_mark_invoked(ctx, owner, "xfenxin2"); u05_recover(ctx, owner, 1); u05_change_max_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:2818-2867
        "luanjian" => { set_flag(ctx.core, owner, "xianjian_skill:luanjian", 1);  { u05_mark_invoked(ctx, owner, "luanjian"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); }; }
        // character/xianjian/skill.js:2919-2950
        "ctianfu2" => { set_flag(ctx.core, owner, "xianjian_skill:ctianfu2", 1);  { u05_mark_invoked(ctx, owner, "ctianfu2"); u05_damage_other(ctx, owner, 1, DamageNature::Thunder); }; }
        // character/xianjian/skill.js:2962-3047
        "shuiyun" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun", 1);  { u05_mark_invoked(ctx, owner, "shuiyun"); let _ = u05_store_pile(ctx, owner, "u05_pile:shuiyun"); }; }
        // character/xianjian/skill.js:3048-3089
        "shuiyun5" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun5", 1);  { u05_mark_invoked(ctx, owner, "shuiyun5"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:3200-3207
        "changnian2" => { set_flag(ctx.core, owner, "xianjian_skill:changnian2", 1);  { u05_mark_invoked(ctx, owner, "changnian2"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3208-3243
        "sajin" => { set_flag(ctx.core, owner, "xianjian_skill:sajin", 1);  { u05_mark_invoked(ctx, owner, "sajin"); if !u05_check_usable(ctx, owner, "sajin", 1) { return true; } if !u05_pay_discard(ctx, owner, "sajin", 1, true) { return true; } u05_recover(ctx, owner, 1); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:sajin", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } }; }
        // character/xianjian/skill.js:3288-3328
        "duci" => { set_flag(ctx.core, owner, "xianjian_skill:duci", 1);  { u05_mark_invoked(ctx, owner, "duci"); u05_damage_aoe(ctx, owner, 1, DamageNature::Normal); }; }
        // character/xianjian/skill.js:3398-3415
        "guiyuan" => { set_flag(ctx.core, owner, "xianjian_skill:guiyuan", 1);  { u05_mark_invoked(ctx, owner, "guiyuan"); if !u05_check_usable(ctx, owner, "guiyuan", 1) { return true; } if !u05_pay_discard(ctx, owner, "guiyuan", 1, true) { return true; } u05_recover(ctx, owner, 1); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3436-3471
        "shenmu" => { set_flag(ctx.core, owner, "xianjian_skill:shenmu", 1);  { u05_mark_invoked(ctx, owner, "shenmu"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:3472-3548
        "qianfang" => { set_flag(ctx.core, owner, "xianjian_skill:qianfang", 1);  { u05_mark_invoked(ctx, owner, "qianfang"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3557-3581
        "poyun" => { set_flag(ctx.core, owner, "xianjian_skill:poyun", 1);  { u05_mark_invoked(ctx, owner, "poyun"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/xianjian/skill.js:3582-3594
        "poyun2" => { set_flag(ctx.core, owner, "xianjian_skill:poyun2", 1);  { u05_mark_invoked(ctx, owner, "poyun2"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3596-3673
        "zhuyue" => { set_flag(ctx.core, owner, "xianjian_skill:zhuyue", 1);  { u05_mark_invoked(ctx, owner, "zhuyue"); if !u05_check_usable(ctx, owner, "zhuyue", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } }; }
        // character/xianjian/skill.js:3681-3732
        "longxi" => { set_flag(ctx.core, owner, "xianjian_skill:longxi", 1);  { u05_mark_invoked(ctx, owner, "longxi"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3733-3780
        "guanri" => { set_flag(ctx.core, owner, "xianjian_skill:guanri", 1);  { u05_mark_invoked(ctx, owner, "guanri"); u05_damage_other(ctx, owner, 2, DamageNature::Fire); u05_lose_hp(ctx, owner, 1); }; }
        // character/xianjian/skill.js:3781-3813
        "tianxian" => { set_flag(ctx.core, owner, "xianjian_skill:tianxian", 1);  { u05_mark_invoked(ctx, owner, "tianxian"); if let Some(jc) = super::commit::pop_deck(ctx) { let r = ctx.catalog.cards.get(jc as usize).map(|c| c.rank).unwrap_or(7); super::commit::discard_some(ctx, owner, vec![jc]); set_flag(ctx.core, owner, "u05_judge:tianxian", u32::from(r)); if r > 6 { ctx.start_draw(owner, 1); } } }; }
        // character/xianjian/skill.js:3814-3855
        "runxin" => { set_flag(ctx.core, owner, "xianjian_skill:runxin", 1);  { u05_mark_invoked(ctx, owner, "runxin"); u05_recover(ctx, owner, 1); }; }
        // character/xianjian/skill.js:3897-3922
        "xjzhimeng2" => { set_flag(ctx.core, owner, "xianjian_skill:xjzhimeng2", 1);  { u05_mark_invoked(ctx, owner, "xjzhimeng2"); ctx.start_draw(owner, 1); }; }
        // character/xianjian/skill.js:3948-3986
        "tannang" => { set_flag(ctx.core, owner, "xianjian_skill:tannang", 1);  { u05_mark_invoked(ctx, owner, "tannang"); if !u05_check_usable(ctx, owner, "tannang", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shunshou"), 1); }; }
        // character/xianjian/skill.js:3987-4001
        "tuoqiao" => { set_flag(ctx.core, owner, "xianjian_skill:tuoqiao", 1);  { u05_mark_invoked(ctx, owner, "tuoqiao"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "shihuifen"), 1); }; }
        // character/xianjian/skill.js:4002-4011
        "huimeng" => { set_flag(ctx.core, owner, "xianjian_skill:huimeng", 1);  { u05_mark_invoked(ctx, owner, "huimeng"); ctx.start_draw(owner, 2); }; }
        // character/xianjian/skill.js:4043-4054
        "tianshe2" => { set_flag(ctx.core, owner, "xianjian_skill:tianshe2", 1);  { u05_mark_invoked(ctx, owner, "tianshe2"); u05_recover(ctx, owner, 1); }; }
        // character/*/character.js cross-ref `beige`
        "beige" =>  { u05_mark_invoked(ctx, owner, "beige"); if !u05_pay_discard(ctx, owner, "beige", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/*/character.js cross-ref `biyue`
        "biyue" =>  { u05_mark_invoked(ctx, owner, "biyue"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `feiying`
        "feiying" =>  { u05_mark_invoked(ctx, owner, "feiying"); set_flag(ctx.core, owner, "u05_cross_mod:feiying", 1); },
        // character/*/character.js cross-ref `gongxin`
        "gongxin" =>  { u05_mark_invoked(ctx, owner, "gongxin"); if !u05_check_usable(ctx, owner, "gongxin", 1) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/*/character.js cross-ref `hujia`
        "hujia" =>  { u05_mark_invoked(ctx, owner, "hujia"); u05_change_hujia(ctx, owner, 1); },
        // character/*/character.js cross-ref `jiang`
        "jiang" =>  { u05_mark_invoked(ctx, owner, "jiang"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `jiuchi`
        "jiuchi" =>  { u05_mark_invoked(ctx, owner, "jiuchi"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "jiuchi"), 1); },
        // character/*/character.js cross-ref `jueqing`
        "jueqing" =>  { u05_mark_invoked(ctx, owner, "jueqing"); set_flag(ctx.core, owner, "u05_cross_mod:jueqing", 1); },
        // character/*/character.js cross-ref `lianpo`
        "lianpo" =>  { u05_mark_invoked(ctx, owner, "lianpo"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `lieren`
        "lieren" =>  { u05_mark_invoked(ctx, owner, "lieren"); if !u05_check_usable(ctx, owner, "lieren", 1) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/*/character.js cross-ref `liuli`
        "liuli" =>  { u05_mark_invoked(ctx, owner, "liuli"); if !u05_pay_discard(ctx, owner, "liuli", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/*/character.js cross-ref `longdan`
        "longdan" =>  { u05_mark_invoked(ctx, owner, "longdan"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "longdan"), 1); },
        // character/*/character.js cross-ref `luanji`
        "luanji" =>  { u05_mark_invoked(ctx, owner, "luanji"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "luanji"), 1); },
        // character/*/character.js cross-ref `mashu`
        "mashu" =>  { u05_mark_invoked(ctx, owner, "mashu"); set_flag(ctx.core, owner, "u05_cross_mod:mashu", 1); },
        // character/*/character.js cross-ref `niepan`
        "niepan" =>  { u05_mark_invoked(ctx, owner, "niepan"); if !u05_check_limited(ctx, owner, "niepan") { return true; } u05_recover(ctx, owner, 2); ctx.start_draw(owner, 3); },
        // character/*/character.js cross-ref `qiangxi`
        "qiangxi" =>  { u05_mark_invoked(ctx, owner, "qiangxi"); if !u05_check_usable(ctx, owner, "qiangxi", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/*/character.js cross-ref `qianxun`
        "qianxun" =>  { u05_mark_invoked(ctx, owner, "qianxun"); set_flag(ctx.core, owner, "u05_cross_mod:qianxun", 1); },
        // character/*/character.js cross-ref `rejieming`
        "rejieming" =>  { u05_mark_invoked(ctx, owner, "rejieming"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `reyingzi`
        "reyingzi" =>  { u05_mark_invoked(ctx, owner, "reyingzi"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `shangshi`
        "shangshi" =>  { u05_mark_invoked(ctx, owner, "shangshi"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `songci`
        "songci" =>  { u05_mark_invoked(ctx, owner, "songci"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `stonedraw`
        "stonedraw" =>  { u05_mark_invoked(ctx, owner, "stonedraw"); ctx.start_draw(owner, 1); },
        // character/*/character.js cross-ref `stonesha`
        "stonesha" =>  { u05_mark_invoked(ctx, owner, "stonesha"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "stonesha"), 1); },
        // character/*/character.js cross-ref `stoneshan`
        "stoneshan" =>  { u05_mark_invoked(ctx, owner, "stoneshan"); set_flag(ctx.core, owner, &format!("u05_viewas:{}", "stoneshan"), 1); },
        // character/*/character.js cross-ref `tianxiang`
        "tianxiang" =>  { u05_mark_invoked(ctx, owner, "tianxiang"); if !u05_pay_discard(ctx, owner, "tianxiang", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/*/character.js cross-ref `wushuang`
        "wushuang" =>  { u05_mark_invoked(ctx, owner, "wushuang"); set_flag(ctx.core, owner, "u05_cross_mod:wushuang", 1); },
        // character/*/character.js cross-ref `xinliegong`
        "xinliegong" =>  { u05_mark_invoked(ctx, owner, "xinliegong"); if !u05_check_usable(ctx, owner, "xinliegong", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/*/character.js cross-ref `xinwuyan`
        "xinwuyan" =>  { u05_mark_invoked(ctx, owner, "xinwuyan"); set_flag(ctx.core, owner, "u05_cross_mod:xinwuyan", 1); },
        // character/*/character.js cross-ref `yicong`
        "yicong" =>  { u05_mark_invoked(ctx, owner, "yicong"); set_flag(ctx.core, owner, "u05_cross_mod:yicong", 1); },
        // character/*/character.js cross-ref `yijue`
        "yijue" =>  { u05_mark_invoked(ctx, owner, "yijue"); if !u05_check_usable(ctx, owner, "yijue", 1) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/*/character.js cross-ref `zhenlie`
        "zhenlie" =>  { u05_mark_invoked(ctx, owner, "zhenlie"); if !u05_check_usable(ctx, owner, "zhenlie", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/*/character.js cross-ref `zhuhai`
        "zhuhai" =>  { u05_mark_invoked(ctx, owner, "zhuhai"); if !u05_check_usable(ctx, owner, "zhuhai", 1) { return true; } u05_damage_other(ctx, owner, 1, DamageNature::Normal); },
        // character/ow/skill.js:5-78
        "woliu" | "woliu2" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:woliu", 1); },
        // character/ow/skill.js:924-935
        "paotai3" =>  { u05_mark_invoked(ctx, owner, "paotai3"); u05_damage_other(ctx, owner, 1, DamageNature::Fire); },
        // character/ow/skill.js:964-1002
        "bfengshi2" | "bfengshi3" =>  { u05_mark_invoked(ctx, owner, skill); set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1); },
        // character/ow/skill.js:1098-1180
        "shoujia" | "shoujia3" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { u05_turn_over(ctx, t); } },
        // character/ow/skill.js:1181-1228
        "liudan" =>  { u05_mark_invoked(ctx, owner, "liudan"); set_flag(ctx.core, owner, "u05_mod:woliu", 1); },
        // character/ow/skill.js:1229-1265
        "tiandan" =>  { u05_mark_invoked(ctx, owner, "tiandan"); if !u05_pay_discard(ctx, owner, "tiandan", 2, true) { return true; } },
        // character/ow/skill.js:1503-1626
        "juji" | "juji2" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_check_usable(ctx, owner, skill, 1) { return true; } set_flag(ctx.core, owner, "u05_mod:juji3", 1); },
        // character/ow/skill.js:1627-1700
        "dulei" =>  { u05_mark_invoked(ctx, owner, "dulei"); if !u05_check_usable(ctx, owner, "dulei", 1) { return true; } if let Some(t) = hostile_target(ctx, owner) { u05_lose_hp(ctx, t, 1); } },
        // character/ow/skill.js:1790-1810
        "bingqiang_remove" =>  { u05_mark_invoked(ctx, owner, "bingqiang_remove"); if !u05_pay_discard(ctx, owner, "bingqiang_remove", 1, true) { return true; } },
        // character/ow/skill.js:1927-1960
        "tuijin" =>  { u05_mark_invoked(ctx, owner, "tuijin"); set_flag(ctx.core, owner, "u05_mod:tuijin2", 1); },
        // character/ow/skill.js:2145-2197
        "xiandan2" =>  { u05_mark_invoked(ctx, owner, "xiandan2"); if !u05_pay_discard(ctx, owner, "xiandan2", 1, true) { return true; } },
        // character/ow/skill.js:2198-2228
        "mujing2" =>  { u05_mark_invoked(ctx, owner, "mujing2"); if !u05_pay_discard(ctx, owner, "mujing2", 1, true) { return true; } set_flag(ctx.core, owner, "u05_mod:mujing", 1); },
        // character/ow/skill.js:2229-2270
        "lichang2" =>  { u05_mark_invoked(ctx, owner, "lichang2"); u05_recover(ctx, owner, 1); },
        // character/ow/skill.js:2356-2375
        "feiren3" =>  { u05_mark_invoked(ctx, owner, "feiren3"); if !u05_pay_discard(ctx, owner, "feiren3", 1, true) { return true; } },
        // character/ow/skill.js:2466-2495
        "xie3" | "xie4" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_buff:xie2", 1); } },
        // character/ow/skill.js:2550-2575
        "luan3" | "luan4" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { set_flag(ctx.core, t, "u05_debuff:luan", 1); } },
        // character/ow/skill.js:2658-2710
        "yihun" =>  { u05_mark_invoked(ctx, owner, "yihun"); if !u05_check_usable(ctx, owner, "yihun", 1) { return true; } set_flag(ctx.core, owner, &format!("u05_viewas:{}", "sha"), 1); },
        // character/ow/skill.js:3000-3020
        "kuoyin2" =>  { u05_mark_invoked(ctx, owner, "kuoyin2"); if !u05_pay_discard(ctx, owner, "kuoyin2", 1, true) { return true; } },
        // character/ow/skill.js:3239-3260
        "shanxian2" =>  { u05_mark_invoked(ctx, owner, "shanxian2"); if !u05_pay_discard(ctx, owner, "shanxian2", 1, true) { return true; } },
        // character/xianjian/skill.js:43-85
        "qiongguang" =>  { u05_mark_invoked(ctx, owner, "qiongguang"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/xianjian/skill.js:281-310
        "yuexing2" =>  { u05_mark_invoked(ctx, owner, "yuexing2"); set_flag(ctx.core, owner, "u05_mod:tuijin2", 1); },
        // character/xianjian/skill.js:358-415
        "lingdi" =>  { u05_mark_invoked(ctx, owner, "lingdi"); if !u05_check_usable(ctx, owner, "lingdi", 1) { return true; } if !u05_pay_discard(ctx, owner, "lingdi", 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/xianjian/skill.js:704-749
        "guijin2" =>  { u05_mark_invoked(ctx, owner, "guijin2"); if !u05_pay_discard(ctx, owner, "guijin2", 1, true) { return true; } },
        // character/xianjian/skill.js:750-772
        "chengxin2" =>  { u05_mark_invoked(ctx, owner, "chengxin2"); set_flag(ctx.core, owner, "u05_debuff:chengxin", 1); },
        // character/xianjian/skill.js:773-815
        "tianwu" =>  { u05_mark_invoked(ctx, owner, "tianwu"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/xianjian/skill.js:1535-1580
        "qixia" =>  { u05_mark_invoked(ctx, owner, "qixia"); set_flag(ctx.core, owner, "u05_mod:extra_turn", 1); ctx.start_draw(owner, 2); },
        // character/xianjian/skill.js:1830-1850
        "yanzhan2" =>  { u05_mark_invoked(ctx, owner, "yanzhan2"); if !u05_pay_discard(ctx, owner, "yanzhan2", 1, true) { return true; } },
        // character/xianjian/skill.js:2137-2205
        "danqing" | "danqing_count" =>  { u05_mark_invoked(ctx, owner, skill); if !u05_pay_discard(ctx, owner, skill, 1, true) { return true; } ctx.start_draw(owner, 1); },
        // character/xianjian/skill.js:2345-2365
        "zhangmu_ai" =>  { u05_mark_invoked(ctx, owner, "zhangmu_ai"); if !u05_pay_discard(ctx, owner, "zhangmu_ai", 1, true) { return true; } },
        // character/xianjian/skill.js:2413-2441
        "leiyu2" | "leiyu4" =>  { u05_mark_invoked(ctx, owner, skill); if let Some(t) = hostile_target(ctx, owner) { ctx.start_draw(t, 1); } },
        // character/xianjian/skill.js:2442-2480
        "feizhua" =>  { u05_mark_invoked(ctx, owner, "feizhua"); set_flag(ctx.core, owner, "u05_mod:woliu", 1); },
        // character/xianjian/skill.js:2592-2676
        "fenxing" =>  { u05_mark_invoked(ctx, owner, "fenxing"); if !u05_pay_discard(ctx, owner, "fenxing", 1, true) { return true; } },
        // character/xianjian/skill.js:2677-2720
        "duxinshu" =>  { u05_mark_invoked(ctx, owner, "duxinshu"); if !u05_check_usable(ctx, owner, "duxinshu", 1) { return true; } let _ = u05_prey_other(ctx, owner, true); },
        // character/xianjian/skill.js:3549-3565
        "qianfang2" =>  { u05_mark_invoked(ctx, owner, "qianfang2"); if !u05_pay_discard(ctx, owner, "qianfang2", 1, true) { return true; } },
        // character/xianjian/skill.js:3674-3690
        "zhuyue2" =>  { u05_mark_invoked(ctx, owner, "zhuyue2"); if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        // character/xianjian/skill.js:3923-3950
        "xjzhimeng3" =>  { u05_mark_invoked(ctx, owner, "xjzhimeng3"); if !u05_pay_discard(ctx, owner, "xjzhimeng3", 1, true) { return true; } if let Some(t) = hostile_target(ctx, owner) { let _ = u05_discard_up_to(ctx, t, 1); } },
        _ => return false,
    }
    true
}
