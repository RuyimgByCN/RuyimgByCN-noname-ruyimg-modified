//! Authoritative `U05 杀海拾遗` (`gujian` & `gwent`) host-side skill/card resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_gujian_gwent(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // card/gujian/skill.js:5-24
        "ziyangdan" => u05_exec(ctx, owner, "ziyangdan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, -1, U05Extra::GrantBuff("u05_buff:ziyangdan", 1)),
        // card/gujian/skill.js:25-60
        "luyugeng" => u05_exec(ctx, owner, "luyugeng", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:luyugeng", 1)),
        // card/gujian/skill.js:61-97
        "xiajiao" => u05_exec(ctx, owner, "xiajiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xiajiao", 1)),
        // card/gujian/skill.js:98-104
        "xiajiao2" => u05_exec(ctx, owner, "xiajiao2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiajiao2", 1)),
        // card/gujian/skill.js:106-150
        "mizhilianou" => u05_exec(ctx, owner, "mizhilianou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("tao")),
        // card/gujian/skill.js:151-177
        "chunbing" => u05_exec(ctx, owner, "chunbing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:chunbing", 1)),
        // card/gujian/skill.js:178-224
        "gudonggeng" => u05_exec(ctx, owner, "gudonggeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gudonggeng", 1)),
        // card/gujian/skill.js:225-260
        "qingtuan" => u05_exec(ctx, owner, "qingtuan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:qingtuan", 1)),
        // card/gujian/skill.js:261-286
        "liyutang" => u05_exec(ctx, owner, "liyutang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_buff:liyutang", 1)),
        // card/gujian/skill.js:287-312
        "yougeng" => u05_exec(ctx, owner, "yougeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:yougeng", 1)),
        // card/gujian/skill.js:313-342
        "molicha" => u05_exec(ctx, owner, "molicha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:molicha", 1)),
        // card/gujian/skill.js:343-371
        "yuanbaorou" => u05_exec(ctx, owner, "yuanbaorou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuanbaorou", 1)),
        // card/gujian/skill.js:372-403
        "tanhuadong" => u05_exec(ctx, owner, "tanhuadong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:tanhuadong", 1)),
        // card/gujian/skill.js:404-438
        "mapodoufu" => u05_exec(ctx, owner, "mapodoufu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gujian/skill.js:439-455
        "yunvyuanshen_skill" => u05_exec(ctx, owner, "yunvyuanshen_skill", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:yunvyuanshen_skill", 1)),
        // card/gujian/skill.js:456-467
        "heilonglinpian" => u05_exec(ctx, owner, "heilonglinpian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:heilonglinpian", 1)),
        // card/gujian/skill.js:468-497
        "mutoumianju_skill" => u05_exec(ctx, owner, "mutoumianju_skill", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // card/gujian/skill.js:498-552
        "gjyuheng_skill" => u05_exec(ctx, owner, "gjyuheng_skill", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gjyuheng_skill", 1)),
        // card/gujian/skill.js:554-608
        "gjyuheng_plus_skill" => u05_exec(ctx, owner, "gjyuheng_plus_skill", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gjyuheng_plus_skill", 1)),
        // card/gujian/skill.js:609-662
        "gjyuheng_pro_skill" => u05_exec(ctx, owner, "gjyuheng_pro_skill", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gjyuheng_pro_skill", 1)),
        // card/gujian/skill.js:663-675
        "shihuifen" => u05_exec(ctx, owner, "shihuifen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:shihuifen", 1)),
        // card/gujian/skill.js:676-704
        "g_shihuifen" => u05_exec(ctx, owner, "g_shihuifen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_shihuifen", 1)),
        // card/gujian/skill.js:705-727
        "g_jinlianzhu" => u05_exec(ctx, owner, "g_jinlianzhu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_jinlianzhu", 1)),
        // character/gujian/skill.js:5-91
        "qingshu" => u05_exec(ctx, owner, "qingshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:qingshu")),
        // character/gujian/skill.js:92-129
        "yunyou" => u05_exec(ctx, owner, "yunyou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yunyou", 1)),
        // character/gujian/skill.js:130-228
        "xuanzhen" => u05_exec(ctx, owner, "xuanzhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xuanzhen", 1)),
        // character/gujian/skill.js:229-267
        "xuanci" => u05_exec(ctx, owner, "xuanci", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("feibiao")),
        // character/gujian/skill.js:268-314
        "xianju" => u05_exec(ctx, owner, "xianju", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xianju", 1)),
        // character/gujian/skill.js:315-397
        "humeng" => u05_exec(ctx, owner, "humeng", 0, true, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:398-416
        "xunjian" => u05_exec(ctx, owner, "xunjian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xunjian", 1)),
        // character/gujian/skill.js:417-480
        "lingyan" => u05_exec(ctx, owner, "lingyan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lingyan", 1)),
        // character/gujian/skill.js:481-501
        "woxue" => u05_exec(ctx, owner, "woxue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:woxue", 1)),
        // character/gujian/skill.js:502-602
        "yange" => u05_exec(ctx, owner, "yange", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yange", 1)),
        // character/gujian/skill.js:603-716
        "lianjing" => u05_exec(ctx, owner, "lianjing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lianjing", 1)),
        // character/gujian/skill.js:717-802
        "cihong" => u05_exec(ctx, owner, "cihong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/gujian/skill.js:803-869
        "zhenying" => u05_exec(ctx, owner, "zhenying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zhenying", 1)),
        // character/gujian/skill.js:870-976
        "lingnu" => u05_exec(ctx, owner, "lingnu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:lingnu", 1)),
        // character/gujian/skill.js:977-994
        "zuiji" => u05_exec(ctx, owner, "zuiji", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("jiu")),
        // character/gujian/skill.js:995-1010
        "manwu" => u05_exec(ctx, owner, "manwu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:manwu", 1)),
        // character/gujian/skill.js:1011-1030
        "xfanghua" => u05_exec(ctx, owner, "xfanghua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xfanghua", 1)),
        // character/gujian/skill.js:1031-1091
        "duhun" => u05_exec(ctx, owner, "duhun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/gujian/skill.js:1092-1202
        "yunyin" => u05_exec(ctx, owner, "yunyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yunyin", 1)),
        // character/gujian/skill.js:1203-1236
        "shishui" => u05_exec(ctx, owner, "shishui", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/gujian/skill.js:1237-1277
        "chizhen" => u05_exec(ctx, owner, "chizhen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:chizhen", 1)),
        // character/gujian/skill.js:1278-1311
        "dangping" => u05_exec(ctx, owner, "dangping", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:dangping", 1)),
        // character/gujian/skill.js:1313-1345
        "xiuhua" => u05_exec(ctx, owner, "xiuhua", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xiuhua", 1)),
        // character/gujian/skill.js:1346-1403
        "liuying" => u05_exec(ctx, owner, "liuying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:liuying", 1)),
        // character/gujian/skill.js:1404-1555
        "yanjia" => u05_exec(ctx, owner, "yanjia", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanjia", 1)),
        // character/gujian/skill.js:1556-1589
        "meiying" => u05_exec(ctx, owner, "meiying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:meiying", 1)),
        // character/gujian/skill.js:1590-1599
        "jianwu" => u05_exec(ctx, owner, "jianwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jianwu", 1)),
        // character/gujian/skill.js:1600-1623
        "zuizhan" => u05_exec(ctx, owner, "zuizhan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:zuizhan", 1)),
        // character/gujian/skill.js:1624-1643
        "xidie" => u05_exec(ctx, owner, "xidie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xidie", 1)),
        // character/gujian/skill.js:1644-1654
        "xidie2" => u05_exec(ctx, owner, "xidie2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xidie2", 1)),
        // character/gujian/skill.js:1655-1688
        "meihu" => u05_exec(ctx, owner, "meihu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:meihu", 1)),
        // character/gujian/skill.js:1689-1716
        "xlqianhuan" => u05_exec(ctx, owner, "xlqianhuan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:1717-1752
        "fumo" => u05_exec(ctx, owner, "fumo", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:fumo", 1)),
        // character/gujian/skill.js:1753-1823
        "fanyin" => u05_exec(ctx, owner, "fanyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:1824-1851
        "mingkong" => u05_exec(ctx, owner, "mingkong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mingkong", 1)),
        // character/gujian/skill.js:1852-1866
        "mingkong2" => u05_exec(ctx, owner, "mingkong2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mingkong2", 1)),
        // character/gujian/skill.js:1867-1875
        "gjqtwuxie" => u05_exec(ctx, owner, "gjqtwuxie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gjqtwuxie", 1)),
        // character/gujian/skill.js:1876-1898
        "yuehua" => u05_exec(ctx, owner, "yuehua", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuehua", 1)),
        // character/gujian/skill.js:1899-1947
        "qinglan" => u05_exec(ctx, owner, "qinglan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:qinglan", 1)),
        // character/gujian/skill.js:1948-1963
        "fanshi" => u05_exec(ctx, owner, "fanshi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/gujian/skill.js:1964-2031
        "xuelu" => u05_exec(ctx, owner, "xuelu", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:xuelu", 1)),
        // character/gujian/skill.js:2032-2081
        "shahun" => u05_exec(ctx, owner, "shahun", 0, true, 0, 3, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/gujian/skill.js:2082-2096
        "shahun2" => u05_exec(ctx, owner, "shahun2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:shahun2", 1)),
        // character/gujian/skill.js:2097-2105
        "xuanning" => {
            u05_mark_invoked(ctx, owner, "xuanning");
            set_flag(ctx.core, owner, "xianjian_skill:xuanning", 1);
            set_flag(ctx.core, owner, "xianjian_ward_active", 1);
            let _ = u05_store_pile(ctx, owner, "xuanning");
            u05_recover(ctx, owner, 1);
        }
        // character/gujian/skill.js:2106-2144
        "xuanning1" => u05_exec(ctx, owner, "xuanning1", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xuanning1", 1)),
        // character/gujian/skill.js:2145-2169
        "xuanning2" => u05_exec(ctx, owner, "xuanning2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xuanning2", 1)),
        // character/gujian/skill.js:2170-2234
        "liuguang" => u05_exec(ctx, owner, "liuguang", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gujian/skill.js:2235-2266
        "yangming" => u05_exec(ctx, owner, "yangming", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yangming", 1)),
        // character/gujian/skill.js:2267-2305
        "yangming2" => u05_exec(ctx, owner, "yangming2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:yangming2", 1)),
        // character/gujian/skill.js:2306-2344
        "zhaolu" => u05_exec(ctx, owner, "zhaolu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/gujian/skill.js:2345-2379
        "jiehuo" => u05_exec(ctx, owner, "jiehuo", 0, true, 0, 0, 0, 2, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:jiehuo", 1)),
        // character/gujian/skill.js:2380-2387
        "jiehuo2" => u05_exec(ctx, owner, "jiehuo2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jiehuo2", 1)),
        // character/gujian/skill.js:2388-2418
        "yuling" => u05_exec(ctx, owner, "yuling", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuling", 1)),
        // character/gujian/skill.js:2419-2430
        "yuling1" => u05_exec(ctx, owner, "yuling1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuling1", 1)),
        // character/gujian/skill.js:2431-2444
        "yuling2" => u05_exec(ctx, owner, "yuling2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuling2", 1)),
        // character/gujian/skill.js:2445-2458
        "yuling3" => u05_exec(ctx, owner, "yuling3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yuling3", 1)),
        // character/gujian/skill.js:2459-2471
        "yuling4" => u05_exec(ctx, owner, "yuling4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yuling4", 1)),
        // character/gujian/skill.js:2472-2478
        "yuling5" => u05_exec(ctx, owner, "yuling5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yuling5", 1)),
        // character/gujian/skill.js:2479-2485
        "yuling6" => u05_exec(ctx, owner, "yuling6", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:yuling6", 1)),
        // card/gujian/card.js:26-56
        "jinlianzhu" => u05_exec(ctx, owner, "jinlianzhu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jinlianzhu", 1)),
        // card/gujian/card.js:406-417
        "mutoumianju" => u05_exec(ctx, owner, "mutoumianju", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:mutoumianju", 1)),
        // card/gujian/card.js:419-434
        "gjyuheng" => u05_exec(ctx, owner, "gjyuheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gjyuheng", 1)),
        // card/gujian/card.js:435-451
        "gjyuheng_plus" => u05_exec(ctx, owner, "gjyuheng_plus", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gjyuheng_plus", 1)),
        // card/gujian/card.js:452-465
        "gjyuheng_pro" => u05_exec(ctx, owner, "gjyuheng_pro", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gjyuheng_pro", 1)),
        // card/gujian/card.js:466-503
        "shatang" => u05_exec(ctx, owner, "shatang", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 1, U05Extra::GrantBuff("u05_state:shatang", 1)),
        // card/gujian/card.js:504-550
        "shujinsan" => u05_exec(ctx, owner, "shujinsan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gujian/card.js:551-584
        "dinvxuanshuang" => u05_exec(ctx, owner, "dinvxuanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::DiscardOther(1)),
        // card/gujian/card.js:621-655
        "yunvyuanshen" => u05_exec(ctx, owner, "yunvyuanshen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:yunvyuanshen", 1)),
        // card/gujian/card.js:656-780
        "bingpotong" => u05_exec(ctx, owner, "bingpotong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/gujian/card.js:781-819
        "feibiao" => u05_exec(ctx, owner, "feibiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/gujian/card.js:820-869
        "longxugou" => u05_exec(ctx, owner, "longxugou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:longxugou", 1)),
        // card/gujian/card.js:870-905
        "qiankunbiao" => u05_exec(ctx, owner, "qiankunbiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gujian/card.js:906-991
        "shenhuofeiya" => u05_exec(ctx, owner, "shenhuofeiya", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:shenhuofeiya", 1)),
        // card/gujian/card.js:992-1016
        "mianlijinzhen" => u05_exec(ctx, owner, "mianlijinzhen", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/gujian/card.js:1017-1042
        "liutouge" => u05_exec(ctx, owner, "liutouge", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:liutouge", 1)),
        // card/gujian/card.js:1043-1091
        "liufengsan" => u05_exec(ctx, owner, "liufengsan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:liufengsan", 1)),
        // character/gujian/card.js:4-67
        "yanjiadan_heart" => u05_exec(ctx, owner, "yanjiadan_heart", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanjiadan_heart", 1)),
        // character/gujian/card.js:68-116
        "yanjiadan_diamond" => u05_exec(ctx, owner, "yanjiadan_diamond", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanjiadan_diamond", 1)),
        // character/gujian/card.js:117-148
        "yanjiadan_club" => u05_exec(ctx, owner, "yanjiadan_club", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanjiadan_club", 1)),
        // character/gujian/card.js:149-198
        "yanjiadan_spade" => u05_exec(ctx, owner, "yanjiadan_spade", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yanjiadan_spade", 1)),
        // card/gwent/skill.js:5-35
        "gw_aerdeyin" => u05_exec(ctx, owner, "gw_aerdeyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // card/gwent/skill.js:36-79
        "gw_kunenfayin" => u05_exec(ctx, owner, "gw_kunenfayin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gw_kunenfayin", 1)),
        // card/gwent/skill.js:80-122
        "gw_baobaoshu" => u05_exec(ctx, owner, "gw_baobaoshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_baobaoshu", 1)),
        // card/gwent/skill.js:123-148
        "gw_nuhaifengbao" => u05_exec(ctx, owner, "gw_nuhaifengbao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gw_nuhaifengbao", 1)),
        // card/gwent/skill.js:149-172
        "gw_youer" => u05_exec(ctx, owner, "gw_youer", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gw_youer", 1)),
        // card/gwent/skill.js:173-197
        "gw_qinpendayu" => u05_exec(ctx, owner, "gw_qinpendayu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gw_qinpendayu", 1)),
        // card/gwent/skill.js:198-224
        "gw_birinongwu" => u05_exec(ctx, owner, "gw_birinongwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gw_birinongwu", 1)),
        // card/gwent/skill.js:225-243
        "gw_ciguhanshuang" => u05_exec(ctx, owner, "gw_ciguhanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_ciguhanshuang", 1)),
        // card/gwent/skill.js:244-269
        "gw_dieyi" => u05_exec(ctx, owner, "gw_dieyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gw_dieyi", 1)),
        // card/gwent/skill.js:270-335
        "gw_leizhoushu" => u05_exec(ctx, owner, "gw_leizhoushu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gw_leizhoushu", 1)),
        // card/gwent/skill.js:336-389
        "_gainspell" => u05_exec(ctx, owner, "_gainspell", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:_gainspell", 1)),
        // character/gwent/skill.js:5-30
        "sqlongyin" => u05_exec(ctx, owner, "sqlongyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:sqlongyin", 1)),
        // character/gwent/skill.js:31-73
        "sqlongnu" => u05_exec(ctx, owner, "sqlongnu", 0, false, 0, 1, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:sqlongnu", 1)),
        // character/gwent/skill.js:74-119
        "sqlonghuo" => u05_exec(ctx, owner, "sqlonghuo", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:sqlonghuo", 1)),
        // character/gwent/skill.js:120-148
        "sqlongwu" => u05_exec(ctx, owner, "sqlongwu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:sqlongwu", 1)),
        // character/gwent/skill.js:149-210
        "jielue" => u05_exec(ctx, owner, "jielue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jielue", 1)),
        // character/gwent/skill.js:211-222
        "gwmaoxian_hengsaite_sha" => u05_exec(ctx, owner, "gwmaoxian_hengsaite_sha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_hengsaite_sha", 1)),
        // character/gwent/skill.js:223-340
        "gwhuanshuang" => u05_exec(ctx, owner, "gwhuanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwhuanshuang", 1)),
        // character/gwent/skill.js:341-367
        "gw_xianzumaijiu" => u05_exec(ctx, owner, "gw_xianzumaijiu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_xianzumaijiu", 1)),
        // character/gwent/skill.js:368-416
        "gwjinli" => u05_exec(ctx, owner, "gwjinli", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwjinli", 1)),
        // character/gwent/skill.js:417-449
        "gwliaotian" => u05_exec(ctx, owner, "gwliaotian", 2, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwliaotian", 1)),
        // character/gwent/skill.js:450-499
        "gwhuanbi" => u05_exec(ctx, owner, "gwhuanbi", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwhuanbi", 1)),
        // character/gwent/skill.js:500-575
        "gwminxiang" => u05_exec(ctx, owner, "gwminxiang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwminxiang", 1)),
        // character/gwent/skill.js:576-605
        "gwlangshi" => u05_exec(ctx, owner, "gwlangshi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwlangshi", 1)),
        // character/gwent/skill.js:606-703
        "gwjingtian" => u05_exec(ctx, owner, "gwjingtian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwjingtian", 1)),
        // character/gwent/skill.js:705-793
        "gwjingshi" => u05_exec(ctx, owner, "gwjingshi", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwjingshi", 1)),
        // character/gwent/skill.js:794-861
        "gwweitu" => u05_exec(ctx, owner, "gwweitu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_buff:gwweitu", 1)),
        // character/gwent/skill.js:862-962
        "gwgouhun" => u05_exec(ctx, owner, "gwgouhun", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:963-1042
        "gwfutian" => u05_exec(ctx, owner, "gwfutian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:1043-1061
        "gwzhongmo" => u05_exec(ctx, owner, "gwzhongmo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwzhongmo", 1)),
        // character/gwent/skill.js:1062-1123
        "gwyewu" => u05_exec(ctx, owner, "gwyewu", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwyewu", 1)),
        // character/gwent/skill.js:1124-1133
        "g_gw_yewu" => u05_exec(ctx, owner, "g_gw_yewu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:g_gw_yewu", 1)),
        // character/gwent/skill.js:1134-1180
        "shuangxi" => u05_exec(ctx, owner, "shuangxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shuangxi", 1)),
        // character/gwent/skill.js:1181-1200
        "yangfan" => u05_exec(ctx, owner, "yangfan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yangfan", 1)),
        // character/gwent/skill.js:1201-1242
        "gwfengshi" => u05_exec(ctx, owner, "gwfengshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwfengshi", 1)),
        // character/gwent/skill.js:1243-1320
        "gwchenshui" => u05_exec(ctx, owner, "gwchenshui", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwchenshui", 1)),
        // character/gwent/skill.js:1321-1362
        "gwliedi" => u05_exec(ctx, owner, "gwliedi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwliedi", 1)),
        // character/gwent/skill.js:1363-1381
        "julian" => u05_exec(ctx, owner, "julian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:julian", 1)),
        // character/gwent/skill.js:1382-1413
        "gwfusheng" => u05_exec(ctx, owner, "gwfusheng", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverOther),
        // character/gwent/skill.js:1414-1454
        "gwqinwu" => u05_exec(ctx, owner, "gwqinwu", 1, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwqinwu", 1)),
        // character/gwent/skill.js:1455-1460
        "gwqinwu2" => u05_exec(ctx, owner, "gwqinwu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwqinwu2", 1)),
        // character/gwent/skill.js:1461-1484
        "huanshu" => u05_exec(ctx, owner, "huanshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huanshu", 1)),
        // character/gwent/skill.js:1485-1520
        "huanshu2" => u05_exec(ctx, owner, "huanshu2", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huanshu2", 1)),
        // character/gwent/skill.js:1521-1529
        "huanshu3" => u05_exec(ctx, owner, "huanshu3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huanshu3", 1)),
        // character/gwent/skill.js:1530-1635
        "gwjieyin" => u05_exec(ctx, owner, "gwjieyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwjieyin", 1)),
        // character/gwent/skill.js:1636-1700
        "zhengjun" => u05_exec(ctx, owner, "zhengjun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/gwent/skill.js:1701-1710
        "gwxuezhan" => u05_exec(ctx, owner, "gwxuezhan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwxuezhan", 1)),
        // character/gwent/skill.js:1711-1774
        "bolang" => u05_exec(ctx, owner, "bolang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:bolang", 1)),
        // character/gwent/skill.js:1775-1802
        "gwjushi" => u05_exec(ctx, owner, "gwjushi", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwjushi", 1)),
        // character/gwent/skill.js:1803-1834
        "gwjushi2" => u05_exec(ctx, owner, "gwjushi2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwjushi2", 1)),
        // character/gwent/skill.js:1835-1894
        "gwfengchi" => u05_exec(ctx, owner, "gwfengchi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwfengchi", 1)),
        // character/gwent/skill.js:1895-1940
        "lingji" => u05_exec(ctx, owner, "lingji", 1, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lingji", 1)),
        // character/gwent/skill.js:1941-1990
        "gwjinyan" => u05_exec(ctx, owner, "gwjinyan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwjinyan", 1)),
        // character/gwent/skill.js:1991-2091
        "gwshenyu" => u05_exec(ctx, owner, "gwshenyu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:gwshenyu", 1)),
        // character/gwent/skill.js:2092-2130
        "junchi" => u05_exec(ctx, owner, "junchi", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:junchi", 1)),
        // character/gwent/skill.js:2131-2196
        "hupeng" => u05_exec(ctx, owner, "hupeng", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hupeng", 1)),
        // character/gwent/skill.js:2197-2234
        "hupeng2" => u05_exec(ctx, owner, "hupeng2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hupeng2", 1)),
        // character/gwent/skill.js:2235-2250
        "hupeng3" => u05_exec(ctx, owner, "hupeng3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hupeng3", 1)),
        // character/gwent/skill.js:2251-2274
        "hupeng4" => u05_exec(ctx, owner, "hupeng4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hupeng4", 1)),
        // character/gwent/skill.js:2275-2353
        "hunmo" => u05_exec(ctx, owner, "hunmo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/gwent/skill.js:2355-2392
        "shuijian" => u05_exec(ctx, owner, "shuijian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shuijian", 1)),
        // character/gwent/skill.js:2393-2417
        "yunhuo" => u05_exec(ctx, owner, "yunhuo", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:yunhuo", 1)),
        // character/gwent/skill.js:2418-2445
        "yinzhang" => u05_exec(ctx, owner, "yinzhang", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yinzhang", 1)),
        // character/gwent/skill.js:2446-2516
        "gwtianbian" => u05_exec(ctx, owner, "gwtianbian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwtianbian", 1)),
        // character/gwent/skill.js:2517-2539
        "gwxiaoshou" => u05_exec(ctx, owner, "gwxiaoshou", 2, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwxiaoshou", 1)),
        // character/gwent/skill.js:2540-2556
        "kuanglie" => u05_exec(ctx, owner, "kuanglie", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kuanglie", 1)),
        // character/gwent/skill.js:2558-2612
        "gwjiquan" => u05_exec(ctx, owner, "gwjiquan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/gwent/skill.js:2613-2660
        "nuhou" => u05_exec(ctx, owner, "nuhou", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:nuhou", 1)),
        // character/gwent/skill.js:2661-2711
        "shewu" => u05_exec(ctx, owner, "shewu", 1, false, 1, 3, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:shewu", 1)),
        // character/gwent/skill.js:2712-2771
        "gwzhanjiang" => u05_exec(ctx, owner, "gwzhanjiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwzhanjiang", 1)),
        // character/gwent/skill.js:2773-2783
        "gwzhanjiang3" => u05_exec(ctx, owner, "gwzhanjiang3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwzhanjiang3", 1)),
        // character/gwent/skill.js:2784-2809
        "gwchuanxin" => u05_exec(ctx, owner, "gwchuanxin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:gwchuanxin", 1)),
        // character/gwent/skill.js:2810-2844
        "fengjian" => u05_exec(ctx, owner, "fengjian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:fengjian", 1)),
        // character/gwent/skill.js:2845-2901
        "huandie" => u05_exec(ctx, owner, "huandie", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huandie", 1)),
        // character/gwent/skill.js:2902-2945
        "xuezhou" => u05_exec(ctx, owner, "xuezhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:xuezhou", 1)),
        // character/gwent/skill.js:2946-2982
        "xuezhou_hp" => u05_exec(ctx, owner, "xuezhou_hp", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)),
        // character/gwent/skill.js:2984-3029
        "fayin" => u05_exec(ctx, owner, "fayin", 0, false, 0, 0, 1, 1, DamageNature::Fire, 0, 1, U05Extra::TurnOverOther),
        // character/gwent/skill.js:3030-3087
        "gwbaquan" => u05_exec(ctx, owner, "gwbaquan", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwbaquan", 1)),
        // character/gwent/skill.js:3088-3144
        "huihun" => u05_exec(ctx, owner, "huihun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huihun", 1)),
        // character/gwent/skill.js:3145-3197
        "lanquan" => u05_exec(ctx, owner, "lanquan", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:lanquan", 1)),
        // card/gwent/card.js:7-23
        "gw_dieyi_equip1" => u05_exec(ctx, owner, "gw_dieyi_equip1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_equip1", 1)),
        // card/gwent/card.js:24-39
        "gw_dieyi_equip2" => u05_exec(ctx, owner, "gw_dieyi_equip2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_equip2", 1)),
        // card/gwent/card.js:40-55
        "gw_dieyi_equip3" => u05_exec(ctx, owner, "gw_dieyi_equip3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_equip3", 1)),
        // card/gwent/card.js:56-71
        "gw_dieyi_equip4" => u05_exec(ctx, owner, "gw_dieyi_equip4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_equip4", 1)),
        // card/gwent/card.js:72-87
        "gw_dieyi_equip5" => u05_exec(ctx, owner, "gw_dieyi_equip5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_equip5", 1)),
        // card/gwent/card.js:88-99
        "gw_dieyi_judge" => u05_exec(ctx, owner, "gw_dieyi_judge", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dieyi_judge", 1)),
        // card/gwent/card.js:100-157
        "gw_hudiewu" => u05_exec(ctx, owner, "gw_hudiewu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_hudiewu", 1)),
        // card/gwent/card.js:158-244
        "gw_yigeniyin" => u05_exec(ctx, owner, "gw_yigeniyin", 0, false, 0, 0, 0, -1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:gw_yigeniyin", 1)),
        // card/gwent/card.js:356-397
        "gw_ansha" => u05_exec(ctx, owner, "gw_ansha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_ansha", 1)),
        // card/gwent/card.js:398-513
        "gw_xinsheng" => u05_exec(ctx, owner, "gw_xinsheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_xinsheng", 1)),
        // card/gwent/card.js:514-608
        "gw_niuquzhijing" => u05_exec(ctx, owner, "gw_niuquzhijing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_niuquzhijing", 1)),
        // card/gwent/card.js:609-660
        "gw_zhongmozhizhan" => u05_exec(ctx, owner, "gw_zhongmozhizhan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_zhongmozhizhan", 1)),
        // card/gwent/card.js:661-736
        "gw_ganhan" => u05_exec(ctx, owner, "gw_ganhan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHpOther(1)),
        // card/gwent/card.js:737-778
        "gw_huangjiashenpan" => u05_exec(ctx, owner, "gw_huangjiashenpan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_huangjiashenpan", 1)),
        // card/gwent/card.js:779-843
        "gw_tunshi" => u05_exec(ctx, owner, "gw_tunshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)),
        // card/gwent/card.js:844-929
        "gw_chongci" => u05_exec(ctx, owner, "gw_chongci", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_chongci", 1)),
        // card/gwent/card.js:968-1000
        "gw_tongdi" => u05_exec(ctx, owner, "gw_tongdi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // card/gwent/card.js:1001-1026
        "gw_fuyuan" => u05_exec(ctx, owner, "gw_fuyuan", 0, false, 0, 0, 1, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:gw_fuyuan", 1)),
        // card/gwent/card.js:1027-1056
        "gw_zhuoshao" => u05_exec(ctx, owner, "gw_zhuoshao", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:gw_zhuoshao", 1)),
        // card/gwent/card.js:1057-1136
        "gw_butianshu" => u05_exec(ctx, owner, "gw_butianshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_butianshu", 1)),
        // card/gwent/card.js:1137-1188
        "gw_shizizhaohuan" => u05_exec(ctx, owner, "gw_shizizhaohuan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_shizizhaohuan", 1)),
        // card/gwent/card.js:1189-1227
        "gw_zuihouyuanwang" => u05_exec(ctx, owner, "gw_zuihouyuanwang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_zuihouyuanwang", 1)),
        // card/gwent/card.js:1228-1344
        "gw_zirankuizeng" => u05_exec(ctx, owner, "gw_zirankuizeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_zirankuizeng", 1)),
        // card/gwent/card.js:1367-1387
        "gw_baishuang" => u05_exec(ctx, owner, "gw_baishuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gw_baishuang", 1)),
        // card/gwent/card.js:1411-1455
        "gw_guaiwuchaoxue" => u05_exec(ctx, owner, "gw_guaiwuchaoxue", 1, false, 0, 0, 0, 1, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:gw_guaiwuchaoxue", 1)),
        // card/gwent/card.js:1546-1584
        "gw_baoxueyaoshui" => u05_exec(ctx, owner, "gw_baoxueyaoshui", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gwent/card.js:1585-1615
        "gw_zhihuanjun" => u05_exec(ctx, owner, "gw_zhihuanjun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHpOther(1)),
        // card/gwent/card.js:1616-1647
        "gw_zumoshoukao" => u05_exec(ctx, owner, "gw_zumoshoukao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, -1, U05Extra::GrantOtherBuff("u05_debuff:gw_zumoshoukao", 1)),
        // card/gwent/card.js:1648-1680
        "gw_aozuzhilei" => u05_exec(ctx, owner, "gw_aozuzhilei", 0, false, 0, 0, 1, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:gw_aozuzhilei", 1)),
        // card/gwent/card.js:1681-1757
        "gw_poxiao" => u05_exec(ctx, owner, "gw_poxiao", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // card/gwent/card.js:1785-1813
        "gw_yanziyaoshui" => u05_exec(ctx, owner, "gw_yanziyaoshui", 0, false, 0, 0, 2, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_yanziyaoshui", 1)),
        // card/gwent/card.js:1814-1849
        "gw_wenyi" => u05_exec(ctx, owner, "gw_wenyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // card/gwent/card.js:1850-1906
        "gw_shanbengshu" => u05_exec(ctx, owner, "gw_shanbengshu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_shanbengshu", 1)),
        // character/gwent/card.js:4-8
        "gwjinli_jiu" => u05_exec(ctx, owner, "gwjinli_jiu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwjinli_jiu", 1)),
        // character/gwent/card.js:119-157
        "gwmaoxian_yioufeisi" => u05_exec(ctx, owner, "gwmaoxian_yioufeisi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_yioufeisi", 1)),
        // character/gwent/card.js:158-200
        "gwmaoxian_luoqi" => u05_exec(ctx, owner, "gwmaoxian_luoqi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_luoqi", 1)),
        // character/gwent/card.js:201-260
        "gwmaoxian_jieluote" => u05_exec(ctx, owner, "gwmaoxian_jieluote", 0, false, 0, 0, 0, 2, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_jieluote", 1)),
        // character/gwent/card.js:261-293
        "gwmaoxian_yenaifa" => u05_exec(ctx, owner, "gwmaoxian_yenaifa", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_yenaifa", 1)),
        // character/gwent/card.js:294-328
        "gwmaoxian_telisi" => u05_exec(ctx, owner, "gwmaoxian_telisi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_telisi", 1)),
        // character/gwent/card.js:329-370
        "gwmaoxian_hengsaite" => u05_exec(ctx, owner, "gwmaoxian_hengsaite", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:gwmaoxian_hengsaite", 1)),
        // character/gwent/card.js:371-405
        "gwmaoxian_fuertaisite" => u05_exec(ctx, owner, "gwmaoxian_fuertaisite", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:gwmaoxian_fuertaisite", 1)),
        // character/gwent/card.js:406-437
        "gwmaoxian_laduoweide" => u05_exec(ctx, owner, "gwmaoxian_laduoweide", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwmaoxian_laduoweide", 1)),
        // character/gwent/card.js:438-473
        "gwmaoxian_enxier" => u05_exec(ctx, owner, "gwmaoxian_enxier", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_enxier", 1)),
        // character/gwent/card.js:474-525
        "gwmaoxian_fulisi" => u05_exec(ctx, owner, "gwmaoxian_fulisi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_fulisi", 1)),
        // character/gwent/card.js:526-560
        "gwmaoxian_kaerweite" => u05_exec(ctx, owner, "gwmaoxian_kaerweite", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_kaerweite", 1)),
        // character/gwent/card.js:561-602
        "gwmaoxian_bulanwang" => u05_exec(ctx, owner, "gwmaoxian_bulanwang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_bulanwang", 1)),
        // character/gwent/card.js:603-636
        "gwmaoxian_kuite" => u05_exec(ctx, owner, "gwmaoxian_kuite", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_kuite", 1)),
        // character/gwent/card.js:637-667
        "gwmaoxian_haluo" => u05_exec(ctx, owner, "gwmaoxian_haluo", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_haluo", 1)),
        // character/gwent/card.js:668-703
        "gwmaoxian_dagong" => u05_exec(ctx, owner, "gwmaoxian_dagong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwmaoxian_dagong", 1)),
        // character/gwent/card.js:704-750
        "gwmaoxian_gaier" => u05_exec(ctx, owner, "gwmaoxian_gaier", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpOther(1)),
        // character/gwent/card.js:751-795
        "gwmaoxian_airuiting" => u05_exec(ctx, owner, "gwmaoxian_airuiting", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/gwent/card.js:796-832
        "gwmaoxian_aisinie" => u05_exec(ctx, owner, "gwmaoxian_aisinie", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_aisinie", 1)),
        // character/gwent/card.js:833-874
        "gwmaoxian_falanxisika" => u05_exec(ctx, owner, "gwmaoxian_falanxisika", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_falanxisika", 1)),
        // character/gwent/card.js:875-924
        "gwmaoxian_huoge" => u05_exec(ctx, owner, "gwmaoxian_huoge", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gwmaoxian_huoge", 1)),
        // character/gwent/card.js:925-937
        "gw_wuyao" => u05_exec(ctx, owner, "gw_wuyao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_wuyao", 1)),
        // character/gwent/card.js:938-950
        "gw_lang" => u05_exec(ctx, owner, "gw_lang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_lang", 1)),
        // character/gwent/card.js:951-974
        "gw_dudayuanshuai1" => u05_exec(ctx, owner, "gw_dudayuanshuai1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dudayuanshuai1", 1)),
        // character/gwent/card.js:975-984
        "gw_dudayuanshuai2" => u05_exec(ctx, owner, "gw_dudayuanshuai2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:gw_dudayuanshuai2", 1)),
        _ => return false,
    }
    true
}
