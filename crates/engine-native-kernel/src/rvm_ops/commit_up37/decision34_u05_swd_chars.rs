//! Authoritative `U05 杀海拾遗` (`character/swd/skill.js`) host-side character skill resolvers.

use super::decision34_u05_ops::*;
use super::*;

pub(super) fn try_resolve_u05_swd_chars(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) -> bool {
    match skill {
        // character/swd/skill.js:4496-4551: duijue 对决 (limited: true, filterTarget: target.hp > 1)
        "duijue" => {
            u05_mark_invoked(ctx, owner, "duijue");
            if flag(ctx.core, owner, "duijue_used") == 1 {
                set_flag(ctx.core, owner, "duijue_blocked_limited", 1);
                return true;
            }
            let Some(t) = first_other_alive(ctx.core, owner) else {
                return true;
            };
            if flag(ctx.core, owner, "duijue_target_hp_too_low") == 1 || ctx.core.player(t).hp <= 1 {
                set_flag(ctx.core, owner, "duijue_blocked_hp", 1);
                return true;
            }
            set_flag(ctx.core, owner, "duijue_used", 1);
            set_flag(ctx.core, owner, "duijue_active", 1);
            u05_damage_other(ctx, owner, 2, DamageNature::Normal);
            ctx.start_draw(owner, 1);
        }
        // character/swd/skill.js:5109-5158: tanlin 探林 (pindian / discover or take 1 damage)
        "tanlin" => {
            u05_mark_invoked(ctx, owner, "tanlin");
            let Some(t) = first_other_alive(ctx.core, owner) else {
                return true;
            };
            if ctx.core.player(owner).hand.is_empty() || ctx.core.player(t).hand.is_empty() {
                set_flag(ctx.core, owner, "tanlin_blocked_empty", 1);
                return true;
            }
            let step = flag(ctx.core, owner, "tanlin_step") + 1;
            set_flag(ctx.core, owner, "tanlin_step", step);
            if flag(ctx.core, owner, "tanlin_force_lose") == 1 {
                set_flag(ctx.core, owner, "tanlin_lost_damage", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(Some(t), owner, 1)));
                return true;
            }
            ctx.start_draw(owner, 1);
            if step >= 2 {
                set_flag(ctx.core, owner, "tanlin_discovered", 1);
                u05_recover(ctx, owner, 1);
            }
        }
        // character/swd/skill.js:5-60: usable 2, steals 1 card from target / top deck into `cyqiaoxie` pile
        "cyqiaoxie" => {
            if !guard_phase_auto_trigger(ctx, owner, "u05", "cyqiaoxie", false) {
                return true;
            }
            u05_mark_invoked(ctx, owner, "cyqiaoxie");
            if u05_check_usable(ctx, owner, "cyqiaoxie", 2) {
                let _ = u05_prey_other(ctx, owner, true);
                let _ = u05_store_pile(ctx, owner, "cyqiaoxie");
                set_flag(ctx.core, owner, "u05_state:cyqiaoxie", 1);
            }
        }
        // character/swd/skill.js:61-104: consumes 1 card from `cyqiaoxie` pile so owner & target draw 1
        "cyxianjiang" => {
            if !guard_phase_auto_trigger(ctx, owner, "u05", "cyxianjiang", false) {
                return true;
            }
            u05_mark_invoked(ctx, owner, "cyxianjiang");
            if let Some(c) = ctx
                .core
                .vm_aux
                .piles
                .get_mut(&(owner, "cyqiaoxie".to_string()))
                .and_then(Vec::pop)
            {
                super::commit::discard_some(ctx, owner, vec![c]);
                ctx.start_draw(owner, 1);
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    ctx.start_draw(t, 1);
                }
            }
            set_flag(ctx.core, owner, "u05_buff:cyxianjiang", 1);
        }
        // character/swd/skill.js:105-244: usable 1, discard 1 card -> thunder damage / recover + draw 1
        "gxianyin" => u05_exec(ctx, owner, "gxianyin", 1, false, 1, 1, 0, 1, DamageNature::Thunder, 1, 0, U05Extra::GrantBuff("u05_state:gxianyin", 1)),
        // character/swd/skill.js:245-260
        "yeying" => u05_exec(ctx, owner, "yeying", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("qiankunbiao")),
        // character/swd/skill.js:261-340
        "juxi" => u05_exec(ctx, owner, "juxi", 1, false, 0, 0, 0, 1, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:juxi", 1)),
        // character/swd/skill.js:341-397
        "jiefen" => u05_exec(ctx, owner, "jiefen", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jiefen", 1)),
        // character/swd/skill.js:398-418
        "datong" => u05_exec(ctx, owner, "datong", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:datong", 1)),
        // character/swd/skill.js:419-472
        "huodan" => u05_exec(ctx, owner, "huodan", 1, false, 1, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/swd/skill.js:597-671
        "zhanxing" => u05_exec(ctx, owner, "zhanxing", 1, false, 1, 2, 0, -1, DamageNature::Thunder, 1, 1, U05Extra::GrantBuff("u05_state:zhanxing", 1)),
        // character/swd/skill.js:683-719
        "kbolan2" => u05_exec(ctx, owner, "kbolan2", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kbolan2", 1)),
        // character/swd/skill.js:720-741
        "hujing" => u05_exec(ctx, owner, "hujing", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:hujing", 1)),
        // character/swd/skill.js:812-846
        "lingshi" => u05_exec(ctx, owner, "lingshi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:lingshi", 1)),
        // character/swd/skill.js:847-877
        "tiebi" => u05_exec(ctx, owner, "tiebi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:tiebi", 1)),
        // character/swd/skill.js:878-929
        "shenyan" => u05_exec(ctx, owner, "shenyan", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:shenyan")),
        // character/swd/skill.js:930-946
        "shenyan2" => u05_exec(ctx, owner, "shenyan2", 0, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_buff:shenyan2", 1)),
        // character/swd/skill.js:947-969
        "senluo" => u05_exec(ctx, owner, "senluo", 1, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:970-1112
        "xuanying" => u05_exec(ctx, owner, "xuanying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LinkOther),
        // character/swd/skill.js:1113-1148
        "hwendao" => u05_exec(ctx, owner, "hwendao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/swd/skill.js:1149-1185
        "lingfeng" => u05_exec(ctx, owner, "lingfeng", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:lingfeng", 1)),
        // character/swd/skill.js:1393-1432
        "hlingbo" => u05_exec(ctx, owner, "hlingbo", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:hlingbo", 1)),
        // character/swd/skill.js:1433-1479
        "gtiandao" => u05_exec(ctx, owner, "gtiandao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:gtiandao")),
        // character/swd/skill.js:1480-1542
        "jinlin" => u05_exec(ctx, owner, "jinlin", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 3, U05Extra::GrantOtherBuff("u05_debuff:jinlin", 1)),
        // character/swd/skill.js:1543-1559
        "jinlin2" => u05_exec(ctx, owner, "jinlin2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, -1, U05Extra::GrantBuff("u05_buff:jinlin2", 1)),
        // character/swd/skill.js:1560-1572
        "lingyue" => u05_exec(ctx, owner, "lingyue", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:1573-1590
        "fengze" => u05_exec(ctx, owner, "fengze", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("taoyuan")),
        // character/swd/skill.js:1591-1664
        "huanxia" => u05_exec(ctx, owner, "huanxia", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:huanxia")),
        // character/swd/skill.js:1665-1675
        "huajing" => u05_exec(ctx, owner, "huajing", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:huajing", 1)),
        // character/swd/skill.js:1703-1806
        "bingfeng" => u05_exec(ctx, owner, "bingfeng", 0, true, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseMaxHp(1)),
        // character/swd/skill.js:1829-1891
        "yudun" => u05_exec(ctx, owner, "yudun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/swd/skill.js:1892-1986
        "guozao" => u05_exec(ctx, owner, "guozao", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:guozao", 1)),
        // character/swd/skill.js:1987-2030
        "heihuo" => u05_exec(ctx, owner, "heihuo", 0, false, 0, 0, 0, 3, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_buff:heihuo", 1)),
        // character/swd/skill.js:2045-2060
        "yaotong1" => u05_exec(ctx, owner, "yaotong1", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/swd/skill.js:2061-2076
        "yaotong2" => u05_exec(ctx, owner, "yaotong2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("shan")),
        // character/swd/skill.js:2077-2099
        "yaotong3" => u05_exec(ctx, owner, "yaotong3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("wuxie")),
        // character/swd/skill.js:2100-2122
        "yaotong4" => u05_exec(ctx, owner, "yaotong4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("tao")),
        // character/swd/skill.js:2123-2156
        "pojian" => u05_exec(ctx, owner, "pojian", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:pojian", 1)),
        // character/swd/skill.js:2157-2191
        "huajin" => u05_exec(ctx, owner, "huajin", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:huajin", 1)),
        // character/swd/skill.js:2208-2242
        "yuchen" => u05_exec(ctx, owner, "yuchen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:2243-2276
        "bingjian" => u05_exec(ctx, owner, "bingjian", 1, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:2385-2450
        "xianghui" => u05_exec(ctx, owner, "xianghui", 1, false, 1, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xianghui", 1)),
        // character/swd/skill.js:2504-2541
        "shangxi" => u05_exec(ctx, owner, "shangxi", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:shangxi", 1)),
        // character/swd/skill.js:2542-2582
        "fuyan" => u05_exec(ctx, owner, "fuyan", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:fuyan", 1)),
        // character/swd/skill.js:2611-2619
        "guaili2" => { u05_mark_invoked(ctx, owner, "guaili2"); let _ = u05_discard_up_to(ctx, owner, 2); }
        // character/swd/skill.js:2620-2670
        "xingzhui" => u05_exec(ctx, owner, "xingzhui", 1, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:2776-2844
        "shouyin" => u05_exec(ctx, owner, "shouyin", 0, true, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/swd/skill.js:2845-2853
        "sliufeng" => u05_exec(ctx, owner, "sliufeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:sliufeng", 1)),
        // character/swd/skill.js:2854-2873
        "linyun" => u05_exec(ctx, owner, "linyun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        // character/swd/skill.js:2901-2912
        "linyun3" => u05_exec(ctx, owner, "linyun3", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:linyun3", 1)),
        // character/swd/skill.js:2914-2976
        "hutian" => u05_exec(ctx, owner, "hutian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantOtherBuff("u05_debuff:hutian", 1)),
        // character/swd/skill.js:3133-3145
        "guiying" => u05_exec(ctx, owner, "guiying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("toulianghuanzhu")),
        // character/swd/skill.js:3175-3253
        "ningxian" => u05_exec(ctx, owner, "ningxian", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:ningxian", 1)),
        // character/swd/skill.js:3254-3288
        "xuanyuan" => u05_exec(ctx, owner, "xuanyuan", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xuanyuan", 1)),
        // character/swd/skill.js:3308-3363
        "jilve" => u05_exec(ctx, owner, "jilve", 3, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jilve", 1)),
        // character/swd/skill.js:3483-3513
        "duanyue" => u05_exec(ctx, owner, "duanyue", 1, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:duanyue", 1)),
        // character/swd/skill.js:3536-3551
        "mojian" => u05_exec(ctx, owner, "mojian", 0, false, 0, 0, 1, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:mojian", 1)),
        // character/swd/skill.js:3552-3561
        "swdliuhong" => u05_exec(ctx, owner, "swdliuhong", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:swdliuhong", 1)),
        // character/swd/skill.js:3598-3652
        "jianji" => u05_exec(ctx, owner, "jianji", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jianji", 1)),
        // character/swd/skill.js:3653-3674
        "huangyu" => u05_exec(ctx, owner, "huangyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("chiyuxi")),
        // character/swd/skill.js:3786-3824
        "duishi" => u05_exec(ctx, owner, "duishi", 0, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StealOther),
        // character/swd/skill.js:3875-3898
        "mufeng" => u05_exec(ctx, owner, "mufeng", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:mufeng", 1)),
        // character/swd/skill.js:3924-3962
        "touxi" => u05_exec(ctx, owner, "touxi", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:3971-4022
        "nlianji" => u05_exec(ctx, owner, "nlianji", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::PindianOther),
        // character/swd/skill.js:4034-4046
        "lianji4" => u05_exec(ctx, owner, "lianji4", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_buff:lianji4", 1)),
        // character/swd/skill.js:4064-4073
        "miedao2" => {
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as usize;
            if lost > 0 {
                u05_mark_invoked(ctx, owner, "miedao2");
                let _ = u05_discard_up_to(ctx, owner, lost);
            }
        }
        // character/swd/skill.js:4074-4131
        "aojian" => u05_exec(ctx, owner, "aojian", 1, false, 1, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:aojian", 1)),
        // character/swd/skill.js:4132-4192
        "moyan" => u05_exec(ctx, owner, "moyan", 1, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:moyan", 1)),
        // character/swd/skill.js:4193-4227
        "wangchen" => u05_exec(ctx, owner, "wangchen", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverOther),
        // character/swd/skill.js:4294-4319
        "youyin" => u05_exec(ctx, owner, "youyin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:youyin", 1)),
        // character/swd/skill.js:4320-4352
        "anlianying" => u05_exec(ctx, owner, "anlianying", 0, false, 0, 2, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:anlianying", 1)),
        // character/swd/skill.js:4353-4389
        "zhanlu" => u05_exec(ctx, owner, "zhanlu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:zhanlu", 1)),
        // character/swd/skill.js:4390-4411
        "huopu" => u05_exec(ctx, owner, "huopu", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("liuxinghuoyu")),
        // character/swd/skill.js:4412-4426
        "rexue" => u05_exec(ctx, owner, "rexue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:rexue", 1)),
        // character/swd/skill.js:4427-4440
        "shengshou" => u05_exec(ctx, owner, "shengshou", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("caoyao")),
        // character/swd/skill.js:4441-4463
        "huanjian" => u05_exec(ctx, owner, "huanjian", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("bingpotong")),
        // character/swd/skill.js:4464-4474
        "moyu" => u05_exec(ctx, owner, "moyu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:moyu", 1)),
        // character/swd/skill.js:4535-4633
        "kunlunjing1" => u05_exec(ctx, owner, "kunlunjing1", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_buff:kunlunjing1", 1)),
        // character/swd/skill.js:4634-4675
        "kunlunjing2" => u05_exec(ctx, owner, "kunlunjing2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::StorePile("u05_pile:kunlunjing2")),
        // character/swd/skill.js:4676-4722
        "oldliaoyuan" => u05_exec(ctx, owner, "oldliaoyuan", 1, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:4768-4855
        "shehun" => u05_exec(ctx, owner, "shehun", 1, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:4936-4981
        "dunxing" => u05_exec(ctx, owner, "dunxing", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:dunxing")),
        // character/swd/skill.js:5159-5185
        "guanhu" => u05_exec(ctx, owner, "guanhu", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:5196-5239
        "huanhun" => u05_exec(ctx, owner, "huanhun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::JudgeEffect("u05_judge:huanhun")),
        // character/swd/skill.js:5240-5260
        "yinyue" => u05_exec(ctx, owner, "yinyue", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:yinyue", 1)),
        // character/swd/skill.js:5361-5409
        "lqingcheng" => u05_exec(ctx, owner, "lqingcheng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:lqingcheng")),
        // character/swd/skill.js:5456-5568
        "shengong" => u05_exec(ctx, owner, "shengong", 0, false, 0, 0, 1, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:5601-5617
        "polang" => u05_exec(ctx, owner, "polang", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:5660-5704
        "xielei" => u05_exec(ctx, owner, "xielei", 0, false, 0, 0, 0, 1, DamageNature::Thunder, 0, 0, U05Extra::GrantBuff("u05_state:xielei", 1)),
        // character/swd/skill.js:5705-5765
        "pozhen" => u05_exec(ctx, owner, "pozhen", 0, false, 0, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:pozhen", 1)),
        // character/swd/skill.js:5766-5802
        "tanlin_defence" => u05_exec(ctx, owner, "tanlin_defence", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:tanlin_defence", 1)),
        // character/swd/skill.js:5860-5874
        "tanlin3" => u05_exec(ctx, owner, "tanlin3", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tanlin3", 1)),
        // character/swd/skill.js:5875-5883
        "tanlin4" => u05_exec(ctx, owner, "tanlin4", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:tanlin4", 1)),
        // character/swd/skill.js:5884-5924
        "yunchou" => u05_exec(ctx, owner, "yunchou", 1, false, 1, 1, 1, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:5925-5949
        "jqimou" => u05_exec(ctx, owner, "jqimou", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:jqimou", 1)),
        // character/swd/skill.js:6281-6326
        "xingdian" => u05_exec(ctx, owner, "xingdian", 1, false, 1, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:xingdian", 1)),
        // character/swd/skill.js:6356-6397
        "funiao" => u05_exec(ctx, owner, "funiao", 1, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::GrantBuff("u05_state:funiao", 1)),
        // character/swd/skill.js:6504-6562
        "ningshuang" => u05_exec(ctx, owner, "ningshuang", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/swd/skill.js:6563-6618
        "xielv" => u05_exec(ctx, owner, "xielv", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:xielv", 1)),
        // character/swd/skill.js:6619-6655
        "xiaomoyu" => u05_exec(ctx, owner, "xiaomoyu", 0, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:xiaomoyu", 1)),
        // character/swd/skill.js:6657-6700
        "tianhuo" => u05_exec(ctx, owner, "tianhuo", 1, false, 0, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:6701-6741
        "huanyin" => u05_exec(ctx, owner, "huanyin", 0, false, 0, 1, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:huanyin")),
        // character/swd/skill.js:6815-6875
        "tianlun" => u05_exec(ctx, owner, "tianlun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::JudgeEffect("u05_judge:tianlun")),
        // character/swd/skill.js:6876-6925
        "lanzhi" => u05_exec(ctx, owner, "lanzhi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:lanzhi", 1)),
        // character/swd/skill.js:6926-6956
        "duanyi" => u05_exec(ctx, owner, "duanyi", 1, false, 2, 0, 0, 1, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:7023-7031
        "miles_xueyi" => u05_exec(ctx, owner, "miles_xueyi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/swd/skill.js:7032-7071
        "jifeng" => u05_exec(ctx, owner, "jifeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:jifeng", 1)),
        // character/swd/skill.js:7072-7118
        "swd_xiuluo" => u05_exec(ctx, owner, "swd_xiuluo", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:7119-7159
        "mohua2" => u05_exec(ctx, owner, "mohua2", 0, false, 0, 2, 0, 0, DamageNature::Normal, 2, 0, U05Extra::GrantBuff("u05_buff:mohua2", 1)),
        // character/swd/skill.js:7287-7315
        "duijue2" => u05_exec(ctx, owner, "duijue2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:duijue2", 1)),
        // character/swd/skill.js:7316-7342
        "yueren" => u05_exec(ctx, owner, "yueren", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:7344-7366
        "busi" => u05_exec(ctx, owner, "busi", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::TurnOverSelf),
        // character/swd/skill.js:7367-7386
        "xuying" => u05_exec(ctx, owner, "xuying", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::LoseHpSelf(1)),
        // character/swd/skill.js:7490-7499
        "guiyan2" => u05_exec(ctx, owner, "guiyan2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:guiyan2", 1)),
        // character/swd/skill.js:7500-7555
        "xianyin" => u05_exec(ctx, owner, "xianyin", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::DiscardOther(1)),
        // character/swd/skill.js:7556-7681
        "mailun" => u05_exec(ctx, owner, "mailun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::LoseHpSelf(1)),
        // character/swd/skill.js:7714-7726
        "mailun5" => u05_exec(ctx, owner, "mailun5", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mailun5", 1)),
        // character/swd/skill.js:7727-7739
        "mailun6" => u05_exec(ctx, owner, "mailun6", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mailun6", 1)),
        // character/swd/skill.js:7740-7749
        "mailun7" => u05_exec(ctx, owner, "mailun7", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mailun7", 1)),
        // character/swd/skill.js:7750-7759
        "mailun8" => u05_exec(ctx, owner, "mailun8", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:mailun8", 1)),
        // character/swd/skill.js:7760-7785
        "fengming" => u05_exec(ctx, owner, "fengming", 1, false, 1, 0, 1, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_state:fengming", 1)),
        // character/swd/skill.js:7786-7811
        "wanjun" => u05_exec(ctx, owner, "wanjun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("nanman")),
        // character/swd/skill.js:7812-7858
        "huanling" => u05_exec(ctx, owner, "huanling", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::TurnOverSelf),
        // character/swd/skill.js:7869-7880
        "ljifeng" => u05_exec(ctx, owner, "ljifeng", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ModFlag("u05_mod:ljifeng", 1)),
        // character/swd/skill.js:7942-7955
        "zhenjiu2" => u05_exec(ctx, owner, "zhenjiu2", 0, false, 0, 0, 0, 0, DamageNature::Normal, 1, 0, U05Extra::GrantBuff("u05_buff:zhenjiu2", 1)),
        // character/swd/skill.js:7988-7998
        "hudun" => u05_exec(ctx, owner, "hudun", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 1, U05Extra::GrantBuff("u05_state:hudun", 1)),
        // character/swd/skill.js:7999-8040
        "toudan" => u05_exec(ctx, owner, "toudan", 1, false, 1, 0, 0, 1, DamageNature::Fire, 0, 0, U05Extra::GrantBuff("u05_state:toudan", 1)),
        // character/swd/skill.js:8041-8106
        "shending" => u05_exec(ctx, owner, "shending", 1, false, 0, 1, 0, 0, DamageNature::Normal, 1, 0, U05Extra::JudgeEffect("u05_judge:shending")),
        // character/swd/skill.js:8107-8155
        "poxiao" => u05_exec(ctx, owner, "poxiao", 0, false, 0, 0, 0, 0, DamageNature::Normal, 0, 0, U05Extra::ViewAs("sha")),
        _ => return false,
    }
    true
}
