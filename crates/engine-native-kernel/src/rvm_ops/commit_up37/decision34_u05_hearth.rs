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
        "hshuanyu" => u05_exec(ctx, owner, "hshuanyu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:57-130
        "wxuying" => u05_exec(ctx, owner, "wxuying", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:145-154
        "yibian" => u05_exec(ctx, owner, "yibian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:481-561
        "asyouzhang" => u05_exec(ctx, owner, "asyouzhang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:562-678
        "ylyuchu" => u05_exec(ctx, owner, "ylyuchu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:ylyuchu")),
        // character/hearth/skill.js:679-709
        "nsaiqi" => u05_exec(ctx, owner, "nsaiqi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:nsaiqi", 1)),
        // character/hearth/skill.js:710-729
        "nsbeiming" => u05_exec(ctx, owner, "nsbeiming", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:730-902
        "hsnitai" => u05_exec(ctx, owner, "hsnitai", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:hsnitai", 1)),
        // character/hearth/skill.js:903-938
        "hspuzhao" => u05_exec(ctx, owner, "hspuzhao", 1, false, 1, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:939-973
        "hsyanxin" => u05_exec(ctx, owner, "hsyanxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:974-1106
        "hstianqi" => u05_exec(ctx, owner, "hstianqi", 1, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1107-1116
        "hstianqi_dalian" => u05_exec(ctx, owner, "hstianqi_dalian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:1117-1126
        "hstianqi_shali" => { u05_mark_invoked(ctx, owner, "hstianqi_shali"); u05_change_hujia(ctx, owner, 1); }
        // character/hearth/skill.js:1127-1279
        "ysjqisha" => u05_exec(ctx, owner, "ysjqisha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:1280-1320
        "yindan" => u05_exec(ctx, owner, "yindan", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:1321-1391
        "hllingxi" => u05_exec(ctx, owner, "hllingxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:1424-1453
        "xiyong" => u05_exec(ctx, owner, "xiyong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1454-1466
        "srjici" => u05_exec(ctx, owner, "srjici", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1467-1481
        "yinzong" => u05_exec(ctx, owner, "yinzong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1482-1519
        "tansuo" => u05_exec(ctx, owner, "tansuo", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1520-1591
        "lieqi" => u05_exec(ctx, owner, "lieqi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1677-1704
        "shouwang" => u05_exec(ctx, owner, "shouwang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::GrantOtherBuff("u05_debuff:shouwang", 1)),
        // character/hearth/skill.js:1747-1814
        "qianfu2" => u05_exec(ctx, owner, "qianfu2", 0, false, 0, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qianfu2", 1)),
        // character/hearth/skill.js:1815-1828
        "shimo" => u05_exec(ctx, owner, "shimo", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:1829-1846
        "lieyang" => u05_exec(ctx, owner, "lieyang", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1847-1855
        "lieyang2" => u05_exec(ctx, owner, "lieyang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:lieyang2", 1)),
        // character/hearth/skill.js:1856-1892
        "zhuilie" => u05_exec(ctx, owner, "zhuilie", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1927-1963
        "kekao" => u05_exec(ctx, owner, "kekao", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2028-2082
        "kqizhou" => u05_exec(ctx, owner, "kqizhou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2083-2109
        "jingcu" => u05_exec(ctx, owner, "jingcu", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:2110-2119
        "shengzhang" => u05_exec(ctx, owner, "shengzhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:2120-2164
        "pyuhuo" => u05_exec(ctx, owner, "pyuhuo", 0, false, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:2165-2209
        "mengye" => u05_exec(ctx, owner, "mengye", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:2251-2265
        "lianzhan" => u05_exec(ctx, owner, "lianzhan", 0, false, 0, 2, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:2412-2448
        "moxie" => u05_exec(ctx, owner, "moxie", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2514-2522
        "hlongyi" => u05_exec(ctx, owner, "hlongyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hlongyi", 1)),
        // character/hearth/skill.js:2523-2549
        "zhongji" => {
            if u05_pay_discard(ctx, owner, "zhongji", 1, false) {
                u05_mark_invoked(ctx, owner, "zhongji");
                set_flag(ctx.core, owner, "u05_mod:zhongji_dmg", 1);
            }
        }
        // character/hearth/skill.js:2550-2566
        "fuwen" => u05_exec(ctx, owner, "fuwen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:2604-2629
        "midian" => u05_exec(ctx, owner, "midian", 1, false, 1, 3, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
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
        "yuelu" => u05_exec(ctx, owner, "yuelu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 1, U05Extra::None),
        // character/hearth/skill.js:2716-2796
        "yushou" => u05_exec(ctx, owner, "yushou", 0, false, 1, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/hearth/skill.js:2813-2834
        "yushou_misha" => u05_exec(ctx, owner, "yushou_misha", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:2835-2853
        "yushou_huofu" => u05_exec(ctx, owner, "yushou_huofu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("juedou")),
        // character/hearth/skill.js:2862-2923
        "qingzun" => u05_exec(ctx, owner, "qingzun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qingzun", 1)),
        // character/hearth/skill.js:2924-2956
        "ayuling" => u05_exec(ctx, owner, "ayuling", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2957-2978
        "aoshu" => u05_exec(ctx, owner, "aoshu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("wuzhong")),
        // character/hearth/skill.js:2979-2995
        "bzhuiji" => u05_exec(ctx, owner, "bzhuiji", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3187-3224
        "shouji" => u05_exec(ctx, owner, "shouji", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3370-3380
        "shengteng" => u05_exec(ctx, owner, "shengteng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:3401-3428
        "chouhuo" => u05_exec(ctx, owner, "chouhuo", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 2, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:3429-3512
        "nuyan2" => u05_exec(ctx, owner, "nuyan2", 3, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:3603-3625
        "duxin" => u05_exec(ctx, owner, "duxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3626-3645
        "oldduxin" => u05_exec(ctx, owner, "oldduxin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3730-3771
        "mobao" => u05_exec(ctx, owner, "mobao", 1, false, 1, 0, 0, -1, DamageNature::Thunder, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3845-3858
        "xianji3" => u05_exec(ctx, owner, "xianji3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3859-3885
        "tanmi" => u05_exec(ctx, owner, "tanmi", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3886-3914
        "xueren" => u05_exec(ctx, owner, "xueren", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:3988-4001
        "yiwen" => u05_exec(ctx, owner, "yiwen", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4058-4087
        "zhuizong" => u05_exec(ctx, owner, "zhuizong", 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4153-4161
        "hsbaowu_cangbaotu" => u05_exec(ctx, owner, "hsbaowu_cangbaotu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4162-4176
        "hsbaowu_huangjinyuanhou" => u05_exec(ctx, owner, "hsbaowu_huangjinyuanhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hsbaowu_huangjinyuanhou", 1)),
        // character/hearth/skill.js:4185-4208
        "xieneng" => u05_exec(ctx, owner, "xieneng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4209-4262
        "fbeifa" => u05_exec(ctx, owner, "fbeifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4263-4308
        "oldfbeifa" => u05_exec(ctx, owner, "oldfbeifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4309-4334
        "yufa" => u05_exec(ctx, owner, "yufa", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4396-4441
        "shifa" => u05_exec(ctx, owner, "shifa", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4442-4486
        "oldshifa" => u05_exec(ctx, owner, "oldshifa", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4487-4496
        "yuanzheng" => u05_exec(ctx, owner, "yuanzheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4497-4535
        "yulu" => u05_exec(ctx, owner, "yulu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4536-4574
        "oldyulu" => u05_exec(ctx, owner, "oldyulu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4601-4614
        "duzhang2" => u05_exec(ctx, owner, "duzhang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:duzhang2", 1)),
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
        "hhudun" => u05_exec(ctx, owner, "hhudun", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:4829-4876
        "xiaorong" => u05_exec(ctx, owner, "xiaorong", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:xiaorong", 1)),
        // character/hearth/skill.js:4916-4930
        "jixuan" => u05_exec(ctx, owner, "jixuan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5084-5091
        "bingdong" => u05_exec(ctx, owner, "bingdong", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5146-5182
        "fushi" => u05_exec(ctx, owner, "fushi", 1, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseMaxHpOther(1)),
        // character/hearth/skill.js:5183-5218
        "oldfushi" => u05_exec(ctx, owner, "oldfushi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseMaxHpOther(1)),
        // character/hearth/skill.js:5219-5229
        "moyao" => u05_exec(ctx, owner, "moyao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:moyao", 1)),
        // character/hearth/skill.js:5258-5270
        "bimeng" => u05_exec(ctx, owner, "bimeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5271-5284
        "zhoujiang" => u05_exec(ctx, owner, "zhoujiang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5312-5321
        "xjumo" => u05_exec(ctx, owner, "xjumo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:xjumo", 1)),
        // character/hearth/skill.js:5322-5345
        "malymowang" => u05_exec(ctx, owner, "malymowang", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5346-5368
        "oldmalymowang" => u05_exec(ctx, owner, "oldmalymowang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5403-5420
        "mieshi" => u05_exec(ctx, owner, "mieshi", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5444-5459
        "xshixin" => u05_exec(ctx, owner, "xshixin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5460-5513
        "enze" => u05_exec(ctx, owner, "enze", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:5514-5561
        "oldenze" => u05_exec(ctx, owner, "oldenze", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:5562-5623
        "chongsheng" => u05_exec(ctx, owner, "chongsheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
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
        "hanshuang" => u05_exec(ctx, owner, "hanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/hearth/skill.js:5720-5742
        "oldhanshuang" => u05_exec(ctx, owner, "oldhanshuang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5747-5767
        "bingshi2" => u05_exec(ctx, owner, "bingshi2", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5768-5799
        "huanwu" => u05_exec(ctx, owner, "huanwu", 1, false, 0, 0, 2, 0, DamageNature::Normal, 1, 0, U05Extra::GainMaxHp(1)),
        // character/hearth/skill.js:5800-5824
        "fengnu" => u05_exec(ctx, owner, "fengnu", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:fengnu", 1)),
        // character/hearth/skill.js:5825-5835
        "shengdun" => u05_exec(ctx, owner, "shengdun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:5836-5854
        "jingmeng" => u05_exec(ctx, owner, "jingmeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5855-5867
        "hswuji" => {
            let used = ctx.core.turn.cards_used_play_phase;
            if used > 0 && guard_phase_auto_trigger(ctx, owner, "u05", "hswuji", false) {
                u05_mark_invoked(ctx, owner, "hswuji");
                ctx.start_draw(owner, used);
            }
        }
        // character/hearth/skill.js:5868-5890
        "bingshuang" => u05_exec(ctx, owner, "bingshuang", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5891-5909
        "yanshu" => u05_exec(ctx, owner, "yanshu", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5910-5927
        "oldyanshu" => u05_exec(ctx, owner, "oldyanshu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5928-5944
        "shengyan" => u05_exec(ctx, owner, "shengyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:5946-5963
        "liechao" => u05_exec(ctx, owner, "liechao", 1, false, 0, 4, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/hearth/skill.js:5984-6016
        "zhongjia" => u05_exec(ctx, owner, "zhongjia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:6017-6047
        "dunji" => u05_exec(ctx, owner, "dunji", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6048-6071
        "fengxing" => u05_exec(ctx, owner, "fengxing", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6073-6103
        "fengxian" => u05_exec(ctx, owner, "fengxian", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:6120-6140
        "zhanhou" => u05_exec(ctx, owner, "zhanhou", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:6141-6163
        "oldzhanhou" => u05_exec(ctx, owner, "oldzhanhou", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:6164-6193
        "shijie" => u05_exec(ctx, owner, "shijie", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6194-6240
        "shengguang" => u05_exec(ctx, owner, "shengguang", 1, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpOther(1)),
        // character/hearth/skill.js:6241-6268
        "xinci" => u05_exec(ctx, owner, "xinci", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpOther(1)),
        // character/hearth/skill.js:6269-6303
        "anying" => u05_exec(ctx, owner, "anying", 0, true, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6304-6423
        "huanjue" => u05_exec(ctx, owner, "huanjue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6424-6448
        "bingjia" => u05_exec(ctx, owner, "bingjia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:bingjia", 1)),
        // character/hearth/skill.js:6449-6482
        "bingjia2" => u05_exec(ctx, owner, "bingjia2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_buff:bingjia2", 1)),
        // character/hearth/skill.js:6592-6636
        "xianzhi" => u05_exec(ctx, owner, "xianzhi", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:xianzhi")),
        // character/hearth/skill.js:6703-6722
        "wlianji" => u05_exec(ctx, owner, "wlianji", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6779-6794
        "mengun2" => u05_exec(ctx, owner, "mengun2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mengun2", 1)),
        // character/hearth/skill.js:6795-6821
        "jianren" => u05_exec(ctx, owner, "jianren", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:7031-7058
        "zuling" => u05_exec(ctx, owner, "zuling", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:7104-7110
        "tuteng_h" => u05_exec(ctx, owner, "tuteng_h", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuteng_h", 1)),
        // character/hearth/skill.js:7137-7151
        "tuteng1" => u05_exec(ctx, owner, "tuteng1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:7152-7163
        "tuteng2" => u05_exec(ctx, owner, "tuteng2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:7196-7207
        "tuteng5" => u05_exec(ctx, owner, "tuteng5", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:7224-7250
        "tuteng7" => u05_exec(ctx, owner, "tuteng7", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:7251-7262
        "tuteng8" => u05_exec(ctx, owner, "tuteng8", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tuteng8", 1)),
        // character/hearth/skill.js:7263-7275
        "hongxi" => u05_exec(ctx, owner, "hongxi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:133-154
        "buwendingyibian_lose" => u05_exec(ctx, owner, "buwendingyibian_lose", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:155-207
        "hualing" => u05_exec(ctx, owner, "hualing", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:208-235
        "muyin" => u05_exec(ctx, owner, "muyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:yunchuizhen", 1)),
        // character/hearth/skill.js:236-272
        "tqchuanyue" => u05_exec(ctx, owner, "tqchuanyue", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:273-366
        "oldhuanjue" => u05_exec(ctx, owner, "oldhuanjue", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:367-464
        "hshuanling" => u05_exec(ctx, owner, "hshuanling", 1, true, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:465-505
        "hsxingyi" => u05_exec(ctx, owner, "hsxingyi", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1392-1413
        "zhaochao" => u05_exec(ctx, owner, "zhaochao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/hearth/skill.js:1414-1425
        "zhaochao2" => u05_exec(ctx, owner, "zhaochao2", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:1592-1628
        "azaowu" => u05_exec(ctx, owner, "azaowu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/hearth/skill.js:1705-1711
        "shouwang2" => u05_exec(ctx, owner, "shouwang2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_debuff:shouwang", 1)),
        // character/hearth/skill.js:1712-1727
        "qingtian" => u05_exec(ctx, owner, "qingtian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_debuff:shouwang", 1)),
        // character/hearth/skill.js:1728-1790
        "qianfu" => u05_exec(ctx, owner, "qianfu", 0, false, 3, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:qianfu2", 1)),
        // character/hearth/skill.js:1893-1963
        "szbianshen" => u05_exec(ctx, owner, "szbianshen", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 1, U05Extra::None),
        // character/hearth/skill.js:1964-2002
        "jinhua" => u05_exec(ctx, owner, "jinhua", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2210-2266
        "mengye2" => u05_exec(ctx, owner, "mengye2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:2267-2305
        "kuixin" => u05_exec(ctx, owner, "kuixin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/hearth/skill.js:2306-2448
        "fuhua" | "fuhua2" | "fuhua3" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("juedou")),
        // character/hearth/skill.js:2449-2566
        "qianhou" => u05_exec(ctx, owner, "qianhou", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2567-2610
        "jinzhou" => u05_exec(ctx, owner, "jinzhou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:guijiang2", 1)),
        // character/hearth/skill.js:2797-2853
        "yushou_lose" => u05_exec(ctx, owner, "yushou_lose", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:2854-2868
        "yushou_leiouke" => u05_exec(ctx, owner, "yushou_leiouke", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/hearth/skill.js:2996-3080
        "lianjin" => u05_exec(ctx, owner, "lianjin", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3225-3264
        "yingxi" | "yingxi2" => u05_exec(ctx, owner, skill, 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/hearth/skill.js:3265-3305
        "hsguimou" => u05_exec(ctx, owner, "hsguimou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/hearth/skill.js:3306-3338
        "peiyu" | "peiyu2" => u05_exec(ctx, owner, skill, 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3339-3380
        "wzhanyi" => u05_exec(ctx, owner, "wzhanyi", 0, false, 3, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3381-3420
        "yuansu" => u05_exec(ctx, owner, "yuansu", 0, false, 0, 0, 0, 2, DamageNature::Thunder, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/hearth/skill.js:3513-3575
        "nuyan" => u05_exec(ctx, owner, "nuyan", 1, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::ViewAs("sha")),
        // character/hearth/skill.js:3646-3660
        "hsdusu_shinancao" => u05_exec(ctx, owner, "hsdusu_shinancao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        // character/hearth/skill.js:3661-3687
        "xiubu" => u05_exec(ctx, owner, "xiubu", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3688-3730
        "zengli" => u05_exec(ctx, owner, "zengli", 0, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3772-3794
        "mobao2" | "mobao3" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3795-3860
        "xianji" | "xianji2" => u05_exec(ctx, owner, skill, 1, false, 1, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:3915-4002
        "maoxian" => u05_exec(ctx, owner, "maoxian", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4003-4087
        "qianghuax" => u05_exec(ctx, owner, "qianghuax", 1, true, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4088-4185
        "xunbao" | "xunbao2" | "hsbaowu_huangjinyuanhou2" => u05_exec(ctx, owner, skill, 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::None),
        // character/hearth/skill.js:4335-4351
        "yufa2" | "yufa3" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:guijiang2", 1)),
        // character/hearth/skill.js:4352-4410
        "bingyan" => u05_exec(ctx, owner, "bingyan", 1, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::ViewAs("chiyuxi")),
        // character/hearth/skill.js:4575-4620
        "duzhang" => u05_exec(ctx, owner, "duzhang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:duzhang2", 1)),
        // character/hearth/skill.js:4716-4796
        "fenlie" | "oldfenlie" => u05_exec(ctx, owner, skill, 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4797-4830
        "nianfu" => u05_exec(ctx, owner, "nianfu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/hearth/skill.js:4877-4930
        "shixu" => u05_exec(ctx, owner, "shixu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:4931-4996
        "qianghua" => u05_exec(ctx, owner, "qianghua", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:4997-5038
        "biri" => u05_exec(ctx, owner, "biri", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mingkong2", 1)),
        // character/hearth/skill.js:5039-5091
        "stuxi" | "stuxi2" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_debuff:gwjinli", 1)),
        // character/hearth/skill.js:5092-5115
        "luoshi" => u05_exec(ctx, owner, "luoshi", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:5116-5150
        "ronghuo" | "ronghuo2" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5230-5284
        "jiaohui" => u05_exec(ctx, owner, "jiaohui", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5285-5320
        "liehun" => u05_exec(ctx, owner, "liehun", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5369-5420
        "lingzhou" => u05_exec(ctx, owner, "lingzhou", 0, false, 1, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5421-5460
        "xmojian" => u05_exec(ctx, owner, "xmojian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/hearth/skill.js:5743-5770
        "bingshi" => u05_exec(ctx, owner, "bingshi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:5964-5980
        "qingliu" => u05_exec(ctx, owner, "qingliu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mingkong2", 1)),
        // character/hearth/skill.js:6104-6130
        "hsmengjing_mengye" => u05_exec(ctx, owner, "hsmengjing_mengye", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/hearth/skill.js:6484-6521
        "zuzhou" => u05_exec(ctx, owner, "zuzhou", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6522-6591
        "mdzhoufu" | "mdzhoufu2" => u05_exec(ctx, owner, skill, 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::None),
        // character/hearth/skill.js:6637-6702
        "jingxiang" => u05_exec(ctx, owner, "jingxiang", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/hearth/skill.js:6723-6778
        "mengun" => u05_exec(ctx, owner, "mengun", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantOtherBuff("u05_mod:mengun2", 1)),
        // character/hearth/skill.js:6822-6842
        "jihuo" => u05_exec(ctx, owner, "jihuo", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ExtraTurn),
        // character/hearth/skill.js:6843-7136
        "tuteng_s" | "s_tuteng" | "tuteng" | "huanfeng" | "tuteng_lose" => u05_exec(ctx, owner, skill, 0, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::ModFlag("u05_mod:tuteng_h", 1)),
        // character/hearth/skill.js:7164-7179
        "tuteng3" => u05_exec(ctx, owner, "tuteng3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mingkong2", 1)),
        // character/hearth/skill.js:7180-7223
        "tuteng4" | "tuteng6" => u05_exec(ctx, owner, skill, 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:zhongji_dmg", 1)),
        _ => return false,
    }
    true
}
