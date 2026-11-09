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
        // character/ow/skill.js:5-41
        "woliu" => { set_flag(ctx.core, owner, "ow_skill:woliu", 1); u05_exec(ctx, owner, "woliu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:woliu", 1)); }
        // character/ow/skill.js:42-127
        "woliu2" => { set_flag(ctx.core, owner, "ow_skill:woliu2", 1); u05_exec(ctx, owner, "woliu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:woliu2", 1)); }
        // character/ow/skill.js:128-150
        "qianggu" => { set_flag(ctx.core, owner, "ow_skill:qianggu", 1); u05_exec(ctx, owner, "qianggu", 1, false, 2, 0, 0, 0, DamageNature::Normal, 0, 2, U05Extra::GrantBuff("u05_buff:qianggu", 1)); }
        // character/ow/skill.js:151-216
        "qianggu2" => { set_flag(ctx.core, owner, "ow_skill:qianggu2", 1); u05_exec(ctx, owner, "qianggu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qianggu2", 1)); }
        // character/ow/skill.js:217-240
        "dianji" => { set_flag(ctx.core, owner, "ow_skill:dianji", 1); u05_exec(ctx, owner, "dianji", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jingleishan")); }
        // character/ow/skill.js:241-286
        "feitiao" => { set_flag(ctx.core, owner, "ow_skill:feitiao", 1); u05_exec(ctx, owner, "feitiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:feitiao", 1)); }
        // character/ow/skill.js:287-300
        "feitiao2" => { set_flag(ctx.core, owner, "ow_skill:feitiao2", 1); u05_exec(ctx, owner, "feitiao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feitiao2", 1)); }
        // character/ow/skill.js:301-345
        "zhencha" => { set_flag(ctx.core, owner, "ow_skill:zhencha", 1); u05_exec(ctx, owner, "zhencha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:zhencha", 1)); }
        // character/ow/skill.js:346-422
        "bshaowei" => { set_flag(ctx.core, owner, "ow_skill:bshaowei", 1); u05_exec(ctx, owner, "bshaowei", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bshaowei", 1)); }
        // character/ow/skill.js:424-509
        "pingzhang" => { set_flag(ctx.core, owner, "ow_skill:pingzhang", 1); u05_exec(ctx, owner, "pingzhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:pingzhang", 1)); }
        // character/ow/skill.js:512-521
        "owliyong" => { set_flag(ctx.core, owner, "ow_skill:owliyong", 1); u05_exec(ctx, owner, "owliyong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:owliyong", 1)); }
        // character/ow/skill.js:522-561
        "liangou" => { set_flag(ctx.core, owner, "ow_skill:liangou", 1); u05_exec(ctx, owner, "liangou", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:liangou")); }
        // character/ow/skill.js:562-578
        "liangou2" => { set_flag(ctx.core, owner, "ow_skill:liangou2", 1); u05_exec(ctx, owner, "liangou2", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:liangou2", 1)); }
        // character/ow/skill.js:579-593
        "xiyang" => { set_flag(ctx.core, owner, "ow_skill:xiyang", 1); u05_exec(ctx, owner, "xiyang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 2, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:594-702
        "qinru" => { set_flag(ctx.core, owner, "ow_skill:qinru", 1); u05_exec(ctx, owner, "qinru", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:qinru")); }
        // character/ow/skill.js:703-720
        "yinshen" => { set_flag(ctx.core, owner, "ow_skill:yinshen", 1); u05_exec(ctx, owner, "yinshen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yinshen", 1)); }
        // character/ow/skill.js:764-774
        "mengji" => { set_flag(ctx.core, owner, "ow_skill:mengji", 1); u05_exec(ctx, owner, "mengji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mengji", 1)); }
        // character/ow/skill.js:818-833
        "maoding" => { set_flag(ctx.core, owner, "ow_skill:maoding", 1); u05_exec(ctx, owner, "maoding", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:maoding", 1)); }
        // character/ow/skill.js:834-855
        "maoding2" => { set_flag(ctx.core, owner, "ow_skill:maoding2", 1); u05_exec(ctx, owner, "maoding2", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:maoding2", 1)); }
        // character/ow/skill.js:856-894
        "paotai" => { set_flag(ctx.core, owner, "ow_skill:paotai", 1); u05_exec(ctx, owner, "paotai", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:paotai", 1)); }
        // character/ow/skill.js:895-923
        "paotai2" => { set_flag(ctx.core, owner, "ow_skill:paotai2", 1); u05_exec(ctx, owner, "paotai2", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_buff:paotai2", 1)); }
        // character/ow/skill.js:924-940
        "paotai3" => { set_flag(ctx.core, owner, "ow_skill:paotai3", 1); u05_exec(ctx, owner, "paotai3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:paotai3", 1)); }
        // character/ow/skill.js:941-963
        "bfengshi" => { set_flag(ctx.core, owner, "ow_skill:bfengshi", 1); u05_exec(ctx, owner, "bfengshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:bfengshi", 1)); }
        // character/ow/skill.js:964-980
        "bfengshi2" => { set_flag(ctx.core, owner, "ow_skill:bfengshi2", 1); u05_exec(ctx, owner, "bfengshi2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bfengshi2", 1)); }
        // character/ow/skill.js:981-993
        "bfengshi3" => { set_flag(ctx.core, owner, "ow_skill:bfengshi3", 1); u05_exec(ctx, owner, "bfengshi3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:bfengshi3", 1)); }
        // character/ow/skill.js:995-1034
        "yinbo" => { set_flag(ctx.core, owner, "ow_skill:yinbo", 1); u05_exec(ctx, owner, "yinbo", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/ow/skill.js:1035-1074
        "aqianghua" => { set_flag(ctx.core, owner, "ow_skill:aqianghua", 1); u05_exec(ctx, owner, "aqianghua", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantOtherBuff("u05_debuff:aqianghua", 1)); }
        // character/ow/skill.js:1075-1087
        "aqianghua2" => { set_flag(ctx.core, owner, "ow_skill:aqianghua2", 1); u05_exec(ctx, owner, "aqianghua2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:aqianghua2", 1)); }
        // character/ow/skill.js:1088-1097
        "shihuo" => { set_flag(ctx.core, owner, "ow_skill:shihuo", 1); u05_exec(ctx, owner, "shihuo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shihuo", 1)); }
        // character/ow/skill.js:1098-1127
        "shoujia" => { set_flag(ctx.core, owner, "ow_skill:shoujia", 1); u05_exec(ctx, owner, "shoujia", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:shoujia", 1)); }
        // character/ow/skill.js:1128-1163
        "shoujia2" => { set_flag(ctx.core, owner, "ow_skill:shoujia2", 1); u05_exec(ctx, owner, "shoujia2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:1164-1180
        "shoujia3" => { set_flag(ctx.core, owner, "ow_skill:shoujia3", 1); u05_exec(ctx, owner, "shoujia3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shoujia3", 1)); }
        // character/ow/skill.js:1181-1218
        "liudan" => { set_flag(ctx.core, owner, "ow_skill:liudan", 1); u05_exec(ctx, owner, "liudan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:liudan", 1)); }
        // character/ow/skill.js:1219-1228
        "shenqiang" => { set_flag(ctx.core, owner, "ow_skill:shenqiang", 1); u05_exec(ctx, owner, "shenqiang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shenqiang", 1)); }
        // character/ow/skill.js:1229-1251
        "tiandan" => { set_flag(ctx.core, owner, "ow_skill:tiandan", 1); u05_exec(ctx, owner, "tiandan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tiandan", 1)); }
        // character/ow/skill.js:1252-1284
        "shanguang" => { set_flag(ctx.core, owner, "ow_skill:shanguang", 1); u05_exec(ctx, owner, "shanguang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:shanguang", 1)); }
        // character/ow/skill.js:1285-1311
        "shanguang2" => { set_flag(ctx.core, owner, "ow_skill:shanguang2", 1); u05_exec(ctx, owner, "shanguang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:shanguang2", 1)); }
        // character/ow/skill.js:1312-1383
        "baoxue" => { set_flag(ctx.core, owner, "ow_skill:baoxue", 1); u05_exec(ctx, owner, "baoxue", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/ow/skill.js:1415-1436
        "mianzhen2" => { set_flag(ctx.core, owner, "ow_skill:mianzhen2", 1); u05_exec(ctx, owner, "mianzhen2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mianzhen2", 1)); }
        // character/ow/skill.js:1437-1476
        "zhiyuan" => { set_flag(ctx.core, owner, "ow_skill:zhiyuan", 1); u05_exec(ctx, owner, "zhiyuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:zhiyuan", 1)); }
        // character/ow/skill.js:1477-1492
        "duwen" => { set_flag(ctx.core, owner, "ow_skill:duwen", 1); u05_exec(ctx, owner, "duwen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duwen", 1)); }
        // character/ow/skill.js:1493-1502
        "duwen2" => { set_flag(ctx.core, owner, "ow_skill:duwen2", 1); u05_exec(ctx, owner, "duwen2", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duwen2", 1)); }
        // character/ow/skill.js:1503-1589
        "juji" => { set_flag(ctx.core, owner, "ow_skill:juji", 1); u05_exec(ctx, owner, "juji", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:juji", 1)); }
        // character/ow/skill.js:1590-1609
        "juji2" => { set_flag(ctx.core, owner, "ow_skill:juji2", 1); u05_exec(ctx, owner, "juji2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:juji2", 1)); }
        // character/ow/skill.js:1610-1626
        "juji3" => { set_flag(ctx.core, owner, "ow_skill:juji3", 1); u05_exec(ctx, owner, "juji3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:juji3", 1)); }
        // character/ow/skill.js:1627-1651
        "dulei" => { set_flag(ctx.core, owner, "ow_skill:dulei", 1); u05_exec(ctx, owner, "dulei", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:dulei", 1)); }
        // character/ow/skill.js:1652-1689
        "dulei2" => { set_flag(ctx.core, owner, "ow_skill:dulei2", 1); u05_exec(ctx, owner, "dulei2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/ow/skill.js:1790-1804
        "bingqiang_remove" => { set_flag(ctx.core, owner, "ow_skill:bingqiang_remove", 1); u05_exec(ctx, owner, "bingqiang_remove", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:bingqiang_remove", 1)); }
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
        // character/ow/skill.js:1927-1960
        "tuijin" => { set_flag(ctx.core, owner, "ow_skill:tuijin", 1); u05_exec(ctx, owner, "tuijin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tuijin", 1)); }
        // character/ow/skill.js:1961-1974
        "tuijin2" => { set_flag(ctx.core, owner, "ow_skill:tuijin2", 1); u05_exec(ctx, owner, "tuijin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuijin2", 1)); }
        // character/ow/skill.js:1975-2016
        "jijia" => { set_flag(ctx.core, owner, "ow_skill:jijia", 1); u05_exec(ctx, owner, "jijia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jijia", 1)); }
        // character/ow/skill.js:2105-2144
        "xiandan" => { set_flag(ctx.core, owner, "ow_skill:xiandan", 1); u05_exec(ctx, owner, "xiandan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xiandan", 1)); }
        // character/ow/skill.js:2145-2155
        "xiandan2" => { set_flag(ctx.core, owner, "ow_skill:xiandan2", 1); u05_exec(ctx, owner, "xiandan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiandan2", 1)); }
        // character/ow/skill.js:2156-2162
        "shouge" => { set_flag(ctx.core, owner, "ow_skill:shouge", 1); u05_exec(ctx, owner, "shouge", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shouge", 1)); }
        // character/ow/skill.js:2163-2171
        "tuji" => { set_flag(ctx.core, owner, "ow_skill:tuji", 1); u05_exec(ctx, owner, "tuji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuji", 1)); }
        // character/ow/skill.js:2172-2197
        "mujing" => { set_flag(ctx.core, owner, "ow_skill:mujing", 1); u05_exec(ctx, owner, "mujing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/ow/skill.js:2198-2209
        "mujing2" => { set_flag(ctx.core, owner, "ow_skill:mujing2", 1); u05_exec(ctx, owner, "mujing2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mujing2", 1)); }
        // character/ow/skill.js:2229-2256
        "lichang2" => { set_flag(ctx.core, owner, "ow_skill:lichang2", 1); u05_exec(ctx, owner, "lichang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lichang2", 1)); }
        // character/ow/skill.js:2322-2330
        "zhanlong2" => { set_flag(ctx.core, owner, "ow_skill:zhanlong2", 1); u05_exec(ctx, owner, "zhanlong2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhanlong2", 1)); }
        // character/ow/skill.js:2331-2355
        "feiren" => { set_flag(ctx.core, owner, "ow_skill:feiren", 1); u05_exec(ctx, owner, "feiren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feiren", 1)); }
        // character/ow/skill.js:2356-2386
        "feiren3" => { set_flag(ctx.core, owner, "ow_skill:feiren3", 1); u05_exec(ctx, owner, "feiren3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:feiren3", 1)); }
        // character/ow/skill.js:2387-2427
        "xie" => { set_flag(ctx.core, owner, "ow_skill:xie", 1); u05_exec(ctx, owner, "xie", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:xie", 1)); }
        // character/ow/skill.js:2428-2465
        "xie2" => { set_flag(ctx.core, owner, "ow_skill:xie2", 1); u05_exec(ctx, owner, "xie2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:xie2", 1)); }
        // character/ow/skill.js:2466-2479
        "xie3" => { set_flag(ctx.core, owner, "ow_skill:xie3", 1); u05_exec(ctx, owner, "xie3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xie3", 1)); }
        // character/ow/skill.js:2480-2491
        "xie4" => { set_flag(ctx.core, owner, "ow_skill:xie4", 1); u05_exec(ctx, owner, "xie4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xie4", 1)); }
        // character/ow/skill.js:2492-2534
        "luan" => { set_flag(ctx.core, owner, "ow_skill:luan", 1); u05_exec(ctx, owner, "luan", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:luan", 1)); }
        // character/ow/skill.js:2535-2549
        "luan2" => { set_flag(ctx.core, owner, "ow_skill:luan2", 1); u05_exec(ctx, owner, "luan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/ow/skill.js:2550-2558
        "luan3" => { set_flag(ctx.core, owner, "ow_skill:luan3", 1); u05_exec(ctx, owner, "luan3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:luan3", 1)); }
        // character/ow/skill.js:2559-2570
        "luan4" => { set_flag(ctx.core, owner, "ow_skill:luan4", 1); u05_exec(ctx, owner, "luan4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:luan4", 1)); }
        // character/ow/skill.js:2571-2642
        "sheng" => { set_flag(ctx.core, owner, "ow_skill:sheng", 1); u05_exec(ctx, owner, "sheng", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:2643-2657
        "sheng2" => { set_flag(ctx.core, owner, "ow_skill:sheng2", 1); u05_exec(ctx, owner, "sheng2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:sheng2", 1)); }
        // character/ow/skill.js:2658-2701
        "yihun" => { set_flag(ctx.core, owner, "ow_skill:yihun", 1); u05_exec(ctx, owner, "yihun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yihun", 1)); }
        // character/ow/skill.js:2702-2723
        "yihun2" => { set_flag(ctx.core, owner, "ow_skill:yihun2", 1); u05_exec(ctx, owner, "yihun2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yihun2", 1)); }
        // character/ow/skill.js:2724-2779
        "huoyu" => { set_flag(ctx.core, owner, "ow_skill:huoyu", 1); u05_exec(ctx, owner, "huoyu", 0, true, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huoyu", 1)); }
        // character/ow/skill.js:2780-2838
        "feidan" => { set_flag(ctx.core, owner, "ow_skill:feidan", 1); u05_exec(ctx, owner, "feidan", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:feidan", 1)); }
        // character/ow/skill.js:2839-2890
        "yuedong" => { set_flag(ctx.core, owner, "ow_skill:yuedong", 1); u05_exec(ctx, owner, "yuedong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yuedong", 1)); }
        // character/ow/skill.js:2891-2930
        "huhuan" => { set_flag(ctx.core, owner, "ow_skill:huhuan", 1); u05_exec(ctx, owner, "huhuan", 0, false, 2, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huhuan", 1)); }
        // character/ow/skill.js:2931-2999
        "kuoyin" => { set_flag(ctx.core, owner, "ow_skill:kuoyin", 1); u05_exec(ctx, owner, "kuoyin", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kuoyin", 1)); }
        // character/ow/skill.js:3000-3008
        "kuoyin2" => { set_flag(ctx.core, owner, "ow_skill:kuoyin2", 1); u05_exec(ctx, owner, "kuoyin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kuoyin2", 1)); }
        // character/ow/skill.js:3009-3065
        "guangshu" => { set_flag(ctx.core, owner, "ow_skill:guangshu", 1); u05_exec(ctx, owner, "guangshu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:guangshu", 1)); }
        // character/ow/skill.js:3066-3077
        "guangshu_diamond" => { set_flag(ctx.core, owner, "ow_skill:guangshu_diamond", 1); u05_exec(ctx, owner, "guangshu_diamond", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:guangshu_diamond", 1)); }
        // character/ow/skill.js:3078-3090
        "guangshu_heart" => { set_flag(ctx.core, owner, "ow_skill:guangshu_heart", 1); u05_exec(ctx, owner, "guangshu_heart", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:guangshu_heart", 1)); }
        // character/ow/skill.js:3091-3109
        "guangshu_club" => { set_flag(ctx.core, owner, "ow_skill:guangshu_club", 1); u05_exec(ctx, owner, "guangshu_club", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:guangshu_club", 1)); }
        // character/ow/skill.js:3110-3121
        "guangshu_spade" => { set_flag(ctx.core, owner, "ow_skill:guangshu_spade", 1); u05_exec(ctx, owner, "guangshu_spade", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:guangshu_spade", 1)); }
        // character/ow/skill.js:3122-3131
        "ziyu" => { set_flag(ctx.core, owner, "ow_skill:ziyu", 1); u05_exec(ctx, owner, "ziyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:ziyu", 1)); }
        // character/ow/skill.js:3132-3160
        "shouhu" => { set_flag(ctx.core, owner, "ow_skill:shouhu", 1); u05_exec(ctx, owner, "shouhu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:shouhu", 1)); }
        // character/ow/skill.js:3161-3238
        "shanxian" => { set_flag(ctx.core, owner, "ow_skill:shanxian", 1); u05_exec(ctx, owner, "shanxian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf); }
        // character/ow/skill.js:3239-3246
        "shanxian2" => { set_flag(ctx.core, owner, "ow_skill:shanxian2", 1); u05_exec(ctx, owner, "shanxian2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shanxian2", 1)); }
        // character/xianjian/skill.js:5-25
        "lingquan" => { set_flag(ctx.core, owner, "xianjian_skill:lingquan", 1); u05_exec(ctx, owner, "lingquan", 0, true, 0, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lingquan", 1)); }
        // character/xianjian/skill.js:26-42
        "shenwu" => { set_flag(ctx.core, owner, "xianjian_skill:shenwu", 1); u05_exec(ctx, owner, "shenwu", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:43-92
        "qiongguang" => { set_flag(ctx.core, owner, "xianjian_skill:qiongguang", 1); u05_exec(ctx, owner, "qiongguang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qiongguang", 1)); }
        // character/xianjian/skill.js:93-151
        "txianqu" => { set_flag(ctx.core, owner, "xianjian_skill:txianqu", 1); u05_exec(ctx, owner, "txianqu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:txianqu", 1)); }
        // character/xianjian/skill.js:153-171
        "xunying" => { set_flag(ctx.core, owner, "xianjian_skill:xunying", 1); u05_exec(ctx, owner, "xunying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xunying", 1)); }
        // character/xianjian/skill.js:172-237
        "liefeng" => { set_flag(ctx.core, owner, "xianjian_skill:liefeng", 1); u05_exec(ctx, owner, "liefeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:liefeng", 1)); }
        // character/xianjian/skill.js:238-280
        "yuexing" => { set_flag(ctx.core, owner, "xianjian_skill:yuexing", 1); u05_exec(ctx, owner, "yuexing", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:yuexing", 1)); }
        // character/xianjian/skill.js:281-296
        "yuexing2" => { set_flag(ctx.core, owner, "xianjian_skill:yuexing2", 1); u05_exec(ctx, owner, "yuexing2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuexing2", 1)); }
        // character/xianjian/skill.js:297-357
        "minsha" => { set_flag(ctx.core, owner, "xianjian_skill:minsha", 1); u05_exec(ctx, owner, "minsha", 1, false, 2, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_buff:minsha", 1)); }
        // character/xianjian/skill.js:358-410
        "lingdi" => { set_flag(ctx.core, owner, "xianjian_skill:lingdi", 1); u05_exec(ctx, owner, "lingdi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lingdi", 1)); }
        // character/xianjian/skill.js:411-438
        "xiaoyue" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue", 1); u05_exec(ctx, owner, "xiaoyue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:xiaoyue", 1)); }
        // character/xianjian/skill.js:439-447
        "xiaoyue2" => { set_flag(ctx.core, owner, "xianjian_skill:xiaoyue2", 1); u05_exec(ctx, owner, "xiaoyue2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:xiaoyue2", 1)); }
        // character/xianjian/skill.js:448-463
        "huanlei" => { set_flag(ctx.core, owner, "xianjian_skill:huanlei", 1); u05_exec(ctx, owner, "huanlei", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:huanlei", 1)); }
        // character/xianjian/skill.js:464-479
        "anwugu" => { set_flag(ctx.core, owner, "xianjian_skill:anwugu", 1); u05_exec(ctx, owner, "anwugu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:anwugu", 1)); }
        // character/xianjian/skill.js:480-512
        "anwugu2" => { set_flag(ctx.core, owner, "xianjian_skill:anwugu2", 1); u05_exec(ctx, owner, "anwugu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:anwugu2", 1)); }
        // character/xianjian/skill.js:513-573
        "xtanxi" => { set_flag(ctx.core, owner, "xianjian_skill:xtanxi", 1); u05_exec(ctx, owner, "xtanxi", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther); }
        // character/xianjian/skill.js:574-592
        "linghuo" => { set_flag(ctx.core, owner, "xianjian_skill:linghuo", 1); u05_exec(ctx, owner, "linghuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:linghuo", 1)); }
        // character/xianjian/skill.js:593-703
        "guijin" => { set_flag(ctx.core, owner, "xianjian_skill:guijin", 1); u05_exec(ctx, owner, "guijin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guijin", 1)); }
        // character/xianjian/skill.js:704-718
        "guijin2" => { set_flag(ctx.core, owner, "xianjian_skill:guijin2", 1); u05_exec(ctx, owner, "guijin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:guijin2", 1)); }
        // character/xianjian/skill.js:719-749
        "chengxin" => { set_flag(ctx.core, owner, "xianjian_skill:chengxin", 1); u05_exec(ctx, owner, "chengxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantOtherBuff("u05_debuff:chengxin", 1)); }
        // character/xianjian/skill.js:750-772
        "chengxin2" => { set_flag(ctx.core, owner, "xianjian_skill:chengxin2", 1); u05_exec(ctx, owner, "chengxin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:chengxin2", 1)); }
        // character/xianjian/skill.js:773-783
        "tianwu" => { set_flag(ctx.core, owner, "xianjian_skill:tianwu", 1); u05_exec(ctx, owner, "tianwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tianwu", 1)); }
        // character/xianjian/skill.js:785-832
        "shiying" => { set_flag(ctx.core, owner, "xianjian_skill:shiying", 1); u05_exec(ctx, owner, "shiying", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shiying", 1)); }
        // character/xianjian/skill.js:833-872
        "liguang" => { set_flag(ctx.core, owner, "xianjian_skill:liguang", 1); u05_exec(ctx, owner, "liguang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:liguang", 1)); }
        // character/xianjian/skill.js:873-900
        "xiepan" => { set_flag(ctx.core, owner, "xianjian_skill:xiepan", 1); u05_exec(ctx, owner, "xiepan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiepan", 1)); }
        // character/xianjian/skill.js:973-1026
        "yanshi" => { set_flag(ctx.core, owner, "xianjian_skill:yanshi", 1); u05_exec(ctx, owner, "yanshi", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:1028-1050
        "tanhua" => { set_flag(ctx.core, owner, "xianjian_skill:tanhua", 1); u05_exec(ctx, owner, "tanhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:tanhua", 1)); }
        // character/xianjian/skill.js:1051-1073
        "xjyingfeng" => { set_flag(ctx.core, owner, "xianjian_skill:xjyingfeng", 1); u05_exec(ctx, owner, "xjyingfeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xjyingfeng", 1)); }
        // character/xianjian/skill.js:1074-1248
        "ywuhun" => { set_flag(ctx.core, owner, "xianjian_skill:ywuhun", 1); u05_exec(ctx, owner, "ywuhun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:ywuhun", 1)); }
        // character/xianjian/skill.js:1249-1384
        "fenglue" => { set_flag(ctx.core, owner, "xianjian_skill:fenglue", 1); u05_exec(ctx, owner, "fenglue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:fenglue", 1)); }
        // character/xianjian/skill.js:1385-1408
        "xjzongyu" => { set_flag(ctx.core, owner, "xianjian_skill:xjzongyu", 1); u05_exec(ctx, owner, "xjzongyu", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xjzongyu", 1)); }
        // character/xianjian/skill.js:1409-1422
        "fanling" => { set_flag(ctx.core, owner, "xianjian_skill:fanling", 1); u05_exec(ctx, owner, "fanling", 1, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:fanling", 1)); }
        // character/xianjian/skill.js:1423-1446
        "dujiang" => { set_flag(ctx.core, owner, "xianjian_skill:dujiang", 1); u05_exec(ctx, owner, "dujiang", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:dujiang", 1)); }
        // character/xianjian/skill.js:1447-1467
        "sheying" => { set_flag(ctx.core, owner, "xianjian_skill:sheying", 1); u05_exec(ctx, owner, "sheying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:sheying")); }
        // character/xianjian/skill.js:1468-1518
        "huahu" => { set_flag(ctx.core, owner, "xianjian_skill:huahu", 1); u05_exec(ctx, owner, "huahu", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::LoseMaxHp(1)); }
        // character/xianjian/skill.js:1519-1534
        "binxin" => { set_flag(ctx.core, owner, "xianjian_skill:binxin", 1); u05_exec(ctx, owner, "binxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:binxin", 1)); }
        // character/xianjian/skill.js:1535-1580
        "qixia" => { set_flag(ctx.core, owner, "xianjian_skill:qixia", 1); u05_exec(ctx, owner, "qixia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qixia", 1)); }
        // character/xianjian/skill.js:1620-1754
        "husha" => { set_flag(ctx.core, owner, "xianjian_skill:husha", 1); u05_exec(ctx, owner, "husha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:husha", 1)); }
        // character/xianjian/skill.js:1755-1768
        "fenshi" => { set_flag(ctx.core, owner, "xianjian_skill:fenshi", 1); u05_exec(ctx, owner, "fenshi", 0, true, 0, 2, 0, 0, DamageNature::Normal, 0, 2, U05Extra::GrantBuff("u05_buff:fenshi", 1)); }
        // character/xianjian/skill.js:1769-1798
        "longhuo" => { set_flag(ctx.core, owner, "xianjian_skill:longhuo", 1); u05_exec(ctx, owner, "longhuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:longhuo", 1)); }
        // character/xianjian/skill.js:1799-1829
        "yanzhan" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan", 1); u05_exec(ctx, owner, "yanzhan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/xianjian/skill.js:1830-1840
        "yanzhan2" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan2", 1); u05_exec(ctx, owner, "yanzhan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yanzhan2", 1)); }
        // character/xianjian/skill.js:1841-1849
        "yanzhan3" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan3", 1); u05_exec(ctx, owner, "yanzhan3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yanzhan3", 1)); }
        // character/xianjian/skill.js:1850-1858
        "yanzhan4" => { set_flag(ctx.core, owner, "xianjian_skill:yanzhan4", 1); u05_exec(ctx, owner, "yanzhan4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yanzhan4", 1)); }
        // character/xianjian/skill.js:1859-1883
        "xjyufeng" => { set_flag(ctx.core, owner, "xianjian_skill:xjyufeng", 1); u05_exec(ctx, owner, "xjyufeng", 2, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xjyufeng", 1)); }
        // character/xianjian/skill.js:1884-1909
        "feixia" => { set_flag(ctx.core, owner, "xianjian_skill:feixia", 1); u05_exec(ctx, owner, "feixia", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:feixia", 1)); }
        // character/xianjian/skill.js:1910-1958
        "lueying" => { set_flag(ctx.core, owner, "xianjian_skill:lueying", 1); u05_exec(ctx, owner, "lueying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:1985-2008
        "feng" => { set_flag(ctx.core, owner, "xianjian_skill:feng", 1); u05_exec(ctx, owner, "feng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:feng", 1)); }
        // character/xianjian/skill.js:2009-2037
        "ya" => { set_flag(ctx.core, owner, "xianjian_skill:ya", 1); u05_exec(ctx, owner, "ya", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:ya", 1)); }
        // character/xianjian/skill.js:2038-2060
        "song" => { set_flag(ctx.core, owner, "xianjian_skill:song", 1); u05_exec(ctx, owner, "song", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:song", 1)); }
        // character/xianjian/skill.js:2061-2074
        "longxiang" => { set_flag(ctx.core, owner, "xianjian_skill:longxiang", 1); u05_exec(ctx, owner, "longxiang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:2105-2136
        "xuanmo" => { set_flag(ctx.core, owner, "xianjian_skill:xuanmo", 1); u05_exec(ctx, owner, "xuanmo", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanmo", 1)); }
        // character/xianjian/skill.js:2137-2184
        "danqing" => { set_flag(ctx.core, owner, "xianjian_skill:danqing", 1); u05_exec(ctx, owner, "danqing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:danqing", 1)); }
        // character/xianjian/skill.js:2185-2195
        "danqing_count" => { set_flag(ctx.core, owner, "xianjian_skill:danqing_count", 1); u05_exec(ctx, owner, "danqing_count", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:danqing_count", 1)); }
        // character/xianjian/skill.js:2196-2230
        "sheling" => { set_flag(ctx.core, owner, "xianjian_skill:sheling", 1); u05_exec(ctx, owner, "sheling", 3, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:sheling", 1)); }
        // character/xianjian/skill.js:2231-2297
        "zhaoyao" => { set_flag(ctx.core, owner, "xianjian_skill:zhaoyao", 1); u05_exec(ctx, owner, "zhaoyao", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::PindianOther); }
        // character/xianjian/skill.js:2298-2344
        "zhangmu" => { set_flag(ctx.core, owner, "xianjian_skill:zhangmu", 1); u05_exec(ctx, owner, "zhangmu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:zhangmu", 1)); }
        // character/xianjian/skill.js:2345-2354
        "zhangmu_ai" => { set_flag(ctx.core, owner, "xianjian_skill:zhangmu_ai", 1); u05_exec(ctx, owner, "zhangmu_ai", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhangmu_ai", 1)); }
        // character/xianjian/skill.js:2413-2419
        "leiyu2" => { set_flag(ctx.core, owner, "xianjian_skill:leiyu2", 1); u05_exec(ctx, owner, "leiyu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:leiyu2", 1)); }
        // character/xianjian/skill.js:2420-2431
        "leiyu3" => { set_flag(ctx.core, owner, "xianjian_skill:leiyu3", 1); u05_exec(ctx, owner, "leiyu3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:leiyu3", 1)); }
        // character/xianjian/skill.js:2432-2441
        "leiyu4" => { set_flag(ctx.core, owner, "xianjian_skill:leiyu4", 1); u05_exec(ctx, owner, "leiyu4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:leiyu4", 1)); }
        // character/xianjian/skill.js:2442-2501
        "feizhua" => { set_flag(ctx.core, owner, "xianjian_skill:feizhua", 1); u05_exec(ctx, owner, "feizhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:feizhua", 1)); }
        // character/xianjian/skill.js:2502-2508
        "lingxue" => { set_flag(ctx.core, owner, "xianjian_skill:lingxue", 1); u05_exec(ctx, owner, "lingxue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:lingxue", 1)); }
        // character/xianjian/skill.js:2509-2525
        "diesha" => { set_flag(ctx.core, owner, "xianjian_skill:diesha", 1); u05_exec(ctx, owner, "diesha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:2526-2559
        "guijiang" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang", 1); u05_exec(ctx, owner, "guijiang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:guijiang", 1)); }
        // character/xianjian/skill.js:2560-2582
        "guijiang2" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang2", 1); u05_exec(ctx, owner, "guijiang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:guijiang2", 1)); }
        // character/xianjian/skill.js:2583-2591
        "guijiang3" => { set_flag(ctx.core, owner, "xianjian_skill:guijiang3", 1); u05_exec(ctx, owner, "guijiang3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:guijiang3", 1)); }
        // character/xianjian/skill.js:2592-2618
        "fenxing" => { set_flag(ctx.core, owner, "xianjian_skill:fenxing", 1); u05_exec(ctx, owner, "fenxing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:fenxing", 1)); }
        // character/xianjian/skill.js:2619-2649
        "diewu" => { set_flag(ctx.core, owner, "xianjian_skill:diewu", 1); u05_exec(ctx, owner, "diewu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:diewu", 1)); }
        // character/xianjian/skill.js:2651-2676
        "lingyu" => { set_flag(ctx.core, owner, "xianjian_skill:lingyu", 1); u05_exec(ctx, owner, "lingyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:lingyu", 1)); }
        // character/xianjian/skill.js:2677-2721
        "duxinshu" => { set_flag(ctx.core, owner, "xianjian_skill:duxinshu", 1); u05_exec(ctx, owner, "duxinshu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duxinshu", 1)); }
        // character/xianjian/skill.js:2722-2739
        "feixu" => { set_flag(ctx.core, owner, "xianjian_skill:feixu", 1); u05_exec(ctx, owner, "feixu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:feixu", 1)); }
        // character/xianjian/skill.js:2740-2743
        "xuanyan" => { set_flag(ctx.core, owner, "xianjian_skill:xuanyan", 1); u05_exec(ctx, owner, "xuanyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanyan", 1)); }
        // character/xianjian/skill.js:2744-2753
        "xuanyan2" => { set_flag(ctx.core, owner, "xianjian_skill:xuanyan2", 1); u05_exec(ctx, owner, "xuanyan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanyan2", 1)); }
        // character/xianjian/skill.js:2754-2764
        "xuanyan3" => { set_flag(ctx.core, owner, "xianjian_skill:xuanyan3", 1); u05_exec(ctx, owner, "xuanyan3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:2765-2786
        "ningbin" => { set_flag(ctx.core, owner, "xianjian_skill:ningbin", 1); u05_exec(ctx, owner, "ningbin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:ningbin", 1)); }
        // character/xianjian/skill.js:2787-2809
        "xfenxin" => { set_flag(ctx.core, owner, "xianjian_skill:xfenxin", 1); u05_exec(ctx, owner, "xfenxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xfenxin", 1)); }
        // character/xianjian/skill.js:2810-2817
        "xfenxin2" => { set_flag(ctx.core, owner, "xianjian_skill:xfenxin2", 1); u05_exec(ctx, owner, "xfenxin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)); }
        // character/xianjian/skill.js:2818-2867
        "luanjian" => { set_flag(ctx.core, owner, "xianjian_skill:luanjian", 1); u05_exec(ctx, owner, "luanjian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")); }
        // character/xianjian/skill.js:2868-2880
        "luanjian2" => { set_flag(ctx.core, owner, "xianjian_skill:luanjian2", 1); u05_exec(ctx, owner, "luanjian2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:luanjian2", 1)); }
        // character/xianjian/skill.js:2881-2918
        "ctianfu" => { set_flag(ctx.core, owner, "xianjian_skill:ctianfu", 1); u05_exec(ctx, owner, "ctianfu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:ctianfu", 1)); }
        // character/xianjian/skill.js:2919-2950
        "ctianfu2" => { set_flag(ctx.core, owner, "xianjian_skill:ctianfu2", 1); u05_exec(ctx, owner, "ctianfu2", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_buff:ctianfu2", 1)); }
        // character/xianjian/skill.js:2951-2961
        "ctianfu3" => { set_flag(ctx.core, owner, "xianjian_skill:ctianfu3", 1); u05_exec(ctx, owner, "ctianfu3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:ctianfu3", 1)); }
        // character/xianjian/skill.js:2962-3047
        "shuiyun" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun", 1); u05_exec(ctx, owner, "shuiyun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:shuiyun")); }
        // character/xianjian/skill.js:3048-3089
        "shuiyun5" => { set_flag(ctx.core, owner, "xianjian_skill:shuiyun5", 1); u05_exec(ctx, owner, "shuiyun5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:shuiyun5", 1)); }
        // character/xianjian/skill.js:3090-3151
        "wangyou" => { set_flag(ctx.core, owner, "xianjian_skill:wangyou", 1); u05_exec(ctx, owner, "wangyou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:wangyou", 1)); }
        // character/xianjian/skill.js:3152-3161
        "wangyou2" => { set_flag(ctx.core, owner, "xianjian_skill:wangyou2", 1); u05_exec(ctx, owner, "wangyou2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:wangyou2", 1)); }
        // character/xianjian/skill.js:3163-3199
        "changnian" => { set_flag(ctx.core, owner, "xianjian_skill:changnian", 1); u05_exec(ctx, owner, "changnian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:changnian", 1)); }
        // character/xianjian/skill.js:3200-3207
        "changnian2" => { set_flag(ctx.core, owner, "xianjian_skill:changnian2", 1); u05_exec(ctx, owner, "changnian2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:changnian2", 1)); }
        // character/xianjian/skill.js:3208-3243
        "sajin" => { set_flag(ctx.core, owner, "xianjian_skill:sajin", 1); u05_exec(ctx, owner, "sajin", 1, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::JudgeEffect("u05_judge:sajin")); }
        // character/xianjian/skill.js:3244-3286
        "jtjubao" => { set_flag(ctx.core, owner, "xianjian_skill:jtjubao", 1); u05_exec(ctx, owner, "jtjubao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:jtjubao", 1)); }
        // character/xianjian/skill.js:3288-3328
        "duci" => { set_flag(ctx.core, owner, "xianjian_skill:duci", 1); u05_exec(ctx, owner, "duci", 0, false, 0, 0, 0, -1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duci", 1)); }
        // character/xianjian/skill.js:3329-3384
        "xshuangren" => { set_flag(ctx.core, owner, "xianjian_skill:xshuangren", 1); u05_exec(ctx, owner, "xshuangren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xshuangren", 1)); }
        // character/xianjian/skill.js:3385-3397
        "xshuangren2" => { set_flag(ctx.core, owner, "xianjian_skill:xshuangren2", 1); u05_exec(ctx, owner, "xshuangren2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xshuangren2", 1)); }
        // character/xianjian/skill.js:3398-3415
        "guiyuan" => { set_flag(ctx.core, owner, "xianjian_skill:guiyuan", 1); u05_exec(ctx, owner, "guiyuan", 1, false, 1, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:guiyuan", 1)); }
        // character/xianjian/skill.js:3436-3471
        "shenmu" => { set_flag(ctx.core, owner, "xianjian_skill:shenmu", 1); u05_exec(ctx, owner, "shenmu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:shenmu", 1)); }
        // character/xianjian/skill.js:3472-3548
        "qianfang" => { set_flag(ctx.core, owner, "xianjian_skill:qianfang", 1); u05_exec(ctx, owner, "qianfang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qianfang", 1)); }
        // character/xianjian/skill.js:3549-3556
        "qianfang2" => { set_flag(ctx.core, owner, "xianjian_skill:qianfang2", 1); u05_exec(ctx, owner, "qianfang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qianfang2", 1)); }
        // character/xianjian/skill.js:3557-3581
        "poyun" => { set_flag(ctx.core, owner, "xianjian_skill:poyun", 1); u05_exec(ctx, owner, "poyun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:3582-3594
        "poyun2" => { set_flag(ctx.core, owner, "xianjian_skill:poyun2", 1); u05_exec(ctx, owner, "poyun2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:poyun2", 1)); }
        // character/xianjian/skill.js:3596-3673
        "zhuyue" => { set_flag(ctx.core, owner, "xianjian_skill:zhuyue", 1); u05_exec(ctx, owner, "zhuyue", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)); }
        // character/xianjian/skill.js:3674-3680
        "zhuyue2" => { set_flag(ctx.core, owner, "xianjian_skill:zhuyue2", 1); u05_exec(ctx, owner, "zhuyue2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:zhuyue2", 1)); }
        // character/xianjian/skill.js:3681-3732
        "longxi" => { set_flag(ctx.core, owner, "xianjian_skill:longxi", 1); u05_exec(ctx, owner, "longxi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:longxi", 1)); }
        // character/xianjian/skill.js:3733-3780
        "guanri" => { set_flag(ctx.core, owner, "xianjian_skill:guanri", 1); u05_exec(ctx, owner, "guanri", 0, false, 0, 0, 0, 2, DamageNature::Fire, 0, 0, U05Extra::LoseHpSelf(1)); }
        // character/xianjian/skill.js:3781-3813
        "tianxian" => { set_flag(ctx.core, owner, "xianjian_skill:tianxian", 1); u05_exec(ctx, owner, "tianxian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:tianxian")); }
        // character/xianjian/skill.js:3814-3855
        "runxin" => { set_flag(ctx.core, owner, "xianjian_skill:runxin", 1); u05_exec(ctx, owner, "runxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:runxin", 1)); }
        // character/xianjian/skill.js:3856-3896
        "xjzhimeng" => { set_flag(ctx.core, owner, "xianjian_skill:xjzhimeng", 1); u05_exec(ctx, owner, "xjzhimeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:xjzhimeng", 1)); }
        // character/xianjian/skill.js:3897-3922
        "xjzhimeng2" => { set_flag(ctx.core, owner, "xianjian_skill:xjzhimeng2", 1); u05_exec(ctx, owner, "xjzhimeng2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xjzhimeng2", 1)); }
        // character/xianjian/skill.js:3923-3947
        "xjzhimeng3" => { set_flag(ctx.core, owner, "xianjian_skill:xjzhimeng3", 1); u05_exec(ctx, owner, "xjzhimeng3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xjzhimeng3", 1)); }
        // character/xianjian/skill.js:3948-3986
        "tannang" => { set_flag(ctx.core, owner, "xianjian_skill:tannang", 1); u05_exec(ctx, owner, "tannang", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shunshou")); }
        // character/xianjian/skill.js:3987-4001
        "tuoqiao" => { set_flag(ctx.core, owner, "xianjian_skill:tuoqiao", 1); u05_exec(ctx, owner, "tuoqiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shihuifen")); }
        // character/xianjian/skill.js:4002-4011
        "huimeng" => { set_flag(ctx.core, owner, "xianjian_skill:huimeng", 1); u05_exec(ctx, owner, "huimeng", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:huimeng", 1)); }
        // character/xianjian/skill.js:4012-4042
        "tianshe" => { set_flag(ctx.core, owner, "xianjian_skill:tianshe", 1); u05_exec(ctx, owner, "tianshe", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tianshe", 1)); }
        // character/xianjian/skill.js:4043-4054
        "tianshe2" => { set_flag(ctx.core, owner, "xianjian_skill:tianshe2", 1); u05_exec(ctx, owner, "tianshe2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:tianshe2", 1)); }
        // character/*/character.js cross-ref `beige`
        "beige" => u05_exec(ctx, owner, "beige", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_deflect:beige", 1)),
        // character/*/character.js cross-ref `biyue`
        "biyue" => u05_exec(ctx, owner, "biyue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:biyue", 1)),
        // character/*/character.js cross-ref `feiying`
        "feiying" => u05_exec(ctx, owner, "feiying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:feiying", 1)),
        // character/*/character.js cross-ref `gongxin`
        "gongxin" => u05_exec(ctx, owner, "gongxin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/*/character.js cross-ref `hujia`
        "hujia" => u05_exec(ctx, owner, "hujia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/*/character.js cross-ref `jiang`
        "jiang" => u05_exec(ctx, owner, "jiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:jiang", 1)),
        // character/*/character.js cross-ref `jiuchi`
        "jiuchi" => u05_exec(ctx, owner, "jiuchi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jiuchi")),
        // character/*/character.js cross-ref `jueqing`
        "jueqing" => u05_exec(ctx, owner, "jueqing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_cross_mod:jueqing", 1)),
        // character/*/character.js cross-ref `lianpo`
        "lianpo" => u05_exec(ctx, owner, "lianpo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:lianpo", 1)),
        // character/*/character.js cross-ref `lieren`
        "lieren" => u05_exec(ctx, owner, "lieren", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/*/character.js cross-ref `liuli`
        "liuli" => u05_exec(ctx, owner, "liuli", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_deflect:liuli", 1)),
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
        "rejieming" => u05_exec(ctx, owner, "rejieming", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:rejieming", 1)),
        // character/*/character.js cross-ref `reyingzi`
        "reyingzi" => u05_exec(ctx, owner, "reyingzi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:reyingzi", 1)),
        // character/*/character.js cross-ref `shangshi`
        "shangshi" => u05_exec(ctx, owner, "shangshi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:shangshi", 1)),
        // character/*/character.js cross-ref `songci`
        "songci" => u05_exec(ctx, owner, "songci", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:songci", 1)),
        // character/*/character.js cross-ref `stonedraw`
        "stonedraw" => u05_exec(ctx, owner, "stonedraw", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_cross:stonedraw", 1)),
        // character/*/character.js cross-ref `stonesha`
        "stonesha" => u05_exec(ctx, owner, "stonesha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("stonesha")),
        // character/*/character.js cross-ref `stoneshan`
        "stoneshan" => u05_exec(ctx, owner, "stoneshan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("stoneshan")),
        // character/*/character.js cross-ref `tianxiang`
        "tianxiang" => u05_exec(ctx, owner, "tianxiang", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_deflect:tianxiang", 1)),
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
        _ => return false,
    }
    true
}
