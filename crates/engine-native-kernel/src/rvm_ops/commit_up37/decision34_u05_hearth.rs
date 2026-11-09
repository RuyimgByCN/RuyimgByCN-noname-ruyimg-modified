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
        "shihuawuqi" => u05_exec(ctx, owner, "shihuawuqi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:shihuawuqi", 1)),
        // card/hearth/skill.js:14-20
        "jihuocard2" => u05_exec(ctx, owner, "jihuocard2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jihuocard2", 1)),
        // character/hearth/skill.js:5-56
        "hshuanyu" => u05_exec(ctx, owner, "hshuanyu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hshuanyu", 1)),
        // character/hearth/skill.js:57-130
        "wxuying" => u05_exec(ctx, owner, "wxuying", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:wxuying", 1)),
        // character/hearth/skill.js:133-144
        "buwendingyibian_lose" => u05_exec(ctx, owner, "buwendingyibian_lose", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:buwendingyibian_lose", 1)),
        // character/hearth/skill.js:145-154
        "yibian" => u05_exec(ctx, owner, "yibian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yibian", 1)),
        // character/hearth/skill.js:155-207
        "hualing" => u05_exec(ctx, owner, "hualing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hualing", 1)),
        // character/hearth/skill.js:208-235
        "muyin" => u05_exec(ctx, owner, "muyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:muyin", 1)),
        // character/hearth/skill.js:236-272
        "tqchuanyue" => u05_exec(ctx, owner, "tqchuanyue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tqchuanyue", 1)),
        // character/hearth/skill.js:273-366
        "oldhuanjue" => u05_exec(ctx, owner, "oldhuanjue", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldhuanjue", 1)),
        // character/hearth/skill.js:367-464
        "hshuanling" => u05_exec(ctx, owner, "hshuanling", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hshuanling", 1)),
        // character/hearth/skill.js:465-480
        "hsxingyi" => u05_exec(ctx, owner, "hsxingyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hsxingyi", 1)),
        // character/hearth/skill.js:481-561
        "asyouzhang" => u05_exec(ctx, owner, "asyouzhang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:asyouzhang", 1)),
        // character/hearth/skill.js:562-678
        "ylyuchu" => u05_exec(ctx, owner, "ylyuchu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:ylyuchu")),
        // character/hearth/skill.js:679-709
        "nsaiqi" => u05_exec(ctx, owner, "nsaiqi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:nsaiqi", 1)),
        // character/hearth/skill.js:710-729
        "nsbeiming" => u05_exec(ctx, owner, "nsbeiming", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nsbeiming", 1)),
        // character/hearth/skill.js:730-902
        "hsnitai" => u05_exec(ctx, owner, "hsnitai", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hsnitai", 1)),
        // character/hearth/skill.js:903-938
        "hspuzhao" => u05_exec(ctx, owner, "hspuzhao", 1, false, 1, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hspuzhao", 1)),
        // character/hearth/skill.js:939-973
        "hsyanxin" => u05_exec(ctx, owner, "hsyanxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsyanxin", 1)),
        // character/hearth/skill.js:974-1106
        "hstianqi" => u05_exec(ctx, owner, "hstianqi", 1, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hstianqi", 1)),
        // character/hearth/skill.js:1107-1116
        "hstianqi_dalian" => u05_exec(ctx, owner, "hstianqi_dalian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:hstianqi_dalian", 1)),
        // character/hearth/skill.js:1117-1126
        "hstianqi_shali" => u05_exec(ctx, owner, "hstianqi_shali", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hstianqi_shali", 1)),
        // character/hearth/skill.js:1127-1279
        "ysjqisha" => u05_exec(ctx, owner, "ysjqisha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:1280-1320
        "yindan" => u05_exec(ctx, owner, "yindan", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:1321-1391
        "hllingxi" => u05_exec(ctx, owner, "hllingxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:1392-1413
        "zhaochao" => u05_exec(ctx, owner, "zhaochao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:zhaochao", 1)),
        // character/hearth/skill.js:1414-1423
        "zhaochao2" => u05_exec(ctx, owner, "zhaochao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:zhaochao2", 1)),
        // character/hearth/skill.js:1424-1453
        "xiyong" => u05_exec(ctx, owner, "xiyong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiyong", 1)),
        // character/hearth/skill.js:1454-1466
        "srjici" => u05_exec(ctx, owner, "srjici", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:srjici", 1)),
        // character/hearth/skill.js:1467-1481
        "yinzong" => u05_exec(ctx, owner, "yinzong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yinzong", 1)),
        // character/hearth/skill.js:1482-1519
        "tansuo" => u05_exec(ctx, owner, "tansuo", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tansuo", 1)),
        // character/hearth/skill.js:1520-1591
        "lieqi" => u05_exec(ctx, owner, "lieqi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lieqi", 1)),
        // character/hearth/skill.js:1592-1676
        "azaowu" => u05_exec(ctx, owner, "azaowu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:azaowu", 1)),
        // character/hearth/skill.js:1677-1704
        "shouwang" => u05_exec(ctx, owner, "shouwang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::GrantOtherBuff("u05_debuff:shouwang", 1)),
        // character/hearth/skill.js:1705-1711
        "shouwang2" => u05_exec(ctx, owner, "shouwang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shouwang2", 1)),
        // character/hearth/skill.js:1712-1727
        "qingtian" => u05_exec(ctx, owner, "qingtian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qingtian", 1)),
        // character/hearth/skill.js:1728-1746
        "qianfu" => u05_exec(ctx, owner, "qianfu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qianfu", 1)),
        // character/hearth/skill.js:1747-1814
        "qianfu2" => u05_exec(ctx, owner, "qianfu2", 0, false, 0, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qianfu2", 1)),
        // character/hearth/skill.js:1815-1828
        "shimo" => u05_exec(ctx, owner, "shimo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:shimo", 1)),
        // character/hearth/skill.js:1829-1846
        "lieyang" => u05_exec(ctx, owner, "lieyang", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lieyang", 1)),
        // character/hearth/skill.js:1847-1855
        "lieyang2" => u05_exec(ctx, owner, "lieyang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:lieyang2", 1)),
        // character/hearth/skill.js:1856-1892
        "zhuilie" => u05_exec(ctx, owner, "zhuilie", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhuilie", 1)),
        // character/hearth/skill.js:1893-1926
        "szbianshen" => u05_exec(ctx, owner, "szbianshen", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:szbianshen", 1)),
        // character/hearth/skill.js:1927-1963
        "kekao" => u05_exec(ctx, owner, "kekao", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:kekao", 1)),
        // character/hearth/skill.js:1964-2027
        "jinhua" => u05_exec(ctx, owner, "jinhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:jinhua", 1)),
        // character/hearth/skill.js:2028-2082
        "kqizhou" => u05_exec(ctx, owner, "kqizhou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kqizhou", 1)),
        // character/hearth/skill.js:2083-2109
        "jingcu" => u05_exec(ctx, owner, "jingcu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:2110-2119
        "shengzhang" => u05_exec(ctx, owner, "shengzhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:2120-2164
        "pyuhuo" => u05_exec(ctx, owner, "pyuhuo", 0, false, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:2165-2209
        "mengye" => u05_exec(ctx, owner, "mengye", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:mengye", 1)),
        // character/hearth/skill.js:2210-2249
        "mengye2" => u05_exec(ctx, owner, "mengye2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mengye2", 1)),
        // character/hearth/skill.js:2251-2265
        "lianzhan" => u05_exec(ctx, owner, "lianzhan", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:2267-2305
        "kuixin" => u05_exec(ctx, owner, "kuixin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:kuixin", 1)),
        // character/hearth/skill.js:2306-2360
        "fuhua" => u05_exec(ctx, owner, "fuhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:fuhua", 1)),
        // character/hearth/skill.js:2361-2399
        "fuhua2" => u05_exec(ctx, owner, "fuhua2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:fuhua2", 1)),
        // character/hearth/skill.js:2400-2411
        "fuhua3" => u05_exec(ctx, owner, "fuhua3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:fuhua3", 1)),
        // character/hearth/skill.js:2412-2448
        "moxie" => u05_exec(ctx, owner, "moxie", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:moxie", 1)),
        // character/hearth/skill.js:2449-2513
        "qianhou" => u05_exec(ctx, owner, "qianhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qianhou", 1)),
        // character/hearth/skill.js:2514-2522
        "hlongyi" => u05_exec(ctx, owner, "hlongyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hlongyi", 1)),
        // character/hearth/skill.js:2523-2549
        "zhongji" => u05_exec(ctx, owner, "zhongji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhongji", 1)),
        // character/hearth/skill.js:2550-2566
        "fuwen" => u05_exec(ctx, owner, "fuwen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:fuwen", 1)),
        // character/hearth/skill.js:2567-2603
        "jinzhou" => u05_exec(ctx, owner, "jinzhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:jinzhou", 1)),
        // character/hearth/skill.js:2604-2629
        "midian" => u05_exec(ctx, owner, "midian", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:midian", 1)),
        // character/hearth/skill.js:2630-2683
        "xingluo" => u05_exec(ctx, owner, "xingluo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xingluo", 1)),
        // character/hearth/skill.js:2684-2715
        "yuelu" => u05_exec(ctx, owner, "yuelu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::GrantBuff("u05_state:yuelu", 1)),
        // character/hearth/skill.js:2716-2796
        "yushou" => u05_exec(ctx, owner, "yushou", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yushou", 1)),
        // character/hearth/skill.js:2797-2812
        "yushou_lose" => u05_exec(ctx, owner, "yushou_lose", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yushou_lose", 1)),
        // character/hearth/skill.js:2813-2834
        "yushou_misha" => u05_exec(ctx, owner, "yushou_misha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:yushou_misha", 1)),
        // character/hearth/skill.js:2835-2853
        "yushou_huofu" => u05_exec(ctx, owner, "yushou_huofu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("juedou")),
        // character/hearth/skill.js:2854-2861
        "yushou_leiouke" => u05_exec(ctx, owner, "yushou_leiouke", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yushou_leiouke", 1)),
        // character/hearth/skill.js:2862-2923
        "qingzun" => u05_exec(ctx, owner, "qingzun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qingzun", 1)),
        // character/hearth/skill.js:2924-2956
        "ayuling" => u05_exec(ctx, owner, "ayuling", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:ayuling", 1)),
        // character/hearth/skill.js:2957-2978
        "aoshu" => u05_exec(ctx, owner, "aoshu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("wuzhong")),
        // character/hearth/skill.js:2979-2995
        "bzhuiji" => u05_exec(ctx, owner, "bzhuiji", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bzhuiji", 1)),
        // character/hearth/skill.js:2996-3186
        "lianjin" => u05_exec(ctx, owner, "lianjin", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lianjin", 1)),
        // character/hearth/skill.js:3187-3224
        "shouji" => u05_exec(ctx, owner, "shouji", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shouji", 1)),
        // character/hearth/skill.js:3225-3253
        "yingxi" => u05_exec(ctx, owner, "yingxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yingxi", 1)),
        // character/hearth/skill.js:3254-3264
        "yingxi2" => u05_exec(ctx, owner, "yingxi2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yingxi2", 1)),
        // character/hearth/skill.js:3265-3305
        "hsguimou" => u05_exec(ctx, owner, "hsguimou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsguimou", 1)),
        // character/hearth/skill.js:3306-3331
        "peiyu" => u05_exec(ctx, owner, "peiyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:peiyu", 1)),
        // character/hearth/skill.js:3332-3338
        "peiyu2" => u05_exec(ctx, owner, "peiyu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:peiyu2", 1)),
        // character/hearth/skill.js:3339-3369
        "wzhanyi" => u05_exec(ctx, owner, "wzhanyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:wzhanyi", 1)),
        // character/hearth/skill.js:3370-3380
        "shengteng" => u05_exec(ctx, owner, "shengteng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:3381-3400
        "yuansu" => u05_exec(ctx, owner, "yuansu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuansu", 1)),
        // character/hearth/skill.js:3401-3428
        "chouhuo" => u05_exec(ctx, owner, "chouhuo", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 2, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:3429-3512
        "nuyan2" => u05_exec(ctx, owner, "nuyan2", 3, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:3513-3602
        "nuyan" => u05_exec(ctx, owner, "nuyan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:nuyan", 1)),
        // character/hearth/skill.js:3603-3625
        "duxin" => u05_exec(ctx, owner, "duxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duxin", 1)),
        // character/hearth/skill.js:3626-3645
        "oldduxin" => u05_exec(ctx, owner, "oldduxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldduxin", 1)),
        // character/hearth/skill.js:3646-3660
        "hsdusu_shinancao" => u05_exec(ctx, owner, "hsdusu_shinancao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsdusu_shinancao", 1)),
        // character/hearth/skill.js:3661-3687
        "xiubu" => u05_exec(ctx, owner, "xiubu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiubu", 1)),
        // character/hearth/skill.js:3688-3729
        "zengli" => u05_exec(ctx, owner, "zengli", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zengli", 1)),
        // character/hearth/skill.js:3730-3771
        "mobao" => u05_exec(ctx, owner, "mobao", 1, false, 1, 0, 0, -1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_buff:mobao", 1)),
        // character/hearth/skill.js:3772-3787
        "mobao2" => u05_exec(ctx, owner, "mobao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mobao2", 1)),
        // character/hearth/skill.js:3788-3794
        "mobao3" => u05_exec(ctx, owner, "mobao3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mobao3", 1)),
        // character/hearth/skill.js:3795-3799
        "xianji" => u05_exec(ctx, owner, "xianji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xianji", 1)),
        // character/hearth/skill.js:3800-3844
        "xianji2" => u05_exec(ctx, owner, "xianji2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xianji2", 1)),
        // character/hearth/skill.js:3845-3858
        "xianji3" => u05_exec(ctx, owner, "xianji3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xianji3", 1)),
        // character/hearth/skill.js:3859-3885
        "tanmi" => u05_exec(ctx, owner, "tanmi", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tanmi", 1)),
        // character/hearth/skill.js:3886-3914
        "xueren" => u05_exec(ctx, owner, "xueren", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:3915-3987
        "maoxian" => u05_exec(ctx, owner, "maoxian", 2, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:maoxian", 1)),
        // character/hearth/skill.js:3988-4001
        "yiwen" => u05_exec(ctx, owner, "yiwen", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yiwen", 1)),
        // character/hearth/skill.js:4003-4057
        "qianghuax" => u05_exec(ctx, owner, "qianghuax", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qianghuax", 1)),
        // character/hearth/skill.js:4058-4087
        "zhuizong" => u05_exec(ctx, owner, "zhuizong", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhuizong", 1)),
        // character/hearth/skill.js:4088-4120
        "xunbao" => u05_exec(ctx, owner, "xunbao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xunbao", 1)),
        // character/hearth/skill.js:4121-4152
        "xunbao2" => u05_exec(ctx, owner, "xunbao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xunbao2", 1)),
        // character/hearth/skill.js:4153-4161
        "hsbaowu_cangbaotu" => u05_exec(ctx, owner, "hsbaowu_cangbaotu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsbaowu_cangbaotu", 1)),
        // character/hearth/skill.js:4162-4176
        "hsbaowu_huangjinyuanhou" => u05_exec(ctx, owner, "hsbaowu_huangjinyuanhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hsbaowu_huangjinyuanhou", 1)),
        // character/hearth/skill.js:4177-4184
        "hsbaowu_huangjinyuanhou2" => u05_exec(ctx, owner, "hsbaowu_huangjinyuanhou2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:hsbaowu_huangjinyuanhou2", 1)),
        // character/hearth/skill.js:4185-4208
        "xieneng" => u05_exec(ctx, owner, "xieneng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xieneng", 1)),
        // character/hearth/skill.js:4209-4262
        "fbeifa" => u05_exec(ctx, owner, "fbeifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:fbeifa", 1)),
        // character/hearth/skill.js:4263-4308
        "oldfbeifa" => u05_exec(ctx, owner, "oldfbeifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldfbeifa", 1)),
        // character/hearth/skill.js:4309-4334
        "yufa" => u05_exec(ctx, owner, "yufa", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yufa", 1)),
        // character/hearth/skill.js:4335-4344
        "yufa2" => u05_exec(ctx, owner, "yufa2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yufa2", 1)),
        // character/hearth/skill.js:4345-4351
        "yufa3" => u05_exec(ctx, owner, "yufa3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yufa3", 1)),
        // character/hearth/skill.js:4352-4395
        "bingyan" => u05_exec(ctx, owner, "bingyan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bingyan", 1)),
        // character/hearth/skill.js:4396-4441
        "shifa" => u05_exec(ctx, owner, "shifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shifa", 1)),
        // character/hearth/skill.js:4442-4486
        "oldshifa" => u05_exec(ctx, owner, "oldshifa", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldshifa", 1)),
        // character/hearth/skill.js:4487-4496
        "yuanzheng" => u05_exec(ctx, owner, "yuanzheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4497-4535
        "yulu" => u05_exec(ctx, owner, "yulu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4536-4574
        "oldyulu" => u05_exec(ctx, owner, "oldyulu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4575-4600
        "duzhang" => u05_exec(ctx, owner, "duzhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:duzhang", 1)),
        // character/hearth/skill.js:4601-4614
        "duzhang2" => u05_exec(ctx, owner, "duzhang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:duzhang2", 1)),
        // character/hearth/skill.js:4615-4673
        "hannu" => u05_exec(ctx, owner, "hannu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hannu", 1)),
        // character/hearth/skill.js:4674-4683
        "chuidiao" => u05_exec(ctx, owner, "chuidiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:chuidiao", 1)),
        // character/hearth/skill.js:4684-4715
        "hhudun" => u05_exec(ctx, owner, "hhudun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:hhudun", 1)),
        // character/hearth/skill.js:4716-4756
        "fenlie" => u05_exec(ctx, owner, "fenlie", 2, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:fenlie", 1)),
        // character/hearth/skill.js:4757-4796
        "oldfenlie" => u05_exec(ctx, owner, "oldfenlie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldfenlie", 1)),
        // character/hearth/skill.js:4797-4828
        "nianfu" => u05_exec(ctx, owner, "nianfu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:nianfu", 1)),
        // character/hearth/skill.js:4829-4876
        "xiaorong" => u05_exec(ctx, owner, "xiaorong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:xiaorong", 1)),
        // character/hearth/skill.js:4877-4915
        "shixu" => u05_exec(ctx, owner, "shixu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shixu", 1)),
        // character/hearth/skill.js:4916-4930
        "jixuan" => u05_exec(ctx, owner, "jixuan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jixuan", 1)),
        // character/hearth/skill.js:4931-4995
        "qianghua" => u05_exec(ctx, owner, "qianghua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qianghua", 1)),
        // character/hearth/skill.js:4997-5038
        "biri" => u05_exec(ctx, owner, "biri", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:biri", 1)),
        // character/hearth/skill.js:5039-5068
        "stuxi" => u05_exec(ctx, owner, "stuxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:stuxi", 1)),
        // character/hearth/skill.js:5069-5083
        "stuxi2" => u05_exec(ctx, owner, "stuxi2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:stuxi2", 1)),
        // character/hearth/skill.js:5084-5091
        "bingdong" => u05_exec(ctx, owner, "bingdong", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bingdong", 1)),
        // character/hearth/skill.js:5092-5115
        "luoshi" => u05_exec(ctx, owner, "luoshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:luoshi", 1)),
        // character/hearth/skill.js:5116-5137
        "ronghuo" => u05_exec(ctx, owner, "ronghuo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:ronghuo", 1)),
        // character/hearth/skill.js:5138-5145
        "ronghuo2" => u05_exec(ctx, owner, "ronghuo2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:ronghuo2", 1)),
        // character/hearth/skill.js:5146-5182
        "fushi" => u05_exec(ctx, owner, "fushi", 1, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseMaxHpOther(1)),
        // character/hearth/skill.js:5183-5218
        "oldfushi" => u05_exec(ctx, owner, "oldfushi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseMaxHpOther(1)),
        // character/hearth/skill.js:5219-5229
        "moyao" => u05_exec(ctx, owner, "moyao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:moyao", 1)),
        // character/hearth/skill.js:5230-5257
        "jiaohui" => u05_exec(ctx, owner, "jiaohui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jiaohui", 1)),
        // character/hearth/skill.js:5258-5270
        "bimeng" => u05_exec(ctx, owner, "bimeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bimeng", 1)),
        // character/hearth/skill.js:5271-5284
        "zhoujiang" => u05_exec(ctx, owner, "zhoujiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhoujiang", 1)),
        // character/hearth/skill.js:5285-5311
        "liehun" => u05_exec(ctx, owner, "liehun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:liehun", 1)),
        // character/hearth/skill.js:5312-5321
        "xjumo" => u05_exec(ctx, owner, "xjumo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:xjumo", 1)),
        // character/hearth/skill.js:5322-5345
        "malymowang" => u05_exec(ctx, owner, "malymowang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:malymowang", 1)),
        // character/hearth/skill.js:5346-5368
        "oldmalymowang" => u05_exec(ctx, owner, "oldmalymowang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldmalymowang", 1)),
        // character/hearth/skill.js:5369-5402
        "lingzhou" => u05_exec(ctx, owner, "lingzhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lingzhou", 1)),
        // character/hearth/skill.js:5403-5420
        "mieshi" => u05_exec(ctx, owner, "mieshi", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5421-5443
        "xmojian" => u05_exec(ctx, owner, "xmojian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xmojian", 1)),
        // character/hearth/skill.js:5444-5459
        "xshixin" => u05_exec(ctx, owner, "xshixin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5460-5513
        "enze" => u05_exec(ctx, owner, "enze", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:5514-5561
        "oldenze" => u05_exec(ctx, owner, "oldenze", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:5562-5623
        "chongsheng" => u05_exec(ctx, owner, "chongsheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5624-5646
        "guozai" => u05_exec(ctx, owner, "guozai", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guozai", 1)),
        // character/hearth/skill.js:5647-5659
        "guozai2" => u05_exec(ctx, owner, "guozai2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guozai2", 1)),
        // character/hearth/skill.js:5660-5682
        "guozaix" => u05_exec(ctx, owner, "guozaix", 2, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guozaix", 1)),
        // character/hearth/skill.js:5683-5695
        "guozaix2" => u05_exec(ctx, owner, "guozaix2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:guozaix2", 1)),
        // character/hearth/skill.js:5696-5719
        "hanshuang" => u05_exec(ctx, owner, "hanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5720-5742
        "oldhanshuang" => u05_exec(ctx, owner, "oldhanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5743-5746
        "bingshi" => u05_exec(ctx, owner, "bingshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:bingshi", 1)),
        // character/hearth/skill.js:5747-5767
        "bingshi2" => u05_exec(ctx, owner, "bingshi2", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:bingshi2", 1)),
        // character/hearth/skill.js:5768-5799
        "huanwu" => u05_exec(ctx, owner, "huanwu", 1, false, 0, 0, 2, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:5800-5824
        "fengnu" => u05_exec(ctx, owner, "fengnu", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:fengnu", 1)),
        // character/hearth/skill.js:5825-5835
        "shengdun" => u05_exec(ctx, owner, "shengdun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:shengdun", 1)),
        // character/hearth/skill.js:5836-5854
        "jingmeng" => u05_exec(ctx, owner, "jingmeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jingmeng", 1)),
        // character/hearth/skill.js:5855-5867
        "hswuji" => u05_exec(ctx, owner, "hswuji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hswuji", 1)),
        // character/hearth/skill.js:5868-5890
        "bingshuang" => u05_exec(ctx, owner, "bingshuang", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5891-5909
        "yanshu" => u05_exec(ctx, owner, "yanshu", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanshu", 1)),
        // character/hearth/skill.js:5910-5927
        "oldyanshu" => u05_exec(ctx, owner, "oldyanshu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:oldyanshu", 1)),
        // character/hearth/skill.js:5928-5944
        "shengyan" => u05_exec(ctx, owner, "shengyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:shengyan", 1)),
        // character/hearth/skill.js:5946-5963
        "liechao" => u05_exec(ctx, owner, "liechao", 1, false, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5964-5983
        "qingliu" => u05_exec(ctx, owner, "qingliu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:qingliu", 1)),
        // character/hearth/skill.js:5984-6016
        "zhongjia" => u05_exec(ctx, owner, "zhongjia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:zhongjia", 1)),
        // character/hearth/skill.js:6017-6047
        "dunji" => u05_exec(ctx, owner, "dunji", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:dunji", 1)),
        // character/hearth/skill.js:6048-6071
        "fengxing" => u05_exec(ctx, owner, "fengxing", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:fengxing", 1)),
        // character/hearth/skill.js:6073-6103
        "fengxian" => u05_exec(ctx, owner, "fengxian", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:6104-6119
        "hsmengjing_mengye" => u05_exec(ctx, owner, "hsmengjing_mengye", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsmengjing_mengye", 1)),
        // character/hearth/skill.js:6120-6140
        "zhanhou" => u05_exec(ctx, owner, "zhanhou", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:zhanhou", 1)),
        // character/hearth/skill.js:6141-6163
        "oldzhanhou" => u05_exec(ctx, owner, "oldzhanhou", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:oldzhanhou", 1)),
        // character/hearth/skill.js:6164-6193
        "shijie" => u05_exec(ctx, owner, "shijie", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shijie", 1)),
        // character/hearth/skill.js:6194-6240
        "shengguang" => u05_exec(ctx, owner, "shengguang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpOther(1)),
        // character/hearth/skill.js:6241-6268
        "xinci" => u05_exec(ctx, owner, "xinci", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/hearth/skill.js:6269-6303
        "anying" => u05_exec(ctx, owner, "anying", 0, true, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:anying", 1)),
        // character/hearth/skill.js:6304-6423
        "huanjue" => u05_exec(ctx, owner, "huanjue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:huanjue", 1)),
        // character/hearth/skill.js:6424-6448
        "bingjia" => u05_exec(ctx, owner, "bingjia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:bingjia", 1)),
        // character/hearth/skill.js:6449-6482
        "bingjia2" => u05_exec(ctx, owner, "bingjia2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_buff:bingjia2", 1)),
        // character/hearth/skill.js:6484-6521
        "zuzhou" => u05_exec(ctx, owner, "zuzhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zuzhou", 1)),
        // character/hearth/skill.js:6522-6558
        "mdzhoufu" => u05_exec(ctx, owner, "mdzhoufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:mdzhoufu", 1)),
        // character/hearth/skill.js:6559-6591
        "mdzhoufu2" => u05_exec(ctx, owner, "mdzhoufu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mdzhoufu2", 1)),
        // character/hearth/skill.js:6592-6636
        "xianzhi" => u05_exec(ctx, owner, "xianzhi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:xianzhi")),
        // character/hearth/skill.js:6637-6702
        "jingxiang" => u05_exec(ctx, owner, "jingxiang", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:jingxiang", 1)),
        // character/hearth/skill.js:6703-6722
        "wlianji" => u05_exec(ctx, owner, "wlianji", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:wlianji", 1)),
        // character/hearth/skill.js:6723-6778
        "mengun" => u05_exec(ctx, owner, "mengun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mengun", 1)),
        // character/hearth/skill.js:6779-6794
        "mengun2" => u05_exec(ctx, owner, "mengun2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mengun2", 1)),
        // character/hearth/skill.js:6795-6821
        "jianren" => u05_exec(ctx, owner, "jianren", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jianren", 1)),
        // character/hearth/skill.js:6822-6842
        "jihuo" => u05_exec(ctx, owner, "jihuo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jihuo", 1)),
        // character/hearth/skill.js:6843-6879
        "tuteng_s" => u05_exec(ctx, owner, "tuteng_s", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tuteng_s", 1)),
        // character/hearth/skill.js:6880-6901
        "s_tuteng" => u05_exec(ctx, owner, "s_tuteng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:s_tuteng", 1)),
        // character/hearth/skill.js:6902-7030
        "tuteng" => u05_exec(ctx, owner, "tuteng", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tuteng", 1)),
        // character/hearth/skill.js:7031-7058
        "zuling" => u05_exec(ctx, owner, "zuling", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:7059-7103
        "huanfeng" => u05_exec(ctx, owner, "huanfeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huanfeng", 1)),
        // character/hearth/skill.js:7104-7110
        "tuteng_h" => u05_exec(ctx, owner, "tuteng_h", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuteng_h", 1)),
        // character/hearth/skill.js:7111-7136
        "tuteng_lose" => u05_exec(ctx, owner, "tuteng_lose", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng_lose", 1)),
        // character/hearth/skill.js:7137-7151
        "tuteng1" => u05_exec(ctx, owner, "tuteng1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:tuteng1", 1)),
        // character/hearth/skill.js:7152-7163
        "tuteng2" => u05_exec(ctx, owner, "tuteng2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng2", 1)),
        // character/hearth/skill.js:7164-7179
        "tuteng3" => u05_exec(ctx, owner, "tuteng3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng3", 1)),
        // character/hearth/skill.js:7180-7195
        "tuteng4" => u05_exec(ctx, owner, "tuteng4", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng4", 1)),
        // character/hearth/skill.js:7196-7207
        "tuteng5" => u05_exec(ctx, owner, "tuteng5", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng5", 1)),
        // character/hearth/skill.js:7208-7223
        "tuteng6" => u05_exec(ctx, owner, "tuteng6", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tuteng6", 1)),
        // character/hearth/skill.js:7224-7250
        "tuteng7" => u05_exec(ctx, owner, "tuteng7", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:tuteng7", 1)),
        // character/hearth/skill.js:7251-7262
        "tuteng8" => u05_exec(ctx, owner, "tuteng8", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuteng8", 1)),
        // character/hearth/skill.js:7263-7275
        "hongxi" => u05_exec(ctx, owner, "hongxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:hongxi", 1)),
        // card/hearth/card.js:4-46
        "linghunzhihuo" => u05_exec(ctx, owner, "linghunzhihuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:linghunzhihuo", 1)),
        // card/hearth/card.js:47-69
        "jihuocard" => u05_exec(ctx, owner, "jihuocard", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:jihuocard", 1)),
        // card/hearth/card.js:70-134
        "zhaomingdan" => u05_exec(ctx, owner, "zhaomingdan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/hearth/card.js:135-209
        "shijieshu" => u05_exec(ctx, owner, "shijieshu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/hearth/card.js:210-341
        "shandianjian" => u05_exec(ctx, owner, "shandianjian", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:shandianjian", 1)),
        // card/hearth/card.js:371-398
        "dunpaigedang" => u05_exec(ctx, owner, "dunpaigedang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::DiscardOther(1)),
        // card/hearth/card.js:399-457
        "chuansongmen" => u05_exec(ctx, owner, "chuansongmen", 3, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:chuansongmen", 1)),
        // card/hearth/card.js:458-507
        "tanshezhiren" => u05_exec(ctx, owner, "tanshezhiren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tanshezhiren", 1)),
        // card/hearth/card.js:508-548
        "xingjiegoutong" => u05_exec(ctx, owner, "xingjiegoutong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // card/hearth/card.js:549-590
        "shenenshu" => u05_exec(ctx, owner, "shenenshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shenenshu", 1)),
        // card/hearth/card.js:591-637
        "zhiliaobo" => u05_exec(ctx, owner, "zhiliaobo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::JudgeEffect("u05_judge:zhiliaobo")),
        // card/hearth/card.js:638-723
        "yuansuhuimie" => u05_exec(ctx, owner, "yuansuhuimie", 0, false, 0, 0, 0, 2, DamageNature::Thunder, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/card.js:4-10
        "hsfashu_anyingjingxiang" => u05_exec(ctx, owner, "hsfashu_anyingjingxiang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsfashu_anyingjingxiang", 1)),
        // character/hearth/card.js:11-93
        "hsfashu_buwendingyibian" => u05_exec(ctx, owner, "hsfashu_buwendingyibian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hsfashu_buwendingyibian", 1)),
        // character/hearth/card.js:120-135
        "hstianqi_nazigelin" => u05_exec(ctx, owner, "hstianqi_nazigelin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:hstianqi_nazigelin", 1)),
        // character/hearth/card.js:136-154
        "hstianqi_suolasi" => u05_exec(ctx, owner, "hstianqi_suolasi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:hstianqi_suolasi", 1)),
        // character/hearth/card.js:155-183
        "hsjixie_zhadan" => u05_exec(ctx, owner, "hsjixie_zhadan", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:hsjixie_zhadan", 1)),
        // character/hearth/card.js:184-235
        "hsqizhou_feng" => u05_exec(ctx, owner, "hsqizhou_feng", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:hsqizhou_feng", 1)),
        // character/hearth/card.js:236-262
        "hsqizhou_shui" => u05_exec(ctx, owner, "hsqizhou_shui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 2, 0, U05Extra::GrantBuff("u05_state:hsqizhou_shui", 1)),
        // character/hearth/card.js:263-286
        "hsqizhou_huo" => u05_exec(ctx, owner, "hsqizhou_huo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:hsqizhou_huo", 1)),
        // character/hearth/card.js:287-308
        "hsqizhou_tu" => u05_exec(ctx, owner, "hsqizhou_tu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:hsqizhou_tu", 1)),
        // character/hearth/card.js:309-347
        "hsqingyu_feibiao" => u05_exec(ctx, owner, "hsqingyu_feibiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/card.js:348-378
        "hsqingyu_zhanfang" => u05_exec(ctx, owner, "hsqingyu_zhanfang", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/card.js:379-399
        "hsqingyu_hufu" => u05_exec(ctx, owner, "hsqingyu_hufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:hsqingyu_hufu", 1)),
        // character/hearth/card.js:400-423
        "hsqingyu_shandian" => u05_exec(ctx, owner, "hsqingyu_shandian", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:hsqingyu_shandian", 1)),
        // character/hearth/card.js:424-439
        "hsqingyu_zhao" => u05_exec(ctx, owner, "hsqingyu_zhao", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsqingyu_zhao", 1)),
        // character/hearth/card.js:440-467
        "hsdusu_xueji" => u05_exec(ctx, owner, "hsdusu_xueji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/card.js:468-502
        "hsdusu_kuyecao" => u05_exec(ctx, owner, "hsdusu_kuyecao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsdusu_kuyecao", 1)),
        // character/hearth/card.js:503-524
        "hsdusu_huangxuecao" => u05_exec(ctx, owner, "hsdusu_huangxuecao", 0, false, 0, 0, 2, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsdusu_huangxuecao", 1)),
        // character/hearth/card.js:525-549
        "hsdusu_huoyanhua" => u05_exec(ctx, owner, "hsdusu_huoyanhua", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:hsdusu_huoyanhua", 1)),
        // character/hearth/card.js:663-698
        "hsshenqi_nengliangzhiguang" => u05_exec(ctx, owner, "hsshenqi_nengliangzhiguang", 0, false, 0, 0, 3, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/card.js:699-732
        "hsshenqi_kongbusangzhong" => u05_exec(ctx, owner, "hsshenqi_kongbusangzhong", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/card.js:733-774
        "hsshenqi_morijingxiang" => u05_exec(ctx, owner, "hsshenqi_morijingxiang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/hearth/card.js:775-796
        "hsmengjing_feicuiyoulong" => u05_exec(ctx, owner, "hsmengjing_feicuiyoulong", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsmengjing_feicuiyoulong", 1)),
        // character/hearth/card.js:797-822
        "hsmengjing_suxing" => u05_exec(ctx, owner, "hsmengjing_suxing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/hearth/card.js:848-891
        "hsmengjing_mengjing" => u05_exec(ctx, owner, "hsmengjing_mengjing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hsmengjing_mengjing", 1)),
        // character/hearth/card.js:892-914
        "hsmengjing_huanxiaojiemei" => u05_exec(ctx, owner, "hsmengjing_huanxiaojiemei", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:hsmengjing_huanxiaojiemei", 1)),
        // character/hearth/card.js:915-956
        "hszuzhou_nvwudeganguo" => u05_exec(ctx, owner, "hszuzhou_nvwudeganguo", 0, false, 0, 1, 1, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/card.js:957-974
        "hszuzhou_nvwudepingguo" => u05_exec(ctx, owner, "hszuzhou_nvwudepingguo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hszuzhou_nvwudepingguo", 1)),
        // character/hearth/card.js:975-1008
        "hszuzhou_nvwudexuetu" => u05_exec(ctx, owner, "hszuzhou_nvwudexuetu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hszuzhou_nvwudexuetu", 1)),
        // character/hearth/card.js:1009-1059
        "hszuzhou_wushushike" => u05_exec(ctx, owner, "hszuzhou_wushushike", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hszuzhou_wushushike", 1)),
        // character/hearth/card.js:1060-1086
        "hszuzhou_guhuo" => u05_exec(ctx, owner, "hszuzhou_guhuo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hszuzhou_guhuo", 1)),
        _ => return false,
    }
    true
}
