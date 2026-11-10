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
                if let Some(t) = first_other_alive(ctx.core, owner) {
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
        "qianggu" => { set_flag(ctx.core, owner, "ow_skill:qianggu", 1); u05_exec(ctx, owner, "qianggu", 1, false, 2, 0, 0, 0, DamageNature::Normal, 0, 2, U05Extra::None); }
        // character/ow/skill.js:217-240
        "dianji" => { set_flag(ctx.core, owner, "ow_skill:dianji", 1); u05_exec(ctx, owner, "dianji", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jingleishan")); }
        // character/ow/skill.js:287-300
        "feitiao2" => { set_flag(ctx.core, owner, "ow_skill:feitiao2", 1); u05_exec(ctx, owner, "feitiao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feitiao2", 1)); }
        // character/ow/skill.js:346-422
        "bshaowei" => { set_flag(ctx.core, owner, "ow_skill:bshaowei", 1); u05_exec(ctx, owner, "bshaowei", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bshaowei", 1)); }
        // character/ow/skill.js:522-561
        "liangou" => { set_flag(ctx.core, owner, "ow_skill:liangou", 1); u05_exec(ctx, owner, "liangou", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:liangou")); }
        // character/ow/skill.js:562-578
        "liangou2" => { set_flag(ctx.core, owner, "ow_skill:liangou2", 1); u05_exec(ctx, owner, "liangou2", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:liangou2", 1)); }
        // character/ow/skill.js:579-593
        "xiyang" => { set_flag(ctx.core, owner, "ow_skill:xiyang", 1); u05_exec(ctx, owner, "xiyang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 2, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:594-702
        "qinru" => { set_flag(ctx.core, owner, "ow_skill:qinru", 1); u05_exec(ctx, owner, "qinru", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:qinru")); }
        // character/ow/skill.js:818-833
        "maoding" => { set_flag(ctx.core, owner, "ow_skill:maoding", 1); u05_exec(ctx, owner, "maoding", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:834-855
        "maoding2" => { set_flag(ctx.core, owner, "ow_skill:maoding2", 1); u05_exec(ctx, owner, "maoding2", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None); }
        // character/ow/skill.js:856-894
        "paotai" => { set_flag(ctx.core, owner, "ow_skill:paotai", 1); u05_exec(ctx, owner, "paotai", 0, false, 1, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None); }
        // character/ow/skill.js:895-923
        "paotai2" => { set_flag(ctx.core, owner, "ow_skill:paotai2", 1); u05_exec(ctx, owner, "paotai2", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None); }
        // character/ow/skill.js:941-963
        "bfengshi" => { set_flag(ctx.core, owner, "ow_skill:bfengshi", 1); u05_exec(ctx, owner, "bfengshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bfengshi", 1)); }
        // character/ow/skill.js:995-1034
        "yinbo" => { set_flag(ctx.core, owner, "ow_skill:yinbo", 1); u05_exec(ctx, owner, "yinbo", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/ow/skill.js:1035-1074
        "aqianghua" => { set_flag(ctx.core, owner, "ow_skill:aqianghua", 1); u05_exec(ctx, owner, "aqianghua", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantOtherBuff("u05_buff:aqianghua2", 1)); }
        // character/ow/skill.js:1075-1087
        "aqianghua2" => { set_flag(ctx.core, owner, "ow_skill:aqianghua2", 1); u05_exec(ctx, owner, "aqianghua2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:aqianghua2", 1)); }
        // character/ow/skill.js:1088-1097
        "shihuo" => { set_flag(ctx.core, owner, "ow_skill:shihuo", 1); u05_exec(ctx, owner, "shihuo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:1128-1163
        "shoujia2" => { set_flag(ctx.core, owner, "ow_skill:shoujia2", 1); u05_exec(ctx, owner, "shoujia2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:1252-1284
        "shanguang" => { set_flag(ctx.core, owner, "ow_skill:shanguang", 1); u05_exec(ctx, owner, "shanguang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:shanguang2", 1)); }
        // character/ow/skill.js:1285-1311
        "shanguang2" => { set_flag(ctx.core, owner, "ow_skill:shanguang2", 1); u05_exec(ctx, owner, "shanguang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:shanguang2", 1)); }
        // character/ow/skill.js:1312-1383
        "baoxue" => { set_flag(ctx.core, owner, "ow_skill:baoxue", 1); u05_exec(ctx, owner, "baoxue", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/ow/skill.js:1415-1436
        "mianzhen2" => { set_flag(ctx.core, owner, "ow_skill:mianzhen2", 1); u05_exec(ctx, owner, "mianzhen2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mianzhen2", 1)); }
        // character/ow/skill.js:1437-1476
        "zhiyuan" => { set_flag(ctx.core, owner, "ow_skill:zhiyuan", 1); u05_exec(ctx, owner, "zhiyuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/ow/skill.js:1493-1502
        "duwen2" => { set_flag(ctx.core, owner, "ow_skill:duwen2", 1); u05_exec(ctx, owner, "duwen2", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:1610-1626
        "juji3" => { set_flag(ctx.core, owner, "ow_skill:juji3", 1); u05_exec(ctx, owner, "juji3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:juji3", 1)); }
        // character/ow/skill.js:1652-1689
        "dulei2" => { set_flag(ctx.core, owner, "ow_skill:dulei2", 1); u05_exec(ctx, owner, "dulei2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/ow/skill.js:1805-1817
        "bingqiang2" => { set_flag(ctx.core, owner, "ow_skill:bingqiang2", 1); u05_exec(ctx, owner, "bingqiang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bingqiang2", 1)); }
        // character/ow/skill.js:1818-1830
        "bingqiang3" => { set_flag(ctx.core, owner, "ow_skill:bingqiang3", 1); u05_exec(ctx, owner, "bingqiang3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bingqiang3", 1)); }
        // character/ow/skill.js:1831-1843
        "bingqiang4" => { set_flag(ctx.core, owner, "ow_skill:bingqiang4", 1); u05_exec(ctx, owner, "bingqiang4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bingqiang4", 1)); }
        // character/ow/skill.js:1844-1856
        "bingqiang5" => { set_flag(ctx.core, owner, "ow_skill:bingqiang5", 1); u05_exec(ctx, owner, "bingqiang5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bingqiang5", 1)); }
        // character/ow/skill.js:1857-1879
        "jidong" => { set_flag(ctx.core, owner, "ow_skill:jidong", 1); u05_exec(ctx, owner, "jidong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 2, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:1880-1905
        "jidong2" => { set_flag(ctx.core, owner, "ow_skill:jidong2", 1); u05_exec(ctx, owner, "jidong2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jidong2", 1)); }
        // character/ow/skill.js:1906-1926
        "chongzhuang" => { set_flag(ctx.core, owner, "ow_skill:chongzhuang", 1); u05_exec(ctx, owner, "chongzhuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:chongzhuang", 1)); }
        // character/ow/skill.js:1961-1974
        "tuijin2" => { set_flag(ctx.core, owner, "ow_skill:tuijin2", 1); u05_exec(ctx, owner, "tuijin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuijin2", 1)); }
        // character/ow/skill.js:1975-2016
        "jijia" => { set_flag(ctx.core, owner, "ow_skill:jijia", 1); u05_exec(ctx, owner, "jijia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jijia", 1)); }
        // character/ow/skill.js:2156-2162
        "shouge" => { set_flag(ctx.core, owner, "ow_skill:shouge", 1); u05_exec(ctx, owner, "shouge", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:2163-2171
        "tuji" => { set_flag(ctx.core, owner, "ow_skill:tuji", 1); u05_exec(ctx, owner, "tuji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuji", 1)); }
        // character/ow/skill.js:2172-2197
        "mujing" => { set_flag(ctx.core, owner, "ow_skill:mujing", 1); u05_exec(ctx, owner, "mujing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/ow/skill.js:2322-2330
        "zhanlong2" => { set_flag(ctx.core, owner, "ow_skill:zhanlong2", 1); u05_exec(ctx, owner, "zhanlong2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhanlong2", 1)); }
        // character/ow/skill.js:2331-2355
        "feiren" => { set_flag(ctx.core, owner, "ow_skill:feiren", 1); u05_exec(ctx, owner, "feiren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feiren", 1)); }
        // character/ow/skill.js:2387-2427
        "xie" => { set_flag(ctx.core, owner, "ow_skill:xie", 1); u05_exec(ctx, owner, "xie", 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantOtherBuff("u05_buff:xie2", 1)); }
        // character/ow/skill.js:2428-2465
        "xie2" => { set_flag(ctx.core, owner, "ow_skill:xie2", 1); u05_exec(ctx, owner, "xie2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:xie2", 1)); }
        // character/ow/skill.js:2492-2534
        "luan" => { set_flag(ctx.core, owner, "ow_skill:luan", 1); u05_exec(ctx, owner, "luan", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:luan", 1)); }
        // character/ow/skill.js:2535-2549
        "luan2" => { set_flag(ctx.core, owner, "ow_skill:luan2", 1); u05_exec(ctx, owner, "luan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/ow/skill.js:2571-2642
        "sheng" => { set_flag(ctx.core, owner, "ow_skill:sheng", 1); u05_exec(ctx, owner, "sheng", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:2643-2657
        "sheng2" => { set_flag(ctx.core, owner, "ow_skill:sheng2", 1); u05_exec(ctx, owner, "sheng2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:sheng2", 1)); }
        // character/ow/skill.js:2702-2723
        "yihun2" => { set_flag(ctx.core, owner, "ow_skill:yihun2", 1); u05_exec(ctx, owner, "yihun2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yihun2", 1)); }
        // character/ow/skill.js:2724-2779
        "huoyu" => { set_flag(ctx.core, owner, "ow_skill:huoyu", 1); u05_exec(ctx, owner, "huoyu", 0, true, 2, 0, 0, -2, DamageNature::Fire, 0, 0, U05Extra::ViewAs("chiyuxi")); }
        // character/ow/skill.js:2780-2838
        "feidan" => { set_flag(ctx.core, owner, "ow_skill:feidan", 1); u05_exec(ctx, owner, "feidan", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feidan", 1)); }
        // character/ow/skill.js:2891-2930
        "huhuan" => { set_flag(ctx.core, owner, "ow_skill:huhuan", 1); u05_exec(ctx, owner, "huhuan", 0, false, 2, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/ow/skill.js:2931-2999
        "kuoyin" => { set_flag(ctx.core, owner, "ow_skill:kuoyin", 1); u05_exec(ctx, owner, "kuoyin", 0, false, 1, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:3009-3065
        "guangshu" => { set_flag(ctx.core, owner, "ow_skill:guangshu", 1); u05_exec(ctx, owner, "guangshu", 0, false, 1, 0, 1, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/ow/skill.js:3066-3077
        "guangshu_diamond" => { set_flag(ctx.core, owner, "ow_skill:guangshu_diamond", 1); u05_exec(ctx, owner, "guangshu_diamond", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/ow/skill.js:3078-3090
        "guangshu_heart" => { set_flag(ctx.core, owner, "ow_skill:guangshu_heart", 1); u05_exec(ctx, owner, "guangshu_heart", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/ow/skill.js:3110-3121
        "guangshu_spade" => { set_flag(ctx.core, owner, "ow_skill:guangshu_spade", 1); u05_exec(ctx, owner, "guangshu_spade", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None); }
        // character/ow/skill.js:3132-3160
        "shouhu" => { set_flag(ctx.core, owner, "ow_skill:shouhu", 1); u05_exec(ctx, owner, "shouhu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:shouhu", 1)); }
        // character/ow/skill.js:3161-3238
        "shanxian" => { set_flag(ctx.core, owner, "ow_skill:shanxian", 1); u05_exec(ctx, owner, "shanxian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf); }
        // character/xianjian/skill.js:5-25
        "lingquan" => { set_flag(ctx.core, owner, "xianjian_skill:lingquan", 1); u05_exec(ctx, owner, "lingquan", 0, true, 0, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:26-42
        "shenwu" => { set_flag(ctx.core, owner, "xianjian_skill:shenwu", 1); u05_exec(ctx, owner, "shenwu", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:93-151
        "txianqu" => { set_flag(ctx.core, owner, "xianjian_skill:txianqu", 1); u05_exec(ctx, owner, "txianqu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:297-357
        "minsha" => { set_flag(ctx.core, owner, "xianjian_skill:minsha", 1); u05_exec(ctx, owner, "minsha", 1, false, 2, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:411-438
        "xiaoyue" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue", 1); u05_exec(ctx, owner, "xiaoyue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None); }
        // character/xianjian/skill.js:439-447
        "xiaoyue2" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue2", 1); u05_exec(ctx, owner, "xiaoyue2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:xiaoyue2", 1)); }
        // character/xianjian/skill.js:448-463
        "huanlei" => { set_flag(ctx.core, owner, "xianjian_skill:huanlei", 1); u05_exec(ctx, owner, "huanlei", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:513-573
        "xtanxi" => { set_flag(ctx.core, owner, "xianjian_skill:xtanxi", 1); u05_exec(ctx, owner, "xtanxi", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther); }
        // character/xianjian/skill.js:574-592
        "linghuo" => { set_flag(ctx.core, owner, "xianjian_skill:linghuo", 1); u05_exec(ctx, owner, "linghuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:593-703
        "guijin" => { set_flag(ctx.core, owner, "xianjian_skill:guijin", 1); u05_exec(ctx, owner, "guijin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:719-749
        "chengxin" => { set_flag(ctx.core, owner, "xianjian_skill:chengxin", 1); u05_exec(ctx, owner, "chengxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantOtherBuff("u05_debuff:chengxin", 1)); }
        // character/xianjian/skill.js:873-900
        "xiepan" => { set_flag(ctx.core, owner, "xianjian_skill:xiepan", 1); u05_exec(ctx, owner, "xiepan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:973-1026
        "yanshi" => { set_flag(ctx.core, owner, "xianjian_skill:yanshi", 1); u05_exec(ctx, owner, "yanshi", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:1028-1050
        "tanhua" => { set_flag(ctx.core, owner, "xianjian_skill:tanhua", 1); u05_exec(ctx, owner, "tanhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:1074-1248
        "ywuhun" => { set_flag(ctx.core, owner, "xianjian_skill:ywuhun", 1); u05_exec(ctx, owner, "ywuhun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:1385-1408
        "xjzongyu" => { set_flag(ctx.core, owner, "xianjian_skill:xjzongyu", 1); u05_exec(ctx, owner, "xjzongyu", 1, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::ViewAs("feibiao")); }
        // character/xianjian/skill.js:1409-1422
        "fanling" => { set_flag(ctx.core, owner, "xianjian_skill:fanling", 1); u05_exec(ctx, owner, "fanling", 1, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:1423-1446
        "dujiang" => { set_flag(ctx.core, owner, "xianjian_skill:dujiang", 1); u05_exec(ctx, owner, "dujiang", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:1447-1467
        "sheying" => { set_flag(ctx.core, owner, "xianjian_skill:sheying", 1); u05_exec(ctx, owner, "sheying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:sheying")); }
        // character/xianjian/skill.js:1468-1518
        "huahu" => { set_flag(ctx.core, owner, "xianjian_skill:huahu", 1); u05_exec(ctx, owner, "huahu", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::LoseMaxHp(1)); }
        // character/xianjian/skill.js:1519-1534
        "binxin" => { set_flag(ctx.core, owner, "xianjian_skill:binxin", 1); u05_exec(ctx, owner, "binxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None); }
        // character/xianjian/skill.js:1755-1768
        "fenshi" => { set_flag(ctx.core, owner, "xianjian_skill:fenshi", 1); u05_exec(ctx, owner, "fenshi", 0, true, 0, 2, 0, 0, DamageNature::Normal, 0, 2, U05Extra::None); }
        // character/xianjian/skill.js:1769-1798
        "longhuo" => { set_flag(ctx.core, owner, "xianjian_skill:longhuo", 1); u05_exec(ctx, owner, "longhuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:1799-1829
        "yanzhan" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan", 1); u05_exec(ctx, owner, "yanzhan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/xianjian/skill.js:1841-1849
        "yanzhan3" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan3", 1); u05_exec(ctx, owner, "yanzhan3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yanzhan3", 1)); }
        // character/xianjian/skill.js:1850-1858
        "yanzhan4" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan4", 1); u05_exec(ctx, owner, "yanzhan4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yanzhan4", 1)); }
        // character/xianjian/skill.js:1884-1909
        "feixia" => { set_flag(ctx.core, owner, "xianjian_skill:feixia", 1); u05_exec(ctx, owner, "feixia", 1, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/xianjian/skill.js:1910-1958
        "lueying" => { set_flag(ctx.core, owner, "xianjian_skill:lueying", 1); u05_exec(ctx, owner, "lueying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:2061-2074
        "longxiang" => { set_flag(ctx.core, owner, "xianjian_skill:longxiang", 1); u05_exec(ctx, owner, "longxiang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:2105-2136
        "xuanmo" => { set_flag(ctx.core, owner, "xianjian_skill:xuanmo", 1); u05_exec(ctx, owner, "xuanmo", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:2231-2297
        "zhaoyao" => { set_flag(ctx.core, owner, "xianjian_skill:zhaoyao", 1); u05_exec(ctx, owner, "zhaoyao", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::PindianOther); }
        // character/xianjian/skill.js:2420-2431
        "leiyu3" => { set_flag(ctx.core, owner, "xianjian_skill:leiyu3", 1); u05_exec(ctx, owner, "leiyu3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:2502-2508
        "lingxue" => { set_flag(ctx.core, owner, "xianjian_skill:lingxue", 1); u05_exec(ctx, owner, "lingxue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None); }
        // character/xianjian/skill.js:2509-2525
        "diesha" => { set_flag(ctx.core, owner, "xianjian_skill:diesha", 1); u05_exec(ctx, owner, "diesha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:2526-2559
        "guijiang" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang", 1); u05_exec(ctx, owner, "guijiang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:guijiang2", 1)); }
        // character/xianjian/skill.js:2560-2582
        "guijiang2" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang2", 1); u05_exec(ctx, owner, "guijiang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:guijiang2", 1)); }
        // character/xianjian/skill.js:2583-2591
        "guijiang3" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang3", 1); u05_exec(ctx, owner, "guijiang3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:guijiang3", 1)); }
        // character/xianjian/skill.js:2619-2649
        "diewu" => { set_flag(ctx.core, owner, "xianjian_skill:diewu", 1); u05_exec(ctx, owner, "diewu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:2651-2676
        "lingyu" => { set_flag(ctx.core, owner, "xianjian_skill:lingyu", 1); u05_exec(ctx, owner, "lingyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:2722-2739
        "feixu" => { set_flag(ctx.core, owner, "xianjian_skill:feixu", 1); u05_exec(ctx, owner, "feixu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:2754-2764
        "xuanyan3" => { set_flag(ctx.core, owner, "xianjian_skill:xuanyan3", 1); u05_exec(ctx, owner, "xuanyan3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:2765-2786
        "ningbin" => { set_flag(ctx.core, owner, "xianjian_skill:ningbin", 1); u05_exec(ctx, owner, "ningbin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:2787-2809
        "xfenxin" => { u05_mark_invoked(ctx, owner, "xfenxin"); ctx.start_draw(owner, 1); }
        // character/xianjian/skill.js:2810-2817
        "xfenxin2" => { set_flag(ctx.core, owner, "xianjian_skill:xfenxin2", 1); u05_exec(ctx, owner, "xfenxin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:2818-2867
        "luanjian" => { set_flag(ctx.core, owner, "xianjian_skill:luanjian", 1); u05_exec(ctx, owner, "luanjian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/xianjian/skill.js:2919-2950
        "ctianfu2" => { set_flag(ctx.core, owner, "xianjian_skill:ctianfu2", 1); u05_exec(ctx, owner, "ctianfu2", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:2962-3047
        "shuiyun" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun", 1); u05_exec(ctx, owner, "shuiyun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:shuiyun")); }
        // character/xianjian/skill.js:3048-3089
        "shuiyun5" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun5", 1); u05_exec(ctx, owner, "shuiyun5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:3200-3207
        "changnian2" => { set_flag(ctx.core, owner, "xianjian_skill:changnian2", 1); u05_exec(ctx, owner, "changnian2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3208-3243
        "sajin" => { set_flag(ctx.core, owner, "xianjian_skill:sajin", 1); u05_exec(ctx, owner, "sajin", 1, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::JudgeEffect("u05_judge:sajin")); }
        // character/xianjian/skill.js:3288-3328
        "duci" => { set_flag(ctx.core, owner, "xianjian_skill:duci", 1); u05_exec(ctx, owner, "duci", 0, false, 0, 0, 0, -1, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3398-3415
        "guiyuan" => { set_flag(ctx.core, owner, "xianjian_skill:guiyuan", 1); u05_exec(ctx, owner, "guiyuan", 1, false, 1, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:3436-3471
        "shenmu" => { set_flag(ctx.core, owner, "xianjian_skill:shenmu", 1); u05_exec(ctx, owner, "shenmu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:3472-3548
        "qianfang" => { set_flag(ctx.core, owner, "xianjian_skill:qianfang", 1); u05_exec(ctx, owner, "qianfang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3557-3581
        "poyun" => { set_flag(ctx.core, owner, "xianjian_skill:poyun", 1); u05_exec(ctx, owner, "poyun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:3582-3594
        "poyun2" => { set_flag(ctx.core, owner, "xianjian_skill:poyun2", 1); u05_exec(ctx, owner, "poyun2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3596-3673
        "zhuyue" => { set_flag(ctx.core, owner, "xianjian_skill:zhuyue", 1); u05_exec(ctx, owner, "zhuyue", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:3681-3732
        "longxi" => { set_flag(ctx.core, owner, "xianjian_skill:longxi", 1); u05_exec(ctx, owner, "longxi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3733-3780
        "guanri" => { set_flag(ctx.core, owner, "xianjian_skill:guanri", 1); u05_exec(ctx, owner, "guanri", 0, false, 0, 0, 0, 2, DamageNature::Fire, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:3781-3813
        "tianxian" => { set_flag(ctx.core, owner, "xianjian_skill:tianxian", 1); u05_exec(ctx, owner, "tianxian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:tianxian")); }
        // character/xianjian/skill.js:3814-3855
        "runxin" => { set_flag(ctx.core, owner, "xianjian_skill:runxin", 1); u05_exec(ctx, owner, "runxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/xianjian/skill.js:3897-3922
        "xjzhimeng2" => { set_flag(ctx.core, owner, "xianjian_skill:xjzhimeng2", 1); u05_exec(ctx, owner, "xjzhimeng2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:3948-3986
        "tannang" => { set_flag(ctx.core, owner, "xianjian_skill:tannang", 1); u05_exec(ctx, owner, "tannang", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shunshou")); }
        // character/xianjian/skill.js:3987-4001
        "tuoqiao" => { set_flag(ctx.core, owner, "xianjian_skill:tuoqiao", 1); u05_exec(ctx, owner, "tuoqiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shihuifen")); }
        // character/xianjian/skill.js:4002-4011
        "huimeng" => { set_flag(ctx.core, owner, "xianjian_skill:huimeng", 1); u05_exec(ctx, owner, "huimeng", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None); }
        // character/xianjian/skill.js:4043-4054
        "tianshe2" => { set_flag(ctx.core, owner, "xianjian_skill:tianshe2", 1); u05_exec(ctx, owner, "tianshe2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None); }
        // character/*/character.js cross-ref `beige`
        "beige" => u05_exec(ctx, owner, "beige", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `biyue`
        "biyue" => u05_exec(ctx, owner, "biyue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `feiying`
        "feiying" => u05_exec(ctx, owner, "feiying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:feiying", 1)),
        // character/*/character.js cross-ref `gongxin`
        "gongxin" => u05_exec(ctx, owner, "gongxin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/*/character.js cross-ref `hujia`
        "hujia" => u05_exec(ctx, owner, "hujia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/*/character.js cross-ref `jiang`
        "jiang" => u05_exec(ctx, owner, "jiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `jiuchi`
        "jiuchi" => u05_exec(ctx, owner, "jiuchi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jiuchi")),
        // character/*/character.js cross-ref `jueqing`
        "jueqing" => u05_exec(ctx, owner, "jueqing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:jueqing", 1)),
        // character/*/character.js cross-ref `lianpo`
        "lianpo" => u05_exec(ctx, owner, "lianpo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `lieren`
        "lieren" => u05_exec(ctx, owner, "lieren", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/*/character.js cross-ref `liuli`
        "liuli" => u05_exec(ctx, owner, "liuli", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `longdan`
        "longdan" => u05_exec(ctx, owner, "longdan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("longdan")),
        // character/*/character.js cross-ref `luanji`
        "luanji" => u05_exec(ctx, owner, "luanji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("luanji")),
        // character/*/character.js cross-ref `mashu`
        "mashu" => u05_exec(ctx, owner, "mashu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:mashu", 1)),
        // character/*/character.js cross-ref `niepan`
        "niepan" => u05_exec(ctx, owner, "niepan", 0, true, 0, 3, 0, 0, DamageNature::Normal, 2, 0, U05Extra::None),
        // character/*/character.js cross-ref `qiangxi`
        "qiangxi" => u05_exec(ctx, owner, "qiangxi", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `qianxun`
        "qianxun" => u05_exec(ctx, owner, "qianxun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:qianxun", 1)),
        // character/*/character.js cross-ref `rejieming`
        "rejieming" => u05_exec(ctx, owner, "rejieming", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `reyingzi`
        "reyingzi" => u05_exec(ctx, owner, "reyingzi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `shangshi`
        "shangshi" => u05_exec(ctx, owner, "shangshi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `songci`
        "songci" => u05_exec(ctx, owner, "songci", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `stonedraw`
        "stonedraw" => u05_exec(ctx, owner, "stonedraw", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `stonesha`
        "stonesha" => u05_exec(ctx, owner, "stonesha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("stonesha")),
        // character/*/character.js cross-ref `stoneshan`
        "stoneshan" => u05_exec(ctx, owner, "stoneshan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("stoneshan")),
        // character/*/character.js cross-ref `tianxiang`
        "tianxiang" => u05_exec(ctx, owner, "tianxiang", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `wushuang`
        "wushuang" => u05_exec(ctx, owner, "wushuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:wushuang", 1)),
        // character/*/character.js cross-ref `xinliegong`
        "xinliegong" => u05_exec(ctx, owner, "xinliegong", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `xinwuyan`
        "xinwuyan" => u05_exec(ctx, owner, "xinwuyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:xinwuyan", 1)),
        // character/*/character.js cross-ref `yicong`
        "yicong" => u05_exec(ctx, owner, "yicong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:yicong", 1)),
        // character/*/character.js cross-ref `yijue`
        "yijue" => u05_exec(ctx, owner, "yijue", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/*/character.js cross-ref `zhenlie`
        "zhenlie" => u05_exec(ctx, owner, "zhenlie", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/*/character.js cross-ref `zhuhai`
        "zhuhai" => u05_exec(ctx, owner, "zhuhai", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:5-78
        "woliu" | "woliu2" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:woliu", 1)),
        // character/ow/skill.js:924-935
        "paotai3" => u05_exec(ctx, owner, "paotai3", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None),
        // character/ow/skill.js:964-1002
        "bfengshi2" | "bfengshi3" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/ow/skill.js:1098-1180
        "shoujia" | "shoujia3" => u05_exec(ctx, owner, skill, 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
        // character/ow/skill.js:1181-1228
        "liudan" => u05_exec(ctx, owner, "liudan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:woliu", 1)),
        // character/ow/skill.js:1229-1265
        "tiandan" => u05_exec(ctx, owner, "tiandan", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:1503-1626
        "juji" | "juji2" => u05_exec(ctx, owner, skill, 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:juji3", 1)),
        // character/ow/skill.js:1627-1700
        "dulei" => u05_exec(ctx, owner, "dulei", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/ow/skill.js:1790-1810
        "bingqiang_remove" => u05_exec(ctx, owner, "bingqiang_remove", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:1927-1960
        "tuijin" => u05_exec(ctx, owner, "tuijin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuijin2", 1)),
        // character/ow/skill.js:2145-2197
        "xiandan2" => u05_exec(ctx, owner, "xiandan2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:2198-2228
        "mujing2" => u05_exec(ctx, owner, "mujing2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mujing", 1)),
        // character/ow/skill.js:2229-2270
        "lichang2" => u05_exec(ctx, owner, "lichang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/ow/skill.js:2356-2375
        "feiren3" => u05_exec(ctx, owner, "feiren3", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:2466-2495
        "xie3" | "xie4" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_buff:xie2", 1)),
        // character/ow/skill.js:2550-2575
        "luan3" | "luan4" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:luan", 1)),
        // character/ow/skill.js:2658-2710
        "yihun" => u05_exec(ctx, owner, "yihun", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/ow/skill.js:3000-3020
        "kuoyin2" => u05_exec(ctx, owner, "kuoyin2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/ow/skill.js:3239-3260
        "shanxian2" => u05_exec(ctx, owner, "shanxian2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:43-85
        "qiongguang" => u05_exec(ctx, owner, "qiongguang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/xianjian/skill.js:281-310
        "yuexing2" => u05_exec(ctx, owner, "yuexing2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuijin2", 1)),
        // character/xianjian/skill.js:358-415
        "lingdi" => u05_exec(ctx, owner, "lingdi", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:704-749
        "guijin2" => u05_exec(ctx, owner, "guijin2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:750-772
        "chengxin2" => u05_exec(ctx, owner, "chengxin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_debuff:chengxin", 1)),
        // character/xianjian/skill.js:773-815
        "tianwu" => u05_exec(ctx, owner, "tianwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/xianjian/skill.js:1535-1580
        "qixia" => u05_exec(ctx, owner, "qixia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ExtraTurn),
        // character/xianjian/skill.js:1830-1850
        "yanzhan2" => u05_exec(ctx, owner, "yanzhan2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:2137-2205
        "danqing" | "danqing_count" => u05_exec(ctx, owner, skill, 0, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:2345-2365
        "zhangmu_ai" => u05_exec(ctx, owner, "zhangmu_ai", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:2413-2441
        "leiyu2" | "leiyu4" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:2442-2480
        "feizhua" => u05_exec(ctx, owner, "feizhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:woliu", 1)),
        // character/xianjian/skill.js:2592-2676
        "fenxing" => u05_exec(ctx, owner, "fenxing", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:2677-2720
        "duxinshu" => u05_exec(ctx, owner, "duxinshu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/xianjian/skill.js:3549-3565
        "qianfang2" => u05_exec(ctx, owner, "qianfang2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/xianjian/skill.js:3674-3690
        "zhuyue2" => u05_exec(ctx, owner, "zhuyue2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/xianjian/skill.js:3923-3950
        "xjzhimeng3" => u05_exec(ctx, owner, "xjzhimeng3", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        _ => return false,
    }
    true
}
